// the state projections
//
// Split out of the single 2.7k-line verbs/workflow_store.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, LockGuard, MAX_ATTEMPTS};
use crate::verbs::reservations::{
    date_parse_val, jget, js_disp, js_disp_opt, js_trim, now_iso, pseudo_uuid_v4,
    truthy, Err2, Ex, Exotic,
};
use crate::verbs::state_group::{
    adopt_claim, coerce_legacy_phase, default_gates, handoff_path, io_read_reason, parse_json_v8,
    read_claim, read_state_peek, spread_gates, write_state, AdoptOutcome, ParsedJson,
};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

// ─── projections (lib/state-projection.mjs) ────────────────────────────────

/// state-projection.mjs workflowGatesToApprovedGates(gates, planRev) — the
/// PLAN-REV-EFFECTIVE approval, in the fixed GATE_NAMES key order.
pub(crate) fn workflow_gates_to_approved_gates(gates: Option<&Value>, plan_rev: Option<&Value>) -> Value {
    let mut approved = Map::new();
    for name in GATE_NAMES {
        let entry = gates.and_then(|g| jget(g, name));
        let entry_truthy = entry.map(truthy).unwrap_or(false);
        let is_approved = entry_truthy
            && matches!(entry, Some(Value::Object(e)) if e.get("approved") == Some(&Value::Bool(true)));
        // `entry ? entry.approved_for_plan_rev : undefined`; property access on
        // a truthy primitive also yields undefined.
        let rev = if entry_truthy {
            match entry {
                Some(Value::Object(e)) => e.get("approved_for_plan_rev"),
                _ => None,
            }
        } else {
            None
        };
        let rev_effective = match rev {
            None | Some(Value::Null) => true,
            Some(v) => match plan_rev {
                Some(p) => v == p,
                None => false, // `rev === undefined` already handled; a real rev never === undefined
            },
        };
        approved.insert(name.to_string(), Value::Bool(is_approved && rev_effective));
    }
    Value::Object(approved)
}

/// state-projection.mjs pickNewestActiveWorkflow — active only, never
/// compounding-complete; created_at descending, then id descending.
pub(crate) fn pick_newest_active_workflow(
    workflows: &[Map<String, Value>],
) -> Ex<Option<&Map<String, Value>>> {
    let mut active: Vec<&Map<String, Value>> = Vec::new();
    for wf in workflows {
        if wf.get("status") == Some(&json!("active"))
            && wf.get("phase").unwrap_or(&Value::Null) != &json!("compounding-complete")
        {
            active.push(wf);
        }
    }
    if active.is_empty() {
        return Ok(None);
    }
    // `Date.parse(x.created_at) || 0` — NaN and 0 both collapse to 0.
    let stamp = |wf: &Map<String, Value>| -> Ex<f64> {
        Ok(date_parse_val(wf.get("created_at"))?.filter(|v| *v != 0.0).unwrap_or(0.0))
    };
    let mut keyed: Vec<(f64, String, &Map<String, Value>)> = Vec::new();
    for wf in active {
        keyed.push((stamp(wf)?, js_disp_opt(wf.get("id")), wf));
    }
    keyed.sort_by(|a, b| {
        if a.0 != b.0 {
            return b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal);
        }
        if a.1 == b.1 {
            return std::cmp::Ordering::Equal;
        }
        b.1.cmp(&a.1) // `a.id < b.id ? 1 : -1` — id descending
    });
    Ok(Some(keyed[0].2))
}

/// `next[key] = wf[key]` with JS undefined semantics: an absent source key
/// makes the destination key `undefined`, which JSON.stringify drops.
pub(crate) fn set_from(next: &mut Map<String, Value>, key: &str, wf: &Map<String, Value>) {
    match wf.get(key) {
        Some(v) => {
            next.insert(key.to_string(), v.clone());
        }
        None => {
            next.shift_remove(key);
        }
    }
}

pub(crate) fn apply_workflow_d1_fields(next: &mut Map<String, Value>, wf: &Map<String, Value>) {
    set_from(next, "phase", wf);
    set_from(next, "feature", wf);
    set_from(next, "mode", wf);
    next.insert(
        "approved_gates".into(),
        workflow_gates_to_approved_gates(wf.get("gates"), wf.get("plan_rev")),
    );
    set_from(next, "summary", wf);
    set_from(next, "next_action", wf);
    // D4/D7: run_state is WRITTEN on the record (record.rs's
    // derive_run_state); this list is the only path it takes into
    // `.bee/state.json`. A new record field never reaches the projection
    // unless it is named here — this is the exact trap plan.md calls out,
    // pinned by
    // `apply_workflow_d1_fields_carries_run_state_...` in tests.rs.
    set_from(next, "run_state", wf);
}

/// state-projection.mjs rebuildStateProjection(root) with NO overrides — the
/// CLI's own shape (bee-state-sync is the only caller that passes
/// cellCounts/lastActivity, and that path lives in hooks/state_sync.rs). With
/// no overrides `applyOverridesOnly()` never writes, so every non-authoritative
/// branch here is a pure no-op.
pub(crate) fn rebuild_state_projection(root: &Path) -> Result<(), Err2> {
    rebuild_state_projection_reporting(root).map(|_| ())
}

/// The `{ authoritative, source, state|lane }` triple state-projection.mjs's
/// rebuild* functions return. Only `state rebuild-projections` reports it —
/// every other caller writes for effect, so `rebuild_state_projection` /
/// `rebuild_lane_projection` stay the thin shapes they already had.
pub(crate) struct Proj<T> {
    pub(crate) authoritative: bool,
    pub(crate) source: Value,
    pub(crate) record: T,
}

/// provenance: state-projection.mjs rebuildStateProjection's own return value
/// (see `rebuild_state_projection` above for the body's provenance).
pub(crate) fn rebuild_state_projection_reporting(
    root: &Path,
) -> Result<Proj<Map<String, Value>>, Err2> {
    let workflows = list_workflows(root)?;
    let current = read_state_peek(root)?; // read in Node's own order
    // applyOverridesOnly() with no overrides — never writes, `state: current`.
    let no_op = |state: Map<String, Value>| Proj {
        authoritative: false,
        source: Value::Null,
        record: state,
    };
    if workflows.is_empty() {
        return Ok(no_op(current)); // C1: zero workflow records — no write at all
    }
    let feature = current.get("feature").cloned().unwrap_or(Value::Null);
    if truthy(&feature) {
        // Branch (1) — feature-matched (msn-10).
        // A non-string truthy feature never `===` a record's string feature.
        if let Value::String(f) = &feature {
            if let Some(wf) = find_live_workflow(&workflows, f) {
                let source = wf.get("id").cloned().unwrap_or(Value::Null);
                let mut next = current.clone();
                apply_workflow_d1_fields(&mut next, wf);
                write_state(root, &next)?;
                return Ok(Proj { authoritative: true, source, record: next });
            }
        }
        // feature set, no live workflow names it → the idle-bootstrap branch
        // below requires `!current.feature`, so this is always a no-op.
        return Ok(no_op(current));
    }
    // Branch (2) — idle bootstrap (msn-7).
    let phase = current.get("phase").cloned().unwrap_or(Value::Null);
    let current_is_idle = &phase == &json!("idle")
        || &phase == &json!("compounding-complete")
        || !truthy(&phase);
    if !current_is_idle {
        return Ok(no_op(current));
    }
    let active = pick_newest_active_workflow(&workflows)?;
    let source = match active {
        Some(wf) => wf.get("id").cloned().unwrap_or(Value::Null),
        None => Value::Null,
    };
    let mut next = current.clone();
    match active {
        Some(wf) => apply_workflow_d1_fields(&mut next, wf),
        None => {
            next.insert("phase".into(), json!("idle"));
            next.insert("feature".into(), Value::Null);
            next.insert("mode".into(), Value::Null);
            next.insert("approved_gates".into(), Value::Object(default_gates()));
            next.insert("summary".into(), json!(""));
            next.insert(
                "next_action".into(),
                json!("No active bee work \u{2014} awaiting a user request."),
            );
        }
    }
    write_state(root, &next)?;
    Ok(Proj { authoritative: true, source, record: next })
}

/// state-projection.mjs rebuildLaneProjection(root, feature). Returns the
/// rebuilt lane record when the projection took authority, otherwise the
/// existing (fail-open) lane read — exactly `rebuilt.lane`.
pub(crate) fn rebuild_lane_projection(
    root: &Path,
    feature: &str,
) -> Result<Option<Map<String, Value>>, Err2> {
    Ok(rebuild_lane_projection_reporting(root, feature)?.record)
}

/// provenance: state-projection.mjs rebuildLaneProjection's own return value.
pub(crate) fn rebuild_lane_projection_reporting(
    root: &Path,
    feature: &str,
) -> Result<Proj<Option<Map<String, Value>>>, Err2> {
    let workflows = list_workflows(root)?;
    let no_op = |lane| Proj { authoritative: false, source: Value::Null, record: lane };
    if workflows.is_empty() {
        return Ok(no_op(read_lane_display(root, feature)?));
    }
    let Some(wf) = find_live_workflow(&workflows, feature) else {
        return Ok(no_op(read_lane_display(root, feature)?));
    };
    let source = wf.get("id").cloned().unwrap_or(Value::Null);
    let existing = read_lane_display(root, feature)?;
    let mut next = existing.clone().unwrap_or_default();
    next.insert("schema_version".into(), json!("1.0"));
    set_from(&mut next, "feature", wf);
    set_from(&mut next, "mode", wf);
    set_from(&mut next, "phase", wf);
    next.insert(
        "approved_gates".into(),
        workflow_gates_to_approved_gates(wf.get("gates"), wf.get("plan_rev")),
    );
    set_from(&mut next, "summary", wf);
    set_from(&mut next, "next_action", wf);
    // D4/D7: same additive field as apply_workflow_d1_fields (this function
    // duplicates that field list rather than calling it — kept in step so a
    // lane reader sees run_state too).
    set_from(&mut next, "run_state", wf);
    // `(existing && existing.created_at) || wf.created_at || new Date()...`
    let created_at = existing
        .as_ref()
        .and_then(|e| e.get("created_at"))
        .filter(|v| truthy(v))
        .or_else(|| wf.get("created_at").filter(|v| truthy(v)))
        .cloned()
        .unwrap_or_else(|| json!(now_iso()));
    next.insert("created_at".into(), created_at);
    write_lane(root, &next)?;
    Ok(Proj { authoritative: true, source, record: Some(next) })
}

/// state-projection.mjs rebuildHandoffProjection(root) — the legacy
/// .bee/HANDOFF.json as a projection of the newest OPEN mailbox record across
/// every workflow. No-op at zero workflow records (C1); removes the legacy
/// file when workflows exist but none carries an open handoff.
pub(crate) fn rebuild_handoff_projection(root: &Path) -> Result<(), Err2> {
    rebuild_handoff_projection_reporting(root).map(|_| ())
}

/// provenance: state-projection.mjs rebuildHandoffProjection's own return value.
pub(crate) fn rebuild_handoff_projection_reporting(root: &Path) -> Result<Proj<()>, Err2> {
    let workflows = list_workflows(root)?;
    if workflows.is_empty() {
        return Ok(Proj { authoritative: false, source: Value::Null, record: () });
    }
    let mut newest: Option<(Map<String, Value>, String)> = None;
    for wf in &workflows {
        let id = wf_id(wf);
        let open: Vec<Map<String, Value>> = list_handoff_mailbox(root, &id)?
            .into_iter()
            .filter(|r| matches!(r.get("status"), Some(Value::String(s)) if s == "open"))
            .collect();
        let Some(candidate) = open.last().cloned() else { continue };
        match &newest {
            None => newest = Some((candidate, id)),
            Some((cur, cur_id)) => {
                let a = date_parse_val(candidate.get("written_at"))?
                    .filter(|v| *v != 0.0)
                    .unwrap_or(0.0);
                let b = date_parse_val(cur.get("written_at"))?
                    .filter(|v| *v != 0.0)
                    .unwrap_or(0.0);
                if a > b || (a == b && id > *cur_id) {
                    newest = Some((candidate, id));
                }
            }
        }
    }
    let Some((record, source_id)) = newest else {
        let _ = std::fs::remove_file(handoff_path(root)); // rmSync force:true
        return Ok(Proj { authoritative: true, source: Value::Null, record: () });
    };
    // Drop the mailbox-only fields so a legacy reader sees writeHandoff's shape.
    let mut projected = record;
    for key in ["seq", "status", "id", "workflow_id", "target_role", "from_session"] {
        projected.shift_remove(key);
    }
    write_json_atomic(&handoff_path(root), &Value::Object(projected)).map_err(|_| Err2::Ex)?;
    Ok(Proj { authoritative: true, source: json!(source_id), record: () })
}
