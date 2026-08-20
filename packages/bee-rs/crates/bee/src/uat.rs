//! uat-stop-placement (docs/history/uat-stop-placement/CONTEXT.md D1, D2):
//! the whole uat-placement policy, stated once, so the merge side and the
//! close side never carry two copies of it.

use serde_json::Value;
use std::path::Path;

/// D1: where the uat stop sits for a feature — `Merge` (the door blocks
/// `bee worktree merge`), `Close` (default when both `uat_stop` and
/// `uat_before_merge` are absent: merge first, accept after — the door
/// moves to `bee close` instead), or `Off` (no uat stop anywhere).
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
/// `Close` — absent means the merge lands on main and the uat door sits at
/// `bee close`, the worktree held while uat is pending. Reads the merged
/// tracked+overlay config via `crate::state::read_config_raw`.
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
            None => Some(UatStop::Close),
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

/// uat-stop-placement D4 revision (usp-3): the ONE lane-classification read
/// for the uat door, lifted verbatim from the merge side's
/// `uat_merge_precheck` (`verbs/worktree/phases.rs`) so `bee close` and
/// `bee worktree merge` never disagree on which lane a feature is in.
///
/// Before this, `close`'s uat door classified through `feature_route`
/// (which prefers a lane record's `route.lane`) while `merge` classified
/// through the live workflow's — or the lane record's — `mode`. The two
/// disagree whenever a record's `route.lane` and `mode` name different lane
/// classes (12 of 95 real records in `.bee/lanes` at the time of this fix,
/// e.g. `.bee/lanes/knowledge-loop.json`: `mode` "standard", `route.lane`
/// "small"): under `uat_stop: "close"` the merge side would set the uat
/// wait while `bee close` reported the same feature exempt, and the stop
/// vanished silently.
///
/// Read order, exactly mirroring the merge side: prefer the live workflow
/// record's own `mode` field, falling back to `.bee/lanes/<feature>.json`'s
/// `mode` (`read_lane_display`, the same fail-open display read `close`'s
/// own scoping already reuses) when no live workflow names the feature.
/// `route.lane` is never consulted here — only `judge-debt` and the other
/// doors that call `feature_route` directly still read it, unchanged.
/// Every read is fail-open by construction (`list_workflows`,
/// `read_lane_display` never throw for an ordinary missing/corrupt shape),
/// so an unreadable store returns `None`, which `uat_gate_applies_to_lane`
/// then reads as "standard" (applies) — the safe direction.
pub(crate) fn uat_lane_mode(main_root: &Path, feature: &str) -> Option<String> {
    let workflows = crate::verbs::workflow_store::list_workflows(main_root).unwrap_or_default();
    let live = crate::verbs::workflow_store::find_live_workflow(&workflows, feature);
    live.and_then(|wf| wf.get("mode"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            crate::verbs::workflow_store::read_lane_display(main_root, feature)
                .ok()
                .flatten()
                .and_then(|rec| rec.get("mode").and_then(Value::as_str).map(str::to_string))
        })
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
    fn both_keys_absent_resolves_to_close() {
        let tmp = tempfile::tempdir().unwrap();
        write_config(tmp.path(), r#"{}"#);
        assert_eq!(uat_stop_config(tmp.path()), Some(UatStop::Close));
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
}
