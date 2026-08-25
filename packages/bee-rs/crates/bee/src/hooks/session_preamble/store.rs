// sessions, cells, the capture queue, decisions and backlog counts
//
// Split out of the single 3.1k-line hooks/session_preamble.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, warn_corrupt_json, ReadJson};
use crate::jsjson;
use crate::state::{bypass_level, ship_visibility};
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use crate::version::BEE_VERSION;

// ─── cells (cells.mjs listCells / scribingDebt / globalScribingDebt / …) ───
//
// provenance: lib/cells.mjs l. 692-738 (listCells), 2019-2058 (scribingDebt),
// 2098-2187 (scribingRunStampMs/bestScribingStampMs/globalScribingDebt),
// 2280-2307 (tierMix/ceilingScarcityWarning); Rust lift of
// verbs/status_full.rs:1837-2000 and hooks/chain_nudge.rs:645-830.

/// cells.mjs listCells over the ACTIVE dir only (no caller here passes
/// includeArchived), sorted by numeric-aware id compare.
pub(crate) fn list_cells(root: &Path, feature: Option<&Value>, status: Option<&str>) -> Vec<JMap> {
    let dir = root.join(".bee").join("cells");
    let Ok(rd) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut cells: Vec<JMap> = Vec::new();
    for entry in rd.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue; // the `archive` child (or any dir) is never a cell
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        let Some(cell) = read_json_object(&entry.path()) else { continue };
        // JS `if (feature && cell.feature !== feature) continue`.
        if let Some(f) = feature.filter(|f| truthy(f)) {
            if !strict_eq(cell.get("feature"), Some(f)) {
                continue;
            }
        }
        if let Some(s) = status {
            if !str_eq(cell.get("status"), s) {
                continue;
            }
        }
        cells.push(cell);
    }
    cells.sort_by(|a, b| {
        locale_cmp(&tpl(a.get("id")), &tpl(b.get("id")), true)
    });
    cells
}

/// Archive-aware sibling of `list_cells`, for debt-door-archive dda-2: `bee
/// close` archives a feature's cells on a green close
/// (`.bee/cells/archive/<feature>/*.json`), so a debt counter that only
/// walks the live store the way `list_cells` does goes structurally silent
/// the moment its own feature closes. This reads the live store (exactly as
/// `list_cells` does) THEN every archived `.json` under
/// `.bee/cells/archive/<feature>/` (a truthy `feature` filter) or under
/// every `.bee/cells/archive/*/` subdirectory (no filter — the global orphan
/// sweep needs every feature's archive slot), deduplicating by id with the
/// LIVE copy winning on a duplicate — the same live-copy-wins pattern
/// `verbs/drivers/guard.rs:218 list_cells_including_archive` already uses
/// for `bee close`'s own door. `list_cells` itself is untouched and stays
/// active-only. Only `scribing_debt` and `global_scribing_debt` below call
/// this variant.
pub(crate) fn list_cells_including_archive(
    root: &Path,
    feature: Option<&Value>,
    status: Option<&str>,
) -> Vec<JMap> {
    let mut cells = list_cells(root, feature, status);
    let mut seen_ids: HashSet<String> = cells.iter().map(|c| tpl(c.get("id"))).collect();
    let archive_root = root.join(".bee").join("cells").join("archive");
    let feature_dirs: Vec<PathBuf> = match feature.filter(|f| truthy(f)) {
        Some(f) => vec![archive_root.join(jsjson::js_to_string(f))],
        None => {
            let Ok(rd) = std::fs::read_dir(&archive_root) else { return cells };
            rd.filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .map(|e| e.path())
                .collect()
        }
    };
    for dir in feature_dirs {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for entry in rd.filter_map(|e| e.ok()) {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            let Some(cell) = read_json_object(&entry.path()) else { continue };
            if let Some(f) = feature.filter(|f| truthy(f)) {
                if !strict_eq(cell.get("feature"), Some(f)) {
                    continue;
                }
            }
            if let Some(s) = status {
                if !str_eq(cell.get("status"), s) {
                    continue;
                }
            }
            let id = tpl(cell.get("id"));
            if !seen_ids.insert(id) {
                continue; // the live copy above already claimed this id
            }
            cells.push(cell);
        }
    }
    cells.sort_by(|a, b| locale_cmp(&tpl(a.get("id")), &tpl(b.get("id")), true));
    cells
}

/// cells.mjs scribingRunStampMs: Date.parse(run.at || run.date).
pub(crate) fn scribing_run_stamp_ms(run: Option<&Value>) -> Option<f64> {
    let run = run?;
    if !truthy(run) {
        return None;
    }
    let at = vget(run, "at").filter(|v| truthy(v));
    let candidate = match at {
        Some(v) => Some(v),
        None => vget(run, "date"),
    };
    let parsed = date_parse_val(candidate);
    parsed.is_finite().then_some(parsed)
}

/// cells.mjs bestScribingStampMs — ledger max, then the feature's own lane
/// stamp, then the default record's stamp (only when it names this feature).
pub(crate) fn best_scribing_stamp_ms(
    root: &Path,
    feature: &Value,
    ledger: &[Value],
    state: &JMap,
) -> Option<f64> {
    let mut best: Option<f64> = None;
    let mut consider = |ms: Option<f64>| {
        if let Some(v) = ms {
            if best.is_none() || v > best.unwrap() {
                best = Some(v);
            }
        }
    };
    for entry in ledger {
        if !truthy(entry) || !strict_eq(vget(entry, "feature"), Some(feature)) {
            continue;
        }
        let parsed = date_parse_val(vget(entry, "ts"));
        consider(parsed.is_finite().then_some(parsed));
    }
    if let Some(lane) = read_lane(root, Some(feature)) {
        consider(scribing_run_stamp_ms(lane.get("last_scribing_run")));
    }
    if let Some(run) = state.get("last_scribing_run") {
        if truthy(run) && strict_eq(vget(run, "feature"), Some(feature)) {
            consider(scribing_run_stamp_ms(Some(run)));
        }
    }
    best
}

pub(crate) fn read_scribing_ledger(root: &Path) -> Vec<Value> {
    read_jsonl(&root.join(".bee").join("logs").join("scribing-runs.jsonl"))
}

/// cells.mjs scribingDebt(root) — no opts on the preamble path.
///
/// trun-9 rework (D5): this is the preamble's OWN copy of the scribing-debt
/// scan — what the capture-pending line in the session preamble actually
/// reads (via `hooks::session_preamble::budget`) — separate from
/// `drivers/close.rs::scribing_debt`, which close's own door calls. The
/// first pass at this cell wired the queue into close's copy only, which the
/// semantic judge caught: completing a `scribe` deferred-queue record
/// cleared close's door but left this line still reporting the debt. Both
/// copies now read the SAME queue fold (`state_group::scribe_queue_cells`)
/// and decide with the SAME OR rule (`state_group::deferred_debt_cleared`),
/// so a completed `scribe` record clears the debt everywhere, not only at
/// `bee close`. This scan never materializes a missing record itself —
/// `drivers/close.rs::scribing_debt` already is the one place that does,
/// and every relevant mutation calls that scan too, so a record exists by
/// the time this read-only preamble path runs.
pub(crate) fn scribing_debt(root: &Path) -> (usize, Vec<Value>) {
    let state = read_state(root);
    let feature = state.get("feature").cloned().unwrap_or(Value::Null);
    if !truthy(&feature) {
        return (0, Vec::new());
    }
    let ledger = read_scribing_ledger(root);
    let threshold = best_scribing_stamp_ms(root, &feature, &ledger, &state).unwrap_or(0.0);
    let feature_str = jsjson::js_to_string(&feature);
    let completed_cells = crate::verbs::state_group::scribe_queue_cells(root, &feature_str).completed;
    let mut ids = Vec::new();
    // dda-2: archive-aware, so a feature that just closed (and got archived)
    // still shows its unpaid debt instead of going structurally silent.
    for cell in list_cells_including_archive(root, Some(&feature), Some("capped")) {
        let trace = match cell.get("trace") {
            Some(v) if truthy(v) => v.clone(),
            _ => json!({}),
        };
        if vget(&trace, "behavior_change") != Some(&Value::Bool(true)) {
            continue;
        }
        let capped_at = date_parse_val(vget(&trace, "capped_at"));
        let legacy_cleared = !(capped_at.is_finite() && capped_at > threshold);
        let id_str = cell.get("id").and_then(Value::as_str).unwrap_or("").to_string();
        let queue_completed = !id_str.is_empty() && completed_cells.contains(&id_str);
        if crate::verbs::state_group::deferred_debt_cleared(legacy_cleared, queue_completed) {
            continue;
        }
        ids.push(cell.get("id").cloned().unwrap_or(Value::Null));
    }
    (ids.len(), ids)
}

/// cells.mjs globalScribingDebt — the orphan sweep across every feature.
///
/// trun-9 rework (D5): reconciled through the same shared queue read as
/// `scribing_debt` just above — a feature's orphaned entry here is exactly
/// what the "### Orphaned scribing debt" preamble section renders, so it is
/// covered by the same must_have. `completed_cache` mirrors `stamp_cache`'s
/// per-feature memoization; unlike the stamp, the queue fold is per-feature
/// too (`state_group::scribe_queue_cells`), so it is computed once per
/// feature the loop actually visits, not once per cell.
pub(crate) fn global_scribing_debt(root: &Path) -> (usize, Vec<(String, Vec<Value>)>) {
    // dda-2: archive-aware. An orphaned feature's cells are exactly the ones
    // most likely to have been archived already, so the sweep must see them.
    let cells: Vec<JMap> = list_cells_including_archive(root, None, Some("capped"))
        .into_iter()
        .filter(|cell| {
            let trace = match cell.get("trace") {
                Some(v) if truthy(v) => v.clone(),
                _ => json!({}),
            };
            vget(&trace, "behavior_change") == Some(&Value::Bool(true))
        })
        .collect();
    if cells.is_empty() {
        return (0, Vec::new());
    }
    let state = read_state(root);
    let ledger = read_scribing_ledger(root);
    let mut stamp_cache: HashMap<String, Option<f64>> = HashMap::new();
    let mut completed_cache: HashMap<String, HashSet<String>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut by_feature: HashMap<String, Vec<Value>> = HashMap::new();
    for cell in &cells {
        let Some(feature_v) = cell.get("feature").filter(|f| truthy(f)) else { continue };
        let key = jsjson::js_to_string(feature_v);
        let trace = match cell.get("trace") {
            Some(v) if truthy(v) => v.clone(),
            _ => json!({}),
        };
        let capped_at = date_parse_val(vget(&trace, "capped_at"));
        let stamp = match stamp_cache.get(&key) {
            Some(s) => *s,
            None => {
                let s = best_scribing_stamp_ms(root, feature_v, &ledger, &state);
                stamp_cache.insert(key.clone(), s);
                s
            }
        };
        let legacy_cleared = match stamp {
            None => false,
            Some(s) => !(capped_at.is_finite() && capped_at > s),
        };
        let completed = completed_cache
            .entry(key.clone())
            .or_insert_with(|| crate::verbs::state_group::scribe_queue_cells(root, &key).completed);
        let id_str = cell.get("id").and_then(Value::as_str).unwrap_or("").to_string();
        let queue_completed = !id_str.is_empty() && completed.contains(&id_str);
        if crate::verbs::state_group::deferred_debt_cleared(legacy_cleared, queue_completed) {
            continue;
        }
        if !by_feature.contains_key(&key) {
            order.push(key.clone());
            by_feature.insert(key.clone(), Vec::new());
        }
        by_feature
            .get_mut(&key)
            .unwrap()
            .push(cell.get("id").cloned().unwrap_or(Value::Null));
    }
    // .sort((a, b) => a.feature.localeCompare(b.feature, 'en')) — non-numeric.
    order.sort_by(|a, b| locale_cmp(a, b, false));
    let mut features = Vec::new();
    let mut count = 0usize;
    for feature in order {
        let ids = by_feature.remove(&feature).unwrap_or_default();
        count += ids.len();
        features.push((feature, ids));
    }
    (count, features)
}

/// Is this cell escalated onto the session model? (D5, store 97ce5225.)
///
/// A LIFT of `verbs/cells/validate.rs`'s `cell_is_escalated`, not a second
/// implementation — one flag, one legacy spelling — following the precedent
/// `verbs/cells/handlers_close.rs` set at this same module boundary. The
/// published predicate takes a `&Value`; every cell here is already a `JMap`
/// and cloning ~500 of them per preamble render to satisfy the signature
/// would be the wrong trade. What it must NOT do is restate the field NAMES,
/// so it reads the same two constants the published predicate reads: a rename
/// there fails the build here rather than silently zeroing this counter.
fn map_is_escalated(cell: &JMap) -> bool {
    match cell.get(crate::verbs::cells::ESCALATE_FIELD) {
        Some(Value::Bool(true)) => true,
        // escalate-off-disarm D1: mirror of cell_is_escalated's
        // explicit-false arm — a recorded disarm outranks the legacy
        // spelling here too, or the preamble counts a cell the ration
        // no longer does.
        Some(Value::Bool(false)) => false,
        _ => matches!(
            cell.get("tier"),
            Some(Value::String(t)) if t == crate::verbs::drivers::ESCALATION_WORD
        ),
    }
}

/// cells.mjs ceilingScarcityWarning -> (pct, escalated, cells).
///
/// D6 (store 97ce5225) moved both halves of this arithmetic, and the second
/// one is a correction rather than a translation — the same one
/// `handlers_close.rs`'s ration took (store b39d045f).
///
/// WHICH cells count: the escalation flag, with the legacy `tier: "ceiling"`
/// spelling still honoured. Before this change a cell carrying
/// `escalate: true` and no `tier` at all was INVISIBLE here — the enforcing
/// door already read the flag while this advice line still watched a tier
/// value, so the preamble could report all-clear on a feature the ration
/// would have refused.
///
/// WHAT they are counted against: the feature's cells, full stop. It used to
/// be "cells that recorded a tier at all", which is 0 for every cell authored
/// under D7's required `role`. Left alone, the denominator would go to zero,
/// `share` would be 0.0, and this warning would report all-clear forever —
/// which is worse than reporting nothing, because nobody looks at a section
/// that never appears.
pub(crate) fn ceiling_scarcity_warning(root: &Path) -> Option<(f64, i64, i64)> {
    let state = read_state(root);
    let feature = state.get("feature").cloned().unwrap_or(Value::Null);
    let filter = if truthy(&feature) { Some(feature) } else { None };
    let cells = list_cells(root, filter.as_ref(), None);
    let (mut escalated, mut counted) = (0i64, 0i64);
    for cell in &cells {
        counted += 1;
        if map_is_escalated(cell) {
            escalated += 1;
        }
    }
    if counted < SCARCITY_MIN_CELLS {
        return None;
    }
    let share = if counted > 0 { escalated as f64 / counted as f64 } else { 0.0 };
    if share <= CEILING_MAX_SHARE {
        return None;
    }
    Some((js_round(share * 100.0), escalated, counted))
}

// ─── capture queue (capture.mjs pendingCaptureStubs / captureQueue) ────────
//
// provenance: lib/capture.mjs l. 85-103; Rust lift of
// verbs/status_full.rs:2463-2490 (only the count is read here).

pub(crate) fn capture_queue_count(root: &Path) -> usize {
    let events = read_jsonl(&root.join(".bee").join("capture-queue.jsonl"));
    let mut flushed: Vec<Value> = Vec::new();
    let mut stubs: Vec<&Value> = Vec::new();
    for event in &events {
        if !matches!(event, Value::Object(_)) {
            continue;
        }
        if str_eq(vget(event, "kind"), "flush") && opt_truthy(vget(event, "id")) {
            flushed.push(vget(event, "id").unwrap().clone());
        } else if str_eq(vget(event, "kind"), "stub") && opt_truthy(vget(event, "id")) {
            stubs.push(event);
        }
    }
    stubs
        .into_iter()
        .filter(|s| !flushed.iter().any(|f| strict_eq(Some(f), vget(s, "id"))))
        .count()
}

// ─── decisions (decisions.mjs activeDecisions / datamark) ──────────────────
//
// provenance: lib/decisions.mjs l. 810-838 (default branch — the preamble
// never passes `all`) and l. 1047-1054 (datamark); Rust lift of
// verbs/status_full.rs:2280-2440.

pub(crate) fn decisions_path(root: &Path) -> PathBuf {
    root.join(".bee").join("decisions.jsonl")
}

/// decisions.mjs buildTagOverlay — latest tag event wins (date, then file
/// order). A mixed finite/NaN date set delegated in the verb port; here the
/// NaN rows simply keep file order, which is what V8's sort does for them in
/// practice and is the fail-open direction for an orientation block.
pub(crate) fn build_tag_overlay(events: &[Value]) -> Vec<(Value, (Option<Value>, Option<Value>))> {
    let mut tag_events: Vec<(usize, &Value, f64)> = Vec::new();
    for (idx, e) in events.iter().enumerate() {
        if truthy(e)
            && str_eq(vget(e, "type"), "tag")
            && matches!(vget(e, "target"), Some(Value::String(_)))
        {
            tag_events.push((idx, e, date_parse_val(vget(e, "date"))));
        }
    }
    tag_events.sort_by(|a, b| {
        let (x, y) = (a.2, b.2);
        let ord = match (x.is_finite(), y.is_finite()) {
            (true, true) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
            _ => Ordering::Equal,
        };
        ord.then(a.0.cmp(&b.0))
    });
    let mut overlay: Vec<(Value, (Option<Value>, Option<Value>))> = Vec::new();
    for (_, e, _) in tag_events {
        let target = vget(e, "target").cloned().unwrap_or(Value::Null);
        let patch = (
            match vget(e, "tags") {
                Some(Value::Array(a)) => Some(Value::Array(a.clone())),
                _ => None,
            },
            match vget(e, "scope") {
                Some(Value::String(s)) if !s.is_empty() => Some(Value::String(s.clone())),
                _ => None,
            },
        );
        if let Some(slot) = overlay.iter_mut().find(|(k, _)| strict_eq(Some(k), Some(&target))) {
            slot.1 = patch;
        } else {
            overlay.push((target, patch));
        }
    }
    overlay
}

pub(crate) fn apply_tag_overlay(
    event: &Value,
    overlay: &[(Value, (Option<Value>, Option<Value>))],
) -> Value {
    let Some(id) = vget(event, "id") else { return event.clone() };
    let Some((_, (tags, scope))) = overlay.iter().find(|(k, _)| strict_eq(Some(k), Some(id))) else {
        return event.clone();
    };
    let Value::Object(m) = event else { return event.clone() };
    let mut next = m.clone();
    if let Some(tags) = tags {
        next.insert("tags".into(), tags.clone());
    }
    if let Some(scope) = scope {
        next.insert("scope".into(), scope.clone());
    }
    Value::Object(next)
}

/// decisions.mjs activeDecisions(root, { recent }) — default branch only.
pub(crate) fn active_decisions(root: &Path, recent: Option<usize>) -> Vec<Value> {
    let events = read_jsonl(&decisions_path(root));
    let overlay = build_tag_overlay(&events);
    let mut superseded: Vec<Value> = Vec::new();
    let mut redacted: Vec<Value> = Vec::new();
    for event in &events {
        if str_eq(vget(event, "type"), "supersede") && opt_truthy(vget(event, "supersedes")) {
            superseded.push(vget(event, "supersedes").unwrap().clone());
        }
        if str_eq(vget(event, "type"), "redact") && opt_truthy(vget(event, "redacts")) {
            redacted.push(vget(event, "redacts").unwrap().clone());
        }
    }
    let in_set = |set: &[Value], id: Option<&Value>| set.iter().any(|v| strict_eq(Some(v), id));
    let mut active: Vec<Value> = events
        .iter()
        .filter(|event| {
            let ty = vget(event, "type");
            (str_eq(ty, "decide") || str_eq(ty, "supersede"))
                && !in_set(&superseded, vget(event, "id"))
                && !in_set(&redacted, vget(event, "id"))
        })
        .cloned()
        .collect();
    active.reverse();
    let mut out: Vec<Value> = active.iter().map(|e| apply_tag_overlay(e, &overlay)).collect();
    if let Some(n) = recent {
        out.truncate(n);
    }
    out
}

/// decisions.mjs datamark — neutralize resurfaced text.
pub(crate) fn datamark(text: Option<&Value>) -> String {
    let s = match text {
        None | Some(Value::Null) => String::new(),
        Some(v) => jsjson::js_to_string(v),
    };
    // .replace(/```+/g, '')
    let chars: Vec<char> = s.chars().collect();
    let mut no_ticks = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '`' {
            let mut j = i;
            while j < chars.len() && chars[j] == '`' {
                j += 1;
            }
            if j - i < 3 {
                for k in i..j {
                    no_ticks.push(chars[k]);
                }
            }
            i = j;
            continue;
        }
        no_ticks.push(chars[i]);
        i += 1;
    }
    let no_tags = strip_role_tags(&no_ticks);
    let cleaned: String = no_tags
        .chars()
        .filter(|c| {
            let cp = *c as u32;
            !(cp <= 0x08 || cp == 0x0B || cp == 0x0C || (0x0E..=0x1F).contains(&cp) || cp == 0x7F)
        })
        .collect();
    format!("«{}»", js_trim(&cleaned))
}

/// `/<\/?\s*(?:system|assistant|user|developer|tool)\b[^>]*>/gi`
pub(crate) fn strip_role_tags(s: &str) -> String {
    const ROLES: [&str; 5] = ["system", "assistant", "user", "developer", "tool"];
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    'outer: while i < chars.len() {
        if chars[i] == '<' {
            let mut j = i + 1;
            if j < chars.len() && chars[j] == '/' {
                j += 1;
            }
            while j < chars.len() && js_is_space(chars[j]) {
                j += 1;
            }
            for role in ROLES {
                let rl: Vec<char> = role.chars().collect();
                if j + rl.len() <= chars.len()
                    && chars[j..j + rl.len()]
                        .iter()
                        .zip(rl.iter())
                        .all(|(a, b)| a.to_ascii_lowercase() == *b)
                {
                    let after = j + rl.len();
                    // \b — the role name must not run straight into a word char.
                    let boundary = after >= chars.len()
                        || !(chars[after].is_alphanumeric() || chars[after] == '_');
                    if boundary {
                        let mut k = after;
                        while k < chars.len() && chars[k] != '>' {
                            k += 1;
                        }
                        if k < chars.len() {
                            i = k + 1;
                            continue 'outer;
                        }
                    }
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

// ─── backlog counts (backlog.mjs readBacklogCounts) ────────────────────────
//
// provenance: lib/backlog.mjs foldPbis/foldedBacklogCounts/
// legacyBacklogCounts; Rust lift of verbs/status_full.rs:2512-2632.

/// backlog.mjs tokenKey: 'in-flight' -> 'inFlight'.
pub(crate) fn token_key(token: &str) -> String {
    let mut out = String::new();
    let mut upper_next = false;
    for c in token.chars() {
        if c == '-' {
            upper_next = true;
        } else if upper_next {
            out.extend(c.to_uppercase());
            upper_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

pub(crate) fn read_backlog_counts(root: &Path) -> Option<JMap> {
    let text = read_text_opt(&root.join(".bee").join("backlog.jsonl"));
    let mut has_events = false;
    let mut items: HashMap<String, String> = HashMap::new();
    if let Some(text) = text {
        for line in text.split('\n') {
            let line = line.strip_suffix('\r').unwrap_or(line);
            let trimmed = js_trim(line);
            if trimmed.is_empty() {
                continue;
            }
            let Ok(row) = serde_json::from_str::<Value>(trimmed) else { continue };
            if !matches!(row, Value::Object(_)) || !str_eq(vget(&row, "kind"), "pbi") {
                continue;
            }
            has_events = true;
            let id = match vget(&row, "id") {
                Some(Value::String(s)) if !s.is_empty() => s.clone(),
                _ => continue,
            };
            match vget(&row, "event").and_then(|v| v.as_str()).unwrap_or("") {
                "add" => {
                    if items.contains_key(&id) {
                        continue;
                    }
                    let status = match vget(&row, "status").and_then(|v| v.as_str()) {
                        Some(s) if PBI_STATUSES.contains(&s) => s.to_string(),
                        _ => "proposed".to_string(),
                    };
                    items.insert(id, status);
                }
                "status" => {
                    if let Some(item) = items.get_mut(&id) {
                        if let Some(s) = vget(&row, "status").and_then(|v| v.as_str()) {
                            if PBI_STATUSES.contains(&s) {
                                *item = s.to_string();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if has_events {
        let mut counts = JMap::new();
        for status in PBI_STATUSES {
            counts.insert(token_key(status), json!(0));
        }
        let mut total = 0i64;
        for status in items.values() {
            if PBI_STATUSES.contains(&status.as_str()) {
                let key = token_key(status);
                let n = counts.get(&key).and_then(|v| v.as_i64()).unwrap_or(0) + 1;
                counts.insert(key, json!(n));
                total += 1;
            }
        }
        counts.insert("total".into(), json!(total));
        return Some(counts);
    }
    // legacyBacklogCounts over <productRoot>/docs/backlog.md.
    let file = resolve_product_root(root).join("docs").join("backlog.md");
    let text = read_text_opt(&file)?;
    let mut counts = JMap::new();
    for status in BACKLOG_STATUSES {
        counts.insert(token_key(status), json!(0));
    }
    let normalize_status = |cell: &str| -> String {
        cell.chars()
            .filter(|c| !matches!(c, '*' | '`' | '_'))
            .collect::<String>()
            .trim()
            .to_lowercase()
    };
    let split_row = |line: &str| -> Vec<String> {
        let mut cells: Vec<String> = line.split('|').map(|c| js_trim(c).to_string()).collect();
        if cells.first().map(|c| c.is_empty()).unwrap_or(false) {
            cells.remove(0);
        }
        if cells.last().map(|c| c.is_empty()).unwrap_or(false) {
            cells.pop();
        }
        cells
    };
    let mut status_index: Option<usize> = None;
    for raw_line in text.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if !line.contains('|') {
            continue;
        }
        let cells = split_row(line);
        match status_index {
            None => {
                if let Some(idx) = cells.iter().position(|c| normalize_status(c) == "status") {
                    status_index = Some(idx);
                }
            }
            Some(idx) => {
                if cells.len() <= idx {
                    continue;
                }
                let token = normalize_status(&cells[idx]);
                if BACKLOG_STATUSES.contains(&token.as_str()) {
                    let key = token_key(&token);
                    let n = counts.get(&key).and_then(|v| v.as_i64()).unwrap_or(0) + 1;
                    counts.insert(key, json!(n));
                }
            }
        }
    }
    let total: i64 = BACKLOG_STATUSES
        .iter()
        .map(|s| counts.get(&token_key(s)).and_then(|v| v.as_i64()).unwrap_or(0))
        .sum();
    counts.insert("total".into(), json!(total));
    Some(counts)
}
