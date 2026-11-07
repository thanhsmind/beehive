use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const RULE_ID: &str = "agents-knowledge-before-code";
const SEARCH_COMMAND: &str = ".bee/bin/bee knowledge search --text";

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = dunce::canonicalize(dir.path()).unwrap();
    std::fs::create_dir_all(root.join(".bee").join("logs")).unwrap();
    std::fs::create_dir_all(root.join(".bee").join("cells")).unwrap();
    std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
    std::fs::write(
        root.join(".bee").join("state.json"),
        serde_json::to_string_pretty(&json!({
            "phase": "swarming",
            "mode": "standard",
            "feature": "demo",
            "approved_gates": {"context": true, "shape": true, "execution": true, "review": false}
        }))
        .unwrap()
            + "\n",
    )
    .unwrap();
    Fixture { _dir: dir, root }
}

fn run_hook(hook: &str, payload: &Value, cwd: &Path, worker: bool) -> Output {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("bee"));
    cmd.args(["hook", hook])
        .current_dir(cwd)
        .env("BEE_HOOK_NO_DELEGATE", "1")
        .env_remove("BEE_HERDING_WORKER")
        .env_remove("BEE_HERDING_JOB_ID")
        .env_remove("BEE_SESSION_ID")
        .env_remove("CLAUDE_CODE_SESSION_ID")
        .env_remove("PI_SESSION_ID")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if worker {
        cmd.env("BEE_HERDING_WORKER", "1");
    }
    let mut child = cmd.spawn().unwrap();
    child.stdin.take().unwrap().write_all(payload.to_string().as_bytes()).ok();
    child.wait_with_output().unwrap()
}

fn pre_tool(fx: &Fixture, session: &str, tool: &str, input: Value) -> Value {
    json!({
        "hook_event_name": "PreToolUse",
        "session_id": session,
        "cwd": fx.root.to_string_lossy(),
        "tool_name": tool,
        "tool_input": input,
    })
}

fn guard(fx: &Fixture, payload: &Value) -> String {
    let out = run_hook("write-guard", payload, &fx.root, false);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn read_code(fx: &Fixture, session: &str) -> String {
    guard(fx, &pre_tool(fx, session, "Read", json!({"file_path": fx.root.join("packages/bee-rs/src/lib.rs").to_string_lossy()})))
}

fn assert_nudge(stdout: &str) {
    let v: Value = serde_json::from_str(stdout).unwrap_or_else(|e| panic!("not json ({e}): {stdout:?}"));
    let hso = &v["hookSpecificOutput"];
    assert!(hso.get("permissionDecision").is_none(), "an advisory must never carry a verdict: {stdout}");
    let ctx = hso["additionalContext"].as_str().unwrap_or_default();
    assert!(ctx.contains(RULE_ID), "names the rule id: {ctx}");
    assert!(ctx.contains(SEARCH_COMMAND), "names the exact command: {ctx}");
}

fn assert_silent(stdout: &str) {
    assert!(stdout.trim().is_empty(), "expected silence, got: {stdout}");
}

#[test]
fn first_code_read_without_a_search_gets_one_advisory() {
    let fx = fixture();
    assert_nudge(&read_code(&fx, "s1"));
}

#[test]
fn the_advisory_points_to_the_rule_and_does_not_restate_it() {
    let fx = fixture();
    let out = read_code(&fx, "s1");
    let v: Value = serde_json::from_str(&out).unwrap();
    let ctx = v["hookSpecificOutput"]["additionalContext"].as_str().unwrap();
    assert!(ctx.len() < 400, "a pointer, not the rule text: {ctx}");
}

#[test]
fn glob_and_grep_on_code_paths_nudge_too() {
    let fx = fixture();
    let out = guard(&fx, &pre_tool(&fx, "g1", "Grep", json!({"pattern": "fn main", "path": "packages/bee-rs"})));
    assert_nudge(&out);
    let out = guard(&fx, &pre_tool(&fx, "g2", "Glob", json!({"pattern": "src/**/*.ts"})));
    assert_nudge(&out);
}

#[test]
fn a_read_shaped_bash_command_on_code_nudges() {
    let fx = fixture();
    let out = guard(&fx, &pre_tool(&fx, "b1", "Bash", json!({"command": "rg -n handler packages/bee-rs/crates"})));
    assert_nudge(&out);
    let out = guard(&fx, &pre_tool(&fx, "b2", "Bash", json!({"command": "sed -n 1,40p tools/build.py"})));
    assert_nudge(&out);
}

#[test]
fn docs_bee_skills_and_markdown_reads_are_silent() {
    let fx = fixture();
    for (i, path) in [
        "docs/knowledge/areas/x.md",
        ".bee/LEADER.md",
        "skills/bee-hive/SKILL.md",
        "packages/bee/LEADER.md",
        "README.md",
        "src/notes.md",
    ]
    .iter()
    .enumerate()
    {
        let out = guard(&fx, &pre_tool(&fx, &format!("d{i}"), "Read", json!({"file_path": path})));
        assert_silent(&out);
    }
    let out = guard(&fx, &pre_tool(&fx, "dbash", "Bash", json!({"command": "cat docs/history/x/plan.md"})));
    assert_silent(&out);
    let out = guard(&fx, &pre_tool(&fx, "dgit", "Bash", json!({"command": "git status --short"})));
    assert_silent(&out);
}

#[test]
fn a_docs_read_does_not_spend_the_nudge() {
    let fx = fixture();
    assert_silent(&guard(&fx, &pre_tool(&fx, "s1", "Read", json!({"file_path": "docs/a.md"}))));
    assert_nudge(&read_code(&fx, "s1"));
}

#[test]
fn a_knowledge_search_in_the_turn_silences_the_nudge() {
    let fx = fixture();
    let search = pre_tool(&fx, "s1", "Bash", json!({"command": ".bee/bin/bee knowledge search --text \"write guard\""}));
    assert_silent(&guard(&fx, &search));
    assert_silent(&read_code(&fx, "s1"));
}

#[test]
fn a_knowledge_context_in_the_turn_silences_the_nudge() {
    let fx = fixture();
    let ctx = pre_tool(&fx, "s1", "Bash", json!({"command": "bee knowledge context --work \"read the write guard\" --budget 2000"}));
    assert_silent(&guard(&fx, &ctx));
    assert_silent(&read_code(&fx, "s1"));
}

#[test]
fn a_search_in_one_session_does_not_silence_another() {
    let fx = fixture();
    let search = pre_tool(&fx, "s1", "Bash", json!({"command": ".bee/bin/bee knowledge search --text \"x\""}));
    assert_silent(&guard(&fx, &search));
    assert_nudge(&read_code(&fx, "s2"));
}

#[test]
fn the_nudge_fires_once_per_turn_and_rearms_at_the_next_prompt() {
    let fx = fixture();
    assert_nudge(&read_code(&fx, "s1"));
    assert_silent(&read_code(&fx, "s1"));
    assert_silent(&read_code(&fx, "s1"));

    let prompt = json!({"hook_event_name": "UserPromptSubmit", "session_id": "s1", "cwd": fx.root.to_string_lossy(), "prompt": "next"});
    let out = run_hook("prompt-context", &prompt, &fx.root, false);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    assert_nudge(&read_code(&fx, "s1"));
    assert_silent(&read_code(&fx, "s1"));
}

#[test]
fn a_new_prompt_also_rearms_after_a_search_turn() {
    let fx = fixture();
    let search = pre_tool(&fx, "s1", "Bash", json!({"command": ".bee/bin/bee knowledge search --text \"x\""}));
    assert_silent(&guard(&fx, &search));
    assert_silent(&read_code(&fx, "s1"));

    let prompt = json!({"hook_event_name": "UserPromptSubmit", "session_id": "s1", "cwd": fx.root.to_string_lossy(), "prompt": "next"});
    run_hook("prompt-context", &prompt, &fx.root, false);

    assert_nudge(&read_code(&fx, "s1"));
}

#[test]
fn a_herded_worker_is_never_nudged() {
    let fx = fixture();
    let payload = pre_tool(&fx, "w1", "Read", json!({"file_path": "packages/bee-rs/src/lib.rs"}));
    let out = run_hook("write-guard", &payload, &fx.root, true);
    assert_eq!(out.status.code(), Some(0));
    assert_silent(&String::from_utf8_lossy(&out.stdout));
}

#[test]
fn a_native_subagent_payload_is_never_nudged() {
    let fx = fixture();
    for (i, key) in ["agent_id", "agent_type"].iter().enumerate() {
        let mut payload = pre_tool(&fx, &format!("a{i}"), "Read", json!({"file_path": "packages/bee-rs/src/lib.rs"}));
        payload[*key] = json!("worker-7");
        assert_silent(&guard(&fx, &payload));
    }
}

#[test]
fn an_empty_agent_id_does_not_count_as_a_subagent() {
    let fx = fixture();
    let mut payload = pre_tool(&fx, "e1", "Read", json!({"file_path": "packages/bee-rs/src/lib.rs"}));
    payload["agent_id"] = json!("");
    assert_nudge(&guard(&fx, &payload));
}

#[test]
fn the_nudge_never_denies() {
    let fx = fixture();
    let out = run_hook("write-guard", &pre_tool(&fx, "s1", "Read", json!({"file_path": "packages/x/main.rs"})), &fx.root, false);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty(), "{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn the_rule_text_has_one_home_in_the_leader_doctrine() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let leader = std::fs::read_to_string(repo.join("packages/bee/LEADER.md")).unwrap();
    let marker = format!("<!-- rule: {RULE_ID} -->");
    assert_eq!(leader.matches(&marker).count(), 1, "one rule block in packages/bee/LEADER.md");
    assert_eq!(leader.matches("## Start a session").count(), 1, "one Start a session heading");
}
