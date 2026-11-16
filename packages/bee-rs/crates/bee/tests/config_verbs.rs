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
    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(&full)
        .current_dir(repo)
        .env("BEEHIVE_CONFIG_DIR", global_dir(repo))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let parsed = serde_json::from_str(stdout.trim()).unwrap_or(Value::Null);
    (out.status.success(), parsed, stdout)
}

fn global_dir(repo: &Path) -> PathBuf {
    repo.parent().unwrap().join("global")
}

fn put_global(repo: &Path, value: Value) {
    std::fs::create_dir_all(global_dir(repo)).unwrap();
    std::fs::write(global_dir(repo).join("config.json"), value.to_string()).unwrap();
}

fn put(repo: &Path, rel: &str, value: Value) {
    std::fs::write(repo.join(rel), serde_json::to_string_pretty(&value).unwrap() + "\n").unwrap();
}

fn raw(repo: &Path, rel: &str) -> String {
    std::fs::read_to_string(repo.join(rel)).unwrap()
}

fn text(repo: &Path, args: &[&str]) -> String {
    let mut full: Vec<&str> = vec!["config"];
    full.extend_from_slice(args);
    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(&full)
        .current_dir(repo)
        .env("BEEHIVE_CONFIG_DIR", global_dir(repo))
        .output()
        .unwrap();
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
    assert_eq!(
        v,
        json!({"key":"gate_bypass","effective":"normal","source":"local","global":null,"tracked":"off","team":null,"local":"normal"})
    );

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

const TEAM_TABLE: &str = r#"{
  "ceiling_seconds": 1800,
  "agent_command": "paseo-claude",
  "control_command": ["claude", "-p", "{PROMPT}"],
  "agents": {
    "paseo-claude": {"paseo": {"provider": "claude", "mode": "bypassPermissions"}},
    "paseo-pi-flash": {"paseo": {"provider": "pi", "model": "hive/flash", "thinking": "medium"}},
    "claude-sonnet": ["claude", "--model", "sonnet"]
  }
}"#;

fn real_shaped_config() -> Value {
    json!({
        "gate_bypass": "off",
        "commands": {"test": "cargo test"},
        "team": {"pi": {"code": {"kind": "herding", "agent": "paseo-pi-flash", "description": "write code"}}},
        "herding": serde_json::from_str::<Value>(TEAM_TABLE).unwrap(),
        "guards": {"cross_project": true}
    })
}

fn effective(repo: &Path, key: &str) -> Value {
    bee(repo, &["explain", "--key", key]).1["effective"].clone()
}

#[test]
fn config_global_explain_names_each_of_the_five_sources() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"from_tracked":"t"}"#);
    put_global(&r, json!({"from_global":"g"}));
    put(&r, ".bee/team.json", json!({"team":{"claude":{"code":{"kind":"cli"}}}}));
    put(&r, ".bee/config.local.json", json!({"from_local":"l"}));
    for (key, source) in [
        ("from_global", "global"),
        ("from_tracked", "tracked"),
        ("team.claude.code", "team"),
        ("from_local", "local"),
        ("nothing_sets_this", "unset"),
    ] {
        let (ok, v, out) = bee(&r, &["explain", "--key", key]);
        assert!(ok, "{out}");
        assert_eq!(v["source"], json!(source), "{key}: {out}");
    }
    let (_, v, _) = bee(&r, &["explain", "--key", "from_global"]);
    assert_eq!(v["global"], json!("g"));
    assert_eq!(v["tracked"], Value::Null);
    assert_eq!(v["team"], Value::Null);
    assert_eq!(v["effective"], json!("g"));
    let (_, v, _) = bee(&r, &["explain", "--key", "team.claude.code.kind"]);
    assert_eq!(v["team"], json!("cli"));
}

#[test]
fn config_global_explain_a_key_in_two_layers_names_the_higher_one() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"k":"tracked"}"#);
    put_global(&r, json!({"k":"global"}));
    let (_, v, _) = bee(&r, &["explain", "--key", "k"]);
    assert_eq!(v["source"], json!("tracked"));
    assert_eq!(v["global"], json!("global"));
    assert_eq!(v["effective"], json!("tracked"));
}

#[test]
fn config_global_validate_reports_each_problem_kind_with_file_and_key() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(
        tmp.path(),
        r#"{"team":{"claude":{"code":{"kind":"cli","command":["claude"],"prompt":"arg"}}},"herding":{"agent_command":"a"}}"#,
    );
    put(
        &r,
        ".bee/team.json",
        json!({"herding":{"agent_command":"b","max_parallel":3},"ship_visibility":"draft-pr"}),
    );
    put_global(&r, json!({"guards":{"cross_project":false},"gate_bypass":"total","hooks":{"x":false},"fine":1}));
    let (ok, v, out) = bee(&r, &["validate"]);
    assert!(!ok, "{out}");
    let problems = v["problems"].as_array().unwrap();
    let has = |code: &str, file: &str, key: &str| {
        problems.iter().any(|p| p["code"] == json!(code) && p["file"] == json!(file) && p["key"] == json!(key))
    };
    assert!(has("team-foreign-key", ".bee/team.json", "ship_visibility"), "{out}");
    assert!(has("team-foreign-key", ".bee/team.json", "herding.max_parallel"), "{out}");
    assert!(has("duplicate", ".bee/team.json", "herding.agent_command"), "{out}");
    for key in ["guards", "gate_bypass", "hooks"] {
        let file = global_dir(&r).join("config.json").display().to_string();
        assert!(has("global-forbidden-key", &file, key), "{key}: {out}");
    }

    std::fs::write(r.join(".bee/team.json"), "{broken").unwrap();
    std::fs::write(r.join(".bee/config.local.json"), "[1]").unwrap();
    let (ok, v, out) = bee(&r, &["validate"]);
    assert!(!ok, "{out}");
    let files: Vec<&str> =
        v["problems"].as_array().unwrap().iter().filter(|p| p["code"] == json!("config-unparseable")).map(|p| p["file"].as_str().unwrap()).collect();
    assert!(files.contains(&".bee/team.json") && files.contains(&".bee/config.local.json"), "{out}");
}

#[test]
fn config_global_validate_lists_the_keys_the_global_layer_supplies() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"set_in_both":"t"}"#);
    put_global(&r, json!({"set_in_both":"g","only_global":1,"nested":{"deep":true}}));
    let (ok, v, out) = bee(&r, &["validate"]);
    assert!(ok, "{out}");
    assert_eq!(v["global_supplied"], json!(["nested.deep", "only_global"]));
    let t = text(&r, &["validate"]);
    assert!(t.contains("only_global") && t.contains("nested.deep"), "{t}");
}

#[test]
fn config_global_get_reads_only_the_global_file() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"k":"tracked","only_tracked":1}"#);
    put_global(&r, json!({"k":"global","only_global":2}));
    let (ok, v, out) = bee(&r, &["get", "--key", "k", "--global"]);
    assert!(ok, "{out}");
    assert_eq!((v["present"].clone(), v["value"].clone()), (json!(true), json!("global")));
    let (_, v, _) = bee(&r, &["get", "--global", "--key", "only_tracked"]);
    assert_eq!(v["present"], json!(false));
    let (_, v, _) = bee(&r, &["get", "--key", "only_global"]);
    assert_eq!(v["present"], json!(false));
    assert_eq!(text(&r, &["get", "--key", "only_global", "--global"]).trim(), "only_global = 2");
}

#[test]
fn config_global_set_and_unset_refuse_the_global_flag() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"k":"tracked"}"#);
    put_global(&r, json!({"k":"global"}));
    let before = raw(&r, ".bee/config.json");
    let global_before = std::fs::read_to_string(global_dir(&r).join("config.json")).unwrap();
    for args in [vec!["set", "--key", "k", "--value", "x", "--global"], vec!["unset", "--key", "k", "--global"]] {
        let (ok, _, out) = bee(&r, &args);
        assert!(!ok, "{args:?} must be refused: {out}");
    }
    assert_eq!(raw(&r, ".bee/config.json"), before);
    assert_eq!(std::fs::read_to_string(global_dir(&r).join("config.json")).unwrap(), global_before);
}

#[test]
fn config_global_with_local_refuses() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), "{}");
    put_global(&r, json!({"k":1}));
    let (ok, v, out) = bee(&r, &["get", "--key", "k", "--global", "--local"]);
    assert!(!ok, "{out}");
    assert!(v["error"].as_str().is_some_and(|e| e.contains("--global") && e.contains("--local")), "{out}");
}

#[test]
fn config_global_governed_keys_still_refuse_with_a_global_layer_present() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"team":{"claude":{"code":"opus"}}}"#);
    put_global(&r, json!({"team":{"claude":{"review":"sonnet"}}}));
    let before = raw(&r, ".bee/config.json");
    let (ok, v, out) = bee(&r, &["set", "--key", "team.claude.code", "--value", "sonnet"]);
    assert!(!ok, "{out}");
    assert!(v["error"].as_str().is_some_and(|e| e.contains("ask the user to edit this mapping directly")), "{out}");
    assert_eq!(raw(&r, ".bee/config.json"), before);
}

#[test]
fn split_team_moves_the_team_table_and_keeps_the_merged_view() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), "{}");
    put(&r, ".bee/config.json", real_shaped_config());
    let views: Vec<Value> = ["team", "herding", "herding.agents", "herding.agent_command", "gate_bypass"]
        .iter()
        .map(|k| effective(&r, k))
        .collect();
    let (ok, v, out) = bee(&r, &["split-team"]);
    assert!(ok, "{out}");
    assert_eq!(v["moved"], json!(["team", "herding.agents", "herding.agent_command"]));
    let tracked = file(&r, ".bee/config.json");
    assert_eq!(tracked["herding"], json!({"ceiling_seconds": 1800, "control_command": ["claude", "-p", "{PROMPT}"]}));
    assert!(tracked.get("team").is_none());
    let team = file(&r, ".bee/team.json");
    assert_eq!(team["herding"]["agents"], real_shaped_config()["herding"]["agents"]);
    assert_eq!(team["herding"]["agent_command"], json!("paseo-claude"));
    assert_eq!(team["team"], real_shaped_config()["team"]);
    assert!(raw(&r, ".bee/team.json").ends_with("}\n"));
    assert!(raw(&r, ".bee/team.json").starts_with("{\n  \""));
    let after: Vec<Value> = ["team", "herding", "herding.agents", "herding.agent_command", "gate_bypass"]
        .iter()
        .map(|k| effective(&r, k))
        .collect();
    assert_eq!(views, after);
    let t = text(&tmp.path().join("repo"), &["validate"]);
    assert!(t.starts_with("config validate: OK"), "{t}");
}

#[test]
fn split_team_says_to_commit_both_files_together() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), "{}");
    put(&r, ".bee/config.json", real_shaped_config());
    let t = text(&r, &["split-team"]);
    assert!(t.contains(".bee/config.json") && t.contains(".bee/team.json") && t.contains("together"), "{t}");
}

#[test]
fn split_team_is_idempotent() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), "{}");
    put(&r, ".bee/config.json", real_shaped_config());
    assert!(bee(&r, &["split-team"]).0);
    let (config, team) = (raw(&r, ".bee/config.json"), raw(&r, ".bee/team.json"));
    let (ok, v, out) = bee(&r, &["split-team"]);
    assert!(ok, "{out}");
    assert_eq!(v["moved"], json!([]));
    assert!(text(&r, &["split-team"]).contains("nothing to move"));
    assert_eq!((raw(&r, ".bee/config.json"), raw(&r, ".bee/team.json")), (config, team));
}

#[test]
fn split_team_with_nothing_to_move_creates_no_team_file() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"gate_bypass":"off"}"#);
    let (ok, _, out) = bee(&r, &["split-team"]);
    assert!(ok, "{out}");
    assert!(!r.join(".bee/team.json").exists());
}

#[test]
fn split_team_refuses_a_conflicting_value_with_zero_change() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), "{}");
    put(&r, ".bee/config.json", real_shaped_config());
    put(&r, ".bee/team.json", json!({"herding":{"agent_command":"someone-else"}}));
    let (config, team) = (raw(&r, ".bee/config.json"), raw(&r, ".bee/team.json"));
    let (ok, v, out) = bee(&r, &["split-team"]);
    assert!(!ok, "{out}");
    assert!(v["error"].as_str().is_some_and(|e| e.contains("herding.agent_command")), "{out}");
    assert_eq!((raw(&r, ".bee/config.json"), raw(&r, ".bee/team.json")), (config, team));
}

#[test]
fn split_team_finishes_from_the_in_between_state() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), "{}");
    put(&r, ".bee/config.json", real_shaped_config());
    let full = real_shaped_config();
    put(
        &r,
        ".bee/team.json",
        json!({"team": full["team"], "herding": {"agents": full["herding"]["agents"], "agent_command": full["herding"]["agent_command"]}}),
    );
    let (ok, v, out) = bee(&r, &["split-team"]);
    assert!(ok, "{out}");
    assert_eq!(v["moved"], json!(["team", "herding.agents", "herding.agent_command"]));
    assert!(file(&r, ".bee/config.json").get("team").is_none());
    assert_eq!(file(&r, ".bee/team.json")["herding"]["agent_command"], json!("paseo-claude"));
}

#[test]
fn split_team_merges_into_an_existing_team_file() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"team":{"pi":{"code":{"agent":"a"}}}}"#);
    put(&r, ".bee/team.json", json!({"herding":{"agent_command":"x"}}));
    let (ok, _, out) = bee(&r, &["split-team"]);
    assert!(ok, "{out}");
    let team = file(&r, ".bee/team.json");
    assert_eq!(team["herding"]["agent_command"], json!("x"));
    assert_eq!(team["team"], json!({"pi":{"code":{"agent":"a"}}}));
}

#[test]
fn split_team_drop_global_removes_equal_leaves_and_keeps_differing_ones() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), "{}");
    put(&r, ".bee/config.json", real_shaped_config());
    let agents = real_shaped_config()["herding"]["agents"].clone();
    put_global(
        &r,
        json!({
            "team": {"pi": {"code": real_shaped_config()["team"]["pi"]["code"].clone()}},
            "herding": {
                "agent_command": "paseo-claude",
                "agents": {"paseo-claude": agents["paseo-claude"].clone(), "paseo-pi-flash": {"paseo": {"provider": "other"}}}
            }
        }),
    );
    let keys = ["team", "herding", "herding.agents", "herding.agent_command"];
    let before: Vec<Value> = keys.iter().map(|k| effective(&r, k)).collect();
    let (ok, v, out) = bee(&r, &["split-team", "--drop-global"]);
    assert!(ok, "{out}");
    assert!(v["dropped"].as_array().is_some_and(|d| !d.is_empty()), "{out}");
    let team = file(&r, ".bee/team.json");
    assert!(team.get("team").is_none(), "an emptied team table is pruned: {team}");
    assert!(team["herding"].get("agent_command").is_none());
    assert!(team["herding"]["agents"].get("paseo-claude").is_none());
    assert_eq!(team["herding"]["agents"]["paseo-pi-flash"], agents["paseo-pi-flash"]);
    assert_eq!(team["herding"]["agents"]["claude-sonnet"], agents["claude-sonnet"]);
    let after: Vec<Value> = keys.iter().map(|k| effective(&r, k)).collect();
    assert_eq!(before, after);
}

#[test]
fn split_team_without_drop_global_never_compares_with_global() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), "{}");
    put(&r, ".bee/config.json", real_shaped_config());
    put_global(&r, json!({"herding": {"agent_command": "paseo-claude"}}));
    let (ok, v, out) = bee(&r, &["split-team"]);
    assert!(ok, "{out}");
    assert_eq!(v["dropped"], json!([]));
    assert_eq!(file(&r, ".bee/team.json")["herding"]["agent_command"], json!("paseo-claude"));
}

fn run(repo: &Path, args: &[&str]) -> (bool, String, String) {
    let out = Command::cargo_bin("bee")
        .unwrap()
        .args(args)
        .current_dir(repo)
        .env("BEEHIVE_CONFIG_DIR", global_dir(repo))
        .env_remove("BEE_HERDING_WORKER")
        .env_remove("BEE_HERDING_JOB_ID")
        .output()
        .unwrap();
    (out.status.success(), String::from_utf8_lossy(&out.stdout).into_owned(), String::from_utf8_lossy(&out.stderr).into_owned())
}

fn team_show(repo: &Path) -> Value {
    let (ok, out, err) = run(repo, &["team", "show", "--json"]);
    assert!(ok, "{out}{err}");
    serde_json::from_str(out.trim()).unwrap()
}

fn agent_command_template(repo: &Path) -> String {
    run(repo, &["herding", "command-template", "agent_command", "--main-root", repo.to_str().unwrap()]).1
}

fn claude_role_model(show: &Value, role: &str) -> Value {
    show["runtimes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|rt| rt["runtime"] == json!("claude"))
        .flat_map(|rt| rt["roles"].as_array().unwrap().iter())
        .find(|r| r["role"] == json!(role))
        .map(|r| r["model"].clone())
        .unwrap_or(Value::Null)
}

#[test]
fn config_global_forbidden_keys_never_reach_the_merged_view() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"uat_stop":"close"}"#);
    put_global(
        &r,
        json!({"gate_bypass":"total","hooks":{"write-guard":false},"guards":{"cross_project":false},"fine":1}),
    );
    for key in ["gate_bypass", "hooks", "hooks.write-guard", "guards", "guards.cross_project"] {
        let (ok, v, out) = bee(&r, &["explain", "--key", key]);
        assert!(ok, "{out}");
        assert_eq!(
            (v["source"].clone(), v["effective"].clone(), v["global"].clone()),
            (json!("unset"), Value::Null, Value::Null),
            "{key}: {out}"
        );
    }
    assert_eq!(effective(&r, "fine"), json!(1));
}

#[test]
fn config_global_corrupt_file_is_named_and_read_as_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), r#"{"team":{"claude":{"code":{"model":"opus"}}}}"#);
    std::fs::create_dir_all(global_dir(&r)).unwrap();
    std::fs::write(global_dir(&r).join("config.json"), "{not json").unwrap();
    let path = global_dir(&r).join("config.json").display().to_string();
    let (ok, v, out) = bee(&r, &["validate"]);
    assert!(!ok, "{out}");
    assert!(
        v["problems"].as_array().unwrap().iter().any(|p| p["code"] == json!("config-unparseable") && p["file"] == json!(path)),
        "{out}"
    );
    let (ok, v, out) = bee(&r, &["explain", "--key", "team"]);
    assert!(!ok && v["error"].as_str().is_some_and(|e| e.contains(&path)), "{out}");
    let (ok, out, err) = run(&r, &["team", "show", "--runtime", "claude", "--json"]);
    assert!(ok, "{out}{err}");
    assert!(err.contains(&path), "{err}");
    assert_eq!(claude_role_model(&serde_json::from_str(out.trim()).unwrap(), "code"), json!("opus"));
}

#[test]
fn config_global_team_slots_show_in_team_show_and_a_project_slot_wins() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), "{}");
    put_global(&r, json!({"team":{"claude":{"code":{"model":"opus"},"review":{"model":"haiku"}}}}));
    put(&r, ".bee/team.json", json!({"team":{"claude":{"review":{"model":"sonnet"}}}}));
    let show = team_show(&r);
    assert_eq!(claude_role_model(&show, "code"), json!("opus"));
    assert_eq!(claude_role_model(&show, "review"), json!("sonnet"));
}

#[test]
fn split_team_and_drop_global_keep_team_show_and_the_agent_command_identical() {
    let tmp = tempfile::tempdir().unwrap();
    let r = repo(tmp.path(), "{}");
    let mut cfg = real_shaped_config();
    cfg["herding"]["agent_command"] = json!(["claude", "--model", "sonnet"]);
    put(&r, ".bee/config.json", cfg);
    let (show, command) = (team_show(&r), agent_command_template(&r));
    assert_eq!(command, "claude\n--model\nsonnet\n");

    let (ok, v, out) = bee(&r, &["split-team"]);
    assert!(ok, "{out}");
    assert_eq!(v["moved"], json!(["team", "herding.agents", "herding.agent_command"]));
    assert_eq!((team_show(&r), agent_command_template(&r)), (show.clone(), command.clone()));
    let (ok, v, out) = bee(&r, &["split-team"]);
    assert!(ok && v["moved"] == json!([]), "{out}");

    put_global(&r, file(&r, ".bee/team.json"));
    let (ok, v, out) = bee(&r, &["split-team", "--drop-global"]);
    assert!(ok && v["dropped"].as_array().is_some_and(|d| !d.is_empty()), "{out}");
    assert_eq!(file(&r, ".bee/team.json"), json!({}));
    assert!(file(&r, ".bee/config.json").get("team").is_none());
    assert_eq!((team_show(&r), agent_command_template(&r)), (show, command));

    let team_bytes = raw(&r, ".bee/team.json");
    let (ok, v, out) = bee(&r, &["split-team", "--drop-global"]);
    assert!(ok && v["moved"] == json!([]) && v["dropped"] == json!([]), "{out}");
    assert_eq!(raw(&r, ".bee/team.json"), team_bytes);
}
