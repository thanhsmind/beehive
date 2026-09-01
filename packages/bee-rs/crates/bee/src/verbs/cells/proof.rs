// D7/D8 (docs/history/test-doctrine/CONTEXT.md) — the shared proof-check
// helper the boundary doors (`bee close`, and `bee worktree merge` in its
// own cell) read instead of spawning `commands.test` themselves. td-1
// (finish_support.rs `parse_report_flag`/`parse_tests_proof`) already made
// every NEW cap's `trace.report.tests` a validated D8 proof string; this
// module is the read side — it walks a feature's capped cells (including
// its archive, dda-1's own "a closed feature's cells still count" posture)
// and classifies each one:
//   - no `trace.report` at all: a legacy cap, from before `--report` was
//     required — passes ungated, never refused, so a pre-contract feature
//     can still close.
//   - `trace.report.tests` a well-formed D8 proof string: proven — the cap
//     already carries what the boundary needs.
//   - anything else with a `trace.report` present (missing `tests`, an
//     empty string, one that fails `parse_tests_proof`): a cap that CLAIMS
//     the D8 contract but does not actually carry a valid proof line — this
//     is the one shape the door refuses, naming the cell so a cold reader
//     knows exactly which cap to redo.
//
// This module owns only the READ + classify step. Every door built on top
// of it (close.rs today) decides its own wording, headline, and command —
// same split `scribing_debt`/`judge_debt` (drivers/close.rs) already use:
// a `{count, ids}`-shaped summary in, door prose out.

use super::*;
use serde_json::Value;
use std::path::Path;

/// One feature's proof-check verdict: `blocking` is true the moment even
/// one capped cell carries a `trace.report` with no valid proof line —
/// `bad_ids` names every such cell (never just the first) so a refusal
/// lists the whole remedy set in one pass. `proven_count`/`legacy_count`
/// are display-only tallies for the door's own "GREEN" wording.
pub(crate) struct ProofCheck {
    pub(crate) blocking: bool,
    pub(crate) bad_ids: Vec<String>,
    pub(crate) proven_count: usize,
    pub(crate) legacy_count: usize,
    /// D4 (docs/history/proof-strength-and-expiry/CONTEXT.md): `(cell id,
    /// the commit that cap's report recorded)` for every PROVEN cap that
    /// recorded a real one. Read exactly the way `handlers_close.rs` reads
    /// the same key — the `"none"` sentinel (and an empty string) is dropped
    /// HERE, so a cap that never recorded a commit is absent rather than
    /// carrying an uncomparable value that a caller could measure as stale.
    /// Store data only: whether a commit is old is a git question, and this
    /// module runs no git.
    pub(crate) commits: Vec<(String, String)>,
}

/// The D8 proof-check itself: every `status: "capped"` cell for `feature`
/// (live store + archive — `list_cells_including_archive`, the same
/// archive-inclusive read `scribing_debt`/`judge_debt` already use, so an
/// auto-archived cell still counts) is classified per the module doc above.
/// `Err(Delegate)` only ever comes from the underlying store read (a JS-
/// exotic shape this port cannot carry) — never from a proof-string
/// classification, which always has a definite answer.
pub(crate) fn feature_proof_check(
    root: &Path,
    feature: &str,
) -> Result<ProofCheck, crate::verbs::drivers::Delegate> {
    let mut bad_ids: Vec<String> = Vec::new();
    let mut commits: Vec<(String, String)> = Vec::new();
    let mut proven_count = 0usize;
    let mut legacy_count = 0usize;
    for cell in crate::verbs::drivers::list_cells_including_archive(root, feature, Some("capped"))? {
        let report = cell.get("trace").and_then(|t| t.get("report"));
        let Some(report) = report else {
            legacy_count += 1;
            continue;
        };
        let valid = matches!(report.get("tests"), Some(Value::String(s)) if parse_tests_proof(s).is_some());
        let id = cell.get("id").and_then(Value::as_str).unwrap_or("").to_string();
        if valid {
            proven_count += 1;
            let commit = report
                .get("commit")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_string();
            if !id.is_empty() && !commit.is_empty() && commit != "none" {
                commits.push((id, commit));
            }
            continue;
        }
        if !id.is_empty() {
            bad_ids.push(id);
        }
    }
    Ok(ProofCheck { blocking: !bad_ids.is_empty(), bad_ids, proven_count, legacy_count, commits })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn w(root: &Path, rel: &str, body: &str) {
        let file = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, body).unwrap();
    }

    /// D2 (docs/history/proof-strength-and-expiry/CONTEXT.md) — this is the
    /// write/read split's own evidence. The fixture's bare `green` is the
    /// form `parse_report_flag` now REFUSES on a new cap, and it still reads
    /// as proven here, because `feature_proof_check` checks shape only. Keep
    /// the bare `green`: qualifying it would delete the assertion that ~200
    /// historical caps stay closeable.
    #[test]
    fn a_capped_cell_with_a_valid_proof_line_is_proven_not_blocking() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(
            root,
            ".bee/cells/demo-1.json",
            &json!({
                "id": "demo-1",
                "feature": "demo",
                "status": "capped",
                "trace": {
                    "report": {
                        "outcome": "did the thing",
                        "commit": "abc123",
                        "files": ["src/a.rs"],
                        "tests": "cargo test -p bee — green — touched a.rs",
                        "deviations": []
                    }
                }
            })
            .to_string(),
        );
        let check = feature_proof_check(root, "demo").unwrap();
        assert!(!check.blocking, "a valid proof line must never block");
        assert_eq!(check.proven_count, 1);
        assert_eq!(check.legacy_count, 0);
        assert!(check.bad_ids.is_empty());
    }

    #[test]
    fn a_present_but_empty_proof_refuses_naming_the_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(
            root,
            ".bee/cells/demo-2.json",
            &json!({
                "id": "demo-2",
                "feature": "demo",
                "status": "capped",
                "trace": {
                    "report": {
                        "outcome": "did the thing",
                        "commit": "abc123",
                        "files": [],
                        "tests": "",
                        "deviations": []
                    }
                }
            })
            .to_string(),
        );
        let check = feature_proof_check(root, "demo").unwrap();
        assert!(check.blocking, "an empty proof string must refuse");
        assert_eq!(check.bad_ids, vec!["demo-2".to_string()]);
    }

    #[test]
    fn a_malformed_proof_string_missing_a_segment_refuses() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(
            root,
            ".bee/cells/demo-3.json",
            &json!({
                "id": "demo-3",
                "feature": "demo",
                "status": "capped",
                "trace": {
                    "report": {
                        "outcome": "did the thing",
                        "commit": "abc123",
                        "files": [],
                        "tests": "cargo test -p bee — green",
                        "deviations": []
                    }
                }
            })
            .to_string(),
        );
        let check = feature_proof_check(root, "demo").unwrap();
        assert!(check.blocking, "a two-segment string must refuse — three are required");
        assert_eq!(check.bad_ids, vec!["demo-3".to_string()]);
    }

    #[test]
    fn a_capped_cell_with_no_report_at_all_is_legacy_and_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(
            root,
            ".bee/cells/demo-4.json",
            &json!({
                "id": "demo-4",
                "feature": "demo",
                "status": "capped",
                "trace": { "behavior_change": true, "capped_at": "2026-07-01T00:00:00.000Z" }
            })
            .to_string(),
        );
        let check = feature_proof_check(root, "demo").unwrap();
        assert!(!check.blocking, "a report-less legacy cap must never refuse");
        assert_eq!(check.legacy_count, 1);
        assert_eq!(check.proven_count, 0);
        assert!(check.bad_ids.is_empty());
    }

    #[test]
    fn a_bad_cap_alongside_a_good_one_names_only_the_bad_one() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(
            root,
            ".bee/cells/demo-good.json",
            &json!({
                "id": "demo-good",
                "feature": "demo",
                "status": "capped",
                "trace": {
                    "report": {
                        "outcome": "o",
                        "commit": "c",
                        "files": [],
                        // D2 evidence, like the fixture above: a historical
                        // bare `green` still reads as proven. Never qualify it.
                        "tests": "cargo test -p bee — green — touched x.rs",
                        "deviations": []
                    }
                }
            })
            .to_string(),
        );
        w(
            root,
            ".bee/cells/demo-bad.json",
            &json!({
                "id": "demo-bad",
                "feature": "demo",
                "status": "capped",
                "trace": {
                    "report": { "outcome": "o", "commit": "c", "files": [], "tests": "not a proof string", "deviations": [] }
                }
            })
            .to_string(),
        );
        let check = feature_proof_check(root, "demo").unwrap();
        assert!(check.blocking);
        assert_eq!(check.bad_ids, vec!["demo-bad".to_string()]);
        assert_eq!(check.proven_count, 1);
    }

    #[test]
    fn a_feature_with_no_capped_cells_at_all_never_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        let check = feature_proof_check(root, "demo").unwrap();
        assert!(!check.blocking);
        assert_eq!(check.proven_count, 0);
        assert_eq!(check.legacy_count, 0);
        assert!(check.bad_ids.is_empty());
    }

    /// D4: the commit-carrying read the merge door's staleness advisory
    /// measures. A recorded sha is carried with its cell id; the `"none"`
    /// sentinel `bee cells finish` writes when a cap has no commit is
    /// dropped here, so no caller can ever measure it as an old commit.
    #[test]
    fn a_recorded_commit_is_carried_and_the_none_sentinel_is_dropped() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let report = |commit: &str| {
            json!({
                "outcome": "o",
                "commit": commit,
                "files": [],
                "tests": "cargo test -p bee — green:unit — touched a.rs",
                "deviations": []
            })
        };
        w(
            root,
            ".bee/cells/demo-1.json",
            &json!({"id": "demo-1", "feature": "demo", "status": "capped", "trace": {"report": report("abc123")}})
                .to_string(),
        );
        w(
            root,
            ".bee/cells/demo-2.json",
            &json!({"id": "demo-2", "feature": "demo", "status": "capped", "trace": {"report": report("none")}})
                .to_string(),
        );
        let check = feature_proof_check(root, "demo").unwrap();
        assert_eq!(check.proven_count, 2, "both caps are proven — only the COMMIT differs");
        assert_eq!(check.commits, vec![("demo-1".to_string(), "abc123".to_string())]);
    }

    /// dda-1 parity: an archived-only capped cell still counts, same as
    /// `scribing_debt`/`judge_debt`'s own archive-inclusive read.
    #[test]
    fn an_archived_only_capped_cell_still_counts() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(
            root,
            ".bee/cells/archive/demo/demo-1.json",
            &json!({
                "id": "demo-1",
                "feature": "demo",
                "status": "capped",
                "trace": {
                    "report": { "outcome": "o", "commit": "c", "files": [], "tests": "", "deviations": [] }
                }
            })
            .to_string(),
        );
        let check = feature_proof_check(root, "demo").unwrap();
        assert!(check.blocking, "an archived cap with an empty proof must still refuse");
        assert_eq!(check.bad_ids, vec!["demo-1".to_string()]);
    }
}
