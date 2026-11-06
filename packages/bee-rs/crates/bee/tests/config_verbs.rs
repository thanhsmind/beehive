use assert_cmd::Command;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn repo(base: &Path, config: &str) -> PathBuf {
    let dir = base.join("repo");
    std::fs::create_dir_all(dir.join(".bee").join("logs")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    std::fs::write(dir.join(".bee/state.json"), r#"{"phase":"executing","gates":{}}"#).unwrap();
    std::fs::write(dir.join(".bee/config.json"), config).unwrap();
    dir
}

fn bee(repo: &Path, args: &[&str]) -> (bool, Value, String) {
    let mut full: Vec<&str> = vec!["config"];
    full.extend_from_slice(args);
    full.push("--json");
    let out = Command::cargo_bin("bee").unwrap().args(&full).current_dir(repo).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let parsed = serde_json::from_str(stdout.trim()).unwrap_or(Value::Null);
    (out.status.success(), parsed, stdout)
}

fn text(repo: &Path, args: &[&str]) -> String {
    let mut full: Vec<&str> = vec!["config"];
    full.extend_from_slice(args);
    let out = Command::cargo_bin("bee").unwrap().args(&full).current_dir(repo).output().unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn file(repo: &Path, rel: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(repo.join(rel)).unwrap()).unwrap()
}

#[test]
fn set_get_unset_round_trip_with_dotted_keys_and_parent_pruning() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"commands":{"test":"none"}}"#);

    let (ok, v, out) = bee(&r, &["set", "--key", "a.b.c", "--value", "12"]);
    assert!(ok, "{out}");
    assert_eq!(v["value"], json!(12));
    bee(&r, &["set", "--key", "a.b.d", "--value", r#"{"x":true}"#]);
    bee(&r, &["set", "--key", "a.label", "--value", "not json"]);
    assert_eq!(
        file(&r, ".bee/config.json"),
        json!({"commands":{"test":"none"},"a":{"b":{"c":12,"d":{"x":true}},"label":"not json"}})
    );

    let (_, v, _) = bee(&r, &["get", "--key", "a.b.d.x"]);
    assert_eq!(v["present"], json!(true));
    assert_eq!(v["value"], json!(true));
    assert_eq!(text(&r, &["get", "--key", "a.b.c"]).trim(), "a.b.c = 12");
    let (_, v, _) = bee(&r, &["get", "--key", "a.zzz"]);
    assert_eq!(v["present"], json!(false));
    assert_eq!(v["value"], Value::Null);
    assert_eq!(text(&r, &["get", "--key", "a.zzz"]).trim(), r#"config get: "a.zzz" is not set."#);

    let (_, v, _) = bee(&r, &["unset", "--key", "a.b.c"]);
    assert_eq!(v["removed"], json!(true));
    bee(&r, &["unset", "--key", "a.b.d.x"]);
    bee(&r, &["unset", "--key", "a.label"]);
    assert_eq!(file(&r, ".bee/config.json"), json!({"commands":{"test":"none"}}));
    let (ok, v, _) = bee(&r, &["unset", "--key", "a.label"]);
    assert!(ok);
    assert_eq!(v["removed"], json!(false));
}

#[test]
fn string_flag_keeps_a_json_looking_value_as_a_string() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), "{}");
    bee(&r, &["set", "--key", "flag", "--value", "false"]);
    bee(&r, &["set", "--key", "word", "--value", "false", "--string"]);
    assert_eq!(file(&r, ".bee/config.json"), json!({"flag": false, "word": "false"}));
}

#[test]
fn local_writes_the_overlay_and_warns_when_the_tracked_file_sets_the_key() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"gate_bypass":"off"}"#);
    let tracked_before = std::fs::read(r.join(".bee/config.json")).unwrap();

    let (ok, v, out) = bee(&r, &["set", "--local", "--key", "gate_bypass", "--value", "normal"]);
    assert!(ok, "{out}");
    assert!(v["warning"].as_str().is_some_and(|w| w.contains(".bee/config.json")), "{out}");
    assert_eq!(file(&r, ".bee/config.local.json"), json!({"gate_bypass": "normal"}));
    assert_eq!(std::fs::read(r.join(".bee/config.json")).unwrap(), tracked_before);

    let (_, v, _) = bee(&r, &["get", "--local", "--key", "gate_bypass"]);
    assert_eq!(v["value"], json!("normal"));
    assert_eq!(v["local"], json!(true));
    assert!(v["warning"].is_string());
    let (_, v, _) = bee(&r, &["get", "--key", "gate_bypass"]);
    assert_eq!(v["value"], json!("off"));
    assert_eq!(v["warning"], Value::Null);

    let (_, v, _) = bee(&r, &["set", "--local", "--key", "uat_stop", "--value", "merge"]);
    assert_eq!(v["warning"], Value::Null);
}

#[test]
fn explain_names_the_layer_the_effective_value_comes_from() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"gate_bypass":"off","uat_stop":"close"}"#);
    std::fs::write(r.join(".bee/config.local.json"), r#"{"gate_bypass":"normal"}"#).unwrap();

    let (ok, v, out) = bee(&r, &["explain", "--key", "gate_bypass"]);
    assert!(ok, "{out}");
    assert_eq!(v, json!({"key":"gate_bypass","effective":"normal","source":"local","tracked":"off","local":"normal"}));

    let (_, v, _) = bee(&r, &["explain", "--key", "uat_stop"]);
    assert_eq!(v["source"], json!("tracked"));
    assert_eq!(v["effective"], json!("close"));
    assert_eq!(v["local"], Value::Null);

    let (_, v, _) = bee(&r, &["explain", "--key", "worktree_first"]);
    assert_eq!(v["source"], json!("unset"));
    assert_eq!(v["effective"], Value::Null);
    let t = text(&r, &["explain", "--key", "worktree_first"]);
    assert!(t.contains("built-in default") && t.contains("docs/config-reference.md"), "{t}");
}

#[test]
fn an_invalid_file_refuses_the_write_and_keeps_its_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    for broken in ["{not json", "[1,2]"] {
        let r = repo(&tmp.path().join(broken.len().to_string()), broken);
        let (ok, v, out) = bee(&r, &["set", "--key", "a", "--value", "1"]);
        assert!(!ok, "{out}");
        assert!(v["error"].as_str().is_some_and(|e| e.contains(".bee/config.json")), "{out}");
        let (ok, _, _) = bee(&r, &["unset", "--key", "a"]);
        assert!(!ok);
        assert_eq!(std::fs::read_to_string(r.join(".bee/config.json")).unwrap(), broken);
    }
}

#[test]
fn every_governed_key_is_refused_by_set_and_unset_with_the_file_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let config = r#"{"team":{"claude":{"code":"opus"}},"herding":{"enabled":true,"agents":{"a":{"command":"x"}},"agent_command":"run"},"hooks":{"write_guard":true}}"#;
    let r = repo(tmp.path(), config);
    let refused = [
        vec!["set", "--key", "team.claude.code", "--value", "sonnet"],
        vec!["set", "--key", "team.claude.review", "--value", "sonnet"],
        vec!["set", "--key", "team", "--value", "{}"],
        vec!["set", "--key", "herding.agents.b", "--value", r#"{"command":"y"}"#],
        vec!["set", "--key", "herding.agents.a.command", "--value", "z"],
        vec!["set", "--key", "herding.agent_command", "--value", "other"],
        vec!["set", "--key", "herding", "--value", "{}"],
        vec!["set", "--key", "hooks.write_guard", "--value", "false"],
        vec!["set", "--key", "hooks.new_hook", "--value", "false"],
        vec!["set", "--local", "--key", "hooks.write_guard", "--value", "false"],
        vec!["unset", "--key", "team.claude.code"],
        vec!["unset", "--key", "team"],
        vec!["unset", "--key", "herding.agents.a"],
        vec!["unset", "--key", "herding.agent_command"],
        vec!["unset", "--key", "herding"],
        vec!["unset", "--key", "hooks.write_guard"],
        vec!["unset", "--key", "hooks"],
    ];
    for args in &refused {
        let (ok, v, out) = bee(&r, args);
        assert!(!ok, "{args:?} must be refused: {out}");
        assert!(
            v["error"].as_str().is_some_and(|e| e.contains("ask the user to edit this mapping directly")),
            "{args:?}: {out}"
        );
        assert_eq!(std::fs::read_to_string(r.join(".bee/config.json")).unwrap(), config, "{args:?}");
        assert!(!r.join(".bee/config.local.json").exists(), "{args:?}");
    }
    let (ok, _, out) = bee(&r, &["set", "--key", "herding.enabled", "--value", "false"]);
    assert!(ok, "a key beside the governed ones stays settable: {out}");
}

#[test]
fn validate_reports_parse_errors_and_exits_nonzero() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"gate_bypass":"off"}"#);
    let (ok, v, out) = bee(&r, &["validate"]);
    assert!(ok, "{out}");
    assert_eq!(v["ok"], json!(true));
    std::fs::write(r.join(".bee/config.local.json"), "{oops").unwrap();
    let (ok, v, out) = bee(&r, &["validate"]);
    assert!(!ok, "{out}");
    assert_eq!(v["problems"][0]["code"], json!("config-unparseable"));
}
