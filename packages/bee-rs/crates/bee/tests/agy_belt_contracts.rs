use serde_json::{Value, json};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const SESSION_VARS: &[&str] = &[
    "BEE_HERDING_WORKER",
    "BEE_HERDING_JOB_ID",
    "BEE_SESSION_ID",
    "CLAUDE_CODE_SESSION_ID",
    "PI_SESSION_ID",
    "ANTIGRAVITY_CONVERSATION_ID",
    "CODEX_THREAD_ID",
];

fn bee_bin() -> PathBuf {
    assert_cmd::cargo::cargo_bin("bee")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .expect("CARGO_MANIFEST_DIR is four levels below the checkout root")
        .to_path_buf()
}

fn isolated_global_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().expect("isolated global config dir"))
        .path()
}

fn bee() -> Command {
    let mut cmd = Command::new(bee_bin());
    for var in SESSION_VARS {
        cmd.env_remove(var);
    }
    let dir = isolated_global_dir();
    cmd.env("BEEHIVE_CONFIG_DIR", dir)
        .env("HOME", dir)
        .env("USERPROFILE", dir);
    cmd
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?} failed in {}", dir.display());
}

struct Repo {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

fn git_repo(owner_hooks: Option<&str>) -> Repo {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dunce::canonicalize(dir.path()).expect("canonical tempdir");
    git(&root, &["init", "-q"]);
    std::fs::write(root.join("README.md"), "# agy belt fixture\n").unwrap();
    git(&root, &["add", "README.md"]);
    git(
        &root,
        &[
            "-c",
            "user.name=Bee Tester",
            "-c",
            "user.email=tester@example.com",
            "commit",
            "-q",
            "-m",
            "initial",
        ],
    );
    if let Some(raw) = owner_hooks {
        std::fs::create_dir_all(root.join(".agents")).unwrap();
        std::fs::write(root.join(".agents/hooks.json"), raw).unwrap();
    }
    Repo { _dir: dir, root }
}

fn onboard(root: &Path) -> Output {
    bee()
        .args(["onboard", "--repo-root", root.to_str().unwrap(), "--apply", "--json"])
        .current_dir(repo_root())
        .output()
        .expect("bee onboard --apply runs")
}

fn onboarded_repo(owner_hooks: Option<&str>) -> Repo {
    let repo = git_repo(owner_hooks);
    let out = onboard(&repo.root);
    assert!(
        out.status.success(),
        "bee onboard --apply failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    repo
}

fn one_json_object(out: &Output, row: &str) -> Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{row}: agy-belt must exit 0\nstdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let body = stdout.trim_end_matches('\n');
    assert!(
        !body.is_empty() && !body.contains('\n'),
        "{row}: agy-belt must print exactly one JSON line, got: {stdout:?}"
    );
    let value: Value = serde_json::from_str(body)
        .unwrap_or_else(|e| panic!("{row}: stdout is not JSON ({e}): {stdout:?}"));
    assert!(value.is_object(), "{row}: answer must be a JSON object: {value}");
    value
}

fn belt_raw(root: &Path, stdin: &[u8], row: &str) -> Value {
    run_belt(bee(), root, stdin, row)
}

fn herded_belt(root: &Path, payload: Value, row: &str) -> Value {
    let mut cmd = bee();
    cmd.env("BEE_HERDING_WORKER", "1");
    run_belt(cmd, root, payload.to_string().as_bytes(), row)
}

fn run_belt(mut cmd: Command, root: &Path, stdin: &[u8], row: &str) -> Value {
    let mut child = cmd
        .args(["hook", "agy-belt"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("bee hook agy-belt spawns");
    child.stdin.take().unwrap().write_all(stdin).ok();
    let out = child.wait_with_output().expect("bee hook agy-belt exits");
    one_json_object(&out, row)
}

fn belt(root: &Path, payload: Value, row: &str) -> Value {
    belt_raw(root, payload.to_string().as_bytes(), row)
}

fn common(root: &Path, event: &str) -> Value {
    json!({
        "conversationId": "agy-conv-1",
        "workspacePaths": [root.to_str().unwrap()],
        "transcriptPath": "",
        "modelName": "gemini-3.8-flash-medium",
        "lastUserInput": "build the feature",
        "hookEventName": event,
        "cwd": root.to_str().unwrap()
    })
}

fn tool_call(root: &Path, event: &str, name: &str, args: Value) -> Value {
    let mut payload = common(root, event);
    payload["toolCall"] = json!({ "id": "call-1", "name": name, "args": args });
    payload["stepIdx"] = json!(3);
    payload
}

fn abs(root: &Path, rel: &str) -> String {
    root.join(rel).to_str().unwrap().to_string()
}

fn assert_deny(answer: &Value, row: &str) -> String {
    assert_eq!(answer["decision"], json!("deny"), "{row}: {answer}");
    let reason = answer["reason"].as_str().unwrap_or_default().to_string();
    assert!(!reason.trim().is_empty(), "{row}: a deny names its reason: {answer}");
    reason
}

#[test]
fn write_file_to_a_governed_path_before_the_gate_is_denied() {
    let repo = onboarded_repo(None);
    let answer = belt(
        &repo.root,
        tool_call(
            &repo.root,
            "PreToolUse",
            "write_file",
            json!({ "Path": abs(&repo.root, "src/main.rs"), "Content": "fn main() {}\n" }),
        ),
        "write_file src/main.rs while idle",
    );
    let reason = assert_deny(&answer, "write_file src/main.rs while idle");
    assert!(reason.contains("src/main.rs"), "reason names the refused path: {reason}");
}

#[test]
fn edit_file_to_a_governed_path_before_the_gate_is_denied() {
    let repo = onboarded_repo(None);
    let answer = belt(
        &repo.root,
        tool_call(
            &repo.root,
            "PreToolUse",
            "edit_file",
            json!({
                "Path": abs(&repo.root, "README.md"),
                "Target": "fixture",
                "Replacement": "changed"
            }),
        ),
        "edit_file README.md while idle",
    );
    assert_deny(&answer, "edit_file README.md while idle");
}

#[test]
fn write_file_to_a_secret_shaped_file_is_denied() {
    let repo = onboarded_repo(None);
    let answer = belt(
        &repo.root,
        tool_call(
            &repo.root,
            "PreToolUse",
            "write_file",
            json!({ "Path": abs(&repo.root, ".env"), "Content": "TOKEN=x\n" }),
        ),
        "write_file .env",
    );
    assert_deny(&answer, "write_file .env");
}

#[test]
fn read_file_of_a_secret_shaped_file_is_denied_by_the_privacy_guard() {
    let repo = onboarded_repo(None);
    let answer = belt(
        &repo.root,
        tool_call(&repo.root, "PreToolUse", "read_file", json!({ "Path": abs(&repo.root, ".env") })),
        "read_file .env",
    );
    let reason = assert_deny(&answer, "read_file .env");
    assert!(reason.contains(".env"), "reason names the secret file: {reason}");
}

#[test]
fn an_allowed_read_file_answers_ask_never_an_empty_object() {
    let repo = onboarded_repo(None);
    let answer = belt(
        &repo.root,
        tool_call(
            &repo.root,
            "PreToolUse",
            "read_file",
            json!({ "Path": abs(&repo.root, "README.md") }),
        ),
        "read_file README.md",
    );
    assert_eq!(answer, json!({ "decision": "ask" }));
}

#[test]
fn a_no_write_tool_answers_ask() {
    let repo = onboarded_repo(None);
    let answer = belt(
        &repo.root,
        tool_call(&repo.root, "PreToolUse", "search_web", json!({ "query": "rust" })),
        "search_web",
    );
    assert_eq!(answer, json!({ "decision": "ask" }));
}

#[test]
fn run_command_the_cli_shape_guard_refuses_is_denied() {
    let repo = onboarded_repo(None);
    let answer = belt(
        &repo.root,
        tool_call(
            &repo.root,
            "PreToolUse",
            "run_command",
            json!({
                "CommandLine": "bee cells claim --bogus",
                "WaitMsBeforeAsync": 0,
                "toolSummary": "claim a cell"
            }),
        ),
        "run_command bee cells claim --bogus",
    );
    let reason = assert_deny(&answer, "run_command bee cells claim --bogus");
    assert!(reason.contains("CLI-shape"), "reason names the CLI-shape guard: {reason}");
}

#[test]
fn run_command_that_writes_a_governed_path_before_the_gate_is_denied() {
    let repo = onboarded_repo(None);
    let answer = belt(
        &repo.root,
        tool_call(
            &repo.root,
            "PreToolUse",
            "run_command",
            json!({ "CommandLine": "echo x > src/lib.rs", "WaitMsBeforeAsync": 0, "toolSummary": "write" }),
        ),
        "run_command echo > src/lib.rs",
    );
    let reason = assert_deny(&answer, "run_command echo > src/lib.rs");
    assert!(reason.contains("src/lib.rs"), "reason names the refused path: {reason}");
}

#[test]
fn a_harmless_run_command_answers_ask() {
    let repo = onboarded_repo(None);
    let answer = belt(
        &repo.root,
        tool_call(
            &repo.root,
            "PreToolUse",
            "run_command",
            json!({ "CommandLine": "git status", "WaitMsBeforeAsync": 0, "toolSummary": "status" }),
        ),
        "run_command git status",
    );
    assert_eq!(answer, json!({ "decision": "ask" }));
}

#[test]
fn invoke_subagent_is_denied_naming_the_agy_dispatch_door() {
    let repo = onboarded_repo(None);
    let answer = belt(
        &repo.root,
        tool_call(
            &repo.root,
            "PreToolUse",
            "invoke_subagent",
            json!({ "Prompt": "research this", "Role": "researcher", "TypeName": "research", "Model": "inherit" }),
        ),
        "invoke_subagent",
    );
    let reason = assert_deny(&answer, "invoke_subagent");
    assert!(
        reason.contains("bee dispatch prepare --runtime agy"),
        "reason names the one dispatch door: {reason}"
    );
}

#[test]
fn garbage_stdin_on_pre_tool_use_denies_and_exits_zero() {
    let repo = onboarded_repo(None);
    for (row, raw) in [
        ("not json", &b"not json {{{"[..]),
        ("empty stdin", &b""[..]),
        ("json array", &b"[1, 2]"[..]),
    ] {
        let answer = belt_raw(&repo.root, raw, row);
        assert_deny(&answer, row);
    }
    let mut no_tool = common(&repo.root, "PreToolUse");
    no_tool["stepIdx"] = json!(1);
    let answer = belt(&repo.root, no_tool, "PreToolUse without toolCall");
    assert_deny(&answer, "PreToolUse without toolCall");
}

#[test]
fn the_first_pre_invocation_injects_steps_and_a_later_one_prints_an_empty_object() {
    let repo = onboarded_repo(None);
    let mut first = common(&repo.root, "PreInvocation");
    first["invocationNum"] = json!(0);
    first["initialNumSteps"] = json!(0);
    let answer = belt(&repo.root, first, "PreInvocation 0");
    let steps = answer["injectSteps"]
        .as_array()
        .unwrap_or_else(|| panic!("PreInvocation 0 must inject steps: {answer}"));
    assert!(!steps.is_empty(), "{answer}");
    let text = steps[0]["systemMessage"]["systemMessage"].as_str().unwrap_or_default();
    assert!(text.contains("bee"), "injected text carries the bee context: {text}");

    let mut later = common(&repo.root, "PreInvocation");
    later["invocationNum"] = json!(3);
    later["initialNumSteps"] = json!(7);
    assert_eq!(belt(&repo.root, later, "PreInvocation 3"), json!({}));
}

#[test]
fn session_start_injects_steps_with_bee_text() {
    let repo = onboarded_repo(None);
    let answer = belt(&repo.root, common(&repo.root, "SessionStart"), "SessionStart");
    let steps = answer["injectSteps"]
        .as_array()
        .unwrap_or_else(|| panic!("SessionStart must inject steps: {answer}"));
    let all: String = steps
        .iter()
        .filter_map(|s| s["systemMessage"]["systemMessage"].as_str())
        .collect();
    assert!(all.contains("bee"), "SessionStart text carries the bee preamble: {all}");
}

#[test]
fn stop_and_post_tool_use_print_one_json_object() {
    let repo = onboarded_repo(None);
    let mut stop = common(&repo.root, "Stop");
    stop["executionNum"] = json!(1);
    stop["terminationReason"] = json!("completed");
    stop["error"] = json!("");
    stop["fullyIdle"] = json!(true);
    stop["continuationActive"] = json!(false);
    let answer = belt(&repo.root, stop, "Stop");
    if !answer.as_object().unwrap().is_empty() {
        assert_eq!(answer["decision"], json!("continue"), "Stop answers {{}} or continue: {answer}");
    }

    let post = tool_call(
        &repo.root,
        "PostToolUse",
        "read_file",
        json!({ "Path": abs(&repo.root, "README.md") }),
    );
    assert_eq!(belt(&repo.root, post, "PostToolUse"), json!({}));
}

#[test]
fn a_herded_pane_list_dir_answers_ask() {
    let repo = onboarded_repo(None);
    let answer = herded_belt(
        &repo.root,
        tool_call(&repo.root, "PreToolUse", "list_dir", json!({ "DirectoryPath": abs(&repo.root, "") })),
        "herded list_dir",
    );
    assert_eq!(answer, json!({ "decision": "ask" }));
}

#[test]
fn a_herded_pane_write_file_the_write_guard_refuses_is_denied() {
    let repo = onboarded_repo(None);
    let answer = herded_belt(
        &repo.root,
        tool_call(
            &repo.root,
            "PreToolUse",
            "write_file",
            json!({ "Path": abs(&repo.root, ".bee/config.json"), "Content": "{}\n" }),
        ),
        "herded write_file .bee/config.json",
    );
    let reason = assert_deny(&answer, "herded write_file .bee/config.json");
    assert!(reason.contains(".bee/config.json"), "reason names the refused path: {reason}");
}

#[test]
fn a_herded_pane_invoke_subagent_is_denied() {
    let repo = onboarded_repo(None);
    let answer = herded_belt(
        &repo.root,
        tool_call(
            &repo.root,
            "PreToolUse",
            "invoke_subagent",
            json!({ "Prompt": "research this", "Role": "researcher", "TypeName": "research", "Model": "inherit" }),
        ),
        "herded invoke_subagent",
    );
    assert_deny(&answer, "herded invoke_subagent");
}

#[test]
fn a_herded_pane_session_start_and_stop_print_one_json_object() {
    let repo = onboarded_repo(None);
    herded_belt(&repo.root, common(&repo.root, "SessionStart"), "herded SessionStart");
    let mut stop = common(&repo.root, "Stop");
    stop["executionNum"] = json!(1);
    stop["terminationReason"] = json!("completed");
    stop["error"] = json!("");
    stop["fullyIdle"] = json!(true);
    stop["continuationActive"] = json!(false);
    herded_belt(&repo.root, stop, "herded Stop");
}

#[test]
fn the_shipped_hooks_json_command_reaches_the_belt_from_the_agents_folder() {
    let repo = onboarded_repo(None);
    let bin_dir = repo.root.join(".bee/bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let target = bin_dir.join("bee");
    std::fs::copy(bee_bin(), &target).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let hooks: Value =
        serde_json::from_str(&std::fs::read_to_string(repo.root.join(".agents/hooks.json")).unwrap())
            .unwrap();
    let command = hooks["bee"]["PreToolUse"][0]["hooks"][0]["command"]
        .as_str()
        .expect("bee PreToolUse handler command");
    let payload = tool_call(
        &repo.root,
        "PreToolUse",
        "invoke_subagent",
        json!({ "Prompt": "p", "Role": "r", "TypeName": "self", "Model": "inherit" }),
    );
    let mut sh = Command::new("sh");
    sh.args(["-c", command]).current_dir(repo.root.join(".agents"));
    for var in SESSION_VARS {
        sh.env_remove(var);
    }
    let mut child = sh
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let answer = one_json_object(&out, "hooks.json PreToolUse via sh -c");
    let reason = assert_deny(&answer, "hooks.json PreToolUse via sh -c");
    assert!(reason.contains("bee dispatch prepare --runtime agy"), "{reason}");
}

const OWNER_HOOKS: &str = r#"{
  "owner": {
    "PreToolUse": [
      {
        "matcher": "run_command",
        "hooks": [{ "type": "command", "command": "echo owner", "timeout": 5 }]
      }
    ]
  }
}
"#;

fn shipped_bee_key() -> Value {
    let raw = std::fs::read_to_string(repo_root().join(".agents/hooks.json"))
        .expect("checked-in .agents/hooks.json");
    let parsed: Value = serde_json::from_str(&raw).unwrap();
    parsed["bee"].clone()
}

#[test]
fn onboarding_merges_the_bee_key_and_keeps_the_owner_key() {
    let repo = onboarded_repo(Some(OWNER_HOOKS));
    let hooks_path = repo.root.join(".agents/hooks.json");
    let after_first = std::fs::read(&hooks_path).unwrap();
    let merged: Value = serde_json::from_slice(&after_first).unwrap();
    let owner_before: Value = serde_json::from_str(OWNER_HOOKS).unwrap();

    assert_eq!(merged["owner"], owner_before["owner"], "owner key unchanged: {merged}");
    assert_eq!(merged["bee"], shipped_bee_key(), "bee key is the shipped belt");
    let keys: Vec<&String> = merged.as_object().unwrap().keys().collect();
    assert_eq!(keys.len(), 2, "only the bee key was added: {keys:?}");
    for event in ["SessionStart", "PreInvocation", "PreToolUse", "PostToolUse", "Stop"] {
        assert!(merged["bee"].get(event).is_some(), "bee key wires {event}");
    }

    let backup = repo.root.join(".agents/hooks.json.bee-backup");
    assert_eq!(
        std::fs::read_to_string(&backup).expect("pre-existing hooks.json is backed up"),
        OWNER_HOOKS
    );

    let second = onboard(&repo.root);
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    assert_eq!(
        std::fs::read(&hooks_path).unwrap(),
        after_first,
        "a second apply leaves .agents/hooks.json byte-equal"
    );
}

#[test]
fn onboarding_writes_the_bee_key_into_a_fresh_repo() {
    let repo = onboarded_repo(None);
    let merged: Value =
        serde_json::from_str(&std::fs::read_to_string(repo.root.join(".agents/hooks.json")).unwrap())
            .unwrap();
    assert_eq!(merged, json!({ "bee": shipped_bee_key() }));
}

#[test]
fn onboarding_refuses_a_malformed_hooks_json_and_leaves_it_unchanged() {
    for (row, raw) in [("truncated", "{\"owner\": [1,"), ("array", "[1, 2]\n")] {
        let repo = git_repo(Some(raw));
        let out = onboard(&repo.root);
        assert!(!out.status.success(), "{row}: a malformed hooks.json is refused");
        let report: Value = serde_json::from_slice(&out.stdout)
            .unwrap_or_else(|e| panic!("{row}: onboard --json prints JSON ({e})"));
        assert_eq!(report["status"], json!("blocked_hooks_merge"), "{row}: {report}");
        assert!(
            report["reason"].as_str().unwrap_or_default().contains(".agents/hooks.json"),
            "{row}: {report}"
        );
        assert_eq!(
            std::fs::read_to_string(repo.root.join(".agents/hooks.json")).unwrap(),
            raw,
            "{row}: refused file left byte-equal"
        );
        assert!(
            !repo.root.join(".agents/hooks.json.bee-backup").exists(),
            "{row}: nothing written beside a refused file"
        );
    }
}

#[test]
fn dispatch_prepare_on_agy_with_a_native_slot_refuses_with_the_herding_remedy() {
    let repo = onboarded_repo(None);
    let team_path = repo.root.join(".bee/team.json");
    let mut team: Value =
        serde_json::from_str(&std::fs::read_to_string(&team_path).unwrap()).unwrap();
    let native = json!({ "kind": "native", "model": "gemini-3" });
    team["team"]["agy"] = json!({
        "generation": native,
        "read": native,
        "code": native,
        "review": native,
        "advisor": native
    });
    std::fs::write(&team_path, serde_json::to_string_pretty(&team).unwrap()).unwrap();

    let out = bee()
        .args(["dispatch", "prepare", "--runtime", "agy", "--kind", "gather", "--purpose", "probe", "--json"])
        .current_dir(&repo.root)
        .output()
        .unwrap();
    let answer: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "dispatch prepare prints JSON ({e})\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    });
    let explain = bee()
        .args(["config", "explain", "--key", "team.agy.read", "--json"])
        .current_dir(&repo.root)
        .output()
        .unwrap();
    let explain_stdout = String::from_utf8_lossy(&explain.stdout);
    assert_eq!(answer["ok"], json!(false), "{answer}\nexplain: {explain_stdout}");
    assert_eq!(answer["reason"], json!("agy_requires_herding"), "{answer}");
    assert_eq!(answer["runtime"], json!("agy"), "{answer}");
    let fix = answer["fix"].as_str().unwrap_or_default();
    assert!(fix.contains("\"kind\":\"herding\""), "fix names the herding remedy: {fix}");
    assert!(fix.contains("team.agy."), "fix names the team.agy slot: {fix}");
}
