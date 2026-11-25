use assert_cmd::Command;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

struct Fixture {
    _tmp: tempfile::TempDir,
    repo: PathBuf,
    fake_dir: PathBuf,
    scratch: PathBuf,
}

fn git(cwd: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(["-c", "user.email=t@example.invalid", "-c", "user.name=t"])
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git on PATH");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn path_without_waggledance() -> String {
    let raw = std::env::var_os("PATH").unwrap_or_default();
    let dirs: Vec<PathBuf> = std::env::split_paths(&raw)
        .filter(|d| !d.join("waggledance").exists())
        .collect();
    std::env::join_paths(dirs).unwrap().to_string_lossy().into_owned()
}

fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let base = std::fs::canonicalize(tmp.path()).unwrap();
    let repo = base.join("repo");
    let fake_dir = base.join("fake-bin");
    let scratch = base.join("scratch");
    std::fs::create_dir_all(repo.join(".bee").join("logs")).unwrap();
    std::fs::create_dir_all(fake_dir.join("runs")).unwrap();
    std::fs::create_dir_all(&scratch).unwrap();

    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "init"]);
    git(&repo, &["checkout", "-q", "-b", "wt/feat-x"]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "feature work"]);
    git(&repo, &["checkout", "-q", "main"]);

    std::fs::write(
        repo.join(".bee/onboarding.json"),
        r#"{"version":1,"completed":true}"#,
    )
    .unwrap();
    std::fs::write(repo.join(".bee/config.json"), r#"{"commands":{"test":"none"}}"#).unwrap();
    std::fs::write(
        repo.join(".bee/state.json"),
        r#"{"phase":"executing","gates":{},"feature":"feat-x"}"#,
    )
    .unwrap();

    let script = format!(
        r#"#!/usr/bin/env bash
D="$(cd "$(dirname "$0")" && pwd)"
printf '%s\n' "$*" >> "$D/argv.log"
case "$1" in
  list) printf '{{"projects":[{{"id":"demo","root_path":"%s"}}]}}\n' "{root}" ;;
  dispatch)
    n=$(cat "$D/count" 2>/dev/null || echo 0)
    n=$((n+1))
    echo "$n" > "$D/count"
    printf '{{"run_id":"run-new-%s"}}\n' "$n" ;;
  runs)
    if [ "$2" = show ] && [ -f "$D/runs/$3.json" ]; then cat "$D/runs/$3.json"; else exit 1; fi ;;
  *) exit 2 ;;
esac
"#,
        root = repo.display()
    );
    let bin = fake_dir.join("waggledance");
    std::fs::write(&bin, script).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();

    Fixture {
        _tmp: tmp,
        repo,
        fake_dir,
        scratch,
    }
}

impl Fixture {
    fn path_with_door(&self) -> String {
        format!("{}:{}", self.fake_dir.display(), path_without_waggledance())
    }

    fn cmd(&self, door: bool) -> Command {
        let mut cmd = Command::cargo_bin("bee").unwrap();
        cmd.current_dir(&self.repo)
            .env(
                "PATH",
                if door {
                    self.path_with_door()
                } else {
                    path_without_waggledance()
                },
            )
            .env("BEEHIVE_PERF_DIR", self.scratch.join("perf"))
            .env("CLAUDE_CONFIG_DIR", self.scratch.join("claude"))
            .env("BEE_HOOK_NO_DELEGATE", "1")
            .env_remove("BEE_HERDING_WORKER")
            .env_remove("BEE_HERDING_JOB_ID")
            .env_remove("BEE_SESSION_ID")
            .env_remove("ANTIGRAVITY_CONVERSATION_ID")
            .env_remove("CLAUDE_CODE_SESSION_ID")
            .env_remove("PI_SESSION_ID")
            .env_remove("PASEO_AGENT_ID");
        cmd
    }

    fn bee(&self, door: bool, args: &[&str]) -> String {
        let out = self.cmd(door).args(args).output().unwrap();
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    }

    fn send(&self) -> String {
        self.bee(
            true,
            &["state", "handoff", "send", "--task", "finish feat-x", "--session-id", "sess-a"],
        )
    }

    fn check(&self, door: bool, extra: &[&str]) -> String {
        let mut args = vec!["state", "handoff", "check"];
        args.extend_from_slice(extra);
        self.bee(door, &args)
    }

    fn set_run(&self, run_id: &str, status: &str) {
        let doc = json!({"runs": [{"id": run_id, "status": status, "task": "finish feat-x"}]});
        std::fs::write(
            self.fake_dir.join("runs").join(format!("{run_id}.json")),
            doc.to_string(),
        )
        .unwrap();
    }

    fn argv_log(&self) -> Vec<String> {
        std::fs::read_to_string(self.fake_dir.join("argv.log"))
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn dispatches(&self) -> Vec<Vec<String>> {
        self.argv_log()
            .into_iter()
            .filter(|l| l.starts_with("dispatch "))
            .map(|l| l.split_whitespace().map(str::to_string).collect())
            .collect()
    }

    fn ledger_files(&self) -> Vec<PathBuf> {
        let dir = self.repo.join(".bee").join("handoffs");
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut out: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        out.sort();
        out
    }

    fn ledger_row(&self, request_id: &str) -> Value {
        let path = self
            .repo
            .join(".bee")
            .join("handoffs")
            .join(format!("{request_id}.json"));
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("ledger row {} missing: {e}", path.display()));
        serde_json::from_str(&raw).unwrap()
    }

    fn merge_feature_branch(&self) {
        git(&self.repo, &["merge", "-q", "--no-edit", "wt/feat-x"]);
    }
}

fn flag_after(argv: &[String], flag: &str) -> Option<String> {
    argv.iter()
        .position(|a| a == flag)
        .and_then(|i| argv.get(i + 1))
        .cloned()
}

#[test]
fn two_sends_dispatch_two_different_request_ids() {
    let fx = fixture();
    let first = fx.send();
    let second = fx.send();
    assert!(first.contains("[handoff_sent]"), "{first}");
    assert!(second.contains("[handoff_sent]"), "{second}");
    let ids: Vec<String> = fx
        .dispatches()
        .iter()
        .map(|argv| flag_after(argv, "--request-id").expect("dispatch carries --request-id"))
        .collect();
    assert_eq!(ids.len(), 2, "{:?}", fx.argv_log());
    assert_ne!(ids[0], ids[1], "two sends reused one request id");
    for id in &ids {
        assert!(id.starts_with("bee-handoff-sess-a-mark-"), "{id}");
    }
}

#[test]
fn a_dispatch_answered_by_a_done_run_ends_deduped_with_no_ledger_row() {
    let fx = fixture();
    fx.set_run("run-new-1", "done");
    let out = fx.send();
    assert!(
        out.contains("[handoff_deduped] run run-new-1 was already done"),
        "{out}"
    );
    assert!(!out.contains("[handoff_sent]"), "{out}");
    assert!(fx.ledger_files().is_empty(), "{:?}", fx.ledger_files());
}

#[test]
fn a_sent_handoff_writes_a_ledger_row_expecting_the_unmerged_feature_branch() {
    let fx = fixture();
    let out = fx.send();
    assert!(out.contains("[handoff_sent] run run-new-1"), "{out}");
    let dispatch = fx.dispatches().pop().expect("one dispatch");
    let request_id = flag_after(&dispatch, "--request-id").unwrap();
    let row = fx.ledger_row(&request_id);
    assert_eq!(row["request_id"], request_id.as_str());
    assert_eq!(row["run_id"], "run-new-1");
    assert_eq!(row["status"], "open");
    assert_eq!(
        row["expects"],
        json!([{"kind": "branch-merged", "branch": "wt/feat-x"}])
    );
    assert_eq!(fx.ledger_files().len(), 1);
}

#[test]
fn a_done_run_with_an_unmerged_branch_gets_a_follow_up_dispatch() {
    let fx = fixture();
    fx.set_run("run-old", "done");
    let out = fx.check(true, &["--run", "run-old", "--expect-merge", "wt/feat-x"]);
    assert!(out.contains("follow-up sent run run-new-1"), "{out}");
    let dispatches = fx.dispatches();
    assert_eq!(dispatches.len(), 1, "{:?}", fx.argv_log());
    let argv = &dispatches[0];
    assert_eq!(flag_after(argv, "--follow-up-of").as_deref(), Some("run-old"));
    assert!(argv.iter().any(|a| a == "--handoff"), "{argv:?}");
    assert!(argv.iter().any(|a| a == "--leader"), "{argv:?}");
    assert_eq!(flag_after(argv, "--project").as_deref(), Some("demo"));
    assert_eq!(
        flag_after(argv, "--request-id").as_deref(),
        Some("bee-handoff-followup-run-old-1")
    );
    let task_file = flag_after(argv, "--task-file").unwrap();
    let task = std::fs::read_to_string(&task_file).unwrap();
    assert!(task.contains("wt/feat-x"), "{task}");
    assert!(task.contains("finish feat-x"), "{task}");

    let row = fx.ledger_row("adopted-run-old");
    assert_eq!(row["status"], "open");
    assert_eq!(row["run_id"], "run-new-1");
    let follow_ups = row["follow_ups"].as_array().unwrap();
    assert_eq!(follow_ups.len(), 1);
    assert_eq!(follow_ups[0]["run_id"], "run-new-1");
}

#[test]
fn a_merged_branch_flips_the_row_to_met_without_a_dispatch() {
    let fx = fixture();
    fx.merge_feature_branch();
    fx.set_run("run-old", "done");
    let out = fx.check(true, &["--run", "run-old", "--expect-merge", "wt/feat-x"]);
    assert!(out.contains("run run-old: met"), "{out}");
    assert!(fx.dispatches().is_empty(), "{:?}", fx.argv_log());
    assert_eq!(fx.ledger_row("adopted-run-old")["status"], "met");
}

#[test]
fn a_fourth_check_after_three_follow_ups_dispatches_nothing_and_needs_owner() {
    let fx = fixture();
    fx.set_run("run-old", "done");
    for n in 1..=3 {
        fx.set_run(&format!("run-new-{n}"), "done");
    }
    for n in 1..=3 {
        let out = fx.check(true, &["--run", "run-old", "--expect-merge", "wt/feat-x"]);
        assert!(out.contains(&format!("follow-up sent run run-new-{n}")), "{out}");
    }
    assert_eq!(fx.dispatches().len(), 3, "{:?}", fx.argv_log());
    let fourth = fx.check(true, &[]);
    assert!(fourth.contains("needs-owner: wt/feat-x"), "{fourth}");
    assert_eq!(fx.dispatches().len(), 3, "{:?}", fx.argv_log());
    let row = fx.ledger_row("adopted-run-old");
    assert_eq!(row["status"], "needs-owner");
    assert_eq!(row["follow_ups"].as_array().unwrap().len(), 3);
}

#[test]
fn with_waggledance_off_path_the_check_dispatches_nothing() {
    let fx = fixture();
    fx.set_run("run-old", "done");
    let out = fx.check(false, &["--run", "run-old", "--expect-merge", "wt/feat-x"]);
    assert!(out.contains("no door: wt/feat-x"), "{out}");
    assert!(fx.argv_log().is_empty(), "{:?}", fx.argv_log());
    let row = fx.ledger_row("adopted-run-old");
    assert_eq!(row["status"], "open");
    assert!(row["follow_ups"].as_array().unwrap().is_empty());
}

const HOOK_SID: &str = "sess-stop";

fn write_transcript(fx: &Fixture, reply: &str) {
    let transcript = fx.scratch.join(format!("{HOOK_SID}.jsonl"));
    let lines = [
        json!({"type": "user", "sessionId": HOOK_SID, "message": {"role": "user", "content": "keep going"}}),
        json!({"type": "assistant", "sessionId": HOOK_SID, "message": {"role": "assistant", "content": [{"type": "text", "text": reply}]}}),
    ];
    let body: String = lines.iter().map(|l| format!("{l}\n")).collect();
    std::fs::write(&transcript, body).unwrap();
    let sessions = fx.repo.join(".bee").join("sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    std::fs::write(
        sessions.join(format!("{HOOK_SID}.json")),
        json!({
            "id": HOOK_SID,
            "started_at": "2020-01-01T00:00:00.000Z",
            "last_heartbeat": "2020-01-01T00:00:00.000Z",
            "transcript_path": transcript.to_string_lossy(),
        })
        .to_string(),
    )
    .unwrap();
}

fn stop_hook(fx: &Fixture, door: bool, stop_hook_active: bool) -> Value {
    let payload = json!({
        "hook_event_name": "Stop",
        "session_id": HOOK_SID,
        "cwd": fx.repo.to_string_lossy(),
        "stop_hook_active": stop_hook_active,
    });
    let out = fx
        .cmd(door)
        .args(["hook", "session-close"])
        .write_stdin(payload.to_string())
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("stdout not JSON ({e}): {text}"))
    }
}

const OWNER_ASK: &str = "The feature is merged. Please open a new session to continue the next one.";

#[test]
fn stop_net_blocks_an_open_a_new_session_reply_when_a_door_is_present() {
    let fx = fixture();
    write_transcript(&fx, OWNER_ASK);
    let out = stop_hook(&fx, true, false);
    assert_eq!(out["decision"], "block", "{out}");
    let reason = out["reason"].as_str().unwrap_or_default();
    assert!(reason.contains("open a new session"), "{reason}");
    assert!(reason.contains("bee state handoff send --task"), "{reason}");
    assert!(reason.contains("bee state handoff check"), "{reason}");
}

#[test]
fn stop_net_passes_a_second_stop_in_the_same_turn() {
    let fx = fixture();
    write_transcript(&fx, OWNER_ASK);
    let out = stop_hook(&fx, true, true);
    assert_ne!(out["decision"], "block", "{out}");
}

#[test]
fn stop_net_adds_an_advisory_line_and_never_blocks_without_a_door() {
    let fx = fixture();
    write_transcript(&fx, OWNER_ASK);
    let out = stop_hook(&fx, false, false);
    assert_ne!(out["decision"], "block", "{out}");
    let message = out["systemMessage"].as_str().unwrap_or_default();
    assert!(message.contains("no waggledance door exists"), "{out}");
    assert!(message.contains("open a new session"), "{out}");
}
