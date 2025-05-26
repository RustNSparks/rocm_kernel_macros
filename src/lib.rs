extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use std::process::Command;
use std::{fs, path::Path};
use syn::{File, Item, parse_macro_input};
mod preamble;

#[proc_macro]
pub fn amdgpu_kernel(input: TokenStream) -> TokenStream {
    let cloned = input.clone();
    let file = parse_macro_input!(cloned as File);

    let mut kernel_body = String::new();
    let mut kernel_names = Vec::new();
    for item in file.items {
        let modified_item = if let Item::Fn(mut func) = item.clone() {
            let name = func.sig.ident.to_string();
            kernel_names.push(name);

            // Add attributes
            func.attrs.push(syn::parse_quote!(#[unsafe(no_mangle)]));
            func.vis = syn::parse_quote!(pub);
            func.sig.abi = Some(syn::parse_quote!(extern "gpu-kernel"));

            quote!(#func)
        } else {
            quote!(#item)
        };

        // Append the code of all items to be written into lib.rs
        kernel_body += &modified_item.to_string();
        kernel_body += "\n\n";
    }

    let full_source = format!("{}\n\n{}", preamble::preamble(), kernel_body);

    let kernel_dir_name = kernel_names.join("_");

    generate_enviorment(&kernel_dir_name, &full_source);

    let binary_path_str = build(&kernel_dir_name);

        let extra = quote! {
        pub const KERNEL_BINARY_PATH: &str = #binary_path_str;
    };

    let mut output = TokenStream::new();
    output.extend([preamble::dummy_preamble(), input, extra.into()].into_iter());

    output
}

fn generate_enviorment(name: &str, src: &str) {
    // create kernel source path
    let kernel_dir = Path::new("kernel_sources").join(name);
    fs::create_dir_all(kernel_dir.join("src")).unwrap();

    // write source
    fs::write(kernel_dir.join("src/lib.rs"), src).unwrap();

    // generate and write cargo config
    let cargo_config_dir = kernel_dir.join(".cargo");
    fs::create_dir_all(&cargo_config_dir).expect("Failed to create .cargo directory");

    let config_toml = include_str!("cargo_config_template.toml");
    fs::write(cargo_config_dir.join("config.toml"), config_toml)
        .expect("Failed to write .cargo/config.toml");

    // write cargo toml
    let cargo_toml = include_str!("cargo_template.toml");
    fs::write(kernel_dir.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");
}

fn build(fn_name: &str) -> String {
    // building with cargo

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

    binary_path.display().to_string()
}
