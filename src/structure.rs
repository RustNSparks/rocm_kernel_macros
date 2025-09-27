use std::{collections::HashMap, fs, path::Path};

use crate::preamble;


pub fn create_kernel_structure(name: &str) {
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

pub fn store_kernel_item(name: &str, id: &str, item: &str) {
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

pub fn reconstruct_kernel_lib(name: &str) {
    let kernel_dir = Path::new("kernel_sources").join(name);
    let store_path = kernel_dir.join("items.json");
    let lib_path = kernel_dir.join("src/lib.rs");

    let mut lib_code = String::new();
    lib_code.push_str(&preamble::preamble());
    lib_code.push_str("\n\n");

    if store_path.exists() {
        let items: HashMap<String, String> =
            serde_json::from_str(&fs::read_to_string(store_path).unwrap()).unwrap();

        let mut keys: Vec<_> = items.keys().collect();
        keys.sort();

        for key in keys {
            lib_code.push_str(&items[key]);
            lib_code.push_str("\n\n");
        }
    }

    fs::write(lib_path, lib_code).unwrap();
}