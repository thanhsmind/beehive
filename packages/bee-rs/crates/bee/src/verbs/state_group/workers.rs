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

/// D4 (store `97ce5225`): the ONE shape check both `worker add` and `worker
/// update` apply to the value a worker row carries. `role` is the sole model
/// selector now, so that value is the ROLE the worker's dispatch resolved,
/// never a cost tier — the closed `extraction | generation | ceiling` enum
/// that used to gate it is retired. What is left is presence and shape (a
/// non-blank name), never membership, exactly as `addCell`'s required-`role`
/// refusal reads it (`verbs/cells/validate.rs`, D7 store `4eaf1b71`). One
/// function rather than a copy per verb: two hand-maintained copies of one
/// rule is the drift D1 exists to remove.
fn worker_role_value(verb: &str, role: &str) -> Result<Value, Err2> {
    if role.trim().is_empty() {
        return Err(Err2::Msg(format!(
            "{verb}: invalid role \"{role}\" under --tier — the value records the ROLE this worker's dispatch resolved, and must be a non-empty name (store 97ce5225: role is the sole model selector; the closed extraction/generation/ceiling enum is retired; the flag and the persisted key keep the historical spelling `tier` on purpose). FIX: pass the cell's own role, e.g. --tier code."
        )));
    }
    Ok(json!(role))
}

/// The shared record-push body for `state worker add` — the exact shape
/// (`{nickname, cell, tier, status}`, absent role/status writing `null`)
/// that `worker add`'s own CLI door builds. Any OTHER native
/// door that needs to register a worker (dispatch prepare --claim, dp-r1)
/// calls this too, through [`worker_mutate`], rather than re-deriving the
/// record shape — forking it would let the two callers' records drift.
///
/// The value is validated by [`worker_role_value`] — shape, never
/// membership. The persisted KEY stays `tier` deliberately. Nothing in the tree reads
/// `workers[].tier` — the cap door's registered-worker check
/// (`verbs/cells/handlers_close.rs`'s `registered_worker_for_cell`) matches
/// on `nickname` + `cell` alone — so a rename would buy no behavior, while
/// every worker row already on disk (including the in-flight ones this very
/// change is dispatched under) carries `tier`. A split key would leave the
/// registry half one spelling and half the other for no gain.
///
/// Upserts by the `(nickname, cell)` pair: the SAME pair refreshes the live
/// record in place (role/status take the freshly passed values) rather than
/// appending a second row — `run_worker_add` and `register_worker_for_cell`
/// both inherit this automatically, since neither re-derives the record
/// shape. The SAME nickname against a DIFFERENT cell still appends: one
/// worker legitimately holding several cells is not a duplicate.
pub(crate) fn push_worker_record(
    workers: &mut Vec<Value>,
    nickname: &str,
    cell: &str,
    role: Option<&str>,
    status: Option<&str>,
) -> Result<String, Err2> {
    let role_val: Value = match role {
        None => Value::Null,
        Some(r) => worker_role_value("worker add", r)?,
    };
    let status_val: Value = match status {
        None => Value::Null,
        Some(s) => json!(s),
    };
    let record = json!({"nickname": nickname, "cell": cell, "tier": role_val, "status": status_val});
    let existing = workers.iter_mut().find(|w| {
        truthy(w)
            && opt_strict_eq(jget(w, "nickname"), Some(&Value::String(nickname.to_string())))
            && opt_strict_eq(jget(w, "cell"), Some(&Value::String(cell.to_string())))
    });
    match existing {
        Some(slot) => *slot = record,
        None => workers.push(record),
    }
    Ok(format!("Added worker \"{nickname}\" (cell {cell})."))
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
        let role = flag_string(&flags, "tier");
        let status = flag_string(&flags, "status");
        push_worker_record(workers, &nickname, &cell, role.as_deref(), status.as_deref())
    }));
    finish(&ctx, out)
}

/// dp-r1: register the worker a successful `dispatch prepare --claim` just
/// claimed the cell for — the SAME record `bee state worker add --nickname
/// <w> --cell <id> --tier <t> --status running` writes, through the SAME
/// [`worker_mutate`] lock+read+write frame and the SAME [`push_worker_record`]
/// body `run_worker_add` uses, never a second copy of the record shape.
///
/// Every failure here (a blank `role`, a lock/read/write error) is
/// returned as `Err(message)` rather than propagated as a delegate or a
/// thrown command failure — the caller's claim already stands and must
/// never be unwound over a registration problem; it only needs to know the
/// registration did not happen and why.
pub(crate) fn register_worker_for_cell(
    root: &Path,
    nickname: &str,
    cell: &str,
    role: Option<&str>,
) -> Result<(), String> {
    match worker_mutate(root, |workers| {
        push_worker_record(workers, nickname, cell, role, Some("running"))
    }) {
        Ok(Out::Thrown(m)) => Err(m),
        Ok(_) => Ok(()),
        Err(Err2::Msg(m)) => Err(m),
        Err(Err2::Ex) => {
            Err("registering the claimed worker hit an unsupported store shape".to_string())
        }
    }
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
        // D4 (store 97ce5225): the value is the ROLE the dispatch resolved,
        // so the closed three-value enum is gone — shape only, never
        // membership. The persisted key stays `tier` for the reason
        // `push_worker_record` documents.
        if let Some(r) = flag_string(&flags, "tier") {
            worker.insert("tier".into(), worker_role_value("worker update", &r)?);
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

// ─── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    // ── rph-4 item 1: (nickname, cell) upsert ──────────────────────────────

    /// The SAME `(nickname, cell)` pair upserts: a second `push_worker_record`
    /// call for the identical pair refreshes role/status on the live record
    /// in place — never a second, duplicate row.
    #[test]
    fn push_worker_record_upserts_the_same_nickname_cell_pair() {
        let mut workers: Vec<Value> = Vec::new();
        push_worker_record(&mut workers, "w1", "c1", Some("code"), Some("running")).unwrap();
        push_worker_record(&mut workers, "w1", "c1", Some("review"), Some("capped")).unwrap();
        assert_eq!(workers.len(), 1, "the same (nickname, cell) pair must never duplicate");
        assert_eq!(workers[0].get("tier"), Some(&json!("review")));
        assert_eq!(workers[0].get("status"), Some(&json!("capped")));
    }

    /// The SAME nickname against a DIFFERENT cell is a legitimate second row
    /// — one worker really can hold several cells at once.
    #[test]
    fn push_worker_record_appends_for_the_same_nickname_a_different_cell() {
        let mut workers: Vec<Value> = Vec::new();
        push_worker_record(&mut workers, "w1", "c1", Some("code"), Some("running")).unwrap();
        push_worker_record(&mut workers, "w1", "c2", Some("code"), Some("running")).unwrap();
        assert_eq!(workers.len(), 2, "a different cell for the same worker is not a duplicate");
        assert_eq!(workers[0].get("cell"), Some(&json!("c1")));
        assert_eq!(workers[1].get("cell"), Some(&json!("c2")));
    }

    /// `run_worker_add` and `register_worker_for_cell` inherit the upsert
    /// automatically because both funnel through `push_worker_record` inside
    /// the SAME `worker_mutate` lock+read+write frame: a re-registration of
    /// the still-live `(nickname, cell)` pair mutates the one record on
    /// disk, it never appends a stale duplicate beside it.
    #[test]
    fn worker_mutate_re_registration_mutates_the_live_record_on_disk() {
        let tmp = tmp_root();
        let root = tmp.path();
        worker_mutate(root, |workers| {
            push_worker_record(workers, "w1", "c1", Some("code"), Some("running"))
        })
        .unwrap();
        worker_mutate(root, |workers| {
            push_worker_record(workers, "w1", "c1", Some("code"), Some("capped"))
        })
        .unwrap();
        let state = read_state_strict(root).unwrap();
        let workers = state.get("workers").unwrap().as_array().unwrap();
        assert_eq!(
            workers.len(),
            1,
            "re-registering the live (nickname, cell) pair must not duplicate it: {workers:?}"
        );
        assert_eq!(workers[0].get("status"), Some(&json!("capped")));
    }

    // ── D4 (store 97ce5225): the closed tier enum is retired ──────────────

    /// The worker registry records the ROLE a dispatch resolved, so no closed
    /// three-value list constrains it any more. Every name here — the job
    /// names D8 recommends, a name bee ships no config for, and the two cost
    /// words the retired enum used to be — is accepted identically: the check
    /// asks "is this a name", never "is this one of three words".
    #[test]
    fn a_worker_row_records_any_non_blank_role_name() {
        for role in ["code", "read", "test", "docs", "review", "design", "generation", "ceiling"] {
            let mut workers: Vec<Value> = Vec::new();
            push_worker_record(&mut workers, "w1", "c1", Some(role), Some("running"))
                .unwrap_or_else(|_| panic!("role \"{role}\" must be recorded, not refused"));
            assert_eq!(workers[0].get("tier"), Some(&json!(role)), "role {role}");
        }
    }

    /// Shape is still checked, on BOTH doors: presence-and-shape replaced
    /// membership, it did not replace validation. A blank or whitespace-only
    /// value names no job at all, so it is refused with the FIX line naming
    /// the verb the caller actually typed.
    #[test]
    fn a_blank_role_is_refused_by_both_worker_doors() {
        for blank in ["", " ", "\t", "   \n "] {
            let Err(Err2::Msg(m)) = worker_role_value("worker add", blank) else {
                panic!("a blank role must refuse, not record: {blank:?}");
            };
            assert!(m.starts_with("worker add: invalid role"), "{m}");
            assert!(m.contains("FIX:"), "the refusal must name its remedy: {m}");
            let Err(Err2::Msg(m)) = worker_role_value("worker update", blank) else {
                panic!("a blank role must refuse on update too: {blank:?}");
            };
            assert!(m.starts_with("worker update: invalid role"), "{m}");
        }
        let mut workers: Vec<Value> = Vec::new();
        assert!(
            push_worker_record(&mut workers, "w1", "c1", Some(""), Some("running")).is_err(),
            "push_worker_record must inherit the shape refusal"
        );
        assert!(workers.is_empty(), "a refused row must never be pushed: {workers:?}");
    }

    /// State transition (test matrix row 5): a worker row written BEFORE this
    /// change — a live one, carrying `tier: "generation"` from the retired
    /// enum — must still satisfy the cap door's registered-worker check, or
    /// this change would refuse correct caps for work already in flight. The
    /// check matches on `nickname` + `cell` and reads the role value not at
    /// all, which is why the persisted key was left alone.
    #[test]
    fn a_worker_row_written_before_this_change_still_satisfies_the_cap_door() {
        let tmp = tmp_root();
        let root = tmp.path();
        worker_mutate(root, |workers| {
            workers.push(json!({
                "nickname": "old-w", "cell": "c1", "tier": "generation", "status": "running"
            }));
            Ok("seeded a pre-change row".to_string())
        })
        .unwrap();
        worker_mutate(root, |workers| {
            push_worker_record(workers, "new-w", "c2", Some("code"), Some("running"))
        })
        .unwrap();

        assert!(
            crate::verbs::cells::registered_worker_for_cell(root, "c1", Some("old-w")).unwrap(),
            "the pre-change row must keep proving its worker"
        );
        assert!(
            crate::verbs::cells::registered_worker_for_cell(root, "c2", Some("new-w")).unwrap(),
            "a row written after the change proves its worker the same way"
        );
        assert!(
            !crate::verbs::cells::registered_worker_for_cell(root, "c2", Some("old-w")).unwrap(),
            "the check still fails closed for a worker that holds a different cell"
        );
    }
}
