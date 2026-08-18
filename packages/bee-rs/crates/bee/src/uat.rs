//! uat-stop-placement (docs/history/uat-stop-placement/CONTEXT.md D1, D2):
//! the whole uat-placement policy, stated once, so the merge side and the
//! close side never carry two copies of it.

use serde_json::Value;
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
}
