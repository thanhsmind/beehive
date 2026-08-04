// `state worker …`, `state scribing-run` and `state compounding-run`
//
// Split out of the single 6.1k-line verbs/state_group.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{
    append_jsonl, ensure_dir, read_json, warn_corrupt_json, write_json_atomic, ReadJson,
};
use crate::jsjson;
use crate::lock::{self, AcquireOnce, LockGuard};
use crate::textutil::js_default_sort;
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

// ─── state worker add/update/remove/clear ──────────────────────────────────

/// stateWorkerMutate — the shared lock + strict-read + write frame.
pub(crate) fn worker_mutate(
    root: &Path,
    mutate: impl FnOnce(&mut Vec<Value>) -> Result<String, Err2>,
) -> R2<Out> {
    let guard = acquire_state_lock(root)?;
    let mut state = read_state_strict(root)?;
    let mut workers: Vec<Value> = match state.get("workers") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let text = mutate(&mut workers)?;
    state.insert("workers".into(), Value::Array(workers));
    write_state(root, &state)?;
    drop(guard);
    Ok(Out::Emit(Value::Object(state), text, 0))
}

/// A thrown-Error inside the mutate closure must surface as emitError, not a
/// delegate — collapse Err2::Msg into Out::Thrown at the boundary.
pub(crate) fn thrown_ok(out: R2<Out>) -> R2<Out> {
    match out {
        Err(Err2::Msg(m)) => Ok(Out::Thrown(m)),
        other => other,
    }
}

pub(crate) fn run_worker_add(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["nickname", "cell", "tier", "status"]) {
        return None;
    }
    let ctx = match go("state worker add", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = thrown_ok(worker_mutate(&ctx.root, |workers| {
        let nickname = require_flag(&flags, "nickname")?;
        let cell = require_flag(&flags, "cell")?;
        let tier: Value = match flag_string(&flags, "tier") {
            None => Value::Null,
            Some(t) => {
                if !MODEL_TIERS.contains(&t.as_str()) {
                    return Err(Err2::Msg(format!(
                        "worker add: invalid tier \"{t}\" — must be one of extraction, generation, ceiling."
                    )));
                }
                json!(t)
            }
        };
        let status: Value = match flag_string(&flags, "status") {
            None => Value::Null,
            Some(s) => json!(s),
        };
        workers.push(json!({"nickname": nickname, "cell": cell, "tier": tier, "status": status}));
        Ok(format!("Added worker \"{nickname}\" (cell {cell})."))
    }));
    finish(&ctx, out)
}

pub(crate) fn run_worker_update(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["nickname", "cell", "tier", "status"]) {
        return None;
    }
    let ctx = match go("state worker update", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = thrown_ok(worker_mutate(&ctx.root, |workers| {
        let nickname = require_flag(&flags, "nickname")?;
        let idx = workers.iter().position(|w| {
            truthy(w) && opt_strict_eq(jget(w, "nickname"), Some(&Value::String(nickname.clone())))
        });
        let Some(idx) = idx else {
            return Err(Err2::Msg(format!(
                "worker update: nickname \"{nickname}\" not found — use \"worker add\" to create it first."
            )));
        };
        // const worker = { ...workers[idx] } — always an object once matched.
        let mut worker = match &workers[idx] {
            Value::Object(m) => m.clone(),
            _ => return Err(Err2::Ex),
        };
        if let Some(c) = flag_string(&flags, "cell") {
            worker.insert("cell".into(), json!(c));
        }
        if let Some(t) = flag_string(&flags, "tier") {
            if !MODEL_TIERS.contains(&t.as_str()) {
                return Err(Err2::Msg(format!(
                    "worker update: invalid tier \"{t}\" — must be one of extraction, generation, ceiling."
                )));
            }
            worker.insert("tier".into(), json!(t));
        }
        if let Some(s) = flag_string(&flags, "status") {
            worker.insert("status".into(), json!(s));
        }
        workers[idx] = Value::Object(worker);
        Ok(format!("Updated worker \"{nickname}\"."))
    }));
    finish(&ctx, out)
}

pub(crate) fn run_worker_remove(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["nickname"]) {
        return None;
    }
    let ctx = match go("state worker remove", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = thrown_ok(worker_mutate(&ctx.root, |workers| {
        let nickname = require_flag(&flags, "nickname")?;
        let before = workers.len();
        workers.retain(|w| {
            !(truthy(w)
                && opt_strict_eq(jget(w, "nickname"), Some(&Value::String(nickname.clone()))))
        });
        if workers.len() == before {
            return Err(Err2::Msg(format!("worker remove: nickname \"{nickname}\" not found.")));
        }
        Ok(format!("Removed worker \"{nickname}\"."))
    }));
    finish(&ctx, out)
}

pub(crate) fn run_worker_clear(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match go("state worker clear", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = thrown_ok(worker_mutate(&ctx.root, |workers| {
        let removed = workers.len();
        workers.clear();
        Ok(format!("Cleared {removed} worker(s)."))
    }));
    finish(&ctx, out)
}

// ─── state worker prune ────────────────────────────────────────────────────

/// WORKER_TRANSIENT_SUFFIX — leftmost match of
/// /\.(prompt\.md|result\.md|result\.json|out\d*\.log|log)$/, returning the
/// matched suffix length.
pub(crate) fn worker_transient_suffix_len(name: &str) -> Option<usize> {
    let bytes = name.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] != b'.' {
            continue;
        }
        let tail = &name[i..];
        let matched = tail == ".prompt.md"
            || tail == ".result.md"
            || tail == ".result.json"
            || tail == ".log"
            || tail
                .strip_prefix(".out")
                .map(|rest| {
                    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
                    &rest[digits..] == ".log"
                })
                .unwrap_or(false);
        if matched {
            return Some(name.len() - i);
        }
    }
    None
}

/// keptByPruneKeepSet — "<id>" or "<id>.<anything>" is protected.
pub(crate) fn kept_by_keep_set(name: &str, keep: &[String]) -> bool {
    keep.iter()
        .any(|id| name == id || name.starts_with(&format!("{id}.")))
}

/// readPruneKeepSet — strict state read + non-capped/corrupt cell stems.
pub(crate) fn read_prune_keep_set(root: &Path) -> Result<Vec<String>, Err2> {
    let state = read_state_strict(root)?;
    let workers = match state.get("workers") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(a)) => a.clone(),
        Some(_) => {
            return Err(Err2::Msg(
                "worker prune: state.workers is not an array — refusing to prune against a malformed keep set (a destructive verb fails closed). FIX: repair .bee/state.json via the bee state worker verbs first.".to_string(),
            ));
        }
    };
    let mut keep: Vec<String> = Vec::new();
    let mut push_unique = |s: String| {
        if !keep.contains(&s) {
            keep.push(s);
        }
    };
    for w in &workers {
        if !truthy(w) {
            continue;
        }
        match jget(w, "cell") {
            None | Some(Value::Null) => {}
            Some(cell) => push_unique(js_disp(cell)),
        }
    }
    let cells_dir = root.join(".bee").join("cells");
    if cells_dir.exists() {
        let entries = std::fs::read_dir(&cells_dir).map_err(|_| Err2::Ex)?;
        for entry in entries.flatten() {
            let file = entry.file_name().to_string_lossy().into_owned();
            let Some(stem) = file.strip_suffix(".json") else { continue };
            let capped = match std::fs::read(cells_dir.join(&file)) {
                Err(_) => false, // JSON.parse throws → cell null → keep
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes).into_owned();
                    match parse_json_v8(&text).map_err(Err2::from)? {
                        ParsedJson::Unparseable => false,
                        ParsedJson::Parsed(v) => {
                            truthy(&v)
                                && matches!(jget(&v, "status"), Some(Value::String(s)) if s == "capped")
                        }
                    }
                }
            };
            if !capped {
                push_unique(stem.to_string());
            }
        }
    }
    Ok(keep)
}

pub(crate) fn run_worker_prune(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["dry-run"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "dry-run") {
        return None;
    }
    let ctx = match go("state worker prune", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let dry_run = flags.get("dry-run").is_some();
        let workers_dir = ctx.root.join(".bee").join("workers");
        let keep = match read_prune_keep_set(&ctx.root) {
            Ok(k) => k,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let mut candidates: Vec<String> = Vec::new();
        let mut kept: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&workers_dir) {
            for entry in entries.flatten() {
                let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
                if !is_file {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                let Some(suffix_len) = worker_transient_suffix_len(&name) else { continue };
                if name.len() == suffix_len {
                    continue; // empty stem is not a transient
                }
                if kept_by_keep_set(&name, &keep) {
                    kept.push(name);
                    continue;
                }
                candidates.push(name);
            }
        }
        let mut pruned: Vec<String> = Vec::new();
        if dry_run {
            pruned.extend(candidates);
        } else if !candidates.is_empty() {
            // C1: re-read the keep set immediately before the destructive loop.
            let keep2 = match read_prune_keep_set(&ctx.root) {
                Ok(k) => k,
                Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
                Err(Err2::Ex) => return Err(Err2::Ex),
            };
            for name in candidates {
                if kept_by_keep_set(&name, &keep2) {
                    kept.push(name);
                    continue;
                }
                let path = workers_dir.join(&name);
                if let Err(e) = std::fs::remove_file(&path) {
                    // fs.rmSync throws — reconstruct the Node errno message for
                    // the two realistic classes (documented approximation).
                    let msg = match e.kind() {
                        std::io::ErrorKind::NotFound => format!(
                            "ENOENT: no such file or directory, rm '{}'",
                            path.display()
                        ),
                        _ => format!("EPERM: operation not permitted, rm '{}'", path.display()),
                    };
                    return Ok(Out::Thrown(msg));
                }
                pruned.push(name);
            }
        }
        js_default_sort(&mut pruned);
        js_default_sort(&mut kept);
        let verb = if dry_run { "Would prune" } else { "Pruned" };
        let text = format!(
            "{verb} {} worker transient(s) from .bee/workers/ (kept {} still-active).",
            pruned.len(),
            kept.len()
        );
        Ok(Out::Emit(json!({"dry_run": dry_run, "pruned": pruned, "kept": kept}), text, 0))
    })();
    finish(&ctx, out)
}

// ─── state scribing-run ────────────────────────────────────────────────────

pub(crate) fn run_scribing_run(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["feature", "areas", "next-action", "lane", "no-lane", "show"]) {
        return None;
    }
    for b in ["no-lane", "show"] {
        if !bool_flag_ok(&flags, b) {
            return None;
        }
    }
    let ctx = match go("state scribing-run", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        // sqs-b3: --show is a READ-ONLY query mode, above every write-side check.
        if matches!(flags.get("show"), Some(FlagV::Present)) {
            let show_feature = flag_string(&flags, "feature");
            let stamp_ms: Option<f64> = match &show_feature {
                Some(f) => {
                    let state = read_state_peek(&ctx.root)?;
                    best_scribing_stamp_ms(&ctx.root, f, &state)?
                }
                None => {
                    let mut best: Option<f64> = None;
                    for entry in read_scribing_ledger(&ctx.root)? {
                        if !truthy(&entry) {
                            continue;
                        }
                        if let Some(ms) = date_parse_val(jget(&entry, "ts"))? {
                            if best.map(|b| ms > b).unwrap_or(true) {
                                best = Some(ms);
                            }
                        }
                    }
                    best
                }
            };
            let stamp_iso = match stamp_ms {
                Some(ms) => Some(iso_from_ms(ms)?),
                None => None,
            };
            let feature_v = show_feature.as_ref().map(|f| json!(f)).unwrap_or(Value::Null);
            let for_note = show_feature
                .as_ref()
                .map(|f| format!(" for \"{f}\""))
                .unwrap_or_default();
            let text = match &stamp_iso {
                Some(iso) => format!("Last scribing run{for_note}: {iso}"),
                None => format!("No scribing run recorded{for_note}."),
            };
            let stamp_v = stamp_iso.map(|s| json!(s)).unwrap_or(Value::Null);
            return Ok(Out::Emit(json!({"feature": feature_v, "stamp": stamp_v}), text, 0));
        }
        // write path
        let values = match require_flags(
            &flags,
            &[("feature", None), ("areas", None), ("next-action", None)],
            EXAMPLE_SCRIBING,
        ) {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let (feature, areas_raw, next_action) =
            (values[0].clone(), values[1].clone(), values[2].clone());
        let areas: Vec<Value> = split_list(&areas_raw).into_iter().map(|s| json!(s)).collect();
        let at = now_iso();
        let date = at[..10].to_string();
        let (lane_feature, no_lane) = match mutation_lane_selector(&flags, "scribing-run") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let scope = resolve_mutation_lock_scope(&ctx.root, lane_feature.as_deref(), no_lane)?;
        let workflows = list_workflows(&ctx.root)?;
        let locks = acquire_mutation_locks(&ctx.root, &scope, &workflows)?;
        let mut target =
            resolve_mutation_target(&ctx.root, lane_feature.as_deref(), "scribing-run", no_lane)?;
        let lane_note = target.lane_note();
        let is_lane = target.lane().is_some();
        let active_feature_at_call = target.record().get("feature").cloned().unwrap_or(Value::Null);
        // tst-1: only a call that ACTUALLY produces a phase transition on the
        // record it targets passes the D3 door — a lane call always does; a
        // default-record call only when it stamps its OWN active feature (or
        // none). A mismatch is the si-1 ledger-only repair path.
        let stamped_active = is_lane
            || !truthy(&active_feature_at_call)
            || opt_strict_eq(
                Some(&active_feature_at_call),
                Some(&Value::String(feature.clone())),
            );
        if stamped_active {
            if let Some(reason) = check_scribing_run_phase(target.record().get("phase")) {
                return Ok(Out::Thrown(reason));
            }
            {
                let state = target.record_mut();
                let mut run = Map::new();
                run.insert("feature".into(), json!(feature));
                run.insert("date".into(), json!(date));
                run.insert("at".into(), json!(at));
                run.insert("areas_synced".into(), Value::Array(areas.clone()));
                run.insert("next_action".into(), json!(next_action));
                state.insert("last_scribing_run".into(), Value::Object(run));
                state.insert("phase".into(), json!("compounding"));
                state.insert("next_action".into(), json!(next_action));
            }
            let record = target.record().clone();
            write_through_projection(&ctx.root, &target, &record, &[])?;
        }
        let record = target.record().clone();
        drop(locks);
        // si-1: the durable ledger append — ALWAYS, even on the repair path.
        // Fail-open; the Node warning embeds a Node error message, not replicated.
        let _ = append_jsonl(
            &scribing_ledger_path(&ctx.root),
            &json!({"ts": at, "feature": feature, "areas": areas}),
        );
        let repair_note = if stamped_active {
            String::new()
        } else {
            format!(
                " \u{2014} recorded in the durable ledger only: the default record tracks feature \"{}\", not \"{feature}\", so its phase/last_scribing_run were left untouched (repair path for an orphaned feature; `bee status --json`'s scribing_debt.orphaned names it).",
                js_disp(&active_feature_at_call)
            )
        };
        let text = format!("Recorded scribing run for \"{feature}\" at {at}.{lane_note}{repair_note}");
        Ok(Out::Emit(Value::Object(record), text, 0))
    })();
    finish(&ctx, out)
}

// ─── state compounding-run ─────────────────────────────────────────────────

pub(crate) fn run_compounding_run(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["feature", "learnings", "next-action", "lane", "no-lane"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "no-lane") {
        return None;
    }
    let ctx = match go("state compounding-run", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let values = match require_flags(
            &flags,
            &[("feature", None), ("learnings", None)],
            EXAMPLE_COMPOUNDING,
        ) {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let (feature, learnings) = (values[0].clone(), values[1].clone());
        let next_action = flag_string(&flags, "next-action");
        let at = now_iso();
        let date = at[..10].to_string();
        let (lane_feature, no_lane) = match mutation_lane_selector(&flags, "compounding-run") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let scope = resolve_mutation_lock_scope(&ctx.root, lane_feature.as_deref(), no_lane)?;
        let workflows = list_workflows(&ctx.root)?;
        let locks = acquire_mutation_locks(&ctx.root, &scope, &workflows)?;
        let mut target = resolve_mutation_target(
            &ctx.root,
            lane_feature.as_deref(),
            "compounding-run",
            no_lane,
        )?;
        let lane_note = target.lane_note();
        if let Some(reason) = check_compounding_run_phase(target.record().get("phase")) {
            return Ok(Out::Thrown(reason));
        }
        {
            let state = target.record_mut();
            let mut run = Map::new();
            run.insert("feature".into(), json!(feature));
            run.insert("date".into(), json!(date));
            run.insert("at".into(), json!(at));
            run.insert("learnings".into(), json!(learnings));
            run.insert(
                "next_action".into(),
                next_action.as_ref().map(|n| json!(n)).unwrap_or(Value::Null),
            );
            state.insert("last_compounding_run".into(), Value::Object(run));
            if let Some(n) = &next_action {
                state.insert("next_action".into(), json!(n));
            }
        }
        let record = target.record().clone();
        write_through_projection(&ctx.root, &target, &record, &[])?;
        drop(locks);
        let text = format!("Recorded compounding run for \"{feature}\" at {at}.{lane_note}");
        Ok(Out::Emit(Value::Object(record), text, 0))
    })();
    finish(&ctx, out)
}
