//! decisions — the READ side of `.bee/decisions.jsonl`, ported from
//! `.bee/bin/lib/decisions.mjs`'s `activeDecisions` (rust-port-13,
//! CONTEXT.md D3/D5). Read-only, zero subprocess: no `logDecision`/
//! `supersedeDecision`/`redactDecision`/`archiveDecisions` writer port here
//! — this cell's prohibition is "no decision/capture/backlog writes".
//!
//! `.bee/bin/lib/decisions.mjs` is FROZEN for the duration of the rust-port
//! feature (D1). This module mirrors `activeDecisions`'s full read-path
//! semantics: supersession/redaction exclusion, the tag overlay
//! (decision-propagation dp-5/D7c — latest `tag` event wins by date then
//! file order, REPLACING the whole `tags` array and `scope` only when the
//! winning event actually carries one), and the `all: true` archive-union
//! branch (dp-3/D4c — active-file events win ties by id, ordering by date
//! descending with an index tiebreak equivalent to `.reverse()` on an
//! unarchived store). Oracle-diffed against the real mjs module in
//! `tests/status_readers_a.rs`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::fsutil::read_jsonl;
use crate::jsdate::parse_iso_ms;

pub fn decisions_path(root: &Path) -> PathBuf {
    root.join(".bee").join("decisions.jsonl")
}

pub fn decisions_archive_path(root: &Path) -> PathBuf {
    root.join(".bee").join("decisions-archive.jsonl")
}

struct TagPatch {
    /// `Some(vec)` — REPLACE the whole `tags` array (even `Some(vec![])`);
    /// `None` — the winning tag event carried no array, leave `tags` alone.
    tags: Option<Vec<Value>>,
    /// `Some(scope)` — replace; `None` — leave `scope` alone.
    scope: Option<String>,
}

// A JS truthy-string check: `typeof v === 'string' && v` (empty string is
// falsy). Used everywhere the mjs source gates a field on "is this actually
// a non-empty string", not merely "is this present".
fn truthy_str(v: Option<&Value>) -> Option<&str> {
    v.and_then(Value::as_str).filter(|s| !s.is_empty())
}

fn event_date_ms(event: &Value) -> Option<i64> {
    event.get("date").and_then(Value::as_str).and_then(parse_iso_ms)
}

/// `buildTagOverlay(root)`: the active file's `tag` events only, latest
/// (by date, ties broken by later file position) wins per `target` id.
fn build_tag_overlay(root: &Path) -> HashMap<String, TagPatch> {
    let events: Vec<Value> = read_jsonl(&decisions_path(root));
    // rust-port-22: lowest-shared-primitive counter, see
    // `crate::read_accounting`'s module doc comment. This is one of the
    // TWO real decisions.jsonl reads a single `active_decisions` call
    // performs today (the other is below, in `active_decisions` itself).
    crate::read_accounting::record_decisions_journal_parse();
    let mut tag_events: Vec<(usize, Value)> = events
        .into_iter()
        .enumerate()
        .filter(|(_, e)| {
            e.get("type").and_then(Value::as_str) == Some("tag") && e.get("target").and_then(Value::as_str).is_some()
        })
        .collect();
    tag_events.sort_by(|(ia, ea), (ib, eb)| {
        let ma = event_date_ms(ea);
        let mb = event_date_ms(eb);
        if let (Some(x), Some(y)) = (ma, mb) {
            if x != y {
                return x.cmp(&y); // ascending: earlier date first
            }
        }
        ia.cmp(ib) // ascending: earlier file position first
    });
    let mut overlay: HashMap<String, TagPatch> = HashMap::new();
    for (_, event) in tag_events {
        let Some(target) = event.get("target").and_then(Value::as_str) else { continue };
        let tags = event.get("tags").and_then(Value::as_array).cloned();
        let scope = truthy_str(event.get("scope")).map(str::to_string);
        overlay.insert(target.to_string(), TagPatch { tags, scope });
    }
    overlay
}

/// `applyTagOverlay(event, overlay)`: returns `event` unchanged (a clone,
/// since Rust values aren't shared references the way the mjs source's
/// "same object" identity check implies) when there is no patch for its id;
/// otherwise a copy with `tags`/`scope` overridden per [`TagPatch`].
fn apply_tag_overlay(event: &Value, overlay: &HashMap<String, TagPatch>) -> Value {
    let Some(id) = event.get("id").and_then(Value::as_str) else { return event.clone() };
    let Some(patch) = overlay.get(id) else { return event.clone() };
    let mut obj = event.as_object().cloned().unwrap_or_default();
    if let Some(tags) = &patch.tags {
        obj.insert("tags".to_string(), Value::Array(tags.clone()));
    }
    if let Some(scope) = &patch.scope {
        obj.insert("scope".to_string(), Value::String(scope.clone()));
    }
    Value::Object(obj)
}

fn is_decide_or_supersede(event: &Value) -> bool {
    matches!(event.get("type").and_then(Value::as_str), Some("decide") | Some("supersede"))
}

/// `activeDecisions(root, {recent, all})`: decide/supersede events not
/// themselves superseded or redacted, newest first (tag-overlay applied).
///
/// - `all: false` (default): reads ONLY the active store, ordered by a
///   plain positional reverse (byte-identical to the pre-dp-3 behavior).
/// - `all: true`: additionally unions in `.bee/decisions-archive.jsonl`
///   (missing/empty archive silently treated as "nothing extra"),
///   de-duplicated by id with the active copy winning, ordered by event
///   date descending with an original-insertion-index tiebreak — which is
///   mathematically identical to the `all: false` reverse whenever the
///   archive contributes nothing new.
pub fn active_decisions(root: &Path, recent: Option<usize>, all: bool) -> Vec<Value> {
    let overlay = build_tag_overlay(root);

    if !all {
        let events: Vec<Value> = read_jsonl(&decisions_path(root));
        crate::read_accounting::record_decisions_journal_parse();
        let mut superseded: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut redacted: std::collections::HashSet<String> = std::collections::HashSet::new();
        for event in &events {
            if event.get("type").and_then(Value::as_str) == Some("supersede") {
                if let Some(s) = truthy_str(event.get("supersedes")) {
                    superseded.insert(s.to_string());
                }
            }
            if event.get("type").and_then(Value::as_str) == Some("redact") {
                if let Some(r) = truthy_str(event.get("redacts")) {
                    redacted.insert(r.to_string());
                }
            }
        }
        let mut active: Vec<Value> = events
            .into_iter()
            .filter(|event| {
                let id = event.get("id").and_then(Value::as_str).unwrap_or("");
                is_decide_or_supersede(event) && !superseded.contains(id) && !redacted.contains(id)
            })
            .collect();
        active.reverse();
        let active: Vec<Value> = active.iter().map(|e| apply_tag_overlay(e, &overlay)).collect();
        return match recent {
            Some(n) => active.into_iter().take(n).collect(),
            None => active,
        };
    }

    let active_events: Vec<Value> = read_jsonl(&decisions_path(root));
    crate::read_accounting::record_decisions_journal_parse();
    // The archive file is a DIFFERENT store (`decisions-archive.jsonl`),
    // deliberately excluded from the `decisions_journal_parses` bucket,
    // which counts the journal (`decisions.jsonl`) only.
    let archived_events: Vec<Value> = read_jsonl(&decisions_archive_path(root));

    // Map insertion-order semantics (JS `Map.set` on an existing key keeps
    // its original position but updates the value): track first-seen order
    // separately from the value store.
    let mut order: Vec<String> = Vec::new();
    let mut by_id: HashMap<String, Value> = HashMap::new();
    for event in &active_events {
        if let Some(id) = event.get("id").and_then(Value::as_str) {
            if !by_id.contains_key(id) {
                order.push(id.to_string());
            }
            by_id.insert(id.to_string(), event.clone());
        }
    }
    for event in &archived_events {
        if let Some(id) = event.get("id").and_then(Value::as_str) {
            if !by_id.contains_key(id) {
                order.push(id.to_string());
                by_id.insert(id.to_string(), event.clone());
            }
        }
    }
    // ORDER-IRRELEVANT `remove` (rust-port-15 sweep): `by_id` is a
    // `std::collections::HashMap`, not a `serde_json::Map`, so the
    // `preserve_order`/`swap_remove` aliasing does not apply here at all —
    // and the output sequence comes from `order`, never from iterating
    // this map.
    let events: Vec<Value> = order.into_iter().map(|id| by_id.remove(&id).unwrap()).collect();

    let indexed: Vec<(usize, Value)> = events.into_iter().enumerate().collect();
    let mut superseded: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut redacted: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (_, event) in &indexed {
        if event.get("type").and_then(Value::as_str) == Some("supersede") {
            if let Some(s) = truthy_str(event.get("supersedes")) {
                superseded.insert(s.to_string());
            }
        }
        if event.get("type").and_then(Value::as_str) == Some("redact") {
            if let Some(r) = truthy_str(event.get("redacts")) {
                redacted.insert(r.to_string());
            }
        }
    }

    let mut active: Vec<(usize, Value)> = indexed
        .into_iter()
        .filter(|(_, event)| {
            let id = event.get("id").and_then(Value::as_str).unwrap_or("");
            is_decide_or_supersede(event) && !superseded.contains(id) && !redacted.contains(id)
        })
        .collect();

    active.sort_by(|(ia, ea), (ib, eb)| {
        let ma = event_date_ms(ea);
        let mb = event_date_ms(eb);
        if let (Some(x), Some(y)) = (ma, mb) {
            if x != y {
                return y.cmp(&x); // descending: later date first
            }
        }
        ib.cmp(ia) // descending: higher original index first
    });

    let result: Vec<Value> = active.iter().map(|(_, event)| apply_tag_overlay(event, &overlay)).collect();
    match recent {
        Some(n) => result.into_iter().take(n).collect(),
        None => result,
    }
}

// Tests live in crates/bee-core/tests/status_readers_a.rs (this cell's
// single integration target) rather than here.
