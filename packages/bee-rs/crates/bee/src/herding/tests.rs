use super::*;
use std::collections::BTreeMap;
use tempfile::tempdir;

#[test]
fn child_env_construction_excludes_leader_session_and_sets_worker_marker() {
    let ambient = vec![
        ("PATH".to_string(), "/usr/bin:/bin".to_string()),
        ("HOME".to_string(), "/home/user".to_string()),
        ("BEE_SESSION_ID".to_string(), "leader-sess-1".to_string()),
        ("CLAUDE_CODE_SESSION_ID".to_string(), "leader-claude-2".to_string()),
        ("PI_SESSION_ID".to_string(), "leader-pi-3".to_string()),
    ];

    let mut agent_env = BTreeMap::new();
    agent_env.insert("CUSTOM_AGENT_VAR".to_string(), "custom_val".to_string());

    let mut passthrough = BTreeMap::new();
    passthrough.insert("BEE_RELEASE_VERSION".to_string(), "1.0.0".to_string());
    passthrough.insert("PI_SESSION_ID".to_string(), "attempted-leak".to_string());

    let env = build_child_env(ambient, &agent_env, &passthrough, "job-test-123");

    // Ambient preserved
    assert_eq!(env.get("PATH").map(String::as_str), Some("/usr/bin:/bin"));
    assert_eq!(env.get("HOME").map(String::as_str), Some("/home/user"));

    // Leader session IDs excluded
    assert_eq!(env.get("BEE_SESSION_ID"), None);
    assert_eq!(env.get("CLAUDE_CODE_SESSION_ID"), None);
    assert_eq!(env.get("PI_SESSION_ID"), None);

    // Agent env and passthrough merged
    assert_eq!(env.get("CUSTOM_AGENT_VAR").map(String::as_str), Some("custom_val"));
    assert_eq!(env.get("BEE_RELEASE_VERSION").map(String::as_str), Some("1.0.0"));

    // Worker marker and job id present
    assert_eq!(env.get("BEE_HERDING_WORKER").map(String::as_str), Some("1"));
    assert_eq!(env.get("BEE_HERDING_JOB_ID").map(String::as_str), Some("job-test-123"));
}

#[test]
fn child_argv_construction_matches_pi_subagent_pattern() {
    let agent_args = vec![
        "-a".to_string(),
        "--model".to_string(),
        "openai-codex/gpt-5.6-luna:high".to_string(),
        "--thinking".to_string(),
        "high".to_string(),
        "--tools".to_string(),
        "read,bash".to_string(),
    ];

    let argv = build_child_argv(&agent_args, "do the thing");

    assert_eq!(
        argv,
        vec![
            "--mode",
            "json",
            "-p",
            "--no-session",
            "--model",
            "openai-codex/gpt-5.6-luna:high",
            "--thinking",
            "high",
            "--tools",
            "read,bash",
            "do the thing"
        ]
    );
}

#[test]
fn child_argv_construction_omits_absent_options() {
    let agent_args = vec!["-a".to_string()];
    let argv = build_child_argv(&agent_args, "minimal task");

    assert_eq!(
        argv,
        vec!["--mode", "json", "-p", "--no-session", "minimal task"]
    );
}

#[test]
fn noise_tolerant_parsing_skips_banners_and_extracts_assistant_text_and_cost() {
    let stdout = r#"mise ~/.config/mise/config.toml tools: pi@0.85.1
{"type":"session","version":3,"id":"01a0b519-dfae-754f-9de0-d5aeb8b634ee","timestamp":"2026-09-18T15:19:26.895Z","cwd":"/home/user/project"}
{"type":"agent_start"}
{"type":"turn_start"}
not a json line at all!
{"type":"message_update","usage":{"input":0,"output":0},"assistantMessageEvent":{"type":"text_delta","contentIndex":1,"delta":"Yes.\n\n"}}
{"type":"message_end","message":{"role":"assistant","content":[{"type":"thinking","thinking":"checking context"},{"type":"text","text":"Yes.\n\nPhase: `idle` | Mode: `none`  \nFeature: `pi-native-stage-driver`  \nGates: `none pending (no active work)`"}],"usage":{"input":13924,"output":114,"cacheRead":0,"cacheWrite":0,"reasoning":76,"totalTokens":14038,"cost":{"input":0.001233749944,"output":0.000020202168,"cacheRead":0,"cacheWrite":0,"total":0.001253952112}},"stopReason":"stop"}}
{"type":"turn_end","message":{"role":"assistant","content":[{"type":"text","text":"Yes.\n\nPhase: `idle` | Mode: `none`  \nFeature: `pi-native-stage-driver`  \nGates: `none pending (no active work)`"}]}}
{"type":"agent_settled"}
"#;

    let output = parse_child_jsonl(stdout);
    assert_eq!(
        output.assistant_text,
        "Yes.\n\nPhase: `idle` | Mode: `none`  \nFeature: `pi-native-stage-driver`  \nGates: `none pending (no active work)`"
    );

    let usage = output.usage.expect("usage should be parsed");
    assert_eq!(usage.get("totalTokens").and_then(Value::as_u64), Some(14038));
    let cost = usage.get("cost").and_then(|c| c.get("total")).and_then(Value::as_f64);
    assert!((cost.unwrap() - 0.001253952112).abs() < 1e-9);
}

#[test]
fn noise_tolerant_parsing_falls_back_to_text_deltas_when_no_message_end() {
    let stdout = r#"banner noise
{"type":"message_update","assistantMessageEvent":{"type":"text_delta","delta":"Hello "}}
invalid json {}{
{"type":"message_update","assistantMessageEvent":{"type":"text_delta","delta":"world!"}}
trailing noise
"#;

    let output = parse_child_jsonl(stdout);
    assert_eq!(output.assistant_text, "Hello world!");
    assert!(output.usage.is_none());
}

#[test]
fn non_zero_exit_surfaces_stderr_and_fails() {
    let tmp = tempdir().unwrap();
    let bee_dir = tmp.path().join(".bee");
    std::fs::create_dir_all(&bee_dir).unwrap();

    let cfg = serde_json::json!({
        "herding": {
            "agents": {
                "failing-agent": [
                    "sh",
                    "-c",
                    "echo 'catastrophic failure details' >&2; exit 7"
                ]
            }
        }
    });
    std::fs::write(bee_dir.join("config.json"), cfg.to_string()).unwrap();

    let opts = Options {
        task: "do failing task".to_string(),
        cwd: tmp.path().to_path_buf(),
        job_id: "job-fail-test".to_string(),
        idle_timeout_secs: 60,
        ceiling_secs: 30,
        close_always: false,
        main_root: tmp.path().to_path_buf(),
        json: true,
        dry_run: false,
        is_continue: false,
        ready_wait_secs: 60,
        agent: Some("failing-agent".to_string()),
        expertise: vec![],
        has_explicit_expertise: false,
        caller_is_worker: false,
        nickname: "worker-test".to_string(),
        cell_id: Some("cell-1".to_string()),
        seat: Some("hat-test".to_string()),
        inbox_session: None,
        pane_env_passthrough: BTreeMap::new(),
        no_pane: true,
    };

    let result = execute_no_pane(&opts);
    match &result.outcome {
        RunOutcome::SpawnFailed(err) => {
            assert!(
                err.contains("catastrophic failure details"),
                "stderr not surfaced in error: {err}"
            );
        }
        other => panic!("expected RunOutcome::SpawnFailed, got: {other:?}"),
    }

    assert_eq!(exit_code_for(&result.outcome), ExitCode::FAILURE);

    // No success result written
    let res_path = mailbox::result_path(&bee_dir, "job-fail-test", 1);
    assert!(!res_path.exists(), "result file should not exist on non-zero exit");
}

#[test]
fn no_pane_success_writes_report_and_result_files() {
    let tmp = tempdir().unwrap();
    let bee_dir = tmp.path().join(".bee");
    std::fs::create_dir_all(&bee_dir).unwrap();

    let cfg = serde_json::json!({
        "herding": {
            "agents": {
                "mock-pi": [
                    "sh",
                    "-c",
                    "echo 'startup notice'; echo '{\"type\":\"message_end\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"Task outcome succeeded.\"}],\"usage\":{\"totalTokens\":50,\"cost\":{\"total\":0.005}}}}'"
                ]
            }
        }
    });
    std::fs::write(bee_dir.join("config.json"), cfg.to_string()).unwrap();

    let opts = Options {
        task: "do success task".to_string(),
        cwd: tmp.path().to_path_buf(),
        job_id: "job-success-test".to_string(),
        idle_timeout_secs: 60,
        ceiling_secs: 30,
        close_always: false,
        main_root: tmp.path().to_path_buf(),
        json: true,
        dry_run: false,
        is_continue: false,
        ready_wait_secs: 60,
        agent: Some("mock-pi".to_string()),
        expertise: vec![],
        has_explicit_expertise: false,
        caller_is_worker: false,
        nickname: "worker-test".to_string(),
        cell_id: Some("cell-1".to_string()),
        seat: Some("hat-test".to_string()),
        inbox_session: None,
        pane_env_passthrough: BTreeMap::new(),
        no_pane: true,
    };

    let result = execute_no_pane(&opts);
    match &result.outcome {
        RunOutcome::Result(r) => {
            assert_eq!(r.status, MailboxStatus::Done);
            assert_eq!(r.summary, "Task outcome succeeded.");
            assert!(r.proof.contains("tokens: 50"));
            let rep_path = r.report_path.as_ref().expect("report_path must be set");
            let rep_text = std::fs::read_to_string(rep_path).expect("report file must exist");
            assert_eq!(rep_text, "Task outcome succeeded.");
        }
        other => panic!("expected RunOutcome::Result, got: {other:?}"),
    }

    assert_eq!(exit_code_for(&result.outcome), ExitCode::SUCCESS);
}

#[test]
fn parse_options_handles_no_pane_and_runner_flags() {
    let opts1 = parse_options(&["--task", "test", "--no-pane"]).unwrap();
    assert!(opts1.no_pane);

    let opts2 = parse_options(&["--task", "test", "--runner", "no-pane"]).unwrap();
    assert!(opts2.no_pane);

    let opts3 = parse_options(&["--task", "test"]).unwrap();
    assert!(!opts3.no_pane);
}
