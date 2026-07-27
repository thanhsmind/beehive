//! capture — read-only projection over `.bee/capture-queue.jsonl` (the
//! capture queue, decision 0017), ported from `.bee/bin/lib/capture.mjs`'s
//! `captureQueue`/`pendingCaptureStubs` (rust-port-13, CONTEXT.md D3/D5).
//! Read-only, zero subprocess: never appends a stub or flush record.
//!
//! `.bee/bin/lib/capture.mjs` is FROZEN for the duration of the rust-port
//! feature (D1) — this reader mirrors `pendingCaptureStubs`'s fold
//! (stubs minus flushed, oldest first) exactly. Oracle-diffed against the
//! real mjs module in `tests/status_readers_a.rs`.

use std::collections::HashSet;
use std::path::Path;

use serde_json::Value;

use crate::fsutil::read_jsonl;

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

// Tests live in crates/bee-core/tests/status_readers_a.rs (this cell's
// single integration target) rather than here.
