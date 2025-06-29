extern crate proc_macro;
use fslock::LockFile;
use proc_macro::TokenStream;
use quote::quote;
use std::process::Command;
use std::{fs, path::Path};
use syn::{File, Item, parse_macro_input};
mod preamble;
use std::collections::HashMap;

const LOCK_PATH: &str = "rocm_attr.lock";

#[proc_macro]
pub fn amdgpu_kernel_begin(_item: TokenStream) -> TokenStream {
    let kernel_dir = Path::new("kernel_sources").join("kernel");
    let src_path = kernel_dir.join("src/lib.rs");
    let store_path = kernel_dir.join("items.json");

    let _ = fs::remove_file(&src_path);
    let _ = fs::remove_file(&store_path);

    create_kernel_structure("kernel");
    
    preamble::dummy_preamble().into()
}

#[proc_macro]
pub fn amdgpu_kernel_finalize(_item: TokenStream) -> TokenStream {
    let mut lockfile = LockFile::open(LOCK_PATH).unwrap();
    lockfile.lock().unwrap();

    reconstruct_kernel_lib("kernel");
    let binary_path = build("kernel");

    lockfile.unlock().unwrap();

    quote! {
        const AMDGPU_KERNEL_BINARY_PATH: &str = #binary_path;
    }
    .into()
}

#[proc_macro_attribute]
pub fn amdgpu_kernel_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let cloned = item.clone();
    let item_parsed = parse_macro_input!(cloned as Item);

    let normalized = match item_parsed.clone() {
        Item::Fn(mut func) => {
            func.attrs.push(syn::parse_quote!(#[unsafe(no_mangle)]));
            func.vis = syn::parse_quote!(pub);
            func.sig.abi = Some(syn::parse_quote!(extern "gpu-kernel"));
            quote!(#func).to_string()
        }
        _ => quote!(#item_parsed).to_string(),
    };

    let mut lockfile = LockFile::open(LOCK_PATH).unwrap();
    lockfile.lock().unwrap();

    let identifier = get_item_identifier(&item_parsed);

    store_kernel_item("kernel", &identifier, &normalized);

    lockfile.unlock().unwrap();

    item
}

fn get_item_identifier(item: &Item) -> String {
    match item {
        Item::Fn(f) => format!("fn {}", f.sig.ident),
        Item::Struct(s) => format!("struct {}", s.ident),
        Item::Impl(i) => {
            let ty_str = match i.self_ty.as_ref() {
                syn::Type::Path(type_path) => {
                    quote!(#type_path.path).to_string()
                }
                _ => "impl_unknown".into(),
            };
            format!("impl {}", ty_str.trim())
        }
        Item::Enum(e) => format!("enum {}", e.ident),
        Item::Trait(t) => format!("trait {}", t.ident),
        _ => "unknown".to_string(),
    }
}

fn create_kernel_structure(name: &str) {
    let kernel_dir = Path::new("kernel_sources").join(name);
    fs::create_dir_all(kernel_dir.join("src")).unwrap();
    fs::create_dir_all(kernel_dir.join(".cargo")).unwrap();
    fs::write(
        kernel_dir.join(".cargo/config.toml"),
        include_str!("cargo_config_template.toml"),
    )
    .unwrap();

    fs::write(
        kernel_dir.join("Cargo.toml"),
        include_str!("cargo_template.toml"),
    )
    .unwrap();
}

fn store_kernel_item(name: &str, id: &str, item: &str) {
    let kernel_dir = Path::new("kernel_sources").join(name);
    let store_path = kernel_dir.join("items.json");

    let mut items: HashMap<String, String> = if store_path.exists() {
        serde_json::from_str(&fs::read_to_string(&store_path).unwrap()).unwrap()
    } else {
        HashMap::new()
    };

    items.insert(id.to_string(), item.to_string());

    fs::write(store_path, serde_json::to_string_pretty(&items).unwrap()).unwrap();
}

fn reconstruct_kernel_lib(name: &str) {
    let kernel_dir = Path::new("kernel_sources").join(name);
    let store_path = kernel_dir.join("items.json");
    let lib_path = kernel_dir.join("src/lib.rs");

    let mut lib_code = String::new();
    lib_code.push_str(preamble::preamble());

    if store_path.exists() {
        let items: HashMap<String, String> =
            serde_json::from_str(&fs::read_to_string(store_path).unwrap()).unwrap();

        let mut keys: Vec<_> = items.keys().collect();
        keys.sort();

        for key in keys {
            lib_code.push_str(&items[key]);
            lib_code.push('\n');
        }
    }

    fs::write(lib_path, lib_code).unwrap();
}

fn build(name: &str) -> String {
    let current_dir = std::env::current_dir().unwrap();
    let kernel_dir = current_dir.join("kernel_sources").join(name);

    let status = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(&kernel_dir)
        .status()
        .expect("Failed to execute cargo build");

    if !status.success() {
        panic!("Kernel compilation failed for {}", name);
    }

    kernel_dir
        .join("target")
        .join("amdgcn-amd-amdhsa")
        .join("release")
        .join("kernels.elf")
        .display()
        .to_string()
}
