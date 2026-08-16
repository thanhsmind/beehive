// `bee state session release` (ser-3) — an explicit release for an OPEN
// session that would otherwise keep holding write-guard/concurrency locks
// for up to the 900s heartbeat-stale window. Black-box over the built
// binary: the verb's `Ctx` resolves its root from `std::env::current_dir()`
// (verbs/reservations/emit.rs `prelude`), so exercising the flag/env
// resolution honestly needs a real process per case, not an in-process
// chdir shared across parallel tests.

use assert_cmd::Command;
use serde_json::Value;
use std::path::Path;

/// A repo the binary will accept as a root — mirrors
/// tests/registry_dispatch.rs's `scratch_repo` fixture.
fn scratch_repo(base: &Path, name: &str) -> std::path::PathBuf {
    let dir = base.join(name);
    std::fs::create_dir_all(dir.join(".bee").join("logs")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    std::fs::write(dir.join(".bee/config.json"), r#"{"commands":{"test":"none"}}"#).unwrap();
    std::fs::write(dir.join(".bee/state.json"), r#"{"phase":"executing","gates":{}}"#).unwrap();
    dir
}

fn write_session(root: &Path, id: &str) {
    let dir = root.join(".bee").join("sessions");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(format!("{id}.json")),
        serde_json::to_string(&serde_json::json!({
            "id": id,
            "started_at": "2020-01-01T00:00:00.000Z",
            "last_heartbeat": "2020-01-01T00:00:00.000Z"
        }))
        .unwrap(),
    )
    .unwrap();
}

fn read_session(root: &Path, id: &str) -> Value {
    let raw = std::fs::read_to_string(root.join(".bee").join("sessions").join(format!("{id}.json")))
        .unwrap();
    serde_json::from_str(&raw).unwrap()
}

#[test]
fn release_marks_status_closed_and_released_via_the_session_id_flag() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "flag");
    write_session(&repo, "sess-flag");

    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(["state", "session", "release", "--session-id", "sess-flag", "--json"])
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["id"], "sess-flag");
    assert_eq!(v["released"], true);
    assert!(v["closed_at"].as_str().is_some());

    let record = read_session(&repo, "sess-flag");
    assert_eq!(record["status"], "closed");
    assert_eq!(record["released"], true);
    assert!(record["closed_at"].as_str().is_some());
}

#[test]
fn release_resolves_the_session_id_from_bee_session_id_env() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "env-bee");
    write_session(&repo, "sess-env-bee");

    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(["state", "session", "release", "--json"])
        .env("BEE_SESSION_ID", "sess-env-bee")
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["id"], "sess-env-bee");
    assert_eq!(v["released"], true);

    let record = read_session(&repo, "sess-env-bee");
    assert_eq!(record["status"], "closed");
    assert_eq!(record["released"], true);
}

#[test]
fn release_resolves_the_session_id_from_claude_code_session_id_env_when_bee_session_id_is_absent() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "env-claude");
    write_session(&repo, "sess-env-claude");

    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(["state", "session", "release", "--json"])
        .env_remove("BEE_SESSION_ID")
        .env("CLAUDE_CODE_SESSION_ID", "sess-env-claude")
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["id"], "sess-env-claude");
    assert_eq!(v["released"], true);
}

/// The explicit `--session-id` flag outranks both env vars — same order
/// claims.rs's `resolve_session_flag_env` documents.
#[test]
fn release_flag_outranks_the_env_chain() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "flag-wins");
    write_session(&repo, "sess-flag-wins");
    write_session(&repo, "sess-env-loses");

    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(["state", "session", "release", "--session-id", "sess-flag-wins", "--json"])
        .env("BEE_SESSION_ID", "sess-env-loses")
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["id"], "sess-flag-wins");

    let untouched = read_session(&repo, "sess-env-loses");
    assert!(untouched.get("status").is_none(), "{untouched:?}");
}

/// A session id that names no record is a typed no-op, not a refusal — the
/// same "missing record reads as done, not broken" shape `heartbeatSession`
/// gives every other session-touching seam.
#[test]
fn release_of_a_missing_session_record_is_a_typed_noop_not_a_refusal() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "missing");

    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(["state", "session", "release", "--session-id", "ghost", "--json"])
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["id"], "ghost");
    assert_eq!(v["released"], false);
    assert!(!repo.join(".bee/sessions/ghost.json").exists());
}
