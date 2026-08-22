// `state plan-conflicts derive` / `state plan-conflicts verdict` — D5's
// plan-time conflict check (knowledge-one-home, cell koh-8).
//
// WHY THIS FILE EXISTS. D5 locks the shape: "bee derives candidates from the
// plan (cell titles, touched paths, area tags) via `decisions active` and
// `applied_at` (touched path -> rules homed there). Each candidate gets a
// verdict recorded on the plan: compatible / conflicts / retires-prior.
// '0 conflicts' is valid only when bee returned zero candidates." This module
// is the derivation and the verdict record ONLY. The `gate --merge`
// precondition that reads what is written here is a separate cell (koh-9) by
// explicit scope boundary — nothing in this file refuses a gate.
//
// WHAT IS REUSED, NOT REBUILT. The candidate rule is not a new heuristic:
//   * decisions — `decisions::conflict_candidates` (the same ranked top-3
//     `decisions log` already prints as conflict hints) UNION every active
//     decision whose `decisions::count_term_hits` score against the same term
//     set is >= 2 (the truncation is `log`'s output budget, not a statement
//     about which decisions the plan touches, so the full >=2 set is taken).
//     Neither scorer is modified here.
//   * rules — `knowledge::ownership::load_ownership`'s rule homes, selected
//     when the rule's own home path or any of its `applied_at` patterns
//     intersects the plan's touched paths (`knowledge::matches_owned`, both
//     directions, because a cell's `affects_skills`/`affects_specs` entries
//     are themselves patterns).
//
// THE TERM SET. Built from the feature's OPEN and CAPPED cells (dropped and
// blocked cells are not plan surface): every cell title, plus the stem of
// every path in `files`, `affects_skills` and `affects_specs`, split further
// on `_`/`-`/`.` so `plan_conflicts.rs` also contributes `plan` and
// `conflicts`. Terms are lowercased, stripped of punctuation, and filtered to
// length >= 4 minus a small stop list — WITHOUT that filter a title's "the"
// and "a" alone reach `count_term_hits`'s >= 2 threshold against nearly every
// decision ever logged, which makes "0 conflicts" unreachable and the whole
// check noise. The filter is a property of BUILDING the term set, not of the
// scoring rule, which is untouched.
//
// LOCKING. Both verbs are lane-targeted exactly like `plan-rev bump` and take
// the same locks in the same order (`workflow:<id>` first, then the
// projection lock), with the same "the target changed while this call was
// starting" re-check. `conflict_review` lives on the workflow record beside
// `plan_rev` (seeded ABSENT — a record that has never been derived against
// says so by having no field at all, which is exactly what koh-9's door needs
// to tell "never derived" from "derived and empty").

#![allow(unused_imports)]

use super::*;
use crate::fsutil::{
    append_jsonl, ensure_dir, read_json, warn_corrupt_json, write_json_atomic, ReadJson,
};
use crate::jsjson;
use crate::lock::{self, AcquireOnce, LockGuard};
use crate::verbs::cells::list_cells;
use crate::verbs::decisions::{active_decisions, conflict_candidates, count_term_hits, text_haystacks};
use crate::verbs::knowledge::{load_ownership, matches_owned, RuleHome};
use crate::verbs::reservations::{
    date_parse_val, finish, iso_from_ms, jget, js_disp, js_disp_opt,
    js_numberify, js_trim, keys_known, now_iso, now_ms, parse_flags, prelude, truthy,
    Ctx, Err2, Ex, Exotic, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::workflow_store::{
    acquire_named_lock, acquire_workflow_lock, find_live_workflow, list_workflows,
    projection_lock_name, rebuild_lane_projection, update_workflow_assuming_lock_with, wf_id,
};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

/// D5's closed verdict vocabulary. `retires-prior` is one value, not a value
/// plus an id: the id it retires belongs in `--note`, so the vocabulary stays
/// checkable by equality.
pub(crate) const CONFLICT_VERDICTS: [&str; 3] = ["compatible", "conflicts", "retires-prior"];

/// The cell statuses that count as PLAN surface. Open work is what the plan
/// still promises; capped work is what it already changed — both are things a
/// standing decision or homed rule can conflict with. Dropped and blocked
/// cells are neither.
const PLAN_CELL_STATUSES: [&str; 2] = ["open", "capped"];

/// Short, high-frequency English words that would otherwise saturate
/// `count_term_hits`. Only terms of length >= 4 survive the filter at all, so
/// this list only has to cover the four-letter-and-up fillers.
const TERM_STOPWORDS: [&str; 24] = [
    "with", "that", "this", "from", "into", "when", "then", "than", "they", "them", "their",
    "there", "here", "have", "has", "does", "each", "also", "over", "under", "onto", "same",
    "such", "only",
];

// ─── the term set and the touched-path set ──────────────────────────────────

/// The feature's plan surface: every OPEN or CAPPED cell record.
pub(crate) fn plan_cells(root: &Path, feature: &str) -> Result<Vec<Value>, Err2> {
    let mut out: Vec<Value> = Vec::new();
    for status in PLAN_CELL_STATUSES {
        let cells = list_cells(root, Some(feature), Some(status)).map_err(|_| Err2::Ex)?;
        out.extend(cells);
    }
    Ok(out)
}

/// Every string in a cell's array field, ignoring a missing or non-array one.
fn string_list(cell: &Value, key: &str) -> Vec<String> {
    match jget(cell, key) {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str())
            .map(str::to_string)
            .filter(|s| !s.trim().is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// `files` + `affects_skills` + `affects_specs`, normalized to forward
/// slashes with any leading `/` dropped — the shape `matches_owned` compares
/// against.
pub(crate) fn plan_paths(cells: &[Value]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for cell in cells {
        for key in ["files", "affects_skills", "affects_specs"] {
            for raw in string_list(cell, key) {
                let norm = raw.replace('\\', "/").trim_start_matches('/').to_string();
                if !norm.is_empty() && !out.contains(&norm) {
                    out.push(norm);
                }
            }
        }
    }
    out
}

/// One path's stem plus its `_`/`-`/`.`-separated pieces: `plan_conflicts.rs`
/// contributes `plan_conflicts`, `plan` and `conflicts`; `gates.md`
/// contributes `gates`.
fn path_stem_terms(path: &str) -> Vec<String> {
    let last = path.rsplit('/').next().unwrap_or(path);
    let stem = match last.rfind('.') {
        Some(i) if i > 0 => &last[..i],
        _ => last,
    };
    let mut out = vec![stem.to_string()];
    for piece in stem.split(['_', '-', '.']) {
        if !piece.is_empty() {
            out.push(piece.to_string());
        }
    }
    out
}

/// Keeps a raw word only if it survives normalization into a scoreable term:
/// lowercased, stripped of surrounding punctuation, at least four characters,
/// and not a stop word.
fn normalize_term(raw: &str) -> Option<String> {
    let t: String = raw
        .to_lowercase()
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_string();
    if t.chars().count() < 4 {
        return None;
    }
    if TERM_STOPWORDS.contains(&t.as_str()) {
        return None;
    }
    Some(t)
}

/// The plan's term set — cell titles plus path stems, normalized and
/// de-duplicated, in first-seen order.
pub(crate) fn plan_terms(cells: &[Value]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let push = |raw: &str, out: &mut Vec<String>| {
        if let Some(t) = normalize_term(raw) {
            if !out.contains(&t) {
                out.push(t);
            }
        }
    };
    for cell in cells {
        if let Some(Value::String(title)) = jget(cell, "title") {
            for word in title.split_whitespace() {
                push(word, &mut out);
            }
        }
        for key in ["files", "affects_skills", "affects_specs"] {
            for raw in string_list(cell, key) {
                let norm = raw.replace('\\', "/");
                for term in path_stem_terms(&norm) {
                    push(&term, &mut out);
                }
            }
        }
    }
    out
}

// ─── the candidates ─────────────────────────────────────────────────────────

fn candidate(id: &str, kind: &str, title: &str) -> Value {
    json!({
        "id": id,
        "kind": kind,
        "title": title,
        "verdict": Value::Null,
        "note": Value::Null,
    })
}

/// Does this rule's home — or any pattern it declares itself applied at —
/// intersect the plan's touched paths? Checked in BOTH directions because
/// either side may be the pattern: a rule's `applied_at` carries globs, and a
/// cell's `affects_skills`/`affects_specs` entries do too.
fn rule_intersects(rule: &RuleHome, plan_paths: &[String]) -> bool {
    let mut patterns: Vec<String> = Vec::new();
    let home = rule.home.replace('\\', "/").trim_start_matches('/').to_string();
    if !home.is_empty() {
        patterns.push(home);
    }
    for a in &rule.applied_at {
        let norm = a.replace('\\', "/").trim_start_matches('/').to_string();
        if !norm.is_empty() && !patterns.contains(&norm) {
            patterns.push(norm);
        }
    }
    if patterns.is_empty() {
        return false;
    }
    for path in plan_paths {
        if matches_owned(&patterns, path) {
            return true;
        }
        for pat in &patterns {
            if matches_owned(std::slice::from_ref(path), pat) {
                return true;
            }
        }
    }
    false
}

/// D5's candidate list for one feature: active decisions the plan's terms
/// select, then rule homes the plan's paths touch. Ids are unique — the first
/// occurrence wins — so a rule declared in two homes contributes one row.
pub(crate) fn derive_candidates(root: &Path, feature: &str) -> Result<Vec<Value>, Err2> {
    let cells = plan_cells(root, feature)?;
    let terms = plan_terms(&cells);
    let paths = plan_paths(&cells);
    let mut out: Vec<Value> = Vec::new();
    let mut seen: Vec<String> = Vec::new();

    if !terms.is_empty() {
        let active = active_decisions(root, false)?;
        let text = terms.join(" ");
        // (1a) the ranked hints `decisions log` itself would print, in its
        // own order — no tags to share here, so this is the top-3 slice of
        // the same >= 2 term-hit rule the loop below completes.
        for c in conflict_candidates(&active, &text, &[], None) {
            let id = js_disp_opt(jget(&c, "id"));
            if id.is_empty() || seen.contains(&id) {
                continue;
            }
            seen.push(id.clone());
            out.push(candidate(&id, "decision", &js_disp_opt(jget(&c, "excerpt"))));
        }
        // (1b) every OTHER active decision scoring >= 2 on the same terms.
        let term_refs: Vec<&str> = terms.iter().map(String::as_str).collect();
        for e in &active {
            let id = js_disp_opt(jget(e, "id"));
            if id.is_empty() || seen.contains(&id) {
                continue;
            }
            let haystacks: Vec<String> = text_haystacks(e, true)
                .into_iter()
                .map(|h| h.to_lowercase())
                .collect();
            if count_term_hits(&haystacks, &term_refs) < 2 {
                continue;
            }
            seen.push(id.clone());
            let title =
                crate::textutil::truncate_chars_head(&js_disp_opt(jget(e, "decision")), 90);
            out.push(candidate(&id, "decision", &title));
        }
    }

    // (2) rule homes whose home or applied_at intersects the plan's paths.
    if !paths.is_empty() {
        for rule in &load_ownership(root).rules {
            let id = rule.rule.trim();
            if id.is_empty() || seen.contains(&id.to_string()) {
                continue;
            }
            if !rule_intersects(rule, &paths) {
                continue;
            }
            seen.push(id.to_string());
            out.push(candidate(id, "rule", &format!("{id} \u{2014} {}", rule.home)));
        }
    }

    Ok(out)
}

// ─── the record field ───────────────────────────────────────────────────────

/// `conflict_review` as it lands on the workflow record. Re-deriving REPLACES
/// this whole object, which is how a re-derive clears every verdict: the rows
/// are rebuilt with `verdict: null`, never patched in place.
pub(crate) fn build_conflict_review(plan_rev: Value, candidates: Vec<Value>, derived_at: &str) -> Value {
    json!({
        "plan_rev": plan_rev,
        "derived_at": derived_at,
        "candidates": Value::Array(candidates),
    })
}

/// Sets exactly one candidate's verdict and note in place. Returns the
/// refusal text for an id no candidate carries — the caller has already
/// checked the verdict value itself.
pub(crate) fn apply_conflict_verdict(
    review: &mut Value,
    id: &str,
    verdict: &str,
    note: Option<&str>,
) -> Result<(), String> {
    let known: Vec<String> = match review.get("candidates") {
        Some(Value::Array(items)) => items.iter().map(|c| js_disp_opt(jget(c, "id"))).collect(),
        _ => Vec::new(),
    };
    let Some(Value::Array(items)) = review.get_mut("candidates") else {
        return Err(unknown_candidate_refusal(id, &known));
    };
    for c in items.iter_mut() {
        if js_disp_opt(jget(c, "id")) != id {
            continue;
        }
        let Some(obj) = c.as_object_mut() else { continue };
        obj.insert("verdict".into(), json!(verdict));
        obj.insert(
            "note".into(),
            match note {
                Some(n) => json!(n),
                None => Value::Null,
            },
        );
        return Ok(());
    }
    Err(unknown_candidate_refusal(id, &known))
}

fn unknown_candidate_refusal(id: &str, known: &[String]) -> String {
    let listed = if known.is_empty() {
        "none \u{2014} the derived list is empty".to_string()
    } else {
        known.join(", ")
    };
    format!(
        "state plan-conflicts verdict: refused \u{2014} \"{id}\" is not a derived conflict candidate. Candidates: {listed}. FIX: name one of them, or re-derive first (\"state plan-conflicts derive\")."
    )
}

fn unknown_verdict_refusal(verdict: &str) -> String {
    format!(
        "state plan-conflicts verdict: refused \u{2014} \"{verdict}\" is not a verdict. Use one of: {}.",
        CONFLICT_VERDICTS.join(", ")
    )
}

// ─── the shared lane-targeted transaction ───────────────────────────────────

/// The `plan-rev bump` transaction shape, verbatim in order and refusals,
/// parameterized on the record patch: peek outside the locks, take
/// `workflow:<id>` then the projection lock, resolve strictly, re-check that
/// the peek picked the record the resolution landed on, patch, rebuild the
/// lane projection. `patch_with` receives the current record and returns
/// (patch, result-value, text) or a refusal string.
fn with_lane_workflow<F>(ctx: &Ctx, flags: &Flags, verb: &str, patch_with: F) -> R2<Out>
where
    F: FnOnce(&Map<String, Value>, &str) -> Result<Result<(Map<String, Value>, Value, String), String>, Err2>,
{
    let (lane_feature, no_lane) = match mutation_lane_selector(flags, verb) {
        Ok(v) => v,
        Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
        Err(Err2::Ex) => return Err(Err2::Ex),
    };
    let scope = resolve_mutation_lock_scope(&ctx.root, lane_feature.as_deref(), no_lane)?;
    let workflows = list_workflows(&ctx.root)?;
    let peeked_id = scope
        .feature
        .as_deref()
        .and_then(|f| find_live_workflow(&workflows, f))
        .map(wf_id);
    let _workflow_guard = match &peeked_id {
        Some(id) => Some(acquire_workflow_lock(&ctx.root, id)?),
        None => None,
    };
    let _projection_guard = acquire_named_lock(
        &ctx.root,
        &projection_lock_name(scope.lane, scope.feature.as_deref()),
    )?;
    let target = resolve_mutation_target(&ctx.root, lane_feature.as_deref(), verb, no_lane)?;
    let Some(lane) = target.lane() else {
        return Ok(Out::Thrown(format!(
            "{verb}: refused \u{2014} resolution landed on the default (non-lane) record. conflict_review lives on a lane's workflow record only, exactly like plan_rev. FIX: target a lane explicitly with --lane <feature>, or bind the calling session to one first (\"state session bind --session-id <id> --lane <feature>\")."
        )));
    };
    let lane = lane.to_string();
    let live = list_workflows(&ctx.root)?;
    let Some(wf) = find_live_workflow(&live, &lane) else {
        return Ok(Out::Thrown(format!(
            "{verb}: no live workflow record found for lane \"{lane}\". FIX: start the lane first (\"state start-feature --feature {lane} --as-lane\")."
        )));
    };
    let id = wf_id(wf);
    if peeked_id.as_deref() != Some(id.as_str()) {
        return Ok(Out::Thrown(format!(
            "{verb}: the target lane's workflow changed while this call was starting (expected \"{}\", resolved \"{id}\"), so the workflow lock this call holds does not protect it. Nothing was written. FIX: re-run the command.",
            peeked_id.as_deref().unwrap_or("none")
        )));
    }
    // The patch is computed BEFORE the write, off the record `list_workflows`
    // just read through the writer's own strict reader, so a refusal (unknown
    // candidate id, nothing derived yet) leaves the record untouched — no
    // empty patch, no rewrite, no rebuilt projection.
    let current = wf.clone();
    let (patch, result, text) = match patch_with(&current, &lane)? {
        Ok(v) => v,
        Err(refusal) => return Ok(Out::Thrown(refusal)),
    };
    update_workflow_assuming_lock_with(&ctx.root, &id, |_| Ok(patch.clone()))?;
    rebuild_lane_projection(&ctx.root, &lane)?;
    Ok(Out::Emit(result, text, 0))
}

// ─── state plan-conflicts derive ────────────────────────────────────────────

pub(crate) fn run_plan_conflicts_derive(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["lane", "no-lane"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "no-lane") {
        return None;
    }
    let ctx = match go("state plan-conflicts derive", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        with_lane_workflow(&ctx, &flags, "state plan-conflicts derive", |current, lane| {
            let plan_rev = current.get("plan_rev").cloned().unwrap_or(json!(0));
            let candidates = derive_candidates(&ctx.root, lane)?;
            let count = candidates.len();
            let review = build_conflict_review(plan_rev.clone(), candidates, &now_iso());
            let mut patch = Map::new();
            patch.insert("conflict_review".into(), review.clone());
            let text = if count == 0 {
                format!(
                    "Derived 0 conflict candidates for lane \"{lane}\" (plan_rev {}) \u{2014} \"0 conflicts\" is true for this plan revision.",
                    js_disp(&plan_rev)
                )
            } else {
                format!(
                    "Derived {count} conflict candidate(s) for lane \"{lane}\" (plan_rev {}); every one needs a verdict (\"state plan-conflicts verdict --id <candidate> --verdict compatible|conflicts|retires-prior\").",
                    js_disp(&plan_rev)
                )
            };
            Ok(Ok((patch, review, text)))
        })
    })();
    finish(&ctx, out)
}

// ─── state plan-conflicts verdict ───────────────────────────────────────────

pub(crate) fn run_plan_conflicts_verdict(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["id", "verdict", "note", "lane", "no-lane"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "no-lane") {
        return None;
    }
    let ctx = match go("state plan-conflicts verdict", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let id = match require_flag(&flags, "id") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let verdict = match require_flag(&flags, "verdict") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        if !CONFLICT_VERDICTS.contains(&verdict.as_str()) {
            return Ok(Out::Thrown(unknown_verdict_refusal(&verdict)));
        }
        let note = flag_value(&flags, "note");
        with_lane_workflow(&ctx, &flags, "state plan-conflicts verdict", |current, lane| {
            let Some(review) = current.get("conflict_review") else {
                return Ok(Err(format!(
                    "state plan-conflicts verdict: refused \u{2014} lane \"{lane}\" has no derived conflict review to record a verdict on. FIX: derive first (\"state plan-conflicts derive --lane {lane}\")."
                )));
            };
            let mut review = review.clone();
            if let Err(refusal) = apply_conflict_verdict(&mut review, &id, &verdict, note.as_deref()) {
                return Ok(Err(refusal));
            }
            let mut patch = Map::new();
            patch.insert("conflict_review".into(), review.clone());
            let text = format!(
                "Recorded verdict \"{verdict}\" for candidate \"{id}\" on lane \"{lane}\"."
            );
            Ok(Ok((patch, review, text)))
        })
    })();
    finish(&ctx, out)
}
