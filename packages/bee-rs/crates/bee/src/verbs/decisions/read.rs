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
    js_numberify, js_quote, js_trim, keys_known, now_iso, parse_flags,
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

/// A Vec-backed set over parsed JSON values, deduplicated by native
/// `Value` equality (deep structural equality; JSON has no NaN literal, so
/// there is no SameValueZero edge case to model here).
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
        self.0.iter().any(|x| x == v)
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
        if let Some(slot) = overlay.iter_mut().find(|(k, _)| k == &target) {
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
    let Some((_, patch)) = overlay.iter().find(|(k, _)| k == id) else {
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

/// dsh-1 (decision-supersede-hygiene): the ids a `supersedes` field names, on
/// ANY event type — a bare string (the type=="supersede" shape) or an array
/// of strings (`decisions log --supersedes`'s shape). A non-string array
/// entry is dropped; a non-string non-array field contributes nothing.
pub(crate) fn supersedes_target_ids(e: &Value) -> Vec<Value> {
    match jget(e, "supersedes") {
        Some(Value::String(s)) if !s.is_empty() => vec![Value::String(s.clone())],
        Some(Value::Array(a)) => a
            .iter()
            .filter(|v| matches!(v, Value::String(s) if !s.is_empty()))
            .cloned()
            .collect(),
        _ => Vec::new(),
    }
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
            // dsh-1: a `supersedes` field excludes its targets on ANY event
            // type — not only type=="supersede" — so `decisions log
            // --supersedes` carries the same exclusion weight inline.
            for s in supersedes_target_ids(e) {
                superseded.add(&s);
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
        // dsh-1: same any-event-type `supersedes` collection as the default
        // branch above.
        for s in supersedes_target_ids(e) {
            superseded.add(&s);
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

// ─── dsh-1: `decisions log --supersedes` target resolution ─────────────────
//
// Deliberately narrower than verbs_write.rs's resolve_tag_target: that one
// resolves against the active+archive union (retro-tagging history is fine),
// this one resolves against the currently ACTIVE decide/supersede set only —
// an already-superseded or already-redacted target cannot be named again,
// which is the hygiene this cell is for.

/// (id, event) pairs for every event currently in the active set, in FILE
/// order (`active_decisions` itself returns newest-first for display; the
/// `.rev()` here undoes that single reversal so an ambiguity list reads the
/// same left-to-right order as resolve_tag_target's, which walks the raw
/// file directly).
pub(crate) fn active_decide_or_supersede_candidates(root: &Path) -> Ex<Vec<(String, Value)>> {
    let active = active_decisions(root, false)?;
    Ok(active
        .into_iter()
        .rev()
        .filter_map(|e| match jget(&e, "id").cloned() {
            Some(Value::String(id)) => Some((id, e)),
            _ => None,
        })
        .collect())
}

/// `Err` carries the refusal message. Full id or unique short8, same
/// matching shape as resolve_tag_target — worded for `decisions log
/// --supersedes` and scoped to `candidates` (the active set only).
pub(crate) fn resolve_supersedes_target(
    candidates: &[(String, Value)],
    raw: &str,
) -> Result<String, String> {
    let raw = js_trim(raw);
    if let Some((id, _)) = candidates.iter().find(|(id, _)| id == raw) {
        return Ok(id.clone());
    }
    // SHORT8_PATTERN /^[0-9a-f]{8}$/i
    let is_short8 = raw.chars().count() == 8 && raw.chars().all(|c| c.is_ascii_hexdigit());
    let mut matches: Vec<&String> = Vec::new();
    if is_short8 {
        let low = raw.to_ascii_lowercase();
        for (id, _) in candidates {
            if id.to_lowercase().starts_with(&low) {
                matches.push(id);
            }
        }
    }
    match matches.len() {
        1 => Ok(matches[0].clone()),
        n if n > 1 => {
            let list = matches.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
            Err(format!(
                "decisions log: --supersedes short id {} is ambiguous — matches {n} events ({list}); use the full id.",
                js_quote(raw)
            ))
        }
        _ => Err(format!(
            "decisions log: --supersedes target {} does not resolve to any active decide/supersede event.",
            js_quote(raw)
        )),
    }
}

// ─── slp-contract S3: the DERIVED contract status (D1, D2) ────────────────
//
// D1 locks that a contract's settled/unsettled status is a DERIVED view
// over the decision log — no registry, no reverse index, no cache, nothing
// stored. Everything below is a pure read: it answers from the events
// already in `.bee/decisions.jsonl` joined against the trigger records
// already in `.bee/triggers/`, and it writes nothing anywhere.
//
// The join is on SHORT8, because that is all a trigger record carries:
// `TriggerRecord.decision` (verbs/triggers/mod.rs) holds the first 8
// characters of the deferring decision's id, never the full id.

/// The derived status of ONE decision id, per D2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContractStatus {
    /// Active, and no trigger keyed to it is still open.
    Settled,
    /// Active, and a trigger keyed to it is `waiting` or `due`.
    Unsettled,
    /// Not in the active decision set at all. D3's word "retired" resolves
    /// here: the store has no `retired` state — only supersession, redaction
    /// and archiving — and all three drop the id out of `active_decisions`,
    /// which is exactly the condition to refuse on. A never-logged id lands
    /// here too, and that is correct: an id nobody logged settles nothing.
    Unknown,
}

/// The ids in the currently ACTIVE decide/supersede set, in file order —
/// one store read, so a caller checking many citations does not pay one
/// read per citation.
pub(crate) fn active_decision_ids(root: &Path) -> Ex<Vec<String>> {
    Ok(active_decide_or_supersede_candidates(root)?
        .into_iter()
        .map(|(id, _)| id)
        .collect())
}

/// Every `decision` key carried by a trigger that is still OPEN — status
/// `waiting` or `due`. Read-only by construction: it goes through
/// `triggers::read_without_evaluating`, which is `read_and_evaluate`'s own
/// body with the persisting predicate flip turned OFF, so this call leaves
/// every trigger file byte-identical.
///
/// Fail-open, inherited from that walk: no `.bee/triggers/` directory is
/// simply no open keys, and a corrupt or shape-invalid trigger file
/// contributes nothing rather than failing the read. A file that cannot be
/// parsed cannot say a decision is unsettled, so the decision reads as
/// settled — the same direction `triggers list` already degrades in.
pub(crate) fn open_trigger_decision_keys(root: &Path) -> Vec<String> {
    use crate::verbs::triggers::TriggerEntry;
    crate::verbs::triggers::read_without_evaluating(root)
        .into_iter()
        .filter_map(|e| match e {
            TriggerEntry::Ok(rec) if rec.status == "waiting" || rec.status == "due" => {
                Some(rec.decision)
            }
            _ => None,
        })
        .collect()
}

/// The derivation itself, over an already-read active set and an
/// already-read open-trigger key set — the same "caller brings the
/// candidates" shape `resolve_supersedes_target` above uses.
///
/// Three facts this join has to live with, all measured over the live
/// store rather than assumed:
///
/// - A `manual`-tier trigger NEVER reaches `due` (verbs/triggers/mod.rs) —
///   it only ever waits for a human, then gets `resolve`d. So a decision
///   with a waiting manual trigger stays `Unsettled` until a person
///   resolves that trigger. That is D2 working as locked, not a bug: the
///   revisit condition is attached and has not been answered.
/// - A trigger `decision` key that is not a short8 of any decision (the
///   live store carries `herding-`, `P72`, `p-c6e61d`, `wayfindi`) simply
///   matches nothing. It is never an error.
/// - Two decision ids sharing a short8 share every trigger keyed to it —
///   the record physically cannot say which of them it meant. Both read
///   `Unsettled`, which is the fail-safe direction for a refusal path.
///   Collisions among the live ids: 0.
///
/// The key comparison is ASCII-case-insensitive. Decision ids are
/// lowercase hex, so this only ever ADDS a match, and an extra match can
/// only move a decision toward `Unsettled` — the safe direction.
pub(crate) fn contract_status_over(
    active_ids: &[String],
    open_trigger_keys: &[String],
    id: &str,
) -> ContractStatus {
    let id = js_trim(id);
    if id.is_empty() || !active_ids.iter().any(|a| a == id) {
        return ContractStatus::Unknown;
    }
    let short8 = crate::textutil::truncate_chars_head(id, 8);
    if open_trigger_keys.iter().any(|k| k.eq_ignore_ascii_case(&short8)) {
        ContractStatus::Unsettled
    } else {
        ContractStatus::Settled
    }
}

/// `contract_status_over` with both reads performed for you — the
/// one-decision spelling.
// Still marked: S4's tripwire walks a WHOLE `cell.decisions` list, so it
// hoists the two reads itself and takes `contract_status_over` directly —
// paying three store reads per cell instead of three per citation. This
// spelling stays as the single-decision entry point (the derived-status
// tests drive it) and for the next caller that has one id and no list.
#[allow(dead_code)]
pub(crate) fn contract_status(root: &Path, id: &str) -> Ex<ContractStatus> {
    let active = active_decision_ids(root)?;
    Ok(contract_status_over(&active, &open_trigger_decision_keys(root), id))
}

/// One `cell.decisions` entry → the store decision it cites, or `None`.
///
/// The field does NOT hold store decision ids: measured over the 92 live
/// cells, 81 citations, only 11 resolve, and the entry-length histogram is
/// `{2: 61, 3: 5, 8: 11, 24: 1, 25: 3}` — it is dominated by LOCAL D-IDs
/// like `D1`, which point into a CONTEXT.md table. So `None` means "this
/// entry is not a store citation", never "this entry is wrong": a caller
/// passes over it silently.
///
/// Resolution, deliberately narrower than `resolve_supersedes_target`'s
/// (that one is a user-typed argument and may complain; this one walks a
/// record field and may not):
///
/// - an exact id match resolves;
/// - a prefix of AT LEAST 8 characters matching EXACTLY ONE candidate
///   resolves;
/// - anything shorter than 8 characters resolves to `None`, ambiguous or
///   not — `D1` must never resolve by accident;
/// - a prefix matching two or more candidates resolves to `None`, because
///   a guard cannot guess which one was meant.
///
/// The candidate set is the CALLER's, and which set it hands over decides
/// what the answer means. S4's tripwire (verbs/cells/handlers_write.rs)
/// passes the active+ARCHIVE union of decide/supersede ids, never the
/// active set alone: a superseded id is exactly the one D3 wants refused,
/// and against the active set it would resolve to `None` and be passed
/// over. Resolution answers "is this entry a store citation"; the ACTIVE
/// set's job is the separate question `contract_status_over` asks.
pub(crate) fn resolve_store_citation(active_ids: &[String], entry: &str) -> Option<String> {
    let raw = js_trim(entry);
    if raw.is_empty() {
        return None;
    }
    if let Some(id) = active_ids.iter().find(|id| id.as_str() == raw) {
        return Some(id.clone());
    }
    if raw.chars().count() < 8 {
        return None;
    }
    let low = raw.to_lowercase();
    let mut matches = active_ids.iter().filter(|id| id.to_lowercase().starts_with(&low));
    let first = matches.next()?;
    match matches.next() {
        None => Some(first.clone()),
        Some(_) => None,
    }
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
            let hits = count_term_hits(&haystacks, &terms);
            if hits > 0 {
                scored.push((e, hits));
            }
        }
        scored.sort_by(|a, b| b.1.cmp(&a.1)); // stable: preserves date order on ties
        result = scored.into_iter().map(|(e, _)| e).collect();
    }
    Ok(result)
}

/// dcc-1: the term-hit scorer `decisions search`'s `--text` filter uses
/// above — pulled out so `conflict_candidates` below can reuse the exact
/// same rule (count of lowercased terms each present as a substring of ANY
/// haystack) instead of a second copy.
pub(crate) fn count_term_hits(haystacks: &[String], terms: &[&str]) -> usize {
    terms
        .iter()
        .filter(|t| haystacks.iter().any(|h| h.contains(*t)))
        .count()
}

// ─── dcc-1 (decision-conflict-candidates): `decisions log` conflict hints ──

/// dcc-1: up to 3 ACTIVE events that might conflict with a just-logged (or
/// about-to-be-refused) decision — ranked by `count_term_hits` over
/// `new_text`'s whitespace-split lowercase terms, matched the same way
/// `decisions search --text` matches (decision/rationale/alternatives plus
/// overlay-applied tags, case-insensitive substring). A candidate qualifies
/// when it shares >=1 final tag with `new_tags` OR scores >=2 term hits;
/// `exclude_id` drops the just-written event itself out of its own active
/// set. Ties keep `active`'s newest-first order (stable sort).
pub(crate) fn conflict_candidates(
    active: &[Value],
    new_text: &str,
    new_tags: &[String],
    exclude_id: Option<&str>,
) -> Vec<Value> {
    let lowered = new_text.to_lowercase();
    let terms: Vec<&str> = lowered.split(js_is_ws).filter(|t| !t.is_empty()).collect();
    let new_tag_set: Vec<String> = new_tags.iter().map(|t| t.to_lowercase()).collect();

    let mut scored: Vec<(Value, usize)> = Vec::new();
    for e in active {
        if let (Some(exclude), Some(Value::String(id))) = (exclude_id, jget(e, "id")) {
            if id == exclude {
                continue;
            }
        }
        let haystacks: Vec<String> = text_haystacks(e, true)
            .into_iter()
            .map(|h| h.to_lowercase())
            .collect();
        let hits = count_term_hits(&haystacks, &terms);
        let shares_tag = match jget(e, "tags") {
            Some(Value::Array(tags)) => tags
                .iter()
                .any(|t| new_tag_set.iter().any(|nt| js_disp(t).to_lowercase() == *nt)),
            _ => false,
        };
        if shares_tag || hits >= 2 {
            scored.push((e.clone(), hits));
        }
    }
    scored.sort_by(|a, b| b.1.cmp(&a.1)); // stable: preserves active's newest-first order on ties
    scored.truncate(3);
    scored
        .into_iter()
        .map(|(e, hits)| {
            let id = js_disp_opt(jget(&e, "id"));
            let short8 = crate::textutil::truncate_chars_head(&id, 8);
            let excerpt = crate::textutil::truncate_chars_head(&js_disp_opt(jget(&e, "decision")), 90);
            json!({
                "id": id,
                "short8": short8,
                "date": js_disp_opt(jget(&e, "date")),
                "excerpt": excerpt,
                "hits": hits,
            })
        })
        .collect()
}

/// dcc-1: the human line format for a single conflict candidate — used in
/// both `decisions log`'s success output and the prose-supersession guard's
/// refusal message, so the fix command is ready-made either way.
pub(crate) fn conflict_candidate_line(c: &Value) -> String {
    let short8 = js_disp_opt(jget(c, "short8"));
    format!(
        "possible conflict: {short8} {} — if replaced, run decisions supersede --id {short8}",
        js_disp_opt(jget(c, "excerpt")),
    )
}

/// dcc-1: joins `conflict_candidate_line` for every candidate, each on its
/// own leading-`\n` line (empty string when `candidates` is empty, so
/// callers can `push_str` unconditionally).
pub(crate) fn conflict_candidate_lines(candidates: &[Value]) -> String {
    candidates
        .iter()
        .map(|c| format!("\n{}", conflict_candidate_line(c)))
        .collect()
}
