use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn scratch_repo(base: &Path, name: &str) -> PathBuf {
    let dir = base.join(name);
    std::fs::create_dir_all(dir.join(".bee").join("logs")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    std::fs::write(dir.join(".bee/config.json"), r#"{"commands":{"test":"none"}}"#).unwrap();
    std::fs::write(
        dir.join(".bee/state.json"),
        r#"{"phase":"planning","feature":"demo-feature","gates":{}}"#,
    )
    .unwrap();
    dir
}

fn run(repo: &Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("bee")
        .unwrap()
        .args(args)
        .env_remove("BEE_HERDING_WORKER")
        .current_dir(repo)
        .output()
        .unwrap()
}

fn bee(repo: &Path, args: &[&str]) -> (bool, String) {
    let out = run(repo, args);
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), text)
}

fn bee_json(repo: &Path, args: &[&str]) -> Value {
    let mut full: Vec<&str> = args.to_vec();
    full.push("--json");
    let text = String::from_utf8_lossy(&run(repo, &full).stdout).into_owned();
    serde_json::from_str(text.trim()).unwrap_or_else(|e| panic!("{e}: {text}"))
}

fn audit_rows(repo: &Path) -> Vec<Value> {
    std::fs::read_to_string(repo.join(".bee/audit.jsonl"))
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn legacy_decision(id: &str, text: &str, tag: &str) -> String {
    format!(
        r#"{{"id":"{id}","type":"decide","date":"2026-11-01T00:00:00.000Z","decision":"{text}","rationale":"r","alternatives":null,"scope":"repo","source":"user","confidence":null,"tags":["{tag}"],"relation":"none"}}"#
    )
}

fn seed_legacy_rows(repo: &Path) -> String {
    let body = format!(
        "{}\n{}\n",
        legacy_decision("11111111-0000-4000-8000-000000000001", "alpha gate approval audit line", "gate-bypass"),
        legacy_decision("22222222-0000-4000-8000-000000000002", "alpha product rule", "rules"),
    );
    std::fs::write(repo.join(".bee/decisions.jsonl"), &body).unwrap();
    body
}

#[test]
fn audit_log_appends_a_row_and_audit_list_returns_it() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "round-trip");
    let logged = bee_json(
        &repo,
        &["audit", "log", "--kind", "merge", "--feature", "f1", "--note", "merged f1", "--cell", "c-1"],
    );
    assert!(logged.to_string().contains("merged f1"), "{logged}");

    let rows = audit_rows(&repo);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["kind"], "merge");
    assert_eq!(rows[0]["feature"], "f1");
    assert_eq!(rows[0]["cell"], "c-1");
    assert_eq!(rows[0]["note"], "merged f1");
    assert!(rows[0]["id"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(rows[0]["ts"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(rows[0]["source"].as_str().is_some_and(|s| !s.is_empty()));

    let listed = bee_json(&repo, &["audit", "list", "--kind", "merge", "--feature", "f1"]);
    let entries = listed["entries"].as_array().expect("entries array");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["note"], "merged f1");

    let other = bee_json(&repo, &["audit", "list", "--kind", "release"]);
    assert_eq!(other["entries"].as_array().unwrap().len(), 0);
}

#[test]
fn audit_log_defaults_the_feature_to_the_workflow_feature() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "default-feature");
    let (ok, text) = bee(&repo, &["audit", "log", "--kind", "close", "--note", "closed"]);
    assert!(ok, "{text}");
    assert_eq!(audit_rows(&repo)[0]["feature"], "demo-feature");
}

#[test]
fn audit_log_refuses_an_unknown_kind_and_lists_the_set() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "bad-kind");
    let (ok, text) = bee(
        &repo,
        &["audit", "log", "--kind", "vibes", "--feature", "f1", "--note", "n"],
    );
    assert!(!ok, "{text}");
    assert!(text.contains("vibes"), "{text}");
    for kind in ["gate-bypass", "impact-deferral", "handoff", "leader-check-deferral"] {
        assert!(text.contains(kind), "the refusal must list {kind}: {text}");
    }
    assert!(audit_rows(&repo).is_empty(), "a refusal writes nothing");
}

#[test]
fn decisions_log_refuses_audit_only_tags_and_names_bee_audit_log() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "refuse-tags");
    for tag in ["impact-deferral", "gate-bypass"] {
        let (ok, text) = bee(
            &repo,
            &[
                "decisions", "log", "--decision", "an audit line", "--rationale", "why",
                "--relation", "none", "--tags", &format!("rules,{tag}"),
            ],
        );
        assert!(!ok, "{tag} must be refused: {text}");
        assert!(text.contains(&format!("bee audit log --kind {tag}")), "{text}");
    }
    assert!(
        !repo.join(".bee/decisions.jsonl").exists()
            || std::fs::read_to_string(repo.join(".bee/decisions.jsonl")).unwrap().trim().is_empty(),
        "a refused log writes no decision"
    );

    for tag in ["release", "merge"] {
        let (ok, text) = bee(
            &repo,
            &[
                "decisions", "log", "--decision", "a real rule", "--rationale", "why",
                "--relation", "none", "--tags", tag,
            ],
        );
        assert!(ok, "{tag} stays a legal decision tag: {text}");
    }
}

#[test]
fn decisions_search_hides_legacy_audit_rows_unless_the_tag_asks_for_them() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "hide-legacy");
    let before = seed_legacy_rows(&repo);

    let hidden = bee_json(&repo, &["decisions", "search", "--text", "alpha"]);
    let texts: Vec<String> = hidden["decisions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["decision"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(texts, vec!["alpha product rule".to_string()], "{hidden}");

    let active = bee_json(&repo, &["decisions", "active"]);
    assert_eq!(active["decisions"].as_array().unwrap().len(), 1, "{active}");

    let asked = bee_json(&repo, &["decisions", "search", "--tag", "gate-bypass"]);
    assert_eq!(asked["decisions"].as_array().unwrap().len(), 1, "{asked}");
    assert_eq!(asked["decisions"][0]["decision"], "alpha gate approval audit line");

    assert_eq!(
        std::fs::read_to_string(repo.join(".bee/decisions.jsonl")).unwrap(),
        before,
        "decisions.jsonl is never rewritten"
    );
}

#[test]
fn status_recent_decisions_hide_legacy_audit_rows() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "hide-status");
    seed_legacy_rows(&repo);
    let status = bee_json(&repo, &["status"]);
    let recent = status["recent_decisions"].as_array().expect("recent_decisions");
    assert_eq!(recent.len(), 1, "{status}");
    assert!(recent[0]["decision"].to_string().contains("alpha product rule"), "{status}");
}

#[test]
fn gate_actor_auto_writes_one_gate_bypass_audit_row_and_no_decision() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = scratch_repo(tmp.path(), "gate-auto");
    let (ok, text) = bee(
        &repo,
        &[
            "gate", "--no-lane", "--name", "context", "--approved", "true", "--actor", "auto",
            "--bypass-level", "normal", "--reason", "gate_bypass=normal",
        ],
    );
    assert!(ok, "{text}");
    let rows = audit_rows(&repo);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["kind"], "gate-bypass");
    assert_eq!(rows[0]["feature"], "demo-feature");
    assert_eq!(rows[0]["bypass_level"], "normal");
    let note = rows[0]["note"].as_str().unwrap();
    assert!(note.contains("context") && note.contains("gate_bypass=normal"), "{note}");

    let decisions = std::fs::read_to_string(repo.join(".bee/decisions.jsonl")).unwrap_or_default();
    assert!(decisions.trim().is_empty(), "no decision is written: {decisions}");

    let (ok, text) = bee(&repo, &["gate", "--no-lane", "--name", "review", "--approved", "true"]);
    assert!(ok, "{text}");
    assert_eq!(audit_rows(&repo).len(), 1, "a user approval writes no audit row");
}
