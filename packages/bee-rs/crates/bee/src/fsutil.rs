// fsutil — Rust port of lib/fsutil.mjs's primitives, with one strangler
// addition: readers distinguish `Corrupt` from `Missing` so a native verb can
// bail to the Node runtime instead of replicating Node's parse-error warning
// text (which embeds V8's own error messages).

use crate::jsjson;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub enum ReadJson {
    Missing,
    /// Present but unreadable/unparseable. Node's readJson warns to stderr
    /// with the V8 message and falls back — native verbs delegate instead.
    Corrupt,
    Parsed(Value),
}

pub fn read_json(file: &Path) -> ReadJson {
    let bytes = match std::fs::read(file) {
        Ok(b) => b,
        Err(_) => return ReadJson::Missing,
    };
    // Node reads utf8 (lossy for invalid sequences) then strips one BOM.
    let mut text: &str = &String::from_utf8_lossy(&bytes).into_owned();
    let owned;
    if let Some(stripped) = text.strip_prefix('\u{feff}') {
        owned = stripped.to_string();
        text = &owned;
    }
    match serde_json::from_str::<Value>(text) {
        Ok(v) => ReadJson::Parsed(v),
        Err(_) => ReadJson::Corrupt,
    }
}

pub fn ensure_dir(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)
}

static WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn to_base36(mut n: u64) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if n == 0 {
        return "0".to_string();
    }
    let mut out = Vec::new();
    while n > 0 {
        out.push(DIGITS[(n % 36) as usize]);
        n /= 36;
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

fn tmp_path_for(file: &Path) -> PathBuf {
    // Same collision-immunity shape as writeJsonAtomic's `pid-counter-random`
    // tmp naming; the random tail here derives from the monotonic clock,
    // which is unique-enough alongside pid+counter (the counter alone already
    // makes same-process collisions impossible).
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let unique = format!(
        "{}-{}-{:08x}",
        std::process::id(),
        to_base36(WRITE_COUNTER.fetch_add(1, Ordering::Relaxed)),
        nanos
    );
    let mut name = file.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{unique}.tmp"));
    file.with_file_name(name)
}

/// writeJsonAtomic: `JSON.stringify(obj, null, 2) + "\n"` to a unique tmp in
/// the same directory, then rename over `file`. On failure the tmp is removed
/// best-effort and the original error propagates.
pub fn write_json_atomic(file: &Path, value: &Value) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        ensure_dir(dir)?;
    }
    let tmp = tmp_path_for(file);
    let content = format!("{}\n", jsjson::stringify_pretty(value));
    let result = std::fs::write(&tmp, content).and_then(|()| std::fs::rename(&tmp, file));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// appendJsonl: compact stringify + "\n", appended (creating parents).
pub fn append_jsonl(file: &Path, value: &Value) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        ensure_dir(dir)?;
    }
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(file)?;
    f.write_all(format!("{}\n", jsjson::stringify(value)).as_bytes())
}

pub fn remove_file_if_exists(file: &Path) {
    let _ = std::fs::remove_file(file);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn atomic_write_matches_node_byte_shape() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("nested").join("out.json");
        write_json_atomic(&file, &json!({"hash": "h", "checked_at": "t"})).unwrap();
        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(content, "{\n  \"hash\": \"h\",\n  \"checked_at\": \"t\"\n}\n");
        // No tmp leftovers.
        let leftovers: Vec<_> = std::fs::read_dir(file.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn read_json_strips_bom_and_flags_corrupt() {
        let dir = tempfile::tempdir().unwrap();
        let bom = dir.path().join("bom.json");
        std::fs::write(&bom, "\u{feff}{\"a\":1}").unwrap();
        assert!(matches!(read_json(&bom), ReadJson::Parsed(_)));
        let bad = dir.path().join("bad.json");
        std::fs::write(&bad, "{nope").unwrap();
        assert!(matches!(read_json(&bad), ReadJson::Corrupt));
        assert!(matches!(read_json(&dir.path().join("absent.json")), ReadJson::Missing));
    }
}
