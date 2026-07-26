//! projection_parity — cross-runtime parity + conformance suite for
//! workflow-store.mjs + state-projection.mjs's rebuild verbs (rust-port-16,
//! CONTEXT.md D1/D3/D9). Node children run the REAL mjs modules (resolved
//! by walking ancestors from CARGO_MANIFEST_DIR, via the file-based driver
//! `tests/support/projection_oracle.mjs` — never `node -e`) on one fixture
//! root while `bee_core::state_projection` runs the SAME rebuild on an
//! IDENTICAL fixture built on a second root; the resulting files
//! (state.json / lanes/<feature>.json / HANDOFF.json) are compared for
//! structural JSON equality (parsed `serde_json::Value` equality, which is
//! already order-independent — the "volatile normalization" the cell calls
//! for reduces to nothing extra here because every fixture below uses
//! fixed, caller-supplied timestamps and ids; nothing nondeterministic ever
//! lands in an output file this suite diffs).
//!
//! Lock interop reuses `tests/support/lock_driver.mjs` directly (D9
//! precedent, `lock_interop.rs`) against the SAME lock name
//! (`workflow:<id>`) `with_workflow_lock`/`withWorkflowLock` both compute —
//! proving the workflow lock is the identical cross-runtime primitive
//! without re-deriving lock semantics here.
//!
//! Every store root below comes from `tempfile::tempdir()` — never the
//! repo's live `.bee/` store (both drivers refuse a root that looks like a
//! bee checkout).

use serde_json::{json, Map, Value};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::thread;

use bee_core::lock::{with_store_lock, LockOptions, WithLockError};
use bee_core::state_projection::{
    rebuild_all_projections, rebuild_handoff_projection, rebuild_lane_projection, rebuild_state_projection, StateOverrides,
};
use bee_core::workflow_store::{update_workflow, WorkflowLockError};

fn find_mjs(rel: &[&str]) -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..12 {
        let mut candidate = dir.clone();
        for part in rel {
            candidate = candidate.join(part);
        }
        if candidate.exists() {
            return candidate;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("could not locate {} walking ancestors from {}", rel.join("/"), env!("CARGO_MANIFEST_DIR"));
}

fn state_projection_mjs() -> PathBuf {
    find_mjs(&[".bee", "bin", "lib", "state-projection.mjs"])
}

fn lock_mjs() -> PathBuf {
    find_mjs(&[".bee", "bin", "lib", "lock.mjs"])
}

fn projection_driver() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/projection_oracle.mjs")
}

fn lock_driver() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/lock_driver.mjs")
}

fn run_oracle(op: &str, root: &Path, extra: &[&str]) -> Value {
    let mut cmd = Command::new("node");
    cmd.arg(projection_driver()).arg(op).arg(state_projection_mjs()).arg(root);
    for a in extra {
        cmd.arg(a);
    }
    let output = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn node projection_oracle — is `node` on PATH?");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let line = stdout.lines().next().unwrap_or_else(|| panic!("oracle {op} produced no output — stderr: {stderr}"));
    serde_json::from_str(line).unwrap_or_else(|e| panic!("oracle {op} emitted non-JSON {line:?}: {e} — stderr: {stderr}"))
}

// ─── fixture builders ────────────────────────────────────────────────────

fn write_json(path: &Path, value: &Value) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut text = serde_json::to_string_pretty(value).unwrap();
    text.push('\n');
    fs::write(path, text).unwrap();
}

/// A workflow record fixture, written IDENTICALLY to two roots.
struct WorkflowFixture {
    id: &'static str,
    feature: &'static str,
    phase: &'static str,
    mode: Value,
    plan_rev: i64,
    gates: Value,
    summary: &'static str,
    next_action: &'static str,
    status: &'static str,
    created_at: &'static str,
    extra: Value,
}

impl Default for WorkflowFixture {
    fn default() -> Self {
        WorkflowFixture {
            id: "wf-aaaaaaaa",
            feature: "demo-feature",
            phase: "executing",
            mode: json!("standard"),
            plan_rev: 2,
            gates: json!({
                "context": {"approved": true, "approved_for_plan_rev": null},
                "shape": {"approved": true, "approved_for_plan_rev": null},
                "execution": {"approved": true, "approved_for_plan_rev": 2},
                "review": {"approved": false, "approved_for_plan_rev": null},
            }),
            summary: "demo summary",
            next_action: "demo next action",
            status: "active",
            created_at: "2026-07-20T10:00:00.000Z",
            extra: json!({}),
        }
    }
}

fn write_workflow_fixture(root: &Path, wf: &WorkflowFixture) {
    let mut record = Map::new();
    record.insert("id".into(), json!(wf.id));
    record.insert("feature".into(), json!(wf.feature));
    record.insert("phase".into(), json!(wf.phase));
    record.insert("mode".into(), wf.mode.clone());
    record.insert("plan_rev".into(), json!(wf.plan_rev));
    record.insert("gates".into(), wf.gates.clone());
    record.insert("summary".into(), json!(wf.summary));
    record.insert("next_action".into(), json!(wf.next_action));
    record.insert("status".into(), json!(wf.status));
    record.insert("created_at".into(), json!(wf.created_at));
    if let Value::Object(extra) = &wf.extra {
        for (k, v) in extra {
            record.insert(k.clone(), v.clone());
        }
    }
    let path = root.join(".bee").join("runtime").join("workflows").join(wf.id).join("state.json");
    write_json(&path, &Value::Object(record));
}

fn write_legacy_state(root: &Path, value: &Value) {
    write_json(&root.join(".bee").join("state.json"), value);
}

fn write_legacy_lane(root: &Path, feature: &str, value: &Value) {
    write_json(&root.join(".bee").join("lanes").join(format!("{feature}.json")), value);
}

fn write_mailbox_record(root: &Path, workflow_id: &str, seq: u32, value: &Value) {
    let path = root.join(".bee").join("runtime").join("handoffs").join(workflow_id).join(format!("{seq:04}.json"));
    write_json(&path, value);
}

fn read_opt(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    Some(serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: invalid JSON output ({e}): {text}", path.display())))
}

fn state_json_path(root: &Path) -> PathBuf {
    root.join(".bee").join("state.json")
}

fn lane_json_path(root: &Path, feature: &str) -> PathBuf {
    root.join(".bee").join("lanes").join(format!("{feature}.json"))
}

fn handoff_json_path(root: &Path) -> PathBuf {
    root.join(".bee").join("HANDOFF.json")
}

/// Two identically-seeded temp roots — one for the node oracle rebuild, one
/// for the Rust rebuild.
struct TwinRoots {
    node: tempfile::TempDir,
    rust: tempfile::TempDir,
}

impl TwinRoots {
    fn new() -> Self {
        TwinRoots { node: tempfile::tempdir().unwrap(), rust: tempfile::tempdir().unwrap() }
    }
}

// ─── state projection parity ────────────────────────────────────────────

#[test]
fn rebuild_state_projection_feature_matched_parity() {
    let roots = TwinRoots::new();
    let wf = WorkflowFixture::default();
    let legacy = json!({
        "schema_version": "1.0",
        "phase": "planning",
        "feature": wf.feature,
        "mode": "standard",
        "approved_gates": {"context": false, "shape": false, "execution": false, "review": false},
        "workers": [],
        "summary": "stale",
        "next_action": "stale",
    });
    for root in [roots.node.path(), roots.rust.path()] {
        write_workflow_fixture(root, &wf);
        write_legacy_state(root, &legacy);
    }

    let oracle = run_oracle("rebuild-state", roots.node.path(), &[]);
    assert_eq!(oracle["authoritative"], json!(true));
    assert_eq!(oracle["source"], json!(wf.id));

    let rust_result = rebuild_state_projection(roots.rust.path(), roots.rust.path(), &StateOverrides::default()).unwrap();
    assert!(rust_result.authoritative);
    assert_eq!(rust_result.source.as_deref(), Some(wf.id));

    let node_state = read_opt(&state_json_path(roots.node.path())).expect("node state.json");
    let rust_state = read_opt(&state_json_path(roots.rust.path())).expect("rust state.json");
    assert_eq!(node_state, rust_state, "state.json diverged between mjs and rust rebuilds");
    assert_eq!(node_state["phase"], json!(wf.phase));
    assert_eq!(node_state["approved_gates"]["execution"], json!(true));
    assert_eq!(node_state["approved_gates"]["review"], json!(false));
}

#[test]
fn rebuild_state_projection_idle_bootstrap_parity() {
    let roots = TwinRoots::new();
    let wf = WorkflowFixture { status: "active", ..WorkflowFixture::default() };
    let legacy = json!({
        "schema_version": "1.0",
        "phase": "idle",
        "feature": null,
        "mode": null,
        "approved_gates": {"context": false, "shape": false, "execution": false, "review": false},
        "workers": [],
        "summary": "",
        "next_action": "No active bee work — awaiting a user request.",
    });
    for root in [roots.node.path(), roots.rust.path()] {
        write_workflow_fixture(root, &wf);
        write_legacy_state(root, &legacy);
    }

    let oracle = run_oracle("rebuild-state", roots.node.path(), &[]);
    assert_eq!(oracle["authoritative"], json!(true));
    assert_eq!(oracle["source"], json!(wf.id));

    let rust_result = rebuild_state_projection(roots.rust.path(), roots.rust.path(), &StateOverrides::default()).unwrap();
    assert!(rust_result.authoritative);
    assert_eq!(rust_result.source.as_deref(), Some(wf.id));

    let node_state = read_opt(&state_json_path(roots.node.path())).unwrap();
    let rust_state = read_opt(&state_json_path(roots.rust.path())).unwrap();
    assert_eq!(node_state, rust_state, "idle-bootstrap state.json diverged between mjs and rust");
    assert_eq!(node_state["feature"], json!(wf.feature));
}

#[test]
fn rebuild_state_projection_no_live_workflow_is_noop_parity() {
    let roots = TwinRoots::new();
    // A workflow exists, but for a DIFFERENT feature than state.json names,
    // and state.json is not idle — neither branch fires.
    let wf = WorkflowFixture { feature: "other-feature", ..WorkflowFixture::default() };
    let legacy = json!({
        "schema_version": "1.0",
        "phase": "planning",
        "feature": "demo-feature",
        "mode": "standard",
        "approved_gates": {"context": true, "shape": false, "execution": false, "review": false},
        "workers": [],
        "summary": "untouched",
        "next_action": "untouched",
    });
    for root in [roots.node.path(), roots.rust.path()] {
        write_workflow_fixture(root, &wf);
        write_legacy_state(root, &legacy);
    }

    let oracle = run_oracle("rebuild-state", roots.node.path(), &[]);
    assert_eq!(oracle["authoritative"], json!(false));

    let rust_result = rebuild_state_projection(roots.rust.path(), roots.rust.path(), &StateOverrides::default()).unwrap();
    assert!(!rust_result.authoritative);

    let node_state = read_opt(&state_json_path(roots.node.path())).unwrap();
    let rust_state = read_opt(&state_json_path(roots.rust.path())).unwrap();
    assert_eq!(node_state, rust_state);
    assert_eq!(node_state, legacy, "a non-matching workflow must leave state.json byte-identical");
}

#[test]
fn rebuild_state_projection_no_workflow_records_is_true_noop_parity() {
    let roots = TwinRoots::new();
    let legacy = json!({
        "schema_version": "1.0",
        "phase": "idle",
        "feature": null,
        "mode": null,
        "approved_gates": {"context": false, "shape": false, "execution": false, "review": false},
        "workers": [],
        "summary": "",
        "next_action": "No active bee work — awaiting a user request.",
    });
    for root in [roots.node.path(), roots.rust.path()] {
        write_legacy_state(root, &legacy);
    }

    let oracle = run_oracle("rebuild-state", roots.node.path(), &[]);
    assert_eq!(oracle["authoritative"], json!(false));
    assert_eq!(oracle["source"], Value::Null);

    let rust_result = rebuild_state_projection(roots.rust.path(), roots.rust.path(), &StateOverrides::default()).unwrap();
    assert!(!rust_result.authoritative);
    assert_eq!(rust_result.source, None);

    let node_state = read_opt(&state_json_path(roots.node.path())).unwrap();
    let rust_state = read_opt(&state_json_path(roots.rust.path())).unwrap();
    assert_eq!(node_state, rust_state);
    assert_eq!(node_state, legacy, "zero workflow records must leave state.json byte-identical — no file rewritten");
}

#[test]
fn rebuild_state_projection_with_overrides_parity() {
    let roots = TwinRoots::new();
    let wf = WorkflowFixture::default();
    let legacy = json!({
        "schema_version": "1.0",
        "phase": "planning",
        "feature": wf.feature,
        "mode": "standard",
        "approved_gates": {"context": false, "shape": false, "execution": false, "review": false},
        "workers": [],
        "summary": "stale",
        "next_action": "stale",
        "cells": {"open": 0},
    });
    for root in [roots.node.path(), roots.rust.path()] {
        write_workflow_fixture(root, &wf);
        write_legacy_state(root, &legacy);
    }

    let overrides_json = json!({"cellCounts": {"open": 3, "capped": 7}, "lastActivity": "2026-07-26T12:00:00.000Z"}).to_string();
    let oracle = run_oracle("rebuild-state", roots.node.path(), &[&overrides_json]);
    assert_eq!(oracle["authoritative"], json!(true));

    let overrides = StateOverrides {
        cell_counts: Some(json!({"open": 3, "capped": 7})),
        last_activity: Some(json!("2026-07-26T12:00:00.000Z")),
    };
    let rust_result = rebuild_state_projection(roots.rust.path(), roots.rust.path(), &overrides).unwrap();
    assert!(rust_result.authoritative);

    let node_state = read_opt(&state_json_path(roots.node.path())).unwrap();
    let rust_state = read_opt(&state_json_path(roots.rust.path())).unwrap();
    assert_eq!(node_state, rust_state, "override-carrying rebuild diverged between mjs and rust");
    assert_eq!(node_state["cells"], json!({"open": 3, "capped": 7}));
    assert_eq!(node_state["last_activity"], json!("2026-07-26T12:00:00.000Z"));
}

// ─── lane projection parity ─────────────────────────────────────────────

#[test]
fn rebuild_lane_projection_feature_matched_parity() {
    let roots = TwinRoots::new();
    let wf = WorkflowFixture::default();
    for root in [roots.node.path(), roots.rust.path()] {
        write_workflow_fixture(root, &wf);
    }

    let oracle = run_oracle("rebuild-lane", roots.node.path(), &[wf.feature]);
    assert_eq!(oracle["authoritative"], json!(true));
    assert_eq!(oracle["source"], json!(wf.id));

    let rust_result = rebuild_lane_projection(roots.rust.path(), roots.rust.path(), wf.feature).unwrap();
    assert!(rust_result.authoritative);
    assert_eq!(rust_result.source.as_deref(), Some(wf.id));

    let node_lane = read_opt(&lane_json_path(roots.node.path(), wf.feature)).expect("node lane file");
    let rust_lane = read_opt(&lane_json_path(roots.rust.path(), wf.feature)).expect("rust lane file");
    assert_eq!(node_lane, rust_lane, "lane file diverged between mjs and rust rebuilds");
    assert_eq!(node_lane["phase"], json!(wf.phase));
    assert_eq!(node_lane["created_at"], json!(wf.created_at));
}

#[test]
fn rebuild_lane_projection_ad_hoc_field_passthrough_parity() {
    let roots = TwinRoots::new();
    let wf = WorkflowFixture::default();
    let existing_lane = json!({
        "schema_version": "1.0",
        "feature": wf.feature,
        "mode": "old-mode",
        "phase": "old-phase",
        "approved_gates": {"context": false, "shape": false, "execution": false, "review": false},
        "summary": "old summary",
        "next_action": "old next action",
        "created_at": "2026-07-19T09:00:00.000Z",
        "last_scribing_run": "2026-07-20T08:00:00.000Z",
        "gate_revoked_at": null,
        "advisor_ref": "fable-slice-2",
    });
    for root in [roots.node.path(), roots.rust.path()] {
        write_workflow_fixture(root, &wf);
        write_legacy_lane(root, wf.feature, &existing_lane);
    }

    let oracle = run_oracle("rebuild-lane", roots.node.path(), &[wf.feature]);
    assert_eq!(oracle["authoritative"], json!(true));

    let rust_result = rebuild_lane_projection(roots.rust.path(), roots.rust.path(), wf.feature).unwrap();
    assert!(rust_result.authoritative);

    let node_lane = read_opt(&lane_json_path(roots.node.path(), wf.feature)).unwrap();
    let rust_lane = read_opt(&lane_json_path(roots.rust.path(), wf.feature)).unwrap();
    assert_eq!(node_lane, rust_lane, "ad-hoc-field lane rebuild diverged between mjs and rust");
    // The ad hoc fields and the ORIGINAL created_at must survive untouched.
    assert_eq!(node_lane["created_at"], json!("2026-07-19T09:00:00.000Z"));
    assert_eq!(node_lane["last_scribing_run"], json!("2026-07-20T08:00:00.000Z"));
    assert_eq!(node_lane["advisor_ref"], json!("fable-slice-2"));
    // The six baseline D1 fields must be fully recomputed from the record.
    assert_eq!(node_lane["phase"], json!(wf.phase));
    assert_eq!(node_lane["summary"], json!(wf.summary));
}

#[test]
fn rebuild_lane_projection_no_workflow_records_is_noop_parity() {
    let roots = TwinRoots::new();
    let existing_lane = json!({
        "schema_version": "1.0",
        "feature": "demo-feature",
        "mode": "standard",
        "phase": "planning",
        "approved_gates": {"context": false, "shape": false, "execution": false, "review": false},
        "summary": "untouched",
        "next_action": "untouched",
        "created_at": "2026-07-19T09:00:00.000Z",
    });
    for root in [roots.node.path(), roots.rust.path()] {
        write_legacy_lane(root, "demo-feature", &existing_lane);
    }

    let oracle = run_oracle("rebuild-lane", roots.node.path(), &["demo-feature"]);
    assert_eq!(oracle["authoritative"], json!(false));

    let rust_result = rebuild_lane_projection(roots.rust.path(), roots.rust.path(), "demo-feature").unwrap();
    assert!(!rust_result.authoritative);

    let node_lane = read_opt(&lane_json_path(roots.node.path(), "demo-feature")).unwrap();
    let rust_lane = read_opt(&lane_json_path(roots.rust.path(), "demo-feature")).unwrap();
    assert_eq!(node_lane, rust_lane);
    assert_eq!(node_lane, existing_lane, "zero workflow records must leave the lane file byte-identical");
}

#[test]
fn rebuild_lane_projection_no_live_workflow_for_feature_is_noop_parity() {
    let roots = TwinRoots::new();
    let wf = WorkflowFixture { status: "closed", ..WorkflowFixture::default() };
    let existing_lane = json!({
        "schema_version": "1.0",
        "feature": wf.feature,
        "mode": "standard",
        "phase": "planning",
        "approved_gates": {"context": false, "shape": false, "execution": false, "review": false},
        "summary": "untouched",
        "next_action": "untouched",
        "created_at": "2026-07-19T09:00:00.000Z",
    });
    for root in [roots.node.path(), roots.rust.path()] {
        write_workflow_fixture(root, &wf);
        write_legacy_lane(root, wf.feature, &existing_lane);
    }

    let oracle = run_oracle("rebuild-lane", roots.node.path(), &[wf.feature]);
    assert_eq!(oracle["authoritative"], json!(false), "a CLOSED workflow must never drive a lane rebuild");

    let rust_result = rebuild_lane_projection(roots.rust.path(), roots.rust.path(), wf.feature).unwrap();
    assert!(!rust_result.authoritative);

    let node_lane = read_opt(&lane_json_path(roots.node.path(), wf.feature)).unwrap();
    let rust_lane = read_opt(&lane_json_path(roots.rust.path(), wf.feature)).unwrap();
    assert_eq!(node_lane, rust_lane);
    assert_eq!(node_lane, existing_lane);
}

// ─── handoff projection parity ──────────────────────────────────────────

#[test]
fn rebuild_handoff_projection_single_open_record_parity() {
    let roots = TwinRoots::new();
    let wf = WorkflowFixture::default();
    let record = json!({
        "kind": "pause",
        "status": "open",
        "written_at": "2026-07-25T10:00:00.000Z",
        "cell": "rust-port-16",
        "writer_session": "sess-a",
        "workflow_id": wf.id,
        "target_role": null,
        "from_session": "sess-a",
    });
    for root in [roots.node.path(), roots.rust.path()] {
        write_workflow_fixture(root, &wf);
        write_mailbox_record(root, wf.id, 1, &record);
    }

    let oracle = run_oracle("rebuild-handoff", roots.node.path(), &[]);
    assert_eq!(oracle["authoritative"], json!(true));
    assert_eq!(oracle["source"], json!(wf.id));

    let rust_result = rebuild_handoff_projection(roots.rust.path(), roots.rust.path()).unwrap();
    assert!(rust_result.authoritative);
    assert_eq!(rust_result.source.as_deref(), Some(wf.id));

    let node_handoff = read_opt(&handoff_json_path(roots.node.path())).expect("node HANDOFF.json");
    let rust_handoff = read_opt(&handoff_json_path(roots.rust.path())).expect("rust HANDOFF.json");
    assert_eq!(node_handoff, rust_handoff, "HANDOFF.json diverged between mjs and rust rebuilds");
    // Mailbox-only fields are dropped from the legacy projection.
    assert!(node_handoff.get("status").is_none());
    assert!(node_handoff.get("workflow_id").is_none());
    assert_eq!(node_handoff["kind"], json!("pause"));
    assert_eq!(node_handoff["cell"], json!("rust-port-16"));
}

#[test]
fn rebuild_handoff_projection_ties_broken_by_written_at_then_workflow_id_parity() {
    let roots = TwinRoots::new();
    let wf_a = WorkflowFixture { id: "wf-aaaaaaaa", feature: "feature-a", ..WorkflowFixture::default() };
    let wf_b = WorkflowFixture { id: "wf-bbbbbbbb", feature: "feature-b", ..WorkflowFixture::default() };
    let older = json!({
        "kind": "pause", "status": "open", "written_at": "2026-07-25T09:00:00.000Z",
        "workflow_id": wf_a.id, "target_role": null, "from_session": "sess-a", "cell": "older",
    });
    let newer = json!({
        "kind": "planned-next", "status": "open", "written_at": "2026-07-25T11:00:00.000Z",
        "workflow_id": wf_b.id, "target_role": null, "from_session": "sess-b", "cell": "newer",
        "writer_session": "sess-b", "previous_cell": "rust-port-15", "next_cell": "rust-port-16",
    });
    for root in [roots.node.path(), roots.rust.path()] {
        write_workflow_fixture(root, &wf_a);
        write_workflow_fixture(root, &wf_b);
        write_mailbox_record(root, wf_a.id, 1, &older);
        write_mailbox_record(root, wf_b.id, 1, &newer);
    }

    let oracle = run_oracle("rebuild-handoff", roots.node.path(), &[]);
    assert_eq!(oracle["source"], json!(wf_b.id), "the NEWER written_at record must win");

    let rust_result = rebuild_handoff_projection(roots.rust.path(), roots.rust.path()).unwrap();
    assert_eq!(rust_result.source.as_deref(), Some(wf_b.id));

    let node_handoff = read_opt(&handoff_json_path(roots.node.path())).unwrap();
    let rust_handoff = read_opt(&handoff_json_path(roots.rust.path())).unwrap();
    assert_eq!(node_handoff, rust_handoff);
    assert_eq!(node_handoff["kind"], json!("planned-next"));
    assert_eq!(node_handoff["cell"], json!("newer"));
}

#[test]
fn rebuild_handoff_projection_no_open_records_removes_legacy_file_parity() {
    let roots = TwinRoots::new();
    let wf = WorkflowFixture::default();
    let cleared = json!({
        "kind": "pause", "status": "cleared", "written_at": "2026-07-25T10:00:00.000Z",
        "workflow_id": wf.id, "target_role": null, "from_session": "sess-a",
    });
    for root in [roots.node.path(), roots.rust.path()] {
        write_workflow_fixture(root, &wf);
        write_mailbox_record(root, wf.id, 1, &cleared);
        // A stale legacy file must be REMOVED, not left behind.
        write_json(&handoff_json_path(root), &json!({"kind": "pause", "cell": "stale"}));
    }

    let oracle = run_oracle("rebuild-handoff", roots.node.path(), &[]);
    assert_eq!(oracle["authoritative"], json!(true));
    assert_eq!(oracle["source"], Value::Null);

    let rust_result = rebuild_handoff_projection(roots.rust.path(), roots.rust.path()).unwrap();
    assert!(rust_result.authoritative);
    assert_eq!(rust_result.source, None);

    assert!(!handoff_json_path(roots.node.path()).exists(), "mjs must remove the stale legacy file");
    assert!(!handoff_json_path(roots.rust.path()).exists(), "rust must remove the stale legacy file");
}

#[test]
fn rebuild_handoff_projection_no_workflow_records_is_true_noop_parity() {
    let roots = TwinRoots::new();
    for root in [roots.node.path(), roots.rust.path()] {
        write_json(&handoff_json_path(root), &json!({"kind": "pause", "cell": "untouched"}));
    }

    let oracle = run_oracle("rebuild-handoff", roots.node.path(), &[]);
    assert_eq!(oracle["authoritative"], json!(false));

    let rust_result = rebuild_handoff_projection(roots.rust.path(), roots.rust.path()).unwrap();
    assert!(!rust_result.authoritative);

    let node_handoff = read_opt(&handoff_json_path(roots.node.path())).unwrap();
    let rust_handoff = read_opt(&handoff_json_path(roots.rust.path())).unwrap();
    assert_eq!(node_handoff, rust_handoff);
    assert_eq!(node_handoff, json!({"kind": "pause", "cell": "untouched"}), "zero workflow records must leave HANDOFF.json byte-identical");
}

// ─── composite rebuild-all parity ───────────────────────────────────────

#[test]
fn rebuild_all_projections_composite_parity() {
    let roots = TwinRoots::new();
    let wf = WorkflowFixture::default();
    let record = json!({
        "kind": "pause", "status": "open", "written_at": "2026-07-25T10:00:00.000Z",
        "workflow_id": wf.id, "target_role": null, "from_session": "sess-a", "cell": "rust-port-16",
    });
    for root in [roots.node.path(), roots.rust.path()] {
        write_workflow_fixture(root, &wf);
        write_mailbox_record(root, wf.id, 1, &record);
    }

    let oracle = run_oracle("rebuild-all", roots.node.path(), &[]);
    assert_eq!(oracle["state"]["authoritative"], json!(true));
    assert_eq!(oracle["handoff"]["authoritative"], json!(true));
    assert_eq!(oracle["lanes"].as_array().unwrap().len(), 1);

    let rust_result = rebuild_all_projections(roots.rust.path(), roots.rust.path()).unwrap();
    assert!(rust_result.state.authoritative);
    assert!(rust_result.handoff.authoritative);
    assert_eq!(rust_result.lanes.len(), 1);

    let node_state = read_opt(&state_json_path(roots.node.path())).unwrap();
    let rust_state = read_opt(&state_json_path(roots.rust.path())).unwrap();
    assert_eq!(node_state, rust_state);

    let node_handoff = read_opt(&handoff_json_path(roots.node.path())).unwrap();
    let rust_handoff = read_opt(&handoff_json_path(roots.rust.path())).unwrap();
    assert_eq!(node_handoff, rust_handoff);

    let node_lane = read_opt(&lane_json_path(roots.node.path(), wf.feature)).unwrap();
    let rust_lane = read_opt(&lane_json_path(roots.rust.path(), wf.feature)).unwrap();
    assert_eq!(node_lane, rust_lane);
}

// ─── unknown-field round-trip (workflow-store CRUD) ─────────────────────

#[test]
fn workflow_record_unknown_fields_survive_round_trip() {
    let root = tempfile::tempdir().unwrap();
    let mut record = Map::new();
    record.insert("id".into(), json!("wf-roundtrip"));
    record.insert("feature".into(), json!("demo-feature"));
    record.insert("phase".into(), json!("executing"));
    record.insert("mode".into(), json!("standard"));
    record.insert("plan_rev".into(), json!(1));
    record.insert(
        "gates".into(),
        json!({
            "context": {"approved": true, "approved_for_plan_rev": null},
            "shape": {"approved": true, "approved_for_plan_rev": null},
            "execution": {"approved": false, "approved_for_plan_rev": null},
            "review": {"approved": false, "approved_for_plan_rev": null},
            // An unknown gate name from a hypothetical future cell.
            "acceptance": {"approved": true, "approved_for_plan_rev": null},
        }),
    );
    record.insert("summary".into(), json!("s"));
    record.insert("next_action".into(), json!("n"));
    record.insert("status".into(), json!("active"));
    record.insert("created_at".into(), json!("2026-07-20T10:00:00.000Z"));
    // Unknown top-level fields a future cell might add.
    record.insert("advisor_ref".into(), json!("fable-x"));
    record.insert("future_blob".into(), json!({"nested": [1, 2, 3]}));

    let path = root.path().join(".bee").join("runtime").join("workflows").join("wf-roundtrip").join("state.json");
    write_json(&path, &Value::Object(record.clone()));

    let read = bee_core::workflow_store::read_workflow(root.path(), "wf-roundtrip").expect("read fixture record");
    assert_eq!(read.extra.get("advisor_ref"), Some(&json!("fable-x")));
    assert_eq!(read.extra.get("future_blob"), Some(&json!({"nested": [1, 2, 3]})));
    assert_eq!(read.gates.get("acceptance"), Some(&json!({"approved": true, "approved_for_plan_rev": null})));

    // updateWorkflowAssumingLock must NOT drop the unknown fields on a
    // write-back that only touches an unrelated field.
    let updated = bee_core::workflow_store::update_workflow_assuming_lock(root.path(), "wf-roundtrip", |_| json!({"summary": "s2"}))
        .expect("update fixture record");
    assert_eq!(updated.summary, "s2");
    assert_eq!(updated.extra.get("advisor_ref"), Some(&json!("fable-x")));
    assert_eq!(updated.extra.get("future_blob"), Some(&json!({"nested": [1, 2, 3]})));
    assert_eq!(updated.gates.get("acceptance"), Some(&json!({"approved": true, "approved_for_plan_rev": null})));

    let on_disk = read_opt(&path).unwrap();
    assert_eq!(on_disk["advisor_ref"], json!("fable-x"));
    assert_eq!(on_disk["future_blob"], json!({"nested": [1, 2, 3]}));
    assert_eq!(on_disk["gates"]["acceptance"], json!({"approved": true, "approved_for_plan_rev": null}));
}

// ─── cross-runtime workflow-lock interop (D9) ───────────────────────────
// Reuses tests/support/lock_driver.mjs directly (the SAME driver
// lock_interop.rs proves against raw lock names) with name =
// `workflow:<id>` — the exact lock name both `withWorkflowLock` (mjs) and
// `with_workflow_lock` (rust) compute, so this proves the two runtimes
// contend the identical cross-process primitive without re-deriving lock
// semantics in this suite.

struct HoldChild {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

fn spawn_node_hold(root: &Path, lock_name: &str) -> HoldChild {
    let mut child = Command::new("node")
        .arg(lock_driver())
        .arg("hold")
        .arg(lock_mjs())
        .arg(root)
        .arg(lock_name)
        .env("BEE_SESSION_ID", "node-workflow-holder")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to spawn node lock_driver hold");
    let stdin = child.stdin.take().expect("hold child stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("hold child stdout"));
    let mut line = String::new();
    stdout.read_line(&mut line).expect("read hold child first line");
    let v: Value = serde_json::from_str(line.trim()).unwrap_or_else(|e| panic!("hold child emitted non-JSON {line:?}: {e}"));
    assert_eq!(v["acquired"], json!(true), "node hold child failed to acquire workflow lock: {v}");
    HoldChild { child, stdin, stdout }
}

impl HoldChild {
    fn release(mut self) {
        self.stdin.write_all(b"release\n").expect("write release line");
        self.stdin.flush().expect("flush release line");
        let mut line = String::new();
        self.stdout.read_line(&mut line).expect("read release ack");
        let v: Value = serde_json::from_str(line.trim()).unwrap_or_else(|e| panic!("hold child release ack non-JSON {line:?}: {e}"));
        assert_eq!(v["released"], json!(true), "node hold child did not confirm release: {v}");
        drop(self.stdin);
        self.child.wait().expect("wait hold child");
    }
}

#[test]
fn node_holds_workflow_lock_rust_update_denied_then_succeeds() {
    let root = tempfile::tempdir().unwrap();
    let wf = WorkflowFixture::default();
    write_workflow_fixture(root.path(), &wf);
    let lock_name = format!("workflow:{}", wf.id);

    let hold = spawn_node_hold(root.path(), &lock_name);

    let denied = update_workflow(root.path(), wf.id, |_| json!({"summary": "denied-attempt"}), LockOptions::try_once());
    match denied {
        Err(WorkflowLockError::Lock(WithLockError::Busy { name, .. })) => assert_eq!(name, lock_name),
        other => panic!("expected Busy while node holds the workflow lock, got {other:?}"),
    }

    hold.release();

    let allowed = update_workflow(root.path(), wf.id, |_| json!({"summary": "after-release"}), LockOptions::try_once())
        .expect("rust update must succeed once node releases the workflow lock");
    assert_eq!(allowed.summary, "after-release");
}

#[test]
fn rust_holds_workflow_lock_node_contender_denied_then_succeeds() {
    let root = tempfile::tempdir().unwrap();
    let wf = WorkflowFixture::default();
    write_workflow_fixture(root.path(), &wf);
    let lock_name = format!("workflow:{}", wf.id);

    let (acquired_tx, acquired_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let (done_tx, done_rx) = mpsc::channel::<()>();
    let root_for_thread = root.path().to_path_buf();
    let name_for_thread = lock_name.clone();
    let holder = thread::spawn(move || {
        let result = with_store_lock(&root_for_thread, &name_for_thread, LockOptions::default(), || {
            acquired_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        });
        result.expect("rust holder must acquire the workflow lock");
        done_tx.send(()).unwrap();
    });
    acquired_rx.recv().expect("rust holder never acquired the lock");

    // node's raw contender: acquire-once must observe the lock busy.
    let denied = Command::new("node")
        .arg(lock_driver())
        .arg("acquire-once")
        .arg(lock_mjs())
        .arg(root.path())
        .arg(&lock_name)
        .env("BEE_SESSION_ID", "node-contender")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn node lock_driver acquire-once");
    let stdout = String::from_utf8_lossy(&denied.stdout);
    let v: Value = serde_json::from_str(stdout.lines().next().expect("acquire-once produced no output")).expect("acquire-once emitted non-JSON");
    assert_eq!(v["acquired"], json!(false), "node contender must be denied while rust holds the workflow lock: {v}");

    release_tx.send(()).unwrap();
    done_rx.recv().expect("rust holder never finished");
    holder.join().expect("rust holder thread panicked");

    // After release, node's own updateWorkflow-shaped acquire must succeed.
    let allowed = Command::new("node")
        .arg(lock_driver())
        .arg("acquire-once")
        .arg(lock_mjs())
        .arg(root.path())
        .arg(&lock_name)
        .env("BEE_SESSION_ID", "node-contender-2")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn node lock_driver acquire-once (2)");
    let stdout = String::from_utf8_lossy(&allowed.stdout);
    let v: Value = serde_json::from_str(stdout.lines().next().expect("acquire-once (2) produced no output")).expect("acquire-once (2) emitted non-JSON");
    assert_eq!(v["acquired"], json!(true), "node must acquire the workflow lock once rust releases it: {v}");
}
