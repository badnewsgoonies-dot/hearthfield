use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=status/asset-manifest.toml");

    let manifest_path = "status/asset-manifest.toml";
    let content = match fs::read_to_string(manifest_path) {
        Ok(c) => c,
        Err(e) => {
            println!("cargo:warning=Could not read {manifest_path}: {e}");
            return;
        }
    };

    let mut found = 0u32;
    let mut total = 0u32;
    let mut current_path: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        if line == "[[sprites]]" {
            if let Some(path) = current_path.take() {
                total += 1;
                if Path::new(&path).exists() {
                    found += 1;
                } else {
                    println!("cargo:warning=Missing sprite: {path}");
                }
            }
        } else if line.starts_with("path = \"") {
            let trimmed = line.trim_start_matches("path = \"");
            if let Some(p) = trimmed.strip_suffix('"') {
                current_path = Some(p.to_string());
            }
        }
    }

    // Handle the last [[sprites]] entry
    if let Some(path) = current_path.take() {
        total += 1;
        if Path::new(&path).exists() {
            found += 1;
        } else {
            println!("cargo:warning=Missing sprite: {path}");
        }
    }

    println!("cargo:warning=Asset validation: {found}/{total} sprites found");
}
