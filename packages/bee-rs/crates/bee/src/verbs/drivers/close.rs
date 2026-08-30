// close, the report-only scribing/capture doors, and routing
//
// Split out of the single 4.9k-line verbs/drivers.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{self, append_jsonl, ensure_dir, read_json, write_json_atomic, write_text_atomic, ReadJson};
use crate::jsjson;
use crate::roots::{resolve_store_root, Roots};
use crate::state::{capture_queue_threshold, read_config_raw};
use crate::verbs::reservations::{
    finish, js_is_ws, now_iso, now_ms, parse_flags, prelude, pseudo_uuid_v4, truthy, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{
    release_reservations_for_agent, reserve_path_atomic, Err2, ReserveOutcome,
};
use crate::verbs::knowledge::{bee_of, collect_concepts, str_array, str_field, touches_subject};
use serde_json::{json, Map, Number, Value};
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

// ═══ close ═════════════════════════════════════════════════════════════════

/// D7/D8 (docs/history/test-doctrine/CONTEXT.md): pinned prefix of the
/// tests-door refusal headline — a capped cell whose `trace.report` is
/// present but carries no valid D8 proof string (verbs/cells/proof.rs).
/// Message-contract tests live in verbs/drivers/tests.rs.
pub(crate) const CLOSE_PROOF_DEBT_PREFIX: &str = "Proof debt for";

/// Pinned prefix of the D1 capture-debt refusal headline (message-contract
/// test: `close_refuses_uncaptured_behavior_change_cells`). Cite: CONTEXT.md
/// D1 (c2a7bd4f item 1).
pub(crate) const CLOSE_CAPTURE_DEBT_PREFIX: &str = "Capture debt for";

/// wl-3: pinned prefix of the judge-debt refusal headline (message-contract
/// tests live in verbs/cells/tests.rs, alongside the rest of the judge
/// surface). docs/history/workflow-lessons/plan.md wl-3.
pub(crate) const CLOSE_JUDGE_DEBT_PREFIX: &str = "Judge debt for";

/// slp-dissent-stop-and-ask sd-4: pinned prefix of the dissent-debt refusal
/// headline (message-contract tests live in verbs/cells/tests.rs, beside the
/// rest of the dissent surface). Cite: a2affcba and 4b7aa303.
pub(crate) const CLOSE_DISSENT_DEBT_PREFIX: &str = "Dissent debt for";

/// slp-advisor-nudge an-3: pinned prefix of the advisor-nudge refusal
/// headline (message-contract tests live in verbs/cells/tests.rs, beside the
/// dissent door's own). Cite: 9e5eda5b.
pub(crate) const CLOSE_ADVISOR_NUDGE_DEBT_PREFIX: &str = "Advisor nudge debt for";

/// D1: pinned prefix of the knowledge-freshness refusal headline (message-
/// contract tests live in verbs/drivers/tests.rs). CONTEXT.md D1.
pub(crate) const CLOSE_KNOWLEDGE_FRESHNESS_PREFIX: &str = "Knowledge freshness debt for";

/// doc-impact-synthesis D1b: pinned prefix of the impact-door refusal
/// headline (message-contract tests live in verbs/drivers/tests.rs).
/// CONTEXT.md D1, plan v2 kds-2.
pub(crate) const CLOSE_IMPACT_PREFIX: &str = "Impact debt for";

/// doc-impact-synthesis D2: pinned prefix of the routing-door refusal
/// headline (message-contract tests live in verbs/drivers/tests.rs).
/// CONTEXT.md D2, plan v2 kds-3.
pub(crate) const CLOSE_ROUTING_PREFIX: &str = "Routing debt for";

/// doc-impact-synthesis D3: pinned prefix of the doc-deferral-door refusal
/// headline (message-contract tests live in verbs/drivers/tests.rs).
/// CONTEXT.md D3, plan v2 kds-3.
pub(crate) const CLOSE_DOC_DEFERRAL_PREFIX: &str = "Doc deferral debt for";

/// uat-stop-placement D4.4/D2 (docs/history/uat-stop-placement/CONTEXT.md):
/// pinned prefix of the close-time uat-door refusal headline
/// (message-contract test lives beside the rest of this door's tests,
/// below in this file's own `mod tests`).
pub(crate) const CLOSE_UAT_PREFIX: &str = "Uat gate pending for";

/// provenance: test-runner.mjs declaredTestCommands + state.mjs
/// normalizeCommands (verbs/test_runner.rs:184 declared_test_commands).
/// `None` == JS `null` (undeclared).
pub(crate) fn declared_test_commands(root: &Path) -> D<Option<Vec<String>>> {
    let config = read_config_raw(root);
    if let Some(Value::Array(items)) = config.get("dogfood_repos") {
        if !items.is_empty() {
            return Err(Delegate); // normalizeDogfoodRepos may warn to stderr
        }
    }
    let raw_test = config
        .get("commands")
        .and_then(Value::as_object)
        .and_then(|c| c.get("test"));
    let normalized: Vec<String> = match raw_test {
        Some(Value::String(s)) => {
            let t = js_trim(s);
            if t.is_empty() { Vec::new() } else { vec![t.to_string()] }
        }
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(js_trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    };
    let cleaned: Vec<String> = normalized.into_iter().filter(|c| c != "none").collect();
    Ok(if cleaned.is_empty() { None } else { Some(cleaned) })
}

// D7 (docs/history/test-doctrine/CONTEXT.md): close used to own a private
// copy of the declared-test runner here (`CommandResult`/`TestRun`/
// `shell_command`/`run_declared_tests`/`tests_result_value`/
// `command_result_value`/`render_test_command_lines`/`first_failure_line`).
// No boundary auto-run remains — close never spawns `commands.test` itself
// — so that whole runner is DELETED here rather than kept unreachable;
// `bee test` (test_runner.rs) keeps its own independent copy, and
// `.bee/logs/test-results.json` stays exactly as it was, per D4/prohibition.
// The tests door now reads recorded proof instead (verbs/cells/proof.rs,
// `feature_proof_check`) — see `close_handler` below. `posix_shell` alone
// survives, unused by `close_handler` itself now, kept only because this
// module's own test fixtures still probe it before building a `commands.test`
// fixture that threads a (now-ignored) shell through `close_handler`'s
// still-compatible signature.
pub(crate) fn posix_shell() -> Option<&'static str> {
    crate::shell::posix_shell()
}

// ── scribing debt + capture queue (the report-only doors) ──────────────────

/// provenance: cells.mjs scribingRunStampMs (verbs/status_full.rs:1700).
pub(crate) fn scribing_run_stamp_ms(run: Option<&Value>) -> Option<f64> {
    let run = run?;
    if !truthy(run) {
        return None;
    }
    let at = vget(run, "at").filter(|v| truthy(v));
    let chosen = at.or_else(|| vget(run, "date"));
    let parsed = date_parse(chosen);
    if parsed.is_finite() { Some(parsed) } else { None }
}

/// provenance: reservations.rs js_date_parse, wrapped: an exotic date shape
/// (which V8 may parse and this port may not) yields NaN here, which is the
/// same control-flow branch Node takes for an unparseable date.
pub(crate) fn date_parse(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::String(s)) => match crate::verbs::reservations::js_date_parse(s) {
            Ok(Some(ms)) => ms,
            _ => f64::NAN,
        },
        _ => f64::NAN,
    }
}

/// provenance: cells.mjs bestScribingStampMs (also ported at
/// verbs/status_full/cells.rs:303 for `status`), scoped to `feature`: the
/// jsonl ledger's max, then the feature's own lane record
/// (`.bee/lanes/<feature>.json`, via `read_lane_display` — the same
/// fail-open display read `status`'s port already uses), then
/// `state.json`'s `last_scribing_run` — the freshest of the three wins, in
/// that order, matching the status-verb port exactly.
pub(crate) fn best_scribing_stamp_ms(root: &Path, feature: &str, state: &Map<String, Value>) -> D<Option<f64>> {
    let feature_value = Value::String(feature.to_string());
    let mut best: Option<f64> = None;
    for entry in read_jsonl(&root.join(".bee").join("logs").join("scribing-runs.jsonl")) {
        if !truthy(&entry) || !strict_eq(vget(&entry, "feature"), Some(&feature_value)) {
            continue;
        }
        let parsed = date_parse(vget(&entry, "ts"));
        if parsed.is_finite() && best.map(|b| parsed > b).unwrap_or(true) {
            best = Some(parsed);
        }
    }
    // read_lane_display is the fail-open DISPLAY read: absent reads as None,
    // and a corrupt/mismatched record warns (naming the path) and reads as
    // None too — it never throws, so a bad lane record never stops close.
    let lane = crate::verbs::workflow_store::read_lane_display(root, feature).map_err(|_| Delegate)?;
    if let Some(lane) = lane {
        if let Some(stamp) = scribing_run_stamp_ms(lane.get("last_scribing_run")) {
            if best.map(|b| stamp > b).unwrap_or(true) {
                best = Some(stamp);
            }
        }
    }
    if let Some(lsr) = state.get("last_scribing_run") {
        if truthy(lsr) && strict_eq(vget(lsr, "feature"), Some(&feature_value)) {
            if let Some(stamp) = scribing_run_stamp_ms(Some(lsr)) {
                if best.map(|b| stamp > b).unwrap_or(true) {
                    best = Some(stamp);
                }
            }
        }
    }
    Ok(best)
}

/// provenance: fsutil.mjs readJsonl (verbs/status_full.rs:526) — unparseable
/// lines are silently skipped.
pub(crate) fn read_jsonl(file: &Path) -> Vec<Value> {
    let Ok(bytes) = std::fs::read(file) else { return Vec::new() };
    let text = String::from_utf8_lossy(&bytes);
    let mut events = Vec::new();
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = js_trim(line);
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            events.push(v);
        }
    }
    events
}

/// provenance: state.mjs readState — the ONE field scribingDebt reads through
/// it here is `last_scribing_run` (the feature comes from --feature, never
/// from the record), and defaultState() carries no such key, so the raw file
/// object IS the merged value for it.
pub(crate) fn read_state(root: &Path) -> D<Map<String, Value>> {
    match rj(&root.join(".bee").join("state.json"))? {
        Some(Value::Object(m)) => Ok(m),
        _ => Ok(Map::new()),
    }
}

pub(crate) struct DebtSummary {
    pub(crate) count: usize,
    pub(crate) ids: Vec<Value>,
}

/// provenance: cells.mjs scribingDebt(root, {feature}) — the feature-scoped
/// overrides arm (scribing-integrity si-1), which is the one close uses.
///
/// debt-door-archive dda-1: reads `list_cells_including_archive` (guard.rs),
/// not the plain active-only `list_cells`, so a behavior_change cell that
/// `bee close`'s own auto-archive already moved to
/// `.bee/cells/archive/<feature>/` still counts against the threshold below
/// — a clear door can no longer be a side effect of archiving the debt away.
///
/// D5 (trun-9, docs/history/traceable-runs/plan.md): this scan is also now
/// the ONE place that MATERIALIZES a claimable `scribe` deferred-queue
/// record the moment it finds debt with no record yet — "capping a
/// behavior_change cell enqueues a scribe record" happens lazily, on the
/// next call here (close's own door, and `state_group/set_gate.rs`'s swap
/// door, both call this on every relevant mutation) rather than inside the
/// cap handler itself, which this cell's `files` list does not reach.
/// Whether a cell's debt still counts is decided by ONE shared rule,
/// `state_group::deferred_debt_cleared` (ledger.rs) — see that function's
/// doc for what "which one wins" means. A cell already named by ANY
/// existing `scribe` record (completed or not) never gets a second one.
pub(crate) fn scribing_debt(root: &Path, feature: &str) -> D<DebtSummary> {
    let state = read_state(root)?;
    let threshold = best_scribing_stamp_ms(root, feature, &state)?.unwrap_or(0.0);

    let crate::verbs::state_group::ScribeQueueCells { queued: queued_cells, completed: completed_cells } =
        crate::verbs::state_group::scribe_queue_cells(root, feature);

    let mut ids = Vec::new();
    // (cell id, that cell's own declared `files`) — only cells no existing
    // record names yet; materialized into one new record below, once.
    let mut to_materialize: Vec<(String, Vec<String>)> = Vec::new();
    for cell in list_cells_including_archive(root, feature, Some("capped"))? {
        let trace = vget(&cell, "trace").cloned().unwrap_or(Value::Object(Map::new()));
        if !matches!(vget(&trace, "behavior_change"), Some(Value::Bool(true))) {
            continue;
        }
        let capped_at = date_parse(vget(&trace, "capped_at"));
        let legacy_cleared = !(capped_at.is_finite() && capped_at > threshold);
        let id_str = vget(&cell, "id").and_then(Value::as_str).unwrap_or("").to_string();
        let queue_completed = !id_str.is_empty() && completed_cells.contains(&id_str);
        if crate::verbs::state_group::deferred_debt_cleared(legacy_cleared, queue_completed) {
            continue;
        }
        ids.push(vget(&cell, "id").cloned().unwrap_or(Value::Null));
        if !id_str.is_empty() && !queued_cells.contains(&id_str) {
            let files: Vec<String> = match vget(&cell, "files") {
                Some(Value::Array(a)) => a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect(),
                _ => Vec::new(),
            };
            to_materialize.push((id_str, files));
        }
    }
    if !to_materialize.is_empty() {
        let cell_ids: Vec<String> = to_materialize.iter().map(|(id, _)| id.clone()).collect();
        let mut files: Vec<String> = to_materialize.iter().flat_map(|(_, f)| f.iter().cloned()).collect();
        files.sort();
        files.dedup();
        let reason = format!(
            "Scribing debt: {} capped behavior_change cell(s) for \"{feature}\" with no scribing/compounding record since capping.",
            cell_ids.len()
        );
        // Best-effort: this scan runs on every close attempt and every gate
        // swap check, so a failed append here is never fatal — the count
        // just returned above is still correct for THIS call, and the next
        // call tries materializing again.
        let _ = crate::verbs::deferred_queue::enqueue(root, "scribe", feature, &cell_ids, &[], &files, &reason);
    }
    Ok(DebtSummary { count: ids.len(), ids })
}

/// wl-3 (docs/history/workflow-lessons/plan.md): the closing feature's
/// route — read via `route.lane`, the SAME field the write-guard and
/// orient doors already key off (hooks/write_guard/hook_local.rs:524-530,
/// status_full/orient.rs:89-92), never `lane["mode"]` (that field carries
/// the workflow's own mode — `"feature"` for the live shape — not the
/// lane classification, so keying off it left the door silently absent on
/// every real store). Read via the fail-open display read `scribing_debt`
/// above already uses, so a corrupt or missing lane record reads as "no
/// route" rather than throwing. When the lane record carries no `route`
/// of its own, fall back to the default state's `route.lane` — the same
/// top-level field the write-guard/orient precedents read directly.
///
/// hpf-1 (review-p1-fixes, 2026-08-12, P1): the default-state route is
/// GLOBAL — one session's most recent `state route --set` — so reading it
/// unconditionally let ANY feature's close inherit whatever OTHER feature
/// last routed. Live and refusing today: `bee close --feature <small
/// feature>` was blocked by a `judge-debt` door that belonged to an
/// unrelated high-risk feature's route. The default-state route is now
/// taken only when it names THIS feature — same identity check as
/// `gated_add_refusal` (verbs/cells/handlers_write.rs:119,133-142),
/// implemented via `route_belongs_to_feature`
/// (verbs/state_group/workflows.rs:683, rti-1), which already exists to
/// stop a route from a PRIOR feature being mistaken for this one's own.
/// When the state route names another feature (or none), the lane
/// record's own `mode` field is the last fallback — but ONLY when it
/// happens to name a lane class (`ROUTE_LANE_VALUES`,
/// verbs/state_group/workflows.rs:289-290: docs/tiny/small/spike/
/// standard/high-risk). `mode` usually carries the WORKFLOW class instead
/// (`ROUTE_CLASS_VALUES`, same file:287-288 — "feature" is the live
/// shape's constant value there), which is not a lane at all; reading it
/// unconditionally would misread an ordinary "feature"-mode lane as lane
/// "feature". `None` covers every remaining case: no lane record, no
/// state route owned by this feature, and no lane `mode` that happens to
/// spell a lane class.
///
/// Mirrors `ROUTE_LANE_VALUES` (verbs/state_group/workflows.rs:289-290)
/// without importing it: that const is module-private there, and this
/// cell's file scope does not extend to changing its visibility.
const FEATURE_ROUTE_LANE_CLASSES: [&str; 6] = ["docs", "tiny", "small", "spike", "standard", "high-risk"];

pub(crate) fn feature_route(root: &Path, feature: &str) -> D<Option<String>> {
    let route_lane = |v: &Value| -> Option<String> {
        match vget(v, "lane") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        }
    };
    let lane = crate::verbs::workflow_store::read_lane_display(root, feature).map_err(|_| Delegate)?;
    if let Some(from_lane) = lane.as_ref().and_then(|l| l.get("route")).and_then(route_lane) {
        return Ok(Some(from_lane));
    }
    let state = read_state(root)?;
    if let Some(Value::Object(route)) = state.get("route") {
        let feature_value = Value::String(feature.to_string());
        if crate::verbs::state_group::route_belongs_to_feature(route, &feature_value) {
            if let Some(from_state) = route_lane(&Value::Object(route.clone())) {
                return Ok(Some(from_state));
            }
        }
    }
    let mode_is_lane_class = lane
        .as_ref()
        .and_then(|l| l.get("mode"))
        .and_then(Value::as_str)
        .filter(|m| FEATURE_ROUTE_LANE_CLASSES.contains(m));
    Ok(mode_is_lane_class.map(str::to_string))
}

/// hpf-1 (review-p1-fixes, 2026-08-12, P1): the day the judge-debt door
/// itself shipped (wfl-3, docs/history/workflow-lessons/plan.md). At that
/// moment the live store already held 122 capped `behavior_change` cells,
/// only 10 of them judged — a door that counted every one of those 112 as
/// debt would block every legacy feature's close on cells that predate
/// the requirement entirely; that is not a migration path, it is a wall.
/// Only a cell capped AT OR AFTER this stamp owes the door anything.
pub(crate) const JUDGE_DOOR_INTRODUCED_AT: &str = "2026-08-11T00:00:00.000Z";

/// wl-3: every capped `behavior_change` cell that carries NO judge record
/// (`trace.semantic_judge` empty or absent) counts as judge debt. A cell
/// capped with a NEEDS_REVISION verdict is unreachable here — `cells cap`
/// already refuses that cap unless it carries an audited
/// `judge_overrides` entry (handlers_close.rs), so every capped cell's
/// `semantic_judge`, once non-empty, ended on a verdict cap itself already
/// accepted. Same archive-inclusive read `scribing_debt` uses above, for
/// the same reason: an auto-archived cell must still count.
///
/// hpf-1: grandfathered by `JUDGE_DOOR_INTRODUCED_AT` — a cell whose
/// `trace.capped_at` is missing, unparseable, or earlier than that stamp
/// predates the door and is never counted as debt, judged or not.
pub(crate) fn judge_debt(root: &Path, feature: &str) -> D<DebtSummary> {
    let cutoff = date_parse(Some(&Value::String(JUDGE_DOOR_INTRODUCED_AT.to_string())));
    let mut ids = Vec::new();
    for cell in list_cells_including_archive(root, feature, Some("capped"))? {
        let trace = vget(&cell, "trace").cloned().unwrap_or(Value::Object(Map::new()));
        if !matches!(vget(&trace, "behavior_change"), Some(Value::Bool(true))) {
            continue;
        }
        let capped_at = date_parse(vget(&trace, "capped_at"));
        if !(capped_at.is_finite() && capped_at >= cutoff) {
            continue; // pre-door (or no capped_at at all): grandfathered, not debt
        }
        let judged = matches!(vget(&trace, "semantic_judge"), Some(Value::Array(a)) if !a.is_empty());
        if !judged {
            ids.push(vget(&cell, "id").cloned().unwrap_or(Value::Null));
        }
    }
    Ok(DebtSummary { count: ids.len(), ids })
}

/// hpf-1: mirrors `has_capture_deferral_decision` below — a logged decision
/// tagged `judge-deferral` naming the feature lifts the judge-debt
/// refusal, the same escape D1 gave the scribing-debt door.
pub(crate) fn has_judge_deferral_decision(root: &Path, feature: &str) -> D<bool> {
    let active = crate::verbs::decisions::active_decisions(root, false).map_err(|_| Delegate)?;
    let filtered = crate::verbs::decisions::filter_decision_events(
        active,
        &crate::verbs::decisions::DecisionFilters {
            tag: Some("judge-deferral".to_string()),
            feature: Some(feature.to_string()),
            ..Default::default()
        },
    )
    .map_err(|_| Delegate)?;
    Ok(!filtered.is_empty())
}

/// uat-stop-placement D2 (docs/history/uat-stop-placement/CONTEXT.md):
/// mirrors `has_judge_deferral_decision` above — a logged decision tagged
/// `uat-deferral` naming the feature lifts the close-time uat-door
/// refusal, the same escape shape `judge-debt` already established.
pub(crate) fn has_uat_deferral_decision(root: &Path, feature: &str) -> D<bool> {
    let active = crate::verbs::decisions::active_decisions(root, false).map_err(|_| Delegate)?;
    let filtered = crate::verbs::decisions::filter_decision_events(
        active,
        &crate::verbs::decisions::DecisionFilters {
            tag: Some("uat-deferral".to_string()),
            feature: Some(feature.to_string()),
            ..Default::default()
        },
    )
    .map_err(|_| Delegate)?;
    Ok(!filtered.is_empty())
}

// uat-stop-placement D4.4/D2, docs/history/uat-approval-reaches-the-door/
// plan.md R1-R3: `gates.uat.approved` now resolves through the single
// shared resolver `crate::uat::uat_gate_approved` — the same one
// `uat_merge_precheck` (verbs/worktree/phases.rs) calls — so this door and
// the merge door never carry two copies of the resolution again.

/// provenance: capture.mjs pendingCaptureStubs + captureQueue
/// (verbs/status_full.rs:2382) — only the COUNT used to reach close's door
/// text (localeCompare sort therefore never mattered); U3 (docs/history/
/// knowledge-usable/CONTEXT.md) also needs the oldest pending stub's age,
/// so both ride the one read below.
pub(crate) fn capture_queue_pending(root: &Path) -> (usize, f64) {
    let events = read_jsonl(&root.join(".bee").join("capture-queue.jsonl"));
    let mut flushed: Vec<Value> = Vec::new();
    let mut stubs: Vec<&Value> = Vec::new();
    for event in &events {
        if !matches!(event, Value::Object(_)) {
            continue;
        }
        let id = vget(event, "id");
        if matches!(vget(event, "kind"), Some(Value::String(k)) if k == "flush")
            && id.map(truthy).unwrap_or(false)
        {
            flushed.push(id.unwrap().clone());
        } else if matches!(vget(event, "kind"), Some(Value::String(k)) if k == "stub")
            && id.map(truthy).unwrap_or(false)
        {
            stubs.push(event);
        }
    }
    let pending: Vec<&&Value> = stubs
        .iter()
        .filter(|s| !flushed.iter().any(|f| strict_eq(Some(f), vget(s, "id"))))
        .collect();
    // The queue also carries bookkeeping that is NOT a settlement: citation
    // `touches-sweep` rows and `promote` pointers. Counting them here made the
    // close door say "flush before new work" about rows the reader cannot act
    // on — a sweep row usually belongs to another feature — every single
    // close. Nothing is dropped: they remain queued, listed and flushable;
    // only this door's count narrows to what the reader actually owes. The
    // Stop-hook nudge applies the same split (hooks/session_close/nudges.rs).
    let settlements: Vec<&&&Value> = pending
        .iter()
        .filter(|s| {
            !matches!(
                vget(s, "source").and_then(Value::as_str),
                Some("touches-sweep") | Some("promote")
            )
        })
        .collect();
    let oldest_ms = settlements
        .iter()
        .map(|s| date_parse(vget(s, "at")))
        .filter(|ms| ms.is_finite())
        .fold(f64::NAN, |acc, ms| if acc.is_nan() || ms < acc { ms } else { acc });
    (settlements.len(), oldest_ms)
}

/// U4 (docs/history/knowledge-usable/CONTEXT.md): the proposal's dominant
/// area — the `area_updates` entry with the most attributed bullets, ties
/// keeping the proposal's own order — names the stub's `area` field. `None`
/// when the proposal named no area at all (D19: a work item with no
/// `bee.areas` and no scribing stamp).
pub(crate) fn dominant_promote_area(proposal: &Value) -> Option<String> {
    proposal["area_updates"]
        .as_array()?
        .iter()
        .max_by_key(|u| u["bullets"].as_array().map(Vec::len).unwrap_or(0))
        .and_then(|u| u["area"].as_str())
        .map(str::to_string)
}

/// U4: once close writes `promote-proposals.md`, it ALSO appends one
/// capture-queue stub pointing at it — the queue is the living channel a
/// proposal reaches flush through (the 22 dead files under
/// docs/history/*/promote-proposals.md proved the standalone file is
/// write-only); the file itself keeps being written unchanged (audit trail,
/// D38). Same stub shape `capture add` writes (verbs/capture.rs run_add) so
/// `bee capture list`/flush treat it identically to a hand-added one.
/// Best-effort: an append failure here never fails close — the proposal file
/// itself is still the durable record.
pub(crate) fn enqueue_promote_stub(root: &Path, feature: &str, proposal: &Value, proposals_rel: &str) {
    let mut stub = Map::new();
    stub.insert("kind".into(), Value::String("stub".into()));
    stub.insert("id".into(), Value::String(pseudo_uuid_v4()));
    stub.insert("at".into(), Value::String(now_iso()));
    stub.insert(
        "outcome".into(),
        Value::String(format!("Promote proposal for \"{feature}\" — {proposals_rel}")),
    );
    stub.insert("dids".into(), Value::Array(Vec::new()));
    stub.insert(
        "area".into(),
        dominant_promote_area(proposal).map(Value::String).unwrap_or(Value::Null),
    );
    stub.insert("files".into(), Value::Array(vec![Value::String(proposals_rel.to_string())]));
    stub.insert("lane".into(), Value::Null);
    stub.insert("source".into(), Value::String("promote".into()));
    let _ = append_jsonl(&root.join(".bee").join("capture-queue.jsonl"), &Value::Object(stub));
}

/// D5 (trun-9, docs/history/traceable-runs/plan.md): the SECOND, separate
/// enqueue this same close writes — into `.bee/deferred-queue.jsonl`, never
/// into `.bee/capture-queue.jsonl` (the stub above is untouched and keeps
/// its own lifecycle; CONTEXT.md leaves absorbing it explicitly undecided).
/// This is the record `status_full::unapplied_promote_proposals` reads back
/// via `deferred_queue::items_for` so a proposal has a real, claimable
/// payload instead of only a derived mtime scan. Best-effort, same as the
/// stub above: an append failure here never fails close.
pub(crate) fn enqueue_promote_deferred_record(root: &Path, feature: &str, proposals_rel: &str) {
    let reason = format!("Promote proposal for \"{feature}\" awaiting apply — {proposals_rel}.");
    let _ = crate::verbs::deferred_queue::enqueue(
        root,
        "promote",
        feature,
        &[],
        &[],
        &[proposals_rel.to_string()],
        &reason,
    );
}

/// D1 escape hatch: a logged decision tagged `capture-deferral` whose
/// decision/rationale/alternatives text names the feature lifts the
/// scribing-debt refusal. Reuses the decisions verb's own read model
/// (crate::verbs::decisions::active_decisions + filter_decision_events)
/// rather than hand-parsing decisions.jsonl a second way — same tag-exact,
/// whole-token feature match `decisions active --tag --feature` already
/// uses. Cite: CONTEXT.md D1 (precedent: decision c8e25271).
pub(crate) fn has_capture_deferral_decision(root: &Path, feature: &str) -> D<bool> {
    let active = crate::verbs::decisions::active_decisions(root, false).map_err(|_| Delegate)?;
    let filtered = crate::verbs::decisions::filter_decision_events(
        active,
        &crate::verbs::decisions::DecisionFilters {
            tag: Some("capture-deferral".to_string()),
            feature: Some(feature.to_string()),
            ..Default::default()
        },
    )
    .map_err(|_| Delegate)?;
    Ok(!filtered.is_empty())
}

/// D1 escape hatch for the knowledge-freshness door below: mirrors
/// `has_capture_deferral_decision` above, tagged `knowledge-freshness-deferral`
/// instead of `capture-deferral` — but returns the deferring decision's own
/// `decision` text (not just a bool) so the door's detail line can quote the
/// reason, never a silent pass (D1: "an explicit recorded deferral with
/// reason, never a silent pass").
pub(crate) fn has_knowledge_freshness_deferral_decision(root: &Path, feature: &str) -> D<Option<String>> {
    let active = crate::verbs::decisions::active_decisions(root, false).map_err(|_| Delegate)?;
    let filtered = crate::verbs::decisions::filter_decision_events(
        active,
        &crate::verbs::decisions::DecisionFilters {
            tag: Some("knowledge-freshness-deferral".to_string()),
            feature: Some(feature.to_string()),
            ..Default::default()
        },
    )
    .map_err(|_| Delegate)?;
    Ok(filtered.last().and_then(|d| d.get("decision")).and_then(Value::as_str).map(|s| s.to_string()))
}

// ── D1: knowledge-freshness close door ──────────────────────────────────────
//
// CONTEXT.md D1: dangling knowledge pointers and unsynced docs in areas a
// feature touches BLOCK close — a hard door, like tests. Reuses `bee
// knowledge check`'s own findings (check_bundle) rather than a second
// detector, filtered post-walk to the file prefixes this close can fairly
// demand freshness from: `areas/<touched-area>/` (touched_bundle_areas
// above — the same touched-file -> area match promote.rs's own area-update
// section already applies) plus `work/<feature>/` (the feature's own work
// bundle) — a feature never blocks on a pointer it never touched, so an
// in-flight sibling feature's own stale pointers never tax this close.
// `dangling_source` and `dangling_required_context` warnings in scope block;
// `not_canonical`/`invalid_evidence_state` stay report-only in the detail —
// named limitation: prose contradictions (the "dark guards" class) have no
// machine detector here, that is S2-c distill work, recorded, not silently
// dropped.
pub(crate) fn build_knowledge_freshness_door(root: &Path, feature: &str) -> D<Door> {
    let Some(dir) = crate::verbs::knowledge::bundle_dir(root) else {
        return Ok(Door { door: "knowledge-freshness", blocking: false, detail: "clear".to_string(), command: None });
    };
    let Some(report) = crate::verbs::knowledge::check_bundle(&dir, false) else {
        return Ok(Door { door: "knowledge-freshness", blocking: false, detail: "clear".to_string(), command: None });
    };
    let touched_files = feature_touched_files(root, feature)?;
    let touched_areas = touched_bundle_areas(&dir, &touched_files);
    let mut prefixes: Vec<String> = touched_areas.iter().map(|a| format!("areas/{a}/")).collect();
    prefixes.push(format!("work/{feature}/"));

    let mut blocking_items: Vec<String> = Vec::new();
    let mut report_only_count = 0usize;
    for w in &report.warnings {
        let code = w.get("code").and_then(Value::as_str).unwrap_or("");
        let file = w.get("file").and_then(Value::as_str).unwrap_or("");
        let message = w.get("message").and_then(Value::as_str).unwrap_or("");
        match code {
            "dangling_source" | "dangling_required_context" => {
                if prefixes.iter().any(|p| file.starts_with(p.as_str())) {
                    blocking_items.push(format!(
                        "{file}: {message} — remedy: point the pointer at its live target, or remove the entry with a one-line reason"
                    ));
                }
            }
            "not_canonical" | "invalid_evidence_state" => report_only_count += 1,
            _ => {}
        }
    }

    if blocking_items.is_empty() {
        let detail = if report_only_count == 0 {
            "clear".to_string()
        } else {
            format!(
                "clear — {report_only_count} report-only finding(s) (not_canonical/invalid_evidence_state) never block"
            )
        };
        return Ok(Door { door: "knowledge-freshness", blocking: false, detail, command: None });
    }

    if let Some(reason) = has_knowledge_freshness_deferral_decision(root, feature)? {
        return Ok(Door {
            door: "knowledge-freshness",
            blocking: false,
            detail: format!(
                "deferred — {} stale pointer(s) in touched area(s)/work bundle ({}); a logged knowledge-freshness-deferral decision names \"{feature}\": {reason}",
                blocking_items.len(),
                blocking_items.join("; ")
            ),
            command: None,
        });
    }

    Ok(Door {
        door: "knowledge-freshness",
        blocking: true,
        detail: format!(
            "{} stale pointer(s) in touched area(s)/work bundle: {}",
            blocking_items.len(),
            blocking_items.join("; ")
        ),
        command: Some("bee knowledge check"),
    })
}

pub(crate) struct Door {
    pub(crate) door: &'static str,
    pub(crate) blocking: bool,
    pub(crate) detail: String,
    pub(crate) command: Option<&'static str>,
}

impl Door {
    pub(crate) fn value(&self) -> Value {
        let mut m = Map::new();
        m.insert("door".into(), Value::String(self.door.into()));
        m.insert("blocking".into(), Value::Bool(self.blocking));
        m.insert("detail".into(), Value::String(self.detail.clone()));
        m.insert(
            "command".into(),
            match self.command {
                Some(c) => Value::String(c.into()),
                None => Value::Null,
            },
        );
        Value::Object(m)
    }
}

// ── doc-impact-synthesis D1b: impact door at close ──────────────────────────
//
// CONTEXT.md D1: every decision the closing feature logged (feature-stamped
// by kds-1) that a docs/** file still cites gets one more fresh sweep at
// close — citations only, never a text scan. Stub-independent by design:
// v1's flush-coverage design sat on the log-time capture queue, but a hit
// written AFTER log time (a doc edited post-log to add a stale citation)
// never had a stub to flush — so this door re-derives its own findings
// every close instead of trusting a queue, exactly like knowledge-freshness
// above re-derives from `check_bundle` rather than a persisted record.

/// doc-impact-synthesis D1b: the closing feature's own decision ids,
/// collected from the structured `feature` field kds-1 stamps onto every new
/// `decide` event — structured field ONLY. Named deviation (plan v2 kds-2,
/// plan-check S2): a decision logged before kds-1 landed carries no
/// `feature` field and is never walked here; a time-window fallback was
/// rejected as unboundable — that debt belongs to the 2026-08-16 audit
/// backfill (kds-4) and the D4 campaign row, not this door.
pub(crate) fn feature_stamped_decision_ids(root: &Path, feature: &str) -> D<Vec<String>> {
    let active = crate::verbs::decisions::active_decisions(root, false).map_err(|_| Delegate)?;
    Ok(active
        .into_iter()
        .filter(|e| {
            e.get("type").and_then(Value::as_str) == Some("decide")
                && e.get("feature").and_then(Value::as_str) == Some(feature)
        })
        .filter_map(|e| e.get("id").and_then(Value::as_str).map(str::to_string))
        .collect())
}

/// D1 escape hatch for the impact door below: mirrors
/// `has_knowledge_freshness_deferral_decision` exactly, tagged
/// `impact-deferral` instead — returns the deferring decision's own
/// `decision` text so the door's detail line can quote the reason, never a
/// silent pass (D1: "an explicit recorded deferral with reason, never a
/// silent pass").
pub(crate) fn has_impact_deferral_decision(root: &Path, feature: &str) -> D<Option<String>> {
    let active = crate::verbs::decisions::active_decisions(root, false).map_err(|_| Delegate)?;
    let filtered = crate::verbs::decisions::filter_decision_events(
        active,
        &crate::verbs::decisions::DecisionFilters {
            tag: Some("impact-deferral".to_string()),
            feature: Some(feature.to_string()),
            ..Default::default()
        },
    )
    .map_err(|_| Delegate)?;
    Ok(filtered.last().and_then(|d| d.get("decision")).and_then(Value::as_str).map(|s| s.to_string()))
}

/// doc-impact-synthesis D1b: does a sweep hit's root-relative file fall
/// inside the write-guard's own generated/vendored tree list? Reused
/// verbatim from `hooks::write_guard::SCOUT_DIRS` (guards.rs, `check_read`'s
/// own match arm) rather than hand-copied — the guard is the one place that
/// list lives.
fn impact_sweep_in_generated_tree(root_relative_file: &str) -> bool {
    let normalized = root_relative_file.replace('\\', "/");
    crate::hooks::write_guard::SCOUT_DIRS
        .iter()
        .any(|dir| normalized.starts_with(*dir) || normalized.contains(&format!("/{dir}")))
}

/// doc-impact-synthesis D1b: the impact door. Walks each of the closing
/// feature's feature-stamped decision ids (`feature_stamped_decision_ids`
/// above) through the same citation sweep the log-time touches-sweep already
/// proved (`sweep_decision_citations`, render.rs:419), excluding the
/// generated decisions index and the feature's own live history dir via
/// `touches_sweep_excluded` (verbs_read.rs — the exact exclusion the
/// log-time sweep already uses, reused rather than re-derived) plus the
/// write-guard's generated-tree list above. Every surviving hit blocks,
/// naming file:line and the fix-and-rerun remedy: the sweep re-runs fresh on
/// every close, so a fixed citing doc clears itself on re-run — no stub, no
/// queue.
pub(crate) fn build_impact_door(root: &Path, feature: &str) -> D<Door> {
    let ids = feature_stamped_decision_ids(root, feature)?;
    let mut blocking_items: Vec<String> = Vec::new();
    for id in &ids {
        let short8 = crate::textutil::truncate_chars_head(id, 8);
        let sweep = crate::verbs::decisions::sweep_decision_citations(root, id, &short8);
        let hits = match sweep.get("files") {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        for hit in &hits {
            let file = hit.get("file").and_then(Value::as_str).unwrap_or("").to_string();
            if crate::verbs::decisions::touches_sweep_excluded(&file, Some(feature)) {
                continue;
            }
            if impact_sweep_in_generated_tree(&file) {
                continue;
            }
            let line = hit.get("line").and_then(Value::as_f64).unwrap_or(0.0) as u64;
            blocking_items.push(format!(
                "{file}:{line} cites decision {short8}, touched by closing feature \"{feature}\" — remedy: fix or annotate the citing doc, then re-run bee close --feature {feature} (the sweep re-runs fresh)."
            ));
        }
    }

    if blocking_items.is_empty() {
        return Ok(Door { door: "impact", blocking: false, detail: "clear".to_string(), command: None });
    }

    if let Some(reason) = has_impact_deferral_decision(root, feature)? {
        return Ok(Door {
            door: "impact",
            blocking: false,
            detail: format!(
                "deferred — {} citing doc(s) of decision(s) touched by \"{feature}\" ({}); a logged impact-deferral decision names \"{feature}\": {reason}",
                blocking_items.len(),
                blocking_items.join("; ")
            ),
            command: None,
        });
    }

    Ok(Door {
        door: "impact",
        blocking: true,
        detail: format!(
            "{} citing doc(s) of decision(s) touched by \"{feature}\": {}",
            blocking_items.len(),
            blocking_items.join("; ")
        ),
        command: None,
    })
}

// ── doc-impact-synthesis D2: routing door at close ──────────────────────────
//
// CONTEXT.md D2: every locked D-ID in the feature's own CONTEXT.md decision
// table must be routed — merged into an area's bundle citation (`context_
// table::context_table_covers_d_id`, plain/range/slash forms, or the
// decision's own logged short8) or explicitly recorded feature-local
// (a logged `feature-local`-tagged decision naming `<feature> D<n>`).
// Reuses `context_table::parse_locked_decision_ids` for the grammar, the
// same tag+feature text-scan `filter_decision_events` shape the deferral
// escapes below already use for `feature-local`, and `matches_whole_token`
// for both the plain-form and short8 citation checks.

/// D2 escape hatch: mirrors `has_impact_deferral_decision` exactly, tagged
/// `routing-deferral` instead.
pub(crate) fn has_routing_deferral_decision(root: &Path, feature: &str) -> D<Option<String>> {
    let active = crate::verbs::decisions::active_decisions(root, false).map_err(|_| Delegate)?;
    let filtered = crate::verbs::decisions::filter_decision_events(
        active,
        &crate::verbs::decisions::DecisionFilters {
            tag: Some("routing-deferral".to_string()),
            feature: Some(feature.to_string()),
            ..Default::default()
        },
    )
    .map_err(|_| Delegate)?;
    Ok(filtered.last().and_then(|d| d.get("decision")).and_then(Value::as_str).map(|s| s.to_string()))
}

/// D3 escape hatch: mirrors `has_impact_deferral_decision` exactly, tagged
/// `doc-deferral` instead.
pub(crate) fn has_doc_deferral_decision(root: &Path, feature: &str) -> D<Option<String>> {
    let active = crate::verbs::decisions::active_decisions(root, false).map_err(|_| Delegate)?;
    let filtered = crate::verbs::decisions::filter_decision_events(
        active,
        &crate::verbs::decisions::DecisionFilters {
            tag: Some("doc-deferral".to_string()),
            feature: Some(feature.to_string()),
            ..Default::default()
        },
    )
    .map_err(|_| Delegate)?;
    Ok(filtered.last().and_then(|d| d.get("decision")).and_then(Value::as_str).map(|s| s.to_string()))
}

/// The bare numeric suffix of a `D<n>` id (`"D12"` -> `Some(12)`).
fn d_id_number(d_id: &str) -> Option<u32> {
    d_id.strip_prefix('D')?.parse().ok()
}

/// D2: the logged `decide` event whose text names `<feature> D<n>` literally
/// (the convention this feature's own decision log already follows: `"doc-
/// impact-synthesis D2: at feature close, ..."`), if any — its own id,
/// truncated to short8 (`sweep_decision_citations`'s own key), so a bundle
/// file can cite the decision by hash instead of by name. `None` when no
/// logged decision names this D-id at all (the plain/range/slash citation
/// forms and the feature-local tag remain the D-id's only routes).
pub(crate) fn decision_short8_for_context_id(root: &Path, feature: &str, d_id: &str) -> D<Option<String>> {
    let active = crate::verbs::decisions::active_decisions(root, false).map_err(|_| Delegate)?;
    let token = format!("{feature} {d_id}");
    Ok(active
        .iter()
        .find(|e| {
            e.get("type").and_then(Value::as_str) == Some("decide")
                && e.get("decision")
                    .and_then(Value::as_str)
                    .map(|t| crate::verbs::decisions::matches_whole_token(&[t.to_string()], &token))
                    .unwrap_or(false)
        })
        .and_then(|e| e.get("id").and_then(Value::as_str))
        .map(|id| crate::textutil::truncate_chars_head(id, 8)))
}

/// D2: every `decide` event tagged `feature-local` whose feature-text-scan
/// matches `feature` (same text-scan `DecisionFilters.feature` shape the
/// deferral escapes use, not kds-1's structured field — feature-local
/// recording is a decisions-log call, not a close-time sweep target).
fn feature_local_decisions(root: &Path, feature: &str) -> D<Vec<Value>> {
    let active = crate::verbs::decisions::active_decisions(root, false).map_err(|_| Delegate)?;
    crate::verbs::decisions::filter_decision_events(
        active,
        &crate::verbs::decisions::DecisionFilters {
            tag: Some("feature-local".to_string()),
            feature: Some(feature.to_string()),
            ..Default::default()
        },
    )
    .map_err(|_| Delegate)
}

/// D2: the routing door itself. `None` from the parser (no canonical table —
/// legacy CONTEXT form, or the file is missing) degrades to a LOUD
/// report-only notice: named deviation from D2's letter, bounded by D4's
/// no-archaeology rationale — the door BLOCKS only for CONTEXT files
/// carrying the canonical grammar.
pub(crate) fn build_routing_door(root: &Path, feature: &str) -> D<Door> {
    let context_path = root.join("docs").join("history").join(feature).join("CONTEXT.md");
    let text = match std::fs::read_to_string(&context_path) {
        Ok(t) => t,
        Err(_) => {
            return Ok(Door {
                door: "routing",
                blocking: false,
                detail: format!(
                    "NOTICE — no docs/history/{feature}/CONTEXT.md found to route (legacy-form gap); the routing door never blocks on it — route it manually or fold it into the D4 historical-routing-sweep campaign backlog row"
                ),
                command: None,
            });
        }
    };
    let Some(ids) = parse_locked_decision_ids(&text) else {
        return Ok(Door {
            door: "routing",
            blocking: false,
            detail: format!(
                "NOTICE — docs/history/{feature}/CONTEXT.md has no canonical '## Locked Decisions' pipe table (legacy bullet/split form, the legacy-form gap); the routing door never blocks on it — route it manually or fold it into the D4 historical-routing-sweep campaign backlog row"
            ),
            command: None,
        });
    };
    if ids.is_empty() {
        return Ok(Door {
            door: "routing",
            blocking: false,
            detail: "clear — canonical table present, no locked decisions to route".to_string(),
            command: None,
        });
    }

    let bundle_files: Vec<(PathBuf, String)> = match crate::verbs::knowledge::bundle_dir(root) {
        Some(dir) => crate::verbs::knowledge::list_bundle_markdown(&dir)
            .unwrap_or_default()
            .into_iter()
            .map(|rel| (crate::verbs::knowledge::join_rel(&dir, &rel), rel))
            .collect(),
        None => Vec::new(),
    };
    let local_decisions = feature_local_decisions(root, feature)?;

    let mut unrouted: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    for d_id in &ids {
        let Some(d_num) = d_id_number(d_id) else { continue };
        let plain_token = format!("{feature} {d_id}");
        let short8 = decision_short8_for_context_id(root, feature, d_id)?;
        let mut citing: Vec<String> = Vec::new();
        for (abs, rel) in &bundle_files {
            let text = crate::verbs::knowledge::read_file_lossy(abs).unwrap_or_default();
            let plain_hit = crate::verbs::decisions::matches_whole_token(&[text.clone()], &plain_token);
            let range_hit = context_table_covers_d_id(&text, feature, d_num);
            let short8_hit = short8
                .as_deref()
                .map(|s8| crate::verbs::decisions::matches_whole_token(&[text.clone()], s8))
                .unwrap_or(false);
            if plain_hit || range_hit || short8_hit {
                citing.push(rel.clone());
            }
        }
        if !citing.is_empty() {
            if citing.len() > 1 {
                warnings.push(format!(
                    "{d_id} cited in {} bundle files ({}) — duplication, report-only",
                    citing.len(),
                    citing.join(", ")
                ));
            }
            continue;
        }
        let feature_local_hit = local_decisions.iter().any(|d| {
            d.get("decision")
                .and_then(Value::as_str)
                .map(|t| crate::verbs::decisions::matches_whole_token(&[t.to_string()], &plain_token))
                .unwrap_or(false)
        });
        if !feature_local_hit {
            unrouted.push(d_id.clone());
        }
    }

    if unrouted.is_empty() {
        let detail = if warnings.is_empty() {
            "clear — every locked D-ID routed".to_string()
        } else {
            format!(
                "clear — every locked D-ID routed; {} duplication warning(s), report-only: {}",
                warnings.len(),
                warnings.join("; ")
            )
        };
        return Ok(Door { door: "routing", blocking: false, detail, command: None });
    }

    if let Some(reason) = has_routing_deferral_decision(root, feature)? {
        return Ok(Door {
            door: "routing",
            blocking: false,
            detail: format!(
                "deferred — {} unrouted locked D-ID(s) ({}); a logged routing-deferral decision names \"{feature}\": {reason}",
                unrouted.len(),
                unrouted.join(", ")
            ),
            command: None,
        });
    }

    Ok(Door {
        door: "routing",
        blocking: true,
        detail: format!(
            "{} unrouted locked D-ID(s) in docs/history/{feature}/CONTEXT.md ({}) — remedy: cite \"{feature} <D-id>\" (plain, a range, a slash-list, or the decision's own short8) in a docs/knowledge/ bundle file, or log a decision tagged feature-local naming \"{feature} <D-id>\" to record it as feature-local",
            unrouted.len(),
            unrouted.join(", ")
        ),
        command: None,
    })
}

// ── doc-impact-synthesis D3: doc-deferral door at close ─────────────────────
//
// CONTEXT.md D3: deferral-shaped prose written into a touched doc must name
// a registered trigger id. Scan set = the closing feature's capped cells'
// `files_changed` filtered to `docs/`, UNION every file that exists on disk
// under `docs/history/<feature>/` (CONTEXT.md is written at shaping, before
// any cell exists — `feature_touched_files` alone would never see it).
// Full-text scan of that bounded set, no git — close spawns no git today
// and a merge-flow base is ill-defined post-merge (plan-check S2).

/// D3's bounded scan set: root-relative paths, deduped, insertion order —
/// `feature_touched_files` filtered to `docs/`, then every markdown file
/// under `docs/history/<feature>/` on disk (present or not, cell-touched or
/// not).
pub(crate) fn doc_deferral_scan_files(root: &Path, feature: &str) -> D<Vec<String>> {
    let touched = feature_touched_files(root, feature)?;
    let mut files: Vec<String> = touched.into_iter().filter(|f| f.starts_with("docs/")).collect();
    let history_dir = root.join("docs").join("history").join(feature);
    if let Some(rels) = crate::verbs::knowledge::list_bundle_markdown(&history_dir) {
        for rel in rels {
            let full = format!("docs/history/{feature}/{rel}");
            if !files.contains(&full) {
                files.push(full);
            }
        }
    }
    Ok(files)
}

/// Every trigger-id candidate cited on one line: a backtick span (`` `<id>`
/// ``) or a `[[trigger:<id>]]` span. Multiple candidates on one line are all
/// checked; any one resolving clears the line.
fn line_trigger_ids(line: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut in_tick = false;
    let mut start = 0usize;
    for (idx, ch) in line.char_indices() {
        if ch == '`' {
            if in_tick {
                ids.push(line[start..idx].trim().to_string());
                in_tick = false;
            } else {
                in_tick = true;
                start = idx + 1;
            }
        }
    }
    let mut rest = line;
    while let Some(pos) = rest.find("[[trigger:") {
        let after = &rest[pos + "[[trigger:".len()..];
        match after.find("]]") {
            Some(end) => {
                ids.push(after[..end].trim().to_string());
                rest = &after[end + 2..];
            }
            None => break,
        }
    }
    ids
}

/// One deferral-shaped, unresolved-by-citation line the scan loop found:
/// `rel`+`norm` are the baseline's identity key (never the line number —
/// D1), `message` is the unchanged `file:line …` wording the door has
/// always used.
struct DeferralCandidate {
    rel: String,
    norm: String,
    message: String,
}

/// D1's single normalization function — used identically when SEEDING the
/// baseline and when MATCHING against it later, per the Agent's Discretion
/// constraint in CONTEXT.md (one function, never two independently-drifting
/// answers to "same line?"). Trims surrounding whitespace only.
pub(crate) fn normalize_doc_deferral_line(line: &str) -> String {
    line.trim().to_string()
}

/// Per-file sets of already-baselined normalized line content, sorted (a
/// `BTreeMap`/`BTreeSet` pair, not a hash map) so writing it back out is
/// deterministic regardless of scan order.
type DocDeferralBaseline = std::collections::BTreeMap<String, std::collections::BTreeSet<String>>;

/// How many seed messages the `--dry-run` detail spells out before it
/// summarises the rest. The count is always exact; only the sample is capped.
pub(crate) const DOC_DEFERRAL_DRY_RUN_SAMPLE: usize = 20;

/// D3: the tracked, git-visible baseline file (D3) — beside `.bee/backlog.
/// jsonl`, not in the gitignored `.bee/state.json`/`.bee/runtime/` family.
pub(crate) fn doc_deferral_baseline_path(root: &Path) -> PathBuf {
    root.join(".bee").join("doc-deferral-baseline.json")
}

fn parse_doc_deferral_baseline(value: &Value) -> DocDeferralBaseline {
    let mut baseline = DocDeferralBaseline::new();
    if let Some(files) = value.get("files").and_then(Value::as_object) {
        for (rel, lines) in files {
            let Some(arr) = lines.as_array() else { continue };
            let set: std::collections::BTreeSet<String> =
                arr.iter().filter_map(Value::as_str).map(|s| s.to_string()).collect();
            baseline.insert(rel.clone(), set);
        }
    }
    baseline
}

/// Sorted keys, sorted per-file lines (BTree iteration order), inserted into
/// the JSON `Map` in that already-sorted order — byte-identical across runs
/// over an unchanged tree even though this crate's `serde_json` carries
/// `preserve_order` (insertion order, not automatic sorting).
fn doc_deferral_baseline_to_value(baseline: &DocDeferralBaseline) -> Value {
    let mut files = Map::new();
    for (rel, lines) in baseline {
        files.insert(rel.clone(), Value::Array(lines.iter().map(|l| Value::String(l.clone())).collect()));
    }
    let mut root = Map::new();
    root.insert("files".to_string(), Value::Object(files));
    Value::Object(root)
}

/// D6's SEED file set: every markdown file under `docs/`, REPO-WIDE, as
/// root-relative paths — deliberately NOT `doc_deferral_scan_files`, which
/// is per-FEATURE (the closing feature's capped cells' `files_changed` plus
/// `docs/history/<feature>/`). A scan-set-wide seed would freeze only the
/// docs one feature happened to touch, so the next feature touching a
/// different long-lived doc would enter enforcement against an empty entry
/// and block on every pre-existing line in it — the exact false-positive
/// class this whole feature exists to end, returning on a delay. Reuses
/// `list_bundle_markdown` (the same walker `doc_deferral_scan_files` already
/// uses for `docs/history/<feature>/`) rather than adding a second walker;
/// it skips symlinks, and its `None` (a path carrying a char at or above
/// U+E000) reads here as "no files", which seeds an empty baseline rather
/// than half of one.
///
/// ENFORCEMENT is untouched by this and stays per-feature over
/// `doc_deferral_scan_files` — freeze all existing debt once, then police
/// only what each feature touches.
fn doc_deferral_seed_files(root: &Path) -> Vec<String> {
    let docs_dir = root.join("docs");
    crate::verbs::knowledge::list_bundle_markdown(&docs_dir)
        .unwrap_or_default()
        .into_iter()
        .map(|rel| format!("docs/{rel}"))
        .collect()
}

/// The opening `<!-- bee:not-a-deferral: <reason> -->` marker's prefix and
/// the exact closing marker line. A line's trimmed content must match one
/// of these exactly to toggle the not-a-deferral exemption.
const NOT_A_DEFERRAL_OPEN_PREFIX: &str = "<!-- bee:not-a-deferral:";
const NOT_A_DEFERRAL_OPEN_SUFFIX: &str = "-->";
const NOT_A_DEFERRAL_CLOSE: &str = "<!-- /bee:not-a-deferral -->";

/// If `trimmed` is a `<!-- bee:not-a-deferral: <reason> -->` opening marker
/// carrying a non-empty reason, returns that reason. An empty or missing
/// reason returns `None` — skipping a guard is a named act, never an
/// oversight (the same posture `regen_obligation_ack` takes on a cell), so
/// an unreasoned marker opens nothing and the lines below it still block.
fn not_a_deferral_open_reason(trimmed: &str) -> Option<&str> {
    let rest = trimmed.strip_prefix(NOT_A_DEFERRAL_OPEN_PREFIX)?;
    let rest = rest.strip_suffix(NOT_A_DEFERRAL_OPEN_SUFFIX)?;
    let reason = rest.trim();
    if reason.is_empty() { None } else { Some(reason) }
}

/// The scan loop itself, unchanged in spirit from the pre-baseline door:
/// `doc_deferral_scan_files` for the file set (D1, untouched), `matches_
/// deferral_prose` for the word list (D1, untouched), the `in_fence` toggle
/// for the fenced-code exemption (D1, untouched), `line_trigger_ids` +
/// `trigger_registered` for the citation escape (D4, untouched) — this just
/// collects what used to go straight into `blocking_items` so the baseline
/// check can sit beside the loop instead of inside it.
fn doc_deferral_candidates(root: &Path, files: &[String]) -> Vec<DeferralCandidate> {
    let mut out = Vec::new();
    for rel in files {
        let Ok(text) = std::fs::read_to_string(root.join(rel)) else { continue };
        let mut in_fence = false;
        let mut in_marker = false;
        for (idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed == NOT_A_DEFERRAL_CLOSE {
                in_marker = false;
                continue;
            }
            if not_a_deferral_open_reason(trimmed).is_some() {
                in_marker = true;
                continue;
            }
            if line.trim_start().starts_with("```") {
                in_fence = !in_fence;
                continue;
            }
            if in_fence || in_marker {
                continue;
            }
            if !crate::verbs::decisions::matches_deferral_prose(line) {
                continue;
            }
            let resolved = line_trigger_ids(line)
                .iter()
                .any(|id| crate::verbs::triggers::trigger_registered(root, id));
            if resolved {
                continue;
            }
            out.push(DeferralCandidate {
                rel: rel.clone(),
                norm: normalize_doc_deferral_line(line),
                message: format!(
                    "{rel}:{} deferral-shaped prose with no registered trigger citation",
                    idx + 1
                ),
            });
        }
    }
    out
}

/// D3: the doc-deferral door itself. A line matching `matches_deferral_
/// prose` inside a ``` fenced block is exempt; every other match needs a
/// same-line trigger citation resolving via `trigger_registered`, OR its
/// normalized content already present in the tracked baseline (D1), or it
/// blocks, naming file:line and the create-the-trigger teach line.
///
/// D2 + D6: a repo with no baseline file seeds it, REPO-WIDE — the seed
/// walks every markdown file under `docs/` (`doc_deferral_seed_files`), not
/// the per-feature scan set enforcement uses. On a REAL run (`dry_run` is
/// false) it records every line still flagged after the citation escape and
/// ALWAYS writes the file, even when it flagged nothing (an absent file IS
/// the seed state, so skipping the write would leave the next close free to
/// adopt the first genuine deferral line ever added); on `--dry-run` (D5) it
/// writes nothing and reports non-blocking, naming the repo-wide count a
/// real close would freeze. Every run after that only reads the baseline —
/// nothing is ever adopted back into it automatically (D2/D4).
pub(crate) fn build_doc_deferral_door(root: &Path, feature: &str, dry_run: bool) -> D<Door> {
    let files = doc_deferral_scan_files(root, feature)?;
    let candidates = doc_deferral_candidates(root, &files);
    let baseline_path = doc_deferral_baseline_path(root);

    let new_items: Vec<&DeferralCandidate> = match read_json(&baseline_path) {
        ReadJson::Missing => {
            // D6: the seed is REPO-WIDE. `candidates` above is the
            // per-feature ENFORCEMENT set and is deliberately ignored here —
            // seeding from it would freeze only this feature's own docs.
            let seed_files = doc_deferral_seed_files(root);
            let seed_candidates = doc_deferral_candidates(root, &seed_files);
            if dry_run {
                // D5: `--dry-run` writes NOTHING, ever. It still has to
                // predict the verdict honestly — and the verdict is
                // non-blocking either way, so a seed that would freeze
                // nothing reports the same plain "clear" every enforcing
                // run reports.
                if seed_candidates.is_empty() {
                    return Ok(Door {
                        door: "doc-deferral",
                        blocking: false,
                        detail: "clear".to_string(),
                        command: None,
                    });
                }
                // The repo-wide seed set runs to four figures on a real docs
                // tree, and this detail is printed AND embedded in the JSON
                // doors payload. Name the count, then show a sample — D5 asks
                // for an honest prediction, not a transcript of it.
                let shown = seed_candidates.len().min(DOC_DEFERRAL_DRY_RUN_SAMPLE);
                let messages: Vec<String> =
                    seed_candidates.iter().take(shown).map(|c| c.message.clone()).collect();
                let remainder = seed_candidates.len() - shown;
                let tail = if remainder > 0 {
                    format!("; and {remainder} more")
                } else {
                    String::new()
                };
                return Ok(Door {
                    door: "doc-deferral",
                    blocking: false,
                    detail: format!(
                        "SEED (dry-run) — no baseline file yet; a real `bee close` would baseline {} pre-existing deferral line(s) across {} markdown file(s) under docs/, repo-wide, and pass: {}{}",
                        seed_candidates.len(),
                        seed_files.len(),
                        messages.join("; "),
                        tail
                    ),
                    command: None,
                });
            }
            // D6: ALWAYS write, even with nothing to record. An absent file
            // IS the seed state — a skipped write leaves the repo in SEED,
            // so the next close reads `Missing` again and ADOPTS whatever it
            // finds, swallowing the first genuine deferral line anyone adds.
            // An empty-`files` baseline is NOT equivalent to an absent one:
            // it takes the `Parsed` arm below, where every candidate is new
            // and blocks.
            let mut baseline: DocDeferralBaseline = DocDeferralBaseline::new();
            for c in &seed_candidates {
                baseline.entry(c.rel.clone()).or_default().insert(c.norm.clone());
            }
            write_json_atomic(&baseline_path, &doc_deferral_baseline_to_value(&baseline)).map_err(|_| Delegate)?;
            Vec::new()
        }
        ReadJson::Corrupt => candidates.iter().collect(),
        ReadJson::Parsed(v) => {
            let baseline = parse_doc_deferral_baseline(&v);
            candidates
                .iter()
                .filter(|c| !baseline.get(&c.rel).map(|set| set.contains(&c.norm)).unwrap_or(false))
                .collect()
        }
    };

    if new_items.is_empty() {
        return Ok(Door { door: "doc-deferral", blocking: false, detail: "clear".to_string(), command: None });
    }

    let messages: Vec<String> = new_items.iter().map(|c| c.message.clone()).collect();

    if let Some(reason) = has_doc_deferral_decision(root, feature)? {
        return Ok(Door {
            door: "doc-deferral",
            blocking: false,
            detail: format!(
                "deferred — {} deferral line(s) with no registered trigger ({}); a logged doc-deferral decision names \"{feature}\": {reason}",
                new_items.len(),
                messages.join("; ")
            ),
            command: None,
        });
    }

    Ok(Door {
        door: "doc-deferral",
        blocking: true,
        detail: format!(
            "{} deferral line(s) with no registered trigger citation: {} — remedy: register the condition first with `bee triggers add --decision <id> --condition \"...\"`, then cite it inline (backtick `<id>` or [[trigger:<id>]]), or if the prose documents deferral machinery rather than promising to act later, wrap it in a reasoned <!-- bee:not-a-deferral: <reason> --> ... <!-- /bee:not-a-deferral --> block",
            new_items.len(),
            messages.join("; ")
        ),
        command: None,
    })
}

/// U3 (docs/history/knowledge-usable/CONTEXT.md): past the configured
/// `capture_queue_threshold` — the pending count exceeds it, OR the oldest
/// pending stub is older than the configured day count — the capture-queue
/// door's detail escalates to name the breach. The door stays report-only
/// (`blocking: false`, decision c8e25271's deferral, untouched by U3) either
/// way; only the wording changes.
pub(crate) fn capture_queue_door_detail(root: &Path, queue: usize, oldest_ms: f64) -> String {
    if queue == 0 {
        return "clear".to_string();
    }
    let config = read_config_raw(root);
    let threshold = capture_queue_threshold(&config);
    let oldest_age_days = if oldest_ms.is_nan() { None } else { Some((now_ms() - oldest_ms) / 86_400_000.0) };
    let over_count = queue as u64 > threshold.count;
    let over_age = oldest_age_days.map(|d| d > threshold.days).unwrap_or(false);
    if over_count || over_age {
        let oldest_days = oldest_age_days.unwrap_or(0.0).max(0.0).floor() as u64;
        return format!(
            "OVERDUE — {queue} stub(s) pending, oldest {oldest_days} days — flush before new work; settle via bee-capturing"
        );
    }
    format!("pending — {queue} capture stub(s) awaiting flush; settle later via bee-capturing")
}

/// provenance: bee.mjs buildCloseReportDoors, extended by D1 — the
/// capture-queue door stays report-only (decision c8e25271's blanket
/// deferral, untouched here), but the scribing-debt door now BLOCKS close
/// when the feature has behavior_change cells with no capture recorded and
/// no logged `capture-deferral` decision names the feature (CONTEXT.md D1).
pub(crate) fn build_close_report_doors(root: &Path, feature: &str) -> D<Vec<Door>> {
    let scribing = scribing_debt(root, feature)?;
    let deferred = if scribing.count > 0 {
        has_capture_deferral_decision(root, feature)?
    } else {
        false
    };
    let scribing_blocking = scribing.count > 0 && !deferred;
    let mut doors = Vec::new();
    doors.push(Door {
        door: "scribing-debt",
        blocking: scribing_blocking,
        detail: if scribing.count == 0 {
            "clear".to_string()
        } else if scribing_blocking {
            format!(
                "pending — {} behavior_change cell(s) uncaptured ({}); run bee-capturing to record the capture, or log a decision tagged capture-deferral naming \"{feature}\" to defer it",
                scribing.count,
                js_join(&scribing.ids, ", ")
            )
        } else {
            format!(
                "deferred — {} behavior_change cell(s) uncaptured ({}); a logged capture-deferral decision names \"{feature}\"",
                scribing.count,
                js_join(&scribing.ids, ", ")
            )
        },
        command: if scribing_blocking { Some("bee-capturing") } else { None },
    });
    let (queue, oldest_ms) = capture_queue_pending(root);
    doors.push(Door {
        door: "capture-queue",
        blocking: false,
        detail: capture_queue_door_detail(root, queue, oldest_ms),
        command: None,
    });

    // wl-3: the judge-debt door exists ONLY for a standard/high-risk
    // closing route — a tiny/small feature never grows this door at all
    // (AGENTS.md's own judge-on-smell carve-out for those lanes), not
    // merely a non-blocking one, so `doors.iter().find(|d| d.door ==
    // "judge-debt")` returns `None` below `standard`.
    if matches!(feature_route(root, feature)?.as_deref(), Some("standard") | Some("high-risk")) {
        let judge = judge_debt(root, feature)?;
        // hpf-1: mirrors the scribing-debt escape above — a logged
        // `judge-deferral` decision naming this feature clears the door
        // without touching the count.
        let judge_deferred = if judge.count > 0 { has_judge_deferral_decision(root, feature)? } else { false };
        let judge_blocking = judge.count > 0 && !judge_deferred;
        // hpf-1: `cells judge-record` refuses an archived cell outright
        // (verbs/cells/util.rs assert_not_archived) — when an offending id
        // only resolves under the archive, the remedy must name the
        // unarchive step FIRST or it sends the reader straight into that
        // refusal.
        let archived_ids: Vec<&str> = judge
            .ids
            .iter()
            .filter_map(|id| id.as_str())
            .filter(|id| {
                !crate::verbs::cells::cell_file(root, id).exists()
                    && crate::verbs::cells::resolve_cell_file(root, id).is_some()
            })
            .collect();
        let judge_command = if !judge_blocking {
            None
        } else if archived_ids.is_empty() {
            Some("bee cells judge-record")
        } else {
            Some("bee cells unarchive")
        };
        doors.push(Door {
            door: "judge-debt",
            blocking: judge_blocking,
            detail: if judge.count == 0 {
                "clear".to_string()
            } else if judge_deferred {
                format!(
                    "deferred — {} behavior_change cell(s) capped with no judge record ({}); a logged judge-deferral decision names \"{feature}\"",
                    judge.count,
                    js_join(&judge.ids, ", ")
                )
            } else if !archived_ids.is_empty() {
                format!(
                    "{} behavior_change cell(s) capped with no judge record ({}); {} archived — run bee cells unarchive --feature {feature} first, then bee cells judge to check, then bee cells judge-record to record a verdict",
                    judge.count,
                    js_join(&judge.ids, ", "),
                    archived_ids.len(),
                )
            } else {
                format!(
                    "{} behavior_change cell(s) capped with no judge record ({}); run bee cells judge to check, then bee cells judge-record to record a verdict",
                    judge.count,
                    js_join(&judge.ids, ", ")
                )
            },
            command: judge_command,
        });
    }

    // uat-stop-placement D4.4/D2 (docs/history/uat-stop-placement/CONTEXT.md):
    // built and ordered beside the judge-debt door just above — same
    // lane-scoped, blocking, deferral-escapable shape. The door is PRESENT
    // ONLY under `UatStop::Close`: under `Merge` the stop already lives at
    // `bee worktree merge` (untouched here), and under `Off` there is no
    // uat stop anywhere, so neither placement grows this door at all. A
    // `None` from `uat_stop_config` (a bogus `uat_stop`/`uat_before_merge`
    // value) still grows the door, but BLOCKING with an invalid-config
    // detail naming both keys and their legal values — the two-key read
    // order is ambiguous on a typo, and guessing either way is worse than
    // refusing.
    // slp-dissent-stop-and-ask sd-4: the dissent-debt door (a2affcba,
    // 4b7aa303). It copies the judge-debt arm above for SHAPE only, and
    // deliberately drops three of its parts:
    //
    //   1. NO LANE GATE. The judge arm exists only for a standard/high-risk
    //      route. a2affcba is unconditional, so this door exists in EVERY
    //      lane, `tiny` included. A dissent record only exists because a
    //      worker wrote one, so its existence is the gate.
    //   2. NO GRANDFATHER CUTOFF. A dissent record cannot predate the feature
    //      that created it.
    //   3. NO `behavior_change` FILTER. A worker dissents against any cell it
    //      was handed.
    //
    // The count and the escape are read from verbs/cells/dissent.rs, never
    // recomputed here, because the merge door reads the SAME two functions:
    // one obligation, two doors.
    //
    // `cells dissent-verdict` writes through `mutate_cell`, which refuses an
    // archived cell outright, so an offender that only resolves under the
    // archive owes the unarchive step FIRST or the remedy sends the reader
    // straight into that refusal. Same reasoning the judge arm records.
    let dissent = crate::verbs::cells::feature_dissent_debt(root, feature)?;
    let dissent_deferred = if dissent.count > 0 {
        crate::verbs::cells::has_dissent_deferral_decision(root, feature)?
    } else {
        false
    };
    let dissent_blocking = dissent.count > 0 && !dissent_deferred;
    let dissent_archived: Vec<&str> = dissent
        .ids
        .iter()
        .filter_map(|id| id.as_str())
        .filter(|id| {
            !crate::verbs::cells::cell_file(root, id).exists()
                && crate::verbs::cells::resolve_cell_file(root, id).is_some()
        })
        .collect();
    doors.push(Door {
        door: "dissent-debt",
        blocking: dissent_blocking,
        detail: if dissent.count == 0 {
            "clear".to_string()
        } else if dissent_deferred {
            format!(
                "deferred — {} cell(s) carry a dissent with no verdict ({}); a logged dissent-deferral decision names \"{feature}\"",
                dissent.count,
                js_join(&dissent.ids, ", ")
            )
        } else if !dissent_archived.is_empty() {
            format!(
                "{} cell(s) carry a dissent with no verdict ({}); {} archived — run bee cells unarchive --feature {feature} first, then bee cells dissent-verdict to record a verdict, or log a decision tagged dissent-deferral naming \"{feature}\" to defer it",
                dissent.count,
                js_join(&dissent.ids, ", "),
                dissent_archived.len(),
            )
        } else {
            format!(
                "{} cell(s) carry a dissent with no verdict ({}); run bee cells dissent-verdict to record a verdict, or log a decision tagged dissent-deferral naming \"{feature}\" to defer it",
                dissent.count,
                js_join(&dissent.ids, ", ")
            )
        },
        command: if !dissent_blocking {
            None
        } else if dissent_archived.is_empty() {
            Some("bee cells dissent-verdict")
        } else {
            Some("bee cells unarchive")
        },
    });

    // slp-advisor-nudge an-3: the advisor-nudge response debt (9e5eda5b). It
    // copies the dissent arm right above — NOT the judge arm's standard-up
    // lane gate — and for the same reason a2affcba gave the dissent one: the
    // nudge only exists because a supervisor wrote a record about THIS work,
    // so its existence is the gate and a lane gate would double-filter it.
    //
    // Three parts of the dissent arm are deliberately absent here:
    //
    //   1. NO ARCHIVE ARM. The offenders are MAILBOX ROW ids, never cell ids
    //      — nothing about them can be under `.bee/cells/archive`, so there
    //      is no unarchive step to name first.
    //   2. NO FEATURE-LEVEL ESCAPE. The dissent door's `dissent-deferral`
    //      decision lifts every dissent in the feature at once; 9e5eda5b puts
    //      the obligation on each nudge, so the escape is per ROW and lives
    //      inside the count itself (a cleared row is simply not counted).
    //      That is why there is no `deferred` detail branch below.
    //   3. NO SECOND READING. The count comes from
    //      `feature_advisor_nudge_debt` (verbs/supervisor.rs), the SAME
    //      function the cap path and the merge door call: one obligation,
    //      three doors.
    let nudge = crate::verbs::supervisor::feature_advisor_nudge_debt(root, feature)?;
    doors.push(Door {
        door: "advisor-nudge-debt",
        blocking: nudge.count > 0,
        detail: if nudge.count == 0 {
            "clear".to_string()
        } else {
            format!(
                "{} advisor nudge(s) unanswered ({}); run the consult, then log a decision tagged advisor-nudge whose text NAMES that row id — or record a reasoned decline the same way",
                nudge.count,
                js_join(&nudge.ids, ", ")
            )
        },
        command: if nudge.count == 0 { None } else { Some("bee decisions log") },
    });

    match crate::uat::uat_stop_config(root) {
        None => {
            doors.push(Door {
                door: "uat",
                blocking: true,
                detail: "invalid config — \"uat_stop\" must be \"merge\", \"close\", or \"off\"; the legacy \"uat_before_merge\" must be a boolean (true reads as \"merge\", false reads as \"off\") — fix .bee/config.json".to_string(),
                command: None,
            });
        }
        Some(crate::uat::UatStop::Close) => {
            // usp-3 revision (D4): classify through `crate::uat::uat_lane_mode`
            // — the SAME lane-mode read the merge side's `uat_merge_precheck`
            // uses — not `feature_route`. `feature_route` prefers a lane
            // record's `route.lane`, which can name a different class than
            // the record's `mode`; the merge side is canonical, so the
            // close-time door must read what the merge side reads, or the
            // two ends of the uat stop can disagree and the stop vanishes
            // silently under `uat_stop: "close"`. Missing or unknown still
            // fails closed as "standard" (applies), only
            // tiny/small/docs/spike are exempt.
            let lane = crate::uat::uat_lane_mode(root, feature);
            let lane_applies = crate::uat::uat_gate_applies_to_lane(lane.as_deref());
            let gate_approved = lane_applies && crate::uat::uat_gate_approved(root, feature);
            let uat_deferred =
                if lane_applies && !gate_approved { has_uat_deferral_decision(root, feature)? } else { false };
            let uat_blocking = lane_applies && !gate_approved && !uat_deferred;
            doors.push(Door {
                door: "uat",
                blocking: uat_blocking,
                detail: if !lane_applies {
                    "clear — this lane is exempt from the close-time uat door".to_string()
                } else if gate_approved {
                    "clear".to_string()
                } else if uat_deferred {
                    format!("deferred — the uat gate for \"{feature}\" is not yet approved; a logged uat-deferral decision names \"{feature}\"")
                } else {
                    format!(
                        "pending — the uat gate for \"{feature}\" is not yet approved; the product is on main now — reload it, test it, then bee gate --name uat --approved true, or fix in the worktree and merge again"
                    )
                },
                command: if uat_blocking { Some("bee gate --name uat --approved true") } else { None },
            });
        }
        Some(crate::uat::UatStop::Merge) | Some(crate::uat::UatStop::Off) => {}
    }
    Ok(doors)
}

// ── U7: close-time pattern-check door ───────────────────────────────────────
//
// docs/history/knowledge-usable/CONTEXT.md U7 (PBI p-21583c96): a report-only
// door that maps the feature's capped cells' touched files to the bundle
// areas they reach — the SAME per-file `touches_subject` match promote.rs's
// own area-update section already applies (decision b032be35: a concept's
// own bundle path plus its recorded `bee.sources`), just unscoped to any one
// work item — then lists the `bee.critical: true` patterns (ku-6's re-graded
// pool, docs/knowledge/areas/okf-profile/critical-bar.md) tagged with any of
// those areas. Smallest transport `bee close` already supports: one new
// value flag, `--pattern-verdicts=<pattern-id>:<verdict>[,<pattern-id>:
// <verdict>...]` (verdict one of violated/respected/not-applicable) — never
// a new answers-file format. A pattern with no matching verdict reports
// `pending` in the detail line and never blocks; a recorded `violated`
// blocks close exactly like a red test, naming the pattern.
pub(crate) const CLOSE_PATTERN_VIOLATED_PREFIX: &str = "Pattern violated for";

pub(crate) const PATTERN_VERDICT_WORDS: [&str; 3] = ["violated", "respected", "not-applicable"];

/// Parses `--pattern-verdicts`' whole value in one pass. A pair with no `:`,
/// an empty id, or a word outside `PATTERN_VERDICT_WORDS` is silently
/// dropped — that pattern reports `pending`, same as one never mentioned at
/// all; malformed input never fails close (the door is report-only until a
/// `violated` verdict is actually recorded).
pub(crate) fn parse_pattern_verdicts(raw: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for pair in raw.split(',') {
        let pair = js_trim(pair);
        let Some(idx) = pair.find(':') else { continue };
        let id = js_trim(&pair[..idx]);
        let verdict = js_trim(&pair[idx + 1..]).to_lowercase();
        if id.is_empty() || !PATTERN_VERDICT_WORDS.contains(&verdict.as_str()) {
            continue;
        }
        out.insert(id.to_string(), verdict);
    }
    out
}

/// The feature's touched files: `files_changed` off every capped cell (live
/// store + archive — the same read `scribing_debt` above uses), deduped,
/// insertion order.
pub(crate) fn feature_touched_files(root: &Path, feature: &str) -> D<Vec<String>> {
    let mut files: Vec<String> = Vec::new();
    for cell in list_cells_including_archive(root, feature, Some("capped"))? {
        let trace = vget(&cell, "trace").cloned().unwrap_or(Value::Object(Map::new()));
        if let Some(Value::Array(items)) = vget(&trace, "files_changed") {
            for item in items {
                if let Value::String(s) = item {
                    if !files.contains(s) {
                        files.push(s.clone());
                    }
                }
            }
        }
    }
    Ok(files)
}

/// Bundle areas the touched files reach. A concept with no `bee.sources`
/// naming a code path (most area concepts, per this very corpus's own
/// pattern `20260805-a-derived-field-empty-for-a-whole-class-of-inputs...`)
/// simply never matches on files alone — the door degrades to `clear`
/// rather than to a wrong answer.
pub(crate) fn touched_bundle_areas(dir: &Path, touched_files: &[String]) -> Vec<String> {
    let Some(concepts) = collect_concepts(dir) else { return Vec::new() };
    let mut areas: Vec<String> = Vec::new();
    for concept in &concepts {
        let bee = bee_of(&concept.data);
        let concept_areas = str_array(&bee, "areas");
        if concept_areas.is_empty() {
            continue;
        }
        let mut subjects: Vec<String> = vec![format!("docs/knowledge/{}", concept.path)];
        subjects.extend(str_array(&bee, "sources").into_iter().filter(|s| !s.is_empty()));
        let touched = touched_files.iter().any(|f| subjects.iter().any(|s| touches_subject(f, s)));
        if !touched {
            continue;
        }
        for a in concept_areas {
            if !areas.contains(&a) {
                areas.push(a);
            }
        }
    }
    areas
}

pub(crate) struct CriticalPattern {
    pub(crate) id: String,
    pub(crate) title: String,
}

/// Every `bee.critical: true` concept tagged with at least one of `areas` —
/// the same predicate `bee knowledge index`'s "Critical patterns" section
/// applies (verbs/knowledge/index.rs), scoped down to the touched areas.
pub(crate) fn critical_patterns_for_areas(dir: &Path, areas: &[String]) -> Vec<CriticalPattern> {
    let Some(concepts) = collect_concepts(dir) else { return Vec::new() };
    let mut out: Vec<CriticalPattern> = concepts
        .iter()
        .filter(|c| {
            let bee = bee_of(&c.data);
            matches!(bee.get("critical"), Some(Value::Bool(true)))
                && str_array(&bee, "areas").iter().any(|a| areas.contains(a))
        })
        .map(|c| {
            let bee = bee_of(&c.data);
            let id = bee.get("id").and_then(Value::as_str).unwrap_or(&c.path).to_string();
            let title = str_field(&c.data, "title").unwrap_or(&c.path).to_string();
            CriticalPattern { id, title }
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// U7's door itself: `blocking` is true the moment ANY matched pattern's
/// verdict is `violated` — the one condition that stops close "exactly like
/// a red test" (CONTEXT.md U7). `respected`/`not-applicable` pass silently;
/// an unanswered pattern reports `pending` in the detail line and never
/// blocks. No bundle, no touched files, no touched area, or no critical
/// pattern in a touched area all collapse to the same `clear` report — the
/// door has nothing to ask for.
pub(crate) fn build_pattern_check_door(root: &Path, feature: &str, verdicts: &HashMap<String, String>) -> D<Door> {
    if !crate::hooks::session_preamble::bundle_mode(root) {
        return Ok(Door { door: "pattern-check", blocking: false, detail: "clear".to_string(), command: None });
    }
    let Some(dir) = crate::verbs::knowledge::bundle_dir(root) else {
        return Ok(Door { door: "pattern-check", blocking: false, detail: "clear".to_string(), command: None });
    };
    let touched_files = feature_touched_files(root, feature)?;
    let touched_areas = touched_bundle_areas(&dir, &touched_files);
    let patterns = critical_patterns_for_areas(&dir, &touched_areas);
    if patterns.is_empty() {
        return Ok(Door { door: "pattern-check", blocking: false, detail: "clear".to_string(), command: None });
    }
    let mut rows: Vec<String> = Vec::new();
    let mut violated: Vec<String> = Vec::new();
    let mut pending = false;
    for p in &patterns {
        let verdict = verdicts.get(&p.id).cloned().unwrap_or_else(|| "pending".to_string());
        if verdict == "violated" {
            violated.push(format!("{} ({})", p.id, p.title));
        }
        if verdict == "pending" {
            pending = true;
        }
        rows.push(format!("{}={verdict}", p.id));
    }
    let mut detail = format!(
        "{} critical pattern(s) in touched area(s) [{}]: {}",
        patterns.len(),
        touched_areas.join(", "),
        rows.join(", ")
    );
    if pending {
        detail.push_str(
            " — unanswered pattern(s) report pending; supply a verdict via \
             --pattern-verdicts=<pattern-id>:<violated|respected|not-applicable>[,<pattern-id>:<verdict>...]",
        );
    }
    Ok(Door { door: "pattern-check", blocking: !violated.is_empty(), detail, command: None })
}

/// JS Array.prototype.join (null/undefined render empty).
pub(crate) fn js_join(items: &[Value], sep: &str) -> String {
    items
        .iter()
        .map(|v| match v {
            Value::Null => String::new(),
            other => jsjson::js_to_string(other),
        })
        .collect::<Vec<_>>()
        .join(sep)
}

/// provenance: bee.mjs renderCloseDoorLines.
pub(crate) fn render_close_door_lines(doors: &[Door]) -> Vec<String> {
    doors
        .iter()
        .map(|d| {
            if !d.blocking && d.detail == "clear" {
                return format!("door {}: clear", d.door);
            }
            format!(
                "door {}: {} — {}{}",
                d.door,
                if d.blocking { "BLOCKING" } else { "open" },
                d.detail,
                match d.command {
                    Some(c) => format!(" | settle: {c}"),
                    None => String::new(),
                }
            )
        })
        .collect()
}

/// D7/D8: the tests door's own wording over a `feature_proof_check` verdict
/// (verbs/cells/proof.rs) — the helper hands back counts and offending ids,
/// same split `scribing_debt`/`judge_debt` already use, and this is the one
/// place that turns them into the door's prose so the dry-run listing, the
/// refusal, and the GREEN headline can never describe the same verdict
/// differently.
fn proof_door_detail(proof: &crate::verbs::cells::ProofCheck) -> String {
    if proof.blocking {
        return format!(
            "{} capped cell(s) carry a report with no valid proof line ({}) — re-cap with a real proof line: \"<command> — <result> — <scope reason>\".",
            proof.bad_ids.len(),
            proof.bad_ids.join(", ")
        );
    }
    if proof.proven_count == 0 && proof.legacy_count == 0 {
        return "no capped cells yet — nothing to prove".to_string();
    }
    if proof.legacy_count > 0 {
        return format!(
            "{} capped cell(s) carry a proof line; {} legacy cap(s) with no report record pass ungated (pre-contract)",
            proof.proven_count, proof.legacy_count
        );
    }
    format!("{} capped cell(s) all carry a proof line", proof.proven_count)
}

/// merge-ready-fact D2: record WHY this close is still standing, onto the
/// feature's stored `merge_ready` fact — the names of every BLOCKING door
/// except `"uat"`, in the same order the result's own `doors` array carries
/// them, and `[]` on a green close. `"uat"` is left out because the fact
/// carries the uat answer as its own `uat` field (`bee gate --name uat`
/// writes it), never twice.
///
/// Called the MOMENT a full doors vector exists and BEFORE any early-return
/// refusal arm, so a close that stops at a door still records the door it
/// stopped at rather than leaving the last green answer standing.
///
/// FAIL-OPEN and result-neutral, by construction: `set_blocked_by` never
/// creates the fact (a feature that is not merge-ready has nothing to
/// write), never throws, and its answer is deliberately dropped here — D3
/// makes this an additive projection nothing in bee ever reads back, so
/// there is no failure for close to report and no door it could change.
fn record_merge_ready_blocked_by(root: &Path, feature: &str, doors: &[Door]) {
    let names: Vec<&str> =
        doors.iter().filter(|d| d.blocking && d.door != "uat").map(|d| d.door).collect();
    let _ = crate::verbs::workflow_store::merge_ready::set_blocked_by(root, feature, &names);
}

// ═══ the token-usage section (decision 2d3abd12) ═══════════════════════════
//
// WHAT IT ANSWERS. "What did this feature cost in tokens?" — asked at the one
// moment the answer is final, and answered from the only place that already
// holds the truth: the Claude transcript each session wrote. No new store, no
// new counter, nothing to keep in sync.
//
// WHY NOT `performance.jsonl`. That record is written by the session-close
// hook, so it is stale for the session that is running close RIGHT NOW — the
// most interesting one — and it is keyed by project, not by feature. Both
// alternatives are named as rejected in decision 2d3abd12.
//
// FAIL-SOFT, ALWAYS. This section is a report line on an already-green close.
// Every read below degrades to "skipped" instead of erroring: a session with
// no record, no stored `transcript_path`, a path that no longer exists on
// disk (transcripts are outside the repo and get cleaned up), or a transcript
// with no events at all. The skipped count is printed rather than swallowed —
// a total summed over 1 of 4 sessions must never read like a total over 4.

/// The three numbers a `Rollup`'s model entries already carry, summed across
/// every model. `ModelAcc::finalize` computed `new`/`cached`/`total` before
/// serialization (`hooks/session_close/perf.rs`), so this only ever ADDS them
/// up — it never re-derives a total from input/output/cache fields, which
/// would be a second, drift-prone definition of the same number.
#[derive(Clone, Copy, Default)]
pub(crate) struct UsageBucket {
    pub(crate) new_t: f64,
    pub(crate) cached: f64,
    pub(crate) total: f64,
}

impl UsageBucket {
    fn add_models(&mut self, models: &Value) {
        let Value::Object(map) = models else { return };
        for m in map.values() {
            self.new_t += crate::hooks::session_close::num_field(m, "new");
            self.cached += crate::hooks::session_close::num_field(m, "cached");
            self.total += crate::hooks::session_close::num_field(m, "total");
        }
    }

    fn plus(&self, other: &UsageBucket) -> UsageBucket {
        UsageBucket {
            new_t: self.new_t + other.new_t,
            cached: self.cached + other.cached,
            total: self.total + other.total,
        }
    }

    fn value(&self) -> Value {
        json!({ "new": self.new_t, "cached": self.cached, "total": self.total })
    }
}

/// One rolled-up session, kept whole (decision e97cc9d4).
///
/// The printed line only ever needed the sums, but the RECORD close writes
/// beside it answers "where did this feature's tokens go" — which model, how
/// many subagents, over what span. Every field is copied straight off the
/// `Rollup` the transcript already produced; nothing here is re-derived, for
/// the same reason [`UsageBucket::add_models`] only ever adds.
///
/// `totals` is this session's WHOLE cost — its own models plus its subagents'
/// — because that is the number a per-session reader wants first. The split
/// is never lost: `models` and `subagent_models` ride beside it.
pub(crate) struct SessionUsage {
    pub(crate) session_id: String,
    pub(crate) models: Value,
    pub(crate) subagent_models: Value,
    pub(crate) subagent_count: usize,
    pub(crate) started_ms: Option<f64>,
    pub(crate) ended_ms: Option<f64>,
    pub(crate) totals: UsageBucket,
}

impl SessionUsage {
    fn value(&self) -> Value {
        json!({
            "session_id": self.session_id,
            "models": self.models,
            "subagent_models": self.subagent_models,
            "subagent_count": self.subagent_count,
            // Null, never 0, when the transcript carried no timestamp: a span
            // that could not be read must not read like an instant one.
            "started_ms": self.started_ms,
            "ended_ms": self.ended_ms,
            "totals": self.totals.value(),
        })
    }
}

/// The whole usage section: what was read, what could not be, and the two
/// buckets. `details` holds the transcripts actually ROLLED UP, never the
/// candidates — with `skipped` beside it the reader can always tell a
/// complete total from a partial one.
///
/// The session COUNT is [`CloseUsage::sessions`], derived from `details` and
/// never stored beside it: one number, one home, so the printed line and the
/// written record can never disagree about how many sessions were read.
pub(crate) struct CloseUsage {
    pub(crate) details: Vec<SessionUsage>,
    pub(crate) skipped: usize,
    pub(crate) main: UsageBucket,
    pub(crate) subagents: UsageBucket,
}

impl CloseUsage {
    pub(crate) fn sessions(&self) -> usize {
        self.details.len()
    }

    fn total(&self) -> UsageBucket {
        self.main.plus(&self.subagents)
    }

    pub(crate) fn value(&self) -> Value {
        json!({
            "sessions": self.sessions(),
            "skipped": self.skipped,
            "main": self.main.value(),
            "subagents": self.subagents.value(),
            "total": self.total().value(),
        })
    }
}

/// Every session id whose usage belongs to this feature: each `.bee/sessions/
/// *.json` record bound to the lane, PLUS the session running close (which
/// may not be lane-bound at all, and whose own record is the freshest and
/// least likely to be in any rolled-up log yet).
///
/// Sorted and deduped. The sums are order-insensitive, but a deterministic
/// candidate order keeps the skipped count and any future per-session detail
/// reproducible across runs — the same reason `walk_subagents` sorts its own
/// directory listing.
pub(crate) fn usage_session_ids(
    control: &Path,
    feature: &str,
    calling_session: Option<&str>,
) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(crate::verbs::cells::sessions_dir(control)) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            // `.bee/sessions/` also holds `<id>.activity.jsonl` sidecars; only
            // the plain `<id>.json` record carries the lane binding.
            let Some(id) = name.strip_suffix(".json") else { continue };
            if id.is_empty() || id.contains(".activity") {
                continue;
            }
            let ReadJson::Parsed(record) = read_json(&entry.path()) else { continue };
            if record.get("lane").and_then(Value::as_str) == Some(feature) {
                ids.push(id.to_string());
            }
        }
    }
    if let Some(calling) = calling_session.map(str::trim).filter(|s| !s.is_empty()) {
        ids.push(calling.to_string());
    }
    ids.sort();
    ids.dedup();
    ids
}

/// Roll every candidate session's transcript up into one usage section.
///
/// `calling_session` is a PARAMETER rather than an env read inside this
/// function so the tests can drive it directly: `BEE_SESSION_ID` is
/// process-global and two tests setting it would race under the default
/// parallel test runner. The env resolution happens once, at the call site.
pub(crate) fn collect_close_usage(
    control: &Path,
    feature: &str,
    calling_session: Option<&str>,
) -> CloseUsage {
    let mut usage = CloseUsage {
        details: Vec::new(),
        skipped: 0,
        main: UsageBucket::default(),
        subagents: UsageBucket::default(),
    };
    for id in usage_session_ids(control, feature, calling_session) {
        let record = match read_json(&crate::verbs::cells::sessions_dir(control).join(format!("{id}.json"))) {
            ReadJson::Parsed(r) => r,
            // No record, or a corrupt one: a session we meant to cover and
            // could not. Counted, never silently dropped.
            _ => {
                usage.skipped += 1;
                continue;
            }
        };
        let transcript = record
            .get("transcript_path")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .filter(|p| p.exists());
        let Some(transcript) = transcript else {
            usage.skipped += 1;
            continue;
        };
        // `rollup_transcript` answers None for an empty/unparseable transcript
        // — same "could not read it" outcome as a missing file.
        let Some(rollup) = crate::hooks::session_close::rollup_transcript(&transcript) else {
            usage.skipped += 1;
            continue;
        };
        // The session's own two buckets, summed once and used twice: added
        // into the feature totals, and kept whole on its own record line.
        let mut session_main = UsageBucket::default();
        session_main.add_models(&rollup.models);
        let mut session_subagents = UsageBucket::default();
        session_subagents.add_models(&rollup.subagent_models);
        usage.main = usage.main.plus(&session_main);
        usage.subagents = usage.subagents.plus(&session_subagents);
        usage.details.push(SessionUsage {
            session_id: id,
            models: rollup.models,
            subagent_models: rollup.subagent_models,
            subagent_count: rollup.subagent_count,
            started_ms: rollup.started_ms,
            ended_ms: rollup.ended_ms,
            totals: session_main.plus(&session_subagents),
        });
    }
    usage
}

/// The record's own version marker. A reader that walks
/// `.bee/usage/*.json` checks this before trusting a field name, so
/// the string is a CONTRACT — a shape change earns `bee-usage/v2`, never a
/// quiet edit of what v1 means.
pub(crate) const USAGE_SCHEMA: &str = "bee-usage/v1";

/// The detailed token record a green close leaves in bee's own store
/// (decision e97cc9d4, relocated by 62331863).
///
/// `closed_at` is a parameter rather than a `utc_now()` read inside, so a test
/// can pin the one field that is not derived from the usage itself.
pub(crate) fn usage_record_value(feature: &str, usage: &CloseUsage, closed_at: &str) -> Value {
    json!({
        "schema": USAGE_SCHEMA,
        "feature": feature,
        "closed_at": closed_at,
        "sessions": usage.details.iter().map(SessionUsage::value).collect::<Vec<Value>>(),
        "skipped": usage.skipped,
        "totals": {
            "main": usage.main.value(),
            "subagents": usage.subagents.value(),
            "total": usage.total().value(),
        },
    })
}

/// Write `.bee/usage/<feature>.json`, answering the written path (relative to
/// the `control` root it was given).
///
/// ALWAYS, on a green close — including the close whose transcripts were all
/// unreadable, which writes `sessions: []` beside a non-zero `skipped`. A
/// missing file and an empty record must never read alike: the file says "we
/// looked and found nothing readable", the absence says nothing at all. That
/// is the same honesty rule [`close_usage_line`] follows by staying SILENT for
/// that case — a printed "0 tokens" would be a false claim, while a stored
/// empty `sessions` list beside `skipped` is a true one.
///
/// THE CONTROL ROOT, like the sessions this record sums: the caller resolves
/// it once through `control_root_for` and hands it to both
/// [`collect_close_usage`] and this write, so a worktree close stores the
/// record in the main checkout's `.bee` — the same place its `.bee/sessions/`
/// evidence lives — instead of in a worktree that is about to disappear.
///
/// `write_json_atomic` is [`write_text_atomic`]'s JSON sibling — the same
/// tmp-then-rename write `promote-proposals.md` gets, plus the repo's one
/// pretty-printer, so this record is spelled like every other JSON file bee
/// writes; it creates `.bee/usage/` on the way. Unlike that proposal file this
/// record lands INSIDE `commit_close_bookkeeping`'s `.bee`-scoped stage
/// (`git add -A -- .bee`), and `.bee/usage/` is not matched by `.gitignore`
/// (only `.bee/logs/`, `.bee/sessions/`, `.bee/cache/` and friends are), so
/// close's own bookkeeping commit puts the record in git — no merge-time
/// dependency, nothing left dirty on disk.
pub(crate) fn write_usage_record(
    control: &Path,
    feature: &str,
    usage: &CloseUsage,
) -> Result<String, String> {
    let rel = format!(".bee/usage/{feature}.json");
    let value = usage_record_value(feature, usage, &crate::verbs::cells::utc_now());
    match write_json_atomic(&control.join(&rel), &value) {
        Ok(()) => Ok(rel),
        Err(e) => Err(e.to_string()),
    }
}

/// The one summary line. `None` when no transcript was readable at all: a
/// "usage: 0 session(s) — 0 tokens" line states a cost of zero, which is a
/// FALSE claim about a feature whose transcripts merely could not be found.
/// Saying nothing is the honest answer there.
///
/// Reuses `fmt_tokens` (`hooks/session_close/perf`'s sibling `html.rs`), the
/// k/M/B spelling every other bee token readout already uses.
pub(crate) fn close_usage_line(usage: &CloseUsage) -> Option<String> {
    if usage.sessions() == 0 {
        return None;
    }
    let f = crate::hooks::session_close::fmt_tokens;
    let total = usage.total();
    let skipped = if usage.skipped > 0 {
        format!(", {} session(s) skipped — no readable transcript", usage.skipped)
    } else {
        String::new()
    };
    Some(format!(
        "usage: {} session(s) — {} tokens (main {}, subagents {}; new {}, cached {}){}",
        usage.sessions(),
        f(total.total),
        f(usage.main.total),
        f(usage.subagents.total),
        f(total.new_t),
        f(total.cached),
        skipped
    ))
}

/// provenance: bee.mjs handleClose (~7643). `worktree` is provably null here
/// (see the file header), so the merge-back line never renders natively.
pub(crate) fn close_handler(
    root: &Path,
    feature: &str,
    dry_run: bool,
    declared: Option<Vec<String>>,
    shell: Option<&'static str>,
    pattern_verdicts: &HashMap<String, String>,
) -> D<Out> {
    // P2-3: a non-boolean, non-null `close_commit_bookkeeping` refuses the
    // WHOLE close UP FRONT — before the dry-run door listing, before the
    // declared tests run, before anything is written. Precedent:
    // `worktree_cleanup_on_merge_config` (verbs/worktree/handlers.rs) reads
    // "present-but-non-boolean" as `None` and refuses rather than guesses —
    // but that verb still has a Node fallback for the bare `None` its own
    // `?` produces. Close has none (`rg -l buildContextManifest --glob
    // '*.mjs'` finds nothing left to delegate to), so the refusal here is a
    // typed `Out::Thrown` that names the key and the offending value
    // instead of a silent `None`. B-P2-4: the refusal names the file that
    // actually carries the offending value — `.bee/config.local.json` when
    // it sets the key, `.bee/config.json` otherwise — since the merged
    // config `close_commit_bookkeeping_invalid_value` reads can be an
    // untracked overlay override the tracked file never mentions.
    if let Some(bad) = close_commit_bookkeeping_invalid_value(root) {
        let offending_file = close_commit_bookkeeping_offending_file(root);
        return Ok(Out::Thrown(format!(
            "close: \"close_commit_bookkeeping\" in {offending_file} must be a boolean, got {bad} — fix the config, then re-run bee close --feature {feature}."
        )));
    }

    // D7 (docs/history/test-doctrine/CONTEXT.md): no boundary auto-run
    // remains — close never spawns `commands.test` itself, whether or not a
    // worktree is granted for the feature. The tests door instead reads
    // whether every capped cell already carries a recorded D8 proof line
    // (verbs/cells/proof.rs `feature_proof_check`) — `declared`/`shell`
    // stay as parameters only so this signature keeps matching every
    // existing caller (verbs/cells/tests.rs `close_refuses_judge_debt_for_
    // a_standard_lane_feature` calls this directly with both).
    let _ = &declared;
    let _ = shell;
    let proof = crate::verbs::cells::feature_proof_check(root, feature)?;

    if dry_run {
        let mut doors = vec![Door {
            door: "tests",
            blocking: proof.blocking,
            detail: proof_door_detail(&proof),
            command: if proof.blocking { Some("bee cells finish") } else { None },
        }];
        doors.extend(build_close_report_doors(root, feature)?);
        doors.push(build_pattern_check_door(root, feature, pattern_verdicts)?);
        doors.push(build_knowledge_freshness_door(root, feature)?);
        doors.push(build_impact_door(root, feature)?);
        doors.push(build_routing_door(root, feature)?);
        doors.push(build_doc_deferral_door(root, feature, true)?);
        record_merge_ready_blocked_by(root, feature, &doors);
        let next_line = if proof.blocking {
            format!(
                "next: re-cap the cell(s) above with a real proof line (\"<command> — <result> — <scope reason>\"), then re-run bee close --feature {feature}"
            )
        } else {
            format!("next: bee close --feature {feature} — checks every capped cell's proof line and reports")
        };
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        let mut lines = render_close_door_lines(&doors);
        lines.push(next_line);
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 0));
    }

    let report_doors = build_close_report_doors(root, feature)?;
    let pattern_door = build_pattern_check_door(root, feature, pattern_verdicts)?;
    let knowledge_freshness_door = build_knowledge_freshness_door(root, feature)?;
    let impact_door = build_impact_door(root, feature)?;
    let routing_door = build_routing_door(root, feature)?;
    // D2/D6: this is the one REAL (writing) call — the door seeds itself
    // here, before the proof-debt refusal just below and before every other
    // blocking door further down. NAMED DEVIATION (recorded on cell ddb-1):
    // a close that refuses at a later door therefore writes the seed file
    // and never reaches `commit_close_bookkeeping`, leaving it untracked
    // until the next GREEN close. Accepted rather than fixed: the file is
    // correct and fully enforcing from the moment it lands, so no verdict
    // is ever wrong; `commit_close_bookkeeping` stages with
    // `git add -A -- .bee`, which picks the untracked file up on that next
    // green close; and moving the write onto close's green path would take
    // the seed out of the door, which D2 locks as the door's own job (and
    // would put it beyond the reach of this door's unit tests).
    let doc_deferral_door = build_doc_deferral_door(root, feature, false)?;

    if proof.blocking {
        let mut doors = vec![Door {
            door: "tests",
            blocking: true,
            detail: proof_door_detail(&proof),
            command: Some("bee cells finish"),
        }];
        doors.extend(report_doors);
        doors.push(pattern_door);
        doors.push(knowledge_freshness_door);
        doors.push(impact_door);
        doors.push(routing_door);
        doors.push(doc_deferral_door);
        // NAMED DEVIATION (mrf-2): the cell named two full-doors vectors —
        // the dry-run one and the green-path one below. There are THREE:
        // this proof-debt refusal arm assembles its own complete vector and
        // returns before the green path is ever reached. Wiring it too is
        // what the cell's own rule ("before any early-return refusal arm, so
        // a blocked close still records why") actually asks for — a close
        // stopped at the tests door would otherwise record nothing.
        record_merge_ready_blocked_by(root, feature, &doors);
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(false));
        result.insert("tests".into(), Value::Null);
        let lines = vec![
            format!(
                "{CLOSE_PROOF_DEBT_PREFIX} \"{feature}\" — close stops at the tests door: {} capped cell(s) carry a report with no valid proof line ({}).",
                proof.bad_ids.len(),
                proof.bad_ids.join(", ")
            ),
            "remedy: re-cap each cell above with a real proof line: \"<command> — <result> — <scope reason>\" (bee cells finish --id <id> --report '{...}').".to_string(),
            format!("next: settle the proof debt above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // Proof clear (including the no-capped-cells and legacy-only cases):
    // what remains is the capture checklist.
    let tests_door = Door {
        door: "tests",
        blocking: false,
        detail: proof_door_detail(&proof),
        command: None,
    };
    let scribing_detail = report_doors
        .iter()
        .find(|d| d.door == "scribing-debt")
        .map(|d| d.detail.clone())
        .unwrap_or_default();
    let queue_detail = report_doors
        .iter()
        .find(|d| d.door == "capture-queue")
        .map(|d| d.detail.clone())
        .unwrap_or_default();
    let mut doors = vec![tests_door];
    doors.extend(report_doors);
    doors.push(pattern_door);
    doors.push(knowledge_freshness_door);
    doors.push(impact_door);
    doors.push(routing_door);
    doors.push(doc_deferral_door);
    record_merge_ready_blocked_by(root, feature, &doors);

    // ── D1: refuse on uncaptured behavior_change cells ──────────────────────
    //
    // Tests are GREEN (or undeclared) — the one remaining door that can still
    // stop close is scribing-debt, and only when it is BLOCKING: the feature
    // has behavior_change cells with no capture recorded and no logged
    // `capture-deferral` decision names it (build_close_report_doors is the
    // one place that decides `blocking` — this reads its verdict rather than
    // recomputing it, so the counter and the refusal can never disagree).
    if doors.iter().any(|d| d.door == "scribing-debt" && d.blocking) {
        let debt = scribing_debt(root, feature)?;
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(false));
        result.insert("tests".into(), Value::Null);
        let lines = vec![
            format!(
                "{CLOSE_CAPTURE_DEBT_PREFIX} \"{feature}\" — close stops at the scribing-debt door: {} behavior_change cell(s) uncaptured ({}).",
                debt.count,
                js_join(&debt.ids, ", ")
            ),
            format!("remedy: run bee-capturing to record the capture, or log a decision tagged capture-deferral naming \"{feature}\" to defer it."),
            format!("next: settle the capture debt above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // ── wl-3: refuse on judge debt for a standard/high-risk closing route ──
    //
    // Runs only here — tests GREEN (or undeclared) and past the D1 capture-
    // debt refusal above. The judge-debt door only EXISTS for a standard or
    // high-risk closing route (build_close_report_doors omits it entirely
    // below `standard`), so a tiny/small feature can never reach this
    // refusal — matching AGENTS.md's own judge-on-smell carve-out for those
    // lanes. Same "reads the door's own verdict, never recomputes it"
    // discipline the scribing-debt refusal above already established.
    if doors.iter().any(|d| d.door == "judge-debt" && d.blocking) {
        let debt = judge_debt(root, feature)?;
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(false));
        result.insert("tests".into(), Value::Null);
        // hpf-1: `cells judge-record` refuses an archived cell outright, so
        // an offending id that only resolves under the archive owes the
        // unarchive step FIRST, named ahead of the judge commands.
        let archived_ids = debt.ids.iter().any(|id| {
            id.as_str().is_some_and(|id| {
                !crate::verbs::cells::cell_file(root, id).exists()
                    && crate::verbs::cells::resolve_cell_file(root, id).is_some()
            })
        });
        let remedy = if archived_ids {
            format!("remedy: some of the cells above are archived — run bee cells unarchive --feature {feature} first, then bee cells judge to check, then bee cells judge-record to record a verdict for each cell.")
        } else {
            "remedy: run bee cells judge to check, then bee cells judge-record to record a verdict for each cell above.".to_string()
        };
        let lines = vec![
            format!(
                "{CLOSE_JUDGE_DEBT_PREFIX} \"{feature}\" — close stops at the judge-debt door: {} behavior_change cell(s) capped with no judge record ({}).",
                debt.count,
                js_join(&debt.ids, ", ")
            ),
            remedy,
            format!("next: settle the judge debt above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // slp-dissent-stop-and-ask sd-4: refuse on dissent debt, in EVERY lane.
    //
    // The `Door` the builder pushed does not refuse by itself, so this is the
    // second, separate half: same "read the door's own verdict, never
    // recompute it" discipline the two refusals above already keep. Unlike
    // the judge refusal, a `tiny` or `small` lane reaches this one, because
    // a2affcba puts the obligation everywhere a worker can dissent.

    if doors.iter().any(|d| d.door == "dissent-debt" && d.blocking) {
        let debt = crate::verbs::cells::feature_dissent_debt(root, feature)?;
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(false));
        result.insert("tests".into(), Value::Null);
        let archived = debt.ids.iter().any(|id| {
            id.as_str().is_some_and(|id| {
                !crate::verbs::cells::cell_file(root, id).exists()
                    && crate::verbs::cells::resolve_cell_file(root, id).is_some()
            })
        });
        let remedy = if archived {
            format!("remedy: some of the cells above are archived — run bee cells unarchive --feature {feature} first, then bee cells dissent-verdict to record a verdict for each dissent, or log a decision tagged dissent-deferral naming \"{feature}\" to defer it.")
        } else {
            format!("remedy: run bee cells dissent-verdict to record a verdict for each dissent on the cells above, or log a decision tagged dissent-deferral naming \"{feature}\" to defer it.")
        };
        let lines = vec![
            format!(
                "{CLOSE_DISSENT_DEBT_PREFIX} \"{feature}\" — close stops at the dissent-debt door: {} cell(s) carry a dissent with no verdict ({}).",
                debt.count,
                js_join(&debt.ids, ", ")
            ),
            remedy,
            format!("next: settle the dissent debt above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // slp-advisor-nudge an-3: refuse on advisor-nudge debt, in EVERY lane.
    //
    // Same second half the dissent refusal above is: the `Door` the builder
    // pushed does not refuse by itself, and this reads the door's own verdict
    // rather than recomputing it. Placed AFTER the dissent arm, so a feature
    // owing both is told about the dissent first — the same masking order the
    // judge arm already has over the dissent one, and deliberate.
    if doors.iter().any(|d| d.door == "advisor-nudge-debt" && d.blocking) {
        let debt = crate::verbs::supervisor::feature_advisor_nudge_debt(root, feature)?;
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(false));
        result.insert("tests".into(), Value::Null);
        let lines = vec![
            format!(
                "{CLOSE_ADVISOR_NUDGE_DEBT_PREFIX} \"{feature}\" — close stops at the advisor-nudge door: {} advisor nudge(s) with no consult and no recorded decline ({}).",
                debt.count,
                js_join(&debt.ids, ", ")
            ),
            "remedy: run the advisor consult for each row above, then record what came of it with bee decisions log --tags advisor-nudge — or record a reasoned decline the same way. The decision text must NAME the row id; one decision answers one row, and a decision naming no row clears nothing.".to_string(),
            format!("next: settle the advisor-nudge debt above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // ── uat-stop-placement D4.4/D2: refuse on a pending close-time uat door ─
    //
    // Runs only here — tests GREEN (or undeclared) and past the D1 capture-
    // debt and judge-debt refusals above — same "reads the door's own
    // verdict, never recomputes it" discipline those doors already
    // established. The door only exists under `uat_stop: "close"` for a
    // lane that cares (build_close_report_doors omits or clears it
    // otherwise), so under `"merge"`/`"off"`, an exempt lane, an approved
    // gate, or a logged `uat-deferral` decision, this refusal is
    // unreachable.
    if doors.iter().any(|d| d.door == "uat" && d.blocking) {
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(false));
        result.insert("tests".into(), Value::Null);
        let lines = vec![
            format!("{CLOSE_UAT_PREFIX} \"{feature}\" — close stops at the uat door: the uat gate is not yet approved."),
            "remedy: the product is on main now — reload it, test it, then bee gate --name uat --approved true, or fix in the worktree and merge again.".to_string(),
            format!("next: settle the uat gate above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // ── U7: refuse on a recorded `violated` pattern verdict ─────────────────
    //
    // Runs only here — tests GREEN (or undeclared) and past the D1 refusal
    // above — same "stops close exactly like a red test" placement the
    // scribing-debt door already established for its own blocking arm.
    if doors.iter().any(|d| d.door == "pattern-check" && d.blocking) {
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(false));
        result.insert("tests".into(), Value::Null);
        let pattern_detail = doors
            .iter()
            .find(|d| d.door == "pattern-check")
            .map(|d| d.detail.clone())
            .unwrap_or_default();
        let lines = vec![
            format!(
                "{CLOSE_PATTERN_VIOLATED_PREFIX} \"{feature}\" — close stops at the pattern-check door: {pattern_detail}"
            ),
            "remedy: fix the violated pattern's finding, or re-run with a corrected --pattern-verdicts if it is a false positive.".to_string(),
            format!("next: settle the violated pattern(s) above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // ── D1: refuse on a stale knowledge pointer in a touched area ──────────
    //
    // Runs only here — tests GREEN (or undeclared) and past the capture-debt,
    // judge-debt and pattern-check refusals above — same "stops close
    // exactly like a red test" placement those doors already established.
    if doors.iter().any(|d| d.door == "knowledge-freshness" && d.blocking) {
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(false));
        result.insert("tests".into(), Value::Null);
        let freshness_detail = doors
            .iter()
            .find(|d| d.door == "knowledge-freshness")
            .map(|d| d.detail.clone())
            .unwrap_or_default();
        let lines = vec![
            format!(
                "{CLOSE_KNOWLEDGE_FRESHNESS_PREFIX} \"{feature}\" — close stops at the knowledge-freshness door: {freshness_detail}"
            ),
            "remedy: fix each pointer above (bee knowledge check names the same findings), or log a decision tagged knowledge-freshness-deferral naming this feature with the reason.".to_string(),
            format!("next: settle the stale pointer(s) above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // ── doc-impact-synthesis D1b: refuse on a surviving citation of a
    // closing-feature decision ──────────────────────────────────────────────
    //
    // Runs only here — tests GREEN (or undeclared) and past the capture-debt,
    // judge-debt, pattern-check and knowledge-freshness refusals above — same
    // "stops close exactly like a red test" placement those doors already
    // established.
    if doors.iter().any(|d| d.door == "impact" && d.blocking) {
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(false));
        result.insert("tests".into(), Value::Null);
        let impact_detail = doors
            .iter()
            .find(|d| d.door == "impact")
            .map(|d| d.detail.clone())
            .unwrap_or_default();
        let lines = vec![
            format!("{CLOSE_IMPACT_PREFIX} \"{feature}\" — close stops at the impact door: {impact_detail}"),
            "remedy: fix or annotate each citing doc above, or log a decision tagged impact-deferral naming this feature with the reason.".to_string(),
            format!("next: settle the citation(s) above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // ── doc-impact-synthesis D2: refuse on an unrouted locked D-ID ──────────
    //
    // Runs only here — tests GREEN (or undeclared) and past the capture-debt,
    // judge-debt, pattern-check, knowledge-freshness and impact refusals
    // above — same "stops close exactly like a red test" placement those
    // doors already established. Never fires for a legacy-form CONTEXT
    // (`build_routing_door`'s own notice branch is never `blocking`).
    if doors.iter().any(|d| d.door == "routing" && d.blocking) {
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(false));
        result.insert("tests".into(), Value::Null);
        let routing_detail = doors
            .iter()
            .find(|d| d.door == "routing")
            .map(|d| d.detail.clone())
            .unwrap_or_default();
        let lines = vec![
            format!("{CLOSE_ROUTING_PREFIX} \"{feature}\" — close stops at the routing door: {routing_detail}"),
            "remedy: cite the unrouted D-ID(s) in a docs/knowledge/ bundle file, or log a decision tagged routing-deferral naming this feature with the reason.".to_string(),
            format!("next: settle the unrouted D-ID(s) above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    // ── doc-impact-synthesis D3: refuse on doc deferral prose with no
    // registered trigger ─────────────────────────────────────────────────────
    //
    // Runs only here — tests GREEN (or undeclared) and past every refusal
    // above, including routing.
    if doors.iter().any(|d| d.door == "doc-deferral" && d.blocking) {
        let mut result = Map::new();
        result.insert("feature".into(), Value::String(feature.to_string()));
        result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
        result.insert("ran_tests".into(), Value::Bool(false));
        result.insert("tests".into(), Value::Null);
        let doc_deferral_detail = doors
            .iter()
            .find(|d| d.door == "doc-deferral")
            .map(|d| d.detail.clone())
            .unwrap_or_default();
        let lines = vec![
            format!(
                "{CLOSE_DOC_DEFERRAL_PREFIX} \"{feature}\" — close stops at the doc-deferral door: {doc_deferral_detail}"
            ),
            "remedy: register the condition with `bee triggers add --decision <id> --condition \"...\"` and cite it inline, or log a decision tagged doc-deferral naming this feature with the reason.".to_string(),
            format!("next: settle the deferral line(s) above, then re-run bee close --feature {feature}"),
        ];
        return Ok(Out::Emit(Value::Object(result), lines.join("\n"), 1));
    }

    let headline = format!("Tests GREEN for \"{feature}\" — {}", proof_door_detail(&proof));
    let mut result = Map::new();
    result.insert("feature".into(), Value::String(feature.to_string()));
    result.insert("doors".into(), Value::Array(doors.iter().map(Door::value).collect()));
    result.insert("ran_tests".into(), Value::Bool(false));
    result.insert("tests".into(), Value::Null);

    // ── D2: soft promote door — computed BEFORE retirement ─────────────────
    //
    // Runs only here — past the tests door (GREEN or undeclared) and past
    // the scribing-debt refusal above, so a red close or one stopped on
    // capture debt never reaches it. It must run before the cells are
    // retired below: `build_promotion` scans `.bee/cells/*.json`, and once
    // retirement moves the feature's just-capped cells into
    // `.bee/cells/archive/` that scan would come back empty. `build_promotion`
    // is read-only, so computing it here has no effect beyond what it can
    // see. `build_promotion`'s `None` arm means "delegate to Node", and
    // there is no Node left to delegate to (`rg -l buildContextManifest
    // --glob '*.mjs'` finds nothing); a `Thrown` is promote's own typed
    // refusal (most commonly `unknown_work` for a feature with neither a
    // work-item concept nor a history anchor). Both degrade through the
    // SAME one-line warning pushed further below (line position unchanged)
    // and close proceeds unchanged either way — SOFT means proposing the
    // knowledge a feature earned never blocks finishing it, and D38
    // (promote proposes, it never writes into docs/knowledge/) stays
    // untouched by this door.
    let promote_outcome: Result<Value, String> = match crate::verbs::knowledge::bundle_dir(root) {
        None => Err("no docs/knowledge/ bundle to mine here".to_string()),
        Some(dir) => match crate::verbs::knowledge::build_promotion(root, &dir, feature) {
            None => Err("no docs/knowledge/ bundle to mine here".to_string()),
            Some(crate::verbs::knowledge::Promo::Thrown(msg)) => Err(msg),
            Some(crate::verbs::knowledge::Promo::Ok(proposal)) => Ok(proposal),
        },
    };

    // ── retire the feature's cells ────────────────────────────────────────
    //
    // Close is the lifecycle event that MEANS "this feature is done", and
    // `.bee/cells/` is on the hot read path — every `status` and `orient`
    // parses each file in it. Left to a human remembering `bee cells
    // archive`, cells accumulate for the life of the repo: bee's own store
    // reached 455 files across 118 features, 441 of them belonging to
    // features that were completely finished, and paid for all of them on
    // every orientation.
    //
    // Three conditions, all necessary: the close is GREEN and past the
    // scribing-debt door (a red close, or one refused on capture debt, never
    // reaches here), every one of the feature's cells is terminal (an open
    // cell is reported, not archived), and the repo has not opted out.
    // Reversible either way: `bee cells archive --feature <f>` has an
    // `unarchive` twin and the files stay in git.
    let retired = auto_archive_on_close(root, feature);

    let mut lines = vec![headline];
    lines.push(format!(
        "Capture (deferred, decision c8e25271): scribing {scribing_detail}; capture queue {queue_detail}."
    ));
    match &retired {
        // `moved == 0` is the feature that had no cells in the first place:
        // real, common (a docs-only close), and not worth a line.
        Retirement::Archived { moved } if *moved > 0 => lines.push(format!(
            "Retired \"{feature}\": {moved} cell(s) moved out of the active scan (bee cells unarchive --feature {feature} to reverse)."
        )),
        Retirement::Archived { .. } => {}
        Retirement::Held { reason } => lines.push(format!(
            "Cells kept in the active scan: {reason}."
        )),
        Retirement::Off => {}
    }
    result.insert("retired".into(), retired.value());

    // `promote_outcome` was computed above, before retirement moved the
    // feature's cells out of `.bee/cells/`, so `build_promotion` still saw
    // them. Rendering the warning line here keeps its position in the
    // output unchanged.
    let promote_line = match promote_outcome {
        Ok(proposal) => {
            let proposals_rel = format!("docs/history/{feature}/promote-proposals.md");
            let text = crate::verbs::knowledge::promote_text(&proposal);
            match write_text_atomic(&root.join(&proposals_rel), &text) {
                Ok(()) => {
                    enqueue_promote_stub(root, feature, &proposal, &proposals_rel);
                    enqueue_promote_deferred_record(root, feature, &proposals_rel);
                    let cells_mined = proposal["cells"].as_array().map(Vec::len).unwrap_or(0);
                    let area_bullets: usize = proposal["area_updates"]
                        .as_array()
                        .map(|updates| {
                            updates
                                .iter()
                                .map(|u| u["bullets"].as_array().map(Vec::len).unwrap_or(0))
                                .sum()
                        })
                        .unwrap_or(0);
                    let pattern_candidates =
                        proposal["pattern_candidates"].as_array().map(Vec::len).unwrap_or(0);
                    format!(
                        "Promote proposed for \"{feature}\": {cells_mined} capped cell(s) mined, {area_bullets} area bullet(s), {pattern_candidates} pattern candidate(s) — see {proposals_rel}."
                    )
                }
                Err(e) => format!(
                    "Promote proposed for \"{feature}\" but {proposals_rel} could not be written: {e}."
                ),
            }
        }
        Err(reason) => format!("Promote skipped for \"{feature}\": {reason}"),
    };
    lines.push(promote_line);

    // ── the token-usage section (decision 2d3abd12) ───────────────────────
    //
    // GREEN, non-dry-run only, like every line around it: `--dry-run` returns
    // near the top of this function and each blocking door returns above, so
    // a refused close never prints a cost for work that is not finished.
    //
    // THE CONTROL ROOT, not `root`. `.bee/sessions/` is control-plane — for a
    // linked worktree it lives in the main checkout, exactly as
    // `record_feature_close_in_mailbox` above resolves it. Reading `root`
    // here would find an empty sessions directory for every worktree feature
    // and report a cost of zero for the ones that ran in a worktree, which is
    // most of them.
    //
    // The calling session resolves through `resolve_session_flag_env` — the
    // same BEE_SESSION_ID → CLAUDE_CODE_SESSION_ID chain the mailbox stop
    // above already uses, so close names one session id, never two.
    let usage_control = crate::hooks::session_init::control_root_for(root);
    let usage = collect_close_usage(
        &usage_control,
        feature,
        crate::verbs::cells::resolve_session_flag_env(None).as_deref(),
    );
    result.insert("usage".into(), usage.value());
    let usage_line = close_usage_line(&usage);
    if let Some(line) = &usage_line {
        lines.push(line.clone());
    }

    // The detailed record beside the printed line (decision e97cc9d4), stored
    // in bee's own store at `.bee/usage/<feature>.json` (decision 62331863).
    // The CONTROL root again — the record belongs beside the `.bee/sessions/`
    // evidence it sums, not in a worktree. Written BEFORE the bookkeeping
    // commit below, which stages `.bee`, so close itself commits the record.
    // FAIL-SOFT like every other write on this tail: a record that could not be
    // written is one warning line, never a failed close.
    match write_usage_record(&usage_control, feature, &usage) {
        Ok(rel) => {
            result.insert("usage_record".into(), json!(rel));
        }
        Err(e) => lines.push(format!(
            "Warning: the token-usage record for \"{feature}\" could not be written: {e}"
        )),
    }

    // ── bookkeeping auto-commit — GREEN, non-dry-run only, path-scoped ─────
    //
    // Runs last, after every other `.bee` write above (retirement,
    // capture-queue stub, promote-proposal enqueue) so the commit captures
    // all of them. Warn-never-block: a git failure here is one line in the
    // text output and close's own exit code stays 0 — the close already
    // succeeded, and a store that could not be tidied is not a failed close.
    let bookkeeping = commit_close_bookkeeping(root, feature);
    result.insert("bookkeeping_commit".into(), bookkeeping.value());
    if let BookkeepingCommit::Skipped { reason, index_restored } = &bookkeeping {
        if let Some(detail) = reason.strip_prefix("git_failed:") {
            // P2-1: name what happened to the stage, not just the git
            // failure — a reader deciding whether to go look at `.bee`
            // themselves needs to know whether it is still sitting staged.
            let stage_note = match index_restored {
                Some(true) => " (index restored)",
                Some(false) => " (WARNING: .bee left staged)",
                None => "",
            };
            lines.push(format!(
                "Warning: bee-store bookkeeping commit failed for \"{feature}\": {detail}{stage_note}"
            ));
        }
    }

    // ── mcl-3 (merge-closes-the-lane R2): green, non-dry-run close sets
    // the feature's lane to the terminal phase ─────────────────────────────
    //
    // This whole GREEN path already sits past every BLOCKING door and past
    // the dry-run branch (which returns early, near the top of this
    // function) — so a blocked or `--dry-run` close never reaches this line
    // at all, by construction, and no separate guard for either is needed
    // here. Reuses `run_set_body` (state_group/set_gate.rs) rather than
    // hand-writing the lane record: it enforces the `--owner` precondition
    // and the phase enum for free. `--owner` is the record's OWN
    // pre-mutation phase, read from `read_lane_display` right here — never
    // guessed. NEVER writes "compounding-complete": that value is gated on
    // a fresh recorded compounding run (state_group/store.rs) and close has
    // no standing to waive it; "idle" is the only terminal value this write
    // ever names. A lane already at a terminal phase (`idle` or
    // `compounding-complete`) is left untouched — checked before the call,
    // not left to `run_set_body` to no-op, so a repeat close never rewrites
    // a record that already reads terminal. Best-effort like the
    // bookkeeping commit right above: no lane record for this feature is
    // silent (nothing to close), and any other read or write failure warns
    // on its own line and leaves the close GREEN.
    match crate::verbs::workflow_store::read_lane_display(root, feature) {
        Ok(Some(lane)) => {
            let current_phase = lane
                .get("phase")
                .and_then(|v| v.as_str())
                .unwrap_or("idle")
                .to_string();
            if current_phase != "idle" && current_phase != "compounding-complete" {
                let next_action = format!("bee close finished \"{feature}\" — feature is closed.");
                let set_flags = Flags(vec![
                    ("lane".to_string(), FlagV::S(feature.to_string())),
                    ("phase".to_string(), FlagV::S("idle".to_string())),
                    ("owner".to_string(), FlagV::S(current_phase)),
                    ("next-action".to_string(), FlagV::S(next_action)),
                ]);
                match crate::verbs::state_group::run_set_body(root, &set_flags) {
                    Ok(Out::Emit(..)) => {
                        lines.push(format!(
                            "Lane phase set to \"idle\" for \"{feature}\" — close is the terminal write."
                        ));
                    }
                    Ok(Out::Thrown(msg)) => {
                        lines.push(format!(
                            "Warning: could not set lane phase to \"idle\" for \"{feature}\": {msg}"
                        ));
                    }
                    Err(Err2::Msg(msg)) => {
                        lines.push(format!(
                            "Warning: could not set lane phase to \"idle\" for \"{feature}\": {msg}"
                        ));
                    }
                    Err(Err2::Ex) => {
                        lines.push(format!(
                            "Warning: could not set lane phase to \"idle\" for \"{feature}\": lane mutation lock or lane read failed."
                        ));
                    }
                }
            }
        }
        Ok(None) => {}
        Err(_) => {
            lines.push(format!(
                "Warning: could not read the lane record for \"{feature}\" to set its terminal phase."
            ));
        }
    }

    // ── the human mailbox: D4's feature-close stop (hm-10) ────────────────
    //
    // ON THE NON-DRY-RUN TAIL, and that placement is the whole point of the
    // fork `verbs/cells/handlers_close.rs` maps above `record_cap_in_mailbox`:
    // `--dry-run` lists the doors, writes nothing and STOPS nothing, so it
    // must append nothing. A letter reports what a run DID. The dry-run branch
    // returns near the top of this function and every blocking door above
    // returns before this line, so a refused close records no stop either.
    //
    // AFTER `commit_close_bookkeeping`, deliberately, and NOT with the other
    // `.bee` writes above it: the mailbox is a per-checkout RUNTIME record —
    // bee's own onboarding block puts `.bee/human-mailbox/` in `.gitignore` —
    // so it is not bookkeeping and must never be swept into the store commit.
    // The commit stages with `git add -A -- .bee`, which in a checkout whose
    // ignore block is absent would otherwise commit a run's letter material
    // into the project's history. The lane write just above is the existing
    // precedent for a `.bee` write that lands after that commit.
    record_feature_close_in_mailbox(root, feature, usage_line.as_deref());

    lines.push(
        "next: done — capture is recorded as pending (run bee-capturing whenever; orient keeps the reminder)."
            .to_string(),
    );
    Ok(Out::Emit(Value::Object(result), lines.join("\n"), 0))
}

// ── the human mailbox: D14's feature-close letter (hm-10) ─────────────────
//
// D7 promised that architecture, behaviour and usage "appear only in the
// feature-close letter". The letter itself, its sections and its dropping
// rule live in `verbs/mailbox.rs` under that module's own D14 section; what
// lives HERE is the one thing only close knows — the material.
//
// D8'S AUTHORSHIP BAN IS THE DESIGN CONSTRAINT. The composing pass may state
// no fact no stored entry carries, so every string this file hands the
// mailbox is read out of the feature's OWN capped cells — never written here,
// never summarised, never inferred. Three lists, three already-recorded
// facts:
//
//   * Architecture — the files the feature's capped work actually changed
//     (`feature_touched_files`, each cell's `trace.files_changed`). It is the
//     answer to "which parts of the system does this feature live in", and it
//     is a fact the cells recorded at their own caps.
//   * Behaviour — each capped cell's `acceptance`: the feature's own written
//     statement of what is true once that cell is done. Planned before the
//     work, checked at the cap, and about the SYSTEM rather than the process.
//   * Usage — the skills and specs the feature's cells declared they change
//     (`affects_skills`, `affects_specs`): the instructions that now describe
//     how the thing is used. Pointers a human can open, not prose about them.
//
// A feature that recorded none of a given fact gets NO section for it — the
// letter drops it (D7), and that silence is the correct outcome. Filling a
// heading with invented prose is the one thing D8 forbids outright.
//
// FAIL-OPEN, exactly like `record_cap_in_mailbox` (D10): the close has
// already landed by the time this runs, and no failure to record a mailbox
// entry may turn a landed close into a refusal. Every read below degrades to
// an empty list, and the append itself warns rather than throws.

/// The three lists D14's extra sections are composed from, read out of the
/// feature's own capped cells. Cells are read INCLUDING the archive, because
/// this runs after `auto_archive_on_close` has already moved them.
///
/// The FOURTH list is the token-usage line (decision e97cc9d4) — the same one
/// string close just printed, handed in rather than recomputed here so the
/// letter and the terminal can never state two different costs. `None` when no
/// transcript was readable: the letter then grows no Token usage section at
/// all, which is `close_usage_line`'s own honesty rule reaching the mailbox.
fn feature_close_note(
    root: &Path,
    feature: &str,
    usage_line: Option<&str>,
) -> crate::verbs::mailbox::CloseNote {
    fn push_unique(out: &mut Vec<String>, value: &str) {
        let value = value.trim();
        if !value.is_empty() && !out.iter().any(|seen| seen == value) {
            out.push(value.to_string());
        }
    }

    let architecture = feature_touched_files(root, feature).unwrap_or_default();
    let mut behaviour: Vec<String> = Vec::new();
    let mut usage: Vec<String> = Vec::new();
    if let Ok(cells) = list_cells_including_archive(root, feature, Some("capped")) {
        for cell in cells {
            if let Some(Value::String(acceptance)) = vget(&cell, "acceptance") {
                push_unique(&mut behaviour, acceptance);
            }
            for key in ["affects_skills", "affects_specs"] {
                if let Some(Value::Array(items)) = vget(&cell, key) {
                    for item in items {
                        if let Value::String(name) = item {
                            push_unique(&mut usage, name);
                        }
                    }
                }
            }
        }
    }
    let token_usage = usage_line
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .into_iter()
        .collect();
    crate::verbs::mailbox::CloseNote { architecture, behaviour, usage, token_usage }
}

/// Record this feature close as ONE human-mailbox entry, the moment the close
/// lands (D4, D8, D14).
///
/// WHICH RUN. The session id, through the SAME `resolve_session_flag_env`
/// chain the cap and the run end use (`verbs/work.rs`
/// `file_letter_at_run_end`). A second, nearly-identical resolution here
/// would file this stop under a run whose letter nobody composes.
///
/// WHICH ROOT. The control root — the main checkout for a linked worktree,
/// which is where `cells finish` already put the caps this letter is composed
/// from, and where the run end goes looking for them.
///
/// UNCONDITIONAL, by D9: every session appends its entries, attended or not.
/// Arming decides only whether a letter is composed at the run's END.
///
/// AND THE LETTER IS FILED HERE, by D2 (aedb5be9): every close files its close
/// letter at the moment of the close, attended sessions included. The entry
/// data was already appended on the line above; filing was the missing step,
/// and waiting for a run end meant an attended session's close letter was
/// never written at all. FAIL-OPEN (D10): a letter that cannot be filed is
/// said out loud and never turns a finished close into a refusal.
fn record_feature_close_in_mailbox(root: &Path, feature: &str, usage_line: Option<&str>) {
    use crate::verbs::mailbox;
    let control = crate::hooks::session_init::control_root_for(root);
    let run = mailbox::run_id(
        crate::verbs::cells::resolve_session_flag_env(None).as_deref(),
    );
    let entry = mailbox::Entry {
        at: crate::verbs::cells::utc_now(),
        kind: mailbox::KIND_FEATURE_CLOSE.to_string(),
        // D8: the plain-language sentence is written at the moment of the
        // event, never at composition.
        what: mailbox::close_sentence(feature),
        // The close itself changed no file — the feature's files are a fact
        // ABOUT the feature, and they ride the Architecture list below rather
        // than pretending this stop edited them.
        files: Vec::new(),
        commit: None,
        proof: None,
        // A close is not a cell, so it has no plan to depart from (D5).
        departure: None,
        // A green close left nothing outstanding; a close that needed the
        // human's call refused at a door and never reached this line (D13).
        needs_you: Vec::new(),
    };
    mailbox::record_close_stop(
        &control,
        &run,
        &entry,
        &feature_close_note(root, feature, usage_line),
    );
    // D2: the same CONTROL root the entry just landed under — the letter must
    // be composed where the entries and the caps are. A worktree root here
    // would file an orphan letter in a checkout the human never reads.
    if let mailbox::RunEnd::Failed(why) = mailbox::file_close_letter(&control, &run) {
        eprintln!(
            "bee: could not file the human-mailbox letter for the close of \"{feature}\" on run \"{run}\" ({why}) — the close itself is recorded; that run has no letter to read."
        );
    }
}

/// What close did with the feature's cells, and why.
pub(crate) enum Retirement {
    Archived { moved: usize },
    Held { reason: String },
    /// `cells_archive_on_close: false` — the repo asked close to leave the
    /// store alone. Silent, because a switch the owner set is not news.
    Off,
}

impl Retirement {
    fn value(&self) -> Value {
        match self {
            Retirement::Archived { moved } => json!({"archived": true, "moved": moved}),
            Retirement::Held { reason } => json!({"archived": false, "reason": reason}),
            Retirement::Off => json!({"archived": false, "reason": "cells_archive_on_close is off"}),
        }
    }
}

/// Default TRUE: the whole point is that it happens without anyone
/// remembering. `.bee/config.json` `cells_archive_on_close: false` opts out —
/// for a repo whose own tooling reads `.bee/cells/*.json` by path.
fn archive_on_close_enabled(root: &Path) -> bool {
    let config = read_config_raw(root);
    !matches!(config.get("cells_archive_on_close"), Some(Value::Bool(false)))
}

fn auto_archive_on_close(root: &Path, feature: &str) -> Retirement {
    if !archive_on_close_enabled(root) {
        return Retirement::Off;
    }
    // Best-effort throughout: close has already succeeded, and a store that
    // could not be tidied is not a failed close. Every arm says what it did.
    match crate::verbs::cells::archive_feature_for_close(root, feature) {
        Ok(moved) => Retirement::Archived { moved },
        Err(reason) => Retirement::Held { reason },
    }
}

// ── bee-store bookkeeping auto-commit ───────────────────────────────────────
//
// A minimal local git-exec helper for every call here EXCEPT the commit
// itself: `verbs::worktree`'s own `run_git`/`is_ordinary_checkout` are a
// sibling module's internals, so this mirrors the shape (status/stdout/
// stderr) rather than reaching across for `rev-parse`, `status`, `add`, and
// `reset`. B-P2-1 is the one deliberate exception: the `git commit` call
// itself now goes through `verbs::worktree::commit_unsigned`, the ONE
// shared helper it and the worktree-merge commit (`phases.rs`) both call,
// so the unsigned-commit mechanism can never drift between the two —
// `commit_close_bookkeeping` converts its `GitOut` result back into a
// `GitRun` below so `git_fail_first_line` (and this module's own wording)
// stay untouched.

/// A completed `git` invocation: `None` means the spawn itself failed (git
/// off PATH), same "every field absent" shape `verbs::worktree::git::GitOut`
/// gives a `spawnSync` that never launched.
struct GitRun {
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

/// P3-1: stdin is ALWAYS `Stdio::null()` — a repo with `commit.gpgsign
/// true` and a tty pinentry configured must never be able to block on a
/// prompt that has nowhere to read from. Every close-bookkeeping git
/// invocation runs through this one helper, so the guarantee holds for all
/// of them, not just `commit`.
fn run_git(root: &Path, args: &[&str]) -> Option<GitRun> {
    match Command::new("git").args(args).current_dir(root).stdin(Stdio::null()).output() {
        Ok(out) => Some(GitRun {
            status: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }),
        Err(_) => None,
    }
}

/// The first line of whichever stream carries the message — stderr wins,
/// stdout is the fallback — trimmed. Feeds the `git_failed:<first line>`
/// reason so a multi-line git error never blows up the one-line warning.
///
/// P2-2: a silent failure (a pre-commit hook that exits non-zero without a
/// word on either stream, for instance) must never render as the bare
/// `git_failed:` prefix with nothing after it — that reads as a truncation
/// bug, not a cause. When both streams are empty or whitespace-only, this
/// falls back to the real exit status (`exit status <code>`), or
/// `killed by signal` for the signal-death case a spawned `git` can still
/// hit (`ExitStatus::code()` returns `None` there on Unix).
fn git_fail_first_line(out: &GitRun) -> String {
    let src = if !out.stderr.trim().is_empty() { out.stderr.as_str() } else { out.stdout.as_str() };
    let first_line = src.trim().lines().next().unwrap_or("").trim();
    if !first_line.is_empty() {
        return first_line.to_string();
    }
    match out.status {
        Some(code) => format!("exit status {code}"),
        None => "killed by signal".to_string(),
    }
}

/// What close's bee-store bookkeeping commit did, and why not when it
/// didn't. `reason` is one of: `clean`, `config_off`, `not_a_repo`, or
/// `git_failed:<first line>`. `index_restored` is only ever `Some` on the
/// one `git_failed` reason that can leave `.bee` staged after `git add`
/// already ran — `git commit` itself failing (P2-1) — every other `Skipped`
/// arm never staged anything, so there is nothing to report restoring.
pub(crate) enum BookkeepingCommit {
    Committed { sha: String },
    Skipped { reason: String, index_restored: Option<bool> },
}

impl BookkeepingCommit {
    fn skipped(reason: impl Into<String>) -> Self {
        BookkeepingCommit::Skipped { reason: reason.into(), index_restored: None }
    }

    fn value(&self) -> Value {
        match self {
            BookkeepingCommit::Committed { sha } => json!({"committed": true, "sha": sha}),
            BookkeepingCommit::Skipped { reason, index_restored: None } => {
                json!({"committed": false, "reason": reason})
            }
            BookkeepingCommit::Skipped { reason, index_restored: Some(restored) } => {
                json!({"committed": false, "reason": reason, "index_restored": restored})
            }
        }
    }
}

/// `close_commit_bookkeeping` in the merged config (`.bee/config.json`
/// overlaid by `.bee/config.local.json`, state.rs:157-182): absent OR
/// `null` reads as ON (mirrors `archive_on_close_enabled`'s absent-means-on
/// default just above) — B-P2-4: `null` is bee's own unset idiom, so a
/// value round-tripped through a tool that only knows JSON's `null` (never
/// "delete the key") must default exactly like an absent key, not refuse.
/// Unlike that helper, a present, non-null, non-boolean value is REFUSED
/// (`None`) rather than silently read as ON — precedent:
/// `worktree_cleanup_on_merge_config` (verbs/worktree/handlers.rs) — a
/// typo'd config value must never resolve to a commit running unasked.
fn close_commit_bookkeeping_config(root: &Path) -> Option<bool> {
    match read_config_raw(root).get("close_commit_bookkeeping") {
        None | Some(Value::Null) => Some(true),
        Some(Value::Bool(b)) => Some(*b),
        Some(_) => None,
    }
}

/// P2-3: the raw offending value, present ONLY when
/// `close_commit_bookkeeping_config` would refuse (present, non-null,
/// non-boolean) — `close_handler` reads this BEFORE anything else runs so
/// the refusal can name the key and the value rather than folding into the
/// bookkeeping commit's own silent `config_off` skip. B-P2-4: `null` is
/// never offending — it reads as unset, same as an absent key.
fn close_commit_bookkeeping_invalid_value(root: &Path) -> Option<Value> {
    match read_config_raw(root).get("close_commit_bookkeeping") {
        None | Some(Value::Bool(_)) | Some(Value::Null) => None,
        Some(other) => Some(other.clone()),
    }
}

/// B-P2-4: which config file actually carries the offending
/// `close_commit_bookkeeping` value, for the refusal `close_handler` renders
/// when [`close_commit_bookkeeping_invalid_value`] finds one. `read_config_raw`
/// (state.rs:157-182) merges `.bee/config.local.json` OVER `.bee/config.json`,
/// and for any non-object value the overlay's own key — when the raw overlay
/// sets the key at all — wins the merge outright (state.rs `merge_config_overlay`),
/// so the local overlay is checked first; the tracked file is the answer for
/// every other case, including the (unreachable in practice) case where
/// neither raw file carries the key.
fn close_commit_bookkeeping_offending_file(root: &Path) -> &'static str {
    let has_key = |file: PathBuf| -> bool {
        matches!(
            read_json(&file),
            ReadJson::Parsed(Value::Object(m)) if m.contains_key("close_commit_bookkeeping")
        )
    };
    if has_key(root.join(".bee").join("config.local.json")) {
        ".bee/config.local.json"
    } else {
        ".bee/config.json"
    }
}

/// Auto-commits the `.bee` bookkeeping a GREEN, non-dry-run close just wrote
/// (retirement, the promote-proposal capture-queue stub) — path-scoped to
/// `.bee` throughout, so unrelated dirt and unrelated staged files are never
/// swept. Warn-never-block: every git step here is best-effort, same
/// contract `auto_archive_on_close` already keeps for retirement — close has
/// already succeeded, and a store that could not be tidied is not a failed
/// close.
fn commit_close_bookkeeping(root: &Path, feature: &str) -> BookkeepingCommit {
    let enabled = match close_commit_bookkeeping_config(root) {
        Some(b) => b,
        // Non-boolean: refused outright, never silently read as on. In
        // practice `close_handler` already refuses the whole close before
        // this function is ever reached (P2-3) — this arm is the
        // defense-in-depth fallback for any other caller of this function.
        None => return BookkeepingCommit::skipped("config_off"),
    };
    if !enabled {
        return BookkeepingCommit::skipped("config_off");
    }

    match run_git(root, &["rev-parse", "--is-inside-work-tree"]) {
        Some(out) if out.status == Some(0) && out.stdout.trim() == "true" => {}
        Some(_) => return BookkeepingCommit::skipped("not_a_repo"),
        None => return BookkeepingCommit::skipped("git_failed:git rev-parse could not be spawned"),
    }

    let status = match run_git(root, &["status", "--porcelain", "--", ".bee"]) {
        Some(out) if out.status == Some(0) => out,
        Some(out) => return BookkeepingCommit::skipped(format!("git_failed:{}", git_fail_first_line(&out))),
        None => return BookkeepingCommit::skipped("git_failed:git status could not be spawned"),
    };
    if status.stdout.trim().is_empty() {
        return BookkeepingCommit::skipped("clean");
    }

    match run_git(root, &["add", "-A", "--", ".bee"]) {
        Some(out) if out.status == Some(0) => {}
        Some(out) => return BookkeepingCommit::skipped(format!("git_failed:{}", git_fail_first_line(&out))),
        None => return BookkeepingCommit::skipped("git_failed:git add could not be spawned"),
    }

    // P3-1 (locked policy): bee's own bookkeeping commit is unsigned —
    // `--no-gpg-sign` means a repo with `commit.gpgsign true` can never turn
    // this into a hung `gpg`/pinentry prompt during close. B-P2-1: the
    // spawn itself now goes through `verbs::worktree`'s shared
    // `commit_unsigned` helper (also used by the worktree-merge commit in
    // `phases.rs`), but the failure text below is still assembled entirely
    // from the returned [`crate::verbs::worktree::GitOut`] fields — this
    // function's own wording (including `git_fail_first_line`'s `exit
    // status <code>` fallback, R81) never changes.
    let message = format!("Record {feature} close bookkeeping in the bee store");
    let commit_out = crate::verbs::worktree::commit_unsigned(root, &message, &[".bee"]);
    if commit_out.stdout.is_none() {
        // `commit_unsigned`'s spawn itself failed (git off PATH) — the
        // `GitOut` "every field null" shape mirrors this module's own
        // `run_git` returning `None` for the same condition.
        let index_restored =
            matches!(run_git(root, &["reset", "--", ".bee"]), Some(r) if r.status == Some(0));
        return BookkeepingCommit::Skipped {
            reason: "git_failed:git commit could not be spawned".to_string(),
            index_restored: Some(index_restored),
        };
    }
    if commit_out.status != Some(0) {
        // P2-1: `.bee` is staged (the `git add` above succeeded) and the
        // commit that would have consumed that stage never landed — a
        // git-degraded close must not leave `.bee` sitting staged on top of
        // whatever the feature's next commit does. Best-effort, same
        // warn-never-block contract as every other git step here: a reset
        // that itself fails is one more line in the warning, never a second
        // failure this function raises.
        let out = GitRun {
            status: commit_out.status,
            stdout: commit_out.stdout.unwrap_or_default(),
            stderr: commit_out.stderr.unwrap_or_default(),
        };
        let index_restored =
            matches!(run_git(root, &["reset", "--", ".bee"]), Some(r) if r.status == Some(0));
        return BookkeepingCommit::Skipped {
            reason: format!("git_failed:{}", git_fail_first_line(&out)),
            index_restored: Some(index_restored),
        };
    }

    let sha = run_git(root, &["rev-parse", "HEAD"])
        .filter(|out| out.status == Some(0))
        .map(|out| out.stdout.trim().to_string())
        .unwrap_or_default();
    BookkeepingCommit::Committed { sha }
}

pub(crate) fn run_close(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["feature", "dry-run", "pattern-verdicts"]) {
        return None;
    }
    // validate(): a boolean-typed flag given as =value must be true/false.
    match flags.get("dry-run") {
        None | Some(FlagV::Present) => {}
        Some(FlagV::S(s)) if s == "true" || s == "false" => {}
        Some(FlagV::S(_)) => return None,
    }
    // validate(): --feature required; requireFlag also rejects ''/true.
    let feature = flags.req_str("feature")?.to_string();
    // `flags['dry-run'] === true`: only the flag-alone form is JS `true`.
    let dry_run = matches!(flags.get("dry-run"), Some(FlagV::Present));
    // U7: `--pattern-verdicts=<id>:<verdict>[,...]` — absent or a bare flag
    // (no value) both read as "no verdicts supplied," same as an empty one.
    let pattern_verdicts: HashMap<String, String> =
        flags.truthy_str("pattern-verdicts").map(parse_pattern_verdicts).unwrap_or_default();

    // ── everything that can still delegate happens BEFORE prelude, whose
    //    drift-cache write would swallow the Node re-run's drift line. ──────
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, "close", use_json, t0, &why))
        }
        Roots::None => return Some(emit_no_root_error(&cwd, "close", use_json, t0)),
    };
    // D7: close never spawns commands.test, so no shell needs resolving and
    // no `.bee/logs/` dir needs creating up front any more — `declared` is
    // still read (and still passed through below) only to keep
    // `close_handler`'s signature matching its other callers.
    let declared = declared_test_commands(&root).ok()?;
    // Delegation pre-flight for the report doors: they are pure reads, so
    // computing them here (and again, for real, in close_handler) can only
    // cost two cheap directory scans — but it means a corrupt store can
    // still hand the whole command to Node BEFORE close_handler runs.
    build_close_report_doors(&root, &feature).ok()?;
    build_pattern_check_door(&root, &feature, &pattern_verdicts).ok()?;

    let ctx = match prelude("close", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out: R2<Out> = close_handler(&ctx.root, &feature, dry_run, declared, None, &pattern_verdicts)
        .map_err(crate::verbs::reservations::Err2::from);
    finish(&ctx, out)
}

// ═══ routing ═══════════════════════════════════════════════════════════════

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    match args.first()?.to_str()? {
        "close" => {
            let toks: Vec<&str> =
                args[1..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
            if toks.iter().any(|t| *t == "--help") {
                return None; // Node renders command-scoped help
            }
            let (flags, use_json) = parse_flags(&toks)?;
            run_close(flags, use_json, t0)
        }
        "dispatch" => {
            let sub = args.get(1)?.to_str()?;
            let toks: Vec<&str> =
                args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
            if toks.iter().any(|t| *t == "--help") {
                return None;
            }
            let (flags, use_json) = parse_flags(&toks)?;
            match sub {
                "prepare" => run_dispatch_prepare(flags, use_json, t0),
                "wave" => run_dispatch_wave(flags, use_json, t0),
                _ => None,
            }
        }
        _ => None,
    }
}

// ─── tests: U3 capture-queue pressure escalation (close door) ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn w(root: &Path, rel: &str, body: &str) {
        let file = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, body).unwrap();
    }

    fn stub_line(id: &str, at: &str) -> String {
        format!(r#"{{"kind":"stub","id":"{id}","at":"{at}","outcome":"x"}}"#)
    }

    /// Below the default threshold (5 stubs, 7 days): the door's wording
    /// stays byte-identical to before U3, same as the nudge's contract.
    #[test]
    fn under_threshold_detail_is_byte_identical_to_before() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let now = crate::verbs::reservations::now_iso();
        w(root, ".bee/capture-queue.jsonl", &format!("{}\n", stub_line("s1", &now)));
        let (queue, oldest_ms) = capture_queue_pending(root);
        assert_eq!(
            capture_queue_door_detail(root, queue, oldest_ms),
            "pending — 1 capture stub(s) awaiting flush; settle later via bee-capturing"
        );
    }

    #[test]
    fn zero_pending_reads_clear() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert_eq!(capture_queue_door_detail(root, 0, f64::NAN), "clear");
    }

    #[test]
    fn over_count_threshold_escalates_the_door_to_overdue_wording() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let now = crate::verbs::reservations::now_iso();
        let lines: String =
            (0..6).map(|i| format!("{}\n", stub_line(&format!("s{i}"), &now))).collect();
        w(root, ".bee/capture-queue.jsonl", &lines);
        let (queue, oldest_ms) = capture_queue_pending(root);
        assert_eq!(queue, 6);
        let detail = capture_queue_door_detail(root, queue, oldest_ms);
        assert!(
            detail.starts_with("OVERDUE — 6 stub(s) pending, oldest 0 days — flush before new work"),
            "{detail}"
        );
        assert!(detail.ends_with("settle via bee-capturing"));
    }

    #[test]
    fn over_age_threshold_escalates_even_under_count() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let old = crate::verbs::reservations::iso_from_ms(now_ms() - 10.0 * 86_400_000.0).ok().unwrap();
        w(root, ".bee/capture-queue.jsonl", &format!("{}\n", stub_line("s1", &old)));
        let (queue, oldest_ms) = capture_queue_pending(root);
        assert_eq!(queue, 1);
        let detail = capture_queue_door_detail(root, queue, oldest_ms);
        assert!(
            detail.starts_with("OVERDUE — 1 stub(s) pending, oldest 10 days — flush before new work"),
            "{detail}"
        );
    }

    #[test]
    fn configured_threshold_overrides_the_default_for_the_door() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/config.json", r#"{"capture_queue_threshold":{"count":1,"days":30}}"#);
        let now = crate::verbs::reservations::now_iso();
        let lines = format!("{}\n{}\n", stub_line("s1", &now), stub_line("s2", &now));
        w(root, ".bee/capture-queue.jsonl", &lines);
        let (queue, oldest_ms) = capture_queue_pending(root);
        let detail = capture_queue_door_detail(root, queue, oldest_ms);
        assert!(detail.starts_with("OVERDUE — 2 stub(s) pending"), "{detail}");
    }

    /// A malformed threshold falls back to the default (5, 7) — the door
    /// never blocks either way (`build_close_report_doors`' capture-queue
    /// row always carries `blocking: false`).
    #[test]
    fn malformed_threshold_falls_back_and_the_door_never_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/config.json", r#"{"capture_queue_threshold":{"count":-1,"days":7}}"#);
        w(root, ".bee/capture-queue.jsonl", &format!("{}\n", stub_line("s1", &crate::verbs::reservations::now_iso())));
        let doors = build_close_report_doors(root, "demo").unwrap();
        let capture_door = doors.iter().find(|d| d.door == "capture-queue").unwrap();
        assert!(!capture_door.blocking);
        assert_eq!(capture_door.detail, "pending — 1 capture stub(s) awaiting flush; settle later via bee-capturing");
    }

    // ─── tests: U7 close-time pattern-check door ───────────────────────────

    /// Writes a minimal bundle: one area concept whose `bee.sources` names
    /// `src/a.rs` (so the touched file matches it) tagged `areas: [demo]`,
    /// and one `bee.critical: true` pattern also tagged `areas: [demo]`.
    fn write_pattern_bundle(root: &Path) {
        w(
            root,
            "docs/knowledge/areas/demo/overview.md",
            "---\ntype: bee.area\ntitle: Demo area\ndescription: d\nbee:\n  id: demo-area\n  lifecycle: active\n  areas: [demo]\n  sources: [src/a.rs]\n---\nbody\n",
        );
        w(
            root,
            "docs/knowledge/patterns/p1.md",
            "---\ntype: bee.pattern\ntitle: Demo critical pattern\ndescription: d\nbee:\n  id: pattern-p1\n  lifecycle: active\n  areas: [demo]\n  critical: true\n---\nbody\n",
        );
    }

    fn write_capped_cell_touching(root: &Path, feature: &str, file: &str) {
        w(
            root,
            &format!(".bee/cells/{feature}-1.json"),
            &format!(
                r#"{{"id":"{feature}-1","feature":"{feature}","status":"capped","trace":{{"behavior_change":true,"outcome":"did the thing","files_changed":["{file}"],"capped_at":"2026-08-10T00:00:00.000Z"}}}}"#
            ),
        );
    }

    #[test]
    fn pattern_check_door_is_clear_with_no_bundle() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_capped_cell_touching(root, "demo", "src/a.rs");
        let door = build_pattern_check_door(root, "demo", &HashMap::new()).unwrap();
        assert!(!door.blocking);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn pattern_check_door_reports_pending_when_no_verdicts_supplied() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_pattern_bundle(root);
        write_capped_cell_touching(root, "demo", "src/a.rs");
        let door = build_pattern_check_door(root, "demo", &HashMap::new()).unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert!(door.detail.contains("pattern-p1=pending"), "{}", door.detail);
        assert!(door.detail.contains("--pattern-verdicts="), "{}", door.detail);
    }

    #[test]
    fn pattern_check_door_blocks_on_a_violated_verdict_naming_the_pattern() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_pattern_bundle(root);
        write_capped_cell_touching(root, "demo", "src/a.rs");
        let mut verdicts = HashMap::new();
        verdicts.insert("pattern-p1".to_string(), "violated".to_string());
        let door = build_pattern_check_door(root, "demo", &verdicts).unwrap();
        assert!(door.blocking);
        assert!(door.detail.contains("pattern-p1=violated"), "{}", door.detail);
    }

    #[test]
    fn pattern_check_door_passes_on_respected_or_not_applicable() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_pattern_bundle(root);
        write_capped_cell_touching(root, "demo", "src/a.rs");
        for verdict in ["respected", "not-applicable"] {
            let mut verdicts = HashMap::new();
            verdicts.insert("pattern-p1".to_string(), verdict.to_string());
            let door = build_pattern_check_door(root, "demo", &verdicts).unwrap();
            assert!(!door.blocking, "{verdict}: {}", door.detail);
            assert!(door.detail.contains(&format!("pattern-p1={verdict}")), "{}", door.detail);
        }
    }

    #[test]
    fn pattern_check_door_clear_when_touched_files_miss_every_area() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_pattern_bundle(root);
        write_capped_cell_touching(root, "demo", "src/unrelated.rs");
        let door = build_pattern_check_door(root, "demo", &HashMap::new()).unwrap();
        assert!(!door.blocking);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn parse_pattern_verdicts_accepts_known_words_and_drops_the_rest() {
        let parsed = parse_pattern_verdicts("pattern-a:violated, pattern-b:Respected ,malformed,pattern-c:bogus");
        assert_eq!(parsed.get("pattern-a").map(String::as_str), Some("violated"));
        assert_eq!(parsed.get("pattern-b").map(String::as_str), Some("respected"));
        assert_eq!(parsed.get("malformed"), None);
        assert_eq!(parsed.get("pattern-c"), None);
        assert_eq!(parsed.len(), 2);
    }

    // ─── tests: bee-store bookkeeping auto-commit on a GREEN close ────────

    fn git_ok(dir: &Path, args: &[&str]) {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }

    fn git_out(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn git_status_porcelain(dir: &Path) -> String {
        git_out(dir, &["status", "--porcelain"])
    }

    fn git_committed_paths(dir: &Path) -> Vec<String> {
        git_out(dir, &["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"])
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// A real repo, seeded with a tracked `.bee/config.json` so later tests
    /// can dirty it and prove the commit is path-scoped — same
    /// `git_repo`/`git_commit` idiom `verbs::reviews` already uses for its
    /// own git-degradation coverage.
    fn init_bee_repo(root: &Path) {
        git_ok(root, &["init", "-q"]);
        git_ok(root, &["config", "user.email", "bee-close@example.com"]);
        git_ok(root, &["config", "user.name", "bee close tests"]);
        // B-P1-2: this fixture used to force `commit.gpgsign false`, which
        // masked the very condition `--no-gpg-sign` exists to defend
        // against — every test in this module would stay green even if
        // that flag were deleted. Left at the repo default (unset/off in a
        // test sandbox) here; `gpgsign_true_with_a_failing_signer_still_lands_the_bookkeeping_commit`
        // below is the one test that turns `commit.gpgsign` ON and pins
        // the flag directly.
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would grow a blocking uat door for this fixture's
        // unclassified "demo" feature — most of this module's tests are
        // about bookkeeping/commit mechanics, not the uat door, so pin
        // "off" here; the handful of tests that DO exercise uat_stop
        // override this seed with their own explicit config.
        w(root, ".bee/config.json", "{\"uat_stop\": \"off\"}\n");
        // D7 (doc-deferral-baseline): every close driven through this
        // fixture runs the doc-deferral door for real, and a repo with NO
        // baseline file is in SEED state — the door would write one (D6:
        // always, even with nothing to record) and the bookkeeping-commit
        // tests below would see a `.bee` path they never asked about.
        // Seeding an empty baseline into the SEED COMMIT keeps those tests
        // in ENFORCE mode, so each goes on asserting exactly what it
        // asserts today about commit scoping.
        w(root, ".bee/doc-deferral-baseline.json", "{\"files\":{}}\n");
        // hm-10 (human-mailbox D4/D14): a green close now records its
        // feature-close stop under `.bee/human-mailbox/`, which bee's own
        // onboarding block puts in `.gitignore` — it is a per-checkout
        // RUNTIME record, never bookkeeping. This fixture had no ignore file
        // at all, so it modelled a checkout bee never produces and the
        // untracked entry read as store dirt. Seeded here rather than
        // loosened in each assertion: every "the store is clean" and "only
        // these paths were committed" test goes on asserting exactly what it
        // asserted before, against a repo shaped the way a real one is.
        w(root, ".gitignore", ".bee/human-mailbox/\n");
        git_ok(
            root,
            &["add", ".bee/config.json", ".bee/doc-deferral-baseline.json", ".gitignore"],
        );
        // D-P3-1: this SEED commit is fixture setup, not the code under
        // test, so it passes `--no-gpg-sign` directly rather than relying on
        // the repo's own (unset) config — a developer whose GLOBAL
        // `commit.gpgsign` is `true` would otherwise have this bare `git
        // commit` try to sign (and hang or fail on a missing/misconfigured
        // agent) before any test module logic even runs.
        git_ok(root, &["commit", "-q", "--no-gpg-sign", "-m", "seed"]);
    }

    fn dirty_tracked_bee_file(root: &Path) {
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would grow a blocking uat door for this fixture's
        // unclassified "demo" feature — this helper is shared by tests
        // about the bookkeeping commit itself, not the uat door, so pin
        // "off" here to keep them exercising what they were about.
        w(root, ".bee/config.json", "{\"seeded\": true, \"uat_stop\": \"off\"}\n");
    }

    #[test]
    fn green_close_commits_only_dirty_bee_paths_leaving_unrelated_dirt_uncommitted() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        dirty_tracked_bee_file(root);
        w(root, "unrelated.txt", "unrelated dirt\n");

        let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(result, _text, code) = out else { panic!("expected Emit") };
        assert_eq!(code, 0);
        assert_eq!(result["bookkeeping_commit"]["committed"], json!(true));
        assert!(result["bookkeeping_commit"]["sha"].as_str().is_some_and(|s| !s.is_empty()));

        // The dirtied tracked file plus the token-usage record this green
        // close wrote (62331863) — both `.bee`, both staged by the same
        // path-scoped `git add`. `unrelated.txt` is the point: it is dirt
        // outside `.bee`, so it stays out of the commit.
        let committed = git_committed_paths(root);
        assert_eq!(
            committed,
            vec![".bee/config.json".to_string(), ".bee/usage/demo.json".to_string()]
        );
        let status = git_status_porcelain(root);
        assert!(status.contains("unrelated.txt"), "{status}");
        assert!(!status.contains(".bee/"), "{status}");
        let subject = git_out(root, &["log", "-1", "--pretty=%s"]);
        assert_eq!(subject, "Record demo close bookkeeping in the bee store");
    }

    /// P3-3: proves the `rev-parse --is-inside-work-tree` detection also
    /// reads a LINKED worktree — the second worktree `git worktree add`
    /// creates, whose own `.git` is a FILE (a gitdir pointer back at the
    /// main checkout's `.git/worktrees/<name>`) rather than the directory
    /// every other test in this file relies on. The commit must land on
    /// the linked worktree's own branch, not the main checkout's.
    #[test]
    fn linked_worktree_root_commits_on_its_own_branch() {
        let base = tempfile::tempdir().unwrap();
        let main_root = base.path().join("main");
        std::fs::create_dir_all(&main_root).unwrap();
        init_bee_repo(&main_root);

        let linked = base.path().join("linked");
        let branch = "feature/linked";
        git_ok(&main_root, &["worktree", "add", linked.to_str().unwrap(), "-b", branch]);
        assert!(linked.join(".git").is_file(), "a linked worktree's .git must be a FILE, not a dir");

        dirty_tracked_bee_file(&linked);

        let out = close_handler(&linked, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(result, _text, code) = out else { panic!("expected Emit") };
        assert_eq!(code, 0);
        assert_eq!(result["bookkeeping_commit"]["committed"], json!(true));
        assert!(result["bookkeeping_commit"]["sha"].as_str().is_some_and(|s| !s.is_empty()));

        // Landed on the linked worktree's own branch, not main's.
        let current_branch = git_out(&linked, &["rev-parse", "--abbrev-ref", "HEAD"]);
        assert_eq!(current_branch, branch);
        let subject = git_out(&linked, &["log", "-1", "--pretty=%s"]);
        assert_eq!(subject, "Record demo close bookkeeping in the bee store");
        let committed = git_committed_paths(&linked);
        assert_eq!(committed, vec![".bee/config.json".to_string()]);

        // The main checkout's own branch never moved.
        let main_branch = git_out(&main_root, &["rev-parse", "--abbrev-ref", "HEAD"]);
        assert_ne!(main_branch, branch);
    }

    #[test]
    fn unrelated_staged_file_stays_staged_out_of_the_commit() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        dirty_tracked_bee_file(root);
        w(root, "staged.txt", "staged dirt\n");
        git_ok(root, &["add", "staged.txt"]);

        let commit = commit_close_bookkeeping(root, "demo");
        assert!(matches!(commit, BookkeepingCommit::Committed { .. }));

        let committed = git_committed_paths(root);
        assert_eq!(committed, vec![".bee/config.json".to_string()]);
        let status = git_status_porcelain(root);
        assert!(status.contains("A  staged.txt"), "{status}");
    }

    /// B-P1-2: pins the `--no-gpg-sign` flag `commit_close_bookkeeping`
    /// passes to its `git commit` — a repo with `commit.gpgsign true` AND a
    /// `gpg.program` pointed at a stub that always fails must still let the
    /// bookkeeping commit land. Without `--no-gpg-sign` this turns red: the
    /// commit would invoke the failing stub, `git commit` would exit
    /// non-zero, and this would return `Skipped { reason: "git_failed:…" }`
    /// instead of `Committed` (verified by hand: temporarily dropping the
    /// flag from `commit_close_bookkeeping` flips this assertion red,
    /// restored after).
    #[cfg(unix)]
    #[test]
    fn gpgsign_true_with_a_failing_signer_still_lands_the_bookkeeping_commit() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        dirty_tracked_bee_file(root);

        // A "gpg" that always fails — proves the bookkeeping commit never
        // invokes a signer at all, without ever risking a real hang.
        let stub = root.join("gpg-stub.sh");
        std::fs::write(&stub, "#!/bin/sh\nexit 1\n").unwrap();
        let mut perms = std::fs::metadata(&stub).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&stub, perms).unwrap();
        git_ok(root, &["config", "commit.gpgsign", "true"]);
        git_ok(root, &["config", "gpg.program", stub.to_str().unwrap()]);

        let commit = commit_close_bookkeeping(root, "demo");
        assert!(matches!(commit, BookkeepingCommit::Committed { .. }));
    }

    #[test]
    fn config_false_skips_the_commit_with_reason_config_off() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        // Dirties `.bee/config.json` itself — writing the `false` opt-out
        // into the very file that must stay uncommitted.
        w(root, ".bee/config.json", r#"{"close_commit_bookkeeping": false}"#);

        let commit = commit_close_bookkeeping(root, "demo");
        match commit {
            BookkeepingCommit::Skipped { reason, index_restored } => {
                assert_eq!(reason, "config_off");
                assert_eq!(index_restored, None);
            }
            BookkeepingCommit::Committed { .. } => panic!("expected no commit"),
        }
        assert!(!git_status_porcelain(root).is_empty(), "dirty .bee must stay uncommitted");
    }

    /// P2-3 defense-in-depth: `commit_close_bookkeeping`'s own `None` arm —
    /// `close_commit_bookkeeping_config` refusing a non-boolean, non-null
    /// value — is normally unreachable through `close_handler`, which
    /// refuses the whole close before this function ever runs. This calls
    /// `commit_close_bookkeeping` directly, bypassing that upfront gate, so
    /// the fallback arm itself still has a test pinning it to
    /// `reason: "config_off"`, same as the boolean-`false` case just above.
    #[test]
    fn non_boolean_config_reaching_commit_close_bookkeeping_directly_yields_config_off() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        w(root, ".bee/config.json", r#"{"close_commit_bookkeeping": "sometimes"}"#);

        let commit = commit_close_bookkeeping(root, "demo");
        match commit {
            BookkeepingCommit::Skipped { reason, index_restored } => {
                assert_eq!(reason, "config_off");
                assert_eq!(index_restored, None);
            }
            BookkeepingCommit::Committed { .. } => panic!("expected no commit"),
        }
        assert!(!git_status_porcelain(root).is_empty(), "dirty .bee must stay uncommitted");
    }

    /// P2-3: a non-boolean `close_commit_bookkeeping` used to fall silently
    /// through `commit_close_bookkeeping`'s own `config_off` skip. It now
    /// refuses the WHOLE close, up front, before anything else runs —
    /// retargeted from pinning that silent skip to pinning this typed
    /// refusal (exit via `Out::Thrown`, `finish()` maps that to exit 1 —
    /// reservations/emit.rs:168).
    #[test]
    fn non_boolean_config_refuses_the_whole_close_up_front() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        w(root, ".bee/config.json", r#"{"close_commit_bookkeeping": "sometimes"}"#);
        let store_before = std::fs::read_to_string(root.join(".bee/config.json")).unwrap();

        let out = close_handler(root, "demo", false, None, None, &HashMap::new());
        match out {
            Ok(Out::Thrown(msg)) => {
                assert!(msg.contains("close_commit_bookkeeping"), "{msg}");
                assert!(msg.contains("\"sometimes\""), "{msg}");
                // B-P2-4: the bad value lives ONLY in the tracked file here,
                // so the refusal must name it, not the (absent) overlay.
                assert!(msg.contains(".bee/config.json"), "{msg}");
                assert!(!msg.contains(".bee/config.local.json"), "{msg}");
            }
            Ok(Out::Emit(..)) => panic!("expected a refusal, got an Emit"),
            Err(_) => panic!("expected Ok(Out::Thrown(_))"),
        }
        // Nothing ran, nothing committed: the config file itself is
        // untouched and no commit was created off the seeded HEAD.
        let store_after = std::fs::read_to_string(root.join(".bee/config.json")).unwrap();
        assert_eq!(store_before, store_after, "the store must stay untouched by a refused close");
        assert!(git_status_porcelain(root).contains(".bee/config.json"), "{}", git_status_porcelain(root));
        let log = git_out(root, &["log", "--oneline"]);
        assert_eq!(log.lines().count(), 1, "no new commit: {log}");
    }

    /// B-P2-4: a bad value living ONLY in the untracked overlay
    /// (`.bee/config.local.json`) must have the refusal name THAT file, not
    /// the hardcoded `.bee/config.json` — `read_config_raw` (state.rs)
    /// merges the overlay OVER the tracked file, so the tracked file (here,
    /// just `{}`) never carries the offending value at all.
    #[test]
    fn non_boolean_value_only_in_local_overlay_names_config_local_json() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        w(root, ".bee/config.local.json", r#"{"close_commit_bookkeeping": "sometimes"}"#);

        let out = close_handler(root, "demo", false, None, None, &HashMap::new());
        match out {
            Ok(Out::Thrown(msg)) => {
                assert!(msg.contains("close_commit_bookkeeping"), "{msg}");
                assert!(msg.contains("\"sometimes\""), "{msg}");
                assert!(msg.contains(".bee/config.local.json"), "{msg}");
                assert!(!msg.contains("in .bee/config.json "), "{msg}");
            }
            Ok(Out::Emit(..)) => panic!("expected a refusal, got an Emit"),
            Err(_) => panic!("expected Ok(Out::Thrown(_))"),
        }
    }

    /// B-P2-4: `null` is bee's own unset idiom — a `close_commit_bookkeeping`
    /// of `null` must read exactly like an absent key (defaults on), never
    /// refuse the close the way every other non-boolean value does.
    #[test]
    fn null_config_value_reads_as_unset_and_close_proceeds_normally() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        w(root, ".bee/config.json", r#"{"close_commit_bookkeeping": null, "uat_stop": "off"}"#);

        let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(result, _text, code) = out else { panic!("expected Emit, got a refusal") };
        assert_eq!(code, 0);
        assert_eq!(result["bookkeeping_commit"]["committed"], json!(true));
        assert!(result["bookkeeping_commit"]["sha"].as_str().is_some_and(|s| !s.is_empty()));
    }

    #[test]
    fn dry_run_and_red_close_never_commit() {
        // --dry-run: close_handler returns before the bookkeeping code runs
        // at all — no `bookkeeping_commit` field, nothing committed.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        dirty_tracked_bee_file(root);
        let out = close_handler(root, "demo", true, None, None, &HashMap::new()).unwrap();
        let Out::Emit(result, _text, code) = out else { panic!("expected Emit") };
        assert_eq!(code, 0);
        assert!(result.get("bookkeeping_commit").is_none(), "{result}");
        assert!(!git_status_porcelain(root).is_empty(), "dry-run must never commit");

        // D7: a capped cell with no valid proof line stops close at the
        // tests door before the bookkeeping code is ever reached.
        let tmp2 = tempfile::tempdir().unwrap();
        let root2 = tmp2.path();
        init_bee_repo(root2);
        dirty_tracked_bee_file(root2);
        w(
            root2,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"report":{"outcome":"o","commit":"c","files":[],"tests":"","deviations":[]}}}"#,
        );
        let out = close_handler(root2, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(result, _text, code) = out else { panic!("expected Emit") };
        assert_eq!(code, 1);
        assert!(result.get("bookkeeping_commit").is_none(), "{result}");
        assert!(!git_status_porcelain(root2).is_empty(), "a proof-debt-refused close must never commit");
    }

    /// P3-4: `GIT_CEILING_DIRECTORIES` is process environment, so a bare
    /// `set_var` around one assertion would leak into every other test that
    /// spawns `git` while it was set. Scoped to exactly the life of one
    /// test: `new` records whatever value (or absence) was already there
    /// and pins the ceiling to `dir`; `Drop` puts it straight back — a
    /// caller can never forget to unwind it, even on a panicking assert.
    struct GitCeilingGuard {
        prior: Option<std::ffi::OsString>,
    }

    impl GitCeilingGuard {
        fn new(dir: &Path) -> Self {
            let prior = std::env::var_os("GIT_CEILING_DIRECTORIES");
            // SAFETY: no other thread reads/writes this specific var across
            // this guard's lifetime — nothing else in this crate consults
            // GIT_CEILING_DIRECTORIES, and it exists only to steer this
            // one test's own `git` child processes.
            unsafe { std::env::set_var("GIT_CEILING_DIRECTORIES", dir) };
            GitCeilingGuard { prior }
        }
    }

    impl Drop for GitCeilingGuard {
        fn drop(&mut self) {
            // SAFETY: see `new` above.
            match self.prior.take() {
                Some(v) => unsafe { std::env::set_var("GIT_CEILING_DIRECTORIES", v) },
                None => unsafe { std::env::remove_var("GIT_CEILING_DIRECTORIES") },
            }
        }
    }

    #[test]
    fn non_repo_root_reports_not_a_repo_and_close_stays_green() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/config.json", "{\"uat_stop\": \"off\"}\n"); // no `git init` — not a repo at all

        // P3-4: pin the ceiling to the tempdir's own parent for the life of
        // this test — a TMPDIR that happens to sit under a real git
        // checkout must never let `rev-parse --is-inside-work-tree` walk up
        // into that enclosing repo and answer "true" by accident.
        let _ceiling = GitCeilingGuard::new(root.parent().unwrap());

        let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(result, _text, code) = out else { panic!("expected Emit") };
        assert_eq!(code, 0);
        assert_eq!(
            result["bookkeeping_commit"],
            json!({"committed": false, "reason": "not_a_repo"})
        );
    }

    /// P2-4(b), rewritten by 62331863: this fixture has no
    /// `docs/knowledge/` bundle and no cells for "demo", so promote is
    /// skipped and retirement moves nothing — the ONE thing a green close
    /// writes on top of the clean seed commit is its token-usage record. That
    /// record now lands at `.bee/usage/demo.json`, INSIDE the `.bee` scope
    /// this commit stages, so the green close no longer reports
    /// `reason: "clean"`: it commits the record it just wrote and leaves the
    /// tree clean. `reason: "clean"` itself is still pinned, one step later,
    /// on a store that really has nothing left to commit.
    #[test]
    fn green_close_commits_the_usage_record_it_just_wrote() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);

        let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(result, _text, code) = out else { panic!("expected Emit") };
        assert_eq!(code, 0);
        assert_eq!(result["bookkeeping_commit"]["committed"], json!(true), "{result}");
        assert_eq!(result["usage_record"], json!(".bee/usage/demo.json"));

        // The record is IN the commit, not merely on disk — no merge-time
        // auto-commit is needed to preserve it any more.
        assert_eq!(git_committed_paths(root), vec![".bee/usage/demo.json".to_string()]);
        let status = git_status_porcelain(root);
        assert!(!status.contains(".bee"), "close must leave nothing in .bee dirty: {status}");

        // And with the record committed, the store really is clean.
        assert_eq!(
            commit_close_bookkeeping(root, "demo").value(),
            json!({"committed": false, "reason": "clean"})
        );
    }

    /// P2-4(a) + P2-1 + P2-2: a `pre-commit` hook that fails SILENTLY (exit
    /// 1, nothing on either stream) drives the `git commit` failure branch.
    /// Proves three things at once: P2-2's non-empty fallback reason (the
    /// bare `git_failed:` prefix is impossible), P2-1's best-effort
    /// `git reset -- .bee` after the failed commit (`.bee` ends up dirty
    /// but UNSTAGED, never left sitting staged), and that the failure stays
    /// warn-never-block (`close` itself still exits 0).
    #[cfg(unix)]
    #[test]
    fn silent_pre_commit_hook_failure_restores_the_index_and_stays_green() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        dirty_tracked_bee_file(root);

        let hook = root.join(".git/hooks/pre-commit");
        std::fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
        let mut perms = std::fs::metadata(&hook).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook, perms).unwrap();

        let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(result, text, code) = out else { panic!("expected Emit") };
        // Warn-never-block: the hook killed the bookkeeping commit, not close.
        assert_eq!(code, 0);
        assert_eq!(
            result["bookkeeping_commit"],
            json!({"committed": false, "reason": "git_failed:exit status 1", "index_restored": true})
        );
        assert!(
            text.contains("Warning: bee-store bookkeeping commit failed for \"demo\": exit status 1 (index restored)"),
            "{text}"
        );

        // `.bee` is dirty (the hook blocked the commit) but UNSTAGED — the
        // index column (first of the two porcelain characters) must never
        // read `A` or `M` on a `.bee` line once the reset ran.
        //
        // `git_status_porcelain`'s helper `.trim()`s the WHOLE captured
        // string (fine for the `contains(...)` checks every other test in
        // this file makes), which would eat exactly the leading space this
        // assertion depends on when `.bee/config.json` is the first porcelain
        // line — so this reads the porcelain output raw instead.
        let raw = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(root)
            .output()
            .unwrap();
        let status = String::from_utf8_lossy(&raw.stdout).into_owned();
        assert!(status.contains(".bee/config.json"), "{status}");
        for line in status.lines() {
            if line.trim_end().ends_with(".bee/config.json") {
                let index_col = line.chars().next().unwrap_or(' ');
                assert!(
                    index_col == ' ' || index_col == '?',
                    "expected .bee/config.json unstaged, got index column {index_col:?}: {status}"
                );
            }
        }
    }

    // ─── tests: deferred-queue reconciliation (trun-9 rework) ──────────────
    //
    // The judge's second FAIL: nothing exercised `scribing_debt`'s two
    // queue-facing behaviors (materializing a `scribe` record, and clearing
    // once one completes) or `enqueue_promote_deferred_record`'s write —
    // every debt-door test elsewhere runs with no `.bee/deferred-queue.jsonl`
    // at all. These cover both enqueue paths directly.

    fn capped_behavior_change_cell(root: &Path, feature: &str, id: &str, capped_at: &str) {
        w(
            root,
            &format!(".bee/cells/{id}.json"),
            &format!(
                r#"{{"id":"{id}","feature":"{feature}","status":"capped","files":["src/{id}.rs"],"trace":{{"behavior_change":true,"capped_at":"{capped_at}"}}}}"#
            ),
        );
    }

    #[test]
    fn scribing_debt_materializes_a_scribe_record_for_debt_with_no_record_yet() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        capped_behavior_change_cell(root, "demo", "demo-1", "2026-08-10T00:00:00.000Z");

        let debt = scribing_debt(root, "demo").unwrap();
        assert_eq!(debt.ids, vec![json!("demo-1")]);

        let queued = crate::verbs::deferred_queue::items_for(root, "scribe", "demo");
        assert_eq!(queued.len(), 1, "expected exactly one materialized record");
        assert_eq!(queued[0].cells, vec!["demo-1".to_string()]);
        assert!(!queued[0].completed);

        // The record's own `files` come from the cell's declared `files` —
        // read raw since `QueuedItem` (the read-only view every caller
        // shares) deliberately strips that detail nobody but this write path
        // needs.
        let raw = std::fs::read_to_string(root.join(".bee/deferred-queue.jsonl")).unwrap();
        assert!(raw.contains("\"files\":[\"src/demo-1.rs\"]"), "{raw}");
    }

    #[test]
    fn scribing_debt_never_double_materializes_once_a_record_already_names_the_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        capped_behavior_change_cell(root, "demo", "demo-1", "2026-08-10T00:00:00.000Z");

        // First call materializes; a second call over the same still-open
        // debt must not enqueue a second record for the same cell.
        scribing_debt(root, "demo").unwrap();
        scribing_debt(root, "demo").unwrap();

        let queued = crate::verbs::deferred_queue::items_for(root, "scribe", "demo");
        assert_eq!(queued.len(), 1, "a second scan must never double-materialize: {:?}", queued.iter().map(|q| &q.cells).collect::<Vec<_>>());
    }

    #[test]
    fn scribing_debt_clears_once_its_materialized_record_is_completed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        capped_behavior_change_cell(root, "demo", "demo-1", "2026-08-10T00:00:00.000Z");

        let (id, _ts) =
            crate::verbs::deferred_queue::enqueue(root, "scribe", "demo", &["demo-1".to_string()], &[], &[], "debt")
                .unwrap();
        // `deferred-queue complete` appends the completion event this scan's
        // `queue_completed` check folds over — same shape the CLI writes.
        crate::fsutil::append_jsonl(
            &root.join(".bee").join("deferred-queue.jsonl"),
            &json!({"ts": now_iso(), "event": "complete", "id": id}),
        )
        .unwrap();

        let debt = scribing_debt(root, "demo").unwrap();
        assert_eq!(debt.ids, Vec::<Value>::new(), "a completed record must clear the debt");
    }

    #[test]
    fn enqueue_promote_deferred_record_writes_a_promote_record_naming_feature_and_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        enqueue_promote_deferred_record(root, "demo", "docs/history/demo/promote-proposals.md");

        let raw = std::fs::read_to_string(root.join(".bee/deferred-queue.jsonl")).unwrap();
        assert!(raw.contains("\"kind\":\"promote\""), "{raw}");
        assert!(raw.contains("\"feature\":\"demo\""), "{raw}");
        assert!(raw.contains("\"files\":[\"docs/history/demo/promote-proposals.md\"]"), "{raw}");

        // `status_full::unapplied_promote_proposals` never checks `cells` for
        // a `promote` record — `completed` alone is what `items_for` gives a
        // caller who only asks "is this feature's proposal applied?".
        let queued = crate::verbs::deferred_queue::items_for(root, "promote", "demo");
        assert_eq!(queued.len(), 1);
        assert!(!queued[0].completed);
    }

    // ─── mcl-3 (merge-closes-the-lane R2): green close sets the lane's
    // terminal phase ──────────────────────────────────────────────────────

    fn read_lane_json(root: &Path, feature: &str) -> Value {
        let ReadJson::Parsed(v) = read_json(&root.join(".bee/lanes").join(format!("{feature}.json")))
        else {
            panic!("lane record must still parse")
        };
        v
    }

    /// Happy path (must-have: "a green non-dry-run close leaves the lane at
    /// phase idle with a next_action naming the close"): a non-terminal
    /// lane phase moves to "idle" and a next_action naming the close is
    /// stamped — on disk, not just claimed in the emitted text.
    #[test]
    fn a_green_close_sets_the_lane_phase_to_idle_with_a_next_action() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would grow a blocking uat door for this fixture's
        // unclassified lane — this test is about the phase write, not the
        // uat door, so pin "off".
        w(root, ".bee/config.json", r#"{"uat_stop": "off"}"#);
        w(root, ".bee/lanes/demo.json", r#"{"feature":"demo","phase":"swarming"}"#);
        let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(_, text, code) = out else { panic!("expected a green close") };
        assert_eq!(code, 0);
        assert!(text.contains("Lane phase set to \"idle\" for \"demo\""), "text: {text}");
        let lane = read_lane_json(root, "demo");
        assert_eq!(lane["phase"], json!("idle"));
        let next_action = lane["next_action"].as_str().unwrap_or_default();
        assert!(next_action.contains("close"), "next_action: {next_action}");
        assert!(next_action.contains("demo"), "next_action: {next_action}");
    }

    /// Edge (must-have: "--dry-run leaves the lane record byte-identical"):
    /// the phase write sits entirely past the dry-run branch, which returns
    /// early, near the top of `close_handler`.
    #[test]
    fn a_dry_run_close_leaves_the_lane_record_byte_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let before = r#"{"feature":"demo","phase":"swarming"}"#;
        w(root, ".bee/lanes/demo.json", before);
        let out = close_handler(root, "demo", true, None, None, &HashMap::new()).unwrap();
        let Out::Emit(..) = out else { panic!("expected the dry-run door report") };
        let after = std::fs::read_to_string(root.join(".bee/lanes/demo.json")).unwrap();
        assert_eq!(after, before);
    }

    /// Edge (must-have: "a close stopped by any BLOCKING door leaves the
    /// lane record byte-identical"): the proof-debt tests door stops this
    /// close at exit 1, well before the tail line this feature's write
    /// lives on.
    #[test]
    fn a_close_blocked_by_a_door_leaves_the_lane_record_byte_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let before = r#"{"feature":"demo","phase":"swarming"}"#;
        w(root, ".bee/lanes/demo.json", before);
        w(
            root,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"report":{"outcome":"o","commit":"c","files":[],"tests":"not a proof string","deviations":[]}}}"#,
        );
        let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(_, _text, code) = out else { panic!("expected the proof-debt refusal") };
        assert_eq!(code, 1);
        let after = std::fs::read_to_string(root.join(".bee/lanes/demo.json")).unwrap();
        assert_eq!(after, before);
    }

    /// Edge (must-have: "a lane already at a terminal phase ('idle' or
    /// 'compounding-complete') is not rewritten"): a green close leaves
    /// either terminal value byte-identical rather than rewriting it —
    /// close never writes "compounding-complete" itself, but must not
    /// clobber a lane that already carries it.
    #[test]
    fn a_lane_already_terminal_is_left_untouched_by_a_green_close() {
        for phase in ["idle", "compounding-complete"] {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path();
            let before = format!(r#"{{"feature":"demo","phase":"{phase}"}}"#);
            w(root, ".bee/config.json", r#"{"uat_stop": "off"}"#);
            w(root, ".bee/lanes/demo.json", &before);
            let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
            let Out::Emit(_, text, code) = out else { panic!("expected a green close") };
            assert_eq!(code, 0);
            assert!(
                !text.contains("Lane phase set to"),
                "an already-terminal lane must not be rewritten: {text}"
            );
            let after = std::fs::read_to_string(root.join(".bee/lanes/demo.json")).unwrap();
            assert_eq!(after, before, "phase {phase} must stay byte-identical");
        }
    }

    /// Error (must-have: "a failing phase write emits a warning line and
    /// the close still exits 0"): a lane whose pre-mutation phase sits
    /// outside the known-phase enum makes `run_set_body`'s own phase-known
    /// guard refuse the write — this warns on its own line rather than
    /// failing the close, and nothing reaches disk.
    #[test]
    fn a_failing_lane_phase_write_warns_and_the_close_still_exits_0() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // defaults-and-agent-env D1: this test is about the phase-write
        // warning, not the uat door — pin "off" so absent uat_stop's new
        // Close default doesn't also block this close.
        w(root, ".bee/config.json", r#"{"uat_stop": "off"}"#);
        w(root, ".bee/lanes/demo.json", r#"{"feature":"demo","phase":"frobnicating"}"#);
        let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(_, text, code) = out else {
            panic!("expected a green close despite the lane write failing")
        };
        assert_eq!(code, 0);
        assert!(text.contains("Warning: could not set lane phase"), "text: {text}");
        assert!(text.starts_with("Tests GREEN for \"demo\""), "text: {text}");
        let after = std::fs::read_to_string(root.join(".bee/lanes/demo.json")).unwrap();
        assert_eq!(after, r#"{"feature":"demo","phase":"frobnicating"}"#);
    }

    // ─── tests: uat-stop-placement D4.4/D2 close-time uat door ─────────────
    //
    // Precedent: judge-debt (above, `judge_debt_door_*`/`close_refuses_
    // judge_debt_for_a_standard_lane_feature`, verbs/cells/tests.rs) —
    // lane-scoped, blocking, escapable by a logged deferral decision. These
    // tests live here rather than there because the door itself, and every
    // helper it calls, is local to this file (this cell's scope is
    // drivers/close.rs alone).

    fn write_lane_mode(root: &Path, feature: &str, mode: &str) {
        w(root, &format!(".bee/lanes/{feature}.json"), &format!(r#"{{"feature":"{feature}","mode":"{mode}"}}"#));
    }

    /// usp-3 revision (D4): a lane record whose `mode` and `route.lane`
    /// name DIFFERENT lane classes — the one shape `write_lane_mode` above
    /// cannot produce, and the exact shape that let `feature_route`
    /// (`route.lane`) and the merge side's `mode` read disagree.
    fn write_lane_mode_and_route(root: &Path, feature: &str, mode: &str, route_lane: &str) {
        w(
            root,
            &format!(".bee/lanes/{feature}.json"),
            &format!(r#"{{"feature":"{feature}","mode":"{mode}","route":{{"lane":"{route_lane}"}}}}"#),
        );
    }

    fn write_uat_gate_state(root: &Path, feature: &str, approved: bool) {
        w(
            root,
            ".bee/state.json",
            &format!(r#"{{"feature":"{feature}","approved_gates":{{"uat":{approved}}}}}"#),
        );
    }

    /// D4.4: under `uat_stop: "merge"` (explicit) and under `"off"`, the
    /// door does not appear in the door list at all, even for a standard
    /// lane whose uat gate is unapproved.
    #[test]
    fn uat_door_is_absent_under_merge_and_under_off() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_lane_mode(root, "demo", "standard");

        w(root, ".bee/config.json", r#"{"uat_stop":"merge"}"#);
        let doors = build_close_report_doors(root, "demo").unwrap();
        assert!(doors.iter().find(|d| d.door == "uat").is_none(), "merge must never grow the door");

        w(root, ".bee/config.json", r#"{"uat_stop":"off"}"#);
        let doors = build_close_report_doors(root, "demo").unwrap();
        assert!(doors.iter().find(|d| d.door == "uat").is_none(), "off must never grow the door");
    }

    /// defaults-and-agent-env D1: with no `uat_stop`/`uat_before_merge` key
    /// at all, absent now reads as `Close` — a standard lane whose uat gate
    /// is unapproved grows a blocking uat door, same as an explicit
    /// `uat_stop: "close"`.
    #[test]
    fn uat_door_blocks_a_standard_lane_feature_when_uat_stop_is_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_lane_mode(root, "demo", "standard");

        let doors = build_close_report_doors(root, "demo").unwrap();
        let uat_door = doors.iter().find(|d| d.door == "uat").expect("door must exist when uat_stop is absent (default close)");
        assert!(uat_door.blocking, "an unapproved uat gate must block when uat_stop is absent");
    }

    /// D2, D4.4: under `close`, a standard lane whose uat gate is
    /// unapproved grows a BLOCKING uat door naming the remedy command.
    #[test]
    fn uat_door_blocks_a_standard_lane_feature_under_close_with_uat_unapproved() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/config.json", r#"{"uat_stop":"close"}"#);
        write_lane_mode(root, "demo", "standard");

        let doors = build_close_report_doors(root, "demo").unwrap();
        let uat_door = doors.iter().find(|d| d.door == "uat").expect("door must exist under close for a standard lane");
        assert!(uat_door.blocking, "an unapproved uat gate must block under close");
        assert_eq!(uat_door.command, Some("bee gate --name uat --approved true"));
        assert!(uat_door.detail.contains("not yet approved"), "{}", uat_door.detail);
    }

    /// D2, D4.4: the same lane, once the uat gate is approved (via the
    /// default-state fallback, same-feature-owned), clears the door — still
    /// present, no longer blocking.
    #[test]
    fn uat_door_does_not_block_once_uat_is_approved() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/config.json", r#"{"uat_stop":"close"}"#);
        write_lane_mode(root, "demo", "standard");
        write_uat_gate_state(root, "demo", true);

        let doors = build_close_report_doors(root, "demo").unwrap();
        let uat_door = doors.iter().find(|d| d.door == "uat").expect("door must exist under close for a standard lane");
        assert!(!uat_door.blocking, "an approved uat gate must not block");
        assert_eq!(uat_door.detail, "clear");
        assert_eq!(uat_door.command, None);
    }

    /// D2: the same lane rule the merge side uses — `tiny`/`small`/`docs`/
    /// `spike` are exempt, so the door never blocks for them even with the
    /// uat gate unapproved.
    #[test]
    fn uat_door_does_not_block_for_exempt_lanes_under_close() {
        for lane in ["tiny", "small", "docs", "spike"] {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path();
            w(root, ".bee/config.json", r#"{"uat_stop":"close"}"#);
            write_lane_mode(root, "demo", lane);

            let doors = build_close_report_doors(root, "demo").unwrap();
            let uat_door = doors.iter().find(|d| d.door == "uat");
            if let Some(uat_door) = uat_door {
                assert!(!uat_door.blocking, "lane {lane} must never block: {}", uat_door.detail);
            }
        }
    }

    /// uat-lane-source (uls-1): the door classifies through `route.lane`,
    /// not through `mode`. `mode` carries the WORKFLOW vocabulary
    /// (feature, release) while the exemption is written in LANE vocabulary
    /// (tiny, small, docs, spike, standard, high-risk), so reading `mode`
    /// meant the exemption almost never fired and every feature was asked
    /// for uat. Agreement with the merge side is preserved by construction,
    /// not by matching sources by hand: both sides call the one helper
    /// `crate::uat::uat_lane_mode` (here, and `verbs/worktree/phases.rs`),
    /// so flipping it flipped both together.
    ///
    /// This case supersedes the usp-3 D4 pair that pinned the `mode` read.
    /// A record whose `mode` is "standard" but whose `route.lane` names
    /// "small" is EXEMPT: the lane is what the exemption speaks about.
    #[test]
    fn uat_door_reads_route_lane_and_is_exempt_when_mode_disagrees_toward_standard() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/config.json", r#"{"uat_stop":"close"}"#);
        write_lane_mode_and_route(root, "demo", "standard", "small");

        let doors = build_close_report_doors(root, "demo").unwrap();
        let uat_door = doors.iter().find(|d| d.door == "uat");
        if let Some(uat_door) = uat_door {
            assert!(
                !uat_door.blocking,
                "route.lane=small must stay exempt even though mode says standard — the exemption is written in lane vocabulary: {}",
                uat_door.detail
            );
        }
    }

    /// Mirror of the above: a record whose `mode` is "small" but whose
    /// `route.lane` names "standard" BLOCKS, because `route.lane` is the
    /// source both sides read.
    #[test]
    fn uat_door_reads_route_lane_and_blocks_when_mode_disagrees_toward_small() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/config.json", r#"{"uat_stop":"close"}"#);
        write_lane_mode_and_route(root, "demo", "small", "standard");

        let doors = build_close_report_doors(root, "demo").unwrap();
        let uat_door = doors
            .iter()
            .find(|d| d.door == "uat")
            .expect("route.lane standard must grow a blocking door even when mode disagrees");
        assert!(
            uat_door.blocking,
            "route.lane=standard must block — the merge side reads the same helper and would wait on this feature: {}",
            uat_door.detail
        );
    }

    /// D1 (`uat_stop_config`'s own fail-closed read): a bogus `uat_stop`
    /// value blocks with an invalid-config detail naming both keys and
    /// their legal values, rather than resolving either way.
    #[test]
    fn uat_door_blocks_with_invalid_config_detail_on_a_bogus_uat_stop_value() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/config.json", r#"{"uat_stop":"sometime"}"#);
        write_lane_mode(root, "demo", "standard");

        let doors = build_close_report_doors(root, "demo").unwrap();
        let uat_door = doors.iter().find(|d| d.door == "uat").expect("a bogus value must still grow the door");
        assert!(uat_door.blocking, "an unresolvable uat_stop must block rather than guess");
        assert!(uat_door.detail.contains("invalid config"), "{}", uat_door.detail);
        assert!(uat_door.detail.contains("uat_stop"), "{}", uat_door.detail);
        assert!(uat_door.detail.contains("uat_before_merge"), "{}", uat_door.detail);
        assert!(uat_door.detail.contains("\"merge\""), "{}", uat_door.detail);
        assert!(uat_door.detail.contains("\"close\""), "{}", uat_door.detail);
        assert!(uat_door.detail.contains("\"off\""), "{}", uat_door.detail);
    }

    /// D2: mirrors the judge-debt door's own escape — a logged
    /// `uat-deferral` decision naming the feature clears the door without
    /// touching the underlying gate state.
    #[test]
    fn uat_door_is_cleared_by_a_logged_uat_deferral_decision() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/config.json", r#"{"uat_stop":"close"}"#);
        write_lane_mode(root, "demo", "standard");
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-08-18T00:00:00.000Z\",\"decision\":\"defer uat for demo\",\"rationale\":\"r\",\"tags\":[\"uat-deferral\"],\"scope\":\"repo\"}\n",
        );

        let doors = build_close_report_doors(root, "demo").unwrap();
        let uat_door = doors.iter().find(|d| d.door == "uat").unwrap();
        assert!(!uat_door.blocking, "a logged uat-deferral decision must clear the door");
        assert!(uat_door.detail.contains("deferred"), "{}", uat_door.detail);
        assert!(uat_door.detail.contains("demo"), "{}", uat_door.detail);
        assert_eq!(uat_door.command, None);

        // A uat-deferral decision naming a DIFFERENT feature never lifts
        // THIS feature's block.
        assert!(!has_uat_deferral_decision(root, "elsewhere").unwrap());
    }

    /// End to end: `bee close` on a standard-lane feature under
    /// `uat_stop: "close"` with an unapproved uat gate refuses even with no
    /// other debt in play, names the pinned prefix, and states the remedy
    /// in the user's own terms.
    #[test]
    fn close_refuses_the_uat_door_for_a_standard_lane_feature_under_uat_stop_close() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/config.json", r#"{"uat_stop":"close"}"#);
        write_lane_mode(root, "demo", "standard");

        let Out::Emit(result, text, code) = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1, "an unapproved uat gate refuses even though nothing else is in debt");
        let lines: Vec<&str> = text.split('\n').collect();
        assert!(lines[0].starts_with(CLOSE_UAT_PREFIX), "{}", lines[0]);
        assert!(lines[1].contains("bee gate --name uat --approved true"), "{}", lines[1]);
        assert!(lines[1].contains("worktree"), "remedy must name the worktree fix-forward path: {}", lines[1]);
        assert!(lines[2].starts_with("next:"));
        let doors = result.get("doors").unwrap().as_array().unwrap();
        assert_eq!(doors.iter().find(|d| d["door"] == "uat").unwrap()["blocking"], json!(true));
    }

    // ── the human mailbox: D14's feature-close stop (hm-10) ─────────────

    /// The run a close appends under is resolved from the environment, so a
    /// test asks the store which run it wrote rather than assuming a name.
    fn mailbox_entries(root: &Path) -> Vec<crate::verbs::mailbox::Entry> {
        crate::verbs::mailbox::runs_with_entries(root)
            .into_iter()
            .flat_map(|run| crate::verbs::mailbox::read_entries(root, &run))
            .collect()
    }

    fn capped_cell(root: &Path) {
        w(
            root,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped",
                "acceptance":"A letter is filed once per run and never twice",
                "affects_skills":["bee-hive"],"affects_specs":["docs/specs/letters.md"],
                "trace":{"files_changed":["src/one.rs","src/two.rs"],
                "report":{"outcome":"o","commit":"c","files":["src/one.rs"],
                "tests":"cargo test — green — the touched module","deviations":[]}}}"#,
        );
    }

    #[test]
    fn a_dry_run_close_stops_nothing_so_it_appends_no_mailbox_entry() {
        // The fork the cap has no equivalent of: --dry-run lists the doors,
        // writes nothing and STOPS nothing. A letter reports what a run DID.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        capped_cell(root);

        let out = close_handler(root, "demo", true, None, None, &HashMap::new()).unwrap();
        let Out::Emit(_result, _text, code) = out else { panic!("expected Emit") };
        assert_eq!(code, 0);
        assert!(
            !crate::verbs::mailbox::entries_dir(root).exists(),
            "a dry-run close appended a mailbox entry"
        );
    }

    #[test]
    fn a_refused_close_appends_no_mailbox_entry_either() {
        // The stop has not happened: close stopped at the tests door, so
        // nothing about this feature is finished and nothing is recorded.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        w(
            root,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"report":{"outcome":"o","commit":"c","files":[],"tests":"","deviations":[]}}}"#,
        );

        let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(_result, _text, code) = out else { panic!("expected Emit") };
        assert_eq!(code, 1, "this close must refuse at the tests door");
        assert!(
            !crate::verbs::mailbox::entries_dir(root).exists(),
            "a refused close appended a mailbox entry"
        );
    }

    #[test]
    fn a_green_close_appends_one_feature_close_entry_carrying_the_three_lists() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        capped_cell(root);

        let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(_result, _text, code) = out else { panic!("expected Emit") };
        assert_eq!(code, 0, "this close must be green");

        let entries = mailbox_entries(root);
        assert_eq!(entries.len(), 1, "one close, one stop");
        assert_eq!(entries[0].kind, crate::verbs::mailbox::KIND_FEATURE_CLOSE);
        // D8: the plain sentence is written at the moment of the stop.
        assert_eq!(entries[0].what, crate::verbs::mailbox::close_sentence("demo"));
        // The close itself edited nothing, and it had no plan to depart from.
        assert!(entries[0].files.is_empty());
        assert!(entries[0].departure.is_none());

        // D14's three lists, each read out of the feature's own capped cell —
        // which retirement has already moved into the archive by now.
        let note = feature_close_note(root, "demo", None);
        assert_eq!(note.architecture, vec!["src/one.rs".to_string(), "src/two.rs".to_string()]);
        assert_eq!(
            note.behaviour,
            vec!["A letter is filed once per run and never twice".to_string()]
        );
        assert_eq!(note.usage, vec!["bee-hive".to_string(), "docs/specs/letters.md".to_string()]);
    }

    #[test]
    fn the_feature_close_material_is_read_never_written() {
        // D8: a feature that recorded none of a fact gets an EMPTY list, and
        // the letter drops that section — nothing is invented to fill it.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        w(
            root,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"report":{"outcome":"o","commit":"c","files":[],"tests":"cargo test — green — one module","deviations":[]}}}"#,
        );
        let note = feature_close_note(root, "demo", None);
        assert!(note.is_empty(), "a feature that recorded nothing must carry nothing: {note:?}");

        // And a feature nobody ever worked on carries nothing either.
        assert!(feature_close_note(root, "never-existed", None).is_empty());

        // e97cc9d4: the token-usage line is the ONE string this note takes
        // from outside the cells, and it is passed through untouched — a
        // blank one is dropped rather than stored as an empty bullet.
        let priced = feature_close_note(root, "demo", Some("usage: 1 session(s) — 9 tokens"));
        assert_eq!(priced.token_usage, vec!["usage: 1 session(s) — 9 tokens".to_string()]);
        assert!(feature_close_note(root, "demo", Some("   ")).is_empty());
    }

    // ─── tests: the token-usage section (decision 2d3abd12) ────────────────

    /// One transcript event, in the exact shape `aggregate_usage` reads:
    /// `type: "assistant"` with a truthy `message.model` and a `message.usage`
    /// block. `requestId` keeps the de-duplication path honest, `timestamp` is
    /// what `walk_subagents` needs to accept a sidecar at all.
    fn usage_event(model: &str, req: &str, input: u32, output: u32, cw: u32, cr: u32) -> String {
        format!(
            r#"{{"type":"assistant","requestId":"{req}","timestamp":"2026-08-30T12:00:00.000Z","message":{{"model":"{model}","usage":{{"input_tokens":{input},"output_tokens":{output},"cache_creation_input_tokens":{cw},"cache_read_input_tokens":{cr}}}}}}}"#
        )
    }

    /// Writes `.bee/sessions/<id>.json` and, when `main` is given, the
    /// transcript it points at — inside the SAME tempdir, so `transcript_path`
    /// is a real absolute path that `.exists()` answers true for.
    fn write_usage_session(
        root: &Path,
        id: &str,
        lane: Option<&str>,
        main: Option<&str>,
        subagent: Option<&str>,
    ) {
        let transcript = root.join("transcripts").join(format!("{id}.jsonl"));
        let lane_field = lane.map(|l| format!(r#","lane":"{l}""#)).unwrap_or_default();
        // The path goes into a JSON string literal: escape the separator so a
        // win32 checkout does not write an unparseable record.
        let transcript_json = transcript.to_string_lossy().replace('\\', "\\\\");
        w(
            root,
            &format!(".bee/sessions/{id}.json"),
            &format!(r#"{{"id":"{id}","transcript_path":"{transcript_json}"{lane_field}}}"#),
        );
        if let Some(main) = main {
            w(root, &format!("transcripts/{id}.jsonl"), &format!("{main}\n"));
        }
        if let Some(sub) = subagent {
            w(root, &format!("transcripts/{id}/subagents/a.jsonl"), &format!("{sub}\n"));
        }
    }

    /// The fixture the sum tests share: two readable sessions (one bound to
    /// the lane and carrying a subagent sidecar, one reached only because it
    /// is the CALLING session), one lane-bound session whose transcript is
    /// gone, and one readable session belonging to a different feature that
    /// must not be counted at all.
    fn usage_fixture(root: &Path) {
        write_usage_session(
            root,
            "s-bound",
            Some("demo"),
            Some(&usage_event("opus", "r1", 100, 10, 5, 1000)),
            Some(&usage_event("sonnet", "s1", 50, 4, 1, 200)),
        );
        write_usage_session(root, "s-call", None, Some(&usage_event("opus", "r2", 200, 20, 0, 0)), None);
        // Lane-bound, but the transcript it names was never written.
        write_usage_session(root, "s-gone", Some("demo"), None, None);
        // A readable transcript that belongs to somebody else's feature.
        write_usage_session(
            root,
            "s-other",
            Some("other-feature"),
            Some(&usage_event("opus", "r3", 9_000, 9_000, 9_000, 9_000)),
            None,
        );
    }

    #[test]
    fn usage_candidates_are_the_lane_bound_records_plus_the_calling_session() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        usage_fixture(root);
        // s-other is bound to another lane; s-call has no lane at all and is
        // reached ONLY through the calling-session argument.
        assert_eq!(
            usage_session_ids(root, "demo", Some("s-call")),
            vec!["s-bound".to_string(), "s-call".to_string(), "s-gone".to_string()]
        );
        assert_eq!(
            usage_session_ids(root, "demo", None),
            vec!["s-bound".to_string(), "s-gone".to_string()]
        );
        // A calling session already bound to the lane is named once, not twice.
        assert_eq!(
            usage_session_ids(root, "demo", Some("s-bound")),
            vec!["s-bound".to_string(), "s-gone".to_string()]
        );
    }

    #[test]
    fn usage_sums_main_and_subagent_tokens_and_counts_the_unreadable_ones() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        usage_fixture(root);
        let usage = collect_close_usage(root, "demo", Some("s-call"));

        // s-bound and s-call rolled up; s-gone counted as skipped; s-other
        // never a candidate, so it is neither summed NOR skipped.
        assert_eq!(usage.sessions(), 2);
        assert_eq!(usage.skipped, 1);

        // main = s-bound (100+10+5 new, 1000 cached) + s-call (220 new, 0 cached)
        assert_eq!(usage.main.new_t, 335.0);
        assert_eq!(usage.main.cached, 1000.0);
        assert_eq!(usage.main.total, 1335.0);
        // subagents = s-bound's one sidecar (50+4+1 new, 200 cached)
        assert_eq!(usage.subagents.new_t, 55.0);
        assert_eq!(usage.subagents.cached, 200.0);
        assert_eq!(usage.subagents.total, 255.0);

        // The JSON object close inserts: main + subagents, never re-derived.
        assert_eq!(
            usage.value(),
            json!({
                "sessions": 2,
                "skipped": 1,
                "main": {"new": 335.0, "cached": 1000.0, "total": 1335.0},
                "subagents": {"new": 55.0, "cached": 200.0, "total": 255.0},
                "total": {"new": 390.0, "cached": 1200.0, "total": 1590.0},
            })
        );
    }

    #[test]
    fn usage_line_spells_both_buckets_and_names_the_skipped_sessions() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        usage_fixture(root);
        let usage = collect_close_usage(root, "demo", Some("s-call"));
        assert_eq!(
            close_usage_line(&usage).unwrap(),
            "usage: 2 session(s) — 1.6k tokens (main 1.3k, subagents 255; new 390, cached 1.2k), \
             1 session(s) skipped — no readable transcript"
        );
    }

    #[test]
    fn usage_line_drops_the_skipped_clause_when_every_transcript_was_read() {
        let usage = CloseUsage {
            details: vec![SessionUsage {
                session_id: "s-one".to_string(),
                models: json!({}),
                subagent_models: json!({}),
                subagent_count: 0,
                started_ms: None,
                ended_ms: None,
                totals: UsageBucket::default(),
            }],
            skipped: 0,
            main: UsageBucket { new_t: 500_000.0, cached: 900_000.0, total: 1_400_000.0 },
            subagents: UsageBucket::default(),
        };
        assert_eq!(
            close_usage_line(&usage).unwrap(),
            "usage: 1 session(s) — 1.40M tokens (main 1.40M, subagents 0; new 500.0k, cached 900.0k)"
        );
    }

    /// The honesty rule: a feature whose transcripts are all gone gets NO
    /// line, because "0 tokens" would be a false claim about its cost. The
    /// JSON object is still present, so a reader can tell "nothing read" from
    /// "nothing spent".
    #[test]
    fn no_readable_transcript_emits_no_usage_line() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_usage_session(root, "s-gone", Some("demo"), None, None);
        let usage = collect_close_usage(root, "demo", None);
        assert_eq!(usage.sessions(), 0);
        assert_eq!(usage.skipped, 1);
        assert!(close_usage_line(&usage).is_none());
        assert_eq!(usage.value()["total"]["total"], json!(0.0));

        // A feature with no session records at all: nothing read, nothing
        // skipped, still no line.
        let empty = collect_close_usage(root, "never-existed", None);
        assert_eq!(empty.sessions(), 0);
        assert_eq!(empty.skipped, 0);
        assert!(close_usage_line(&empty).is_none());
    }

    /// A session record that exists but stores no `transcript_path`, and one
    /// whose transcript is present but empty, are both "could not read it" —
    /// counted, never summed as a zero-cost session.
    #[test]
    fn usage_skips_a_record_with_no_transcript_path_and_an_empty_transcript() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, ".bee/sessions/s-nopath.json", r#"{"id":"s-nopath","lane":"demo"}"#);
        write_usage_session(root, "s-empty", Some("demo"), Some(""), None);
        let usage = collect_close_usage(root, "demo", None);
        assert_eq!(usage.sessions(), 0);
        assert_eq!(usage.skipped, 2);
        assert!(close_usage_line(&usage).is_none());
    }

    // ─── tests: the usage RECORD (decision e97cc9d4) ───────────────────────

    /// The three token numbers under `at`, read back as f64.
    ///
    /// The record is written through the repo's JS-shaped pretty printer, so a
    /// whole f64 comes back as `1115`, not `1115.0` — comparing VALUES rather
    /// than `Value`s keeps these assertions about the tokens instead of about
    /// the spelling.
    fn tokens(at: &Value) -> (f64, f64, f64) {
        let n = |key: &str| at[key].as_f64().unwrap_or_else(|| panic!("{key} missing in {at}"));
        (n("new"), n("cached"), n("total"))
    }

    /// The record keeps what the printed line throws away: which session,
    /// which model, how many subagents — and the feature totals beside them.
    #[test]
    fn the_usage_record_keeps_per_session_detail_beside_the_feature_totals() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        usage_fixture(root);
        let usage = collect_close_usage(root, "demo", Some("s-call"));

        let rel = write_usage_record(root, "demo", &usage).unwrap();
        assert_eq!(rel, ".bee/usage/demo.json");
        let ReadJson::Parsed(record) = read_json(&root.join(&rel)) else {
            panic!("the usage record must be readable JSON")
        };

        assert_eq!(record["schema"], json!(USAGE_SCHEMA));
        assert_eq!(record["feature"], json!("demo"));
        // Written at the close, so only its shape is pinned here.
        assert!(record["closed_at"].as_str().is_some_and(|s| s.ends_with('Z')));
        assert_eq!(record["skipped"], json!(1));

        // Candidate order is the sorted session-id order `usage_session_ids`
        // guarantees: s-bound, then s-call.
        let sessions = record["sessions"].as_array().unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0]["session_id"], json!("s-bound"));
        assert_eq!(sessions[0]["subagent_count"], json!(1));
        assert_eq!(sessions[0]["models"]["opus"]["total"].as_f64(), Some(1115.0));
        assert_eq!(sessions[0]["subagent_models"]["sonnet"]["total"].as_f64(), Some(255.0));
        // The session's WHOLE cost: its own 1115 plus its subagent's 255.
        assert_eq!(tokens(&sessions[0]["totals"]), (170.0, 1200.0, 1370.0));
        assert_eq!(sessions[1]["session_id"], json!("s-call"));
        assert_eq!(sessions[1]["subagent_count"], json!(0));
        assert_eq!(tokens(&sessions[1]["totals"]), (220.0, 0.0, 220.0));

        // Same three buckets the printed line names — one computation, two
        // renderings, so the file and the terminal can never disagree.
        assert_eq!(tokens(&record["totals"]["main"]), (335.0, 1000.0, 1335.0));
        assert_eq!(tokens(&record["totals"]["subagents"]), (55.0, 200.0, 255.0));
        assert_eq!(tokens(&record["totals"]["total"]), (390.0, 1200.0, 1590.0));
    }

    /// The honesty rule the record adds to the line's silence: a close that
    /// could read NO transcript still writes the file, with an empty
    /// `sessions` list beside a non-zero `skipped`. "We looked and found
    /// nothing readable" and "nobody ever wrote a record" must not read alike.
    #[test]
    fn a_green_close_writes_the_usage_record_even_with_nothing_readable() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        // Lane-bound, but the transcript it names was never written.
        write_usage_session(root, "s-gone", Some("demo"), None, None);

        let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(result, text, code) = out else { panic!("expected Emit") };
        assert_eq!(code, 0, "this close must be green");
        assert_eq!(result["usage_record"], json!(".bee/usage/demo.json"));
        // No readable transcript, so no printed line — and still a record.
        assert!(!text.contains("usage: "), "{text}");

        let ReadJson::Parsed(record) = read_json(&root.join(".bee/usage/demo.json")) else {
            panic!("a green close must write the usage record")
        };
        assert_eq!(record["schema"], json!(USAGE_SCHEMA));
        assert_eq!(record["sessions"], json!([]));
        // At least s-gone. `close_handler` also names the CALLING session from
        // the ambient `BEE_SESSION_ID`, which the test runner may or may not
        // carry — pinning an exact count here would make this test depend on
        // the environment it runs in, so it pins what the record is FOR: the
        // unreadable ones are counted, never swallowed.
        assert!(record["skipped"].as_u64().unwrap() >= 1, "{record}");
        assert_eq!(tokens(&record["totals"]["total"]), (0.0, 0.0, 0.0));
    }

    /// The whole green path, end to end: the record lands and the SAME line
    /// close printed is stored for the feature-close letter.
    #[test]
    fn a_green_close_records_the_usage_and_hands_the_line_to_the_letter() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_bee_repo(root);
        write_usage_session(
            root,
            "s-bound",
            Some("demo"),
            Some(&usage_event("opus", "r1", 100, 10, 5, 1000)),
            None,
        );

        let out = close_handler(root, "demo", false, None, None, &HashMap::new()).unwrap();
        let Out::Emit(result, text, code) = out else { panic!("expected Emit") };
        assert_eq!(code, 0, "this close must be green");

        let printed = text
            .lines()
            .find(|l| l.starts_with("usage: "))
            .expect("a readable transcript prints a usage line");
        // `starts_with`, not equality: an ambient `BEE_SESSION_ID` in the test
        // runner adds one unreadable candidate and its skipped clause, which
        // says nothing about what this test is for.
        assert!(
            printed.starts_with(
                "usage: 1 session(s) — 1.1k tokens (main 1.1k, subagents 0; new 115, cached 1.0k)"
            ),
            "{printed}"
        );

        let ReadJson::Parsed(record) = read_json(&root.join(".bee/usage/demo.json")) else {
            panic!("a green close must write the usage record")
        };
        assert_eq!(record["sessions"][0]["session_id"], json!("s-bound"));
        assert_eq!(tokens(&record["totals"]["total"]), (115.0, 1000.0, 1115.0));
        assert_eq!(result["usage"]["sessions"], json!(1));

        // The letter's material: the printed line, verbatim, in the note the
        // feature-close stop stores.
        let note = feature_close_note(root, "demo", Some(printed));
        assert_eq!(note.token_usage, vec![printed.to_string()]);
    }
}
