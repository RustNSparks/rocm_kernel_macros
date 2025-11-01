extern crate proc_macro;
use fslock::LockFile;
use proc_macro::TokenStream;
use quote::quote;
use std::{collections::HashMap, process::Command};
use syn::{Item, parse_macro_input};
mod preamble;

mod structure;
use structure::*;

const LOCK_PATH: &str = "rocm_attr.lock";

/// # Functionality
/// Generates kernel_sources dir.
///
/// If your kernel code is split across multiple files, this macro must be placed before including them.
/// 
/// Args:
///     path: name of the kernel -> path + "_kernel" (if empty defaults to "kernel")
///     gfx: target gfx version -> if empty defaults to gfx1103
#[proc_macro]
pub fn amdgpu_kernel_init(items: TokenStream) -> TokenStream {
    let mut lockfile = LockFile::open(LOCK_PATH).unwrap();
    lockfile.lock().unwrap();

    let (path, gfx) = parse_kernel_init_args(items);

    let path = get_path_from_item(path, "kernel");

    cleanup_kernel_structure(&path);

    create_kernel_structure(&path, gfx);

    lockfile.unlock().unwrap();

    preamble::dummy_preamble().into()
}


/// Parses arguments passed to `amdgpu_kernel_init`, returns name of kernel and optional gfx setting.
fn parse_kernel_init_args(items: TokenStream) -> (String, Option<String>) {
    let items = items
        .into_iter()
        .filter(|e| e.to_string() != ",")
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|chunk| {
            return (chunk[0].to_string(), chunk[2].to_string());})
        .fold(HashMap::new(), |mut acc, (ident, item)| {
            acc.insert(ident, item);
            acc
        });

    let path = items
        .get("path")
        .cloned()
        .get_or_insert_default()
        .to_owned();
    let gfx = items.get("gfx").cloned().map(|s| s.to_owned());

    return (path, gfx);
}

/// # Functionality
/// Compiles kernel.
///
/// # Panics
///
/// Panics if compilation error occurs.
#[proc_macro]
pub fn amdgpu_kernel_finalize(item: TokenStream) -> TokenStream {
    let mut lockfile = LockFile::open(LOCK_PATH).unwrap();
    lockfile.lock().unwrap();

    let path = get_path_from_item(item, "kernel");

    reconstruct_kernel_lib(&path);
    let binary_path = build(&path);

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
pub fn amdgpu_global(attr: TokenStream, item: TokenStream) -> TokenStream {
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

    store_kernel_item(
        &get_path_from_item(attr, "kernel"),
        &identifier,
        &normalized,
    );

    quote!(#[allow(unused)] #item_parsed).into()
}

/// # Functionality
/// Copies marked scope to device. If function is marked as device it can only be called from other device side functions.
///
#[proc_macro_attribute]
pub fn amdgpu_device(attr: TokenStream, item: TokenStream) -> TokenStream {
    let cloned = item.clone();
    let item_parsed = parse_macro_input!(cloned as Item);

    let normalized = quote!(#item_parsed).to_string();

    let mut lockfile = LockFile::open(LOCK_PATH).unwrap();
    lockfile.lock().unwrap();

    let identifier = get_item_identifier(&item_parsed);

    store_kernel_item(
        &get_path_from_item(attr, "kernel"),
        &identifier,
        &normalized,
    );

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
