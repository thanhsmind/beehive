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

fn run(dir: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(binary())
        .args(args)
        .env_remove("BEE_HERDING_WORKER")
        .env_remove("PI_SESSION_ID")
        .env_remove("PASEO_AGENT_ID")
        .env_remove("PASEO_AGENT_CWD")
        .env_remove("PASEO_CLI")
        .current_dir(dir)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)),
    )
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git").args(args).current_dir(dir).output().unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn write(dir: &Path, rel: &str, body: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

fn commit(dir: &Path, rel: &str, body: &str) {
    write(dir, rel, body);
    git(dir, &["add", rel]);
    git(dir, &["commit", "-q", "-m", &format!("change {rel}")]);
}

struct Fixture {
    _tmp: tempfile::TempDir,
    main: PathBuf,
    wt: PathBuf,
    marker: PathBuf,
    red_flag: PathBuf,
}

const ID: &str = "repo--wt--demo";

fn fixture(test_command: Option<&str>) -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().canonicalize().unwrap();
    let main = base.join("repo");
    std::fs::create_dir_all(&main).unwrap();
    let marker = base.join("suite-ran");
    let red_flag = base.join("suite-red");
    git(&main, &["init", "-q", "-b", "main"]);
    git(&main, &["config", "user.email", "t@example.com"]);
    git(&main, &["config", "user.name", "t"]);
    let command = test_command.map(str::to_string).unwrap_or_else(|| {
        format!("test ! -e {} && touch {}", red_flag.display(), marker.display())
    });
    write(&main, ".bee/onboarding.json", r#"{"version":1,"completed":true}"#);
    write(
        &main,
        ".bee/config.json",
        &serde_json::json!({"commands": {"test": command}, "uat_stop": "off"}).to_string(),
    );
    write(&main, ".gitignore", ".bee/logs/\n.bee/runtime/\n.bee/cache/\n.bee/locks/\n.bee/cells/\n.bee/state.json\n");
    write(&main, "src/a.rs", "a\n");
    git(&main, &["add", "-A"]);
    git(&main, &["commit", "-q", "-m", "init"]);
    let (code, text) = run(&main, &["worktree", "new", "--feature", "demo"]);
    assert_eq!(code, 0, "{text}");
    let wt = base.join(ID);
    assert!(wt.is_dir(), "worktree missing: {text}");
    Fixture { _tmp: tmp, main, wt, marker, red_flag }
}

fn merge(f: &Fixture, extra: &[&str]) -> (i32, String) {
    let mut args = vec!["worktree", "merge", "--id", ID, "--detached", "--no-cleanup"];
    args.extend_from_slice(extra);
    run(&f.main, &args)
}

fn record(f: &Fixture) -> Value {
    serde_json::from_str(&std::fs::read_to_string(f.wt.join(".bee/logs/test-results.json")).unwrap()).unwrap()
}

#[test]
fn bee_test_stamps_the_checkout_head_into_its_record() {
    let f = fixture(None);
    commit(&f.wt, "src/b.rs", "b\n");
    let (code, text) = run(&f.wt, &["test"]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(record(&f)["head"], git(&f.wt, &["rev-parse", "HEAD"]));
}

#[test]
fn a_code_feature_with_no_test_record_is_refused_naming_bee_test_in_the_worktree() {
    let f = fixture(None);
    commit(&f.wt, "src/b.rs", "b\n");
    let main_head = git(&f.main, &["rev-parse", "HEAD"]);
    let (code, text) = merge(&f, &[]);
    assert_ne!(code, 0, "{text}");
    assert!(text.contains("WORKTREE_MERGE_FULL_SUITE_MISSING"), "{text}");
    assert!(text.contains(&format!("FIX: run bee test in {}", f.wt.display())), "{text}");
    assert!(text.contains("--full-suite-waiver"), "{text}");
    assert_eq!(git(&f.main, &["rev-parse", "HEAD"]), main_head, "main must stay untouched");
}

#[test]
fn a_red_record_is_refused() {
    let f = fixture(None);
    commit(&f.wt, "src/b.rs", "b\n");
    std::fs::write(&f.red_flag, "x").unwrap();
    let (code, _) = run(&f.wt, &["test"]);
    assert_eq!(code, 1);
    let (code, text) = merge(&f, &[]);
    assert_ne!(code, 0, "{text}");
    assert!(text.contains("WORKTREE_MERGE_FULL_SUITE_RED"), "{text}");
}

#[test]
fn a_record_older_than_a_later_code_commit_is_refused() {
    let f = fixture(None);
    commit(&f.wt, "src/b.rs", "b\n");
    assert_eq!(run(&f.wt, &["test"]).0, 0);
    commit(&f.wt, "src/c.rs", "c\n");
    let (code, text) = merge(&f, &[]);
    assert_ne!(code, 0, "{text}");
    assert!(text.contains("WORKTREE_MERGE_FULL_SUITE_STALE"), "{text}");
}

#[test]
fn a_later_docs_only_commit_keeps_the_record_fresh() {
    let f = fixture(None);
    commit(&f.wt, "src/b.rs", "b\n");
    assert_eq!(run(&f.wt, &["test"]).0, 0);
    commit(&f.wt, "docs/history/demo/notes.md", "notes\n");
    commit(&f.wt, "README.md", "readme\n");
    let (code, text) = merge(&f, &[]);
    assert_eq!(code, 0, "{text}");
}

#[test]
fn a_fresh_green_record_merges_and_the_door_runs_no_test() {
    let f = fixture(None);
    commit(&f.wt, "src/b.rs", "b\n");
    assert_eq!(run(&f.wt, &["test"]).0, 0);
    assert!(f.marker.exists(), "bee test itself ran the suite");
    std::fs::remove_file(&f.marker).unwrap();
    let (code, text) = merge(&f, &[]);
    assert_eq!(code, 0, "{text}");
    assert!(!f.marker.exists(), "the merge door must not run the suite: {text}");
    assert!(f.main.join("src/b.rs").is_file());
}

#[test]
fn the_waiver_merges_and_writes_a_merge_audit_row() {
    let f = fixture(None);
    commit(&f.wt, "src/b.rs", "b\n");
    let (code, text) = merge(&f, &["--full-suite-waiver", "owner accepted the risk"]);
    assert_eq!(code, 0, "{text}");
    assert!(f.main.join("src/b.rs").is_file());
    let audit = std::fs::read_to_string(f.main.join(".bee/audit.jsonl")).unwrap();
    let row = audit
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|r| r["kind"] == "merge")
        .unwrap_or_else(|| panic!("no merge audit row in {audit}"));
    assert_eq!(row["feature"], "demo");
    let note = row["note"].as_str().unwrap();
    assert!(note.starts_with("full-suite waiver:"), "{note}");
    assert!(note.contains("owner accepted the risk"), "{note}");
}

#[test]
fn an_empty_waiver_reason_is_refused() {
    let f = fixture(None);
    commit(&f.wt, "src/b.rs", "b\n");
    let (code, text) = merge(&f, &["--full-suite-waiver", " "]);
    assert_ne!(code, 0, "{text}");
    assert!(!f.main.join("src/b.rs").exists(), "{text}");
}

#[test]
fn a_docs_only_feature_merges_with_no_record() {
    let f = fixture(None);
    commit(&f.wt, "docs/history/demo/CONTEXT.md", "context\n");
    commit(&f.wt, "notes.md", "notes\n");
    let (code, text) = merge(&f, &[]);
    assert_eq!(code, 0, "{text}");
    assert!(f.main.join("notes.md").is_file());
}

#[test]
fn a_repo_with_no_declared_test_merges_code_with_no_record() {
    let f = fixture(Some("none"));
    commit(&f.wt, "src/b.rs", "b\n");
    let (code, text) = merge(&f, &[]);
    assert_eq!(code, 0, "{text}");
    assert!(f.main.join("src/b.rs").is_file());
}
