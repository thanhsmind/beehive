// `bee work show|set` (pwr-2) — the agent's half of the prompt work record.
// Black-box over the built binary for the same reason session_release.rs is:
// the verb resolves its root from `std::env::current_dir()` and its sink from
// process environment (CLAUDE_CODE_SESSION_ID, the herding marker), so an
// honest test of that resolution needs a real process per case.

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn scratch_repo(base: &Path, name: &str) -> PathBuf {
    let dir = base.join(name);
    std::fs::create_dir_all(dir.join(".bee").join("logs")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    std::fs::write(dir.join(".bee/config.json"), r#"{"commands":{"test":"none"}}"#).unwrap();
    std::fs::write(dir.join(".bee/state.json"), r#"{"phase":"executing","gates":{}}"#).unwrap();
    dir
}

/// Open a work record the way the real world does: fire the activity hook with
/// a prompt. Nothing else in bee opens one.
fn prompt(repo: &Path, session: &str, text: &str) {
    let payload = serde_json::json!({
        "hook_event_name": "UserPromptSubmit",
        "session_id": session,
        "cwd": repo.to_string_lossy(),
        "prompt": text,
    });
    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(["hook", "activity"])
        .current_dir(repo)
        .write_stdin(payload.to_string())
        .output()
        .unwrap();
    assert!(out.status.success(), "the hook must always exit 0");
}

fn work(repo: &Path, args: &[&str]) -> (bool, Value, String) {
    let mut full: Vec<&str> = vec!["work"];
    full.extend_from_slice(args);
    full.push("--json");
    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(&full)
        .current_dir(repo)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let parsed: Value = serde_json::from_str(stdout.trim()).unwrap_or(Value::Null);
    (out.status.success(), parsed, stdout)
}

fn session_record(repo: &Path, id: &str) -> Value {
    let raw =
        std::fs::read_to_string(repo.join(".bee").join("sessions").join(format!("{id}.json")))
            .unwrap();
    serde_json::from_str(&raw).unwrap()
}

#[test]
fn show_reports_a_null_record_rather_than_an_error_when_no_prompt_has_opened_one() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "empty");
    let (ok, v, _) = work(&repo, &["show", "--session", "s-none"]);
    assert!(ok, "an absent record is not an error");
    assert_eq!(v["work"], Value::Null);
}

#[test]
fn an_acceptance_lands_on_the_record_the_hook_opened_and_show_reads_it_back() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "upgrade");
    prompt(&repo, "s-up", "make the board name the work");

    let (ok, v, err) = work(
        &repo,
        &["set", "--session", "s-up", "--acceptance", "Every live session names its ask"],
    );
    assert!(ok, "{err}");
    assert_eq!(v["work"]["acceptance"], "Every live session names its ask");
    assert_eq!(v["work"]["status"], "open", "an acceptance alone never moves the status");
    assert_eq!(v["work"]["title"], "make the board name the work");

    let (_, shown, _) = work(&repo, &["show", "--session", "s-up"]);
    assert_eq!(shown["work"]["acceptance"], "Every live session names its ask");
}

#[test]
fn the_status_moves_through_every_legal_value() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "statuses");
    prompt(&repo, "s-st", "the one ask");
    for status in ["active", "done", "dropped", "open"] {
        let (ok, v, err) = work(&repo, &["set", "--session", "s-st", "--status", status]);
        assert!(ok, "{status}: {err}");
        assert_eq!(v["work"]["status"], status);
    }
}

#[test]
fn a_status_outside_the_vocabulary_is_refused_and_nothing_is_written() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "bad-status");
    prompt(&repo, "s-bad", "the one ask");
    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(["work", "set", "--session", "s-bad", "--status", "shipped"])
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(stderr.contains("open, active, done, dropped"), "the refusal names the legal set: {stderr}");
    assert_eq!(session_record(&repo, "s-bad")["work"]["status"], "open");
}

#[test]
fn a_set_with_neither_flag_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "no-flags");
    prompt(&repo, "s-nf", "the one ask");
    let (ok, v, _) = work(&repo, &["set", "--session", "s-nf"]);
    assert!(!ok);
    assert!(v["error"].as_str().unwrap().contains("nothing to set"));
}

#[test]
fn an_empty_acceptance_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "empty-acc");
    prompt(&repo, "s-ea", "the one ask");
    let (ok, v, _) = work(&repo, &["set", "--session", "s-ea", "--acceptance", "   "]);
    assert!(!ok);
    assert!(v["error"].as_str().unwrap().contains("--acceptance is empty"));
}

#[test]
fn a_secret_shaped_acceptance_is_refused_and_never_stored() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "secret-acc");
    prompt(&repo, "s-sa", "the one ask");
    let (ok, v, _) = work(
        &repo,
        &["set", "--session", "s-sa", "--acceptance", "check with ghp_abcdefghijklmnopqrstuvwxyz01"],
    );
    assert!(!ok);
    assert!(v["error"].as_str().unwrap().contains("secret pattern"));
    let record = session_record(&repo, "s-sa");
    assert!(record["work"].get("acceptance").is_none(), "nothing was stored");
    assert!(
        !serde_json::to_string(&record).unwrap().contains("ghp_"),
        "the credential never reached the file"
    );
}

#[test]
fn setting_a_record_that_no_prompt_opened_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "missing");
    let (ok, v, _) = work(&repo, &["set", "--session", "s-missing", "--status", "done"]);
    assert!(!ok);
    assert!(v["error"].as_str().unwrap().contains("a prompt opens one"));
}

#[test]
fn a_record_the_agent_moved_off_open_makes_the_next_prompt_open_a_fresh_one() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "reopen");
    prompt(&repo, "s-re", "the first ask");
    work(&repo, &["set", "--session", "s-re", "--status", "done"]);
    prompt(&repo, "s-re", "a different ask");

    let (_, v, _) = work(&repo, &["show", "--session", "s-re"]);
    assert_eq!(v["work"]["title"], "a different ask");
    assert_eq!(v["work"]["status"], "open");
    assert_eq!(v["work"]["turns"], 1);
    assert!(v["work"].get("acceptance").is_none());
}

#[test]
fn the_session_env_var_resolves_the_record_when_no_flag_names_one() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "env-session");
    prompt(&repo, "s-env", "the env ask");
    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(["work", "show", "--json"])
        .env("CLAUDE_CODE_SESSION_ID", "s-env")
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["work"]["title"], "the env ask");
}

#[test]
fn with_nothing_to_address_the_call_is_refused_rather_than_guessing() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "no-target");
    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(["work", "show", "--json"])
        .env_remove("CLAUDE_CODE_SESSION_ID")
        .env_remove("BEE_HERDING_WORKER")
        .env_remove("BEE_HERDING_JOB_ID")
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(!out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["error"].as_str().unwrap().contains("--session"));
}

#[test]
fn a_herded_pane_reads_and_writes_its_job_mailbox_and_never_a_session_file() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "herded");
    let mailbox = repo.join(".bee").join("mailbox").join("job-a");
    std::fs::create_dir_all(&mailbox).unwrap();
    std::fs::write(mailbox.join("brief-1.txt"), "do the thing\n").unwrap();

    let payload = serde_json::json!({
        "hook_event_name": "UserPromptSubmit",
        "session_id": "s-herded",
        "cwd": repo.to_string_lossy(),
        "prompt": "the herded ask",
    });
    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(["hook", "activity"])
        .env("BEE_HERDING_WORKER", "1")
        .env("BEE_HERDING_JOB_ID", "job-a")
        .current_dir(&repo)
        .write_stdin(payload.to_string())
        .output()
        .unwrap();
    assert!(out.status.success());

    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(["work", "set", "--status", "active", "--json"])
        .env("BEE_HERDING_WORKER", "1")
        .env("BEE_HERDING_JOB_ID", "job-a")
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["target"], "job job-a");
    assert_eq!(v["work"]["status"], "active");
    assert_eq!(v["work"]["text"], "the herded ask");

    let raw = std::fs::read_to_string(mailbox.join("activity.json")).unwrap();
    let record: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(record["work"]["status"], "active");
    assert!(
        !repo.join(".bee").join("sessions").join("s-herded.json").exists(),
        "a herded pane is not a bee session"
    );
}
