// fsutil — Rust port of lib/fsutil.mjs's primitives. Readers distinguish
// `Corrupt` from `Missing` because the two mean different things to a caller:
// an absent file is a normal state, a present-but-unparseable one is a defect
// the user must hear about.
//
// CUTOVER (2026-08-01). While contract C2 bound the port to byte-identical
// output, every `Corrupt` arm returned to Node, because Node's warning
// interpolated V8's own `JSON.parse` message and no Rust string could match
// it. Node is gone, so C2 no longer binds and those arms are native. The
// warning below carries the SAME information Node's did — which file, that it
// could not be parsed, that a fallback was used instead — in our own words,
// plus the position, which V8's message buried in prose. The fail-open
// SEMANTICS are unchanged: a caller that fell back still falls back, a caller
// that refused still refuses.

use crate::jsjson;
use crate::textutil::truncate_chars_tail;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub enum ReadJson {
    Missing,
    /// Present but unreadable/unparseable. Callers warn via
    /// [`warn_corrupt_json`] and take the fallback Node's readJson would have
    /// returned.
    Corrupt,
    Parsed(Value),
}

/// The native replacement for `readJson`'s fail-open warn. Node printed
///
/// ```text
/// bee: could not parse JSON at <file> — <V8 message>. Using fallback; fix the file.
/// ```
///
/// and returned the caller's fallback. This prints the same sentence with our
/// own reason in place of the interpreter's, and the caller still falls back.
/// stderr, never stdout — stdout is reserved for `--json` output.
pub fn warn_corrupt_json(file: &Path) {
    eprintln!(
        "bee: could not parse JSON at {} — {}. Using fallback; fix the file.",
        file.display(),
        corrupt_json_reason(file)
    );
}

/// Why the file would not parse, in bee's words. Re-reads the file (it is
/// already known bad, and this runs once per corrupt file) so `ReadJson`
/// stays a payload-free enum every caller can match without churn.
fn corrupt_json_reason(file: &Path) -> String {
    let Ok(bytes) = std::fs::read(file) else {
        return "the file could not be read".to_string();
    };
    let text = String::from_utf8_lossy(&bytes);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    match serde_json::from_str::<Value>(text) {
        // Raced: it parses now. Say only what is still true.
        Ok(_) => "invalid JSON".to_string(),
        Err(e) if e.line() > 0 => {
            format!("invalid JSON at line {} column {}", e.line(), e.column())
        }
        Err(_) => "invalid JSON".to_string(),
    }
}

/// The JSONL sibling: one bad LINE inside an otherwise readable file. Node's
/// readers skipped such a line silently or warned with the V8 message
/// depending on the store; every native caller that skips says so here.
pub fn warn_corrupt_jsonl_line(file: &Path, line_no: usize) {
    eprintln!(
        "bee: could not parse JSON at {} line {} — invalid JSON. Skipping that line; fix the file.",
        file.display(),
        line_no
    );
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

/// The text sibling of [`write_json_atomic`]: no JSON encoding, no shape —
/// write the caller's exact bytes to a unique tmp in the same directory,
/// then rename over `file`. Same atomic temp-then-rename shape, same
/// failure handling: on error the tmp is removed best-effort and the
/// original error propagates.
pub fn write_text_atomic(file: &Path, content: &str) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        ensure_dir(dir)?;
    }
    let tmp = tmp_path_for(file);
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

/// The bound every failure excerpt below shares (provenance `lib/test-runner.mjs`
/// FAILURE_EXCERPT_MAX_CHARS; decision D2 of full-failure-evidence keeps it
/// unraised). Counts `char`s, not UTF-16 units (js-parity-cleanup D3).
pub(crate) const FAILURE_EXCERPT_MAX_CHARS: usize = 500;

/// The tail-of-output excerpt for one failing declared command: the last
/// [`FAILURE_EXCERPT_MAX_CHARS`] characters of `trimmed_output` (already
/// JS-trimmed by the caller — `bee test`, `bee cells finish` and `bee close`
/// each spawn their own command and trim its output before calling this), or
/// `(no output; exit N)` when nothing survives the trim.
pub(crate) fn failure_excerpt(trimmed_output: &str, exit: Option<i64>) -> String {
    let tail = truncate_chars_tail(trimmed_output, FAILURE_EXCERPT_MAX_CHARS);
    if tail.is_empty() {
        format!(
            "(no output; exit {})",
            exit.map(|e| e.to_string()).unwrap_or_else(|| "null".to_string())
        )
    } else {
        tail
    }
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
    fn write_text_atomic_writes_the_exact_bytes_given() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("nested").join("promote-proposals.md");
        write_text_atomic(&file, "# Title\n\nsome body, unquoted, unreformatted\n").unwrap();
        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(content, "# Title\n\nsome body, unquoted, unreformatted\n");
        // No tmp leftovers, same as write_json_atomic.
        let leftovers: Vec<_> = std::fs::read_dir(file.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty());

        // Overwrite: the second write replaces the first, atomically.
        write_text_atomic(&file, "replaced\n").unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "replaced\n");
    }

    // R5 port of the non-race half of scripts/tests/test_state_write_concurrency.mjs.
    // The race itself lives in tests/concurrency.rs; what is testable in
    // process is the property the race depends on — two writers never pick
    // the same tmp name, so one can never rename another's half-written file
    // into place.
    #[test]
    fn every_atomic_write_picks_a_distinct_tmp_name() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("state.json");
        let names: std::collections::BTreeSet<String> = (0..512)
            .map(|_| tmp_path_for(&file).file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names.len(), 512, "tmp names collided within one process");
        for n in &names {
            assert!(n.starts_with("state.json."), "tmp stays beside its target: {n}");
            assert!(n.ends_with(".tmp"), "tmp keeps its suffix: {n}");
        }
        // And it lands in the SAME directory — a rename across filesystems is
        // not atomic, which is the whole reason for the sibling-tmp shape.
        assert_eq!(tmp_path_for(&file).parent(), file.parent());
    }

    #[test]
    fn a_failed_atomic_write_leaves_no_tmp_and_no_partial_target() {
        // Provenance: writeJsonAtomic's failed-rename arm — the tmp is
        // unlinked best-effort and the ORIGINAL error propagates.
        let dir = tempfile::tempdir().unwrap();
        // A directory where the target file should be: the rename must fail.
        let file = dir.path().join("blocked.json");
        std::fs::create_dir(&file).unwrap();
        let err = write_json_atomic(&file, &json!({"a": 1}));
        assert!(err.is_err(), "renaming over an existing directory must fail");
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "a failed write must not leave a tmp behind");
        assert!(file.is_dir(), "the target must be untouched by the failure");
    }

    /// The native corrupt-JSON warning must carry every piece of information
    /// Node's did — the path, that it could not be PARSED, and that a fallback
    /// was used — without any V8 wording. Asserted on the reason half here;
    /// the full sentence is fixed in `warn_corrupt_json` above.
    #[test]
    fn corrupt_json_reason_names_the_position_and_no_engine() {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("bad.json");
        std::fs::write(&bad, "{\n  \"a\": ,\n}").unwrap();
        let reason = corrupt_json_reason(&bad);
        assert!(reason.starts_with("invalid JSON at line 2 column"), "{reason}");
        assert!(!reason.contains("Unexpected"), "no V8 wording: {reason}");
        assert!(!reason.contains("JSON.parse"), "no V8 wording: {reason}");

        // A file that vanished under us still yields a truthful reason.
        let gone = dir.path().join("gone.json");
        assert_eq!(corrupt_json_reason(&gone), "the file could not be read");
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
