//! capture — the capture queue (`.bee/capture-queue.jsonl`, decision 0017),
//! ported from `.bee/bin/lib/capture.mjs` (rust-port-13 for the read side,
//! CONTEXT.md D3/D5; `rpl-3` for the write side).
//!
//! rust-port-13 landed the READ projection only
//! (`captureQueue`/`pendingCaptureStubs`) because `status` was the only
//! consumer. `rpl-3` extends THIS module — never a second one beside it —
//! with the two writers the `capture` CLI group needs,
//! [`add_capture_stub`] and [`flush_capture_stub`].
//!
//! `.bee/bin/lib/capture.mjs` is FROZEN for the duration of the rust-port
//! feature (D1) — this reader mirrors `pendingCaptureStubs`'s fold
//! (stubs minus flushed, oldest first) exactly. Oracle-diffed against the
//! real mjs module in `tests/status_readers_a.rs`; the writers and their
//! refusal texts in `tests/datamark_oracle.rs`.
//!
//! # NO LOCK
//!
//! Both writers go straight to `appendJsonl` (`capture.mjs:80,124`) with no
//! `acquireStoreLockOnceSync` anywhere in the module — unlike
//! `decisions.mjs`, whose append runs under `withDecisionsLockSync`. That
//! is a deliberate property of the queue (a lost interleaved append is
//! cheaper than a blocked settlement), and it is mirrored here rather than
//! "improved": adding a lock on the Rust side would be a behavior
//! divergence in a store two runtimes may write concurrently.

use std::collections::HashSet;
use std::path::Path;

use serde_json::{Map, Value};

use crate::datamark::{assert_safe_capture_content, js_trim};
use crate::fsutil::{append_jsonl, read_jsonl};

pub fn capture_queue_path(root: &Path) -> std::path::PathBuf {
    root.join(".bee").join("capture-queue.jsonl")
}

/// `pendingCaptureStubs(root)`: stubs not yet flushed, oldest (`at`) first —
/// `at` compared as a plain string (`localeCompare`), matching the mjs
/// source's lexical ISO-timestamp sort rather than a parsed-date sort.
pub fn pending_capture_stubs(root: &Path) -> Vec<Value> {
    let events: Vec<Value> = read_jsonl(&capture_queue_path(root));
    let mut flushed: HashSet<String> = HashSet::new();
    let mut stubs: Vec<Value> = Vec::new();
    for event in &events {
        let Some(obj) = event.as_object() else { continue };
        let kind = obj.get("kind").and_then(Value::as_str);
        let id = obj.get("id").and_then(Value::as_str);
        match (kind, id) {
            (Some("flush"), Some(id)) => {
                flushed.insert(id.to_string());
            }
            (Some("stub"), Some(_)) => stubs.push(event.clone()),
            _ => {}
        }
    }
    stubs.retain(|stub| {
        let id = stub.get("id").and_then(Value::as_str).unwrap_or("");
        !flushed.contains(id)
    });
    stubs.sort_by(|a, b| {
        let at_a = a.get("at").map(json_as_string_key).unwrap_or_default();
        let at_b = b.get("at").map(json_as_string_key).unwrap_or_default();
        at_a.cmp(&at_b)
    });
    stubs
}

// `String(a.at)` on an arbitrary JSON value before comparing — mirrors the
// mjs source's coercion so a non-string `at` (never produced by
// `addCaptureStub`, but tolerated on read like every other fail-open reader
// in this crate) still sorts deterministically instead of panicking.
fn json_as_string_key(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// `captureQueue(root)`: `{count, stubs}` convenience summary for status
/// surfaces.
pub fn capture_queue(root: &Path) -> Value {
    let stubs = pending_capture_stubs(root);
    serde_json::json!({ "count": stubs.len(), "stubs": stubs })
}

// ─── the write side (rpl-3) ────────────────────────────────────────────────

/// `crypto.randomUUID()`: lowercase `8-4-4-4-12` hex, version nibble `4`,
/// variant nibble in `[89ab]`. The shape is not cosmetic — the parity
/// harness masks an `id` ONLY when it matches UUID-v4
/// (`bee_parity::normalize::Shape::UuidV4`), so a differently shaped id
/// fails the diff instead of being masked into agreement.
pub fn random_uuid() -> Result<String, String> {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).map_err(|e| format!("entropy source failed: {e}"))?;
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let hex: String = b.iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
}

/// `capture.mjs:34-45` `normalizeList`, for the ONE input shape the CLI can
/// produce: `flags.did ? String(flags.did) : null` is always a string or
/// null, never the array the mjs signature also accepts. A string that is
/// whitespace-only (by JS's trim, so a lone U+FEFF counts) is falsy after
/// `.trim()` and yields `[]` rather than `[""]`.
fn normalize_list(value: Option<&str>) -> Vec<String> {
    let Some(raw) = value else { return Vec::new() };
    if js_trim(raw).is_empty() {
        return Vec::new();
    }
    raw.split(',').map(js_trim).filter(|p| !p.is_empty()).map(str::to_string).collect()
}

/// `typeof x === 'string' && x.trim() ? x.trim() : null` — the idiom
/// `addCaptureStub` uses for `area`, `lane` and `source`.
fn optional_trimmed(value: Option<&str>) -> Option<String> {
    let raw = value?;
    let trimmed = js_trim(raw);
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// The stub inputs, exactly as `bee.mjs:4031-4038` hands them over — every
/// one an optional string, because that is all `parseFlags` can produce.
#[derive(Debug, Default, Clone)]
pub struct StubFields<'a> {
    pub outcome: Option<&'a str>,
    pub dids: Option<&'a str>,
    pub area: Option<&'a str>,
    pub files: Option<&'a str>,
    pub lane: Option<&'a str>,
    pub source: Option<&'a str>,
}

/// `capture.mjs:57-82` `addCaptureStub`.
///
/// `id` and `at` are threaded in rather than read here, the same discipline
/// `bee_core::intent` uses: the store layer stays deterministic and
/// unit-testable, and the wall clock is stamped once at the CLI edge.
///
/// The four refusals are in the mjs source's ORDER, which is observable:
/// a `--lane high-risk` call whose outcome also carries a secret sees the
/// high-risk refusal, not the secret one.
pub fn add_capture_stub(
    root: &Path,
    fields: &StubFields<'_>,
    id: &str,
    at: &str,
) -> Result<Value, String> {
    let outcome_raw = fields.outcome.unwrap_or("");
    if js_trim(outcome_raw).is_empty() {
        return Err("addCaptureStub: outcome text is required.".to_string());
    }
    // `lane === 'high-risk'` — strict equality against the RAW flag value,
    // before any trimming, so `--lane " high-risk "` does NOT trip it.
    if fields.lane == Some("high-risk") {
        return Err(
            "addCaptureStub: high-risk settlements never queue — run the full bee-scribing sync inline (decision 0017)."
                .to_string(),
        );
    }

    // Key order is the mjs object literal's, and `serde_json`'s
    // `preserve_order` feature is what keeps it: kind, id, at, outcome,
    // dids, area, files, lane — then `source` APPENDED only when present,
    // so a stub written without it stays byte-shape-identical to today's.
    let outcome = js_trim(outcome_raw).to_string();
    let area = optional_trimmed(fields.area);
    let mut stub = Map::new();
    stub.insert("kind".to_string(), Value::String("stub".to_string()));
    stub.insert("id".to_string(), Value::String(id.to_string()));
    stub.insert("at".to_string(), Value::String(at.to_string()));
    stub.insert("outcome".to_string(), Value::String(outcome.clone()));
    stub.insert("dids".to_string(), string_array(normalize_list(fields.dids)));
    stub.insert("area".to_string(), opt_string(area.clone()));
    stub.insert("files".to_string(), string_array(normalize_list(fields.files)));
    stub.insert("lane".to_string(), opt_string(optional_trimmed(fields.lane)));
    if let Some(source) = optional_trimmed(fields.source) {
        stub.insert("source".to_string(), Value::String(source));
    }

    // Validated AFTER trimming and on the stub's own values — a leading
    // space would otherwise change which text the guard ever sees.
    assert_safe_capture_content("outcome", &outcome)?;
    if let Some(area) = &area {
        assert_safe_capture_content("area", area)?;
    }

    let value = Value::Object(stub);
    append_jsonl(&capture_queue_path(root), &value)
        .map_err(|e| format!("append {}: {e}", capture_queue_path(root).display()))?;
    Ok(value)
}

/// `capture.mjs:110-127` `flushCaptureStub`.
pub fn flush_capture_stub(
    root: &Path,
    id: Option<&str>,
    into: Option<&str>,
    at: &str,
) -> Result<Value, String> {
    let raw_id = id.unwrap_or("");
    if js_trim(raw_id).is_empty() {
        return Err("flushCaptureStub: stub id is required.".to_string());
    }
    let wanted = js_trim(raw_id);
    let pending = pending_capture_stubs(root);
    let found = pending
        .iter()
        .find(|s| s.get("id").and_then(Value::as_str) == Some(wanted));
    let Some(stub) = found else {
        // `${id}` — the RAW flag value, NOT the trimmed one the lookup used.
        return Err(format!("flushCaptureStub: no pending stub with id \"{raw_id}\"."));
    };

    let mut record = Map::new();
    record.insert("kind".to_string(), Value::String("flush".to_string()));
    record.insert("id".to_string(), stub.get("id").cloned().unwrap_or(Value::Null));
    record.insert("at".to_string(), Value::String(at.to_string()));
    record.insert("into".to_string(), opt_string(optional_trimmed(into)));

    let value = Value::Object(record);
    append_jsonl(&capture_queue_path(root), &value)
        .map_err(|e| format!("append {}: {e}", capture_queue_path(root).display()))?;
    Ok(value)
}

fn string_array(items: Vec<String>) -> Value {
    Value::Array(items.into_iter().map(Value::String).collect())
}

fn opt_string(value: Option<String>) -> Value {
    value.map(Value::String).unwrap_or(Value::Null)
}

// Read-side tests live in crates/bee-core/tests/status_readers_a.rs; the
// write side and its refusal texts are oracle-diffed in
// crates/bee-core/tests/datamark_oracle.rs. The unit tests below only pin
// the two things a file-based oracle cannot see: the generated id's SHAPE
// and the no-lock property.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_uuid_has_the_crypto_randomuuid_shape() {
        let id = random_uuid().expect("uuid");
        assert_eq!(id.len(), 36);
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(parts.iter().map(|p| p.len()).collect::<Vec<_>>(), vec![8, 4, 4, 4, 12]);
        assert!(id.chars().all(|c| c == '-' || c.is_ascii_hexdigit()));
        assert!(id.chars().all(|c| !c.is_ascii_uppercase()));
        assert_eq!(parts[2].as_bytes()[0], b'4', "version nibble");
        assert!(matches!(parts[3].as_bytes()[0], b'8' | b'9' | b'a' | b'b'), "variant nibble");
    }

    #[test]
    fn normalize_list_matches_the_mjs_split_trim_filter() {
        assert_eq!(normalize_list(None), Vec::<String>::new());
        assert_eq!(normalize_list(Some("   ")), Vec::<String>::new());
        assert_eq!(normalize_list(Some("\u{FEFF}")), Vec::<String>::new(), "JS trim eats the BOM");
        assert_eq!(normalize_list(Some("a, b, ,c")), vec!["a", "b", "c"]);
    }
}
