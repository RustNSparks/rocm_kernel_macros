extern crate proc_macro;
use fslock::LockFile;
use proc_macro::TokenStream;
use quote::quote;
use std::process::Command;
use std::{fs, path::Path};
use syn::{Item, parse_macro_input};
mod preamble;

mod structure;
use structure::*;

const LOCK_PATH: &str = "rocm_attr.lock";

/// # Functionality
/// Generates kernel_sources dir.
/// 
/// If your kernel code is split across multiple files, this macro must be placed before including them.
#[proc_macro]
pub fn amdgpu_kernel_init(_item: TokenStream) -> TokenStream {
    let kernel_dir = Path::new("kernel_sources").join("kernel");
    let src_path = kernel_dir.join("src/lib.rs");
    let store_path = kernel_dir.join("items.json");

    let _ = fs::remove_file(&src_path);
    let _ = fs::remove_file(&store_path);

    create_kernel_structure("kernel");

    preamble::dummy_preamble().into()
}

/// # Functionality
/// Compiles kernel.
///
/// # Panics
///
/// Panics if compilation error occurs.
#[proc_macro]
pub fn amdgpu_kernel_finalize(_item: TokenStream) -> TokenStream {
    let mut lockfile = LockFile::open(LOCK_PATH).unwrap();
    lockfile.lock().unwrap();

    reconstruct_kernel_lib("kernel");
    let binary_path = build("kernel");

    quote! {
        #binary_path
    }
    .into()
}

/// # Functionality
/// Copies marked scope to device. If function is marked as global it can be launched from host side.
/// 
/// For performance reasons, if function doesnt need to be called from host side you can mark it with `#[amdgpu_device]`.
///
#[proc_macro_attribute]
pub fn amdgpu_global(_attr: TokenStream, item: TokenStream) -> TokenStream {
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

    quote!(#[allow(unused)] #item_parsed).into()
}

/// # Functionality
/// Copies marked scope to device. If function is marked as device it can only be called from other device side functions.
///
#[proc_macro_attribute]
pub fn amdgpu_device(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let cloned = item.clone();
    let item_parsed = parse_macro_input!(cloned as Item);

    let normalized = quote!(#item_parsed).to_string();

    let mut lockfile = LockFile::open(LOCK_PATH).unwrap();
    lockfile.lock().unwrap();

    let identifier = get_item_identifier(&item_parsed);

    store_kernel_item("kernel", &identifier, &normalized);

    quote!(#[allow(unused)] #item_parsed).into()
}

fn get_item_identifier(item: &Item) -> String {
    match item {
        Item::Fn(f) => format!("fn {}", f.sig.ident),
        Item::Struct(s) => format!("struct {}", s.ident),
        Item::Impl(i) => {
            let ty_str = match i.self_ty.as_ref() {
                syn::Type::Path(type_path) => quote!(#type_path.path).to_string(),
                _ => "impl_unknown".into(),
            };
            format!("impl {}", ty_str.trim())
        }
        Item::Enum(e) => format!("enum {}", e.ident),
        Item::Trait(t) => format!("trait {}", t.ident),
        _ => "unknown".to_string(),
    }
}

fn build(name: &str) -> String {
    let current_dir = std::env::current_dir().unwrap();
    let kernel_dir = current_dir.join("kernel_sources").join(name);

    let command = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(&kernel_dir)
        .output()
        .expect("Failed to execute cargo build");

    let status = command.status;

    if !status.success() {
        panic!(
            "Kernel compilation failed for {}, status: {}\n\nStdout:\n{}\n\nStderr:\n{}",
            name,
            status,
            String::from_utf8_lossy(&command.stdout),
            String::from_utf8_lossy(&command.stderr)
        );
    }

    kernel_dir
        .join("target")
        .join("amdgcn-amd-amdhsa")
        .join("release")
        .join("kernels.elf")
        .display()
        .to_string()
}
