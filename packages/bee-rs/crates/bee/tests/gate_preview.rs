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
//    approving any gate (no gate approval mutation).
// 4. JSON output emits the structured cell packet.

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixture(base: &Path, feature: &str) -> PathBuf {
    let dir = base.join("repo");
    std::fs::create_dir_all(dir.join(".bee").join("lanes")).unwrap();
    std::fs::create_dir_all(dir.join(".bee").join("logs")).unwrap();
    std::fs::create_dir_all(dir.join(".bee").join("runtime").join("workflows")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    std::fs::write(dir.join(".bee/config.json"), r#"{"commands":{"test":"none"}}"#).unwrap();

    // Default state: idle, no active feature
    std::fs::write(
        dir.join(".bee/state.json"),
        r#"{"schema_version":"1.0","phase":"idle","feature":null,"gates":{}}"#,
    )
    .unwrap();

    // Lane record: planning phase, unapproved gates
    std::fs::write(
        dir.join(".bee/lanes").join(format!("{feature}.json")),
        serde_json::json!({
            "schema_version": "1.0",
            "feature": feature,
            "phase": "planning",
            "mode": "high-risk",
            "gates": {
                "shape": {"approved": false},
                "execution": {"approved": false}
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

#[test]
fn cli_gate_preview_flag_with_following_lane_accepts_and_renders() {
    let tmp = tempfile::tempdir().unwrap();
    let feature = "preview-feat";
    let repo = fixture(tmp.path(), feature);

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
}

#[test]
fn cli_state_gate_preview_flag_with_following_lane_accepts_and_renders() {
    let tmp = tempfile::tempdir().unwrap();
    let feature = "preview-feat";
    let repo = fixture(tmp.path(), feature);

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
}

#[test]
fn cli_state_gate_subcommand_continues_to_work() {
    let tmp = tempfile::tempdir().unwrap();
    let feature = "preview-feat";
    let repo = fixture(tmp.path(), feature);

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
}

#[test]
fn cli_gate_preview_json_outputs_packet_and_leaves_approval_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let feature = "preview-feat";
    let repo = fixture(tmp.path(), feature);

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

    // Verify lane record on disk: gate_preview is updated, gates remain unapproved
    let lane_bytes = std::fs::read_to_string(repo.join(".bee/lanes").join(format!("{feature}.json"))).unwrap();
    let lane_json: Value = serde_json::from_str(&lane_bytes).unwrap();
    assert!(
        lane_json.get("gate_preview").is_some(),
        "gate_preview must be persisted in lane record"
    );
    assert_eq!(lane_json["gate_preview"]["feature"], "preview-feat");
    assert_eq!(
        lane_json["gates"]["shape"]["approved"], false,
        "preview must NOT approve shape gate"
    );
    assert_eq!(
        lane_json["gates"]["execution"]["approved"], false,
        "preview must NOT approve execution gate"
    );
}

#[test]
fn cli_gate_preview_reversed_order_and_default_record() {
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

    // Default record without lane: set feature on default state
    std::fs::write(
        repo.join(".bee/state.json"),
        serde_json::json!({
            "schema_version": "1.0",
            "phase": "planning",
            "feature": feature,
            "mode": "high-risk",
            "gates": {
                "shape": {"approved": false},
                "execution": {"approved": false}
            }
        })
        .to_string()
            + "\n",
    )
    .unwrap();

    let assert = Command::cargo_bin("bee")
        .unwrap()
        .args(["gate", "--preview", "--no-lane"])
        .current_dir(&repo)
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Gate preview for feature \"preview-feat\""));

    // Verify state.json: gate_preview persisted, gates unapproved
    let state_bytes = std::fs::read_to_string(repo.join(".bee/state.json")).unwrap();
    let state_json: Value = serde_json::from_str(&state_bytes).unwrap();
    assert!(state_json.get("gate_preview").is_some());
    assert_eq!(state_json["gates"]["shape"]["approved"], false);
    assert_eq!(state_json["gates"]["execution"]["approved"], false);
}
