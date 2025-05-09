extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use std::process::Command;
use std::{fs, path::Path};
use syn::{ItemFn, parse_macro_input};

#[proc_macro]
pub fn amdgpu_kernel(input: TokenStream) -> TokenStream {
    let kernel_fn = parse_macro_input!(input as ItemFn);
    let fn_name = kernel_fn.sig.ident.to_string();

    let kernel_body = quote!(#kernel_fn).to_string();

    let preamble = r#"
#![no_std]
#![feature(abi_gpu_kernel)]
#![feature(core_intrinsics, link_llvm_intrinsics)]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

unsafe extern "C" {
    #[link_name = "llvm.amdgcn.workitem.id.x"]
    pub fn workitem_id_x() -> u32;
    #[link_name = "llvm.amdgcn.workitem.id.y"]
    pub fn workitem_id_y() -> u32;
    #[link_name = "llvm.amdgcn.workitem.id.z"]
    pub fn workitem_id_z() -> u32;

    #[link_name = "llvm.amdgcn.workgroup.id.x"]
    pub fn workgroup_id_x() -> u32;
    #[link_name = "llvm.amdgcn.workgroup.id.y"]
    pub fn workgroup_id_y() -> u32;
    #[link_name = "llvm.amdgcn.workgroup.id.z"]
    pub fn workgroup_id_z() -> u32;
}
"#;

    let full_source = format!("{preamble}\n\n{kernel_body}");

    let kernel_dir = Path::new("kernel_sources").join(&fn_name);
    fs::create_dir_all(kernel_dir.join("src")).unwrap();

    let cargo_toml = format!(
        r#"
[package]
name = "kernels"
version = "0.1.0"
edition = "2024"

[dependencies]


[lib]
crate-type = ["cdylib"]

[profile.dev]
lto = true
[profile.release]
lto = true
"#
    );

    fs::write(kernel_dir.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

    let cargo_config_dir = kernel_dir.join(".cargo");
    fs::create_dir_all(&cargo_config_dir).expect("Failed to create .cargo directory");

    let config_toml = r#"
[build]
target = "amdgcn-amd-amdhsa"
rustflags = ["-Ctarget-cpu=gfx1102"]

[unstable]
build-std = ["core"]
    "#;
    fs::write(cargo_config_dir.join("config.toml"), config_toml)
        .expect("Failed to write .cargo/config.toml");

    fs::write(kernel_dir.join("src/lib.rs"), full_source).expect("Failed to write lib.rs");

    let build_command = format!(
        "cd {}/kernel_sources/{}; cargo build --release; cd ./../../;",
        std::env::current_dir().unwrap().display(),
        fn_name
    );

    let status = Command::new("sh")
        .arg("-c")
        .arg(build_command)
        .status()
        .expect("Failed to execute cargo build");

    if !status.success() {
        panic!("Kernel compilation failed for {}", fn_name);
    }

    let target_dir = Path::new("kernel_sources")
        .join(fn_name)
        .join("target")
        .join("amdgcn-amd-amdhsa")
        .join("release");
    let binary_path = target_dir.join("kernels.elf");

    let binary_path_str = binary_path.display().to_string();

    let output = quote! {
        pub const KERNEL_BINARY_PATH: &str = #binary_path_str;
    };

    output.into()
}
