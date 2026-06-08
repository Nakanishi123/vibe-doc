use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("server crate should live under crates/");
    let dist_dir = workspace_root.join("apps/web/dist");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let generated = out_dir.join("embedded_assets.rs");

    println!("cargo:rerun-if-changed={}", dist_dir.display());

    let mut entries = Vec::new();
    if dist_dir.is_dir() {
        collect_assets(&dist_dir, &dist_dir, &mut entries);
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));

    let mut source = String::from("static EMBEDDED_ASSETS: &[EmbeddedAsset] = &[\n");
    for (relative_path, absolute_path) in entries {
        source.push_str("    EmbeddedAsset { path: ");
        source.push_str(&rust_string(&relative_path));
        source.push_str(", bytes: include_bytes!(");
        source.push_str(&rust_string(&absolute_path.to_string_lossy()));
        source.push_str(") },\n");
    }
    source.push_str("];\n");

    fs::write(generated, source).unwrap();
}

fn collect_assets(root: &Path, current: &Path, entries: &mut Vec<(String, PathBuf)>) {
    for entry in fs::read_dir(current).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_assets(root, &path, entries);
        } else if path.is_file() {
            let relative = path.strip_prefix(root).unwrap().to_string_lossy();
            entries.push((relative.replace('\\', "/"), path));
        }
    }
}

fn rust_string(value: &str) -> String {
    format!("{value:?}")
}
