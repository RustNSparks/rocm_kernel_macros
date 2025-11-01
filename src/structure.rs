use regex::Regex;
use std::{collections::HashMap, fs, path::Path};

use crate::preamble;

pub fn get_path_from_item(item: impl ToString, postamble: &str) -> String {
    let mut path_addition = String::new();

    let str_item = item.to_string();
    let trimmed_item = str_item.trim();
    if !trimmed_item.is_empty() {
        path_addition += trimmed_item;
        path_addition += "_"
    }

    path_addition + postamble
}

pub fn cleanup_kernel_structure(name: &str) {
    let kernel_dir = Path::new("kernel_sources").join(name);
    let src_path = kernel_dir.join("src/lib.rs");
    let store_path = kernel_dir.join("items.json");

    let _ = fs::remove_file(&src_path);
    let _ = fs::remove_file(&store_path);
}

pub fn create_kernel_structure(name: &str, gfx_ver: Option<String>) {
    let kernel_dir = Path::new("kernel_sources").join(name);
    fs::create_dir_all(kernel_dir.join("src")).unwrap();
    fs::create_dir_all(kernel_dir.join(".cargo")).unwrap();

    let mut cargo_config = include_str!("cargo_config_template.toml").to_owned();

    if let Some(gfx_ver) = gfx_ver {
        let regex = Regex::new(r"gfx1103").unwrap();
        cargo_config = regex.replace(&cargo_config, gfx_ver).to_string();
    }

    fs::write(kernel_dir.join(".cargo/config.toml"), cargo_config).unwrap();

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
