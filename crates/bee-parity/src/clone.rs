//! Recursive directory clone: copies a generated fixture store into a
//! fresh temp root for each leg (CONTEXT.md D7a: "clone the fixture store
//! into two temp roots"). std-only recursive copy — no `fs_extra`/`walkdir`
//! crate (rust-port-1/2 precedent: no new dependencies).

use std::fs;
use std::path::Path;

pub fn copy_tree(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("mkdir {}: {e}", dst.display()))?;
    let entries = fs::read_dir(src).map_err(|e| format!("read_dir {}: {e}", src.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read_dir entry under {}: {e}", src.display()))?;
        let file_type = entry
            .file_type()
            .map_err(|e| format!("file_type {}: {e}", entry.path().display()))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_tree(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("copy {} -> {}: {e}", src_path.display(), dst_path.display()))?;
        }
        // symlinks: none expected in a generated fixture store; skip.
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn copies_nested_files() {
        let base = std::env::temp_dir().join(format!("bee-parity-clone-test-{}", std::process::id()));
        let src = base.join("src");
        let dst = base.join("dst");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(src.join("a/b")).unwrap();
        fs::write(src.join("a/b/file.txt"), b"hello").unwrap();
        fs::write(src.join("top.txt"), b"top").unwrap();

        copy_tree(&src, &dst).unwrap();

        assert_eq!(fs::read(dst.join("a/b/file.txt")).unwrap(), b"hello");
        assert_eq!(fs::read(dst.join("top.txt")).unwrap(), b"top");

        let _ = fs::remove_dir_all(&base);
    }
}
