use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn binary() -> PathBuf {
    let mut dir = std::env::current_exe().expect("test binary path");
    dir.pop();
    if dir.ends_with("deps") {
        dir.pop();
    }
    let bin = dir.join(format!("bee{}", std::env::consts::EXE_SUFFIX));
    assert!(bin.is_file(), "built bee binary not found at {}", bin.display());
    bin
}

fn run(repo: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(binary())
        .args(args)
        .env_remove("BEE_HERDING_WORKER")
        .env_remove("PI_SESSION_ID")
        .current_dir(repo)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)),
    )
}

fn write(repo: &Path, rel: &str, body: &str) {
    let path = repo.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

fn door_repo(base: &Path, name: &str) -> PathBuf {
    let dir = base.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    assert!(Command::new("git").args(["init", "-q"]).current_dir(&dir).status().unwrap().success());
    write(&dir, ".bee/onboarding.json", r#"{"version":1,"completed":true}"#);
    write(&dir, ".bee/config.json", r#"{"commands":{"test":"none"},"uat_stop":"close"}"#);
    write(&dir, ".bee/state.json", r#"{"phase":"executing","feature":"demo","gates":{}}"#);
    write(&dir, ".bee/lanes/demo.json", r#"{"feature":"demo","mode":"standard"}"#);
    write(
        &dir,
        ".bee/cells/demo-1.json",
        r#"{"id":"demo-1","feature":"demo","status":"capped","title":"t","behavior_change":true,"files":["src/a.rs"],"trace":{"worker":"w","behavior_change":true,"capped_at":"2026-11-01T00:00:00.000Z","files_changed":["src/a.rs"],"no_mistakes":true,"dissent":[{"severity":"consider","claim":"c","alternative":"a"}]}}"#,
    );
    dir
}

fn doors(repo: &Path) -> Vec<Value> {
    let (_, text) = run(repo, &["close", "--feature", "demo", "--dry-run", "--json"]);
    let start = text.find('{').unwrap_or_else(|| panic!("no json: {text}"));
    let mut de = serde_json::Deserializer::from_str(&text[start..]).into_iter::<Value>();
    let v = de.next().unwrap().unwrap_or_else(|e| panic!("{e}: {text}"));
    v["doors"].as_array().expect("doors").clone()
}

fn door(repo: &Path, name: &str) -> Value {
    doors(repo).into_iter().find(|d| d["door"] == name).unwrap_or_else(|| panic!("no door {name}"))
}

fn legacy(tag: &str) -> String {
    format!(
        r#"{{"id":"d-{tag}","type":"decide","date":"2026-11-01T00:00:00.000Z","decision":"defer {tag} for demo","rationale":"r","alternatives":null,"scope":"repo","source":"user","confidence":null,"tags":["{tag}"],"relation":"none"}}"#
    ) + "\n"
}

const CLOSE_DOORS: [(&str, &str); 5] = [
    ("scribing-debt", "capture-deferral"),
    ("judge-debt", "judge-deferral"),
    ("dissent-debt", "dissent-deferral"),
    ("leader-check", "leader-check-deferral"),
    ("uat", "uat-deferral"),
];

#[test]
fn an_audit_row_of_each_deferral_kind_satisfies_its_close_door() {
    let tmp = tempfile::tempdir().unwrap();
    for (door_name, kind) in CLOSE_DOORS {
        let repo = door_repo(tmp.path(), &format!("audit-{kind}"));
        assert_eq!(door(&repo, door_name)["blocking"], true, "{door_name} starts refused");
        let (code, text) =
            run(&repo, &["audit", "log", "--kind", kind, "--feature", "demo", "--note", "deferred on purpose"]);
        assert_eq!(code, 0, "{text}");
        let after = door(&repo, door_name);
        assert_eq!(after["blocking"], false, "an audit {kind} row must satisfy {door_name}: {after}");
    }
}

#[test]
fn a_legacy_deferral_decision_row_still_satisfies_each_close_door() {
    let tmp = tempfile::tempdir().unwrap();
    for (door_name, kind) in CLOSE_DOORS {
        let repo = door_repo(tmp.path(), &format!("legacy-{kind}"));
        write(&repo, ".bee/decisions.jsonl", &legacy(kind));
        let after = door(&repo, door_name);
        assert_eq!(after["blocking"], false, "a legacy {kind} decision must satisfy {door_name}: {after}");
    }
}

#[test]
fn an_audit_row_for_another_feature_leaves_the_door_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = door_repo(tmp.path(), "other-feature");
    let (code, text) =
        run(&repo, &["audit", "log", "--kind", "judge-deferral", "--feature", "elsewhere", "--note", "n"]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(door(&repo, "judge-debt")["blocking"], true);
}

#[test]
fn no_close_door_tells_the_agent_to_log_a_deferral_decision() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = door_repo(tmp.path(), "refusal-text");
    for (door_name, kind) in CLOSE_DOORS {
        let detail = door(&repo, door_name)["detail"].as_str().unwrap().to_string();
        if door_name == "judge-debt" || door_name == "uat" {
            continue;
        }
        assert!(
            detail.contains(&format!("bee audit log --kind {kind} --feature demo --note")),
            "{door_name} detail must name bee audit log: {detail}"
        );
    }
    for kind in ["capture-deferral", "judge-deferral", "dissent-deferral", "leader-check-deferral"] {
        let (_, text) = run(&repo, &["close", "--feature", "demo"]);
        assert!(!text.contains("decision tagged"), "close refusal still names a decision tag: {text}");
        assert!(!text.contains("deferral decision"), "close refusal still names a deferral decision: {text}");
        if kind != "judge-deferral" {
            assert!(
                text.contains(&format!("bee audit log --kind {kind} --feature demo --note")),
                "close refusal must name bee audit log for {kind}: {text}"
            );
        }
        let (code, out) = run(&repo, &["audit", "log", "--kind", kind, "--feature", "demo", "--note", "n"]);
        assert_eq!(code, 0, "{out}");
    }
}
