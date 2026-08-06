// applyWritePolicy, startLane and the default legacy body
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

// ─── applyWritePolicy (multisession-native-20, D3) ─────────────────────────

/// The answer applyWritePolicy gives the caller. `Redirect` is the consented
/// isolate-create — `handleStateStartFeature` returns it directly and MUST NOT
/// perform its own write in `root`.
pub(crate) enum Policy {
    Proceed,
    Redirect { result: Map<String, Value>, text: String },
}

pub(crate) fn write_policy_isolate_one_liner(verb_hint: &str) -> String {
    let verb = if js_trim(verb_hint).is_empty() { "<verb>" } else { js_trim(verb_hint) };
    format!(
        "bee {verb} --isolate (creates a fresh worktree for this session), or set guards.auto_isolate to true in .bee/config.json to always auto-isolate on contention."
    )
}

/// isolateNoticeMarkerPath — `${sessionId}`.replace(/[\\/]/g,'_').
pub(crate) fn isolate_notice_marker_path(control_root: &Path, session_id: &str) -> PathBuf {
    let safe: String = session_id
        .chars()
        .map(|c| if c == '\\' || c == '/' { '_' } else { c })
        .collect();
    control_root
        .join(".bee")
        .join("runtime")
        .join("notices")
        .join("isolate")
        .join(format!("{safe}.json"))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_write_policy(
    root: &Path,
    control_root: &Path,
    config: &Map<String, Value>,
    session_id: Option<&str>,
    paths: &[String],
    isolate: bool,
    feature: &str,
    verb_hint: &str,
) -> Result<Policy, Err2> {
    // resolveWritePolicyMode(config)
    let configured = match config.get("guards").and_then(|g| jget(g, "write_policy")) {
        Some(Value::String(s)) => js_trim(s).to_string(),
        _ => String::new(),
    };
    if configured == "observe" {
        return Ok(Policy::Proceed);
    }

    if configured == "shared-disjoint" {
        let declared: Vec<&String> = paths.iter().filter(|p| !js_trim(p).is_empty()).collect();
        if declared.is_empty() {
            return Ok(Policy::Proceed);
        }
        let control_root_s = control_root.to_str().ok_or(Err2::Ex)?.to_string();
        let active = list_reservations(&control_root_s, true, now_ms())?;
        let missing: Vec<&&String> = declared
            .iter()
            .filter(|p| {
                !active.iter().any(|r| {
                    session_id.is_some_and(|sid| &r.session.clone().unwrap_or(Value::Null) == &json!(sid)) && !r.path.ends_with('*')
                        && paths_overlap(&r.path, p)
                })
            })
            .collect();
        if !missing.is_empty() {
            let list: Vec<&str> = missing.iter().map(|p| p.as_str()).collect();
            return Err(Err2::Msg(format!(
                "bee write-policy (shared-disjoint): no exact-path lease held for: {}. A broad/glob reservation never satisfies shared-disjoint — an exact-path lease is mandatory before write. FIX: bee reservations reserve --agent <worker> --cell <id> --path <path>{} for each path, then retry.",
                list.join(", "),
                match session_id {
                    Some(sid) => format!(" --session-id {sid}"),
                    None => String::new(),
                }
            )));
        }
        return Ok(Policy::Proceed);
    }

    // isolated (default). No session identity => nothing to own a workspace
    // with; `enforceIsolation` is always TRUE for this verb.
    let Some(session_id) = session_id.filter(|s| !s.is_empty()) else {
        return Ok(Policy::Proceed);
    };

    // resolveContext(root). This verb only ever runs from an ORDINARY
    // checkout whose control root IS this root (both gated in `run_start_feature`
    // before anything is read), so decideWorktreeStore can only answer
    // 'ordinary' — workspaceId is 'main', worktreeId is null, and
    // workspaceRoot is `root` itself.
    let workspace_id = "main";
    let workspace_root = root.to_str().ok_or(Err2::Ex)?;
    let now = now_ms();
    let now_iso_s = now_iso();
    ws::register_workspace(
        control_root,
        ws::RegisterSpec {
            id: workspace_id,
            kind: "main",
            root: workspace_root,
            branch: None,
            base_sha: None,
        },
        &now_iso_s,
    )
    .map_err(ws_err)?;

    // Production-shaped isOwnerLive, built from THIS module's readSession +
    // heartbeatStale. `preflight` has already proven every session record
    // parses and every heartbeat is date-parseable, so neither `unwrap_or`
    // below can fire on a shape Node would have read differently.
    let is_owner_live = |owner: &str, now_ms_v: f64| -> bool {
        match read_session(control_root, owner) {
            Ok(Some(session)) => !heartbeat_stale(&session, now_ms_v).unwrap_or(true),
            _ => false,
        }
    };
    let attach = ws::attach_workspace(
        control_root,
        workspace_id,
        Some(session_id),
        ws::OwnershipOpts { now, now_iso: &now_iso_s, is_owner_live: Some(&is_owner_live) },
    )
    .map_err(ws_err)?;
    let owner_session = match attach.role {
        ws::AttachRole::Owner { .. } => return Ok(Policy::Proceed),
        ws::AttachRole::ReadOnly { write_owner_session } => write_owner_session,
    };

    // blocked: a DIFFERENT, live session already owns this workspace's writes.
    let auto_isolate = isolate
        || config
            .get("guards")
            .and_then(|g| jget(g, "auto_isolate"))
            .map(|v| v == &Value::Bool(true))
            .unwrap_or(false);
    if !auto_isolate {
        let marker = isolate_notice_marker_path(control_root, session_id);
        let already_shown = marker.exists();
        if !already_shown {
            // Best-effort stamp: a failed write only means the next refusal
            // shows the fuller message again.
            let _ = write_json_atomic(
                &marker,
                &json!({ "session_id": session_id, "shown_at": now_iso() }),
            );
        }
        let one_liner = write_policy_isolate_one_liner(verb_hint);
        return Err(Err2::Msg(if already_shown {
            format!(
                "bee write-policy: workspace \"{workspace_id}\" is still write-owned by session \"{owner_session}\". FIX: {one_liner}"
            )
        } else {
            format!(
                "bee write-policy: a second write-capable session defaults to isolation, not a wait — workspace \"{workspace_id}\" already has a live write owner (session \"{owner_session}\"). Bee never writes into the same checkout as a live owner. FIX: {one_liner}"
            )
        }));
    }

    // Consented (--isolate) or configured (guards.auto_isolate) — a fresh
    // feature worktree. createFeatureWorktree already registers its own
    // workspace record (msn-19); this attaches the ACTING session as that new
    // workspace's first (uncontested) write owner.
    let isolate_feature = if js_trim(feature).is_empty() {
        format!("session-{session_id}")
    } else {
        js_trim(feature).to_string()
    };
    let mut lock_busy: Option<String> = None;
    let created = match crate::verbs::worktree::create_feature_worktree(
        control_root,
        &isolate_feature,
        None,
        // startFeature's isolation worktree never takes a companion — bee.mjs
        // passes neither companion option here (only handleWorktreeNew's
        // --with-companion does).
        crate::verbs::worktree::CompanionSpec::default(),
        &mut lock_busy,
    ) {
        Ok(c) => c,
        Err(crate::verbs::worktree::CErr::Refuse(message)) => return Err(Err2::Msg(message)),
        Err(crate::verbs::worktree::CErr::Ex) => {
            return match lock_busy {
                // Reached AFTER a lock attempt, so native (campaign rule 2).
                Some(message) => Err(Err2::Msg(message)),
                None => Err(Err2::Ex),
            }
        }
    };
    ws::attach_workspace(
        control_root,
        &created.id,
        Some(session_id),
        ws::OwnershipOpts { now, now_iso: &now_iso(), is_owner_live: None },
    )
    .map_err(ws_err)?;

    let worktree_root = created.worktree_root.to_string_lossy().into_owned();
    let cost_disclosure = format!(
        "[bee cost] Isolated worktree created — a FULL working-tree copy at {worktree_root} (disk cost scales with repo size)."
    );
    let text = format!(
        "{cost_disclosure}\nOpen your next session with cwd={worktree_root} to continue \"{isolate_feature}\" there — this checkout stays untouched."
    );
    let mut result = Map::new();
    result.insert("ok".into(), Value::Bool(true));
    result.insert("mode".into(), json!("isolated"));
    result.insert("workspace".into(), json!("isolated-created"));
    result.insert("redirect".into(), Value::Bool(true));
    result.insert("worktreeRoot".into(), json!(worktree_root));
    result.insert("workspaceId".into(), json!(created.id));
    result.insert("branch".into(), json!(created.branch));
    result.insert("costDisclosure".into(), json!(cost_disclosure));
    result.insert("text".into(), json!(text.clone()));
    Ok(Policy::Redirect { result, text })
}

/// A WorkspaceStoreError / LockBusyError message is deterministic and
/// reproduced natively by verbs/workspace_store.rs; only its `Ex` arm (a V8
/// message) delegates.
pub(crate) fn ws_err(e: ws::WsErr) -> Err2 {
    match e {
        ws::WsErr::Err { message, .. } => Err2::Msg(message),
        ws::WsErr::Ex => Err2::Ex,
    }
}

// ─── startLane / the default legacy body ──────────────────────────────────

/// state.mjs startLane — the whole lane branch, run inside the 'state' hold.
#[allow(clippy::too_many_arguments)]
pub(crate) fn start_lane(
    root: &Path,
    control_root: &Path,
    feature: &str,
    mode: Option<&str>,
    phase: &str,
    session_id: Option<&str>,
    paths: &[String],
    workflows: &[Map<String, Value>],
) -> R2<Result<Map<String, Value>, String>> {
    // A corrupt existing lane record refuses loudly with the file untouched.
    let existing = read_lane_strict(root, feature)?;
    if let Some(existing) = &existing {
        let existing_phase = cell_field(existing, "phase");
        if &existing_phase != &json!("idle")
            && &existing_phase != &json!("compounding-complete")
        {
            return Ok(Err(format!(
                "startFeature: refused — lane \"{feature}\" is mid-flight at phase \"{}\", not idle or the terminal alias \"compounding-complete\". FIX: finish or explicitly wind down that lane first, then retry.",
                js_disp_opt(Some(&existing_phase))
            )));
        }
    }

    let cells = list_all_cells_for_start(root)?;

    // (a) nonterminal cells of THIS lane's feature.
    let nonterminal: Vec<String> = cells
        .iter()
        .filter(|c| {
            &cell_field(c, "feature") == &json!(feature) && {
                let s = cell_field(c, "status");
                &s == &json!("open")
                    || &s == &json!("claimed")
                    || &s == &json!("blocked")
            }
        })
        .map(|c| format!("{}({})", cell_id_disp(c), js_disp_opt(c.get("status"))))
        .collect();
    if !nonterminal.is_empty() {
        return Ok(Err(format!(
            "startFeature: refused — feature \"{feature}\" already has nonterminal cell(s): {}. An abandoned cell must first be resolved through the existing drop verb (bee cells drop --id ID --reason R). FIX: cap or drop each listed cell, then retry.",
            nonterminal.join(", ")
        )));
    }

    // (b) the global handoff blocks a LANE start only when it names this feature.
    if let Some(handoff) = read_handoff(root)? {
        if &jget(&handoff, "feature").cloned().unwrap_or(Value::Null) == &json!(feature)
        {
            return Ok(Err(format!(
                "startFeature: refused — .bee/HANDOFF.json names feature \"{feature}\"; its paused work must resume or close before this lane restarts. FIX: resume the handoff (or explicitly delete HANDOFF.json once its work is truly abandoned), then retry."
            )));
        }
    }

    // (c) an OTHER live worker whose claimed cell derives to this feature.
    let lane_workers: Vec<String> = active_workers(control_root, session_id)?
        .into_iter()
        .filter(|w| {
            let Some(cell_id) = &w.cell else { return false };
            cells.iter().any(|c| {
                &cell_field(c, "id") == cell_id
                    && &cell_field(c, "feature") == &json!(feature)
            })
        })
        .map(|w| format!("{}({})", w.session_id, js_disp_opt(w.cell.as_ref())))
        .collect();
    if !lane_workers.is_empty() {
        return Ok(Err(format!(
            "startFeature: refused — active worker session(s) on feature \"{feature}\": {}. FIX: wait for the session's heartbeat to go stale, or have it cap/drop its claimed cell, then retry.",
            lane_workers.join(", ")
        )));
    }

    // (d) declared intended paths vs ANOTHER session's active holds.
    let declared: Vec<String> = paths
        .iter()
        .filter(|p| !js_trim(p).is_empty())
        .map(|p| js_trim(p).to_string())
        .collect();
    if !declared.is_empty() {
        let root_s = root.to_str().ok_or(Err2::Ex)?.to_string();
        let reservation_holds: Vec<String> = list_reservations(&root_s, true, now_ms())?
            .iter()
            .filter(|r| declared.iter().any(|d| paths_overlap(&r.path, d)))
            .map(resv_disp)
            .collect();
        if !reservation_holds.is_empty() {
            return Ok(Err(format!(
                "startFeature: refused — declared path(s) overlap active reservation hold(s): {}. FIX: wait for release/expiry (bee reservations release), or start the lane over non-overlapping paths.",
                reservation_holds.join(", ")
            )));
        }
        let claim_holds = list_claim_holds_for_start(control_root, session_id, &cells, &declared)?;
        if !claim_holds.is_empty() {
            return Ok(Err(format!(
                "startFeature: refused — declared path(s) overlap file(s) of cell(s) claimed by another session: {}. FIX: wait for the claim to release or expire, or start the lane over non-overlapping paths.",
                claim_holds
                    .iter()
                    .map(|h| format!("{}:{}({})", h.session, h.cell, h.path))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    }

    // (e) the shared workflow-precondition layer.
    if let Some(refusal) = check_no_live_workflow_for_feature(workflows, feature) {
        return Ok(Err(refusal));
    }
    if let Some(refusal) = check_no_same_feature_claimed_cells(feature, &cells) {
        return Ok(Err(refusal));
    }

    // ONE atomic write to this lane's record. created_at survives a restart.
    let mut record = Map::new();
    record.insert("schema_version".into(), json!("1.0"));
    record.insert("feature".into(), json!(feature));
    record.insert("mode".into(), mode.map_or(Value::Null, |m| json!(m)));
    record.insert("phase".into(), json!(phase));
    record.insert("approved_gates".into(), Value::Object(default_gates()));
    record.insert(
        "summary".into(),
        json!(format!("Feature \"{feature}\" started at phase \"{phase}\" (lane).")),
    );
    record.insert(
        "next_action".into(),
        json!(format!("Continue bee-{phase} for \"{feature}\" (lane).")),
    );
    let created_at = existing
        .as_ref()
        .and_then(|e| e.get("created_at"))
        .and_then(|v| match v {
            Value::String(s) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(now_iso);
    record.insert("created_at".into(), json!(created_at));
    write_lane(root, &record)?;
    Ok(Ok(record))
}

/// startFeature's DEFAULT (non-lane) legacy body, run inside the 'state' hold.
/// Every read happens before the single write at the end, so a refusal leaves
/// zero mutations.
pub(crate) fn start_default(
    root: &Path,
    feature: &str,
    mode: Option<&str>,
    phase: &str,
    workflows: &[Map<String, Value>],
) -> R2<Result<Map<String, Value>, String>> {
    let mut state = read_state_strict(root)?;

    let current_phase = state.get("phase").cloned().unwrap_or(Value::Null);
    if &current_phase != &json!("idle")
        && &current_phase != &json!("compounding-complete")
    {
        return Ok(Err(format!(
            "startFeature: refused — current phase is \"{}\", not idle or the terminal alias \"compounding-complete\". A prior feature must finish or be explicitly wound down before a new feature starts. FIX: resume/close the current feature through its normal chain, or drop its remaining cells (bee cells drop), then retry.",
            js_disp_opt(Some(&current_phase))
        )));
    }

    // F5: scoped per-feature — a handoff naming a DIFFERENT feature no longer
    // blocks this start.
    if let Some(handoff) = read_handoff(root)? {
        if &jget(&handoff, "feature").cloned().unwrap_or(Value::Null) == &json!(feature)
        {
            return Ok(Err(format!(
                "startFeature: refused — .bee/HANDOFF.json names feature \"{feature}\"; its paused work must resume or close before this feature restarts. FIX: resume the handoff (or explicitly delete HANDOFF.json once its work is truly abandoned), then retry."
            )));
        }
    }

    // msn-20 D3 RETIRED the old "another session is active, wait" precondition
    // — applyWritePolicy above is its replacement, so nothing stands here.

    let root_s = root.to_str().ok_or(Err2::Ex)?.to_string();
    let active_reservations = list_reservations(&root_s, true, now_ms())?;
    if !active_reservations.is_empty() {
        return Ok(Err(format!(
            "startFeature: refused — {} active reservation(s) remain ({}). FIX: release them first (bee reservations release).",
            active_reservations.len(),
            active_reservations.iter().map(resv_disp).collect::<Vec<_>>().join(", ")
        )));
    }

    let cells = list_all_cells_for_start(root)?;
    let claimed: Vec<String> = cells
        .iter()
        .filter(|c| &cell_field(c, "status") == &json!("claimed"))
        .map(cell_id_disp)
        .collect();
    if !claimed.is_empty() {
        return Ok(Err(format!(
            "startFeature: refused — claimed cell(s) remain: {}. FIX: cap or drop them first (bee cells cap / bee cells drop).",
            claimed.join(", ")
        )));
    }

    let prior_feature = state.get("feature").cloned().unwrap_or(Value::Null);
    if truthy(&prior_feature) {
        let nonterminal: Vec<String> = cells
            .iter()
            .filter(|c| {
                &cell_field(c, "feature") == &prior_feature && {
                    let s = cell_field(c, "status");
                    &s == &json!("open")
                        || &s == &json!("claimed")
                        || &s == &json!("blocked")
                }
            })
            .map(|c| format!("{}({})", cell_id_disp(c), js_disp_opt(c.get("status"))))
            .collect();
        if !nonterminal.is_empty() {
            return Ok(Err(format!(
                "startFeature: refused — prior feature \"{}\" has nonterminal cell(s): {}. An abandoned cell must first be resolved through the existing drop verb (bee cells drop --id ID --reason R) — startFeature never auto-clears cells as cleanup. FIX: cap or drop each listed cell, then retry.",
                js_disp_opt(Some(&prior_feature)),
                nonterminal.join(", ")
            )));
        }
    }

    if let Some(refusal) = check_no_live_workflow_for_feature(workflows, feature) {
        return Ok(Err(refusal));
    }
    if let Some(refusal) = check_no_same_feature_claimed_cells(feature, &cells) {
        return Ok(Err(refusal));
    }

    // ONE atomic write. A JS re-assignment keeps each key's ORIGINAL position,
    // so the on-disk key order is readStateStrict's, not this list's.
    state.insert("feature".into(), json!(feature));
    state.insert("mode".into(), mode.map_or(Value::Null, |m| json!(m)));
    state.insert("phase".into(), json!(phase));
    state.insert("approved_gates".into(), Value::Object(default_gates()));
    state.insert(
        "summary".into(),
        json!(format!("Feature \"{feature}\" started at phase \"{phase}\".")),
    );
    state.insert(
        "next_action".into(),
        json!(format!("Continue bee-{phase} for \"{feature}\".")),
    );
    // rti-1: a freshly started feature has not been triaged yet, so it
    // carries no route — whatever the PRIOR feature's record held is
    // dropped here rather than riding along untouched. This is exactly the
    // state `cells claim`'s no-route warning (claimed_feature_has_route)
    // exists to detect; the lane path (start_lane) never had this bug
    // because it always builds a fresh record.
    state.remove("route");
    write_state(root, &state)?;
    Ok(Ok(state))
}

/// Every read the run would make, classified BEFORE the first lock or write.
/// A single `Err(Exotic)` here sends the whole command to Node with zero
/// bytes emitted (campaign rule: a delegation is only sound before a mutation,
/// and seedLegacyWorkflows' first `createWorkflow` is that mutation).
pub(crate) fn preflight(root: &Path, control_root: &Path) -> Ex<()> {
    let now = now_ms();
    list_workflows(control_root).map_err(|_| Exotic)?;
    read_state_peek(root)?;
    list_lanes(root)?;
    list_all_cells_for_start(root)?;
    read_handoff(root)?;
    let root_s = root.to_str().ok_or(Exotic)?.to_string();
    list_reservations(&root_s, true, now)?;
    if control_root != root {
        let ctrl_s = control_root.to_str().ok_or(Exotic)?.to_string();
        list_reservations(&ctrl_s, true, now)?;
    }
    // Sessions + claims: read AND date-parsed, so `is_owner_live`'s and
    // active_workers' fallbacks can never fire on a shape Node reads
    // differently.
    for session in list_session_records(control_root)? {
        heartbeat_stale(&session, now)?;
    }
    for id in claim_ids(control_root)? {
        if let Some(claim) = read_claim(control_root, &id)? {
            is_claim_active(&claim, now)?;
        }
    }
    // The grants registry backs resolveContext's decideWorktreeStore AND the
    // consented isolate-create's own createFeatureWorktree.
    crate::verbs::worktree::read_grants_strict(&control_root.join(".bee")).ok_or(Exotic)?;
    Ok(())
}

/// bee.mjs handleStateStartFeature.
pub(crate) fn run_start_feature(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(
        &flags,
        &["feature", "mode", "phase", "as-lane", "session-id", "paths", "isolate"],
    ) {
        return None;
    }
    if !bool_flag_ok(&flags, "as-lane") || !bool_flag_ok(&flags, "isolate") {
        return None;
    }
    let ctx = match go("state start-feature", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };

    // controlRootFor(root): every workflow/session/claim/workspace read and
    // write below is control-plane. See the section header for why a mismatch
    // delegates rather than silently reading `root`.
    let root_s = ctx.root.to_str()?.to_string();
    let control_root_s = crate::verbs::reservations::control_root_for(&root_s).ok()?;
    if control_root_s != root_s {
        return None;
    }
    let control_root = ctx.root.clone();
    if preflight(&ctx.root, &control_root).is_err() {
        return None;
    }

    let out = (|| -> R2<Out> {
        // bee.mjs's rejectDryRun is structurally unreachable for this verb:
        // `dry-run` is not in its registry schema, so the dispatcher's own
        // unknown-flag refusal fires first — and `keys_known` above already
        // routes that argv shape to Node.
        let feature = require_flag(&flags, "feature")?;
        let mode = flag_value(&flags, "mode");
        let phase = flag_value(&flags, "phase").unwrap_or_else(|| "exploring".to_string());
        let lane = matches!(flags.get("as-lane"), Some(FlagV::Present))
            || matches!(flags.get("as-lane"), Some(FlagV::S(s)) if s == "true");
        let session_id = flag_value(&flags, "session-id");
        let paths: Vec<String> = match flags.get("paths") {
            Some(FlagV::S(s)) => split_list(s),
            Some(FlagV::Present) => split_list("true"), // String(true)
            None => Vec::new(),
        };
        let isolate = matches!(flags.get("isolate"), Some(FlagV::Present))
            || matches!(flags.get("isolate"), Some(FlagV::S(s)) if s == "true");

        // ── startFeature ──────────────────────────────────────────────────
        let feature_trimmed = js_trim(&feature).to_string();
        if feature_trimmed.is_empty() {
            return Ok(Out::Thrown(
                "startFeature: a non-empty --feature slug is required.".to_string(),
            ));
        }
        if !is_known_phase(&phase) {
            return Ok(Out::Thrown(format!(
                "startFeature: invalid phase \"{phase}\" — not in the known-phase enum (isKnownPhase). FIX: use one of {KNOWN_PHASES_JOINED}."
            )));
        }
        // A lane feature becomes a filename: requireLaneFeature's two throws
        // fire from lanePath, inside the 'state' hold, and are deterministic.
        let session_id_trimmed = session_id
            .as_deref()
            .map(js_trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        // (1) C1 seeding, BEFORE this call's own legacy write.
        seed_legacy_workflows(&ctx.root, &control_root)?;

        // (2) the write policy (DEFAULT path only — lanes are bee's existing,
        // already-coordinated concurrent mechanism and stay byte-untouched).
        if !lane {
            let config = crate::state::read_config_raw(&ctx.root);
            match apply_write_policy(
                &ctx.root,
                &control_root,
                &config,
                session_id_trimmed.as_deref(),
                &paths,
                isolate,
                &feature_trimmed,
                &format!("state start-feature --feature {feature_trimmed}"),
            )? {
                Policy::Proceed => {}
                Policy::Redirect { result, text } => {
                    // A successful isolate-create short-circuits: startFeature
                    // never touched root's own pipeline, so the projection
                    // rebuild below must not run either.
                    return Ok(Out::Emit(Value::Object(result), text, 0));
                }
            }
        }

        // (2b) the legacy read-check-write body, under 'state'.
        let workflows = list_workflows(&control_root)?;
        let guard = acquire_state_lock(&ctx.root)?;
        let legacy = if lane {
            // requireLaneFeature(feature) — the RAW flag, not the trimmed one.
            match crate::verbs::workflow_store::require_lane_feature(&feature) {
                Ok(lane_feature) => start_lane(
                    &ctx.root,
                    &control_root,
                    &lane_feature,
                    mode.as_deref(),
                    &phase,
                    session_id_trimmed.as_deref(),
                    &paths,
                    &workflows,
                ),
                Err(e) => Err(e),
            }
        } else {
            start_default(&ctx.root, &feature_trimmed, mode.as_deref(), &phase, &workflows)
        };
        drop(guard);
        let legacy = match legacy? {
            Ok(record) => record,
            Err(refusal) => return Ok(Out::Thrown(refusal)),
        };

        // (3) workflow record creation OUTSIDE the 'state' lock, carrying the
        // legacy write's own computed summary/next_action.
        ensure_workflow_record_for_feature(
            &control_root,
            &feature_trimmed,
            &phase,
            mode.as_deref(),
            legacy.get("summary"),
            legacy.get("next_action"),
            None,
        )?;
        // Close the OUTGOING work — every OTHER live record. Scoped to the
        // DEFAULT path; a lane start closes nothing, by design.
        if !lane {
            close_workflows_for_feature(&control_root, Some(&feature_trimmed))?;
        }

        // bee.mjs's own projection rebuild, under the lock for the record IT
        // writes (splpr-3: exactly one lock, so no new edge in the global order).
        let record_feature = js_disp_opt(legacy.get("feature"));
        let lock_name = if lane {
            lane_lock_name(&record_feature)
        } else {
            "state".to_string()
        };
        let guard = acquire_named_lock(&ctx.root, &lock_name)?;
        let rebuilt = if lane {
            rebuild_lane_projection(&ctx.root, &record_feature).map(|_| ())
        } else {
            rebuild_state_projection(&ctx.root)
        };
        drop(guard);
        rebuilt?;

        let text = format!(
            "Started feature \"{record_feature}\"{} at phase \"{}\" (mode {}); all four gates reset.",
            if lane { " as a lane" } else { "" },
            js_disp_opt(legacy.get("phase")),
            // `state.mode ?? 'null'` — nullish, so `null` renders "null".
            match legacy.get("mode") {
                None | Some(Value::Null) => "null".to_string(),
                Some(v) => jsjson::js_to_string(v),
            }
        );
        Ok(Out::Emit(Value::Object(legacy), text, 0))
    })();
    finish(&ctx, out)
}
