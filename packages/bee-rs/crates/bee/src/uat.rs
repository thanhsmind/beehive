//! uat-stop-placement (docs/history/uat-stop-placement/CONTEXT.md D1, D2):
//! the whole uat-placement policy, stated once, so the merge side and the
//! close side never carry two copies of it.

use serde_json::{Map, Value};
use std::path::Path;

/// D1: where the uat stop sits for a feature — `Merge` (default, today's
/// behavior: the door blocks `bee worktree merge`), `Close` (merge first,
/// accept after — the door moves to `bee close` instead), or `Off` (no uat
/// stop anywhere).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum UatStop {
    Merge,
    Close,
    Off,
}

/// D1: the uat-stop-placement read order, stated once and only here.
///
/// `.bee/config.json`'s `uat_stop` wins whenever it is present: the strings
/// `"merge"`/`"close"`/`"off"` map to the three variants, and ANY other
/// value — a typo'd string, or a non-string shape — fails closed with
/// `None` rather than guessing. Else the back-compat alias
/// `uat_before_merge` is read: `true` => `Merge`, `false` => `Off`, and any
/// non-boolean shape fails closed with `None`. Else, with both keys absent,
/// `Merge` — absent means today's behavior, unchanged for every existing
/// repo. Reads the merged tracked+overlay config via
/// `crate::state::read_config_raw`.
pub(crate) fn uat_stop_config(main_root: &Path) -> Option<UatStop> {
    let config = crate::state::read_config_raw(main_root);
    match config.get("uat_stop") {
        Some(Value::String(s)) => match s.as_str() {
            "merge" => Some(UatStop::Merge),
            "close" => Some(UatStop::Close),
            "off" => Some(UatStop::Off),
            _ => None,
        },
        Some(_) => None,
        None => match config.get("uat_before_merge") {
            Some(Value::Bool(true)) => Some(UatStop::Merge),
            Some(Value::Bool(false)) => Some(UatStop::Off),
            Some(_) => None,
            None => Some(UatStop::Merge),
        },
    }
}

/// D2, MOVED verbatim from `verbs/worktree/phases.rs:698-700` (formerly
/// named the same there): does `mode` (a record's risk-lane classification)
/// require the uat door — at merge under `uat_stop: "merge"`, or at close
/// under any placement? Only the known LOW-risk lanes are exempt
/// (`tiny`/`small`/`docs`/`spike`, i.e. `ROUTE_LANE_VALUES` minus
/// `standard`/`high-risk`) — a missing record, a null mode, or any value
/// this port does not recognize fails CLOSED as "standard", because an
/// unclassified feature is exactly the case a silent skip would be most
/// dangerous for.
pub(crate) fn uat_gate_applies_to_lane(mode: Option<&str>) -> bool {
    !matches!(mode, Some("tiny") | Some("small") | Some("docs") | Some("spike"))
}

/// docs/history/uat-approval-reaches-the-door/plan.md R1-R3: the single
/// `uat` gate-approval resolver both doors call — the merge-time
/// `uat_merge_precheck` (verbs/worktree/phases.rs) and the close-time door
/// (verbs/drivers/close.rs). Formerly two byte-identical copies, each
/// missing the source below — the defect this function exists to close.
///
/// Fixed three-source precedence. Source 1 stops the cascade on its own
/// presence alone, approved or not — unchanged from before this fix.
/// Sources 2 and 3 do NOT stand alone the same way: `read_lane_display` and
/// `read_state_peek` both merge the record they find over a shared gate
/// default (`crate::state::spread_gates`) that already stamps `uat: false`
/// on ANY record missing an opinion, live or default — so "the lane record
/// has no opinion on uat" and "the lane record explicitly says false" are
/// indistinguishable data once merged, by construction, everywhere in this
/// store. Once no live record stands, this function reads as approved if
/// EITHER remaining source's `approved_gates.uat` is a literal `true` —
/// never a strict fallback chain, an OR:
///
/// 1. the live workflow record's own `gates.uat.approved`
///    (`find_live_workflow`) — unchanged, still first. A live record saying
///    `false` beats a lane file saying `true`; this function never looks
///    past a live record once one is found.
/// 2. NEW: failing that (no live record — closed, or never opened), the
///    LANE record `.bee/lanes/<feature>.json`'s `approved_gates.uat`, read
///    through the existing display reader `read_lane_display` — never by
///    hand-parsing the file. This is the file `bee gate --lane <f>` writes
///    when the live-record lookup comes up empty, and the file neither
///    door used to read.
/// 3. the default `.bee/state.json`'s `approved_gates.uat`, but ONLY when
///    that record is presently tracking THIS feature — a foreign feature's
///    approval must never leak through as "approved" for a different one.
///    Unchanged, for the unbound default-record case — including a lane
///    record that exists but never opined on `uat` (e.g. one carrying only
///    `mode`), which must still let a genuine default-state approval
///    through rather than being shadowed by the lane's merged-in default.
///
/// Every source fails CLOSED on anything but a literal JSON `true`: a
/// missing gate, a missing `approved_gates`, `false`, or a non-boolean
/// value all read as "not approved" — this widens WHERE an approval is
/// found, never WHAT counts as one. Every read here is fail-open by
/// construction (`list_workflows`, `read_lane_display`, `read_state_peek`
/// never throw for an ordinary missing/corrupt shape), so an unreadable
/// store reads as "not approved" too.
pub(crate) fn uat_gate_approved(main_root: &Path, feature: &str) -> bool {
    let workflows = crate::verbs::workflow_store::list_workflows(main_root).unwrap_or_default();
    if let Some(wf) = crate::verbs::workflow_store::find_live_workflow(&workflows, feature) {
        return matches!(
            wf.get("gates").and_then(|g| g.get("uat")).and_then(|e| e.get("approved")),
            Some(Value::Bool(true))
        );
    }

    let lane_approved = crate::verbs::workflow_store::read_lane_display(main_root, feature)
        .ok()
        .flatten()
        .is_some_and(|lane| {
            matches!(
                lane.get("approved_gates").and_then(|g| g.get("uat")),
                Some(Value::Bool(true))
            )
        });

    let default_state_approved = crate::verbs::state_group::read_state_peek(main_root)
        .ok()
        .filter(|state| matches!(state.get("feature"), Some(Value::String(f)) if f == feature))
        .is_some_and(|state| {
            matches!(
                state.get("approved_gates").and_then(|g| g.get("uat")),
                Some(Value::Bool(true))
            )
        });

    lane_approved || default_state_approved
}

/// uls-1 revision (uat-lane-source): `uat_lane_mode` resolves the LANE
/// classification, not the workflow mode. MEASURED on the live store:
/// `mode` carries the WORKFLOW vocabulary (`ROUTE_CLASS_VALUES`:
/// "feature", "release", … — what `state start-feature --mode` writes),
/// while `route.lane` carries the ROUTE-LANE vocabulary
/// (`ROUTE_LANE_VALUES`, verbs/state_group/workflows.rs:289-290:
/// tiny/small/standard/high-risk/docs/spike — what `bee route --set
/// --lane` writes). Because `mode` is almost never a lane value,
/// `uat_gate_applies_to_lane`'s tiny/small/docs/spike exemption had
/// effectively never fired, and every feature was asked for uat
/// regardless of lane — over-strict, never unsafe, which is exactly why
/// no test and no user caught it.
///
/// The prior revision (usp-3) fixed a REAL two-source divergence: the
/// merge side (`uat_merge_precheck`, `verbs/worktree/phases.rs`) and the
/// close side (`feature_route`) disagreed whenever a record's
/// `route.lane` and `mode` named different lane classes (12 of 95 real
/// records in `.bee/lanes` at the time, e.g.
/// `.bee/lanes/knowledge-loop.json`: `mode` "standard", `route.lane`
/// "small"). That divergence was real, and unifying both doors onto ONE
/// function was the right shape. But usp-3's cell brief named the MERGE
/// side canonical by instruction rather than by evidence — and the merge
/// side was reading `mode`, the wrong field. This revision keeps the
/// one-function shape usp-3 established and corrects which field it
/// reads.
///
/// Read order: the live workflow record's own `route.lane` first, then
/// `.bee/lanes/<feature>.json`'s `route.lane` (`read_lane_display`, the
/// same fail-open display read this file already uses elsewhere) when no
/// live workflow names the feature, then — ONLY as a legacy fallback for
/// older records that never got a `route.lane` written — `mode` (live
/// first, then lane), and ONLY when that value is itself a recognized
/// member of the route-lane vocabulary. Mirrors `ROUTE_LANE_VALUES`
/// (verbs/state_group/workflows.rs:289-290) without importing it: that
/// const is module-private there. A `mode` of "feature", "release", or
/// anything else outside that set contributes nothing. Every read is
/// fail-open by construction (`list_workflows`, `read_lane_display` never
/// throw for an ordinary missing/corrupt shape), so an unreadable store
/// returns `None`, which `uat_gate_applies_to_lane` then reads as
/// "standard" (applies) — the safe direction, unchanged.
const ROUTE_LANE_CLASSES: [&str; 6] = ["docs", "tiny", "small", "spike", "standard", "high-risk"];

pub(crate) fn uat_lane_mode(main_root: &Path, feature: &str) -> Option<String> {
    let workflows = crate::verbs::workflow_store::list_workflows(main_root).unwrap_or_default();
    let live = crate::verbs::workflow_store::find_live_workflow(&workflows, feature);
    let lane_record = crate::verbs::workflow_store::read_lane_display(main_root, feature)
        .ok()
        .flatten();

    let route_lane = |v: &Map<String, Value>| -> Option<String> {
        v.get("route")
            .and_then(|r| r.get("lane"))
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let mode_as_lane = |v: &Map<String, Value>| -> Option<String> {
        v.get("mode")
            .and_then(Value::as_str)
            .filter(|m| ROUTE_LANE_CLASSES.contains(m))
            .map(str::to_string)
    };

    live.and_then(route_lane)
        .or_else(|| lane_record.as_ref().and_then(route_lane))
        .or_else(|| live.and_then(mode_as_lane))
        .or_else(|| lane_record.as_ref().and_then(mode_as_lane))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_config(root: &Path, body: &str) {
        fs::create_dir_all(root.join(".bee")).unwrap();
        fs::write(root.join(".bee").join("config.json"), body).unwrap();
    }

    #[test]
    fn uat_stop_string_merge_reads_as_merge() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), r#"{"uat_stop": "merge"}"#);
        assert_eq!(uat_stop_config(tmp.path()), Some(UatStop::Merge));
    }

    #[test]
    fn uat_stop_string_close_reads_as_close() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), r#"{"uat_stop": "close"}"#);
        assert_eq!(uat_stop_config(tmp.path()), Some(UatStop::Close));
    }

    #[test]
    fn uat_stop_string_off_reads_as_off() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), r#"{"uat_stop": "off"}"#);
        assert_eq!(uat_stop_config(tmp.path()), Some(UatStop::Off));
    }

    #[test]
    fn uat_stop_wins_over_a_contradicting_uat_before_merge() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), r#"{"uat_stop": "close", "uat_before_merge": false}"#);
        assert_eq!(uat_stop_config(tmp.path()), Some(UatStop::Close));
    }

    #[test]
    fn uat_before_merge_true_reads_as_merge_when_uat_stop_is_absent() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), r#"{"uat_before_merge": true}"#);
        assert_eq!(uat_stop_config(tmp.path()), Some(UatStop::Merge));
    }

    #[test]
    fn uat_before_merge_false_reads_as_off_when_uat_stop_is_absent() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), r#"{"uat_before_merge": false}"#);
        assert_eq!(uat_stop_config(tmp.path()), Some(UatStop::Off));
    }

    #[test]
    fn both_keys_absent_resolves_to_merge() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), r#"{}"#);
        assert_eq!(uat_stop_config(tmp.path()), Some(UatStop::Merge));
    }

    #[test]
    fn a_bogus_uat_stop_string_fails_closed_with_none() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), r#"{"uat_stop": "sometime"}"#);
        assert_eq!(uat_stop_config(tmp.path()), None);
    }

    #[test]
    fn a_non_string_uat_stop_fails_closed_with_none() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), r#"{"uat_stop": 1}"#);
        assert_eq!(uat_stop_config(tmp.path()), None);
    }

    #[test]
    fn a_non_boolean_uat_before_merge_fails_closed_with_none() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), r#"{"uat_before_merge": "yes"}"#);
        assert_eq!(uat_stop_config(tmp.path()), None);
    }

    #[test]
    fn lane_rule_exempts_tiny_small_docs_spike() {
        assert!(!uat_gate_applies_to_lane(Some("tiny")));
        assert!(!uat_gate_applies_to_lane(Some("small")));
        assert!(!uat_gate_applies_to_lane(Some("docs")));
        assert!(!uat_gate_applies_to_lane(Some("spike")));
    }

    #[test]
    fn lane_rule_applies_to_standard_high_risk_none_and_unknown() {
        assert!(uat_gate_applies_to_lane(Some("standard")));
        assert!(uat_gate_applies_to_lane(Some("high-risk")));
        assert!(uat_gate_applies_to_lane(None));
        assert!(uat_gate_applies_to_lane(Some("bogus-lane")));
    }

    // ─── uls-1: uat_lane_mode resolves route.lane, not mode ────────────────

    fn write_workflow(root: &Path, id: &str, body: &str) {
        let dir = crate::verbs::workflow_store::workflows_dir(root).join(id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("state.json"), body).unwrap();
    }

    fn write_lane(root: &Path, feature: &str, body: &str) {
        let dir = crate::verbs::workflow_store::lanes_dir(root);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("{feature}.json")), body).unwrap();
    }

    #[test]
    fn mode_feature_with_route_lane_small_resolves_small_and_is_exempt() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "wf-1",
            r#"{"id": "wf-1", "feature": "f1", "status": "active", "mode": "feature", "route": {"lane": "small"}}"#,
        );
        let lane = uat_lane_mode(tmp.path(), "f1");
        assert_eq!(lane.as_deref(), Some("small"));
        assert!(!uat_gate_applies_to_lane(lane.as_deref()));
    }

    #[test]
    fn mode_feature_with_route_lane_standard_resolves_standard_and_applies() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "wf-2",
            r#"{"id": "wf-2", "feature": "f2", "status": "active", "mode": "feature", "route": {"lane": "standard"}}"#,
        );
        let lane = uat_lane_mode(tmp.path(), "f2");
        assert_eq!(lane.as_deref(), Some("standard"));
        assert!(uat_gate_applies_to_lane(lane.as_deref()));
    }

    #[test]
    fn legacy_mode_standard_with_no_route_resolves_standard() {
        let tmp = tempfile::tempdir().unwrap();
        write_lane(tmp.path(), "f3", r#"{"feature": "f3", "mode": "standard"}"#);
        assert_eq!(uat_lane_mode(tmp.path(), "f3").as_deref(), Some("standard"));
    }

    #[test]
    fn route_lane_wins_over_a_mode_that_also_looks_like_a_lane() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "wf-4",
            r#"{"id": "wf-4", "feature": "f4", "status": "active", "mode": "standard", "route": {"lane": "small"}}"#,
        );
        assert_eq!(uat_lane_mode(tmp.path(), "f4").as_deref(), Some("small"));
    }

    #[test]
    fn mode_feature_with_no_route_yields_none_and_applies() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "wf-5",
            r#"{"id": "wf-5", "feature": "f5", "status": "active", "mode": "feature"}"#,
        );
        let lane = uat_lane_mode(tmp.path(), "f5");
        assert_eq!(lane, None);
        assert!(uat_gate_applies_to_lane(lane.as_deref()));
    }

    #[test]
    fn live_workflow_route_lane_beats_the_lane_record() {
        let tmp = tempfile::tempdir().unwrap();
        write_workflow(
            tmp.path(),
            "wf-6",
            r#"{"id": "wf-6", "feature": "f6", "status": "active", "mode": "feature", "route": {"lane": "standard"}}"#,
        );
        write_lane(
            tmp.path(),
            "f6",
            r#"{"feature": "f6", "mode": "feature", "route": {"lane": "small"}}"#,
        );
        assert_eq!(uat_lane_mode(tmp.path(), "f6").as_deref(), Some("standard"));
    }
}
