// Integration tests for `gate --preview` and `state gate --preview`.
//
// Documented help spells `--preview` as a boolean flag on `bee gate` and
// `bee state gate`. Because `FLAG_ALONE_BOOLEANS` historically omitted
// `preview`, passing `--preview --lane <feature>` caused the parser to consume
// `--lane` as the value of `--preview`, rejecting the invocation with an
// unsupported argument shape error.
//
// These tests verify that:
// 1. Both documented spellings (`gate --preview` and `state gate --preview`)
//    accept following `--lane` flags and return the cell packet.
// 2. The existing subcommand spelling (`state gate preview --lane ...`)
//    continues to work identically.
// 3. Preview returns the current packet and updates `gate_preview` without
//    mutating real approval gates (`approved_gates.shape` and `approved_gates.execution`).
// 4. Preview preserves existing approvals (both false and already-true) on both
//    lane records and default state records.
// 5. JSON output emits the structured cell packet.

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixture(base: &Path, feature: &str) -> PathBuf {
    fixture_with_gates(base, feature, false, false, false, false)
}

fn fixture_with_gates(
    base: &Path,
    feature: &str,
    lane_shape: bool,
    lane_exec: bool,
    state_shape: bool,
    state_exec: bool,
) -> PathBuf {
    let dir = base.join("repo");
    std::fs::create_dir_all(dir.join(".bee").join("lanes")).unwrap();
    std::fs::create_dir_all(dir.join(".bee").join("logs")).unwrap();
    std::fs::create_dir_all(dir.join(".bee").join("runtime").join("workflows")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    std::fs::write(dir.join(".bee/config.json"), r#"{"commands":{"test":"none"}}"#).unwrap();

    // Default state
    std::fs::write(
        dir.join(".bee/state.json"),
        serde_json::json!({
            "schema_version": "1.0",
            "phase": "idle",
            "feature": null,
            "approved_gates": {
                "context": false,
                "shape": state_shape,
                "execution": state_exec,
                "review": false,
                "uat": false
            }
        })
        .to_string()
            + "\n",
    )
    .unwrap();

    // Lane record: planning phase
    std::fs::write(
        dir.join(".bee/lanes").join(format!("{feature}.json")),
        serde_json::json!({
            "schema_version": "1.0",
            "feature": feature,
            "phase": "planning",
            "mode": "high-risk",
            "approved_gates": {
                "context": true,
                "shape": lane_shape,
                "execution": lane_exec,
                "review": false,
                "uat": false
            }
        })
        .to_string()
            + "\n",
    )
    .unwrap();

    // plan.md with cells section
    let plan_dir = dir.join("docs").join("history").join(feature);
    std::fs::create_dir_all(&plan_dir).unwrap();
    let plan_content = format!(
        "# Plan: {feature}\n\n## Cells\n\n```json\n[\n  {{\n    \"id\": \"c-1\",\n    \"title\": \"Implement parser fix\",\n    \"action\": \"Add preview to alone booleans\",\n    \"verify\": \"cargo test -p bee gate_preview\",\n    \"files\": [\"src/flags.rs\"],\n    \"read_first\": [],\n    \"lane\": \"high-risk\",\n    \"role\": \"code\",\n    \"must_haves\": {{\n      \"truths\": [\"Preview accepts following lane\"]\n    }}\n  }}\n]\n```\n"
    );
    std::fs::write(plan_dir.join("plan.md"), plan_content).unwrap();

    dir
}

fn read_lane_approved_gates(repo: &Path, feature: &str) -> (bool, bool) {
    let lane_bytes =
        std::fs::read_to_string(repo.join(".bee/lanes").join(format!("{feature}.json"))).unwrap();
    let lane_json: Value = serde_json::from_str(&lane_bytes).unwrap();
    (
        lane_json["approved_gates"]["shape"]
            .as_bool()
            .expect("shape must be boolean"),
        lane_json["approved_gates"]["execution"]
            .as_bool()
            .expect("execution must be boolean"),
    )
}

fn read_state_approved_gates(repo: &Path) -> (bool, bool) {
    let state_bytes = std::fs::read_to_string(repo.join(".bee/state.json")).unwrap();
    let state_json: Value = serde_json::from_str(&state_bytes).unwrap();
    (
        state_json["approved_gates"]["shape"]
            .as_bool()
            .expect("shape must be boolean"),
        state_json["approved_gates"]["execution"]
            .as_bool()
            .expect("execution must be boolean"),
    )
}

fn read_status_gates(repo: &Path) -> (bool, bool) {
    let assert = Command::cargo_bin("bee")
        .unwrap()
        .args(["status", "--brief", "--json"])
        .current_dir(repo)
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let parsed: Value = serde_json::from_str(stdout.trim()).unwrap();
    (
        parsed["gates"]["shape"].as_bool().unwrap(),
        parsed["gates"]["execution"].as_bool().unwrap(),
    )
}

#[test]
fn cli_gate_preview_flag_with_following_lane_accepts_and_renders() {
    let tmp = tempfile::tempdir().unwrap();
    let feature = "preview-feat";
    let repo = fixture(tmp.path(), feature);

    let (before_shape, before_exec) = read_lane_approved_gates(&repo, feature);
    assert!(!before_shape);
    assert!(!before_exec);

    let assert = Command::cargo_bin("bee")
        .unwrap()
        .args(["gate", "--preview", "--lane", feature])
        .current_dir(&repo)
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("Gate preview for feature \"preview-feat\""),
        "stdout should announce gate preview: {stdout}"
    );
    assert!(
        stdout.contains("Cell: c-1 — Implement parser fix"),
        "stdout should show cell details: {stdout}"
    );
    assert!(
        stdout.contains("Preview accepts following lane"),
        "stdout should show must_haves: {stdout}"
    );

    let (after_shape, after_exec) = read_lane_approved_gates(&repo, feature);
    assert_eq!(after_shape, false);
    assert_eq!(after_exec, false);
}

#[test]
fn cli_state_gate_preview_flag_with_following_lane_accepts_and_renders() {
    let tmp = tempfile::tempdir().unwrap();
    let feature = "preview-feat";
    let repo = fixture(tmp.path(), feature);

    let (before_shape, before_exec) = read_lane_approved_gates(&repo, feature);
    assert!(!before_shape);
    assert!(!before_exec);

    let assert = Command::cargo_bin("bee")
        .unwrap()
        .args(["state", "gate", "--preview", "--lane", feature])
        .current_dir(&repo)
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("Gate preview for feature \"preview-feat\""),
        "stdout should announce gate preview: {stdout}"
    );
    assert!(
        stdout.contains("Cell: c-1 — Implement parser fix"),
        "stdout should show cell details: {stdout}"
    );

    let (after_shape, after_exec) = read_lane_approved_gates(&repo, feature);
    assert_eq!(after_shape, false);
    assert_eq!(after_exec, false);
}

#[test]
fn cli_state_gate_subcommand_continues_to_work() {
    let tmp = tempfile::tempdir().unwrap();
    let feature = "preview-feat";
    let repo = fixture(tmp.path(), feature);

    let (before_shape, before_exec) = read_lane_approved_gates(&repo, feature);
    assert!(!before_shape);
    assert!(!before_exec);

    let assert = Command::cargo_bin("bee")
        .unwrap()
        .args(["state", "gate", "preview", "--lane", feature])
        .current_dir(&repo)
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("Gate preview for feature \"preview-feat\""),
        "subcommand spelling must still work: {stdout}"
    );
    assert!(
        stdout.contains("Cell: c-1 — Implement parser fix"),
        "subcommand spelling shows cell details: {stdout}"
    );

    let (after_shape, after_exec) = read_lane_approved_gates(&repo, feature);
    assert_eq!(after_shape, false);
    assert_eq!(after_exec, false);
}

#[test]
fn cli_gate_preview_json_outputs_packet_and_leaves_approval_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let feature = "preview-feat";
    let repo = fixture(tmp.path(), feature);

    let (before_shape, before_exec) = read_lane_approved_gates(&repo, feature);
    assert!(!before_shape);
    assert!(!before_exec);

    let assert = Command::cargo_bin("bee")
        .unwrap()
        .args(["gate", "--preview", "--lane", feature, "--json"])
        .current_dir(&repo)
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let parsed: Value = serde_json::from_str(stdout.trim()).expect("must be valid JSON output");
    assert_eq!(parsed["feature"], "preview-feat");
    assert!(parsed["plan_sha256"].is_string());
    assert!(parsed["previewed_at"].is_string());
    let cells = parsed["cells"].as_array().expect("cells must be an array");
    assert_eq!(cells.len(), 1);
    assert_eq!(cells[0]["id"], "c-1");

    // Verify lane record on disk: gate_preview is updated, approved_gates remain unapproved
    let lane_bytes =
        std::fs::read_to_string(repo.join(".bee/lanes").join(format!("{feature}.json"))).unwrap();
    let lane_json: Value = serde_json::from_str(&lane_bytes).unwrap();
    assert!(
        lane_json.get("gate_preview").is_some(),
        "gate_preview must be persisted in lane record"
    );
    assert_eq!(lane_json["gate_preview"]["feature"], "preview-feat");

    let (after_shape, after_exec) = read_lane_approved_gates(&repo, feature);
    assert_eq!(after_shape, false, "preview must NOT approve shape gate");
    assert_eq!(after_exec, false, "preview must NOT approve execution gate");
    assert_eq!(after_shape, before_shape);
    assert_eq!(after_exec, before_exec);
}

#[test]
fn cli_lane_preview_preserves_existing_true_approvals() {
    let tmp = tempfile::tempdir().unwrap();
    let feature = "preview-feat";
    let repo = fixture_with_gates(tmp.path(), feature, true, true, false, false);

    let (before_shape, before_exec) = read_lane_approved_gates(&repo, feature);
    assert!(before_shape);
    assert!(before_exec);

    Command::cargo_bin("bee")
        .unwrap()
        .args(["gate", "--preview", "--lane", feature])
        .current_dir(&repo)
        .assert()
        .success();

    let (after_shape, after_exec) = read_lane_approved_gates(&repo, feature);
    assert_eq!(
        after_shape, true,
        "preview must preserve already-true shape gate"
    );
    assert_eq!(
        after_exec, true,
        "preview must preserve already-true execution gate"
    );

    let lane_bytes =
        std::fs::read_to_string(repo.join(".bee/lanes").join(format!("{feature}.json"))).unwrap();
    let lane_json: Value = serde_json::from_str(&lane_bytes).unwrap();
    assert!(
        lane_json.get("gate_preview").is_some(),
        "gate_preview must be persisted in lane record"
    );
}

#[test]
fn cli_gate_preview_reversed_order_and_default_record_unapproved() {
    let tmp = tempfile::tempdir().unwrap();
    let feature = "preview-feat";
    let repo = fixture(tmp.path(), feature);

    // Reversed order: --lane before --preview
    Command::cargo_bin("bee")
        .unwrap()
        .args(["gate", "--lane", feature, "--preview"])
        .current_dir(&repo)
        .assert()
        .success();

    let (lane_shape, lane_exec) = read_lane_approved_gates(&repo, feature);
    assert!(!lane_shape);
    assert!(!lane_exec);

    // Default record without lane: set feature on default state
    std::fs::write(
        repo.join(".bee/state.json"),
        serde_json::json!({
            "schema_version": "1.0",
            "phase": "planning",
            "feature": feature,
            "mode": "high-risk",
            "approved_gates": {
                "context": true,
                "shape": false,
                "execution": false,
                "review": false,
                "uat": false
            }
        })
        .to_string()
            + "\n",
    )
    .unwrap();

    let (state_before_shape, state_before_exec) = read_state_approved_gates(&repo);
    let (status_before_shape, status_before_exec) = read_status_gates(&repo);
    assert!(!state_before_shape);
    assert!(!state_before_exec);
    assert!(!status_before_shape);
    assert!(!status_before_exec);

    let assert = Command::cargo_bin("bee")
        .unwrap()
        .args(["gate", "--preview", "--no-lane"])
        .current_dir(&repo)
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Gate preview for feature \"preview-feat\""));

    // Verify state.json: gate_preview persisted, approved_gates unchanged
    let state_bytes = std::fs::read_to_string(repo.join(".bee/state.json")).unwrap();
    let state_json: Value = serde_json::from_str(&state_bytes).unwrap();
    assert!(state_json.get("gate_preview").is_some());
    let (state_after_shape, state_after_exec) = read_state_approved_gates(&repo);
    let (status_after_shape, status_after_exec) = read_status_gates(&repo);
    assert_eq!(state_after_shape, false);
    assert_eq!(state_after_exec, false);
    assert_eq!(status_after_shape, false);
    assert_eq!(status_after_exec, false);
}

#[test]
fn cli_default_record_preview_preserves_existing_true_approvals() {
    let tmp = tempfile::tempdir().unwrap();
    let feature = "preview-feat";
    let repo = fixture(tmp.path(), feature);

    // Default record with already-true approvals
    std::fs::write(
        repo.join(".bee/state.json"),
        serde_json::json!({
            "schema_version": "1.0",
            "phase": "planning",
            "feature": feature,
            "mode": "high-risk",
            "approved_gates": {
                "context": true,
                "shape": true,
                "execution": true,
                "review": false,
                "uat": false
            }
        })
        .to_string()
            + "\n",
    )
    .unwrap();

    let (state_before_shape, state_before_exec) = read_state_approved_gates(&repo);
    let (status_before_shape, status_before_exec) = read_status_gates(&repo);
    assert!(state_before_shape);
    assert!(state_before_exec);
    assert!(status_before_shape);
    assert!(status_before_exec);

    Command::cargo_bin("bee")
        .unwrap()
        .args(["state", "gate", "--preview", "--no-lane"])
        .current_dir(&repo)
        .assert()
        .success();

    // Verify state.json: gate_preview persisted, approved_gates preserved
    let state_bytes = std::fs::read_to_string(repo.join(".bee/state.json")).unwrap();
    let state_json: Value = serde_json::from_str(&state_bytes).unwrap();
    assert!(state_json.get("gate_preview").is_some());
    let (state_after_shape, state_after_exec) = read_state_approved_gates(&repo);
    let (status_after_shape, status_after_exec) = read_status_gates(&repo);
    assert_eq!(
        state_after_shape, true,
        "state.json approved_gates.shape must stay true"
    );
    assert_eq!(
        state_after_exec, true,
        "state.json approved_gates.execution must stay true"
    );
    assert_eq!(
        status_after_shape, true,
        "status reader gates.shape must stay true"
    );
    assert_eq!(
        status_after_exec, true,
        "status reader gates.execution must stay true"
    );
}
