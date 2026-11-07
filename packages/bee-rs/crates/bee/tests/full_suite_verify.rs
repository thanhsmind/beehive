use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const SUITE: &str = "cargo test --release --manifest-path Cargo.toml";
const PREFIXED_SUITE: &str = "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path Cargo.toml";

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

fn repo(base: &Path, config: &str) -> PathBuf {
    let dir = base.join("repo");
    std::fs::create_dir_all(dir.join(".bee/cells")).unwrap();
    std::fs::create_dir_all(dir.join(".bee/logs")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    std::fs::write(dir.join(".bee/config.json"), config).unwrap();
    std::fs::write(
        dir.join(".bee/state.json"),
        r#"{"phase":"executing","feature":"fso","gates":{}}"#,
    )
    .unwrap();
    dir
}

fn config_with(test: Value) -> String {
    json!({ "commands": { "test": test } }).to_string()
}

fn cell(id: &str, verify: &str) -> Value {
    json!({
        "id": id,
        "feature": "fso",
        "title": "t",
        "action": "a",
        "verify": verify,
        "lane": "small",
        "role": "code",
        "affects_skills": [],
        "affects_specs": []
    })
}

fn bee(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(binary())
        .args(args)
        .env_remove("PI_SESSION_ID")
        .current_dir(cwd)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn add(cwd: &Path, payload: &Value) -> (i32, String) {
    let file = cwd.join("payload.json");
    std::fs::write(&file, payload.to_string()).unwrap();
    bee(cwd, &["cells", "add", "--file", file.to_str().unwrap()])
}

fn update(cwd: &Path, id: &str, patch: &Value) -> (i32, String) {
    let file = cwd.join("patch.json");
    std::fs::write(&file, patch.to_string()).unwrap();
    bee(cwd, &["cells", "update", "--id", id, "--file", file.to_str().unwrap()])
}

fn stored(cwd: &Path, id: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(cwd.join(".bee/cells").join(format!("{id}.json"))).unwrap()).unwrap()
}

fn has_cell(cwd: &Path, id: &str) -> bool {
    cwd.join(".bee/cells").join(format!("{id}.json")).is_file()
}

#[test]
fn add_refuses_the_bare_declared_suite_and_names_a_scoped_form() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = repo(tmp.path(), &config_with(json!(SUITE)));
    let (code, out) = add(&dir, &cell("c1", SUITE));
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("verify runs the full suite"), "{out}");
    assert!(out.contains("c1"), "{out}");
    assert!(out.contains("FIX:"), "{out}");
    assert!(out.contains(&format!("{SUITE} --test <name>")), "{out}");
    assert!(out.contains("full_suite_reason"), "{out}");
    assert!(!has_cell(&dir, "c1"));
}

#[test]
fn add_refuses_the_path_prefixed_spelling_and_a_suite_inside_a_chain() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = repo(tmp.path(), &config_with(json!(SUITE)));
    let (code, out) = add(&dir, &cell("c1", PREFIXED_SUITE));
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("verify runs the full suite"), "{out}");

    let chained = format!("true && {PREFIXED_SUITE}");
    let (code, out) = add(&dir, &cell("c2", &chained));
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("verify runs the full suite"), "{out}");
}

#[test]
fn add_refuses_a_declared_array_entry_and_bare_bee_test() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = repo(tmp.path(), &config_with(json!(["cargo fmt --check", SUITE])));
    let (code, out) = add(&dir, &cell("c1", SUITE));
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("verify runs the full suite"), "{out}");

    for verify in ["bee test", ".bee/bin/bee test"] {
        let (code, out) = add(&dir, &cell("c2", verify));
        assert_ne!(code, 0, "{verify}: {out}");
        assert!(out.contains("verify runs the full suite"), "{verify}: {out}");
    }
}

#[test]
fn add_accepts_a_suite_command_with_a_filter_after_it() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = repo(tmp.path(), &config_with(json!(SUITE)));
    for (i, suffix) in ["--test full_suite_verify", "--lib verbs::cells", "name_filter", "-- --exact one"]
        .iter()
        .enumerate()
    {
        let id = format!("c{i}");
        let (code, out) = add(&dir, &cell(&id, &format!("{PREFIXED_SUITE} {suffix}")));
        assert_eq!(code, 0, "{suffix}: {out}");
        assert!(has_cell(&dir, &id));
    }
    let (code, out) = add(&dir, &cell("bee-scoped", "bee test --help"));
    assert_eq!(code, 0, "{out}");
}

#[test]
fn a_full_suite_reason_allows_the_bare_suite_and_stays_on_the_cell() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = repo(tmp.path(), &config_with(json!(SUITE)));
    let mut payload = cell("c1", SUITE);
    payload["full_suite_reason"] = json!("renames a shared type used by every crate");
    let (code, out) = add(&dir, &payload);
    assert_eq!(code, 0, "{out}");
    assert_eq!(stored(&dir, "c1")["full_suite_reason"], "renames a shared type used by every crate");
    let (code, shown) = bee(&dir, &["cells", "show", "--id", "c1"]);
    assert_eq!(code, 0, "{shown}");
    assert!(shown.contains("renames a shared type used by every crate"), "{shown}");

    let mut blank = cell("c2", SUITE);
    blank["full_suite_reason"] = json!("   ");
    let (code, out) = add(&dir, &blank);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("verify runs the full suite"), "{out}");
}

#[test]
fn update_refuses_a_verify_patch_to_the_bare_suite_and_leaves_the_cell() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = repo(tmp.path(), &config_with(json!(SUITE)));
    let (code, out) = add(&dir, &cell("c1", &format!("{SUITE} --lib one")));
    assert_eq!(code, 0, "{out}");

    let (code, out) = update(&dir, "c1", &json!({ "verify": SUITE }));
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("verify runs the full suite"), "{out}");
    assert!(out.contains("c1"), "{out}");
    assert!(out.contains("FIX:"), "{out}");
    assert_eq!(stored(&dir, "c1")["verify"], format!("{SUITE} --lib one"));

    let (code, out) = update(&dir, "c1", &json!({ "verify": SUITE, "full_suite_reason": "cross-cutting rename" }));
    assert_eq!(code, 0, "{out}");
    assert_eq!(stored(&dir, "c1")["verify"], SUITE);
    assert_eq!(stored(&dir, "c1")["full_suite_reason"], "cross-cutting rename");

    let (code, out) = update(&dir, "c1", &json!({ "full_suite_reason": null }));
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("verify runs the full suite"), "{out}");
    assert_eq!(stored(&dir, "c1")["full_suite_reason"], "cross-cutting rename");
}

#[test]
fn update_accepts_a_reason_patch_alone_and_a_scoped_verify() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = repo(tmp.path(), &config_with(json!(SUITE)));
    let (code, out) = add(&dir, &cell("c1", &format!("{SUITE} --lib one")));
    assert_eq!(code, 0, "{out}");
    let (code, out) = update(&dir, "c1", &json!({ "full_suite_reason": "because" }));
    assert_eq!(code, 0, "{out}");
    let (code, out) = update(&dir, "c1", &json!({ "verify": format!("{SUITE} --lib two") }));
    assert_eq!(code, 0, "{out}");
}

#[test]
fn a_repo_with_no_declared_suite_or_the_no_test_sentinel_never_refuses() {
    for config in ["{}".to_string(), config_with(json!("none"))] {
        let tmp = tempfile::tempdir().unwrap();
        let dir = repo(tmp.path(), &config);
        for (i, verify) in ["bee test", SUITE, ".bee/bin/bee test"].iter().enumerate() {
            let id = format!("c{i}");
            let (code, out) = add(&dir, &cell(&id, verify));
            assert_eq!(code, 0, "{config} {verify}: {out}");
        }
        let (code, out) = update(&dir, "c0", &json!({ "verify": "bee test" }));
        assert_eq!(code, 0, "{out}");
    }
}
