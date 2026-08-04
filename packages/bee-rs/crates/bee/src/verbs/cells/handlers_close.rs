// verb handlers: cap, finish, block, drop, unclaim, reopen, tier
//
// Split out of the single 9.4k-line verbs/cells.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots};
use crate::state as bstate;
use crate::verbs::reservations as rsv;
use crate::verbs::reservations::{Err2, FlagV, Out, R2};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ── cells cap / cells finish ───────────────────────────────────────────────

pub(crate) const CAP_FLAGS: [&str; 8] = [
    "id",
    "outcome",
    "files",
    "deviations-file",
    "friction",
    "override-judge",
    "session-id",
    "force-ownership",
];

/// resolveDeclaredBehaviorChange (E6).
pub(crate) fn resolve_declared_behavior_change(cell: &Map<String, Value>) -> bool {
    match cell.get("behavior_change") {
        Some(Value::Bool(true)) => true,
        Some(Value::Bool(false)) => false,
        _ => {
            let trace = cell.get("trace");
            matches!(trace, Some(t) if js_truthy(t))
                && matches!(
                    trace.and_then(|t| t.get("behavior_change")),
                    Some(Value::Bool(true))
                )
        }
    }
}

/// parseDeviationsFile.
pub(crate) fn parse_deviations_file(file: &str) -> MR<Vec<Value>> {
    let raw = read_file_text(file, "deviations")?;
    match parse_json_js(&raw, false) {
        JsParse::Value(Value::Array(a)) => Ok(a),
        JsParse::Value(other) => Ok(vec![Value::String(jsjson::js_to_string(&other))]),
        // Node's `catch`: not JSON -> one deviation per non-blank line. A
        // lone-surrogate escape now takes that same branch.
        JsParse::NotJson => Ok(raw
            .split('\n')
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .filter(|l| !js_trim(l).is_empty())
            .map(|l| Value::String(l.to_string()))
            .collect()),
    }
}

pub(crate) struct CapFlags {
    pub(crate) id: String,
    pub(crate) outcome: Option<String>,       // flags.outcome ? String : undefined
    pub(crate) friction: Option<String>,      // flags.friction ? String : null
    pub(crate) files_changed: Vec<Value>,
    pub(crate) deviations: Vec<Value>,
    pub(crate) override_reason: String,       // trimmed; '' = none
    pub(crate) session_flag: Option<String>,
    pub(crate) force_ownership: bool,
}

/// capCellFromFlags — the ONE cap door cap and finish share.
pub(crate) fn cap_cell_from_flags(root: &Path, f: &CapFlags, finish: bool) -> MR<Value> {
    let id = &f.id;
    // Pre-scan (see the pre-scan section header).
    prescan_claim(root, id)?;
    let commands = read_commands_slice(root)?;
    if !f.override_reason.is_empty() {
        delegate_only(load_taxonomy(root))?;
    }
    if finish {
        delegate_only(list_path_lease_records(root))?;
        delegate_only(read_holds_store(root))?;
    }
    // Cheap pre-checks BEFORE the (possibly long) test run.
    let existing = read_cell_norm(root, id)?;
    let Some(existing) = existing else {
        return Err(Fail::Thrown(format!("capCell: cell \"{id}\" not found.")));
    };
    let Value::Object(existing_map) = &existing else { return Err(Fail::Delegate) };
    match existing_map.get("status") {
        Some(Value::String(s)) if s == "capped" => {
            return Err(Fail::Thrown(format!("capCell: cell \"{id}\" is already capped.")))
        }
        Some(Value::String(s)) if s == "dropped" => {
            return Err(Fail::Thrown(format!("capCell: cell \"{id}\" was dropped.")))
        }
        _ => {}
    }
    delegate_only(merge_trace(existing_map.get("trace")))?;

    // The one test door (test-simple, decision 412e9b3a).
    let declared = commands
        .test
        .as_ref()
        .map(|list| list.iter().filter(|c| *c != NO_TEST_SENTINEL).cloned().collect::<Vec<_>>())
        .filter(|l| !l.is_empty());
    let tests_run: Option<TestsRun> = match &declared {
        Some(list) => Some(run_declared_tests(root, list)?),
        None => None,
    };
    if let Some(run) = &tests_run {
        if !run.green {
            let excerpt_line = first_failure_line(run);
            // Record the tests-red attempt BEFORE refusing.
            let mut guard = acquire_named_lock(root, &format!("cells:{id}"))?;
            let recorded = (|| -> MR<()> {
                assert_not_archived(root, "recordTestsRedAttempt", id)?;
                let cell = read_cell_norm(root, id)?;
                let Some(cell) = cell else {
                    return Err(Fail::Thrown(format!(
                        "recordTestsRedAttempt: cell \"{id}\" not found."
                    )));
                };
                let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
                let trace = merge_trace(cell_map.get("trace"))?;
                let trace = append_attempt(
                    root,
                    id,
                    trace,
                    "tests-red",
                    excerpt_line.as_deref().map(normalize_failure_signature),
                    excerpt_line.as_deref(),
                )?;
                cell_map.insert("trace".into(), Value::Object(trace));
                write_cell(root, &Value::Object(cell_map))
            })();
            guard.release();
            recorded?;
            let failing: Vec<&CmdRun> = run
                .commands
                .iter()
                .filter(|c| c.failure_excerpt.as_deref().map(|e| !e.is_empty()).unwrap_or(false))
                .collect();
            let mut lines = vec![format!(
                "refusing to cap \"{id}\" — the declared test run is RED ({} of {} command(s) failed; record: {TEST_RESULTS_RELATIVE}).",
                failing.len(),
                run.commands.len()
            )];
            for c in &failing {
                lines.push(format!(
                    "--- {} (exit {}) ---\n{}",
                    c.command,
                    c.exit.map(jsjson::js_f64_to_string).unwrap_or_else(|| "spawn-failed".to_string()),
                    c.failure_excerpt.as_deref().unwrap_or("")
                ));
            }
            lines.push(format!(
                "The red is the work: fix what the failing output names, then re-run bee cells finish --id {id}. Never build on a red base."
            ));
            return Err(Fail::Thrown(lines.join("\n")));
        }
    }

    // capCell (lib/cells.mjs) under the per-cell lock.
    let mut guard = acquire_named_lock(root, &format!("cells:{id}"))?;
    let saved = (|| -> MR<Value> {
        assert_not_archived(root, "capCell", id)?;
        let cell = read_cell_norm(root, id)?;
        let Some(cell) = cell else {
            return Err(Fail::Thrown(format!("capCell: cell \"{id}\" not found.")));
        };
        let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
        let bc = resolve_declared_behavior_change(&cell_map);
        match cell_map.get("status") {
            Some(Value::String(s)) if s == "capped" => {
                return Err(Fail::Thrown(format!("capCell: cell \"{id}\" is already capped.")))
            }
            Some(Value::String(s)) if s == "dropped" => {
                return Err(Fail::Thrown(format!("capCell: cell \"{id}\" was dropped.")))
            }
            _ => {}
        }
        let mut trace = merge_trace(cell_map.get("trace"))?;
        trace = guard_claim_ownership(
            root,
            id,
            trace,
            "capCell",
            f.session_flag.as_deref(),
            f.force_ownership,
        )?;
        let judge_entries: Vec<Value> = match trace.get("semantic_judge") {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        let latest_judge = judge_entries.last().cloned();
        let latest_needs_revision = matches!(
            latest_judge.as_ref().filter(|l| js_truthy(l)).and_then(|l| l.get("verdict")),
            Some(Value::String(s)) if s == "NEEDS_REVISION"
        );
        if latest_needs_revision && f.override_reason.is_empty() {
            return Err(Fail::Thrown(format!(
                "capCell: cell \"{id}\" has a NEEDS_REVISION semantic-judge verdict — rework the cell and record a PASS verdict (bee cells judge-record), or cap with an audited override (bee cells cap --id {id} --override-judge \"<reason>\")."
            )));
        }
        if !f.override_reason.is_empty() {
            let overrides: Vec<Value> = match trace.get("judge_overrides") {
                Some(Value::Array(a)) => a.clone(),
                _ => Vec::new(),
            };
            let mut entry = Map::new();
            entry.insert("overridden_at".into(), Value::String(utc_now()));
            entry.insert("reason".into(), Value::String(f.override_reason.clone()));
            match &latest_judge {
                None => {
                    entry.insert("last_verdict".into(), Value::Null);
                }
                Some(l) => match l.get("verdict") {
                    Some(v) => {
                        entry.insert("last_verdict".into(), v.clone());
                    }
                    None => {} // {last_verdict: undefined} — JSON.stringify drops the key
                },
            }
            let worker_disp = match trace.get("worker") {
                Some(w) if js_truthy(w) => jsjson::js_to_string(w),
                _ => "unknown".to_string(),
            };
            log_decision(
                root,
                &format!(
                    "«cells cap: cell \"{id}\" judge override by {worker_disp} — {}»",
                    f.override_reason
                ),
                "Audited cap over a NEEDS_REVISION (or absent) semantic-judge verdict (D-GHF-C, GH #27.5) — the verdict itself is never rewritten, only a judge_overrides marker appended.",
                &["cells", "judge"],
            )?;
            let mut next = overrides;
            next.push(Value::Object(entry));
            trace.insert("judge_overrides".into(), Value::Array(next));
        }
        let lane = match cell_map.get("lane") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        if lane == "small" || lane == "standard" || lane == "high-risk" {
            if f.files_changed.is_empty() {
                return Err(Fail::Thrown(format!(
                    "capCell: lane \"{lane}\" cell \"{id}\" requires non-empty files_changed (--files a.js,b.js) — record what the worker actually touched. A cell that changed nothing is a drop or a NOOP, not a cap."
                )));
            }
        }
        if lane == "high-risk" && f.outcome.as_deref().map(|o| js_trim(o).is_empty()).unwrap_or(true) {
            return Err(Fail::Thrown(format!(
                "capCell: high-risk cell \"{id}\" requires an outcome summary."
            )));
        }
        cell_map.insert("status".into(), Value::String("capped".into()));
        trace.insert("files_changed".into(), Value::Array(f.files_changed.clone()));
        trace.insert("deviations".into(), Value::Array(f.deviations.clone()));
        trace.insert(
            "friction".into(),
            f.friction.clone().map(Value::String).unwrap_or(Value::Null),
        );
        trace.insert("behavior_change".into(), Value::Bool(bc));
        let outcome_value = match &f.outcome {
            Some(o) if !js_trim(o).is_empty() => Value::String(o.clone()),
            _ => trace.get("outcome").cloned().unwrap_or(Value::Null),
        };
        trace.insert("outcome".into(), outcome_value);
        trace.insert("capped_at".into(), Value::String(utc_now()));
        // No producer since the E1 impact-registry check retired; the key
        // stays so a capped cell's trace shape is unchanged.
        trace.insert("warnings".into(), Value::Array(Vec::new()));
        match &tests_run {
            None => {
                trace.insert("tests".into(), Value::String("undeclared".into()));
            }
            Some(run) => {
                trace.insert("tests".into(), Value::String("green".into()));
                trace.insert("results".into(), Value::String(TEST_RESULTS_RELATIVE.into()));
                trace.insert("ran_at".into(), Value::String(run.ran_at.clone()));
            }
        }
        cell_map.insert("trace".into(), Value::Object(trace));
        let cell_value = Value::Object(cell_map);
        write_cell(root, &cell_value)?;
        Ok(cell_value)
    })();
    guard.release();
    let saved = saved?;
    release_claim_file_best_effort(root, id); // D1 Δ2: cap clears the claim
    Ok(saved)
}

pub(crate) fn cap_flags_from(flags: &rsv::Flags) -> Option<CapFlags> {
    let id = flags.req_str("id")?.to_string();
    let outcome = flags.truthy_str("outcome").map(str::to_string);
    let friction = flags.truthy_str("friction").map(str::to_string);
    let files_changed: Vec<Value> = flags
        .truthy_str("files")
        .map(|s| {
            s.split(',')
                .map(js_trim)
                .filter(|p| !p.is_empty())
                .map(|p| Value::String(p.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let override_reason = match opt_string_flag(flags, "override-judge")? {
        Some(s) => js_trim(&s).to_string(),
        None => String::new(),
    };
    let session_flag = opt_string_flag(flags, "session-id")?;
    let force_ownership = bool_flag(flags, "force-ownership")?;
    Some(CapFlags {
        id,
        outcome,
        friction,
        files_changed,
        deviations: Vec::new(), // filled inside dispatch (file read may throw)
        override_reason,
        session_flag,
        force_ownership,
    })
}

pub(crate) fn cap_text(cell: &Value) -> String {
    let trace = cell.get("trace");
    let capped_at = trace.and_then(|t| t.get("capped_at"));
    let tests = trace.and_then(|t| t.get("tests"));
    format!(
        "Capped {} at {} (tests: {}).",
        js_string_or_undefined(cell.get("id")),
        js_string_or_undefined(capped_at),
        match tests {
            None | Some(Value::Null) => "not run".to_string(), // ?? 'not run'
            Some(v) => jsjson::js_to_string(v),
        }
    )
}

pub(crate) fn run_cap(finish: bool, flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &CAP_FLAGS) {
        return None;
    }
    let cap_flags = cap_flags_from(&flags)?;
    let deviations_file = opt_string_flag(&flags, "deviations-file")?;
    let cmd: &'static str = if finish { "cells finish" } else { "cells cap" };
    let cap_flags_owned = cap_flags;
    dispatch(cmd, use_json, t0, move |ctx| {
        let mut cap_flags = cap_flags_owned;
        let root = ctx.root.clone();
        if let Some(file) = &deviations_file {
            if !file.is_empty() {
                // `flags['deviations-file'] ? parse : []` — truthy only.
                cap_flags.deviations = parse_deviations_file(file)?;
            }
        }
        let cell = cap_cell_from_flags(&root, &cap_flags, finish)?;
        if !finish {
            let text = cap_text(&cell);
            return Ok(Out::Emit(cell, text, 0));
        }
        // cells.finish: release every reservation the claiming agent holds.
        let agent = match cell.get("trace").and_then(|t| t.get("worker")) {
            Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        };
        let cell_id = js_string_or_undefined(cell.get("id"));
        let mut released: Vec<String> = Vec::new();
        let mut release_failure: Option<(String, String)> = None;
        if let Some(agent) = &agent {
            match release_reservations_for_agent(&root, agent, &cell_id) {
                Ok(outcome) => released = outcome.paths,
                Err(Fail::Thrown(message)) => {
                    release_failure = Some((
                        message,
                        format!("bee reservations release --agent {agent} --cell {cell_id} --json"),
                    ));
                }
                Err(Fail::Delegate) => {
                    // Pre-scanned; a mid-command race lands here (header
                    // residual): report it as a release failure, never a
                    // rollback of the already-committed cap.
                    release_failure = Some((
                        "reservation store shape changed mid-command (unrepresentable natively)".to_string(),
                        format!("bee reservations release --agent {agent} --cell {cell_id} --json"),
                    ));
                }
            }
        }
        let Value::Object(cell_map) = &cell else { return Err(Fail::Delegate) };
        let mut result = cell_map.clone();
        result.insert(
            "released".into(),
            Value::Array(released.iter().map(|p| Value::String(p.clone())).collect()),
        );
        if let Some((error, fix)) = &release_failure {
            let mut rf = Map::new();
            rf.insert("error".into(), Value::String(error.clone()));
            rf.insert("fix".into(), Value::String(fix.clone()));
            result.insert("release_failed".into(), Value::Object(rf));
        }
        let mut lines = vec![cap_text(&cell)];
        lines.push(match (&release_failure, released.len()) {
            (Some((error, fix)), _) => {
                format!("Cap stands, but releasing reservations FAILED ({error}) — run: {fix}")
            }
            (None, 0) => "No active reservations to release.".to_string(),
            (None, n) => format!("Released {n} reservation(s): {}.", released.join(", ")),
        });
        lines.push("next: reply [DONE] with the one-line outcome, files touched, and the commit hash.".to_string());
        Ok(Out::Emit(Value::Object(result), lines.join("\n"), 0))
    })
}

// ── block / drop / unclaim / reopen / tier ─────────────────────────────────

/// Shared frame: pre-scan, per-cell lock, read (Delegate on non-object),
/// mutate, write, optional claim clear.
pub(crate) fn mutate_cell(
    root: &Path,
    id: &str,
    verb_not_found: &str,
    archived_verb: Option<&str>,
    clear_claim_after: bool,
    mutate: impl FnOnce(&mut Map<String, Value>) -> MR<()>,
) -> MR<Value> {
    prescan_claim(root, id)?;
    let mut guard = acquire_named_lock(root, &format!("cells:{id}"))?;
    let saved = (|| -> MR<Value> {
        if let Some(verb) = archived_verb {
            assert_not_archived(root, verb, id)?;
        }
        let cell = read_cell_norm(root, id)?;
        let Some(cell) = cell else {
            return Err(Fail::Thrown(format!("{verb_not_found}: cell \"{id}\" not found.")));
        };
        let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
        mutate(&mut cell_map)?;
        let value = Value::Object(cell_map);
        write_cell(root, &value)?;
        Ok(value)
    })();
    guard.release();
    let saved = saved?;
    if clear_claim_after {
        release_claim_file_best_effort(root, id);
    }
    Ok(saved)
}

pub(crate) fn ownership_args(flags: &rsv::Flags) -> Option<(Option<String>, bool)> {
    Some((opt_string_flag(flags, "session-id")?, bool_flag(flags, "force-ownership")?))
}

pub(crate) fn run_block(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "reason", "session-id", "force-ownership"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let reason = flags.req_str("reason")?.to_string();
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells block", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        if js_trim(&reason).is_empty() {
            return Err(Fail::Thrown("blockCell: a reason is required.".into()));
        }
        let root2 = root.clone();
        let id2 = id.clone();
        let reason2 = reason.clone();
        let cell = mutate_cell(&root, &id, "blockCell", Some("blockCell"), true, move |cell_map| {
            let trace = merge_trace(cell_map.get("trace"))?;
            let trace = guard_claim_ownership(
                &root2,
                &id2,
                trace,
                "blockCell",
                session_flag.as_deref(),
                force,
            )?;
            let mut trace = append_attempt(
                &root2,
                &id2,
                trace,
                "blocked",
                Some(normalize_failure_signature(&reason2)),
                Some(&reason2),
            )?;
            cell_map.insert("status".into(), Value::String("blocked".into()));
            trace.insert("blocked_reason".into(), Value::String(reason2.clone()));
            cell_map.insert("trace".into(), Value::Object(trace));
            Ok(())
        })?;
        let text = format!("Blocked {}.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
}

pub(crate) fn run_drop(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "reason"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let reason = flags.req_str("reason")?.to_string();
    dispatch("cells drop", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        if js_trim(&reason).is_empty() {
            return Err(Fail::Thrown("dropCell: a reason is required.".into()));
        }
        let reason2 = reason.clone();
        let cell = mutate_cell(&root, &id, "dropCell", Some("dropCell"), true, move |cell_map| {
            let mut trace = merge_trace(cell_map.get("trace"))?;
            cell_map.insert("status".into(), Value::String("dropped".into()));
            trace.insert("dropped_reason".into(), Value::String(reason2.clone()));
            cell_map.insert("trace".into(), Value::Object(trace));
            Ok(())
        })?;
        let text = format!("Dropped {}.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
}

pub(crate) fn run_unclaim(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "session-id", "force-ownership"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells unclaim", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let cell = unclaim_cell(&root, &id, session_flag.as_deref(), force)?;
        let text = format!("Unclaimed {} — back to open.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
}

/// cells.mjs `unclaimCell(root, id, {sessionId, forceOwnership})`.
///
/// pub(crate) since the `dispatch prepare --claim` port: bee.mjs's
/// claimAndReserveForDispatch calls this as the SECOND rung of its unwind
/// ladder when a reservation conflicts, so the conflict refusal can promise
/// "the claim was unwound and state restored as found".
pub(crate) fn unclaim_cell(
    root: &Path,
    id: &str,
    session_flag: Option<&str>,
    force: bool,
) -> MR<Value> {
    let root = root.to_path_buf();
    let id = id.to_string();
    {
        let root2 = root.clone();
        let id2 = id.clone();
        let session_flag = session_flag.map(str::to_string);
        // unclaimCell has NO assertNotArchived (an archived cell reads as
        // capped/dropped and takes the not-claimed refusal instead).
        let cell = mutate_cell(&root, &id, "unclaimCell", None, true, move |cell_map| {
            let claimed = matches!(cell_map.get("status"), Some(Value::String(s)) if s == "claimed");
            if !claimed {
                return Err(Fail::Thrown(format!(
                    "unclaimCell: cell \"{id2}\" is \"{}\", not \"claimed\" — only a claimed cell can be unclaimed (returned to open). For a capped/blocked/dropped cell use bee cells reopen.",
                    js_string_or_undefined(cell_map.get("status"))
                )));
            }
            let trace = merge_trace(cell_map.get("trace"))?;
            let trace = guard_claim_ownership(
                &root2,
                &id2,
                trace,
                "unclaimCell",
                session_flag.as_deref(),
                force,
            )?;
            cell_map.insert("status".into(), Value::String("open".into()));
            cell_map.insert("trace".into(), Value::Object(release_trace(trace)));
            Ok(())
        })?;
        Ok(cell)
    }
}

pub(crate) fn run_reopen(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "reason", "session-id", "force-ownership"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let reason = flags.req_str("reason")?.to_string();
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells reopen", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        if js_trim(&reason).is_empty() {
            return Err(Fail::Thrown("reopenCell: a reason is required.".into()));
        }
        let root2 = root.clone();
        let id2 = id.clone();
        let reason2 = reason.clone();
        let cell = mutate_cell(&root, &id, "reopenCell", Some("reopenCell"), true, move |cell_map| {
            match cell_map.get("status") {
                Some(Value::String(s)) if s == "open" => {
                    return Err(Fail::Thrown(format!(
                        "reopenCell: cell \"{id2}\" is already \"open\"."
                    )))
                }
                Some(Value::String(s)) if s == "claimed" => {
                    return Err(Fail::Thrown(format!(
                        "reopenCell: cell \"{id2}\" is \"claimed\" — use bee cells unclaim to release the claim back to open."
                    )))
                }
                _ => {}
            }
            let trace = merge_trace(cell_map.get("trace"))?;
            let trace = guard_claim_ownership(
                &root2,
                &id2,
                trace,
                "reopenCell",
                session_flag.as_deref(),
                force,
            )?;
            cell_map.insert("status".into(), Value::String("open".into()));
            let mut trace = release_trace(trace);
            trace.insert("blocked_reason".into(), Value::Null);
            trace.insert("dropped_reason".into(), Value::Null);
            trace.insert("reopened_at".into(), Value::String(utc_now()));
            trace.insert("reopened_reason".into(), Value::String(reason2.clone()));
            cell_map.insert("trace".into(), Value::Object(trace));
            Ok(())
        })?;
        let text = format!("Reopened {} — back to open.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
}

// ── tier's ceiling-share budget (D3, decision 0012) ────────────────────────
//
// "Keep ceiling scarce" (decision 0012) already backs `bee status`'s
// ceiling-scarcity line — status_full/cells.rs's `tier_mix` computes the
// SAME `ceiling / tiered` share this refusal checks. `tier_mix` takes a
// `status_full::Ctx` whose fields are private to that module (and that
// module is out of this cell's file reservation), so the formula is
// re-derived here rather than imported. This is a LIFT, not a second
// implementation — one membership test (MODEL_TIERS' own partition:
// extraction/generation/else-is-ceiling), one formula (ceiling/tiered) —
// following the precedent hooks/session_preamble/store.rs's own
// `ceiling_scarcity_warning` set for the identical "may not edit that file"
// boundary.

/// D3 (counter-teeth) / decision 0012 — no more than 40% of a feature's
/// tiered cells may sit on "ceiling" (the scarce, session-cost model)
/// without a named `--reason` override. Exactly 40% is allowed; strictly
/// over refuses.
pub(crate) const CEILING_SHARE_REFUSAL_MAX: f64 = 0.4;

struct CeilingShare {
    ceiling: i64,
    tiered: i64,
    share: f64,
}

/// The ceiling share of `feature`'s tiered cells (the whole store when
/// `feature` is absent — matching `tier_mix`'s own null-feature behavior)
/// AFTER a hypothetical `new_tier` assignment to `exclude_id`. `exclude_id`
/// is dropped from the scan first so the assigning cell is counted exactly
/// once, under its NEW tier rather than its old one.
fn ceiling_share_after(
    root: &Path,
    feature: Option<&str>,
    exclude_id: &str,
    new_tier: &str,
) -> MR<CeilingShare> {
    let cells = list_cells(root, feature, None).map_err(|_| Fail::Delegate)?;
    let (mut extraction, mut generation, mut ceiling) = (0i64, 0i64, 0i64);
    for cell in &cells {
        if matches!(cell.get("id"), Some(Value::String(cid)) if cid == exclude_id) {
            continue;
        }
        match cell.get("tier").and_then(|t| t.as_str()) {
            Some(t) if MODEL_TIERS.contains(&t) => match t {
                "extraction" => extraction += 1,
                "generation" => generation += 1,
                _ => ceiling += 1,
            },
            _ => {}
        }
    }
    match new_tier {
        "extraction" => extraction += 1,
        "generation" => generation += 1,
        _ => ceiling += 1,
    }
    let tiered = extraction + generation + ceiling;
    let share = if tiered > 0 { ceiling as f64 / tiered as f64 } else { 0.0 };
    Ok(CeilingShare { ceiling, tiered, share })
}

/// setTier's mutator (D3): assigning tier "ceiling" when the post-assignment
/// ceiling share would exceed `CEILING_SHARE_REFUSAL_MAX` refuses, naming
/// the share and the threshold, unless `reason` is a non-blank override — in
/// which case the reason is recorded on the cell's trace (`tier_reason`).
/// Any other tier is never budget-checked. `--reason` given for an
/// already-under-budget "ceiling" assignment is still persisted (harmless
/// metadata; never required there).
pub(crate) fn set_tier(root: &Path, id: &str, tier: &str, reason: Option<&str>) -> MR<Value> {
    let id2 = id.to_string();
    let tier2 = tier.to_string();
    let reason2 = reason.map(str::to_string);
    mutate_cell(root, id, "setTier", Some("setTier"), false, move |cell_map| {
        if tier2 == "ceiling" {
            let feature = match cell_map.get("feature") {
                Some(Value::String(f)) if !f.is_empty() => Some(f.as_str()),
                _ => None,
            };
            let share = ceiling_share_after(root, feature, &id2, &tier2)?;
            let override_reason = reason2.as_deref().map(js_trim).filter(|r| !r.is_empty());
            if share.share > CEILING_SHARE_REFUSAL_MAX && override_reason.is_none() {
                return Err(Fail::Thrown(format!(
                    "setTier: cell \"{id2}\" refused — assigning tier \"ceiling\" would put {}/{} tiered cells on ceiling ({}%), over the {}% ceiling budget (D3, decision 0012). Pass --reason <text> to override; the reason is recorded on the cell's tier record.",
                    share.ceiling,
                    share.tiered,
                    jsjson::js_f64_to_string(rsv::js_round(share.share * 100.0)),
                    jsjson::js_f64_to_string(rsv::js_round(CEILING_SHARE_REFUSAL_MAX * 100.0)),
                )));
            }
            if let Some(r) = override_reason {
                let mut trace = merge_trace(cell_map.get("trace"))?;
                trace.insert("tier_reason".into(), Value::String(r.to_string()));
                cell_map.insert("trace".into(), Value::Object(trace));
            }
        }
        cell_map.insert("tier".into(), Value::String(tier2.clone()));
        Ok(())
    })
}

pub(crate) fn run_tier(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "tier", "reason"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let tier = flags.req_str("tier")?.to_string();
    if !MODEL_TIERS.contains(&tier.as_str()) {
        return None; // validate()'s required-field enum refusal — Node's bytes
    }
    // --reason <text> (D3): overrides the ceiling-share budget refusal
    // below. A flag-alone `--reason` (no value) is unprovable here — same
    // as every other optional value-flag this verb group parses through
    // `opt_string_flag` — so it delegates to Node's validate().
    let reason = opt_string_flag(&flags, "reason")?;
    dispatch("cells tier", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let cell = set_tier(&root, &id, &tier, reason.as_deref())?;
        let text = format!(
            "Cell {} tier set to {}.",
            js_string_or_undefined(cell.get("id")),
            js_string_or_undefined(cell.get("tier"))
        );
        Ok(Out::Emit(cell, text, 0))
    })
}
