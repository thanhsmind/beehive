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
    write_mailbox_handoff, MailboxAdopt,
};
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
        ["plan-rev", "bump", ..] => ("plan-rev.bump", 2),
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
        ["handoff", "write", ..] => ("handoff.write", 2),
        ["handoff", "adopt", ..] => ("handoff.adopt", 2),
        ["handoff", "show", ..] => ("handoff.show", 2),
        ["workflows", "list", ..] => ("workflows.list", 2),
        ["workflows", "close", ..] => ("workflows.close", 2),
        ["rebuild-projections", ..] => ("rebuild-projections", 1),
        ["route", ..] => ("route", 1),
        ["start-feature", ..] => ("start-feature", 1),
        _ => return None, // advisor-ref/compact-*/unknown → Node
    };
    if leading.len() != consumed {
        return None; // "Unexpected argument" — Node's own refusal path
    }
    let (flags, use_json) = parse_flags(rest)?;
    match verb {
        "set" => run_set(flags, use_json, t0),
        "gate" => run_gate(flags, use_json, t0),
        "plan-rev.bump" => run_plan_rev_bump(flags, use_json, t0),
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
        "handoff.write" => run_handoff_write(flags, use_json, t0),
        "handoff.adopt" => run_handoff_adopt(flags, use_json, t0),
        "handoff.show" => run_handoff_show(flags, use_json, t0),
        "workflows.list" => run_workflows_list(flags, use_json, t0),
        "workflows.close" => run_workflows_close(flags, use_json, t0),
        "rebuild-projections" => run_rebuild_projections(flags, use_json, t0),
        "route" => run_route(flags, use_json, t0),
        "start-feature" => run_start_feature(flags, use_json, t0),
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
    let out = (|| -> R2<Out> {
        let phase_flag = flag_string(&flags, "phase");
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
        let (lane_feature, no_lane) = match mutation_lane_selector(&flags, "set") {
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

        // withMutationLock's own pre-lock reads: the fail-open scope peek, then
        // the workflow listing that picks the lock names.
        let scope = resolve_mutation_lock_scope(&ctx.root, lane_feature.as_deref(), no_lane)?;
        let workflows = list_workflows(&ctx.root)?;

        // The two scribing-debt delegate triggers (for BOTH the lane and
        // default branches) are decided here, BEFORE any lock or output, off
        // a silent peek at the record the strict read will land on.
        if let Some(peek) = peek_target_record(&ctx.root, &scope, lane_feature.as_deref())?
        {
            if let Some(p) = &phase_flag {
                let t = check_phase_transition(peek.get("phase"), p, &peek, waive)?;
                if t.ok && p == "compounding-complete" {
                    return Err(Err2::Ex); // passing close -> the debt door on Node
                }
            }
            if !scope.lane {
                if let Some(f) = flag_string(&flags, "feature") {
                    let current = peek.get("feature");
                    if current.map(truthy).unwrap_or(false)
                        && !opt_strict_eq(current, Some(&Value::String(f.clone())))
                    {
                        return Err(Err2::Ex); // feature swap -> the debt door on Node
                    }
                }
            }
        }

        let locks = acquire_mutation_locks(&ctx.root, &scope, &workflows)?;
        let mut target =
            resolve_mutation_target(&ctx.root, lane_feature.as_deref(), "set", no_lane)?;
        // i54-closeout-7 (D7): a session-AUTO-resolved lane refuses --feature too
        // (the flag-level guard above only ever sees an EXPLICIT --lane).
        if flags.get("feature").is_some() {
            if let Some(lane) = target.lane() {
                return Ok(Out::Thrown(format!(
                    "set: --feature cannot target lane \"{lane}\" (auto-resolved from this session's lane binding) \u{2014} a lane's feature is its identity (the lane record's filename), not a mutable field. FIX: omit --feature, or pass --no-lane to address the default record."
                )));
            }
        }
        if let Some(p) = &phase_flag {
            let record = target.record();
            let t = check_phase_transition(record.get("phase"), p, record, waive)?;
            if !t.ok {
                return Ok(Out::Thrown(t.reason));
            }
            if p == "compounding-complete" {
                return Err(Err2::Ex); // race: the strict read now passes the door
            }
        }
        if target.lane().is_none() {
            if let Some(f) = flag_string(&flags, "feature") {
                let current = target.record().get("feature");
                if current.map(truthy).unwrap_or(false)
                    && !opt_strict_eq(current, Some(&Value::String(f.clone())))
                {
                    return Err(Err2::Ex);
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
        let mut changed: Vec<String> = Vec::new();
        {
            let state = target.record_mut();
            if let Some(p) = &phase_flag {
                state.insert("phase".into(), json!(p));
                changed.push(format!("phase={p}"));
            }
            if let Some(m) = flag_string(&flags, "mode") {
                state.insert("mode".into(), json!(m));
                changed.push(format!("mode={m}"));
            }
            if let Some(f) = flag_string(&flags, "feature") {
                state.insert("feature".into(), json!(f));
                changed.push(format!("feature={f}"));
            }
            if let Some(n) = flag_string(&flags, "next-action") {
                state.insert("next_action".into(), json!(n));
                changed.push("next_action".to_string());
            }
            if let Some(s) = flag_string(&flags, "summary") {
                state.insert("summary".into(), json!(s));
                changed.push("summary".to_string());
            }
        }
        let record = target.record().clone();
        write_through_projection(&ctx.root, &target, &record, &[])?;
        drop(locks);
        Ok(Out::Emit(
            Value::Object(record),
            format!("Updated state: {}.{lane_note}", changed.join(" ")),
            0,
        ))
    })();
    finish(&ctx, out)
}

// ─── state gate ────────────────────────────────────────────────────────────

pub(crate) fn run_gate(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["name", "merge", "approved", "lane", "no-lane", "owner"]) {
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
    let out = (|| -> R2<Out> {
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
        let values = match require_flags(&flags, &spec, EXAMPLE_GATE) {
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
        let (lane_feature, no_lane) = match mutation_lane_selector(&flags, "gate") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let exec_component = merge || name == "execution";

        let scope = resolve_mutation_lock_scope(&ctx.root, lane_feature.as_deref(), no_lane)?;
        let workflows = list_workflows(&ctx.root)?;
        // requireFreshAdvisorForHighRisk (advisorRefStale, lib/state.mjs) is a
        // separate R6 debt — decided off a silent peek, before any lock.
        if exec_component && approved {
            if let Some(peek) =
                peek_target_record(&ctx.root, &scope, lane_feature.as_deref())?
            {
                if matches!(peek.get("mode"), Some(Value::String(s)) if s == "high-risk") {
                    return Err(Err2::Ex);
                }
            }
        }
        let locks = acquire_mutation_locks(&ctx.root, &scope, &workflows)?;
        let mut target =
            resolve_mutation_target(&ctx.root, lane_feature.as_deref(), "gate", no_lane)?;
        if exec_component
            && approved
            && matches!(target.record().get("mode"), Some(Value::String(s)) if s == "high-risk")
        {
            return Err(Err2::Ex); // race: the peek missed the high-risk mode
        }
        let lane_note = target.lane_note();
        // multisession-native-9 D7 / validation-diet D15 — the plan-rev stamp is
        // LANE-ONLY and reads the live workflow's CURRENT plan_rev.
        let mut stamps: Vec<(String, Value)> = Vec::new();
        if exec_component {
            if let Some(lane) = target.lane() {
                let live = list_workflows(&ctx.root)?;
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
        write_through_projection(&ctx.root, &target, &record, &stamps)?;
        drop(locks);
        let text = if merge {
            format!("Gates \"shape\" and \"execution\" set to {approved}.{lane_note}")
        } else {
            format!("Gate \"{name}\" set to {approved}.{lane_note}")
        };
        Ok(Out::Emit(Value::Object(record), text, 0))
    })();
    finish(&ctx, out)
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
