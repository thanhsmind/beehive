// routing, `state set`, `state gate` and `state plan-rev bump`
//
// Split out of the single 6.1k-line verbs/state_group.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{
    append_jsonl, ensure_dir, read_json, warn_corrupt_json, write_json_atomic, ReadJson,
};
use crate::jsjson;
use crate::lock::{self, AcquireOnce, LockGuard};
use crate::verbs::reservations::{
    date_parse_val, finish, iso_from_ms, jget, js_disp, js_disp_opt,
    js_numberify, js_trim, keys_known, now_iso, now_ms, parse_flags, prelude, truthy,
    Ctx, Err2, Ex, Exotic, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::reservations::{list_reservations, paths_overlap, rebuild_reservations_projection};
use crate::verbs::workspace_store as ws;
use crate::verbs::workflow_store::{
    acquire_named_lock, acquire_workflow_lock, adopt_mailbox_handoff, create_workflow,
    find_live_workflow, NewWorkflow,
    gates_patch_from_record, lane_lock_name, lane_path, list_lanes, list_workflows,
    newest_open_handoff_mailbox_record, projection_lock_name, read_lane_display, read_lane_strict,
    rebuild_handoff_projection, rebuild_handoff_projection_reporting, rebuild_lane_projection,
    rebuild_lane_projection_reporting, rebuild_state_projection,
    rebuild_state_projection_reporting, update_workflow, update_workflow_assuming_lock,
    update_workflow_assuming_lock_with, wf_id, workflows_list_sort, write_lane,
    write_mailbox_handoff, MailboxAdopt, GATE_ACTOR_VALUES, GATE_BYPASS_LEVELS,
};
use crate::verbs::decisions::{do_log, LogParams};
use crate::verbs::drivers::{has_capture_deferral_decision, js_join, scribing_debt};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
use std::time::Instant;

// ─── routing ───────────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "state" {
        return None;
    }
    let toks: Vec<&str> = args[1..]
        .iter()
        .map(|a| a.to_str())
        .collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None; // main()'s group-scoped help path
    }
    // splitCommandTokens: leading command tokens end at the first "--" token.
    let split = toks.iter().position(|t| t.starts_with("--")).unwrap_or(toks.len());
    let (leading, rest) = toks.split_at(split);
    let (verb, consumed): (&str, usize) = match leading {
        ["set", ..] => ("set", 1),
        ["gate", ..] => ("gate", 1),
        ["waiting-on", "set", ..] => ("waiting-on.set", 2),
        ["waiting-on", "clear", ..] => ("waiting-on.clear", 2),
        ["plan-rev", "bump", ..] => ("plan-rev.bump", 2),
        ["plan-conflicts", "derive", ..] => ("plan-conflicts.derive", 2),
        ["plan-conflicts", "verdict", ..] => ("plan-conflicts.verdict", 2),
        ["worker", "add", ..] => ("worker.add", 2),
        ["worker", "update", ..] => ("worker.update", 2),
        ["worker", "remove", ..] => ("worker.remove", 2),
        ["worker", "clear", ..] => ("worker.clear", 2),
        ["worker", "prune", ..] => ("worker.prune", 2),
        ["scribing-run", ..] => ("scribing-run", 1),
        ["compounding-run", ..] => ("compounding-run", 1),
        ["lanes", ..] => ("lanes", 1),
        ["session", "list", ..] => ("session.list", 2),
        ["session", "bind", ..] => ("session.bind", 2),
        ["session", "unbind", ..] => ("session.unbind", 2),
        ["session", "release", ..] => ("session.release", 2),
        ["handoff", "write", ..] => ("handoff.write", 2),
        ["handoff", "adopt", ..] => ("handoff.adopt", 2),
        ["handoff", "show", ..] => ("handoff.show", 2),
        ["workflows", "list", ..] => ("workflows.list", 2),
        ["workflows", "close", ..] => ("workflows.close", 2),
        ["rebuild-projections", ..] => ("rebuild-projections", 1),
        ["route", ..] => ("route", 1),
        ["start-feature", ..] => ("start-feature", 1),
        ["advisor-ref", "record", ..] => ("advisor-ref.record", 2),
        ["advisor-ref", "show", ..] => ("advisor-ref.show", 2),
        _ => return None, // compact-*/unknown → Node
    };
    if leading.len() != consumed {
        return None; // "Unexpected argument" — Node's own refusal path
    }
    let (flags, use_json) = parse_flags(rest)?;
    match verb {
        "set" => run_set(flags, use_json, t0),
        "gate" => run_gate(flags, use_json, t0),
        "waiting-on.set" => run_waiting_on_set(flags, use_json, t0),
        "waiting-on.clear" => run_waiting_on_clear(flags, use_json, t0),
        "plan-rev.bump" => run_plan_rev_bump(flags, use_json, t0),
        "plan-conflicts.derive" => run_plan_conflicts_derive(flags, use_json, t0),
        "plan-conflicts.verdict" => run_plan_conflicts_verdict(flags, use_json, t0),
        "worker.add" => run_worker_add(flags, use_json, t0),
        "worker.update" => run_worker_update(flags, use_json, t0),
        "worker.remove" => run_worker_remove(flags, use_json, t0),
        "worker.clear" => run_worker_clear(flags, use_json, t0),
        "worker.prune" => run_worker_prune(flags, use_json, t0),
        "scribing-run" => run_scribing_run(flags, use_json, t0),
        "compounding-run" => run_compounding_run(flags, use_json, t0),
        "lanes" => run_lanes(flags, use_json, t0),
        "session.list" => run_session_list(flags, use_json, t0),
        "session.bind" => run_session_bind(flags, use_json, t0),
        "session.unbind" => run_session_unbind(flags, use_json, t0),
        "session.release" => run_session_release(flags, use_json, t0),
        "handoff.write" => run_handoff_write(flags, use_json, t0),
        "handoff.adopt" => run_handoff_adopt(flags, use_json, t0),
        "handoff.show" => run_handoff_show(flags, use_json, t0),
        "workflows.list" => run_workflows_list(flags, use_json, t0),
        "workflows.close" => run_workflows_close(flags, use_json, t0),
        "rebuild-projections" => run_rebuild_projections(flags, use_json, t0),
        "route" => run_route(flags, use_json, t0),
        "start-feature" => run_start_feature(flags, use_json, t0),
        "advisor-ref.record" => run_advisor_ref_record(flags, use_json, t0),
        "advisor-ref.show" => run_advisor_ref_show(flags, use_json, t0),
        _ => None,
    }
}

pub(crate) fn go(cmd: &'static str, use_json: bool, t0: Instant) -> Option<Result<Ctx, ExitCode>> {
    match prelude(cmd, use_json, t0)? {
        Pre::Go(c) => Some(Ok(c)),
        Pre::Emitted(code) => Some(Err(code)),
    }
}

// ─── state set ─────────────────────────────────────────────────────────────

/// The `compounding-complete` scribing-debt door's outcome (chain-integrity
/// D2 / D1-REVISED, docs/decisions/index.md:1554,1569): entering
/// "compounding-complete" is refused while `scribingDebt(feature) > 0`. Two
/// escapes, both loud, either one sufficient: `--waive-scribing-debt` (the
/// caller logs a decision naming the feature and every waived cell) or an
/// already-logged `capture-deferral` decision naming the feature — the SAME
/// escape `bee close` accepts (drivers::has_capture_deferral_decision, reused
/// verbatim so the two doors can never disagree about what counts as debt).
pub(crate) enum ScribingDoor {
    /// No debt stood, or standing debt was already covered by a logged
    /// `capture-deferral` decision — nothing further to log.
    Clear,
    /// Debt stood and was waived via `--waive-scribing-debt` — the caller
    /// logs a decision naming `ids` AFTER the write succeeds.
    Waived { ids: Vec<Value> },
}

/// Reuses drivers::scribing_debt / has_capture_deferral_decision (the exact
/// counter and reader `bee close`'s own D1 door uses) rather than a second,
/// independently-drifting copy.
pub(crate) fn scribing_debt_close_door(
    root: &Path,
    feature: &str,
    waive: bool,
) -> R2<Result<ScribingDoor, String>> {
    let debt = scribing_debt(root, feature)?;
    if debt.count == 0 {
        return Ok(Ok(ScribingDoor::Clear));
    }
    if waive {
        return Ok(Ok(ScribingDoor::Waived { ids: debt.ids }));
    }
    if has_capture_deferral_decision(root, feature)? {
        return Ok(Ok(ScribingDoor::Clear));
    }
    Ok(Err(format!(
        "set: refusing to enter \"compounding-complete\" for feature \"{feature}\" \u{2014} {} capped behavior_change cell(s) have not been synced to their area spec: {}.\n\"compounding-complete\" asserts that scribing already ran for them. It has not.\nFIX: run bee-capturing to merge the settled behavior, then `bee state scribing-run --feature {feature} ...` to stamp it.\nIf the behavior genuinely belongs in no spec, retry with --waive-scribing-debt \u{2014} it is permitted, but it logs a decision naming every cell you waived; the same escape bee close accepts also works here \u{2014} log a decision tagged capture-deferral naming \"{feature}\" first.",
        debt.count,
        js_join(&debt.ids, ", "),
    )))
}

/// scribing-integrity D1 — the second scribing-debt wall: a real --feature
/// swap on the default record while the OUTGOING feature carries unpaid
/// debt is refused the same way a close would be. Reuses
/// drivers::scribing_debt / has_capture_deferral_decision (the exact
/// counter and reader `bee close`'s own D1 door and
/// scribing_debt_close_door above both use) rather than a second,
/// independently-drifting copy. Lanes never reach here: --feature is
/// already refused alongside --lane (and against a session-auto-resolved
/// lane) before this runs.
pub(crate) fn scribing_debt_swap_door(
    root: &Path,
    outgoing_feature: &str,
    waive: bool,
) -> R2<Result<ScribingDoor, String>> {
    let debt = scribing_debt(root, outgoing_feature)?;
    if debt.count == 0 {
        return Ok(Ok(ScribingDoor::Clear));
    }
    if waive {
        return Ok(Ok(ScribingDoor::Waived { ids: debt.ids }));
    }
    if has_capture_deferral_decision(root, outgoing_feature)? {
        return Ok(Ok(ScribingDoor::Clear));
    }
    Ok(Err(format!(
        "set: refusing to swap away from feature \"{outgoing_feature}\" \u{2014} {} capped behavior_change cell(s) have not been synced to their area spec: {}.\nSetting --feature abandons \"{outgoing_feature}\" without ever reaching its close \u{2014} the scribing debt would go silent, with no session left to hit the compounding-complete wall for it.\nFIX: run bee-capturing to merge the settled behavior into its area spec, then `bee state scribing-run --feature {outgoing_feature} ...` to stamp it.\nIf the behavior genuinely belongs in no spec, retry with --waive-scribing-debt \u{2014} it is permitted, but it logs a decision naming every cell you waived; the same escape bee close accepts also works here \u{2014} log a decision tagged capture-deferral naming \"{outgoing_feature}\" first.",
        debt.count,
        js_join(&debt.ids, ", "),
    )))
}

/// D4 — the waiver is loud and attributable: logged AFTER the state/lane
/// write succeeds (so a refused mutation never leaves a decision claiming one
/// happened; decisions.jsonl sits outside the state/lane lock's scope).
/// Wording precedent: decision ee519057 in .bee/decisions.jsonl.
fn log_scribing_debt_waiver(root: &Path, feature: &str, ids: &[Value]) -> R2<()> {
    let decision = format!(
        "Closed feature \"{feature}\" with scribing debt WAIVED for {} capped behavior_change cell(s): {}.",
        ids.len(),
        js_join(ids, ", "),
    );
    let rationale = "Explicitly waived via `state set --phase compounding-complete --waive-scribing-debt`. bee refuses this close by default (chain-integrity D2); the waiver is the sanctioned door, and this record is its price.".to_string();
    match do_log(
        root,
        LogParams {
            decision,
            rationale,
            alternatives: None,
            scope: "repo".to_string(),
            source: "agent".to_string(),
            confidence_raw: None,
            tags: Some(vec!["scribing".to_string(), "state".to_string()]),
            relation: Some("none".to_string()),
            trigger: None,
            feature: None,
        },
        15,
    )? {
        Out::Emit(..) => Ok(()),
        Out::Thrown(_) => Err(Err2::Ex),
    }
}

/// scribing-integrity D1 — the swap-waiver twin of log_scribing_debt_waiver
/// above: same logged-after-write discipline, same tags, its own decision
/// text naming the ABANDONED feature (there is no "closed feature" here —
/// the record just moved on to a different one).
fn log_scribing_debt_swap_waiver(root: &Path, outgoing_feature: &str, ids: &[Value]) -> R2<()> {
    let decision = format!(
        "Swapped away from feature \"{outgoing_feature}\" with scribing debt WAIVED for {} capped behavior_change cell(s): {}.",
        ids.len(),
        js_join(ids, ", "),
    );
    let rationale = "Explicitly waived via `state set --feature <new> --waive-scribing-debt`. bee refuses a feature swap over unpaid debt by default (scribing-integrity D1); the waiver is the sanctioned door, and this record is its price.".to_string();
    match do_log(
        root,
        LogParams {
            decision,
            rationale,
            alternatives: None,
            scope: "repo".to_string(),
            source: "agent".to_string(),
            confidence_raw: None,
            tags: Some(vec!["scribing".to_string(), "state".to_string()]),
            relation: Some("none".to_string()),
            trigger: None,
            feature: None,
        },
        15,
    )? {
        Out::Emit(..) => Ok(()),
        Out::Thrown(_) => Err(Err2::Ex),
    }
}

/// compounding-gate D2 — the freshness-waiver twin of the block above: logged
/// only when `check_phase_transition` actually NEEDED `--waive-compounding`
/// to pass (never on a naturally fresh transition).
fn log_compounding_waiver(root: &Path, feature: &str) -> R2<()> {
    let decision = format!(
        "Closed feature \"{feature}\" with the compounding-run freshness check WAIVED \u{2014} no fresh recorded `state compounding-run` (matching the last scribing run) was found."
    );
    let rationale = "Explicitly waived via `state set --phase compounding-complete --waive-compounding`. bee refuses this close by default (compounding-gate D2); the waiver is the sanctioned door, and this record is its price.".to_string();
    match do_log(
        root,
        LogParams {
            decision,
            rationale,
            alternatives: None,
            scope: "repo".to_string(),
            source: "agent".to_string(),
            confidence_raw: None,
            tags: Some(vec!["scribing".to_string(), "state".to_string()]),
            relation: Some("none".to_string()),
            trigger: None,
            feature: None,
        },
        15,
    )? {
        Out::Emit(..) => Ok(()),
        Out::Thrown(_) => Err(Err2::Ex),
    }
}

pub(crate) fn run_set(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(
        &flags,
        &[
            "phase", "mode", "feature", "next-action", "summary", "owner", "lane", "no-lane",
            "waive-scribing-debt", "waive-compounding",
        ],
    ) {
        return None;
    }
    for b in ["no-lane", "waive-scribing-debt", "waive-compounding"] {
        if !bool_flag_ok(&flags, b) {
            return None;
        }
    }
    let ctx = match go("state set", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = run_set_body(&ctx.root, &flags);
    finish(&ctx, out)
}

/// The mutation body, root-parameterized so it can be exercised directly in
/// tests without a process CWD (`run_set` above is the only caller in
/// production; `prelude`'s CWD-resolved root is threaded through as `root`).
pub(crate) fn run_set_body(root: &Path, flags: &Flags) -> R2<Out> {
    (|| -> R2<Out> {
        let phase_flag = flag_string(flags, "phase");
        if let Some(p) = &phase_flag {
            if !is_known_phase(p) {
                return Ok(Out::Thrown(format!(
                    "set: invalid phase \"{p}\" \u{2014} not in the known-phase enum (isKnownPhase, not the bare PHASES array \u{2014} the terminal alias \"compounding-complete\" must pass). FIX: use one of {KNOWN_PHASES_JOINED}."
                )));
            }
        }
        if ["phase", "mode", "feature", "next-action", "summary"]
            .iter()
            .all(|n| flags.get(n).is_none())
        {
            return Ok(Out::Thrown(
                "set: at least one of --phase, --mode, --feature, --next-action, --summary is required."
                    .to_string(),
            ));
        }
        let (lane_feature, no_lane) = match mutation_lane_selector(flags, "set") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        if lane_feature.is_some() && flags.get("feature").is_some() {
            return Ok(Out::Thrown(
                "set: --feature cannot be combined with --lane \u{2014} a lane's feature is its identity (the lane record's filename), not a mutable field. FIX: omit --feature, or start a new lane instead.".to_string(),
            ));
        }
        let waive = matches!(flags.get("waive-compounding"), Some(FlagV::Present));
        let waive_scribing_debt =
            matches!(flags.get("waive-scribing-debt"), Some(FlagV::Present));

        // withMutationLock's own pre-lock reads: the fail-open scope peek, then
        // the workflow listing that picks the lock names.
        let scope = resolve_mutation_lock_scope(root, lane_feature.as_deref(), no_lane)?;
        let workflows = list_workflows(root)?;

        let locks = acquire_mutation_locks(root, &scope, &workflows)?;
        let mut target =
            resolve_mutation_target(root, lane_feature.as_deref(), "set", no_lane)?;
        // i54-closeout-7 (D7): a session-AUTO-resolved lane refuses --feature too
        // (the flag-level guard above only ever sees an EXPLICIT --lane).
        if flags.get("feature").is_some() {
            if let Some(lane) = target.lane() {
                return Ok(Out::Thrown(format!(
                    "set: --feature cannot target lane \"{lane}\" (auto-resolved from this session's lane binding) \u{2014} a lane's feature is its identity (the lane record's filename), not a mutable field. FIX: omit --feature, or pass --no-lane to address the default record."
                )));
            }
        }
        let mut waived_scribing: Option<Vec<Value>> = None;
        let mut waived_compounding_feature: Option<String> = None;
        let mut waived_swap: Option<(String, Vec<Value>)> = None;
        if let Some(p) = &phase_flag {
            let record = target.record();
            let t = check_phase_transition(record.get("phase"), p, record, waive)?;
            if !t.ok {
                return Ok(Out::Thrown(t.reason));
            }
            let record_feature = match record.get("feature") {
                Some(v) if truthy(v) => Some(js_disp(v)),
                _ => None,
            };
            if t.waived_compounding {
                waived_compounding_feature = record_feature.clone();
            }
            if p == "compounding-complete" {
                // chain-integrity D2 / D1-REVISED — the scribing-debt wall,
                // native now: same counter, same two escapes bee close's own
                // D1 door accepts (scribing_debt_close_door, above).
                if let Some(feature) = &record_feature {
                    match scribing_debt_close_door(root, feature, waive_scribing_debt)? {
                        Ok(ScribingDoor::Clear) => {}
                        Ok(ScribingDoor::Waived { ids }) => {
                            waived_scribing = Some(ids);
                        }
                        Err(msg) => return Ok(Out::Thrown(msg)),
                    }
                }
            }
        }
        // scribing-integrity D1 — the second scribing-debt wall: a real
        // --feature swap on the default record while the CURRENT feature
        // carries unpaid debt has the same effect as closing it silently
        // (the abandoned feature's cells never get their sync, and no
        // session is left to hit the compounding-complete wall for it).
        // Lanes never reach here: --feature is already refused alongside
        // --lane (and against a session-auto-resolved lane) above.
        if target.lane().is_none() {
            if let Some(f) = flag_string(flags, "feature") {
                let current = target.record().get("feature");
                if current.map(truthy).unwrap_or(false)
                    && !opt_strict_eq(current, Some(&Value::String(f.clone())))
                {
                    let outgoing = js_disp(current.unwrap());
                    match scribing_debt_swap_door(root, &outgoing, waive_scribing_debt)? {
                        Ok(ScribingDoor::Clear) => {}
                        Ok(ScribingDoor::Waived { ids }) => {
                            waived_swap = Some((outgoing, ids));
                        }
                        Err(msg) => return Ok(Out::Thrown(msg)),
                    }
                }
            }
        }
        let selected = target.selected_record();
        let lane_note = target.lane_note();
        let phase_known =
            matches!(target.record().get("phase"), Some(Value::String(s)) if is_known_phase(s));
        if !phase_known {
            // `${state.phase ?? ''}` — nullish coalescing.
            let disp = match target.record().get("phase") {
                None | Some(Value::Null) => String::new(),
                Some(v) => js_disp(v),
            };
            return Ok(Out::Thrown(format!(
                "set: refused \u{2014} selected {selected} has missing or invalid pre-mutation phase \"{disp}\". Ownership cannot be derived from a corrupt routing record, so nothing was written. FIX: restore a valid phase before retrying."
            )));
        }
        let phase_str = js_disp(target.record().get("phase").unwrap());
        let owner = match flags.get("owner") {
            Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
            _ => {
                return Ok(Out::Thrown(format!(
                    "set: missing --owner \u{2014} selected {selected}'s pre-mutation phase is \"{phase_str}\". FIX: retry with --owner {phase_str}."
                )));
            }
        };
        if owner != phase_str {
            return Ok(Out::Thrown(format!(
                "set: owner mismatch \u{2014} selected {selected}'s pre-mutation phase is \"{phase_str}\", not \"{owner}\". FIX: retry with --owner {phase_str}."
            )));
        }
        // ssh-1: capture a REAL --feature swap on the default record BEFORE
        // the mutation below overwrites the pre-mutation feature. Same
        // "current truthy AND different" definition the scribing-debt swap
        // door above already uses on this file's own precedent — a lane
        // target never reaches here (refused earlier), and a same-value
        // --feature is not a swap.
        let feature_swap: Option<String> = if target.lane().is_none() {
            flag_string(flags, "feature").filter(|f| match target.record().get("feature") {
                Some(v) if truthy(v) => js_disp(v) != *f,
                _ => false,
            })
        } else {
            None
        };
        let mut changed: Vec<String> = Vec::new();
        {
            let state = target.record_mut();
            if let Some(p) = &phase_flag {
                state.insert("phase".into(), json!(p));
                changed.push(format!("phase={p}"));
            }
            if let Some(m) = flag_string(flags, "mode") {
                state.insert("mode".into(), json!(m));
                changed.push(format!("mode={m}"));
            }
            if let Some(f) = flag_string(flags, "feature") {
                state.insert("feature".into(), json!(f));
                changed.push(format!("feature={f}"));
            }
            if let Some(n) = flag_string(flags, "next-action") {
                state.insert("next_action".into(), json!(n));
                changed.push("next_action".to_string());
            }
            if let Some(s) = flag_string(flags, "summary") {
                state.insert("summary".into(), json!(s));
                changed.push("summary".to_string());
            }
        }
        let record = target.record().clone();
        write_through_projection(root, &target, &record, &[])?;
        drop(locks);
        // ssh-1: ledger.rs's write_through_projection deliberately drops a
        // --feature swap to the direct C1 write (see the comment at its
        // `routable` computation) — the incoming feature's workflow record
        // is never touched by that write, and the OUTGOING feature's live
        // record is left live, orphaned. Reap it here, OUTSIDE the mutation
        // locks just dropped, reusing the exact policy `start-feature` runs
        // (feature.rs's ensure_workflow_record_for_feature then
        // close_workflows_for_feature) — never a second implementation. A
        // same-feature set takes `feature_swap == None` and this block never
        // runs, so that path stays byte-identical.
        if let Some(new_feature) = &feature_swap {
            let phase_for_new = record.get("phase").filter(|v| truthy(v)).map(js_disp);
            let mode_for_new = record.get("mode").filter(|v| !v.is_null()).map(js_disp);
            ensure_workflow_record_for_feature(
                root,
                new_feature,
                phase_for_new.as_deref().unwrap_or("idle"),
                mode_for_new.as_deref(),
                record.get("summary"),
                record.get("next_action"),
                None,
            )?;
            close_workflows_for_feature(root, Some(new_feature))?;
        }
        // D4 — the waiver is loud and attributable: logged AFTER the write
        // succeeds so a refused mutation never leaves a decision claiming one
        // happened (decisions.jsonl sits outside the state/lane lock).
        let mut waiver_note = String::new();
        if let Some(ids) = &waived_scribing {
            // record_feature was required truthy to reach ScribingDoor at
            // all, so the record's (possibly just-mutated) feature is safe
            // here too — read it back off the record we are about to emit.
            if let Some(feature) = record.get("feature").filter(|v| truthy(v)).map(js_disp) {
                log_scribing_debt_waiver(root, &feature, ids)?;
                waiver_note.push_str(&format!(
                    " \u{2014} SCRIBING DEBT WAIVED for {} cell(s): {} (decision logged)",
                    ids.len(),
                    js_join(ids, ", "),
                ));
            }
        }
        if let Some(feature) = &waived_compounding_feature {
            log_compounding_waiver(root, feature)?;
            waiver_note.push_str(&format!(
                " \u{2014} COMPOUNDING-RUN FRESHNESS WAIVED for feature \"{feature}\" (decision logged)"
            ));
        }
        if let Some((outgoing_feature, ids)) = &waived_swap {
            log_scribing_debt_swap_waiver(root, outgoing_feature, ids)?;
            waiver_note.push_str(&format!(
                " \u{2014} SCRIBING DEBT WAIVED for {} cell(s): {} (decision logged)",
                ids.len(),
                js_join(ids, ", "),
            ));
        }
        Ok(Out::Emit(
            Value::Object(record),
            format!("Updated state: {}.{lane_note}{waiver_note}", changed.join(" ")),
            0,
        ))
    })()
}

// ─── state gate ────────────────────────────────────────────────────────────

pub(crate) fn run_gate(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(
        &flags,
        &["name", "merge", "approved", "lane", "no-lane", "owner", "actor", "bypass-level", "reason"],
    ) {
        return None;
    }
    for b in ["merge", "no-lane"] {
        if !bool_flag_ok(&flags, b) {
            return None;
        }
    }
    let ctx = match go("state gate", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = run_gate_body(&ctx.root, &flags);
    finish(&ctx, out)
}

/// requireFreshAdvisorForHighRisk (advisorRefStale, lib/state.mjs, Gate 3
/// advisor precondition — AO3/AO13): high-risk execution never opens without
/// a non-stale advisor_ref. `record` is the SELECTED record (M1 — a lane
/// approval checks the lane's own advisor_ref against the lane's own
/// plan.md, never the default record's); `lane` is `Some(feature)` only when
/// the target IS a lane, so the FIX line's `--lane` tail matches the target
/// exactly. `None` means the record's mode is not high-risk, or the ref is
/// fresh — either way the approval proceeds. `advisor_ref_stale` never
/// throws (a missing plan.md or decisions store reads as an anchor mismatch,
/// never a crash), so this is fail-closed by construction: any read trouble
/// computing staleness reads as stale, never as fresh.
fn high_risk_advisor_refusal(
    root: &Path,
    record: &Map<String, Value>,
    lane: Option<&str>,
) -> Option<String> {
    if !matches!(record.get("mode"), Some(Value::String(s)) if s == "high-risk") {
        return None;
    }
    let staleness = advisor_ref_stale(root, record.get("advisor_ref"), record);
    if !staleness.stale {
        return None;
    }
    let lane_tail = match lane {
        Some(l) => format!(" --lane {l}"),
        None => String::new(),
    };
    Some(format!(
        "gate: execution approval refused for high-risk work \u{2014} the advisor consult is missing or stale (AO3/AO13). \
         Reason(s): {}. \
         FIX: resolve the advisor from config (models.<runtime>.advisor), run it read-only with the evidence bundle on stdin, \
         then record the consult: bee state advisor-ref record --advisor \"<identity>\" --digest-file <path>{lane_tail}. Nothing is written until a non-stale advisor_ref exists.",
        staleness.reasons.join("; "),
    ))
}

// ─── D5's plan-time conflict precondition (knowledge-one-home, koh-9) ───────
//
// The twin of `high_risk_advisor_refusal` above, in the same place and with
// the same fail-closed shape: a merged (or `--name execution`) APPROVAL never
// opens while the plan-time conflict check is missing, stale, or unfinished.
// D5 verbatim: "gate --merge refuses while any candidate lacks a verdict —
// same precondition shape as the high-risk advisor_ref guard".
//
// WHERE THE FIELD IS READ FROM. `conflict_review` lives on the LIVE WORKFLOW
// RECORD beside `plan_rev` (koh-8), and the lane projection never copies it
// down, so this check reads the workflow record directly — never the lane
// record the gate itself mutates. That also puts the review and the
// `plan_rev` it is compared against in ONE read, so the two can never be
// taken from two different revisions of the same lane.
//
// WHY IT IS LANE-ONLY. `conflict_review` is as lane-scoped as `plan_rev`:
// `state plan-conflicts derive` refuses the default (non-lane) record
// outright, so a default-record gate has no review it could ever have
// recorded. The default record's behaviour is therefore unchanged, exactly
// like the advisor precondition's `--no-lane` arm.

/// The lane's live `conflict_review` beside the lane's CURRENT `plan_rev`.
/// `None` means the precondition does not apply at all: the target is not a
/// lane, or the lane names no live workflow record — with no workflow record
/// there is no `plan_rev` to compare against and no record `state
/// plan-conflicts derive` could have written to, the same C1 shape
/// `write_through_projection` and the durable gate stamp already take. The
/// inner `Option<Value>` keeps ABSENT distinct from any recorded value: a
/// missing (or null) field is "never derived", never an empty review.
fn lane_conflict_review(
    root: &Path,
    lane: Option<&str>,
) -> Result<Option<(Option<Value>, Value)>, Err2> {
    let Some(lane) = lane else { return Ok(None) };
    let workflows = list_workflows(root)?;
    let Some(wf) = find_live_workflow(&workflows, lane) else { return Ok(None) };
    // `unwrap_or(json!(0))` is the SAME default `state plan-conflicts derive`
    // stamps into the review, so an un-bumped lane compares equal instead of
    // reading as stale.
    let plan_rev = wf.get("plan_rev").cloned().unwrap_or(json!(0));
    let review = wf.get("conflict_review").cloned().filter(|v| !v.is_null());
    Ok(Some((review, plan_rev)))
}

/// Every candidate id whose verdict is still unrecorded, in derived order. A
/// non-object candidate, or one with no `id`, still counts as unverdicted —
/// a shape this code did not write is never read as "resolved".
fn unverdicted_candidate_ids(review: &Value) -> Vec<String> {
    let Some(Value::Array(items)) = review.get("candidates") else {
        // A review with no candidates ARRAY at all is not a derived review.
        return vec!["<no candidates list on the recorded review>".to_string()];
    };
    let mut out: Vec<String> = Vec::new();
    for c in items {
        match c.get("verdict") {
            Some(Value::String(v)) if CONFLICT_VERDICTS.contains(&v.as_str()) => continue,
            _ => {}
        }
        let id = js_disp_opt(c.get("id"));
        out.push(if id.is_empty() { "<candidate with no id>".to_string() } else { id });
    }
    out
}

/// D5's `conflicts` verdicts, in derived order — the candidates the approval
/// output names so the user approves with eyes open. A recorded `conflicts`
/// verdict NEVER refuses (D5 verbatim: it is a contradiction taken on
/// deliberately); it is surfaced, not blocked.
fn acknowledged_conflict_ids(review: &Value) -> Vec<String> {
    let Some(Value::Array(items)) = review.get("candidates") else { return Vec::new() };
    items
        .iter()
        .filter(|c| matches!(c.get("verdict"), Some(Value::String(v)) if v == "conflicts"))
        .map(|c| js_disp_opt(c.get("id")))
        .filter(|id| !id.is_empty())
        .collect()
}

/// The refusal text for a lane whose plan-time conflict check does not stand
/// behind this approval, or `None` when the approval may proceed. Three
/// causes, all of them fail-closed: no review recorded at all, a review
/// derived against a different `plan_rev`, or any candidate still without a
/// verdict. Errors are NOT swallowed into an allow — a workflow store that
/// cannot be read propagates as `Err2`, never as a silent pass.
fn conflict_review_refusal(root: &Path, lane: Option<&str>) -> Result<Option<String>, Err2> {
    let Some((review, plan_rev)) = lane_conflict_review(root, lane)? else { return Ok(None) };
    let lane = lane.unwrap_or_default();
    let fix = format!(
        "FIX: derive the candidates (bee state plan-conflicts derive --lane {lane}), \
         then record one verdict for each (bee state plan-conflicts verdict --id <candidate> \
         --verdict compatible|conflicts|retires-prior --lane {lane})."
    );
    let Some(review) = review else {
        return Ok(Some(format!(
            "gate: approval refused for lane \"{lane}\" \u{2014} the plan-time conflict check has never run against this plan (D5): the record carries no conflict_review at all. {fix}"
        )));
    };
    let derived_rev = review.get("plan_rev").cloned().unwrap_or(Value::Null);
    if js_disp(&derived_rev) != js_disp(&plan_rev) {
        return Ok(Some(format!(
            "gate: approval refused for lane \"{lane}\" \u{2014} the conflict review was derived against plan_rev {} but the lane is now at plan_rev {} (D5). A plan-rev bump invalidates the review by itself — there is no separate reset to run, only a fresh derive. {fix}",
            js_disp(&derived_rev),
            js_disp(&plan_rev),
        )));
    }
    let missing = unverdicted_candidate_ids(&review);
    if !missing.is_empty() {
        return Ok(Some(format!(
            "gate: approval refused for lane \"{lane}\" \u{2014} {} conflict candidate(s) still carry no verdict (D5): {}. {fix}",
            missing.len(),
            missing.join(", "),
        )));
    }
    Ok(None)
}

/// The `conflicts` verdicts standing behind an approval that just passed
/// `conflict_review_refusal`. Empty for a non-lane target, a lane with no
/// live workflow record, and every lane whose candidates are all
/// `compatible` / `retires-prior`.
fn acknowledged_conflicts(root: &Path, lane: Option<&str>) -> Result<Vec<String>, Err2> {
    let Some((Some(review), _)) = lane_conflict_review(root, lane)? else { return Ok(Vec::new()) };
    Ok(acknowledged_conflict_ids(&review))
}

/// The mutation body, root-parameterized so it can be exercised directly in
/// tests without a process CWD (`run_gate` above is the only caller in
/// production; `prelude`'s CWD-resolved root is threaded through as `root`).
pub(crate) fn run_gate_body(root: &Path, flags: &Flags) -> R2<Out> {
    if flags.get("owner").is_some() {
        return Ok(Out::Thrown(
            "gate: --owner is not accepted \u{2014} routing ownership protects generic `state set` fields only. FIX: omit --owner and use the dedicated gate command.".to_string(),
        ));
    }
    let merge = matches!(flags.get("merge"), Some(FlagV::Present));
    if merge && flags.get("name").is_some() {
        return Ok(Out::Thrown(
            "gate: --merge cannot be combined with --name \u{2014} --merge always addresses BOTH shape and execution in one call. FIX: pass --merge alone, or drop --merge and use --name to approve a single gate.".to_string(),
        ));
    }
    let spec: Vec<(&str, Option<&[&str]>)> = if merge {
        vec![("approved", Some(&["true", "false"][..]))]
    } else {
        vec![
            ("name", Some(&GATE_NAMES[..])),
            ("approved", Some(&["true", "false"][..])),
        ]
    };
    let values = match require_flags(flags, &spec, EXAMPLE_GATE) {
        Ok(v) => v,
        Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
        Err(Err2::Ex) => return Err(Err2::Ex),
    };
    let (name, approved_raw) = if merge {
        (String::new(), values[0].clone())
    } else {
        (values[0].clone(), values[1].clone())
    };
    let approved = approved_raw == "true";
    // D2/D3: the caller-provided trace — who moved the gate, under what
    // bypass level, and why. Validated and refused BEFORE any lock or
    // mutation, so an invalid or under-specified call writes nothing.
    let actor = flag_value(flags, "actor").unwrap_or_else(|| "user".to_string());
    if !GATE_ACTOR_VALUES.contains(&actor.as_str()) {
        return Ok(Out::Thrown(format!(
            "gate: --actor \"{actor}\" is not recognized \u{2014} must be one of {}.",
            GATE_ACTOR_VALUES.join("/")
        )));
    }
    // uat-gate-before-merge D1: the uat gate is never bypass-approved — an
    // auto-actor call naming it is refused outright, before any lock or
    // write, regardless of --bypass-level/--reason.
    if actor == "auto" && name == "uat" {
        return Ok(Out::Thrown(
            "gate: --name uat cannot be approved by --actor auto \u{2014} the uat gate is never bypass-approved (uat-gate-before-merge D1). FIX: get the user's approval and record it with --actor user (the default).".to_string(),
        ));
    }
    let bypass_level = flag_value(flags, "bypass-level").filter(|s| !s.is_empty());
    if let Some(bl) = &bypass_level {
        if !GATE_BYPASS_LEVELS.contains(&bl.as_str()) {
            return Ok(Out::Thrown(format!(
                "gate: --bypass-level \"{bl}\" is not recognized \u{2014} must be one of {}.",
                GATE_BYPASS_LEVELS.join("/")
            )));
        }
    }
    let reason = flag_value(flags, "reason").filter(|s| !s.is_empty());
    if actor == "auto" && (bypass_level.is_none() || reason.is_none()) {
        // D2: "an auto-approval writes the same record with actor: auto plus
        // the bypass level and reason" — an auto call that cannot say why it
        // did not stop is refused outright, so a bypassed run is never LESS
        // traceable than a stopped one.
        return Ok(Out::Thrown(
            "gate: --actor auto requires both --bypass-level and --reason \u{2014} an auto-approval must record the bypass level and the reason it did not stop (D2). FIX: pass --bypass-level <off|normal|full|total> and --reason \"<text>\".".to_string(),
        ));
    }
    let (lane_feature, no_lane) = match mutation_lane_selector(flags, "gate") {
        Ok(v) => v,
        Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
        Err(Err2::Ex) => return Err(Err2::Ex),
    };
    let exec_component = merge || name == "execution";

    let scope = resolve_mutation_lock_scope(root, lane_feature.as_deref(), no_lane)?;
    let workflows = list_workflows(root)?;
    // Gate 3 advisor precondition (AO3/AO13) — decided off a silent peek,
    // before any lock, so a refusal makes zero mutations.
    if exec_component && approved {
        if let Some(peek) = peek_target_record(root, &scope, lane_feature.as_deref())? {
            let lane = match &scope.feature {
                Some(f) if scope.lane => Some(f.as_str()),
                _ => None,
            };
            if let Some(msg) = high_risk_advisor_refusal(root, &peek, lane) {
                return Ok(Out::Thrown(msg));
            }
        }
        // D5's plan-time conflict precondition, decided off the same silent
        // pre-lock read, for the same reason: a refusal makes zero mutations.
        // It reads the workflow record itself, so it needs no peeked lane
        // record — a lane with no lane file at all still gets checked.
        let lane = match &scope.feature {
            Some(f) if scope.lane => Some(f.as_str()),
            _ => None,
        };
        if let Some(msg) = conflict_review_refusal(root, lane)? {
            return Ok(Out::Thrown(msg));
        }
    }
    let locks = acquire_mutation_locks(root, &scope, &workflows)?;
    let mut target = resolve_mutation_target(root, lane_feature.as_deref(), "gate", no_lane)?;
    // D5 (koh-9): the `conflicts` verdicts standing behind this approval,
    // read INSIDE the lock right after the precondition that validated
    // them — surfaced on the output, never a refusal.
    let mut conflicts_acknowledged: Vec<String> = Vec::new();
    if exec_component && approved {
        // race: the peek missed the high-risk mode, or the ref went stale
        // between the peek and the lock — recompute against the locked read.
        if let Some(msg) = high_risk_advisor_refusal(root, target.record(), target.lane()) {
            return Ok(Out::Thrown(msg));
        }
        // Same race, D5's side: a `plan-rev bump` or a fresh `derive` may
        // have landed between the peek and the lock.
        if let Some(msg) = conflict_review_refusal(root, target.lane())? {
            return Ok(Out::Thrown(msg));
        }
        conflicts_acknowledged = acknowledged_conflicts(root, target.lane())?;
    }
    let lane_note = target.lane_note();
    // multisession-native-9 D7 / validation-diet D15 — the plan-rev stamp is
    // LANE-ONLY and reads the live workflow's CURRENT plan_rev.
    let mut stamps: Vec<(String, Value)> = Vec::new();
    if exec_component {
        if let Some(lane) = target.lane() {
            let live = list_workflows(root)?;
            if let Some(wf) = find_live_workflow(&live, lane) {
                let rev = if approved {
                    wf.get("plan_rev").cloned().unwrap_or(Value::Null)
                } else {
                    Value::Null
                };
                if merge {
                    stamps.push(("shape".to_string(), rev.clone()));
                }
                stamps.push(("execution".to_string(), rev));
            }
        }
    }
    {
        let state = target.record_mut();
        // Revocation tracking (AO13) — execution component only.
        if exec_component && !approved {
            let mut revoked = match state.get("gate_revoked_at") {
                Some(Value::Object(m)) => m.clone(),
                None | Some(Value::Null) | Some(Value::Bool(_)) | Some(Value::Number(_)) => {
                    Map::new()
                }
                Some(_) => return Err(Err2::Ex), // string/array spread exotica
            };
            revoked.insert("execution".into(), json!(now_iso()));
            state.insert("gate_revoked_at".into(), Value::Object(revoked));
        }
        let mut gates = match state.get("approved_gates") {
            Some(Value::Object(m)) => m.clone(),
            _ => Map::new(), // both strict readers always merge an object
        };
        if merge {
            gates.insert("shape".into(), json!(approved));
            gates.insert("execution".into(), json!(approved));
        } else {
            gates.insert(name.clone(), json!(approved));
        }
        state.insert("approved_gates".into(), Value::Object(gates));
    }
    let record = target.record().clone();
    write_through_projection(root, &target, &record, &stamps)?;
    // D2/D3: stamp the caller's trace — actor, timestamp, reason, bypass
    // level — onto exactly the gate entry/entries this call touched, on the
    // LIVE WORKFLOW RECORD only. `approved_gates` (legacy state/lane files)
    // never carried actor/reason, so there is nothing to mirror there. The
    // routed feature is computed the SAME way `write_through_projection`
    // computes it (a lane's own name, or the record's own `feature` field —
    // NOT `scope.feature`, which `resolve_mutation_lock_scope` deliberately
    // leaves `None` for a `--no-lane` default-record call): a gate mutation
    // never changes `feature`, so `target.record()`'s value here is the same
    // one that write above just routed by. No live workflow (C1: a repo
    // with zero workflow records) means nothing to stamp — the run behaves
    // exactly as before for that case.
    let routed_feature = match target.lane() {
        Some(l) => Some(l.to_string()),
        None => target.record().get("feature").filter(|v| truthy(v)).map(js_disp),
    };
    // R4 (uat-approval-reaches-the-door): a LANE target with no live
    // workflow is exactly the half-write this cell surfaces — the
    // `approved_gates.<name>` boolean above already landed on the lane
    // record (write_through_projection's own C1 fallback), but there is no
    // live workflow record to also carry the richer stamp onto. The
    // default-record (`--no-lane`) case is NOT this defect: its write lands
    // in `.bee/state.json`, which the merge/close doors already read as
    // their unchanged source-3 fallback, so nothing there is stranded.
    let mut durable_stamp_unreachable = false;
    if let Some(wf) = routed_feature.as_deref().and_then(|f| find_live_workflow(&workflows, f)) {
        let touched: Vec<&str> = if merge { vec!["shape", "execution"] } else { vec![name.as_str()] };
        let at = now_iso();
        let reason_value = reason.clone().map_or(Value::Null, Value::String);
        let bypass_value = bypass_level.clone().map_or(Value::Null, Value::String);
        update_workflow_assuming_lock_with(root, &wf_id(wf), |_current| {
            let mut gates = Map::new();
            for gate_name in &touched {
                let mut entry = Map::new();
                entry.insert("actor".into(), json!(actor));
                entry.insert("at".into(), json!(at));
                entry.insert("reason".into(), reason_value.clone());
                entry.insert("bypass_level".into(), bypass_value.clone());
                gates.insert((*gate_name).to_string(), Value::Object(entry));
            }
            let mut patch = Map::new();
            patch.insert("gates".into(), Value::Object(gates));
            Ok(patch)
        })?;
    } else if target.lane().is_some() {
        durable_stamp_unreachable = true;
    }
    drop(locks);
    // merge-ready-fact D2: `bee gate --name uat` is the one verb that flips
    // the stored `merge_ready.uat` between "pending" and "approved". It
    // writes onto the SAME record the approval above just landed on —
    // `routed_feature`, `write_through_projection`'s own routing, never
    // `scope.feature` — so the fact and the gate can never disagree about
    // whose uat this was.
    //
    // Placed after `drop(locks)` deliberately: `set_uat` goes through the
    // ledger mutation seam and takes the very locks held just above, so
    // calling it any earlier would find them busy and fail-open into
    // silence. It is still strictly after the `approved_gates` write, which
    // is all the ordering the fact needs.
    //
    // FAIL-OPEN and result-neutral: it never CREATES the fact (a feature
    // that is not merge-ready has no `uat` to flip), never throws, and its
    // answer is dropped — this verb's text, JSON, and exit code are
    // byte-identical either way. D3: the uat gate itself never READS the
    // fact; this is a write-only projection.
    if !merge && name == "uat" {
        if let Some(feature) = routed_feature.as_deref() {
            let _ = crate::verbs::workflow_store::merge_ready::set_uat(root, feature, approved);
        }
    }
    let text = if merge {
        format!("Gates \"shape\" and \"execution\" set to {approved}.{lane_note}")
    } else {
        format!("Gate \"{name}\" set to {approved}.{lane_note}")
    };
    // R4: the success line above stays byte-identical — this is a SECOND
    // line, added beside it, never a replacement, and never a refusal (the
    // lane write is real; the command still exits 0). It names, in the
    // operator's own terms, exactly what the silent skip used to hide: the
    // approval landed on the lane record because the feature's workflow
    // record is closed or does not exist, so the durable (workflow-record)
    // copy could not be updated. The same fact rides the JSON result as a
    // typed field (`gate_durable_stamp_skipped`) for a non-interactive
    // caller, present only when true.
    let (text, record) = if durable_stamp_unreachable {
        let uat_tail = if name == "uat" {
            " The uat door reads the lane record as a second source, so the approval still reaches it."
        } else {
            ""
        };
        let text = format!(
            "{text}\nNote: the feature's workflow record is closed or does not exist, so the durable stamp could not be written there \u{2014} the approval is recorded on the lane record instead.{uat_tail}"
        );
        let mut record = record;
        record.insert("gate_durable_stamp_skipped".into(), Value::Bool(true));
        (text, record)
    } else {
        (text, record)
    };
    // D5 (koh-9): a recorded `conflicts` verdict never refuses — it is a
    // contradiction the plan takes on deliberately. The approval therefore
    // NAMES it, on a second line beside the byte-identical success line and
    // as a typed `conflicts_acknowledged` field on the JSON result (present
    // only when non-empty), so the user approves with eyes open instead of
    // discovering the contradiction later.
    let (text, record) = if conflicts_acknowledged.is_empty() {
        (text, record)
    } else {
        let text = format!(
            "{text}\nNote: approved over {} recorded \"conflicts\" verdict(s) \u{2014} {}. D5 lets a conflict through with eyes open: the contradiction is recorded, not resolved.",
            conflicts_acknowledged.len(),
            conflicts_acknowledged.join(", "),
        );
        let mut record = record;
        record.insert("conflicts_acknowledged".into(), json!(conflicts_acknowledged));
        (text, record)
    };
    Ok(Out::Emit(Value::Object(record), text, 0))
}

// ─── state plan-rev bump ───────────────────────────────────────────────────

pub(crate) fn run_plan_rev_bump(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["lane", "no-lane"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "no-lane") {
        return None;
    }
    let ctx = match go("state plan-rev bump", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let (lane_feature, no_lane) = match mutation_lane_selector(&flags, "plan-rev bump") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        // Read-only PEEK outside both locks (splr-1's canonical order:
        // workflow:<id> FIRST, then the projection lock — never the reverse).
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
        let target =
            resolve_mutation_target(&ctx.root, lane_feature.as_deref(), "plan-rev bump", no_lane)?;
        let Some(lane) = target.lane() else {
            return Ok(Out::Thrown(
                "plan-rev bump: refused \u{2014} resolution landed on the default (non-lane) record. plan_rev bumping is scoped to lanes only, by design (nothing else ever reads or bumps the default pipeline's plan_rev, so stamping it would be meaningless). FIX: target a lane explicitly with --lane <feature>, or bind the calling session to one first (\"state session bind --session-id <id> --lane <feature>\").".to_string(),
            ));
        };
        let live = list_workflows(&ctx.root)?;
        let Some(wf) = find_live_workflow(&live, lane) else {
            return Ok(Out::Thrown(format!(
                "plan-rev bump: no live workflow record found for lane \"{lane}\" \u{2014} nothing to bump. FIX: start the lane first (\"state start-feature --feature {lane} --as-lane\")."
            )));
        };
        let id = wf_id(wf);
        // The peek picked which workflow lock is held right now; a different
        // resolution means that lock protects the wrong record.
        if peeked_id.as_deref() != Some(id.as_str()) {
            return Ok(Out::Thrown(format!(
                "plan-rev bump: the target lane's workflow changed while this call was starting (expected \"{}\", resolved \"{id}\"), so the workflow lock this call holds does not protect it. Nothing was written. FIX: re-run the bump.",
                peeked_id.as_deref().unwrap_or("none")
            )));
        }
        let updated = update_workflow_assuming_lock_with(&ctx.root, &id, |current| {
            // `(current.plan_rev || 0) + 1` — a non-numeric plan_rev would take
            // JS's own coercion path (string concat), which this port delegates.
            let base = match current.get("plan_rev") {
                Some(Value::Number(n)) => n.as_f64().ok_or(Err2::Ex)?,
                None | Some(Value::Null) | Some(Value::Bool(false)) => 0.0,
                Some(Value::String(s)) if s.is_empty() => 0.0,
                Some(_) => return Err(Err2::Ex),
            };
            let mut patch = Map::new();
            patch.insert(
                "plan_rev".into(),
                Value::Number(serde_json::Number::from_f64(base + 1.0).ok_or(Err2::Ex)?),
            );
            Ok(patch)
        })?;
        let rebuilt = rebuild_lane_projection(&ctx.root, lane)?;
        let plan_rev = updated.get("plan_rev").cloned().unwrap_or(Value::Null);
        let mut result = Map::new();
        result.insert("feature".into(), json!(lane));
        result.insert("plan_rev".into(), plan_rev.clone());
        result.insert(
            "lane".into(),
            rebuilt.map(Value::Object).unwrap_or(Value::Null),
        );
        let text = format!(
            "Bumped plan_rev to {} for lane \"{lane}\" (workflow); lane projection rebuilt.",
            js_disp(&plan_rev)
        );
        Ok(Out::Emit(Value::Object(result), text, 0))
    })();
    finish(&ctx, out)
}

// ─── tests: the compounding-complete door (tpp-1) ──────────────────────────
//
// The Node delegate markers this cell replaces (`return Err(Err2::Ex)` at
// what used to be set_gate.rs:179/213) never had a native test — there was
// nothing native to test. These are red-first for the native door: written
// against `scribing_debt_close_door` and `run_set_body` before either
// existed, confirmed red (compile failure — the names did not exist — then a
// failing assertion once stubbed), now green against the real
// implementation above.

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn w(root: &Path, rel: &str, body: &str) {
        let file = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, body).unwrap();
    }

    fn read_json_file(root: &Path, rel: &str) -> Value {
        let text = std::fs::read_to_string(root.join(rel)).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    fn flags(args: &[&str]) -> Flags {
        parse_flags(args).expect("well-formed fixture argv").0
    }

    // ── scribing_debt_close_door — the pure door decision (must-haves 2-4) ──

    #[test]
    fn debt_door_clears_when_no_debt_stands() {
        let tmp = tmp_root();
        let root = tmp.path();
        assert!(matches!(
            scribing_debt_close_door(root, "demo", false).unwrap(),
            Ok(ScribingDoor::Clear)
        ));
    }

    #[test]
    fn debt_door_refuses_and_names_every_unscribed_cell() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        w(root, ".bee/cells/demo-5.json", r#"{"id":"demo-5","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-02T00:00:00.000Z"}}"#);
        let Err(msg) = scribing_debt_close_door(root, "demo", false).unwrap() else {
            panic!("expected a refusal naming the debt")
        };
        assert!(msg.contains("demo-4"), "{msg}");
        assert!(msg.contains("demo-5"), "{msg}");
        assert!(msg.contains("2 capped behavior_change cell"), "{msg}");
        assert!(msg.contains("--waive-scribing-debt"), "{msg}");
        assert!(msg.contains("capture-deferral"), "{msg}");
    }

    #[test]
    fn debt_door_waive_flag_passes_and_reports_every_cell() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        match scribing_debt_close_door(root, "demo", true).unwrap() {
            Ok(ScribingDoor::Waived { ids }) => assert_eq!(ids, vec![json!("demo-4")]),
            other => panic!("expected a waived outcome, got a refusal or a clear: {}", other.is_err()),
        }
    }

    #[test]
    fn debt_door_logged_capture_deferral_decision_passes_without_the_flag() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-07-02T12:00:00.000Z\",\"decision\":\"defer capture for demo until next sprint\",\"rationale\":\"r\",\"tags\":[\"capture-deferral\"],\"scope\":\"repo\"}\n",
        );
        assert!(matches!(
            scribing_debt_close_door(root, "demo", false).unwrap(),
            Ok(ScribingDoor::Clear)
        ));
    }

    // ── run_set_body — the door wired into the real verb (must-haves 1,5,6) ─

    fn fresh_default_state(feature: &str) -> String {
        format!(
            r#"{{"phase":"compounding","feature":"{feature}","last_scribing_run":{{"feature":"{feature}","at":"2026-07-01T00:00:00.000Z"}},"last_compounding_run":{{"feature":"{feature}","at":"2026-07-01T00:00:01.000Z"}}}}"#
        )
    }

    fn fresh_lane_record(feature: &str) -> String {
        format!(
            r#"{{"feature":"{feature}","phase":"compounding","last_scribing_run":{{"feature":"{feature}","at":"2026-07-01T00:00:00.000Z"}},"last_compounding_run":{{"feature":"{feature}","at":"2026-07-01T00:00:01.000Z"}}}}"#
        )
    }

    /// Must-have 1: a clean compounding record reaches compounding-complete
    /// and the phase is actually written to disk — not just claimed.
    #[test]
    fn a_clean_compounding_record_reaches_compounding_complete_on_disk() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/state.json", &fresh_default_state("demo"));
        let out = run_set_body(
            root,
            &flags(&["--no-lane", "--phase", "compounding-complete", "--owner", "compounding"]),
        )
        .unwrap();
        assert!(matches!(out, Out::Emit(..)), "expected a clean write");
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["phase"], json!("compounding-complete"));
    }

    /// Must-have 6: the lane branch reaches the terminal phase the same way
    /// the default record does — same door, same write-through.
    #[test]
    fn the_lane_branch_reaches_compounding_complete_the_same_way() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/lanes/demo.json", &fresh_lane_record("demo"));
        let out = run_set_body(
            root,
            &flags(&["--lane", "demo", "--phase", "compounding-complete", "--owner", "compounding"]),
        )
        .unwrap();
        assert!(matches!(out, Out::Emit(..)), "expected a clean write");
        let lane = read_json_file(root, ".bee/lanes/demo.json");
        assert_eq!(lane["phase"], json!("compounding-complete"));
    }

    /// Must-have 2, wired into the real verb: debt > 0 refuses the whole
    /// mutation (nothing reaches disk) and names every unscribed cell.
    #[test]
    fn debt_over_zero_refuses_the_full_verb_and_never_writes() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/state.json", &fresh_default_state("demo"));
        w(root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:02.000Z"}}"#);
        let out = run_set_body(
            root,
            &flags(&["--no-lane", "--phase", "compounding-complete", "--owner", "compounding"]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else { panic!("expected a refusal, got a write") };
        assert!(msg.contains("demo-4"), "{msg}");
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(
            state["phase"],
            json!("compounding"),
            "the refused phase must never reach disk"
        );
    }

    /// Must-have 3: `--waive-scribing-debt` passes and logs a decision naming
    /// the cells.
    #[test]
    fn waive_scribing_debt_passes_and_logs_a_decision_naming_the_cells() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/state.json", &fresh_default_state("demo"));
        w(root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:02.000Z"}}"#);
        let out = run_set_body(
            root,
            &flags(&[
                "--no-lane",
                "--phase",
                "compounding-complete",
                "--owner",
                "compounding",
                "--waive-scribing-debt",
            ]),
        )
        .unwrap();
        let Out::Emit(_, text, _) = out else { panic!("expected the waiver to pass") };
        assert!(text.contains("SCRIBING DEBT WAIVED"), "{text}");
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["phase"], json!("compounding-complete"));
        let decisions = std::fs::read_to_string(root.join(".bee/decisions.jsonl")).unwrap();
        assert!(decisions.contains("scribing debt WAIVED"), "{decisions}");
        assert!(decisions.contains("demo-4"), "{decisions}");
    }

    /// Must-have 4: an already-logged capture-deferral decision passes
    /// without the flag, and does not append a redundant waiver decision.
    #[test]
    fn a_logged_capture_deferral_decision_passes_without_the_flag() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/state.json", &fresh_default_state("demo"));
        w(root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:02.000Z"}}"#);
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-07-02T12:00:00.000Z\",\"decision\":\"defer capture for demo until next sprint\",\"rationale\":\"r\",\"tags\":[\"capture-deferral\"],\"scope\":\"repo\"}\n",
        );
        let out = run_set_body(
            root,
            &flags(&["--no-lane", "--phase", "compounding-complete", "--owner", "compounding"]),
        )
        .unwrap();
        assert!(
            matches!(out, Out::Emit(..)),
            "the deferral decision must lift the wall without the flag"
        );
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["phase"], json!("compounding-complete"));
        let decisions = std::fs::read_to_string(root.join(".bee/decisions.jsonl")).unwrap();
        assert_eq!(decisions.lines().count(), 1, "no redundant waiver decision appended");
    }

    /// Must-have 5: the freshness half still refuses on its own, and
    /// `--waive-compounding` still lifts it (and logs its own decision).
    #[test]
    fn the_freshness_half_still_refuses_and_waive_compounding_still_lifts_it() {
        let tmp = tmp_root();
        let root = tmp.path();
        // Stale: last_compounding_run predates last_scribing_run.
        w(
            root,
            ".bee/state.json",
            r#"{"phase":"compounding","feature":"demo","last_scribing_run":{"feature":"demo","at":"2026-07-02T00:00:00.000Z"},"last_compounding_run":{"feature":"demo","at":"2026-07-01T00:00:00.000Z"}}"#,
        );
        let refused = run_set_body(
            root,
            &flags(&["--no-lane", "--phase", "compounding-complete", "--owner", "compounding"]),
        )
        .unwrap();
        let Out::Thrown(msg) = refused else {
            panic!("expected the freshness door to refuse")
        };
        assert!(msg.contains("no fresh compounding run recorded"), "{msg}");
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["phase"], json!("compounding"));

        let waived = run_set_body(
            root,
            &flags(&[
                "--no-lane",
                "--phase",
                "compounding-complete",
                "--owner",
                "compounding",
                "--waive-compounding",
            ]),
        )
        .unwrap();
        let Out::Emit(_, text, _) = waived else {
            panic!("expected --waive-compounding to lift the door")
        };
        assert!(text.contains("COMPOUNDING-RUN FRESHNESS WAIVED"), "{text}");
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["phase"], json!("compounding-complete"));
        let decisions = std::fs::read_to_string(root.join(".bee/decisions.jsonl")).unwrap();
        assert!(decisions.contains("compounding-run freshness check WAIVED"), "{decisions}");
    }

    // ── run_gate_body — the high-risk execution-gate refusal (gdr-1) ───────
    //
    // Before gdr-1 both guard arms `return Err(Err2::Ex)`, which `finish`
    // turns into a bare `None` (emit.rs) so the dispatcher falls through to
    // its generic argument-shape classifier and prints
    // "bee: unsupported argument shape" (router.rs, kind
    // "unsupported_argument_shape") — blaming the caller's flags for a
    // lane-driven refusal. There was no existing test for `run_gate`/
    // `run_gate_body` at all (grep confirms `run_gate` only appears at its
    // own definition and its `try_native` dispatch line), so this is new
    // coverage, not a duplicate of anything above.

    /// A high-risk lane's execution-gate approval still refuses (the verdict
    /// never changes), but now via `Out::Thrown` with a message naming the
    /// real cause and the lane — never the generic Err2::Ex path that would
    /// print "unsupported argument shape".
    #[test]
    fn high_risk_execution_gate_approval_refuses_with_the_advisor_cause_and_names_the_lane() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(
            root,
            ".bee/lanes/gate-door-refusal.json",
            r#"{"feature":"gate-door-refusal","phase":"planning","mode":"high-risk"}"#,
        );
        let out = run_gate_body(
            root,
            &flags(&["--lane", "gate-door-refusal", "--merge", "--approved", "true"]),
        );
        let Ok(Out::Thrown(msg)) = out else {
            panic!("expected Ok(Out::Thrown(_)) naming the cause, got an emit or Err(Err2::Ex)");
        };
        assert!(msg.contains("gate-door-refusal"), "{msg}");
        assert!(msg.contains("high-risk"), "{msg}");
        assert!(msg.contains("advisor"), "{msg}");
        assert!(msg.contains("bee state advisor-ref record"), "{msg}");
        assert!(
            !msg.contains("unsupported argument shape"),
            "must not read like the generic dispatcher refusal: {msg}"
        );
        // Nothing was written on the refusal path.
        let lane = read_json_file(root, ".bee/lanes/gate-door-refusal.json");
        assert!(lane.get("approved_gates").is_none(), "{lane:?}");
    }

    // ── run_gate_body — the real Gate 3 precondition (agp-2) ───────────────
    //
    // gdr-1 (above) made the high-risk refusal honest about ITS cause; every
    // approval refused unconditionally because `advisor_ref_stale` did not
    // exist yet. agp-1 ported that check (advisor_ref.rs); this cell wires it
    // in, so the refusal above now only fires while the ref is genuinely
    // missing or stale — a fresh one lets the approval through.

    /// A fresh `advisor_ref` builds the same anchors `advisor_ref_anchors`
    /// (and `bee state advisor-ref record`) would stamp, so a fixture written
    /// straight to disk reads exactly as fresh as one recorded through the
    /// CLI — `advisor_ref.rs`'s own `fresh_ref` test helper follows the same
    /// pattern for the same reason.
    fn fresh_advisor_ref(root: &Path, feature: &str) -> Value {
        let anchors = advisor_ref_anchors(root, &json!(feature));
        json!({
            "consulted_at": "2026-01-01T00:00:00.000Z",
            "feature": anchors["feature"],
            "newest_decision_id": anchors["newest_decision_id"],
            "plan_sha256": anchors["plan_sha256"],
            "advisor": "gpt-5.6-sol",
            "digest_head": "digest",
        })
    }

    /// The whole point of the cell: a high-risk lane's merged (shape +
    /// execution) approval, refused unconditionally before this cell, now
    /// SUCCEEDS once a fresh advisor_ref is on the record.
    #[test]
    fn high_risk_merged_gate_approval_succeeds_after_a_fresh_advisor_ref_is_recorded() {
        let tmp = tmp_root();
        let root = tmp.path();
        let advisor_ref = fresh_advisor_ref(root, "gate-door-open");
        w(
            root,
            ".bee/lanes/gate-door-open.json",
            &format!(
                r#"{{"feature":"gate-door-open","phase":"planning","mode":"high-risk","advisor_ref":{advisor_ref}}}"#
            ),
        );
        let out = run_gate_body(
            root,
            &flags(&["--lane", "gate-door-open", "--merge", "--approved", "true"]),
        )
        .unwrap();
        let Out::Emit(_, text, _) = out else {
            panic!("expected the approval to succeed with a fresh ref, got a refusal")
        };
        assert!(text.contains("Gates \"shape\" and \"execution\" set to true"), "{text}");
        let lane = read_json_file(root, ".bee/lanes/gate-door-open.json");
        assert_eq!(lane["approved_gates"]["shape"], json!(true));
        assert_eq!(lane["approved_gates"]["execution"], json!(true));
    }

    /// Same door, the default (non-lane) record — the peek and the
    /// post-lock arm both read `.bee/state.json`, never a lane file, and the
    /// FIX line carries no `--lane` tail since the target is not one.
    #[test]
    fn high_risk_execution_gate_approval_on_the_default_record_succeeds_after_a_fresh_ref() {
        let tmp = tmp_root();
        let root = tmp.path();
        let advisor_ref = fresh_advisor_ref(root, "demo");
        w(
            root,
            ".bee/state.json",
            &format!(
                r#"{{"phase":"planning","feature":"demo","mode":"high-risk","advisor_ref":{advisor_ref}}}"#
            ),
        );
        let out = run_gate_body(root, &flags(&["--no-lane", "--merge", "--approved", "true"]))
            .unwrap();
        let Out::Emit(..) = out else { panic!("expected the approval to succeed") };
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["approved_gates"]["execution"], json!(true));
    }

    /// Refusal cause 1: no advisor_ref recorded at all — default record this
    /// time, so the FIX line must NOT suggest a `--lane` the caller never
    /// asked for.
    #[test]
    fn high_risk_execution_gate_approval_on_the_default_record_refuses_without_a_ref() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/state.json", r#"{"phase":"planning","feature":"demo","mode":"high-risk"}"#);
        let out = run_gate_body(root, &flags(&["--no-lane", "--merge", "--approved", "true"]))
            .unwrap();
        let Out::Thrown(msg) = out else { panic!("expected a refusal, got Out::Emit") };
        assert!(msg.contains("no advisor_ref recorded"), "{msg}");
        assert!(!msg.contains("--lane"), "default record refusal must not suggest --lane: {msg}");
        let state = read_json_file(root, ".bee/state.json");
        assert!(state.get("approved_gates").is_none(), "{state:?}");
    }

    /// Refusal cause 2: the feature moved since the consult.
    #[test]
    fn high_risk_gate_approval_refuses_when_the_feature_changed_since_the_consult() {
        let tmp = tmp_root();
        let root = tmp.path();
        // The ref was recorded for a DIFFERENT feature than this lane's own.
        let advisor_ref = fresh_advisor_ref(root, "some-other-feature");
        w(
            root,
            ".bee/lanes/gate-door-feature.json",
            &format!(
                r#"{{"feature":"gate-door-feature","phase":"planning","mode":"high-risk","advisor_ref":{advisor_ref}}}"#
            ),
        );
        let out = run_gate_body(
            root,
            &flags(&["--lane", "gate-door-feature", "--merge", "--approved", "true"]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else { panic!("expected a refusal, got Out::Emit") };
        assert!(msg.contains("feature changed since the consult"), "{msg}");
    }

    /// Refusal cause 3: a new decision was logged since the consult.
    #[test]
    fn high_risk_gate_approval_refuses_when_a_new_decision_was_logged_since_the_consult() {
        let tmp = tmp_root();
        let root = tmp.path();
        let advisor_ref = fresh_advisor_ref(root, "gate-door-decision");
        w(
            root,
            ".bee/lanes/gate-door-decision.json",
            &format!(
                r#"{{"feature":"gate-door-decision","phase":"planning","mode":"high-risk","advisor_ref":{advisor_ref}}}"#
            ),
        );
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-01-02T00:00:00.000Z\",\"decision\":\"x\",\"rationale\":\"y\"}\n",
        );
        let out = run_gate_body(
            root,
            &flags(&["--lane", "gate-door-decision", "--merge", "--approved", "true"]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else { panic!("expected a refusal, got Out::Emit") };
        assert!(msg.contains("a new decision was logged since the consult"), "{msg}");
    }

    /// Refusal cause 4: plan.md changed since the consult.
    #[test]
    fn high_risk_gate_approval_refuses_when_the_plan_changed_since_the_consult() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, "docs/history/gate-door-plan/plan.md", "v1");
        let advisor_ref = fresh_advisor_ref(root, "gate-door-plan");
        w(
            root,
            ".bee/lanes/gate-door-plan.json",
            &format!(
                r#"{{"feature":"gate-door-plan","phase":"planning","mode":"high-risk","advisor_ref":{advisor_ref}}}"#
            ),
        );
        // Mutate the plan AFTER the ref captured its hash.
        w(root, "docs/history/gate-door-plan/plan.md", "v2");
        let out = run_gate_body(
            root,
            &flags(&["--lane", "gate-door-plan", "--merge", "--approved", "true"]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else { panic!("expected a refusal, got Out::Emit") };
        assert!(msg.contains("plan.md changed since the consult"), "{msg}");
    }

    /// Refusal cause 5: the consult predates the most recent execution-gate
    /// revocation.
    #[test]
    fn high_risk_gate_approval_refuses_when_the_ref_predates_a_revocation() {
        let tmp = tmp_root();
        let root = tmp.path();
        let advisor_ref = fresh_advisor_ref(root, "gate-door-revoked");
        w(
            root,
            ".bee/lanes/gate-door-revoked.json",
            &format!(
                r#"{{"feature":"gate-door-revoked","phase":"planning","mode":"high-risk","advisor_ref":{advisor_ref},"gate_revoked_at":{{"execution":"2026-01-02T00:00:00.000Z"}}}}"#
            ),
        );
        let out = run_gate_body(
            root,
            &flags(&["--lane", "gate-door-revoked", "--merge", "--approved", "true"]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else { panic!("expected a refusal, got Out::Emit") };
        assert!(msg.contains("predates the most recent execution-gate revocation"), "{msg}");
    }

    /// The unapprove path (`--approved false`) is never gated by the advisor
    /// precondition, even with no advisor_ref at all on a high-risk record —
    /// `exec_component && approved` is false, so the check never runs.
    #[test]
    fn high_risk_gate_unapprove_is_unaffected_by_the_advisor_precondition() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(
            root,
            ".bee/lanes/gate-door-unapprove.json",
            r#"{"feature":"gate-door-unapprove","phase":"planning","mode":"high-risk","approved_gates":{"shape":true,"execution":true}}"#,
        );
        let out = run_gate_body(
            root,
            &flags(&["--lane", "gate-door-unapprove", "--merge", "--approved", "false"]),
        )
        .unwrap();
        let Out::Emit(_, text, _) = out else {
            panic!("expected the unapprove to succeed, got a refusal")
        };
        assert!(text.contains("set to false"), "{text}");
        let lane = read_json_file(root, ".bee/lanes/gate-door-unapprove.json");
        assert_eq!(lane["approved_gates"]["execution"], json!(false));
        assert!(lane["gate_revoked_at"]["execution"].is_string(), "{lane:?}");
    }

    /// A non-high-risk record's gate behaviour is byte-for-byte unchanged —
    /// no advisor_ref at all, and the approval still goes through clean.
    #[test]
    fn non_high_risk_gate_approval_is_unaffected_by_the_advisor_precondition() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(
            root,
            ".bee/lanes/gate-door-safe.json",
            r#"{"feature":"gate-door-safe","phase":"planning","mode":"safe"}"#,
        );
        let out = run_gate_body(
            root,
            &flags(&["--lane", "gate-door-safe", "--merge", "--approved", "true"]),
        )
        .unwrap();
        let Out::Emit(_, text, _) = out else {
            panic!("expected a clean approval, got a refusal")
        };
        assert!(text.contains("Gates \"shape\" and \"execution\" set to true"), "{text}");
        let lane = read_json_file(root, ".bee/lanes/gate-door-safe.json");
        assert_eq!(lane["approved_gates"]["execution"], json!(true));
        assert_eq!(lane["approved_gates"]["shape"], json!(true));
    }

    // ── run_gate_body — D5's plan-time conflict precondition (koh-9) ────────
    //
    // The twin of the advisor block above, and written the same way: the
    // gate refuses a merged/execution APPROVAL while the plan-time conflict
    // check is absent, stale, or unfinished. `conflict_review` lives on the
    // LIVE WORKFLOW RECORD (koh-8 writes it there and the lane projection
    // never copies it down), so every fixture here writes both files — the
    // lane record the gate mutates and the workflow record the precondition
    // reads.

    /// One lane whose workflow record carries `plan_rev` and, optionally,
    /// the `conflict_review` object `state plan-conflicts derive` writes.
    fn write_conflict_fixture(root: &Path, feature: &str, plan_rev: u64, review: Option<&str>) {
        w(
            root,
            &format!(".bee/lanes/{feature}.json"),
            &format!(r#"{{"feature":"{feature}","phase":"planning","mode":"standard"}}"#),
        );
        let review_field = match review {
            Some(r) => format!(r#","conflict_review":{r}"#),
            None => String::new(),
        };
        w(
            root,
            ".bee/runtime/workflows/wf-1/state.json",
            &format!(
                r#"{{"id":"wf-1","feature":"{feature}","status":"active","phase":"planning","plan_rev":{plan_rev},"created_at":"2026-01-01T00:00:00.000Z"{review_field}}}"#
            ),
        );
    }

    fn merged_approval(root: &Path, feature: &str) -> Out {
        run_gate_body(root, &flags(&["--lane", feature, "--merge", "--approved", "true"])).unwrap()
    }

    /// Refusal cause 1: the check has never run at all. ABSENT is its own
    /// fact — no default review is merged in on the read, so "never derived"
    /// can never be mistaken for "derived and empty".
    #[test]
    fn merged_gate_approval_refuses_when_the_lane_has_no_conflict_review() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_conflict_fixture(root, "koh9-absent", 1, None);
        let before =
            std::fs::read_to_string(root.join(".bee/runtime/workflows/wf-1/state.json")).unwrap();
        let Out::Thrown(msg) = merged_approval(root, "koh9-absent") else {
            panic!("expected a refusal with no conflict_review recorded")
        };
        assert!(msg.contains("koh9-absent"), "{msg}");
        assert!(msg.contains("no conflict_review"), "{msg}");
        assert!(msg.contains("bee state plan-conflicts derive --lane koh9-absent"), "{msg}");
        let after =
            std::fs::read_to_string(root.join(".bee/runtime/workflows/wf-1/state.json")).unwrap();
        assert_eq!(after, before, "a refusal must write nothing");
    }

    /// Refusal cause 2: the review was derived against an older plan_rev.
    /// Every verdict below is recorded, so ONLY the rev mismatch can refuse
    /// this call — and the message says the bump is itself the reset.
    #[test]
    fn merged_gate_approval_refuses_when_the_conflict_review_is_stale_by_plan_rev() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_conflict_fixture(
            root,
            "koh9-stale",
            2,
            Some(
                r#"{"plan_rev":1,"derived_at":"2026-01-01T00:00:00.000Z","candidates":[{"id":"efd6cbaa","kind":"decision","title":"t","verdict":"compatible","note":null}]}"#,
            ),
        );
        let Out::Thrown(msg) = merged_approval(root, "koh9-stale") else {
            panic!("expected a refusal against a stale conflict review")
        };
        assert!(msg.contains("derived against plan_rev 1"), "{msg}");
        assert!(msg.contains("now at plan_rev 2"), "{msg}");
        assert!(msg.contains("plan-rev bump invalidates the review"), "{msg}");
    }

    /// Refusal cause 3: a candidate still carries a null verdict. The
    /// refusal NAMES the unverdicted id and leaves the verdicted one out.
    #[test]
    fn merged_gate_approval_refuses_and_names_every_unverdicted_candidate() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_conflict_fixture(
            root,
            "koh9-open",
            1,
            Some(
                r#"{"plan_rev":1,"derived_at":"2026-01-01T00:00:00.000Z","candidates":[{"id":"efd6cbaa","kind":"decision","title":"t","verdict":"compatible","note":null},{"id":"agents-proof-at-cap","kind":"rule","title":"r","verdict":null,"note":null}]}"#,
            ),
        );
        let Out::Thrown(msg) = merged_approval(root, "koh9-open") else {
            panic!("expected a refusal naming the unverdicted candidate")
        };
        assert!(msg.contains("agents-proof-at-cap"), "{msg}");
        assert!(!msg.contains("efd6cbaa"), "the verdicted candidate must not be named: {msg}");
        assert!(msg.contains("1 conflict candidate(s) still carry no verdict"), "{msg}");
        assert!(msg.contains("plan-conflicts verdict"), "{msg}");
    }

    /// The happy path: every candidate verdicted at the current plan_rev —
    /// the approval goes through, and nothing is added to the output.
    #[test]
    fn merged_gate_approval_succeeds_once_every_candidate_carries_a_verdict() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_conflict_fixture(
            root,
            "koh9-green",
            3,
            Some(
                r#"{"plan_rev":3,"derived_at":"2026-01-01T00:00:00.000Z","candidates":[{"id":"efd6cbaa","kind":"decision","title":"t","verdict":"compatible","note":null},{"id":"27e55095","kind":"decision","title":"t","verdict":"retires-prior","note":"replaces 3ea7500a"}]}"#,
            ),
        );
        let Out::Emit(record, text, _) = merged_approval(root, "koh9-green") else {
            panic!("expected the approval to succeed with every verdict recorded")
        };
        assert!(text.contains("Gates \"shape\" and \"execution\" set to true"), "{text}");
        assert!(!text.contains("conflicts"), "no conflict note is owed here: {text}");
        assert_eq!(record["conflicts_acknowledged"], Value::Null, "{record:?}");
        let wf = read_json_file(root, ".bee/runtime/workflows/wf-1/state.json");
        assert_eq!(wf["gates"]["execution"]["state"], json!("approved"), "{wf:?}");
        assert_eq!(wf["gates"]["shape"]["state"], json!("approved"), "{wf:?}");
    }

    /// D5: "0 conflicts" is valid exactly when bee returned zero candidates —
    /// an empty derived list approves, and it is a different fact from the
    /// absent field cause 1 refuses on.
    #[test]
    fn merged_gate_approval_succeeds_on_a_derived_review_with_zero_candidates() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_conflict_fixture(
            root,
            "koh9-zero",
            0,
            Some(r#"{"plan_rev":0,"derived_at":"2026-01-01T00:00:00.000Z","candidates":[]}"#),
        );
        let Out::Emit(..) = merged_approval(root, "koh9-zero") else {
            panic!("a zero-candidate derive is a valid \"0 conflicts\" and must approve")
        };
    }

    /// D5 verbatim, and the cell's prohibition: a recorded `conflicts`
    /// verdict does NOT refuse. The approval succeeds and NAMES the
    /// candidate, on the text and as a typed field, so the user takes the
    /// contradiction on with eyes open.
    #[test]
    fn merged_gate_approval_over_a_conflicts_verdict_succeeds_and_names_it() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_conflict_fixture(
            root,
            "koh9-ack",
            1,
            Some(
                r#"{"plan_rev":1,"derived_at":"2026-01-01T00:00:00.000Z","candidates":[{"id":"efd6cbaa","kind":"decision","title":"t","verdict":"conflicts","note":"taken on"},{"id":"27e55095","kind":"decision","title":"t","verdict":"compatible","note":null}]}"#,
            ),
        );
        let Out::Emit(record, text, _) = merged_approval(root, "koh9-ack") else {
            panic!("a conflicts verdict must never refuse the approval")
        };
        assert!(text.contains("Gates \"shape\" and \"execution\" set to true"), "{text}");
        assert!(text.contains("efd6cbaa"), "{text}");
        assert!(text.contains("eyes open"), "{text}");
        assert_eq!(record["conflicts_acknowledged"], json!(["efd6cbaa"]), "{record:?}");
    }

    /// A `plan-rev bump` after the approval is the reset: the review keeps
    /// its old rev, so the NEXT approval refuses as stale with no separate
    /// clear step. (The bump itself is `run_plan_rev_bump`, which needs a
    /// process CWD; its one effect on this door — `plan_rev` moving on the
    /// workflow record — is written directly here.)
    #[test]
    fn a_plan_rev_bump_after_an_approval_makes_the_next_approval_refuse_as_stale() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_conflict_fixture(
            root,
            "koh9-bump",
            1,
            Some(
                r#"{"plan_rev":1,"derived_at":"2026-01-01T00:00:00.000Z","candidates":[{"id":"efd6cbaa","kind":"decision","title":"t","verdict":"compatible","note":null}]}"#,
            ),
        );
        let Out::Emit(..) = merged_approval(root, "koh9-bump") else {
            panic!("the first approval stands on a fresh review")
        };
        let mut wf = read_json_file(root, ".bee/runtime/workflows/wf-1/state.json");
        wf["plan_rev"] = json!(2);
        w(root, ".bee/runtime/workflows/wf-1/state.json", &wf.to_string());
        let Out::Thrown(msg) = merged_approval(root, "koh9-bump") else {
            panic!("the bumped lane's next approval must refuse as stale")
        };
        assert!(msg.contains("derived against plan_rev 1"), "{msg}");
        assert!(msg.contains("now at plan_rev 2"), "{msg}");
    }

    /// The unapprove path never runs the precondition — `exec_component &&
    /// approved` is false — so a lane that never derived can still revoke.
    #[test]
    fn gate_unapprove_is_unaffected_by_the_conflict_precondition() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_conflict_fixture(root, "koh9-unapprove", 1, None);
        let out = run_gate_body(
            root,
            &flags(&["--lane", "koh9-unapprove", "--merge", "--approved", "false"]),
        )
        .unwrap();
        let Out::Emit(_, text, _) = out else {
            panic!("an unapprove is never gated by the conflict precondition")
        };
        assert!(text.contains("set to false"), "{text}");
    }

    /// Prohibition: the default (no-lane) record is untouched by this door.
    /// `conflict_review` is as lane-scoped as `plan_rev`, so a default-record
    /// approval has no review it could ever have recorded.
    #[test]
    fn default_record_gate_approval_is_unaffected_by_the_conflict_precondition() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_conflict_fixture(root, "koh9-default", 1, None);
        w(
            root,
            ".bee/state.json",
            r#"{"phase":"planning","feature":"koh9-default"}"#,
        );
        let out = run_gate_body(root, &flags(&["--no-lane", "--merge", "--approved", "true"]))
            .unwrap();
        let Out::Emit(..) = out else {
            panic!("the default record must approve exactly as before")
        };
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["approved_gates"]["execution"], json!(true), "{state:?}");
    }

    /// The `--name execution` spelling carries the same precondition as
    /// `--merge` (both are `exec_component`); `--name shape` alone does not.
    #[test]
    fn the_conflict_precondition_covers_name_execution_but_not_name_shape() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_conflict_fixture(root, "koh9-named", 1, None);
        let Out::Thrown(msg) = run_gate_body(
            root,
            &flags(&["--lane", "koh9-named", "--name", "execution", "--approved", "true"]),
        )
        .unwrap() else {
            panic!("a --name execution approval must refuse just like --merge")
        };
        assert!(msg.contains("no conflict_review"), "{msg}");
        let out = run_gate_body(
            root,
            &flags(&["--lane", "koh9-named", "--name", "shape", "--approved", "true"]),
        )
        .unwrap();
        let Out::Emit(..) = out else {
            panic!("a shape-only approval is not an execution approval")
        };
    }

    // ── run_gate_body — input validation (giv-1) ────────────────────────────
    //
    // `bee state gate` must refuse an unknown --name and a non-boolean
    // --approved before any lock or write, on both the plain and --merge
    // paths, and must leave every currently-legal call byte-identical.

    #[test]
    fn gate_refuses_an_unknown_name_and_writes_nothing() {
        let tmp = tmp_root();
        let root = tmp.path();
        let before = r#"{"phase":"planning","feature":"gate-door-unknown-name","approved_gates":{}}"#;
        w(root, ".bee/state.json", before);
        let out = run_gate_body(
            root,
            &flags(&["--no-lane", "--name", "bogus", "--approved", "true"]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else {
            panic!("expected an unknown --name to be refused, got a write")
        };
        assert!(msg.contains("bogus"), "{msg}");
        for legal in GATE_NAMES {
            assert!(msg.contains(legal), "{msg} missing legal name {legal}");
        }
        let after = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        assert_eq!(after, before, "a refusal must write nothing");
    }

    #[test]
    fn gate_refuses_a_non_boolean_approved_and_writes_nothing() {
        let tmp = tmp_root();
        let root = tmp.path();
        let before = r#"{"phase":"planning","feature":"gate-door-non-bool","approved_gates":{}}"#;
        w(root, ".bee/state.json", before);
        let out = run_gate_body(
            root,
            &flags(&["--no-lane", "--name", "execution", "--approved", "yes"]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else {
            panic!("expected a non-boolean --approved to be refused, got a write")
        };
        assert!(msg.contains("yes"), "{msg}");
        let after = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        assert_eq!(after, before, "a refusal must write nothing");
    }

    #[test]
    fn gate_merge_refuses_a_non_boolean_approved_and_writes_nothing() {
        let tmp = tmp_root();
        let root = tmp.path();
        let before = r#"{"phase":"planning","feature":"gate-door-merge-non-bool","approved_gates":{}}"#;
        w(root, ".bee/state.json", before);
        let out = run_gate_body(root, &flags(&["--no-lane", "--merge", "--approved", "maybe"]))
            .unwrap();
        let Out::Thrown(msg) = out else {
            panic!("expected a non-boolean --approved to be refused on --merge, got a write")
        };
        assert!(msg.contains("maybe"), "{msg}");
        let after = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        assert_eq!(after, before, "a refusal must write nothing");
    }

    /// Pin: a legal, non-merge `gate --name` call still succeeds unchanged.
    #[test]
    fn a_legal_gate_by_name_call_still_succeeds() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(
            root,
            ".bee/lanes/gate-door-legal-name.json",
            r#"{"feature":"gate-door-legal-name","phase":"planning","mode":"safe","approved_gates":{}}"#,
        );
        let out = run_gate_body(
            root,
            &flags(&["--lane", "gate-door-legal-name", "--name", "shape", "--approved", "true"]),
        )
        .unwrap();
        let Out::Emit(_, text, _) = out else {
            panic!("expected a clean approval, got a refusal")
        };
        assert!(text.contains("Gate \"shape\" set to true"), "{text}");
        let lane = read_json_file(root, ".bee/lanes/gate-door-legal-name.json");
        assert_eq!(lane["approved_gates"]["shape"], json!(true));
    }

    /// Pin: a legal `gate --merge` call still succeeds unchanged (mirrors
    /// non_high_risk_gate_approval_is_unaffected_by_the_advisor_precondition
    /// above, kept here so the input-validation suite is self-contained).
    #[test]
    fn a_legal_gate_merge_call_still_succeeds() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(
            root,
            ".bee/lanes/gate-door-legal-merge.json",
            r#"{"feature":"gate-door-legal-merge","phase":"planning","mode":"safe"}"#,
        );
        let out = run_gate_body(
            root,
            &flags(&["--lane", "gate-door-legal-merge", "--merge", "--approved", "true"]),
        )
        .unwrap();
        let Out::Emit(_, text, _) = out else {
            panic!("expected a clean approval, got a refusal")
        };
        assert!(text.contains("Gates \"shape\" and \"execution\" set to true"), "{text}");
        let lane = read_json_file(root, ".bee/lanes/gate-door-legal-merge.json");
        assert_eq!(lane["approved_gates"]["execution"], json!(true));
        assert_eq!(lane["approved_gates"]["shape"], json!(true));
    }

    // ── run_gate_body — D2/D3 actor/bypass-level/reason trace (trun-2) ─────
    //
    // `default_gate_entry()` already seeds every gate's `state` as
    // "pending" at record-creation time (trun-1, pinned in tests.rs); this
    // cell's own job is the actor side of D2/D3 — `bee state gate` records
    // WHO moved the gate, under what bypass level, and why, on the LIVE
    // WORKFLOW RECORD (`.bee/runtime/workflows/<id>/state.json`), never on
    // the legacy state/lane file, which never carried actor/reason.

    fn write_gate_trace_fixture(root: &Path, feature: &str, mode: Option<&str>) {
        let state = match mode {
            Some(m) => format!(r#"{{"phase":"planning","feature":"{feature}","mode":"{m}"}}"#),
            None => format!(r#"{{"phase":"planning","feature":"{feature}"}}"#),
        };
        w(root, ".bee/state.json", &state);
        w(
            root,
            ".bee/runtime/workflows/wf-1/state.json",
            &format!(
                r#"{{"id":"wf-1","feature":"{feature}","status":"active","phase":"planning","created_at":"2026-01-01T00:00:00.000Z"}}"#
            ),
        );
    }

    /// Truth 1: `--actor` defaults to "user" and, on approval, the touched
    /// gate's entry on the workflow record carries `state: approved`,
    /// `actor: user` and a real timestamp — untouched gates stay pending.
    #[test]
    fn gate_approval_by_the_default_user_actor_records_state_actor_and_a_timestamp() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_gate_trace_fixture(root, "gate-trace-user", None);
        let out = run_gate_body(
            root,
            &flags(&["--no-lane", "--name", "context", "--approved", "true"]),
        )
        .unwrap();
        let Out::Emit(..) = out else { panic!("expected the approval to succeed") };
        let wf = read_json_file(root, ".bee/runtime/workflows/wf-1/state.json");
        assert_eq!(wf["gates"]["context"]["state"], json!("approved"), "{wf:?}");
        assert_eq!(wf["gates"]["context"]["actor"], json!("user"), "{wf:?}");
        assert!(wf["gates"]["context"]["at"].is_string(), "{wf:?}");
        assert_eq!(wf["gates"]["context"]["reason"], Value::Null, "{wf:?}");
        assert_eq!(wf["gates"]["context"]["bypass_level"], Value::Null, "{wf:?}");
        assert_eq!(wf["gates"]["shape"]["state"], json!("pending"), "untouched gate: {wf:?}");
    }

    /// uat-gate-before-merge D1, must-have 1: `--name uat --approved true`
    /// (default `--actor user`) records normally — the legacy
    /// `approved_gates.uat` boolean AND the workflow record's rich entry.
    #[test]
    fn gate_accepts_uat_from_the_default_user_actor_and_persists_it() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_gate_trace_fixture(root, "gate-trace-uat", None);
        let out = run_gate_body(
            root,
            &flags(&["--no-lane", "--name", "uat", "--approved", "true"]),
        )
        .unwrap();
        let Out::Emit(..) = out else { panic!("expected the uat approval to succeed") };
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["approved_gates"]["uat"], json!(true), "{state:?}");
        let wf = read_json_file(root, ".bee/runtime/workflows/wf-1/state.json");
        assert_eq!(wf["gates"]["uat"]["state"], json!("approved"), "{wf:?}");
        assert_eq!(wf["gates"]["uat"]["actor"], json!("user"), "{wf:?}");
        assert!(wf["gates"]["uat"]["at"].is_string(), "{wf:?}");
    }

    /// uat-gate-before-merge D1, must-have 2: `--name uat --actor auto` is
    /// refused before any lock or write — the uat gate is never
    /// bypass-approved, even with a legal `--bypass-level`/`--reason` pair.
    #[test]
    fn gate_refuses_uat_from_actor_auto_and_writes_nothing() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_gate_trace_fixture(root, "gate-trace-uat-auto", None);
        let before_state = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        let before_wf =
            std::fs::read_to_string(root.join(".bee/runtime/workflows/wf-1/state.json")).unwrap();
        let out = run_gate_body(
            root,
            &flags(&[
                "--no-lane", "--name", "uat", "--approved", "true",
                "--actor", "auto", "--bypass-level", "total", "--reason", "ci",
            ]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else {
            panic!("expected --name uat --actor auto to be refused, got a write")
        };
        assert!(msg.contains("uat"), "{msg}");
        assert!(msg.contains("never bypass-approved") || msg.contains("D1"), "{msg}");
        let after_state = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        let after_wf =
            std::fs::read_to_string(root.join(".bee/runtime/workflows/wf-1/state.json")).unwrap();
        assert_eq!(after_state, before_state, "a refusal must write nothing to the legacy record");
        assert_eq!(after_wf, before_wf, "a refusal must write nothing to the workflow record");
    }

    /// Truth 2: `--actor auto --bypass-level normal --reason X` records all
    /// three on the entry.
    #[test]
    fn gate_approval_by_actor_auto_records_bypass_level_and_reason() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_gate_trace_fixture(root, "gate-trace-auto", None);
        let out = run_gate_body(
            root,
            &flags(&[
                "--no-lane", "--name", "review", "--approved", "true",
                "--actor", "auto", "--bypass-level", "normal", "--reason", "gate_bypass=normal",
            ]),
        )
        .unwrap();
        let Out::Emit(..) = out else { panic!("expected the auto approval to succeed") };
        let wf = read_json_file(root, ".bee/runtime/workflows/wf-1/state.json");
        assert_eq!(wf["gates"]["review"]["state"], json!("approved"), "{wf:?}");
        assert_eq!(wf["gates"]["review"]["actor"], json!("auto"), "{wf:?}");
        assert_eq!(wf["gates"]["review"]["bypass_level"], json!("normal"), "{wf:?}");
        assert_eq!(wf["gates"]["review"]["reason"], json!("gate_bypass=normal"), "{wf:?}");
    }

    /// Truth 3a: `--actor auto` without `--bypass-level` refuses (D2 — an
    /// auto-approval must never be LESS traceable than a stopped run) and
    /// writes nothing to either the legacy record or the workflow record.
    #[test]
    fn gate_actor_auto_without_bypass_level_refuses_and_writes_nothing() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_gate_trace_fixture(root, "gate-trace-auto-no-level", None);
        let before_state = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        let before_wf =
            std::fs::read_to_string(root.join(".bee/runtime/workflows/wf-1/state.json")).unwrap();
        let out = run_gate_body(
            root,
            &flags(&[
                "--no-lane", "--name", "context", "--approved", "true",
                "--actor", "auto", "--reason", "ci",
            ]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else {
            panic!("expected an auto call without --bypass-level to refuse")
        };
        assert!(msg.contains("--bypass-level"), "{msg}");
        assert!(msg.contains("--reason"), "{msg}");
        let after_state = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        let after_wf =
            std::fs::read_to_string(root.join(".bee/runtime/workflows/wf-1/state.json")).unwrap();
        assert_eq!(after_state, before_state, "a refusal must write nothing to the legacy record");
        assert_eq!(after_wf, before_wf, "a refusal must write nothing to the workflow record");
    }

    /// Truth 3b: the same door on the missing side — `--bypass-level`
    /// present, `--reason` absent — also refuses and writes nothing.
    #[test]
    fn gate_actor_auto_without_reason_refuses_and_writes_nothing() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_gate_trace_fixture(root, "gate-trace-auto-no-reason", None);
        let before_state = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        let out = run_gate_body(
            root,
            &flags(&[
                "--no-lane", "--name", "context", "--approved", "true",
                "--actor", "auto", "--bypass-level", "full",
            ]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else {
            panic!("expected an auto call without --reason to refuse")
        };
        assert!(msg.contains("--bypass-level"), "{msg}");
        assert!(msg.contains("--reason"), "{msg}");
        let after_state = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        assert_eq!(after_state, before_state, "a refusal must write nothing");
    }

    /// Truth 4: `--approved false` still stamps `gate_revoked_at` exactly as
    /// before AND the workflow record's entry reads `state: rejected`,
    /// while still carrying who rejected it and why.
    #[test]
    fn gate_rejection_stamps_revoked_and_records_rejected_state_and_actor() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_gate_trace_fixture(root, "gate-trace-reject", None);
        run_gate_body(root, &flags(&["--no-lane", "--merge", "--approved", "true"])).unwrap();
        let out = run_gate_body(
            root,
            &flags(&[
                "--no-lane", "--merge", "--approved", "false",
                "--actor", "user", "--reason", "found a defect",
            ]),
        )
        .unwrap();
        let Out::Emit(..) = out else { panic!("expected the revocation to succeed") };
        let state = read_json_file(root, ".bee/state.json");
        assert!(state["gate_revoked_at"]["execution"].is_string(), "{state:?}");
        let wf = read_json_file(root, ".bee/runtime/workflows/wf-1/state.json");
        assert_eq!(wf["gates"]["execution"]["state"], json!("rejected"), "{wf:?}");
        assert_eq!(wf["gates"]["execution"]["actor"], json!("user"), "{wf:?}");
        assert_eq!(wf["gates"]["execution"]["reason"], json!("found a defect"), "{wf:?}");
    }

    /// Input extremes: an unrecognized `--actor` refuses loudly, naming the
    /// legal values, and writes nothing.
    #[test]
    fn gate_refuses_an_unknown_actor_and_writes_nothing() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_gate_trace_fixture(root, "gate-trace-bad-actor", None);
        let before = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        let out = run_gate_body(
            root,
            &flags(&["--no-lane", "--name", "context", "--approved", "true", "--actor", "robot"]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else { panic!("expected an unknown --actor to be refused") };
        assert!(msg.contains("robot"), "{msg}");
        assert!(msg.contains("user"), "{msg}");
        assert!(msg.contains("auto"), "{msg}");
        let after = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        assert_eq!(after, before, "a refusal must write nothing");
    }

    /// Input extremes: an unrecognized `--bypass-level` refuses loudly, even
    /// while `--actor` is the default "user".
    #[test]
    fn gate_refuses_an_unknown_bypass_level_and_writes_nothing() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_gate_trace_fixture(root, "gate-trace-bad-level", None);
        let before = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        let out = run_gate_body(
            root,
            &flags(&[
                "--no-lane", "--name", "context", "--approved", "true", "--bypass-level", "extreme",
            ]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else {
            panic!("expected an unknown --bypass-level to be refused")
        };
        assert!(msg.contains("extreme"), "{msg}");
        assert!(msg.contains("normal"), "{msg}");
        let after = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        assert_eq!(after, before, "a refusal must write nothing");
    }

    /// `--merge` still addresses shape+execution together (no change to
    /// that semantics) and stamps the SAME actor/bypass/reason trace onto
    /// both entries in one call.
    #[test]
    fn gate_merge_with_actor_auto_writes_the_same_trace_to_shape_and_execution() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_gate_trace_fixture(root, "gate-trace-merge-auto", None);
        let out = run_gate_body(
            root,
            &flags(&[
                "--no-lane", "--merge", "--approved", "true",
                "--actor", "auto", "--bypass-level", "full", "--reason", "gate_bypass=full",
            ]),
        )
        .unwrap();
        let Out::Emit(..) = out else { panic!("expected the merged approval to succeed") };
        let wf = read_json_file(root, ".bee/runtime/workflows/wf-1/state.json");
        for gate in ["shape", "execution"] {
            assert_eq!(wf["gates"][gate]["state"], json!("approved"), "{wf:?}");
            assert_eq!(wf["gates"][gate]["actor"], json!("auto"), "{wf:?}");
            assert_eq!(wf["gates"][gate]["bypass_level"], json!("full"), "{wf:?}");
            assert_eq!(wf["gates"][gate]["reason"], json!("gate_bypass=full"), "{wf:?}");
        }
    }

    // ── run_set_body — the feature-swap scribing-debt door (fsd-1) ─────────
    //
    // scribing-integrity D1: a real `--feature` swap on the default record
    // abandons the CURRENT feature without ever reaching its close, so it is
    // refused the same way `--phase compounding-complete` is above — same
    // counter (drivers::scribing_debt), same two escapes, its own message
    // and its own after-write decision naming the ABANDONED feature.

    fn idle_state_at(phase: &str, feature: &str) -> String {
        format!(r#"{{"phase":"{phase}","feature":"{feature}"}}"#)
    }

    /// Exotic carries no Debug impl — a local expect keeps `Result`
    /// unwrapping panics readable without editing it (tests.rs's own `ok`
    /// follows the same reasoning).
    fn ok_ex<T, E>(r: Result<T, E>) -> T {
        match r {
            Ok(v) => v,
            Err(_) => panic!("unexpected error result"),
        }
    }

    fn write_session(root: &Path, id: &str, lane: Option<&str>) {
        std::fs::create_dir_all(sessions_dir(root)).unwrap();
        let body = match lane {
            Some(l) => format!(r#"{{"id":"{id}","last_heartbeat":"{}","lane":"{l}"}}"#, now_iso()),
            None => format!(r#"{{"id":"{id}","last_heartbeat":"{}"}}"#, now_iso()),
        };
        std::fs::write(sessions_dir(root).join(format!("{id}.json")), body).unwrap();
    }

    fn fixture_session_id(root: &Path) -> String {
        ok_ex(resolve_session_id_no_flag(root)).unwrap_or_else(|| "sess-1".to_string())
    }

    /// A swap over unpaid debt on the outgoing feature refuses the whole
    /// mutation, names the outgoing feature and every unscribed cell id, and
    /// nothing reaches disk.
    #[test]
    fn feature_swap_over_unpaid_debt_refuses_and_writes_nothing() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/state.json", &idle_state_at("swarming", "outgoing"));
        w(root, ".bee/cells/o-1.json", r#"{"id":"o-1","feature":"outgoing","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        let out = run_set_body(
            root,
            &flags(&["--no-lane", "--feature", "incoming", "--owner", "swarming"]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else { panic!("expected a refusal, got a write") };
        assert!(msg.contains("refusing to swap away from feature \"outgoing\""), "{msg}");
        assert!(msg.contains("o-1"), "{msg}");
        assert!(msg.contains("abandons \"outgoing\""), "{msg}");
        assert!(msg.contains("--waive-scribing-debt"), "{msg}");
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["feature"], json!("outgoing"), "the refused swap must never reach disk");
        assert!(!root.join(".bee/decisions.jsonl").exists(), "a refusal logs nothing");
    }

    /// Three shapes that are NOT a debt-bearing swap at all: no debt on the
    /// outgoing feature, no current feature to abandon, and a same-value
    /// `--feature` (never a swap in the first place) — all three succeed
    /// even with standing debt in the third case.
    #[test]
    fn feature_swap_non_refusal_shapes_all_succeed() {
        // No debt on the outgoing feature.
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/state.json", &idle_state_at("swarming", "outgoing"));
        let out = run_set_body(
            root,
            &flags(&["--no-lane", "--feature", "incoming", "--owner", "swarming"]),
        )
        .unwrap();
        assert!(matches!(out, Out::Emit(..)), "a debt-free swap must succeed");
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["feature"], json!("incoming"));

        // No current feature — nothing was abandoned, even with debt sitting
        // under a DIFFERENT feature name.
        let tmp2 = tmp_root();
        let root2 = tmp2.path();
        w(root2, ".bee/state.json", r#"{"phase":"idle","feature":null}"#);
        w(root2, ".bee/cells/x-1.json", r#"{"id":"x-1","feature":"incoming","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        let out2 = run_set_body(
            root2,
            &flags(&["--no-lane", "--feature", "incoming", "--owner", "idle"]),
        )
        .unwrap();
        assert!(matches!(out2, Out::Emit(..)), "no current feature means nothing was abandoned");

        // Same-value --feature is not a swap at all — standing debt on
        // "outgoing" never even gets asked.
        let tmp3 = tmp_root();
        let root3 = tmp3.path();
        w(root3, ".bee/state.json", &idle_state_at("swarming", "outgoing"));
        w(root3, ".bee/cells/o-1.json", r#"{"id":"o-1","feature":"outgoing","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        let out3 = run_set_body(
            root3,
            &flags(&["--no-lane", "--feature", "outgoing", "--owner", "swarming"]),
        )
        .unwrap();
        assert!(matches!(out3, Out::Emit(..)), "a same-value --feature is not a swap");
    }

    /// `--waive-scribing-debt` lets the swap through, logs a decision AFTER
    /// the write naming the ABANDONED feature and every waived cell, and the
    /// emitted text carries the waiver suffix.
    #[test]
    fn feature_swap_waive_scribing_debt_passes_and_logs_the_abandoned_feature_after_write() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/state.json", &idle_state_at("swarming", "outgoing"));
        w(root, ".bee/cells/o-1.json", r#"{"id":"o-1","feature":"outgoing","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        let out = run_set_body(
            root,
            &flags(&[
                "--no-lane",
                "--feature",
                "incoming",
                "--owner",
                "swarming",
                "--waive-scribing-debt",
            ]),
        )
        .unwrap();
        let Out::Emit(_, text, _) = out else { panic!("expected the waiver to pass") };
        assert!(text.contains("SCRIBING DEBT WAIVED"), "{text}");
        assert!(text.contains("o-1"), "{text}");
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["feature"], json!("incoming"));
        let decisions = std::fs::read_to_string(root.join(".bee/decisions.jsonl")).unwrap();
        assert!(decisions.contains("Swapped away from feature"), "{decisions}");
        assert!(decisions.contains("outgoing"), "{decisions}");
        assert!(decisions.contains("scribing debt WAIVED"), "{decisions}");
        assert!(decisions.contains("o-1"), "{decisions}");
    }

    /// The capture-deferral escape is the SAME reader `bee close`'s own door
    /// uses (drivers::has_capture_deferral_decision) — a decision already
    /// logged naming the outgoing feature clears the swap without the flag.
    #[test]
    fn feature_swap_logged_capture_deferral_decision_passes_without_the_flag() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/state.json", &idle_state_at("swarming", "outgoing"));
        w(root, ".bee/cells/o-1.json", r#"{"id":"o-1","feature":"outgoing","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-07-02T12:00:00.000Z\",\"decision\":\"defer capture for outgoing until next sprint\",\"rationale\":\"r\",\"tags\":[\"capture-deferral\"],\"scope\":\"repo\"}\n",
        );
        let out = run_set_body(
            root,
            &flags(&["--no-lane", "--feature", "incoming", "--owner", "swarming"]),
        )
        .unwrap();
        assert!(
            matches!(out, Out::Emit(..)),
            "the deferral decision must lift the wall without the flag"
        );
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["feature"], json!("incoming"));
        let decisions = std::fs::read_to_string(root.join(".bee/decisions.jsonl")).unwrap();
        assert_eq!(decisions.lines().count(), 1, "no redundant waiver decision appended");
    }

    /// Regression: the lane paths keep today's refusals — `--feature`
    /// combined with an explicit `--lane` never even reaches the swap door.
    #[test]
    fn feature_swap_still_refuses_with_an_explicit_lane() {
        let tmp = tmp_root();
        let root = tmp.path();
        let out = run_set_body(
            root,
            &flags(&["--lane", "some-lane", "--feature", "incoming", "--owner", "swarming"]),
        )
        .unwrap();
        let Out::Thrown(msg) = out else { panic!("expected a refusal, got a write") };
        assert!(msg.contains("--feature cannot be combined with --lane"), "{msg}");
    }

    /// Regression: `--feature` against a session-AUTO-resolved lane (no
    /// explicit `--lane` on the argv) still refuses too.
    #[test]
    fn feature_swap_still_refuses_against_a_session_auto_resolved_lane() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(
            root,
            ".bee/lanes/bound-lane.json",
            r#"{"feature":"bound-lane","phase":"swarming"}"#,
        );
        let sid = fixture_session_id(root);
        write_session(root, &sid, Some("bound-lane"));
        let out = run_set_body(root, &flags(&["--feature", "incoming", "--owner", "swarming"]))
            .unwrap();
        let Out::Thrown(msg) = out else { panic!("expected a refusal, got a write") };
        assert!(msg.contains("--feature cannot target lane \"bound-lane\""), "{msg}");
    }

    // ── ssh-1: a swap closes every OTHER live workflow record ──────────────
    //
    // start-feature already reaps correctly (policy.rs's
    // ensure_workflow_record_for_feature + close_workflows_for_feature); a
    // `state set --feature <swap>` did not, because write_through_projection
    // deliberately falls to the C1 direct write for a swap and never routes
    // through the workflow store at all. These are red against pre-fix code:
    // the outgoing feature's live workflow record stayed "active" forever.

    fn write_workflow_fixture(root: &Path, id: &str, body: &str) {
        w(root, &format!(".bee/runtime/workflows/{id}/state.json"), body);
    }

    fn read_workflow_fixture(root: &Path, id: &str) -> Value {
        read_json_file(root, &format!(".bee/runtime/workflows/{id}/state.json"))
    }

    /// A real swap closes the OUTGOING feature's live workflow record and
    /// creates one for the INCOMING feature — the same reap start-feature
    /// already runs, ridden after the C1 write instead of replacing it.
    #[test]
    fn a_feature_swap_closes_the_outgoing_workflow_and_opens_the_incoming_one() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/state.json", &idle_state_at("swarming", "outgoing"));
        write_workflow_fixture(
            root,
            "wf-outgoing",
            r#"{"id":"wf-outgoing","feature":"outgoing","status":"active","phase":"swarming",
                "summary":"","next_action":"","created_at":"2026-01-01T00:00:00.000Z"}"#,
        );
        let out = run_set_body(
            root,
            &flags(&["--no-lane", "--feature", "incoming", "--owner", "swarming"]),
        )
        .unwrap();
        assert!(matches!(out, Out::Emit(..)), "a debt-free swap must succeed");
        // The C1 write itself is untouched: state.json took the swap.
        let state = read_json_file(root, ".bee/state.json");
        assert_eq!(state["feature"], json!("incoming"));
        // The outgoing feature's live record is now terminal…
        let outgoing = read_workflow_fixture(root, "wf-outgoing");
        assert_eq!(outgoing["status"], json!("closed"), "{outgoing}");
        // …and the incoming feature has a live record of its own.
        let workflows = ok_ex(list_workflows(root));
        let incoming = workflows
            .iter()
            .find(|wf| wf.get("feature") == Some(&json!("incoming")))
            .unwrap_or_else(|| panic!("no workflow record for the incoming feature: {workflows:?}"));
        assert_eq!(incoming.get("status"), Some(&json!("active")));
    }

    /// A same-value `--feature` is not a swap at all: the pre-existing live
    /// workflow record for that feature is left completely untouched (no
    /// reap runs, no second record is created).
    #[test]
    fn a_same_feature_set_leaves_the_live_workflow_record_untouched() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/state.json", &idle_state_at("swarming", "outgoing"));
        write_workflow_fixture(
            root,
            "wf-outgoing",
            r#"{"id":"wf-outgoing","feature":"outgoing","status":"active","phase":"swarming",
                "summary":"","next_action":"","created_at":"2026-01-01T00:00:00.000Z"}"#,
        );
        let out = run_set_body(
            root,
            &flags(&["--no-lane", "--feature", "outgoing", "--owner", "swarming"]),
        )
        .unwrap();
        assert!(matches!(out, Out::Emit(..)));
        let workflows = ok_ex(list_workflows(root));
        assert_eq!(workflows.len(), 1, "no second record was created: {workflows:?}");
        assert_eq!(workflows[0].get("status"), Some(&json!("active")));
    }

    // ── run_gate_body — R4, the durable-stamp note (uat-approval-reaches-the-door) ──
    //
    // No existing test in this module approves a gate against a LANE whose
    // feature's workflow record is closed or absent, so this is new
    // coverage, not a duplicate. `gate_accepts_uat_from_the_default_user_actor_...`
    // above pins the live-workflow, no-lane path (the note must never
    // appear there); `high_risk_merged_gate_approval_succeeds_after_a_fresh_advisor_ref_is_recorded`
    // above pins the plain lane success text (`gate_durable_stamp_skipped`
    // must never appear there since a live workflow record answers it).

    /// The load-bearing case: a lane's feature has NO live workflow record
    /// (closed by ordinary housekeeping) and the gate is `uat`. The success
    /// line stays byte-identical; a second line names the half-write and
    /// the uat door's second source; the JSON result carries the same fact
    /// as a typed field.
    #[test]
    fn gate_approval_on_a_lane_with_no_live_workflow_names_the_stranded_uat_stamp() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(
            root,
            ".bee/lanes/uad-closed.json",
            r#"{"feature":"uad-closed","phase":"compounding"}"#,
        );
        write_workflow_fixture(
            root,
            "wf-uad-closed",
            r#"{"id":"wf-uad-closed","feature":"uad-closed","status":"closed","phase":"swarming",
                "summary":"","next_action":"","created_at":"2026-01-01T00:00:00.000Z"}"#,
        );
        let out = run_gate_body(
            root,
            &flags(&["--lane", "uad-closed", "--name", "uat", "--approved", "true"]),
        )
        .unwrap();
        let Out::Emit(json, text, _) = out else {
            panic!("expected the approval to succeed, got a refusal")
        };
        let mut lines = text.lines();
        assert_eq!(
            lines.next(),
            Some("Gate \"uat\" set to true. (lane \"uad-closed\")"),
            "the existing success line must stay byte-identical: {text:?}"
        );
        let note = lines.next().expect("a second line naming the half-write");
        assert!(note.contains("lane record"), "{note}");
        assert!(note.contains("closed") || note.contains("does not exist"), "{note}");
        assert!(note.contains("uat door"), "{note}");
        assert_eq!(lines.next(), None, "exactly one added line, never more: {text:?}");
        assert_eq!(json["gate_durable_stamp_skipped"], json!(true), "{json:?}");
        // The write itself is unchanged: the lane record still carries the
        // approval, and nothing closed reopened.
        let lane = read_json_file(root, ".bee/lanes/uad-closed.json");
        assert_eq!(lane["approved_gates"]["uat"], json!(true), "{lane:?}");
        let wf = read_workflow_fixture(root, "wf-uad-closed");
        assert_eq!(wf["status"], json!("closed"), "a closed record is never reopened: {wf:?}");
    }

    /// A non-uat gate on the same stranded shape still gets the general
    /// note and the field, but never the uat-specific sentence about the
    /// door's second source — that claim is only true for `uat`.
    #[test]
    fn gate_approval_on_a_lane_with_no_live_workflow_omits_the_uat_tail_for_other_gates() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(
            root,
            ".bee/lanes/uad-closed-2.json",
            r#"{"feature":"uad-closed-2","phase":"compounding"}"#,
        );
        write_workflow_fixture(
            root,
            "wf-uad-closed-2",
            r#"{"id":"wf-uad-closed-2","feature":"uad-closed-2","status":"closed","phase":"swarming",
                "summary":"","next_action":"","created_at":"2026-01-01T00:00:00.000Z"}"#,
        );
        let out = run_gate_body(
            root,
            &flags(&["--lane", "uad-closed-2", "--name", "context", "--approved", "true"]),
        )
        .unwrap();
        let Out::Emit(json, text, _) = out else {
            panic!("expected the approval to succeed, got a refusal")
        };
        assert!(text.contains("lane record"), "{text}");
        assert!(!text.contains("uat door"), "the uat-specific tail must not appear: {text}");
        assert_eq!(json["gate_durable_stamp_skipped"], json!(true), "{json:?}");
    }

    /// The live-workflow counterpart: same lane, same gate name, but the
    /// feature's workflow record is still active. Neither the note line nor
    /// the JSON field appears — the durable stamp was reachable and reached.
    #[test]
    fn gate_approval_on_a_lane_with_a_live_workflow_omits_the_note_and_field() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(
            root,
            ".bee/lanes/uad-live.json",
            r#"{"feature":"uad-live","phase":"compounding"}"#,
        );
        write_workflow_fixture(
            root,
            "wf-uad-live",
            r#"{"id":"wf-uad-live","feature":"uad-live","status":"active","phase":"swarming",
                "summary":"","next_action":"","created_at":"2026-01-01T00:00:00.000Z"}"#,
        );
        let out = run_gate_body(
            root,
            &flags(&["--lane", "uad-live", "--name", "uat", "--approved", "true"]),
        )
        .unwrap();
        let Out::Emit(json, text, _) = out else {
            panic!("expected the approval to succeed, got a refusal")
        };
        assert_eq!(
            text, "Gate \"uat\" set to true. (lane \"uad-live\")",
            "no second line when the durable stamp was reached: {text:?}"
        );
        assert!(
            json.get("gate_durable_stamp_skipped").is_none(),
            "the field must not appear on the live-record path: {json:?}"
        );
        let wf = read_workflow_fixture(root, "wf-uad-live");
        assert_eq!(wf["gates"]["uat"]["actor"], json!("user"), "{wf:?}");
    }

    /// The default (`--no-lane`) counterpart with no live workflow record:
    /// this is NOT the defect this cell fixes (the write already lands in
    /// `.bee/state.json`, which the doors' unchanged source-3 fallback
    /// already reads), so neither the note nor the field appears here.
    #[test]
    fn gate_approval_on_the_default_record_with_no_live_workflow_omits_the_note_and_field() {
        let tmp = tmp_root();
        let root = tmp.path();
        w(root, ".bee/state.json", &idle_state_at("compounding", "uad-no-lane"));
        let out = run_gate_body(
            root,
            &flags(&["--no-lane", "--name", "uat", "--approved", "true"]),
        )
        .unwrap();
        let Out::Emit(json, text, _) = out else {
            panic!("expected the approval to succeed, got a refusal")
        };
        assert_eq!(text, "Gate \"uat\" set to true.", "no lane, no added line: {text:?}");
        assert!(
            json.get("gate_durable_stamp_skipped").is_none(),
            "the field must not appear on the default-record path: {json:?}"
        );
    }
}
