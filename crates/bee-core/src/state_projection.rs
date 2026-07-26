//! state_projection — read-only-source, write-through compatibility
//! projections of workflow records onto the legacy single-pipeline stores
//! (`.bee/state.json`, `.bee/lanes/<feature>.json`, `.bee/HANDOFF.json`),
//! ported from `.bee/bin/lib/state-projection.mjs` (rust-port-16,
//! CONTEXT.md D1/D3/D9).
//!
//! A workflow record ([`crate::workflow_store`]) is the unit of
//! coordination state. This module is the ONE place that turns the current
//! set of workflow records back into the legacy shapes every existing
//! reader still expects.
//!
//! **`control_root` deviation from the mjs source, disclosed (rust-port-16
//! deviation):** `state-projection.mjs` resolves its own control root
//! internally via `controlRootFor(root)` (`resolveContext(root).controlRoot`),
//! which walks git worktree topology (linked-worktree detection, grants,
//! `.git` file parsing). That git-topology walk is NOT ported into
//! `bee-core` — it lives in `crates/queen-bee/src/adapter.rs`
//! (`control_root_for`/`resolve_roots`), a binary-crate concern `bee-core`
//! cannot depend on (dependency direction is `queen-bee -> bee-core`, never
//! the reverse) and outside this cell's file scope (`crates/bee-core/*`
//! only). Every function below therefore takes an already-resolved
//! `control_root: &Path` as an explicit parameter instead of resolving it
//! itself — the SAME "topology resolved once by the caller, consumed by
//! bee-core" shape `guards.rs`'s [`crate::guards::WriteTopology`] already
//! established for exactly this reason (see that module's own doc comment).
//! For every ordinary, non-worktree checkout `control_root == root`, matching
//! the mjs source's own documented invariant ("Main/solo checkouts are
//! unaffected either way — `controlRootFor(root) === root` there,
//! byte-identical"), which is what every fixture in this cell's parity
//! suite exercises.
//!
//! `.bee/bin/lib/state-projection.mjs` is FROZEN for the duration of the
//! rust-port feature (D1).

use std::fs;
use std::io;
use std::path::Path;

use serde_json::{json, Map, Value};

use crate::fsutil::{read_json, write_json_atomic};
use crate::jsdate::parse_iso_ms;
use crate::state::{self, default_state, handoff_path, list_handoff_mailbox, read_lane, read_state, write_state, ApprovedGates, State};
use crate::workflow_store::{list_workflows, WorkflowRecord, GATE_NAMES};

/// True once at least one workflow record exists anywhere in the repo — the
/// C1 authority switch (`projectionsAuthoritative`).
pub fn projections_authoritative(control_root: &Path) -> bool {
    !list_workflows(control_root).workflows.is_empty()
}

/// `workflowGatesToApprovedGates(gates, planRev)` — the workflow gates map
/// (per-name `{approved, approved_for_plan_rev}`) -> the legacy boolean
/// `approved_gates` shape, in the FIXED `GATE_NAMES` key order. The
/// projected boolean is the PLAN-REV-EFFECTIVE approval: `approved &&
/// (approved_for_plan_rev == null || approved_for_plan_rev === planRev)`.
/// `plan_rev = None` mirrors the mjs source's `planRev` being omitted
/// (`undefined`): a gate stamped with an explicit rev never matches, so it
/// reads as ineffective unless its own `approved_for_plan_rev` is null/absent.
pub fn workflow_gates_to_approved_gates(gates: &Map<String, Value>, plan_rev: Option<i64>) -> ApprovedGates {
    let mut approved = ApprovedGates::default();
    for name in GATE_NAMES {
        let entry = gates.get(name);
        let is_approved = entry.and_then(|e| e.get("approved")).and_then(Value::as_bool).unwrap_or(false);
        let rev = entry.and_then(|e| e.get("approved_for_plan_rev")).and_then(Value::as_i64);
        let rev_effective = match rev {
            None => true,
            Some(r) => plan_rev == Some(r),
        };
        let value = is_approved && rev_effective;
        match name {
            "context" => approved.context = value,
            "shape" => approved.shape = value,
            "execution" => approved.execution = value,
            "review" => approved.review = value,
            _ => {
                approved.extra.insert(name.to_string(), Value::Bool(value));
            }
        }
    }
    approved
}

/// `pickNewestActiveWorkflow(workflows)` — the newest ACTIVE workflow:
/// `status === 'active'` only; ties broken by `created_at` descending, then
/// `id` descending.
pub fn pick_newest_active_workflow(workflows: &[WorkflowRecord]) -> Option<&WorkflowRecord> {
    let mut active: Vec<&WorkflowRecord> = workflows.iter().filter(|w| w.status == "active").collect();
    if active.is_empty() {
        return None;
    }
    active.sort_by(|a, b| {
        let ta = parse_iso_ms(&a.created_at).unwrap_or(0);
        let tb = parse_iso_ms(&b.created_at).unwrap_or(0);
        if tb != ta {
            return tb.cmp(&ta);
        }
        if a.id == b.id {
            std::cmp::Ordering::Equal
        } else if a.id < b.id {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    });
    Some(active[0])
}

/// Overrides `rebuild_state_projection` writes into the SAME write as its
/// D1-field rebuild, regardless of which branch fired (or none) — mirrors
/// the mjs source's `overrides.cellCounts`/`overrides.lastActivity`.
/// `None` means "caller did not pass this override" (mjs's
/// `hasOwnProperty` check); `Some(Value::Null)` is a legitimate explicit
/// override value.
#[derive(Debug, Clone, Default)]
pub struct StateOverrides {
    pub cell_counts: Option<Value>,
    pub last_activity: Option<Value>,
}

impl StateOverrides {
    fn has_any(&self) -> bool {
        self.cell_counts.is_some() || self.last_activity.is_some()
    }

    fn apply(&self, state: &mut State) {
        if let Some(v) = &self.cell_counts {
            state.extra.insert("cells".to_string(), v.clone());
        }
        if let Some(v) = &self.last_activity {
            state.extra.insert("last_activity".to_string(), v.clone());
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateProjectionResult {
    pub authoritative: bool,
    pub source: Option<String>,
    pub state: State,
}

/// `rebuildStateProjection(root, overrides)` — full rebuild of
/// `.bee/state.json`'s D1-owned fields (phase/feature/mode/approved_gates/
/// summary/next_action). Two sources, tried in order:
///
/// 1. FEATURE-MATCHED: when the current default record already names a
///    feature AND a LIVE (non-closed) workflow record names that SAME
///    feature, that record is authoritative — regardless of phase.
/// 2. IDLE BOOTSTRAP: when the default record names no feature at all
///    (idle, or the terminal alias `compounding-complete`) and at least one
///    workflow record exists, adopt the newest ACTIVE workflow.
///
/// Neither source fires (pure no-op on the D1 fields) when zero workflow
/// records exist anywhere, or a feature is set but no live workflow names
/// it. Every other field already on the file passes through unchanged in
/// every branch; `overrides` are written into the SAME write regardless of
/// which branch fired.
pub fn rebuild_state_projection(root: &Path, control_root: &Path, overrides: &StateOverrides) -> io::Result<StateProjectionResult> {
    let workflows = list_workflows(control_root).workflows;
    let current = read_state(root);
    let has_overrides = overrides.has_any();

    let apply_overrides_only = |current: &State| -> io::Result<StateProjectionResult> {
        if !has_overrides {
            return Ok(StateProjectionResult { authoritative: false, source: None, state: current.clone() });
        }
        let mut next = current.clone();
        overrides.apply(&mut next);
        write_state(root, &next)?;
        Ok(StateProjectionResult { authoritative: false, source: None, state: next })
    };

    if workflows.is_empty() {
        return apply_overrides_only(&current);
    }

    // Branch (1) — feature-matched.
    if let Some(feature) = current.feature.as_ref().filter(|f| !f.trim().is_empty()) {
        if let Some(wf) = workflows.iter().find(|w| &w.feature == feature && w.status != "closed") {
            let mut next = current.clone();
            next.phase = wf.phase.clone();
            next.feature = Some(wf.feature.clone());
            next.mode = wf.mode.clone();
            next.approved_gates = workflow_gates_to_approved_gates(&wf.gates, Some(wf.plan_rev));
            next.summary = wf.summary.clone();
            next.next_action = wf.next_action.clone();
            overrides.apply(&mut next);
            write_state(root, &next)?;
            return Ok(StateProjectionResult { authoritative: true, source: Some(wf.id.clone()), state: next });
        }
        // A feature is set but no LIVE workflow names it — falls through;
        // the idle-bootstrap branch below always requires `!current.feature`
        // and so is skipped too, net effect is apply_overrides_only().
    }

    // Branch (2) — idle bootstrap.
    let current_is_idle = current.feature.as_ref().filter(|f| !f.trim().is_empty()).is_none()
        && (current.phase == "idle" || current.phase == "compounding-complete" || current.phase.trim().is_empty());
    if !current_is_idle {
        return apply_overrides_only(&current);
    }

    let active = pick_newest_active_workflow(&workflows);
    let defaults = default_state();
    let mut next = current.clone();
    next.phase = active.map(|a| a.phase.clone()).unwrap_or_else(|| "idle".to_string());
    next.feature = active.map(|a| a.feature.clone());
    next.mode = active.and_then(|a| a.mode.clone());
    next.approved_gates = match active {
        Some(a) => workflow_gates_to_approved_gates(&a.gates, Some(a.plan_rev)),
        None => defaults.approved_gates,
    };
    next.summary = active.map(|a| a.summary.clone()).unwrap_or(defaults.summary);
    next.next_action = active.map(|a| a.next_action.clone()).unwrap_or(defaults.next_action);
    overrides.apply(&mut next);
    write_state(root, &next)?;
    Ok(StateProjectionResult { authoritative: true, source: active.map(|a| a.id.clone()), state: next })
}

#[derive(Debug, Clone)]
pub struct LaneProjectionResult {
    pub authoritative: bool,
    pub source: Option<String>,
    pub lane: Option<State>,
}

/// `rebuildLaneProjection(root, feature)` — full rebuild of
/// `.bee/lanes/<feature>.json` from the live (non-closed) workflow record
/// naming that feature. The six baseline D1 fields are fully recomputed
/// every time; `created_at` and any ad hoc field already on the existing
/// lane file (`last_scribing_run`, `gate_revoked_at`, `advisor_ref`) pass
/// through UNCHANGED. No-op (`authoritative: false`) when zero workflow
/// records exist anywhere, or no LIVE workflow record names this feature —
/// never guesses, never deletes an existing lane file it cannot derive.
///
/// Built on raw `serde_json::Value`/`Map` (mirroring the mjs source's
/// object-spread `{...existing, ...}`) rather than the typed [`State`]
/// struct: `State` is also `.bee/state.json`'s shape, which carries a
/// `workers` field lane records never have — spreading through a typed
/// struct with a named `workers` field would silently INTRODUCE a
/// `"workers": []` key no mjs lane record has ever written. Working in
/// `Value` space keeps this rebuild byte-faithful to "only the keys that
/// were already there, plus the ones this rebuild sets, survive".
pub fn rebuild_lane_projection(root: &Path, control_root: &Path, feature: &str) -> io::Result<LaneProjectionResult> {
    let workflows = list_workflows(control_root).workflows;
    if workflows.is_empty() {
        return Ok(LaneProjectionResult { authoritative: false, source: None, lane: read_lane(root, feature) });
    }
    let Some(wf) = workflows.iter().find(|w| w.feature == feature && w.status != "closed") else {
        return Ok(LaneProjectionResult { authoritative: false, source: None, lane: read_lane(root, feature) });
    };

    let lane_file = state::lane_path(root, feature).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let existing_raw: Value = read_json(&lane_file, Value::Null);
    let mut next_map: Map<String, Value> = match &existing_raw {
        Value::Object(m) => m.clone(),
        _ => Map::new(),
    };
    next_map.insert("schema_version".to_string(), json!("1.0"));
    next_map.insert("feature".to_string(), json!(wf.feature));
    next_map.insert("mode".to_string(), json!(wf.mode));
    next_map.insert("phase".to_string(), json!(wf.phase));
    next_map.insert("approved_gates".to_string(), json!(workflow_gates_to_approved_gates(&wf.gates, Some(wf.plan_rev))));
    next_map.insert("summary".to_string(), json!(wf.summary));
    next_map.insert("next_action".to_string(), json!(wf.next_action));

    let existing_created_at = existing_raw.get("created_at").and_then(Value::as_str).filter(|s| !s.is_empty()).map(str::to_string);
    let created_at = existing_created_at.unwrap_or_else(|| wf.created_at.clone());
    next_map.insert("created_at".to_string(), json!(created_at));

    let next_value = Value::Object(next_map);
    write_json_atomic(&lane_file, &next_value)?;
    let lane_state: State = serde_json::from_value(next_value).expect("rebuilt lane record must deserialize back into State");
    Ok(LaneProjectionResult { authoritative: true, source: Some(wf.id.clone()), lane: Some(lane_state) })
}

#[derive(Debug, Clone)]
pub struct HandoffProjectionResult {
    pub authoritative: bool,
    pub source: Option<String>,
}

/// `rebuildHandoffProjection(root)` — full rebuild of the legacy
/// `.bee/HANDOFF.json` as a projection of the newest OPEN mailbox handoff
/// across every workflow. No-op (`authoritative: false`, file left
/// untouched) when zero workflow records exist anywhere. When at least one
/// workflow exists but NONE has an open mailbox handoff, the legacy file
/// (if present) is REMOVED. Ties broken by `written_at` descending, then
/// workflow id descending.
pub fn rebuild_handoff_projection(root: &Path, control_root: &Path) -> io::Result<HandoffProjectionResult> {
    let workflows = list_workflows(control_root).workflows;
    if workflows.is_empty() {
        return Ok(HandoffProjectionResult { authoritative: false, source: None });
    }

    let mut newest: Option<(state::HandoffMailboxRecord, String)> = None;
    for wf in &workflows {
        let mut open: Vec<state::HandoffMailboxRecord> = list_handoff_mailbox(control_root, &wf.id)
            .into_iter()
            .filter(|r| r.fields.get("status").and_then(Value::as_str) == Some("open"))
            .collect();
        let Some(candidate) = open.pop() else { continue };
        match &newest {
            None => newest = Some((candidate, wf.id.clone())),
            Some((cur, cur_wf)) => {
                let a = candidate.fields.get("written_at").and_then(Value::as_str).and_then(parse_iso_ms).unwrap_or(0);
                let b = cur.fields.get("written_at").and_then(Value::as_str).and_then(parse_iso_ms).unwrap_or(0);
                if a > b || (a == b && wf.id.as_str() > cur_wf.as_str()) {
                    newest = Some((candidate, wf.id.clone()));
                }
            }
        }
    }

    let Some((record, workflow_id)) = newest else {
        let _ = fs::remove_file(handoff_path(root));
        return Ok(HandoffProjectionResult { authoritative: true, source: None });
    };

    // Translate the mailbox envelope back to the legacy flat shape: drop the
    // mailbox-only fields so a byte-identical legacy reader sees exactly the
    // shape `writeHandoff` would have produced. `writer_session` is kept
    // verbatim (mirrors a planned-next record's own duplication).
    let mut projected = record.fields.clone();
    for key in ["status", "id", "workflow_id", "target_role", "from_session"] {
        projected.remove(key);
    }
    crate::fsutil::write_json_atomic(&handoff_path(root), &Value::Object(projected))?;
    Ok(HandoffProjectionResult { authoritative: true, source: Some(workflow_id) })
}

#[derive(Debug, Clone)]
pub struct AllProjectionsResult {
    pub state: StateProjectionResult,
    pub handoff: HandoffProjectionResult,
    pub lanes: Vec<LaneProjectionResult>,
}

/// `rebuildAllProjections(root)` — the recovery entry point: rebuilds
/// `state.json`, the legacy `HANDOFF.json`, and every active workflow's
/// lane projection ("one per active workflow").
///
/// **Scope note (rust-port-16 deviation, disclosed):** the mjs source also
/// rebuilds `.bee/reservations.json` here via `reservations.mjs`'s
/// `rebuildReservationsProjection` (lease-record -> legacy-reservation-row
/// translation). That translation layer (`listReservations`'s
/// lease-to-reservation mapping, expiry/ttl derivation) is defined in
/// `reservations.mjs`, not `state-projection.mjs` — a different mjs source
/// file outside this cell's `read_first`/action list (which names only
/// `state-projection.mjs`'s own five rebuild verbs) — and is not yet ported
/// into `crate::reservations` (that module is read-only today: it reads the
/// legacy projection and the live sharded leases, but never rebuilds the
/// former from the latter). Porting it is real, separable follow-on work,
/// not folded into this cell silently: this function rebuilds state,
/// handoff, and lanes (every must-have this cell's `verify` target proves)
/// and leaves the reservations leg for the cell that ports
/// `reservations.mjs`'s write side.
pub fn rebuild_all_projections(root: &Path, control_root: &Path) -> io::Result<AllProjectionsResult> {
    let state = rebuild_state_projection(root, control_root, &StateOverrides::default())?;
    let handoff = rebuild_handoff_projection(root, control_root)?;
    let workflows = list_workflows(control_root).workflows;
    let mut lanes = Vec::new();
    for wf in workflows.iter().filter(|w| w.status == "active") {
        lanes.push(rebuild_lane_projection(root, control_root, &wf.feature)?);
    }
    Ok(AllProjectionsResult { state, handoff, lanes })
}

// Tests live in crates/bee-core/tests/projection_parity.rs (this cell's
// single integration target — cargo test -p bee-core --test
// projection_parity), oracle-checked against the real
// workflow-store.mjs/state-projection.mjs via a file-based node driver.
