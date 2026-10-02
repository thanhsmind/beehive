use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../../../../.claude-plugin/plugin.json");
    println!("cargo:rerun-if-changed=../../../../.pi/extensions/bee-guard");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let guard_dir = manifest_dir.join("../../../../.pi/extensions/bee-guard");
    let abs_guard_dir = guard_dir.canonicalize().unwrap_or(guard_dir);

    let mut entries = Vec::new();
    if let Ok(read_dir) = fs::read_dir(&abs_guard_dir) {
        for entry in read_dir.flatten() {
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            if !file_type.is_file() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.ends_with(".ts") {
                let abs_file_path = match entry.path().canonicalize() {
                    Ok(p) => p,
                    Err(_) => entry.path(),
                };
                let abs_str = abs_file_path.to_str().unwrap().replace('\\', "/");
                entries.push((file_name, abs_str));
            }
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("pi_guard_files.rs");

    let mut code = String::new();
    code.push_str("pub(crate) const PI_GUARD_FILES: &[(&str, &str)] = &[\n");
    for (name, path) in &entries {
        code.push_str(&format!("    (\"{name}\", include_str!(\"{path}\")),\n"));
    }
    code.push_str("];\n");

    fs::write(&dest_path, code).unwrap();
}
