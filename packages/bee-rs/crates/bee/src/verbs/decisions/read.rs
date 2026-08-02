// the read model — overlay, activeDecisions, and the filters
//
// Split out of the single 3.5k-line verbs/decisions.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{append_jsonl, ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, AcquireOnce};
use crate::verbs::reservations::{
    date_parse_val, finish, jget, js_date_parse, js_disp, js_disp_opt, js_is_ws, js_number_flag,
    js_numberify, js_quote, js_strict_eq, js_trim, keys_known, now_iso, parse_flags,
    pseudo_uuid_v4, truthy, v_is_str, Err2, Ex, Exotic, FlagV, Flags, Out, Pre, R2,
};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ─── the read model: overlay + activeDecisions (decisions.mjs) ─────────────

/// JS Set over parsed JSON values (SameValueZero ≈ js_strict_eq here).
pub(crate) struct VSet(pub(crate) Vec<Value>);

impl VSet {
    pub(crate) fn new() -> Self {
        VSet(Vec::new())
    }
    pub(crate) fn add(&mut self, v: &Value) {
        if !self.has(v) {
            self.0.push(v.clone());
        }
    }
    pub(crate) fn has(&self, v: &Value) -> bool {
        self.0.iter().any(|x| js_strict_eq(x, v))
    }
    pub(crate) fn has_opt(&self, v: Option<&Value>) -> bool {
        v.map(|v| self.has(v)).unwrap_or(false)
    }
}

pub(crate) struct Patch {
    pub(crate) tags: Option<Value>,
    pub(crate) scope: Option<Value>,
}

/// provenance: decisions.mjs buildTagOverlay — latest tag event wins (date,
/// then file order). A mixed finite/NaN date set would feed V8's sort an
/// inconsistent comparator — Exotic.
pub(crate) fn build_tag_overlay(events: &[Value]) -> Ex<Vec<(Value, Patch)>> {
    let mut tag_events: Vec<(usize, &Value)> = Vec::new();
    for (idx, e) in events.iter().enumerate() {
        let is_tag = !e.is_null()
            && matches!(jget(e, "type"), Some(t) if v_is_str(t, "tag"))
            && matches!(jget(e, "target"), Some(Value::String(_)));
        if is_tag {
            tag_events.push((idx, e));
        }
    }
    let mut with_ms: Vec<(usize, &Value, Option<f64>)> = Vec::new();
    for (idx, e) in &tag_events {
        with_ms.push((*idx, e, date_parse_val(jget(e, "date"))?));
    }
    let finite = with_ms.iter().filter(|(_, _, m)| m.is_some()).count();
    if finite != 0 && finite != with_ms.len() {
        return Err(Exotic); // inconsistent comparator territory
    }
    if finite == with_ms.len() {
        with_ms.sort_by(|a, b| {
            a.2.partial_cmp(&b.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
    } // all-NaN: comparator always falls to idx — already in file order
    let mut overlay: Vec<(Value, Patch)> = Vec::new();
    for (_, e, _) in with_ms {
        let target = jget(e, "target").cloned().unwrap_or(Value::Null);
        let patch = Patch {
            tags: match jget(e, "tags") {
                Some(Value::Array(a)) => Some(Value::Array(a.clone())),
                _ => None,
            },
            scope: match jget(e, "scope") {
                Some(Value::String(s)) if !s.is_empty() => Some(Value::String(s.clone())),
                _ => None,
            },
        };
        if let Some(slot) = overlay.iter_mut().find(|(k, _)| js_strict_eq(k, &target)) {
            slot.1 = patch;
        } else {
            overlay.push((target, patch));
        }
    }
    Ok(overlay)
}

/// provenance: decisions.mjs applyTagOverlay — replaces tags wholesale,
/// scope only when the winning tag event carries one.
pub(crate) fn apply_tag_overlay(event: &Value, overlay: &[(Value, Patch)]) -> Value {
    let Some(id) = jget(event, "id") else {
        return event.clone();
    };
    let Some((_, patch)) = overlay.iter().find(|(k, _)| js_strict_eq(k, id)) else {
        return event.clone();
    };
    let Value::Object(m) = event else {
        return event.clone(); // unreachable: jget found a key ⇒ object
    };
    let mut next = m.clone();
    if let Some(tags) = &patch.tags {
        next.insert("tags".into(), tags.clone());
    }
    if let Some(scope) = &patch.scope {
        next.insert("scope".into(), scope.clone());
    }
    Value::Object(next)
}

pub(crate) fn is_decide_or_supersede(e: &Value) -> bool {
    matches!(jget(e, "type"), Some(t) if v_is_str(t, "decide") || v_is_str(t, "supersede"))
}

/// provenance: decisions.mjs activeDecisions (both branches; `recent` is
/// applied by the callers, matching the handlers).
pub(crate) fn active_decisions(root: &Path, all: bool) -> Ex<Vec<Value>> {
    let events = read_jsonl(&decisions_path(root));
    let overlay = build_tag_overlay(&events)?;
    if !all {
        if events.iter().any(|e| e.is_null()) {
            return Err(Exotic); // `event.type` on null throws in Node
        }
        let mut superseded = VSet::new();
        let mut redacted = VSet::new();
        for e in &events {
            if matches!(jget(e, "type"), Some(t) if v_is_str(t, "supersede")) {
                if let Some(s) = jget(e, "supersedes") {
                    if truthy(s) {
                        superseded.add(s);
                    }
                }
            }
            if matches!(jget(e, "type"), Some(t) if v_is_str(t, "redact")) {
                if let Some(r) = jget(e, "redacts") {
                    if truthy(r) {
                        redacted.add(r);
                    }
                }
            }
        }
        let mut active: Vec<&Value> = events
            .iter()
            .filter(|e| {
                is_decide_or_supersede(e)
                    && !superseded.has_opt(jget(e, "id"))
                    && !redacted.has_opt(jget(e, "id"))
            })
            .collect();
        active.reverse();
        return Ok(active.iter().map(|e| apply_tag_overlay(e, &overlay)).collect());
    }

    // --all: union with the archive, de-dup by id (active copy wins), then
    // an explicit date-desc sort with original-position tiebreak.
    let archived = read_jsonl(&decisions_archive_path(root));
    let mut by_id: Vec<(String, Value)> = Vec::new();
    for e in &events {
        if let Some(Value::String(id)) = jget(e, "id") {
            if let Some(slot) = by_id.iter_mut().find(|(k, _)| k == id) {
                slot.1 = e.clone();
            } else {
                by_id.push((id.clone(), e.clone()));
            }
        }
    }
    for e in &archived {
        if let Some(Value::String(id)) = jget(e, "id") {
            if !by_id.iter().any(|(k, _)| k == id) {
                by_id.push((id.clone(), e.clone()));
            }
        }
    }
    let evs: Vec<Value> = by_id.into_iter().map(|(_, v)| v).collect();
    let mut superseded = VSet::new();
    let mut redacted = VSet::new();
    for e in &evs {
        if matches!(jget(e, "type"), Some(t) if v_is_str(t, "supersede")) {
            if let Some(s) = jget(e, "supersedes") {
                if truthy(s) {
                    superseded.add(s);
                }
            }
        }
        if matches!(jget(e, "type"), Some(t) if v_is_str(t, "redact")) {
            if let Some(r) = jget(e, "redacts") {
                if truthy(r) {
                    redacted.add(r);
                }
            }
        }
    }
    let mut filtered: Vec<(usize, &Value, Option<f64>)> = Vec::new();
    for (idx, e) in evs.iter().enumerate() {
        if is_decide_or_supersede(e)
            && !superseded.has_opt(jget(e, "id"))
            && !redacted.has_opt(jget(e, "id"))
        {
            filtered.push((idx, e, date_parse_val(jget(e, "date"))?));
        }
    }
    let finite = filtered.iter().filter(|(_, _, m)| m.is_some()).count();
    if finite != 0 && finite != filtered.len() {
        return Err(Exotic);
    }
    if finite == filtered.len() {
        filtered.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.0.cmp(&a.0))
        });
    } else {
        filtered.sort_by(|a, b| b.0.cmp(&a.0)); // all-NaN: idx desc (== reverse)
    }
    Ok(filtered
        .into_iter()
        .map(|(_, e, _)| apply_tag_overlay(e, &overlay))
        .collect())
}

// ─── filters (bee.mjs filterDecisionEvents / matchesWholeToken) ────────────

#[derive(Default)]
pub(crate) struct DecisionFilters {
    pub(crate) text: Option<String>,
    pub(crate) tag: Option<String>,
    pub(crate) scope: Option<String>,
    pub(crate) since_ms: Option<f64>,
    pub(crate) untagged: bool,
    pub(crate) cell: Option<String>,
    pub(crate) feature: Option<String>,
}

pub(crate) fn char_ci_eq(a: char, b: char) -> bool {
    a == b || a.to_lowercase().eq(b.to_lowercase())
}

/// (?<![\w-])token(?![\w-]) case-insensitive — sqs-b1's hyphen-aware token
/// match, hand-scanned.
pub(crate) fn matches_whole_token(haystacks: &[String], token: &str) -> bool {
    let tok: Vec<char> = token.chars().collect();
    if tok.is_empty() {
        return false;
    }
    let not_word_dash = |c: char| !(is_word(c) || c == '-');
    for h in haystacks {
        let hc: Vec<char> = h.chars().collect();
        if hc.len() < tok.len() {
            continue;
        }
        for i in 0..=(hc.len() - tok.len()) {
            if !(0..tok.len()).all(|j| char_ci_eq(hc[i + j], tok[j])) {
                continue;
            }
            let before_ok = i == 0 || not_word_dash(hc[i - 1]);
            let after_ok = i + tok.len() == hc.len() || not_word_dash(hc[i + tok.len()]);
            if before_ok && after_ok {
                return true;
            }
        }
    }
    false
}

pub(crate) fn text_haystacks(event: &Value, include_tags: bool) -> Vec<String> {
    let mut fields: Vec<Option<&Value>> = vec![
        jget(event, "decision"),
        jget(event, "rationale"),
        jget(event, "alternatives"),
    ];
    if include_tags {
        if let Some(Value::Array(tags)) = jget(event, "tags") {
            for t in tags {
                fields.push(Some(t));
            }
        }
    }
    fields
        .into_iter()
        .flatten()
        .filter(|v| !matches!(v, Value::Null) && !matches!(v, Value::String(s) if s.is_empty()))
        .map(js_disp)
        .collect()
}

pub(crate) fn filter_decision_events(decisions: Vec<Value>, f: &DecisionFilters) -> Ex<Vec<Value>> {
    let mut result = decisions;
    if f.untagged {
        result.retain(|e| !matches!(jget(e, "tags"), Some(Value::Array(a)) if !a.is_empty()));
    }
    if let Some(cell) = &f.cell {
        result.retain(|e| matches_whole_token(&text_haystacks(e, false), cell));
    }
    if let Some(feature) = &f.feature {
        result.retain(|e| matches_whole_token(&text_haystacks(e, false), feature));
    }
    if let Some(tag) = &f.tag {
        let needle = tag.to_lowercase();
        result.retain(|e| {
            matches!(jget(e, "tags"), Some(Value::Array(tags)) if tags
                .iter()
                .any(|t| js_disp(t).to_lowercase() == needle))
        });
    }
    if let Some(scope) = &f.scope {
        let needle = scope.to_lowercase();
        result.retain(
            |e| matches!(jget(e, "scope"), Some(Value::String(s)) if s.to_lowercase() == needle),
        );
    }
    if let Some(since_ms) = f.since_ms {
        let mut kept = Vec::new();
        for e in result {
            let ms = date_parse_val(jget(&e, "date"))?;
            if matches!(ms, Some(m) if m >= since_ms) {
                kept.push(e);
            }
        }
        result = kept;
    }
    if let Some(text) = &f.text {
        let lowered = text.to_lowercase();
        let terms: Vec<&str> = lowered
            .split(js_is_ws)
            .filter(|t| !t.is_empty())
            .collect();
        let mut scored: Vec<(Value, usize)> = Vec::new();
        for e in result {
            let haystacks: Vec<String> = text_haystacks(&e, true)
                .into_iter()
                .map(|h| h.to_lowercase())
                .collect();
            let hits = terms
                .iter()
                .filter(|t| haystacks.iter().any(|h| h.contains(*t)))
                .count();
            if hits > 0 {
                scored.push((e, hits));
            }
        }
        scored.sort_by(|a, b| b.1.cmp(&a.1)); // stable: preserves date order on ties
        result = scored.into_iter().map(|(e, _)| e).collect();
    }
    Ok(result)
}
