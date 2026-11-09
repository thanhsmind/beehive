use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

fn git(root: &Path, args: &[&str]) {
    let out = Command::new("git").args(args).current_dir(root).output().unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

fn put(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

fn fixture(holdout_percent: u64) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = dunce::canonicalize(dir.path()).unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.email", "test@example.com"]);
    git(&root, &["config", "user.name", "test"]);
    git(&root, &["config", "commit.gpgsign", "false"]);
    std::fs::create_dir_all(root.join(".bee").join("cells")).unwrap();
    std::fs::create_dir_all(root.join(".bee").join("logs")).unwrap();
    put(&root, ".bee/onboarding.json", "{}\n");
    put(
        &root,
        ".bee/config.json",
        &format!("{{\"knowledge_holdout_percent\":{holdout_percent},\"commands\":{{\"test\":\"echo ok\"}},\"uat_stop\":\"off\"}}\n"),
    );
    put(&root, "src/lib.rs", "pub fn one() {}\n");
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "init"]);
    Fixture { _dir: dir, root }
}

fn bee_cmd(fx: &Fixture, args: &[&str]) -> (i32, String, String) {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("bee"));
    cmd.args(args)
        .current_dir(&fx.root)
        .env("BEE_HOOK_NO_DELEGATE", "1")
        .env_remove("BEE_HERDING_WORKER")
        .env_remove("BEE_HERDING_JOB_ID")
        .env_remove("BEE_SESSION_ID")
        .env_remove("CLAUDE_CODE_SESSION_ID")
        .env_remove("PI_SESSION_ID");
    let out = cmd.output().unwrap();
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

fn run_bee(fx: &Fixture, args: &[&str]) -> String {
    let (code, stdout, stderr) = bee_cmd(fx, args);
    assert_eq!(code, 0, "bee {:?} failed (code {code}):\nstdout: {stdout}\nstderr: {stderr}", args);
    stdout
}

fn list_tree(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path.strip_prefix(root).unwrap().to_string_lossy().into_owned());
            }
        }
    }
    files.sort();
    files
}

#[test]
fn flow_end_to_end_with_pack_cap_consolidate_close_and_report() {
    let fx = fixture(0);
    put(
        &fx.root,
        "docs/knowledge/areas/demo/one.md",
        "---\ntype: bee.area\ntitle: one area\ndescription: the one area\nbee:\n  id: one\n  lifecycle: active\n  areas: [one]\n  anchors: [src/lib.rs]\n---\nAnchored by hand.\n",
    );
    put(
        &fx.root,
        "docs/knowledge/tasks/demo/item.md",
        "---\ntype: bee.work-item\ntitle: demo item\ndescription: the demo work item\nbee:\n  id: demo\n  lifecycle: active\n  required_context: [areas/demo/one.md]\n---\nBody.\n",
    );
    put(&fx.root, "docs/history/demo/CONTEXT.md", "# Demo — Context\n\nBody.\n");

    let shaping_out = run_bee(&fx, &["knowledge", "pack", "--feature", "demo", "--stage", "shaping"]);
    assert!(shaping_out.contains("docs/knowledge/areas/demo/one.md"), "{shaping_out}");
    assert!(shaping_out.contains("src/lib.rs"), "{shaping_out}");

    let planning_out = run_bee(&fx, &["knowledge", "pack", "--feature", "demo", "--stage", "planning"]);
    assert!(planning_out.contains("docs/knowledge/areas/demo/one.md"), "{planning_out}");
    assert!(planning_out.contains("src/lib.rs"), "{planning_out}");

    let log_path = fx.root.join(".bee/logs/dispatch.jsonl");
    let text = std::fs::read_to_string(&log_path).unwrap();
    let rows: Vec<Value> = text
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["source"] == "pack" && v["kind"] == "pack")
        .collect();
    assert_eq!(rows.len(), 2, "{text}");
    assert_eq!(rows[0]["stage"], "shaping");
    assert_eq!(rows[0]["feature"], "demo");
    assert_eq!(rows[0]["knowledge_arm"], "knowledge");
    assert_eq!(rows[1]["stage"], "planning");
    assert_eq!(rows[1]["feature"], "demo");
    assert_eq!(rows[1]["knowledge_arm"], "knowledge");

    let cell = json!({
        "id": "c-1",
        "feature": "demo",
        "title": "demo cell",
        "lane": "small",
        "role": "code",
        "status": "claimed",
        "verify": "cargo test"
    });
    put(&fx.root, ".bee/cells/c-1.json", &serde_json::to_string_pretty(&cell).unwrap());

    let prepare_line = json!({
        "ts": "2026-11-09T00:00:00.000Z",
        "source": "prepare",
        "kind": "cell",
        "cell": "c-1",
        "knowledge_arm": "knowledge"
    });
    let mut file = std::fs::OpenOptions::new().append(true).open(&log_path).unwrap();
    writeln!(file, "{}", prepare_line).unwrap();

    let report_payload = json!({
        "outcome": "done",
        "commit": "none",
        "files": ["src/lib.rs", "docs/knowledge/areas/demo/one.md"],
        "tests": "cargo test — green:unit — ok",
        "deviations": []
    });
    let cap_out = run_bee(
        &fx,
        &[
            "cells",
            "cap",
            "--id",
            "c-1",
            "--outcome",
            "done",
            "--files",
            "src/lib.rs,docs/knowledge/areas/demo/one.md",
            "--report",
            &report_payload.to_string(),
            "--no-mistakes",
            "--inline-reason",
            "flow test",
        ],
    );
    assert!(cap_out.contains("knowledge delta: 1 file(s)"), "{cap_out}");

    let before_tree = list_tree(&fx.root);
    let consolidate_out = run_bee(&fx, &["knowledge", "consolidate", "--feature", "demo"]);
    assert!(consolidate_out.contains("docs/knowledge/areas/demo/one.md"), "{consolidate_out}");
    assert_eq!(list_tree(&fx.root), before_tree);

    std::thread::sleep(std::time::Duration::from_millis(15));
    put(&fx.root, "docs/history/demo/delivery.md", "# Delivery\n\n- docs/knowledge/areas/demo/one.md: updated.\n");
    run_bee(&fx, &["audit", "log", "--kind", "leader-check-deferral", "--feature", "demo", "--note", "flow test"]);
    let close_out = run_bee(&fx, &["close", "--feature", "demo"]);
    assert!(close_out.contains("Knowledge consolidated"), "{close_out}");
    assert!(!fx.root.join("docs/history/demo/promote-proposals.md").exists());

    let report_stdout = run_bee(&fx, &["knowledge", "report", "--json"]);
    let report_json: Value = serde_json::from_str(&report_stdout).unwrap();
    let cell_arm_count = report_json["outcomes"]["arms"]["knowledge"]["cells"].as_u64().unwrap();
    assert_eq!(cell_arm_count, 1, "{report_json}");
    let feature_arm_count = report_json["feature_outcomes"]["arms"]["knowledge"]["features"].as_u64().unwrap();
    assert_eq!(feature_arm_count, 1, "{report_json}");
}

#[test]
fn holdout_feature_with_percent_100_gets_empty_pack_and_counted_in_report() {
    let fx = fixture(100);
    put(
        &fx.root,
        "docs/knowledge/areas/holdout-feat/one.md",
        "---\ntype: bee.area\ntitle: holdout area\ndescription: holdout area\nbee:\n  id: holdout-area\n  lifecycle: active\n  areas: [holdout-area]\n  anchors: [src/lib.rs]\n---\nAnchored.\n",
    );
    put(
        &fx.root,
        "docs/knowledge/tasks/holdout-feat/item.md",
        "---\ntype: bee.work-item\ntitle: holdout item\ndescription: holdout work item\nbee:\n  id: holdout-feat\n  lifecycle: active\n  required_context: [areas/holdout-feat/one.md]\n---\nBody.\n",
    );
    put(&fx.root, "docs/history/holdout-feat/CONTEXT.md", "# Holdout — Context\n\nBody.\n");

    let pack_out = run_bee(&fx, &["knowledge", "pack", "--feature", "holdout-feat", "--stage", "shaping"]);
    assert!(pack_out.contains("holdout: this feature loads no pack"), "{pack_out}");
    assert!(!pack_out.contains("docs/knowledge/areas/holdout-feat/one.md"), "{pack_out}");
    assert!(!pack_out.contains("- code:"), "{pack_out}");

    let report_stdout = run_bee(&fx, &["knowledge", "report", "--json"]);
    let report_json: Value = serde_json::from_str(&report_stdout).unwrap();
    let holdout_features = report_json["feature_outcomes"]["arms"]["holdout"]["features"].as_u64().unwrap();
    assert_eq!(holdout_features, 1, "{report_json}");
}
