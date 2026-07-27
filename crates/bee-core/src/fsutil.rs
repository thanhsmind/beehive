//! fsutil — filesystem primitives ported from `.bee/bin/lib/fsutil.mjs` (D3
//! storage-compat contract, rust-port-5). Mirrors that file's read/write/
//! append surface: atomic JSON writes (temp file + same-dir rename),
//! fail-open tolerant JSON reads (BOM stripping, fallback on parse error),
//! and fail-open tolerant JSONL reads (corrupt/truncated lines skipped,
//! never fail the whole read).
//!
//! `.bee/bin/lib/fsutil.mjs` is FROZEN for the duration of the rust-port
//! feature (CONTEXT.md D1) — this module is oracle-checked against it, never
//! edited to "improve" on it. `crates/bee-core/tests/fsutil_oracle.rs` runs
//! the REAL mjs file in a node child (`tests/support/mjs_oracle.mjs`) and
//! diffs results against these functions (advisor note 5) — matching the
//! mjs reader is proven that way, never by this module author's reading of
//! fsutil.mjs alone.
//!
//! One semantic departure from the cell's shorthand description, called out
//! here so it is never "fixed" back by accident: `.bee/bin/lib/fsutil.mjs`'s
//! own `appendJsonl` does NOT swallow I/O errors — it has no try/catch, and
//! every real caller in this repo (`decisions.mjs`, `perf.mjs`, `cells.mjs`,
//! `reviews.mjs`, `capture.mjs`, `compaction.mjs`) relies on that to
//! propagate write failures. The "fail-open" *telemetry* pattern used for
//! `.bee/logs/contention.jsonl` (see `.bee/bin/lib/lock.mjs`'s
//! `appendContentionTelemetry`) is implemented by that call site wrapping
//! its OWN try/catch around a bare `fs.appendFileSync` — it deliberately
//! bypasses fsutil's `appendJsonl` entirely, specifically so fsutil's
//! version can stay a bare propagating write. This port keeps
//! [`append_jsonl`] propagating (`io::Result`), matching the actual frozen
//! source; a future fail-open telemetry helper (rust-port-3's
//! contention-log append) should wrap this the same way lock.mjs does,
//! rather than changing this function's contract.

use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::de::DeserializeOwned;
use serde::Serialize;

/// ensureDir — recursive mkdir, matching `fs.mkdirSync(dir, { recursive: true })`.
pub fn ensure_dir(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)
}

/// hashFile — sha256 hex digest of a file's UTF-8 CONTENT (rust-port-20,
/// `.bee/bin/lib/fsutil.mjs` `hashFile`). Reads the file as a UTF-8 string
/// (not raw bytes) before hashing, matching the mjs source's
/// `fs.readFileSync(file, 'utf8')` — a lossless round-trip for the
/// well-formed UTF-8 source files this hashes (the managed-lib/helpers
/// drift check, `computeRuntimeDrift`), so this agrees byte-for-byte with a
/// raw-byte hash for every real caller, but stays a faithful port of what
/// the mjs source actually does rather than an equivalent-in-practice
/// shortcut. Propagates a read failure as `Err` — callers that need
/// fail-open "missing" semantics (as `computeRuntimeDrift` does) catch it
/// themselves, matching mjs's own bare `hashFile(abs)` call inside a
/// try/catch at the call site, never inside this function.
pub fn hash_file(file: &Path) -> io::Result<String> {
    use sha2::{Digest, Sha256};
    let text = fs::read_to_string(file)?;
    let digest = Sha256::digest(text.as_bytes());
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// readJson — tolerant JSON read. A missing/unreadable file OR malformed
/// JSON both fall open to `fallback`, mirroring `readJson(file, fallback =
/// null)`. A malformed-JSON fallback additionally warns to stderr, matching
/// the mjs source's `console.warn` — a broken store file must never look
/// silently identical to an absent one.
pub fn read_json<T: DeserializeOwned>(file: &Path, fallback: T) -> T {
    let text = match fs::read_to_string(file) {
        Ok(t) => t,
        Err(_) => return fallback,
    };
    // Strip a leading UTF-8 BOM (U+FEFF), same as the mjs reader (Windows
    // tooling BOM compatibility, GitHub #9).
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    match serde_json::from_str::<T>(text) {
        Ok(value) => value,
        Err(err) => {
            eprintln!(
                "bee: could not parse JSON at {} — {}. Using fallback; fix the file.",
                file.display(),
                err
            );
            fallback
        }
    }
}

/// ECMAScript's `String.prototype.trim()` WhiteSpace set is broader than
/// Rust's Unicode `White_Space` property in exactly one way that matters
/// here: it includes ZERO WIDTH NO-BREAK SPACE / BOM (U+FEFF), which the
/// general Unicode standard does NOT classify as whitespace. mjs's
/// `readJsonl` has no explicit BOM handling of its own (unlike `readJson`,
/// which strips a leading BOM by hand); it tolerates a BOM landing on a
/// jsonl line only as a side effect of `line.trim()` silently eating FEFF.
/// This is a real, observed scenario — `fsutil.mjs`'s own comment on
/// `readJson` cites Windows tooling prepending a BOM (GitHub #9) — and the
/// oracle differ in `tests/fsutil_oracle.rs` caught the divergence directly:
/// serde's `str::trim()` leaves FEFF in place, so a BOM-prefixed first jsonl
/// line parsed fine under mjs but failed to parse under a plain Rust trim.
/// Matching it exactly (not approximating) is the point of running the real
/// mjs reader as the oracle instead of guessing from reading the source.
///
/// rpl-3 DEVIATION: this used to be `c.is_whitespace() || c == '\u{FEFF}'`,
/// which fixed the FEFF end of the divergence and opened the other one.
/// Rust's `char::is_whitespace` is Unicode `White_Space`, which includes
/// U+0085 NEXT LINE — and JS `\s`/`trim()` do NOT (measured over the whole
/// BMP on node v24.14.1). So a jsonl line prefixed with U+0085 parsed under
/// this reader and was skipped by mjs. It now delegates to the single
/// enumerated authority in [`crate::datamark`], which the whole
/// content-safety layer is also built on, rather than keeping a second
/// approximate whitespace notion in the crate.
pub(crate) fn js_trim(s: &str) -> &str {
    crate::datamark::js_trim(s)
}

/// readJsonl — tolerant JSONL read. A missing/unreadable file returns `[]`;
/// each line is parsed independently and a corrupt/truncated line is
/// skipped rather than failing the whole read, matching the mjs source's
/// per-line `try { JSON.parse } catch { skip }`. Splits on `\r?\n` like the
/// mjs source's regex split: `\n` is the separator, and a trailing `\r`
/// immediately before it is stripped as part of that separator — but a
/// LONE `\r` with no following `\n` is NOT treated as a line break (mjs's
/// `/\r?\n/` requires the `\n`), so it stays embedded in whatever line it's
/// part of.
///
/// rust-port-22 (reworked after a goal-check NEEDS_REVISION): when `file`
/// is `.bee/decisions.jsonl`, this counts as one
/// `read_accounting::record_decisions_journal_parse()` — HERE, at the
/// actual shared primitive every decisions-journal reader funnels through
/// (directly or transitively), rather than at any of `decisions.rs`'s own
/// call sites. The first cut of this seam counted at those call sites and
/// the judge proved it gameable: it hand-wrote two extra real
/// `read_jsonl(&decisions_path(root))` calls directly against THIS
/// function, from a brand-new location outside `decisions.rs` entirely
/// (simulating a rust-port-23 hoist), and the baseline test's count did not
/// move — three real store reads at a `build_status`-level load point were
/// completely invisible. Counting here instead means ANY call to this
/// function against the decisions journal is counted, from anywhere in the
/// crate, present or future — the property the "lowest shared read
/// primitive" placement rule was always supposed to buy. `read_jsonl` is
/// otherwise used for many unrelated stores (backlog, capture-queue,
/// scribing-runs, decisions-archive, ...); the path check below is
/// deliberately narrow (`.bee/decisions.jsonl` only) so none of those
/// other, unrelated reads are miscounted into this bucket.
pub fn read_jsonl<T: DeserializeOwned>(file: &Path) -> Vec<T> {
    if is_decisions_journal_path(file) {
        crate::read_accounting::record_decisions_journal_parse();
    }
    let text = match fs::read_to_string(file) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut events = Vec::new();
    for raw_line in text.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = js_trim(line);
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<T>(trimmed) {
            events.push(value);
        }
        // Corrupt/truncated line: skip rather than failing the whole read.
    }
    events
}

/// True when `file`'s last two path components are exactly `.bee` then
/// `decisions.jsonl` — deliberately component-wise (via [`Path::ends_with`],
/// not a string suffix check) so it is robust to path separator and
/// trailing-slash differences, and deliberately narrow: it must NOT match
/// `decisions-archive.jsonl` (a different store, excluded from this
/// counter's bucket by design) or any other `.jsonl` file this same
/// generic function reads on behalf of other stores.
fn is_decisions_journal_path(file: &Path) -> bool {
    file.ends_with(Path::new(".bee").join("decisions.jsonl"))
}

#[cfg(test)]
mod read_accounting_placement_tests {
    use super::*;

    #[test]
    fn is_decisions_journal_path_matches_only_the_journal_not_the_archive() {
        assert!(is_decisions_journal_path(Path::new("/tmp/repo/.bee/decisions.jsonl")));
        assert!(is_decisions_journal_path(Path::new(".bee/decisions.jsonl")));
        assert!(!is_decisions_journal_path(Path::new("/tmp/repo/.bee/decisions-archive.jsonl")));
        assert!(!is_decisions_journal_path(Path::new("/tmp/repo/.bee/backlog.jsonl")));
        assert!(!is_decisions_journal_path(Path::new("/tmp/repo/decisions.jsonl"))); // no .bee parent
    }

    /// The load-bearing regression: calling `read_jsonl` DIRECTLY against
    /// the decisions journal path — bypassing `decisions.rs` entirely, the
    /// exact shape of the judge's hoist experiment — must still increment
    /// the shared counter.
    #[test]
    fn read_jsonl_against_the_decisions_journal_path_increments_the_counter_from_any_call_site() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(".bee").join("decisions.jsonl");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "{\"id\":\"d1\"}\n").unwrap();

        crate::read_accounting::reset();
        let _: Vec<serde_json::Value> = read_jsonl(&file);
        assert_eq!(crate::read_accounting::snapshot().decisions_journal_parses, 1);

        // A DIFFERENT store, same generic function: never miscounted.
        crate::read_accounting::reset();
        let backlog = dir.path().join(".bee").join("backlog.jsonl");
        fs::write(&backlog, "{\"n\":1}\n").unwrap();
        let _: Vec<serde_json::Value> = read_jsonl(&backlog);
        assert_eq!(crate::read_accounting::snapshot().decisions_journal_parses, 0);
        crate::read_accounting::reset();
    }
}

static WRITE_JSON_ATOMIC_COUNTER: AtomicU64 = AtomicU64::new(0);

/// writeJsonAtomic — write-then-rename, matching the mjs source. The tmp
/// file's name is unique per call (pid + a monotonic process-local counter
/// + a time-derived component), so concurrent writers targeting the same
/// `file` never share one tmp path — same collision-avoidance rationale as
/// the mjs source's `pid + counter + crypto.randomBytes` scheme. The tmp
/// file lives in the SAME directory as `file`, so the rename is an atomic
/// same-filesystem rename. Pretty-printed with a 2-space indent plus a
/// trailing newline, matching `JSON.stringify(obj, null, 2)` + `"\n"`.
///
/// `file` itself is only ever touched by the single `rename` syscall — the
/// new content is fully written to the tmp path first, so a reader (or a
/// crash) can only ever see `file` holding its OLD content (before the
/// rename) or its fully-written NEW content (after), never a partial mix.
/// On any failure (write or rename), the tmp file is removed best-effort
/// and the original error is rethrown unchanged, matching the mjs source's
/// catch-cleanup-rethrow.
pub fn write_json_atomic<T: Serialize>(file: &Path, obj: &T) -> io::Result<()> {
    if let Some(dir) = file.parent() {
        if !dir.as_os_str().is_empty() {
            ensure_dir(dir)?;
        }
    }
    let pid = std::process::id();
    let seq = WRITE_JSON_ATOMIC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let mut tmp_name = file.as_os_str().to_owned();
    tmp_name.push(format!(".{pid:x}-{seq:x}-{nanos:x}.tmp"));
    let tmp = std::path::PathBuf::from(tmp_name);

    let mut json = serde_json::to_string_pretty(obj).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    json.push('\n');

    let result = fs::write(&tmp, json.as_bytes()).and_then(|()| fs::rename(&tmp, file));
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

/// appendJsonl — append one compact JSON line (`JSON.stringify(obj)` + a
/// trailing `\n`, no pretty-printing), matching the mjs source's shape.
/// Propagates I/O errors (see module doc comment for why this does not
/// fail open).
pub fn append_jsonl<T: Serialize>(file: &Path, obj: &T) -> io::Result<()> {
    if let Some(dir) = file.parent() {
        if !dir.as_os_str().is_empty() {
            ensure_dir(dir)?;
        }
    }
    let mut line = serde_json::to_string(obj).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    line.push('\n');
    let mut f = fs::OpenOptions::new().create(true).append(true).open(file)?;
    f.write_all(line.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::thread;

    fn tmp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("failed to create tempdir")
    }

    #[test]
    fn fsutil_read_json_returns_fallback_on_missing_file() {
        let dir = tmp_dir();
        let file = dir.path().join("missing.json");
        let result: Value = read_json(&file, json!({"fallback": true}));
        assert_eq!(result, json!({"fallback": true}));
    }

    #[test]
    fn fsutil_read_json_returns_fallback_on_malformed_json() {
        let dir = tmp_dir();
        let file = dir.path().join("broken.json");
        fs::write(&file, b"{ not valid json").unwrap();
        let result: Value = read_json(&file, Value::Null);
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn fsutil_read_json_strips_leading_bom() {
        let dir = tmp_dir();
        let file = dir.path().join("bom.json");
        let mut bytes = vec![0xef, 0xbb, 0xbf];
        bytes.extend_from_slice(br#"{"ok":true}"#);
        fs::write(&file, &bytes).unwrap();
        let result: Value = read_json(&file, Value::Null);
        assert_eq!(result, json!({"ok": true}));
    }

    #[test]
    fn fsutil_read_jsonl_returns_empty_on_missing_file() {
        let dir = tmp_dir();
        let file = dir.path().join("missing.jsonl");
        let result: Vec<Value> = read_jsonl(&file);
        assert!(result.is_empty());
    }

    #[test]
    fn fsutil_append_jsonl_appends_compact_lines_in_order() {
        let dir = tmp_dir();
        let file = dir.path().join("log.jsonl");
        append_jsonl(&file, &json!({"n": 1})).unwrap();
        append_jsonl(&file, &json!({"n": 2})).unwrap();
        let text = fs::read_to_string(&file).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines, vec![r#"{"n":1}"#, r#"{"n":2}"#]);
    }

    // Crash-safety property: `file` is only ever mutated by the write
    // primitive as a whole (never partially) — proven here with genuine
    // concurrent renames across threads rather than a hand-simulated
    // "kill between write and rename" (which a unit test cannot literally
    // do). Several writer threads race on the SAME target path while a
    // reader thread polls; at every read, the bytes on disk must either be
    // empty (before the first write lands) or a complete, parseable JSON
    // object — never a half-written mix of two writers' bytes. This is a
    // real red/green check: a naive `fs::write` straight to `file` (no
    // temp-file-then-rename) makes this fail intermittently under
    // concurrency, which is exactly the failure mode the atomic port
    // exists to close.
    #[test]
    fn fsutil_write_json_atomic_concurrent_writers_never_produce_partial_content() {
        let dir = tmp_dir();
        let file = dir.path().join("shared.json");

        // Seed the target with a complete "old" record BEFORE any writer
        // thread starts. From this point on the file always exists with a
        // fully-formed record — an empty or unparseable read is never a
        // legitimate "not started yet" state, it can only be a truncate
        // race (a non-atomic writer opening `file` with truncate-on-open
        // and then writing in a separate step). This is what makes the
        // test able to fail red: `fs::write` truncates-then-writes, so a
        // reader can catch the file at zero length between those two
        // steps.
        write_json_atomic(&file, &json!({"writer": "seed", "iter": 0})).unwrap();

        let writer_count = 6usize;
        let iterations = 60usize;

        let reader_file = file.clone();
        let reader = thread::spawn(move || {
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(800);
            let mut observations = 0usize;
            while std::time::Instant::now() < deadline {
                let text = fs::read_to_string(&reader_file)
                    .unwrap_or_else(|e| panic!("target file must exist after seeding: {e}"));
                assert!(
                    !text.is_empty(),
                    "observed an EMPTY file after seeding — a non-atomic writer truncated `file` \
                     before writing the new content (this is exactly the crash-unsafe window the \
                     atomic write primitive exists to close)"
                );
                let parsed: Result<Value, _> = serde_json::from_str(&text);
                assert!(
                    parsed.is_ok(),
                    "observed non-empty, non-parseable content on disk mid-write: {text:?} — {:?}",
                    parsed.err()
                );
                let obj = parsed.unwrap();
                assert!(
                    obj.get("writer").is_some() && obj.get("iter").is_some(),
                    "observed truncated/partial JSON object missing expected fields: {text:?}"
                );
                observations += 1;
            }
            observations
        });

        let mut writers = Vec::new();
        for w in 0..writer_count {
            let target = file.clone();
            writers.push(thread::spawn(move || {
                for i in 0..iterations {
                    write_json_atomic(&target, &json!({"writer": w, "iter": i})).unwrap();
                }
            }));
        }
        for handle in writers {
            handle.join().expect("writer thread panicked");
        }
        let observations = reader.join().expect("reader thread panicked");
        assert!(observations > 0, "reader thread never observed a write — test is not exercising concurrency");

        // Final state must be a single, complete, valid record.
        let final_text = fs::read_to_string(&file).unwrap();
        let final_value: Value = serde_json::from_str(&final_text).expect("final file content must be valid JSON");
        assert!(final_value.get("writer").is_some() && final_value.get("iter").is_some());

        // No stray tmp files left behind in the directory.
        let leftovers: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|name| name != "shared.json")
            .collect();
        assert!(leftovers.is_empty(), "stray tmp files left behind: {leftovers:?}");
    }
}
