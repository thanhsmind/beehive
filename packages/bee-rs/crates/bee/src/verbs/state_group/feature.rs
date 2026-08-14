// `state rebuild-projections` and `state start-feature`
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

// ─── state rebuild-projections (R6 coverage debt) ──────────────────────────

/// bee.mjs withStoreLocks as RAII: the array IS the acquisition order, and the
/// unwind releases innermost-first exactly like the nested `withStoreLock`
/// closures it replaces (Vec's own Drop would release outermost-first).
pub(crate) struct LockStack(pub(crate) Vec<LockGuard>);

impl Drop for LockStack {
    fn drop(&mut self) {
        while self.0.pop().is_some() {}
    }
}

/// state-projection.mjs rebuildAllProjections(root). Returns the `{state,
/// handoff, reservations, lanes}` literal in its own key order.
pub(crate) fn rebuild_all_projections(root: &Path) -> R2<Value> {
    let state = rebuild_state_projection_reporting(root)?;
    let handoff = rebuild_handoff_projection_reporting(root)?;
    let count = rebuild_reservations_projection(root)?;

    let mut state_out = Map::new();
    state_out.insert("authoritative".into(), json!(state.authoritative));
    state_out.insert("source".into(), state.source);
    state_out.insert("state".into(), Value::Object(state.record));

    let mut handoff_out = Map::new();
    handoff_out.insert("authoritative".into(), json!(handoff.authoritative));
    handoff_out.insert("source".into(), handoff.source);

    // rebuildReservationsProjection is never gated on workflow records — see
    // its own doc comment; `authoritative` is an unconditional literal `true`.
    let mut reservations_out = Map::new();
    reservations_out.insert("authoritative".into(), json!(true));
    reservations_out.insert("count".into(), json!(count));

    let workflows = list_workflows(root)?;
    let mut lanes = Vec::new();
    for wf in workflows.iter().filter(is_active_workflow) {
        let feature = js_disp_opt(wf.get("feature"));
        let lane = rebuild_lane_projection_reporting(root, &feature)?;
        let mut row = Map::new();
        row.insert("authoritative".into(), json!(lane.authoritative));
        row.insert("source".into(), lane.source);
        row.insert(
            "lane".into(),
            lane.record.map(Value::Object).unwrap_or(Value::Null),
        );
        lanes.push(Value::Object(row));
    }

    let mut result = Map::new();
    result.insert("state".into(), Value::Object(state_out));
    result.insert("handoff".into(), Value::Object(handoff_out));
    result.insert("reservations".into(), Value::Object(reservations_out));
    result.insert("lanes".into(), Value::Array(lanes));
    Ok(Value::Object(result))
}

/// `wf.status === 'active'` — the filter both the lane-lock peek and
/// rebuildAllProjections's own lane pass use.
pub(crate) fn is_active_workflow(wf: &&Map<String, Value>) -> bool {
    wf.get("status").unwrap_or(&Value::Null) == &json!("active")
}

/// bee.mjs handleStateRebuildProjections. The ONE seam that holds more than one
/// projection lock: 'state' then every active workflow's `lane:<feature>`, lane
/// names SORTED and de-duplicated so two concurrent rebuilds acquire in the same
/// sequence and can never deadlock. The lane set is peeked immediately before
/// the acquire, exactly as the .mjs does — rebuildAllProjections re-lists inside.
pub(crate) fn run_rebuild_projections(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match go("state rebuild-projections", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let workflows = list_workflows(&ctx.root)?;
        // DELEGATION GUARD (pre-lock, pre-write): Node's lane pass maps over
        // every ACTIVE record — including one whose `feature` is absent or not
        // a plain non-empty string, where rebuildLaneProjection(root, undefined)
        // reaches requireLaneFeature/String(feature) with bytes this port does
        // not model. No bee-created record has that shape; delegate rather than
        // guess.
        for wf in workflows.iter().filter(is_active_workflow) {
            match wf.get("feature") {
                Some(Value::String(s)) if !s.is_empty() => {}
                _ => return Err(Err2::Ex),
            }
        }
        let mut lane_locks: Vec<String> = workflows
            .iter()
            .filter(is_active_workflow)
            .filter(|wf| wf.get("feature").is_some_and(truthy))
            .map(|wf| lane_lock_name(&js_disp_opt(wf.get("feature"))))
            .collect();
        js_default_sort(&mut lane_locks); // `.sort()` then `new Set(...)` — sorted, so
        lane_locks.dedup(); // duplicates are adjacent and Set order is preserved
        let mut names: Vec<String> = vec!["state".to_string()];
        names.extend(lane_locks);

        let mut stack = LockStack(Vec::new());
        for name in &names {
            stack.0.push(acquire_named_lock(&ctx.root, name)?);
        }
        let result = rebuild_all_projections(&ctx.root)?;
        drop(stack);

        let lane_count = match jget(&result, "lanes") {
            Some(Value::Array(rows)) => rows
                .iter()
                .filter(|r| jget(r, "authoritative").is_some_and(truthy))
                .count(),
            _ => 0,
        };
        let state_authoritative = jget(&result, "state")
            .and_then(|s| jget(s, "authoritative"))
            .is_some_and(truthy);
        let state_note = if state_authoritative {
            format!(
                "rebuilt .bee/state.json from workflow {}",
                js_disp_opt(jget(&result, "state").and_then(|s| jget(s, "source")))
            )
        } else {
            "state.json left untouched (no workflow records yet, or a live non-idle default feature \u{2014} see D1 field scoping)".to_string()
        };
        let reservations_note = format!(
            "reservations.json rebuilt ({} active)",
            js_disp_opt(jget(&result, "reservations").and_then(|r| jget(r, "count")))
        );
        let text =
            format!("{state_note}; {lane_count} lane projection(s) rebuilt; {reservations_note}.");
        Ok(Out::Emit(result, text, 0))
    })();
    finish(&ctx, out)
}

// ─── state start-feature ───────────────────────────────────────────────────
//
// lib/state.mjs startFeature + startLane + seedLegacyWorkflows +
// checkNoLiveWorkflowForFeature + checkNoSameFeatureClaimedCells +
// ensureWorkflowRecordForFeature + closeWorkflowsForFeature +
// applyWritePolicy, and bee.mjs's handleStateStartFeature around them.
//
// The three transactions run in Node's order, NONE nested inside another's
// lock (D1/C4, multisession-native-6):
//   (1) seedLegacyWorkflows — `workflow:<id>` per created record
//   (2) applyWritePolicy    — `workspace:<id>` (+ 'worktree-admin' on the
//                             consented isolate-create), then the legacy
//                             read-check-write body under 'state' (or the
//                             lane record under 'state', same hold)
//   (3) ensureWorkflowRecordForFeature, then closeWorkflowsForFeature —
//       `workflow:<id>`, always OUTSIDE 'state'
// then bee.mjs's own projection rebuild under 'state' | `lane:<feature>`
// (splpr-3: exactly one lock, for the record IT writes).
//
// WHAT UNBLOCKED THIS. Both recorded blockers have dissolved:
//   * "applyWritePolicy WRITES into workspace-store.mjs before deciding, and
//     none of workspace-store.mjs is ported" — verbs/workspace_store.rs now
//     carries registerWorkspace / attachWorkspace / decideOwnership /
//     applyOwnershipTakeover with Node's `workspace:<id>` lock name and
//     Node's key ORDER, so registerWorkspace/attachWorkspace here call THE
//     port. The "nothing after that first write can fall back" property is
//     still true and still governs: every delegation gate below is decided in
//     a read-only PREFLIGHT, before seedLegacyWorkflows' first write.
//   * "ensureWorkflowRecordForFeature -> createWorkflow has no Rust
//     implementation" — verbs/workflow_store.rs::create_workflow does, and
//     `close_workflows_for_feature` is the same close-by-feature loop
//     `state workflows close --all-but-active` already drives here.
//
// DELEGATED, by design, decided BEFORE any lock or write:
//   * a control root that is not this store root. Node re-roots
//     seedLegacyWorkflows / checkNoLiveWorkflowForFeature /
//     ensureWorkflowRecordForFeature / closeWorkflowsForFeature /
//     applyWritePolicy through controlRootFor(root); every other verb in this
//     module reads the control plane as if it were `root`, so rather than
//     silently inherit that assumption on a verb that WRITES workflow records,
//     the mismatch is checked and delegated. (The prelude has already sent
//     every linked worktree to Node, so this only fires on the exotic
//     `.bee/onboarding.json`-below-the-git-root layout.)
//   * every corrupt/exotic READ the run would make (config, workflow records,
//     lanes, cells, reservations, sessions, the handoff, the grants registry,
//     the workspace record) — classified by `preflight` up front, so a run
//     that delegates has emitted zero bytes, taken zero locks and written
//     nothing.
//   * `--phase` naming a non-enum value is NOT delegated: the refusal is
//     deterministic and native (it is thrown before seeding, zero mutation).
//
// The one accepted residual, identical in kind to verbs/workspace_store.rs's
// and verbs/worktree.rs's: an fs WRITE that fails after the preflight still
// delegates late. Every step to that point is idempotent by construction
// (ensureWorkflowRecordForFeature is idempotent by feature, registerWorkspace
// is idempotent, and a legacy write that never landed leaves the record
// untouched), so the Node re-run reproduces the same answer over the same
// store.

/// state.mjs listAllCellsForStart — `.bee/cells/*.json`, objects only. A
/// corrupt cell WARNS and is skipped: readJson hands back `null`, which
/// Node's `!cell` guard drops from the list. Same list, same order.
pub(crate) fn list_all_cells_for_start(root: &Path) -> Ex<Vec<Map<String, Value>>> {
    let Ok(entries) = std::fs::read_dir(root.join(".bee").join("cells")) else {
        return Ok(Vec::new());
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        match entry.file_name().to_str() {
            Some(n) => names.push(n.to_string()),
            None => return Err(Exotic),
        }
    }
    names.sort(); // readdirSync order is the filesystem's; only the SET is read
    let mut cells = Vec::new();
    for name in names {
        if !name.ends_with(".json") {
            continue;
        }
        let cell_file = root.join(".bee").join("cells").join(&name);
        let parsed = match read_json(&cell_file) {
            ReadJson::Missing => continue,
            ReadJson::Corrupt => {
                warn_corrupt_json(&cell_file);
                continue;
            }
            ReadJson::Parsed(v) => js_numberify(&v)?,
        };
        // `!cell || typeof cell !== 'object' || Array.isArray(cell)`
        if let Value::Object(m) = parsed {
            cells.push(m);
        }
    }
    Ok(cells)
}

pub(crate) fn cell_field(cell: &Map<String, Value>, key: &str) -> Value {
    cell.get(key).cloned().unwrap_or(Value::Null)
}

pub(crate) fn cell_id_disp(cell: &Map<String, Value>) -> String {
    js_disp_opt(cell.get("id"))
}

/// state.mjs checkNoSameFeatureClaimedCells.
pub(crate) fn check_no_same_feature_claimed_cells(
    feature: &str,
    cells: &[Map<String, Value>],
) -> Option<String> {
    let claimed: Vec<String> = cells
        .iter()
        .filter(|c| {
            &cell_field(c, "feature") == &json!(feature)
                && &cell_field(c, "status") == &json!("claimed")
        })
        .map(cell_id_disp)
        .collect();
    if claimed.is_empty() {
        return None;
    }
    Some(format!(
        "startFeature: refused — feature \"{feature}\" already has claimed cell(s): {}. FIX: cap or drop them first (bee cells cap / bee cells drop).",
        claimed.join(", ")
    ))
}

/// state.mjs checkNoLiveWorkflowForFeature — read-only, runs before any write.
pub(crate) fn check_no_live_workflow_for_feature(
    workflows: &[Map<String, Value>],
    feature: &str,
) -> Option<String> {
    let conflict = workflows.iter().find(|wf| {
        &cell_field(wf, "feature") == &json!(feature)
            && &cell_field(wf, "status") != &json!("closed")
    })?;
    Some(format!(
        "startFeature: refused — a live workflow already exists for feature \"{feature}\" (workflow {}, phase \"{}\", status \"{}\"). FIX: close or resolve that workflow before starting a new one for the same feature.",
        js_disp_opt(conflict.get("id")),
        js_disp_opt(conflict.get("phase")),
        js_disp_opt(conflict.get("status"))
    ))
}

/// state.mjs legacyGatesToWorkflowGates, extended by D3: a legacy
/// `approved_gates` boolean backfills `state` alongside `approved` instead of
/// dropping it, so the seeded record never desyncs on its first read.
/// `actor`/`at`/`reason`/`bypass_level` are left null — a legacy boolean
/// carries no trace of who approved it or why.
pub(crate) fn legacy_gates_to_workflow_gates(approved: Option<&Value>) -> Value {
    let mut gates = Map::new();
    for name in GATE_NAMES {
        let approved_flag = approved
            .and_then(|g| jget(g, name))
            .map(|v| v == &Value::Bool(true))
            .unwrap_or(false);
        gates.insert(
            name.to_string(),
            json!({
                "approved": approved_flag,
                "approved_for_plan_rev": Value::Null,
                "state": if approved_flag { "approved" } else { "pending" },
                "actor": Value::Null,
                "at": Value::Null,
                "reason": Value::Null,
                "bypass_level": Value::Null,
            }),
        );
    }
    Value::Object(gates)
}

/// The `{record, created}` answer ensureWorkflowRecordForFeature gives. Only
/// the record's summary/next_action are consumed downstream, so `created` is
/// not modelled (no caller reads it).
pub(crate) fn ensure_workflow_record_for_feature(
    control_root: &Path,
    feature: &str,
    phase: &str,
    mode: Option<&str>,
    summary: Option<&Value>,
    next_action: Option<&Value>,
    gates: Option<Value>,
) -> Result<(), Err2> {
    let feature_trimmed = js_trim(feature);
    if feature_trimmed.is_empty() {
        return Err(Err2::Msg(
            "ensureWorkflowRecordForFeature: a non-empty feature slug is required.".to_string(),
        ));
    }
    // Idempotent by FEATURE: a live record is returned untouched, never a
    // second record for one feature and never a silent overwrite.
    let live = list_workflows(control_root)?.into_iter().find(|wf| {
        &cell_field(wf, "feature") == &json!(feature_trimmed)
            && &cell_field(wf, "status") != &json!("closed")
    });
    if live.is_some() {
        return Ok(());
    }
    create_workflow(
        control_root,
        NewWorkflow {
            feature: Some(feature_trimmed),
            // `isKnownPhase(phase) ? phase : 'idle'`
            phase: Some(json!(if is_known_phase(phase) { phase } else { "idle" })),
            // `mode == null ? null : String(mode)`
            mode: Some(mode.map_or(Value::Null, |m| json!(m))),
            plan_rev: None,
            gates,
            // `typeof summary === 'string' ? summary : ''`
            summary: Some(match summary {
                Some(Value::String(s)) => json!(s),
                _ => json!(""),
            }),
            next_action: Some(match next_action {
                Some(Value::String(s)) => json!(s),
                _ => json!(""),
            }),
            status: Some("active"),
            id: None,
        },
    )
    .map(|_| ())
}

/// state.mjs closeWorkflowsForFeature(root, { keepFeature }) — close every
/// LIVE record whose feature differs from `keep`, each under its OWN
/// `workflow:<id>` lock, sequentially, never nested inside 'state'.
pub(crate) fn close_workflows_for_feature(control_root: &Path, keep: Option<&str>) -> Result<(), Err2> {
    let keep = keep.map(js_trim).filter(|s| !s.is_empty());
    for wf in list_workflows(control_root)? {
        if &cell_field(&wf, "status") == &json!("closed") {
            continue;
        }
        if let Some(keep) = keep {
            if &cell_field(&wf, "feature") == &json!(keep) {
                continue;
            }
        }
        let mut patch = Map::new();
        patch.insert("status".into(), json!("closed"));
        update_workflow(control_root, &wf_id(&wf), patch)?;
    }
    Ok(())
}

/// state.mjs seedLegacyWorkflows (C1) — materialize every ALREADY-live legacy
/// record into a workflow record, once per repo. Gated on
/// `listWorkflows(controlRoot).workflows.length === 0`, and each candidate goes
/// through the ONE creation seam so seeding and starting share one definition
/// of "already recorded".
pub(crate) fn seed_legacy_workflows(root: &Path, control_root: &Path) -> Result<(), Err2> {
    if !list_workflows(control_root)?.is_empty() {
        return Ok(()); // never re-seed once ANY workflow record exists
    }
    // (a) the legacy default record, read with the FAIL-OPEN readState.
    let legacy = read_state_peek(root)?;
    let feature = legacy.get("feature").cloned().unwrap_or(Value::Null);
    let phase = legacy.get("phase").cloned().unwrap_or(Value::Null);
    let gates_live = match legacy.get("approved_gates") {
        Some(Value::Object(g)) => g.values().any(|v| v == &Value::Bool(true)),
        Some(Value::Array(a)) => a.iter().any(|v| v == &Value::Bool(true)),
        _ => false,
    };
    let phase_live = truthy(&phase)
        && &phase != &json!("idle")
        && &phase != &json!("compounding-complete");
    let legacy_live = truthy(&feature) || phase_live || gates_live;
    if legacy_live && truthy(&feature) {
        ensure_workflow_record_for_feature(
            control_root,
            &js_disp_opt(Some(&feature)),
            &js_disp_opt(Some(&phase)),
            legacy.get("mode").filter(|v| !v.is_null()).map(|_| js_disp_opt(legacy.get("mode"))).as_deref(),
            legacy.get("summary"),
            legacy.get("next_action"),
            Some(legacy_gates_to_workflow_gates(legacy.get("approved_gates"))),
        )?;
    }

    // (b) every non-terminal lane.
    for lane in list_lanes(root)? {
        let lane_feature = cell_field(&lane, "feature");
        if !truthy(&lane_feature) {
            continue;
        }
        let lane_phase = cell_field(&lane, "phase");
        let lane_live = truthy(&lane_phase)
            && &lane_phase != &json!("idle")
            && &lane_phase != &json!("compounding-complete");
        if !lane_live {
            continue;
        }
        ensure_workflow_record_for_feature(
            control_root,
            &js_disp_opt(Some(&lane_feature)),
            &js_disp_opt(Some(&lane_phase)),
            lane.get("mode").filter(|v| !v.is_null()).map(|_| js_disp_opt(lane.get("mode"))).as_deref(),
            lane.get("summary"),
            lane.get("next_action"),
            Some(legacy_gates_to_workflow_gates(lane.get("approved_gates"))),
        )?;
    }
    Ok(())
}

/// claims.mjs activeWorkers — live-heartbeat sessions joined with their first
/// active claim, one row per session. Re-derived here from THIS module's own
/// list_session_records / heartbeat_stale / read_claim rather than widened out
/// of verbs/status_full.rs: those are read-only projections (no store mutation
/// to single-source), and status_full's copy is bound to that module's own
/// error type.
pub(crate) struct Worker {
    pub(crate) session_id: String,
    pub(crate) cell: Option<Value>,
}

/// claims.mjs isClaimExpired/isClaimActive.
pub(crate) fn is_claim_active(claim: &Value, now: f64) -> Ex<bool> {
    let ttl = match jget(claim, "ttl_seconds") {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        _ => return Ok(true), // `typeof ttl !== 'number'` -> never expires
    };
    if !ttl.is_finite() || ttl <= 0.0 {
        return Ok(true);
    }
    match date_parse_val(jget(claim, "claimed_at"))? {
        None => Ok(true),
        Some(at) => Ok(at + ttl * 1000.0 > now),
    }
}

pub(crate) fn claim_ids(root: &Path) -> Ex<Vec<String>> {
    let Ok(entries) = std::fs::read_dir(claims_dir(root)) else {
        return Ok(Vec::new());
    };
    let mut ids = Vec::new();
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            return Err(Exotic);
        };
        if let Some(stem) = name.strip_suffix(".json") {
            ids.push(stem.to_string());
        }
    }
    ids.sort();
    Ok(ids)
}

pub(crate) fn active_workers(root: &Path, exclude_session_id: Option<&str>) -> Ex<Vec<Worker>> {
    let exclude = exclude_session_id.map(js_trim).unwrap_or("");
    let now = now_ms();
    let mut live: Vec<Map<String, Value>> = Vec::new();
    for session in list_session_records(root)? {
        let id = js_disp_opt(session.get("id"));
        if id == exclude || heartbeat_stale(&session, now)? {
            continue;
        }
        live.push(session);
    }
    if live.is_empty() {
        return Ok(Vec::new());
    }
    // First active claim seen for a session wins (one row per worker).
    let mut cell_by_session: Vec<(String, Value)> = Vec::new();
    for id in claim_ids(root)? {
        let Some(claim) = read_claim(root, &id)? else { continue };
        let session = jget(&claim, "session").cloned().unwrap_or(Value::Null);
        if !truthy(&session) || !is_claim_active(&claim, now)? {
            continue;
        }
        let key = js_disp_opt(Some(&session));
        if !cell_by_session.iter().any(|(s, _)| *s == key) {
            cell_by_session.push((key, jget(&claim, "cell").cloned().unwrap_or(Value::Null)));
        }
    }
    Ok(live
        .into_iter()
        .map(|session| {
            let session_id = js_disp_opt(session.get("id"));
            let cell = cell_by_session
                .iter()
                .find(|(s, _)| *s == session_id)
                .map(|(_, c)| c.clone());
            Worker { session_id, cell }
        })
        .collect())
}

/// state.mjs listClaimHoldsForStart — ANOTHER session's active claim whose
/// claimed cell's files overlap a declared path.
pub(crate) struct ClaimHold {
    pub(crate) session: String,
    pub(crate) cell: String,
    pub(crate) path: String,
}

pub(crate) fn list_claim_holds_for_start(
    control_root: &Path,
    session_id: Option<&str>,
    cells: &[Map<String, Value>],
    declared: &[String],
) -> Ex<Vec<ClaimHold>> {
    let now = now_ms();
    let mut holds = Vec::new();
    for id in claim_ids(control_root)? {
        let Some(claim) = read_claim(control_root, &id)? else { continue };
        if !is_claim_active(&claim, now)? {
            continue;
        }
        let claim_session = jget(&claim, "session").cloned().unwrap_or(Value::Null);
        if let Some(sid) = session_id {
            if &claim_session == &json!(sid) {
                continue; // own holds never block
            }
        }
        let claim_cell = jget(&claim, "cell").cloned().unwrap_or(Value::Null);
        let cell = cells
            .iter()
            .find(|c| &cell_field(c, "id") == &claim_cell);
        let files: Vec<String> = match cell.map(|c| cell_field(c, "files")) {
            Some(Value::Array(a)) => a.iter().map(|f| js_disp_opt(Some(f))).collect(),
            _ => Vec::new(),
        };
        for file in files {
            if declared.iter().any(|d| paths_overlap(&file, d)) {
                holds.push(ClaimHold {
                    session: js_disp_opt(Some(&claim_session)),
                    cell: js_disp_opt(Some(&claim_cell)),
                    path: file,
                });
                break;
            }
        }
    }
    Ok(holds)
}

/// The reservation display `${r.agent}:${r.path}` both precondition messages
/// use.
pub(crate) fn resv_disp(r: &crate::verbs::reservations::Resv) -> String {
    format!("{}:{}", js_disp_opt(r.agent.as_ref()), r.path)
}
