//! backlog — read-only status projection over `.bee/backlog.jsonl` and the
//! legacy `docs/backlog.md` table, ported from `.bee/bin/lib/backlog.mjs`'s
//! `readBacklogCounts` (rust-port-13, CONTEXT.md D3/D5). Read-only, zero
//! subprocess: never appends a PBI event, never writes `docs/backlog.md`.
//!
//! `.bee/bin/lib/backlog.mjs` is FROZEN for the duration of the rust-port
//! feature (D1) — this module mirrors `readBacklogCounts`'s fold-first,
//! legacy-fallback contract exactly (backlog-unification D3): once ANY
//! `kind:'pbi'` event exists in `.bee/backlog.jsonl`, counts derive from the
//! fold exclusively; only a repo with zero PBI events falls back to parsing
//! the legacy `docs/backlog.md` table. Oracle-diffed against the real mjs
//! module in `tests/status_readers_a.rs`, same discipline as
//! `fsutil_oracle.rs` (rust-port-5).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::config::resolve_product_root;
use crate::fsutil::read_jsonl;

/// backlog.mjs `PBI_STATUSES` (D4): the real 5-value status enum measured
/// from the legacy table's live values.
pub const PBI_STATUSES: [&str; 5] = ["proposed", "in-flight", "parked", "done", "declined"];

/// backlog.mjs `BACKLOG_STATUSES` (D6): the legacy `docs/backlog.md`
/// table's original 3-value enum — unchanged, kept distinct from
/// [`PBI_STATUSES`].
pub const BACKLOG_STATUSES: [&str; 3] = ["proposed", "in-flight", "done"];

fn backlog_jsonl_path(root: &Path) -> std::path::PathBuf {
    root.join(".bee").join("backlog.jsonl")
}

/// One folded PBI record — `foldPbis`'s per-id accumulator.
#[derive(Debug, Clone, PartialEq)]
pub struct PbiItem {
    pub id: String,
    pub title: String,
    pub cos: String,
    pub status: String,
    pub feature: Option<String>,
}

/// `foldPbis(root)`: last-event-wins fold of every `kind:'pbi'` record in
/// `.bee/backlog.jsonl` (add/status/amend), id-ordered by first appearance
/// via a `BTreeMap` keyed on id string (matches `Map` insertion semantics
/// closely enough for `readBacklogCounts`, which only needs the value set,
/// never insertion order). `has_events` is `true` the moment ANY `kind:'pbi'`
/// row is seen, even a malformed one missing `id` — mirroring the mjs
/// source's `hasEvents = true` placement (set before the `id` presence
/// check).
pub fn fold_pbis(root: &Path) -> (BTreeMap<String, PbiItem>, bool) {
    let mut items: BTreeMap<String, PbiItem> = BTreeMap::new();
    let mut has_events = false;
    // Reuses `fsutil::read_jsonl` rather than a hand-rolled split — this is
    // what gives `fold_pbis` the exact same per-line fail-open tolerance
    // (corrupt/truncated lines skipped) AND the exact same `js_trim` BOM
    // handling `capture.rs`/`decisions.rs` get "for free" through the same
    // helper (the oracle diff in `tests/status_readers_a.rs` caught a real
    // divergence here before this refactor: a plain Rust `.trim()` does not
    // strip a leading BOM the way JS's `String.trim()` does, so a
    // BOM-prefixed first line silently failed to parse under a naive port).
    let rows: Vec<Value> = read_jsonl(&backlog_jsonl_path(root));
    for row in rows {
        let Some(obj) = row.as_object() else { continue };
        if obj.get("kind").and_then(Value::as_str) != Some("pbi") {
            continue;
        }
        has_events = true;
        let Some(id) = obj.get("id").and_then(Value::as_str).filter(|s| !s.is_empty()) else {
            continue;
        };
        let event = obj.get("event").and_then(Value::as_str).unwrap_or("");
        match event {
            "add" => {
                if items.contains_key(id) {
                    continue; // duplicate add refused — first add wins
                }
                let status = obj
                    .get("status")
                    .and_then(Value::as_str)
                    .filter(|s| PBI_STATUSES.contains(s))
                    .unwrap_or("proposed")
                    .to_string();
                items.insert(
                    id.to_string(),
                    PbiItem {
                        id: id.to_string(),
                        title: obj.get("title").and_then(Value::as_str).unwrap_or("").to_string(),
                        cos: obj.get("cos").and_then(Value::as_str).unwrap_or("").to_string(),
                        status,
                        feature: obj
                            .get("feature")
                            .and_then(Value::as_str)
                            .filter(|s| !s.is_empty())
                            .map(str::to_string),
                    },
                );
            }
            "status" => {
                let Some(item) = items.get_mut(id) else { continue };
                if let Some(status) = obj.get("status").and_then(Value::as_str) {
                    if PBI_STATUSES.contains(&status) {
                        item.status = status.to_string();
                    }
                }
                if let Some(feature) = obj.get("feature").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    item.feature = Some(feature.to_string());
                }
            }
            "amend" => {
                let Some(item) = items.get_mut(id) else { continue };
                if let Some(title) = obj.get("title").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    item.title = title.to_string();
                }
                if let Some(cos) = obj.get("cos").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    item.cos = cos.to_string();
                }
            }
            _ => {}
        }
    }
    (items, has_events)
}

// 'in-flight' -> 'inFlight'; general camelCase-from-kebab, mirrors backlog.mjs
// `tokenKey`'s `/-([a-z])/g` replace.
fn token_key(token: &str) -> String {
    let mut out = String::new();
    let mut chars = token.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '-' {
            if let Some(&next) = chars.peek() {
                out.push(next.to_ascii_uppercase());
                chars.next();
                continue;
            }
        }
        out.push(c);
    }
    out
}

/// `foldedBacklogCounts(items)`: one entry per [`PBI_STATUSES`] value (keyed
/// via [`token_key`]) plus `total`.
fn folded_backlog_counts(items: &BTreeMap<String, PbiItem>) -> Value {
    let mut counts: BTreeMap<&str, i64> = PBI_STATUSES.iter().map(|s| (*s, 0)).collect();
    for item in items.values() {
        if let Some(c) = counts.get_mut(item.status.as_str()) {
            *c += 1;
        }
    }
    let total: i64 = counts.values().sum();
    let mut out = serde_json::Map::new();
    for status in PBI_STATUSES {
        out.insert(token_key(status), json!(counts[status]));
    }
    out.insert("total".to_string(), json!(total));
    Value::Object(out)
}

// Split a markdown table line into trimmed cells, dropping the empty edges
// that bordering pipes produce — mirrors backlog.mjs `splitRow`.
fn split_row(line: &str) -> Vec<String> {
    let mut cells: Vec<String> = line.split('|').map(|c| c.trim().to_string()).collect();
    if cells.first().is_some_and(String::is_empty) {
        // ORDER-IRRELEVANT `remove` (rust-port-15 sweep): `Vec::remove`,
        // not `serde_json::Map::remove` — it shifts and is already
        // order-preserving.
        cells.remove(0);
    }
    if cells.last().is_some_and(String::is_empty) {
        cells.pop();
    }
    cells
}

// Strip bold/italic/code markup and lowercase — mirrors backlog.mjs
// `normalizeStatus`.
fn normalize_status(cell: &str) -> String {
    cell.chars()
        .filter(|c| !matches!(c, '*' | '`' | '_'))
        .collect::<String>()
        .trim()
        .to_lowercase()
}

/// `legacyBacklogCounts(root)`: count `docs/backlog.md` table rows by their
/// Status column, resolved against [`resolve_product_root`]. `None` only
/// when the file is absent/unreadable; a present-but-tableless file returns
/// zeroed counts (the file's existence is what gates the preamble line).
fn legacy_backlog_counts(root: &Path) -> Option<Value> {
    let file = resolve_product_root(root).join("docs").join("backlog.md");
    let text = fs::read_to_string(&file).ok()?;

    let mut counts: BTreeMap<&str, i64> = BACKLOG_STATUSES.iter().map(|s| (*s, 0)).collect();

    let mut status_index: Option<usize> = None;
    for raw_line in text.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if !line.contains('|') {
            continue;
        }
        let cells = split_row(line);
        if status_index.is_none() {
            if let Some(idx) = cells.iter().position(|c| normalize_status(c) == "status") {
                status_index = Some(idx);
            }
            continue;
        }
        let idx = status_index.unwrap();
        if cells.len() <= idx {
            continue;
        }
        let token = normalize_status(&cells[idx]);
        if let Some(c) = counts.get_mut(token.as_str()) {
            *c += 1;
        }
    }

    let total: i64 = counts.values().sum();
    let mut out = serde_json::Map::new();
    for status in BACKLOG_STATUSES {
        out.insert(token_key(status), json!(counts[status]));
    }
    out.insert("total".to_string(), json!(total));
    Some(Value::Object(out))
}

/// `readBacklogCounts(root)`: fold-first (backlog-unification D3) — once ANY
/// `kind:'pbi'` event exists, counts derive from the fold exclusively; else
/// falls back to the legacy `docs/backlog.md` table parse. `None` only when
/// there is neither a fold nor a parseable legacy table.
pub fn read_backlog_counts(root: &Path) -> Option<Value> {
    let (items, has_events) = fold_pbis(root);
    if has_events {
        return Some(folded_backlog_counts(&items));
    }
    legacy_backlog_counts(root)
}

// Tests live in crates/bee-core/tests/status_readers_a.rs (this cell's
// single integration target — cargo test -p bee-core --test
// status_readers_a) rather than here, so every reader's oracle-diff proof
// sits in one place per must-have.
