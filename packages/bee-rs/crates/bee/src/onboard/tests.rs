// onboard — end-to-end fixture suite.
//
// Every case builds a miniature bee source checkout (a plugin root with the
// exact geometry onboard_bee.mjs walks) plus a host repo, then drives the
// same entry point `bee onboard` uses. The four repo shapes the port must
// hold are all here: empty, partially onboarded, drifted, and a repo whose
// .claude/settings.json already carries FOREIGN hook entries that must
// survive.

use super::*;
use serde_json::json;
use std::path::{Path, PathBuf};

const VERSION: &str = "9.9.9";

const DESIGN_DIRS: [&str; 6] = [
    "bee-design",
    "bee-design-apple",
    "bee-design-charts",
    "bee-design-figma",
    "bee-design-motion",
    "bee-design-web",
];

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    repo: PathBuf,
}

fn write(p: &Path, body: &str) {
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

/// Every fixture pins the SAME sandbox home before it runs. Two onboarding
/// surfaces are machine-level — the legacy ~/.claude/skills refresh pass and
/// ~/.codex/config.toml's status line — and a test run must never read or
/// write the developer's real ones.
fn pin_sandbox_home() {
    super::util::TEST_HOME.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("bee-onboard-home-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    });
    // Proof the sandbox is inert for both machine-level surfaces.
    assert!(!super::util::exists(&super::source::skills_target_root()));
    assert!(!super::util::exists(&super::hooks_wiring::codex_user_config_path()));
}

/// A source checkout whose release tuple resolves, with one skill, one lib
/// module, one helper, one prompt, one statusline pair, one agent template
/// and a nested expertise tree.
fn fixture() -> Fixture {
    pin_sandbox_home();
    let dir = tempfile::tempdir().unwrap();
    let root = dunce::canonicalize(dir.path()).unwrap().join("src");
    let repo = dunce::canonicalize(dir.path()).unwrap().join("repo");
    std::fs::create_dir_all(&repo).unwrap();

    // Release identity tuple. R6 CUTOVER: this is the two plugin manifests and
    // nothing else — `BEE_VERSION` moved out of packages/bee/lib/state.mjs into
    // .claude-plugin/plugin.json, which is now both the tuple's first member
    // and the "runtime marker" slot.
    write(
        &root.join(".claude-plugin").join("plugin.json"),
        &format!("{{\"version\":\"{VERSION}\"}}"),
    );
    write(
        &root.join(".codex-plugin").join("plugin.json"),
        &format!("{{\"version\":\"{VERSION}\"}}"),
    );
    // A .git marker makes classifySource report source_checkout.
    std::fs::create_dir_all(root.join(".git")).unwrap();
    // THE ENGINE MARKER — what `Engine::locate` walks up looking for, and the
    // file whose presence proves this really is a bee checkout. R6 CUTOVER:
    // this was packages/bee/scripts/onboard_bee.mjs.
    write(&root.join("packages").join("bee").join("AGENTS.block.md"), "# bee\n\nrules here\n\n\n");

    // A vendorable helper and lib module. Nothing in the real source tree ships
    // `.mjs` any more, but the vendoring machinery that copies and (since R6)
    // REMOVES them is still live for hosts installed before the cutover, so the
    // fixture keeps a specimen of each: `a_helper_the_source_stopped_shipping…`
    // and `a_hook_the_source_stopped_shipping…` drive the removal paths with
    // them.
    write(&root.join("packages").join("bee").join("lib").join("cells.mjs"), "// cells\n");
    write(&root.join("packages").join("bee").join("bee.mjs"), "// dispatcher\n");
    write(&root.join("packages").join("bee").join("prompts").join("worker-cell.md"), "prompt\n");
    write(
        &root.join("packages").join("bee").join("statusline").join("statusline-command.sh"),
        "#!/bin/sh\necho hi\n",
    );
    write(
        &root.join("packages").join("bee").join("agents").join("bee-gather.md.tmpl"),
        "---\nmodel: {{TIER_MODEL}}\n---\ngather\n",
    );
    for name in super::templates::HOOK_FILENAMES {
        write(&root.join("packages").join("bee").join("hooks").join(name), &format!("// {name}\n"));
    }
    write(&root.join("skills").join("bee-hive").join("SKILL.md"), "hive\n");
    write(&root.join("skills").join("bee-hive").join("references").join("r.md"), "ref\n");
    for name in DESIGN_DIRS {
        write(&root.join("packages").join("bee").join("design").join(name).join("SKILL.md"), &format!("{name}\n"));
    }
    write(&root.join("expertise").join("tests.md"), "tests guide\n");
    write(
        &root.join("expertise").join("tests").join("patterns").join("differential-testing.md"),
        "differential testing pattern\n",
    );

    Fixture { _dir: dir, root, repo }
}

fn plan(fx: &Fixture, extra: &[&str]) -> Value {
    let mut argv = vec!["--repo-root", fx.repo.to_str().unwrap(), "--json"];
    argv.extend_from_slice(extra);
    let (code, payload) = run_with_root(&fx.root, &argv);
    assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::SUCCESS));
    payload
}

fn apply(fx: &Fixture, extra: &[&str]) -> Value {
    let mut argv = vec!["--repo-root", fx.repo.to_str().unwrap(), "--json", "--apply"];
    argv.extend_from_slice(extra);
    let (_, payload) = run_with_root(&fx.root, &argv);
    payload
}

fn actions(payload: &Value, key: &str) -> Vec<String> {
    payload[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["action"].as_str().unwrap().to_string())
        .collect()
}

fn paths_for(payload: &Value, key: &str, action: &str) -> Vec<String> {
    payload[key]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["action"] == action)
        .map(|i| i["path"].as_str().unwrap().to_string())
        .collect()
}

// ── argv parsing ───────────────────────────────────────────────────────────

#[test]
fn parse_args_mirrors_the_engine_flag_set() {
    let ok = |argv: &[&str]| match parse_args(&argv.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    {
        ParseOutcome::Parsed(a) => *a,
        _ => panic!("expected a parse"),
    };
    let a = ok(&["--repo-root", "/x", "--apply", "--json", "--repo-hooks"]);
    assert_eq!(a.repo_root.as_deref(), Some("/x"));
    assert!(a.apply && a.json && a.repo_hooks);
    assert!(a.claude_md, "CLAUDE.md is a default artifact (D1)");
    assert_eq!(a.runtime, "both");

    let a = ok(&["--repo-root=/y", "--runtime=codex", "--no-claude-md", "--global-skills"]);
    assert_eq!(a.repo_root.as_deref(), Some("/y"));
    assert_eq!(a.runtime, "codex");
    assert!(!a.claude_md);
    assert!(a.global_skills);

    // --claude-md is a no-op alias of the default.
    assert!(ok(&["--no-claude-md", "--claude-md"]).claude_md);
    assert!(ok(&["--plugin-source", "--force-downgrade"]).plugin_source);

    // 4cac0774: the status display is a default artifact; --no-statusline opts
    // this run out.
    assert!(ok(&["--repo-root", "/x"]).statusline, "on by default");
    assert!(!ok(&["--no-statusline"]).statusline);
}

#[test]
fn parse_args_rejects_unknown_flags_and_bad_runtimes() {
    let err = |argv: &[&str]| {
        match parse_args(&argv.iter().map(|s| s.to_string()).collect::<Vec<_>>()) {
            ParseOutcome::Error(m) => m,
            _ => panic!("expected an error"),
        }
    };
    assert_eq!(err(&["--nope"]), "Unknown argument: --nope");
    assert_eq!(
        err(&["--runtime", "rust"]),
        "--runtime must be claude, codex, pi, or both (got: rust)"
    );
}

// ── Pi role table (host-packaging-gaps D2/D3) ──────────────────────────────

fn config_path(fx: &Fixture) -> PathBuf {
    fx.repo.join(".bee").join("config.json")
}

fn pi_items(payload: &Value, key: &str) -> Vec<Value> {
    payload[key].as_array().unwrap().iter().filter(|i| i["action"] == "add_pi_team").cloned().collect()
}

fn read_json(p: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

#[test]
fn runtime_pi_parses_plans_and_applies() {
    let parsed = parse_args(&["--runtime".to_string(), "pi".to_string()]);
    assert!(matches!(parsed, ParseOutcome::Parsed(a) if a.runtime == "pi"));
    let fx = fixture();
    assert_eq!(plan(&fx, &["--runtime", "pi"])["status"], "changes_needed");
    let a = apply(&fx, &["--runtime", "pi"]);
    assert_eq!(a["status"], "applied");
    assert_eq!(a["recheck"], "up_to_date");
}

#[test]
fn a_fresh_apply_writes_the_pi_table_and_records_the_offer() {
    let fx = fixture();
    apply(&fx, &[]);
    let config = read_json(&config_path(&fx));
    let pi = config["team"]["pi"].as_object().unwrap();
    assert_eq!(pi.len(), 19);
    assert_eq!(
        pi.keys().collect::<Vec<_>>(),
        config["team"]["claude"].as_object().unwrap().keys().collect::<Vec<_>>()
    );
    for (name, slot) in pi {
        assert_eq!(slot["kind"], "herding", "{name}");
        assert_eq!(slot["agent"], "pi", "{name}");
        assert!(slot["description"].as_str().is_some_and(|d| !d.is_empty()), "{name}");
    }
    assert_eq!(config["herding"]["agents"]["pi"], json!(["pi"]));
    let ledger = read_json(&fx.repo.join(".bee").join("onboarding.json"));
    assert_eq!(ledger["pi_team_offered"], true);
    // Deleting the table afterwards is respected.
    let mut trimmed = config.clone();
    trimmed["team"].as_object_mut().unwrap().remove("pi");
    let body = format!("{}\n", crate::jsjson::stringify_pretty(&trimmed));
    write(&config_path(&fx), &body);
    assert!(pi_items(&plan(&fx, &[]), "plan").is_empty());
    apply(&fx, &[]);
    assert_eq!(std::fs::read_to_string(config_path(&fx)).unwrap(), body);
}

#[test]
fn a_fresh_apply_seeds_the_design_role_and_renders_the_bee_design_agent() {
    let fx = fixture();
    let template = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..").join("..").join("..").join("bee").join("agents").join("bee-design.md.tmpl");
    write(
        &fx.root.join("packages").join("bee").join("agents").join("bee-design.md.tmpl"),
        &std::fs::read_to_string(&template).unwrap(),
    );
    apply(&fx, &[]);
    let config = read_json(&config_path(&fx));
    assert_eq!(config["team"]["claude"]["design"]["model"], "sonnet");
    assert_eq!(config["team"]["pi"]["design"]["kind"], "herding");
    let rendered =
        std::fs::read_to_string(fx.repo.join(".claude").join("agents").join("bee-design.md")).unwrap();
    assert!(rendered.starts_with("---\nname: bee-design\n"), "{rendered}");
    assert!(rendered.contains("load the `bee-design` skill"), "{rendered}");
}

/// A 4-space, CRLF config with a `team` but no `team.pi`.
const EXISTING_CONFIG: &str = "{\r\n    \"host_shell\": \"posix\",\r\n    \"team\": {\r\n        \"claude\": {\r\n            \"code\": \"opus\"\r\n        }\r\n    },\r\n    \"herding\": {\r\n        \"agents\": {\r\n            \"mine\": [\"claude\"]\r\n        }\r\n    },\r\n    \"zeta\": 1\r\n}\r\n";

#[test]
fn an_existing_config_gains_the_pi_table_once_keeping_everything_else() {
    let fx = fixture();
    write(&config_path(&fx), EXISTING_CONFIG);
    let p = plan(&fx, &[]);
    let items = pi_items(&p, "plan");
    assert_eq!(items, vec![json!({"action": "add_pi_team", "path": ".bee/config.json", "table": "team"})]);
    let notice = |payload: &Value| {
        payload["notices"].as_array().unwrap().iter().any(|n| {
            let n = n.as_str().unwrap();
            n.contains("team.pi") && n.contains("herding.agents.pi") && n.contains(".bee/config.json")
        })
    };
    assert!(notice(&p), "plan names the keys");

    let a = apply(&fx, &[]);
    assert!(notice(&a), "apply names the keys");
    assert_eq!(a["recheck"], "up_to_date");
    let text = std::fs::read_to_string(config_path(&fx)).unwrap();
    assert!(text.ends_with("}\r\n"));
    assert!(!text.replace("\r\n", "").contains('\n'), "every line ending stays CRLF");
    assert!(text.starts_with("{\r\n    \"host_shell\": \"posix\",\r\n    \"team\": {\r\n        \"claude\""));
    let v: Value = serde_json::from_str(&text).unwrap();
    let before: Value = serde_json::from_str(EXISTING_CONFIG).unwrap();
    // Top-level keys: same order, nothing new.
    assert_eq!(
        v.as_object().unwrap().keys().collect::<Vec<_>>(),
        before.as_object().unwrap().keys().collect::<Vec<_>>()
    );
    assert_eq!(v["host_shell"], before["host_shell"]);
    assert_eq!(v["zeta"], before["zeta"]);
    assert_eq!(v["team"]["claude"]["code"], before["team"]["claude"]["code"]);
    assert_eq!(v["team"].as_object().unwrap().keys().collect::<Vec<_>>(), vec!["claude", "pi"]);
    assert_eq!(v["team"]["pi"], super::templates::default_config()["team"]["pi"]);
    assert_eq!(v["herding"]["agents"]["mine"], json!(["claude"]));
    assert_eq!(
        v["herding"]["agents"].as_object().unwrap().keys().collect::<Vec<_>>(),
        vec!["mine", "pi"]
    );
    assert_eq!(v["herding"]["agents"]["pi"], json!(["pi"]));

    // Second run: nothing to add.
    let p2 = plan(&fx, &[]);
    assert!(pi_items(&p2, "plan").is_empty());
    assert!(!notice(&p2));
}

#[test]
fn a_legacy_models_config_gains_models_pi() {
    let fx = fixture();
    write(&config_path(&fx), "{\n  \"models\": {\n    \"claude\": {}\n  }\n}\n");
    assert_eq!(pi_items(&plan(&fx, &[]), "plan")[0]["table"], "models");
    apply(&fx, &[]);
    let v = read_json(&config_path(&fx));
    assert_eq!(v["models"]["pi"], super::templates::default_config()["team"]["pi"]);
    assert!(v.get("team").is_none());
    assert_eq!(v["herding"], json!({"agents": {"pi": ["pi"]}}));
}

#[test]
fn a_config_the_pi_table_must_not_touch_stays_byte_identical() {
    let cases: [(&str, &str, Option<&str>, bool); 5] = [
        ("team.pi present", "{\n  \"team\": {\n    \"pi\": {\n      \"design\": {}\n    }\n  }\n}\n", None, false),
        (
            "team.pi only in config.local.json",
            "{\n  \"team\": {\n    \"claude\": {\n      \"design\": {}\n    }\n  }\n}\n",
            Some("{\"team\": {\"pi\": {\"code\": {\"kind\": \"herding\", \"agent\": \"pi\"}}}}"),
            false,
        ),
        ("unparseable", "{ not json", None, false),
        ("neither team nor models", "{\"host_shell\":\"posix\"}\n", None, false),
        ("offered already, then deleted", "{\n  \"team\": {\n    \"claude\": {}\n  }\n}\n", None, true),
    ];
    for (label, body, local, offered) in cases {
        let fx = fixture();
        write(&config_path(&fx), body);
        if let Some(local) = local {
            write(&fx.repo.join(".bee").join("config.local.json"), local);
        }
        if offered {
            // A first apply adds the table and records the offer; the user
            // then deletes the table again.
            assert_eq!(pi_items(&apply(&fx, &[]), "applied").len(), 1, "{label}: first offer");
            write(&config_path(&fx), body);
        }
        assert!(pi_items(&plan(&fx, &[]), "plan").is_empty(), "{label}: planned");
        let a = apply(&fx, &[]);
        assert_eq!(a["status"], "applied", "{label}");
        assert_eq!(std::fs::read_to_string(config_path(&fx)).unwrap(), body, "{label}: bytes");
        assert!(pi_items(&a, "applied").is_empty(), "{label}: applied");
    }
}

fn design_dirs_by_home(repo: &Path) -> Vec<(String, Vec<String>)> {
    let homes: Vec<(String, Vec<String>)> = skill_homes(repo)
        .into_iter()
        .map(|(rel, root)| {
            let names = super::util::read_dir_sorted(&root)
                .into_iter()
                .map(|e| e.name)
                .filter(|n| n.starts_with("bee-design"))
                .collect();
            (rel, names)
        })
        .collect();
    assert!(!homes.is_empty(), "the host has skill homes to check");
    homes
}

fn every_home_holds(repo: &Path, expected: &[&str]) {
    for (rel, names) in design_dirs_by_home(repo) {
        assert_eq!(names, expected, "{rel}");
    }
}

fn design_packs_recorded(fx: &Fixture) -> Value {
    read_json(&config_path(fx))["design"]["packs"].clone()
}

#[test]
fn picking_web_and_charts_installs_the_core_and_those_two_packs_only() {
    let fx = fixture();
    let a = apply(&fx, &["--design-packs", "web,charts"]);
    assert_eq!(a["status"], "applied");
    assert_eq!(a["recheck"], "up_to_date");
    every_home_holds(&fx.repo, &["bee-design", "bee-design-charts", "bee-design-web"]);
    for (rel, root) in skill_homes(&fx.repo) {
        assert_eq!(
            std::fs::read_to_string(root.join("bee-design-web").join("SKILL.md")).unwrap(),
            "bee-design-web\n",
            "{rel}"
        );
    }
    assert_eq!(design_packs_recorded(&fx), json!(["web", "charts"]));
    assert_eq!(plan(&fx, &[])["status"], "up_to_date", "the recorded pick needs no flag");
}

#[test]
fn dropping_a_pack_removes_only_its_dir_and_keeps_a_host_skill() {
    let fx = fixture();
    apply(&fx, &["--design-packs", "web,charts"]);
    for (_, root) in skill_homes(&fx.repo) {
        write(&root.join("my-design").join("SKILL.md"), "mine\n");
    }
    let a = apply(&fx, &["--design-packs", "web"]);
    assert_eq!(a["recheck"], "up_to_date");
    every_home_holds(&fx.repo, &["bee-design", "bee-design-web"]);
    for (rel, root) in skill_homes(&fx.repo) {
        assert_eq!(
            std::fs::read_to_string(root.join("my-design").join("SKILL.md")).unwrap(),
            "mine\n",
            "{rel}"
        );
    }
    assert_eq!(design_packs_recorded(&fx), json!(["web"]));
}

#[test]
fn a_host_with_no_pick_gets_no_design_dir() {
    let fx = fixture();
    let a = apply(&fx, &[]);
    assert_eq!(a["recheck"], "up_to_date");
    every_home_holds(&fx.repo, &[]);
    assert!(read_json(&config_path(&fx)).get("design").is_none());
}

#[test]
fn picking_none_removes_every_design_dir_and_records_none() {
    let fx = fixture();
    apply(&fx, &["--design-packs", "web"]);
    every_home_holds(&fx.repo, &["bee-design", "bee-design-web"]);
    let a = apply(&fx, &["--design-packs", "none"]);
    assert_eq!(a["recheck"], "up_to_date");
    every_home_holds(&fx.repo, &[]);
    assert_eq!(design_packs_recorded(&fx), json!("none"));
    assert_eq!(plan(&fx, &[])["status"], "up_to_date");
}

#[test]
fn an_unknown_pack_refuses_before_any_write() {
    let refusal = |value: &str| match parse_args(&["--design-packs".to_string(), value.to_string()]) {
        ParseOutcome::Error(m) => m,
        _ => panic!("{value} must refuse"),
    };
    let message = refusal("web,webb");
    assert!(message.contains("webb"), "{message}");
    for pack in ["web", "charts", "motion", "apple", "figma", "none"] {
        assert!(message.contains(pack), "names {pack}: {message}");
    }
    refusal("none,web");
    refusal("");

    let fx = fixture();
    let code = try_native(&[
        OsString::from("onboard"),
        OsString::from("--repo-root"),
        fx.repo.clone().into_os_string(),
        OsString::from("--apply"),
        OsString::from("--design-packs"),
        OsString::from("webb"),
    ]);
    assert_eq!(format!("{code:?}"), format!("{:?}", Some(ExitCode::from(1))));
    assert_eq!(std::fs::read_dir(&fx.repo).unwrap().count(), 0, "nothing written");
}

#[test]
fn a_design_pick_without_apply_reports_the_planned_write_only() {
    let fx = fixture();
    let p = plan(&fx, &["--design-packs", "charts,web"]);
    let writes: Vec<Value> =
        p["plan"].as_array().unwrap().iter().filter(|i| i["action"] == "set_design_packs").cloned().collect();
    assert_eq!(
        writes,
        vec![json!({"action": "set_design_packs", "path": ".bee/config.json", "packs": ["web", "charts"]})]
    );
    let synced: Vec<&str> = p["plan"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["action"] == "sync_skill" && i["target"] == "repo-claude")
        .map(|i| i["skill"].as_str().unwrap())
        .collect();
    assert_eq!(synced, vec!["bee-hive", "bee-design", "bee-design-charts", "bee-design-web"]);
    assert!(!fx.repo.join(".bee").exists());
    assert!(!fx.repo.join(".claude").exists());
}

fn design_offers(payload: &Value) -> usize {
    payload["notices"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n.as_str() == Some(super::templates::DESIGN_PACKS_OFFER))
        .count()
}

#[test]
fn a_host_with_ui_files_and_no_pick_is_offered_the_packs() {
    let fx = fixture();
    write(&fx.repo.join("node_modules").join("pkg").join("index.html"), "<p></p>\n");
    write(&fx.repo.join("dist").join("app.css"), "p {}\n");
    write(&fx.repo.join("target").join("site").join("page.tsx"), "x\n");
    assert_eq!(design_offers(&plan(&fx, &[])), 0, "build and vendor trees are not the project's UI");

    write(&fx.repo.join("web").join("index.html"), "<p></p>\n");
    assert_eq!(design_offers(&plan(&fx, &[])), 1);
    let a = apply(&fx, &[]);
    assert_eq!(design_offers(&a), 1);
    every_home_holds(&fx.repo, &[]);
    assert!(read_json(&config_path(&fx)).get("design").is_none(), "the offer writes nothing");

    apply(&fx, &["--design-packs", "none"]);
    assert_eq!(design_offers(&plan(&fx, &[])), 0, "none is a pick");
    apply(&fx, &["--design-packs", "web"]);
    assert_eq!(design_offers(&plan(&fx, &[])), 0, "a list is a pick");
}

#[test]
fn a_host_with_no_ui_file_is_not_offered_the_packs() {
    let fx = fixture();
    write(&fx.repo.join("src").join("main.rs"), "fn main() {}\n");
    assert_eq!(design_offers(&plan(&fx, &[])), 0);
    assert_eq!(design_offers(&apply(&fx, &[])), 0);
}

const DESIGNLESS_CONFIG: &str = "{\r\n    \"host_shell\": \"posix\",\r\n    \"team\": {\r\n        \"claude\": {\r\n            \"code\": \"opus\"\r\n        },\r\n        \"pi\": {\r\n            \"code\": {\r\n                \"kind\": \"herding\",\r\n                \"agent\": \"pi\"\r\n            }\r\n        }\r\n    },\r\n    \"zeta\": 1\r\n}\r\n";

#[test]
fn an_existing_config_gains_the_design_role_once_keeping_its_other_bytes() {
    let fx = fixture();
    write(&config_path(&fx), DESIGNLESS_CONFIG);
    let a = apply(&fx, &[]);
    assert_eq!(a["recheck"], "up_to_date");

    let text = std::fs::read_to_string(config_path(&fx)).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["team"]["claude"]["design"]["model"], "sonnet");
    let description = v["team"]["claude"]["design"]["description"].as_str().unwrap().to_string();
    assert!(!description.is_empty());
    assert_eq!(
        v["team"]["pi"]["design"],
        json!({"kind": "herding", "agent": "pi", "description": description})
    );
    let expected = DESIGNLESS_CONFIG
        .replace(
            "\"code\": \"opus\"\r\n",
            &format!(
                "\"code\": \"opus\",\r\n            \"design\": {{\r\n                \"model\": \"sonnet\",\r\n                \"description\": \"{description}\"\r\n            }}\r\n"
            ),
        )
        .replace(
            "\"agent\": \"pi\"\r\n            }\r\n",
            &format!(
                "\"agent\": \"pi\"\r\n            }},\r\n            \"design\": {{\r\n                \"kind\": \"herding\",\r\n                \"agent\": \"pi\",\r\n                \"description\": \"{description}\"\r\n            }}\r\n"
            ),
        );
    assert_eq!(text, expected);
    assert_eq!(read_json(&fx.repo.join(".bee").join("onboarding.json"))["design_role_offered"], true);

    write(&config_path(&fx), DESIGNLESS_CONFIG);
    let p = plan(&fx, &[]);
    assert!(!actions(&p, "plan").contains(&"add_design_role".to_string()));
    apply(&fx, &[]);
    assert_eq!(std::fs::read_to_string(config_path(&fx)).unwrap(), DESIGNLESS_CONFIG, "a deleted row stays deleted");
}

#[test]
fn help_declines_so_the_shared_help_surface_answers() {
    assert!(matches!(parse_args(&["--help".to_string()]), ParseOutcome::Delegate));
    assert!(matches!(parse_args(&["-h".to_string()]), ParseOutcome::Delegate));
    assert!(try_native(&[OsString::from("onboard"), OsString::from("--help")]).is_none());
}

#[test]
fn probe_declines_every_non_onboard_argv() {
    assert!(try_native(&[]).is_none());
    assert!(try_native(&[OsString::from("status")]).is_none());
    assert!(try_native(&[OsString::from("cells"), OsString::from("list")]).is_none());
}

// ── plan on an empty repo ──────────────────────────────────────────────────

#[test]
fn plan_on_an_empty_repo_lists_the_whole_install() {
    let fx = fixture();
    let p = plan(&fx, &[]);
    assert_eq!(p["status"], "changes_needed");
    assert_eq!(p["bee_version"], VERSION);
    assert_eq!(p["source"], "source_checkout");

    let acts = actions(&p, "plan");
    // Order is the engine's section order, not alphabetical.
    let expected_head = vec![
        "create_agents_block",
        "propose_agents_header",
        "create_runtime_file", // .bee/state.json
        "create_runtime_file",
        "create_runtime_file",
        "create_runtime_file",
        "create_runtime_file",
        "create_runtime_file", // .bee/config-sample.json (config-sample-herding D3)
        "create_dir",
        "create_dir",
        "copy_helper",
    ];
    assert_eq!(acts[..expected_head.len()], expected_head[..]);
    assert!(acts.contains(&"create_gitignore_block".to_string()));
    assert!(acts.contains(&"create_gitattributes_block".to_string()));
    assert!(acts.contains(&"create_claude_md".to_string()));
    assert!(acts.contains(&"write_onboarding".to_string()));
    assert!(acts.contains(&"sync_skill".to_string()));
    // 4cac0774: a virgin host declares no statusLine, so the whole install now
    // includes writing the entry and vendoring the script — unless the host
    // shell is PowerShell, where onboarding skips the bash status display.
    let statusline = !super::merge::host_shell_is_powershell(&fx.repo);
    assert_eq!(acts.contains(&"merge_statusline_settings".to_string()), statusline);
    assert_eq!(acts.contains(&"copy_statusline".to_string()), statusline);

    // The nested expertise pattern file is planned by its POSIX-relative name.
    let expertise = paths_for(&p, "plan", "copy_expertise");
    assert_eq!(
        expertise,
        vec![
            ".bee/expertise/tests.md".to_string(),
            ".bee/expertise/tests/patterns/differential-testing.md".to_string()
        ]
    );

    // All three in-repo skill roots are resolved, in stable order, fresh.
    let targets = p["skills"]["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 3);
    assert_eq!(targets[0]["kind"], "repo-claude");
    assert_eq!(targets[0]["mode"], "fresh");
    assert_eq!(targets[1]["kind"], "repo-agents");
    assert_eq!(targets[2]["kind"], "repo-opencode");
    assert_eq!(targets[2]["mode"], "fresh");
    assert_eq!(targets[0]["versions"]["source"], VERSION);
    assert_eq!(targets[0]["versions"]["host_helpers"], "absent");
    assert_eq!(targets[0]["versions"]["installed_skills"], "absent");
    // Plan mode never writes.
    assert!(!fx.repo.join(".bee").exists());
}

#[test]
fn plan_payload_key_order_matches_node() {
    let fx = fixture();
    let p = plan(&fx, &[]);
    let keys: Vec<&str> = p.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    assert_eq!(
        keys,
        vec!["repo_root", "status", "source", "bee_version", "plan", "skills", "notices"]
    );
    // No worktree_migration key for an ordinary checkout (must_have truth).
    assert!(p.get("worktree_migration").is_none());
}

// ── apply, then idempotence ────────────────────────────────────────────────

#[test]
fn apply_on_an_empty_repo_then_reapply_is_a_no_op() {
    let fx = fixture();
    let statusline = !super::merge::host_shell_is_powershell(&fx.repo);
    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    assert_eq!(a["recheck"], "up_to_date");
    assert_eq!(a["recheck_plan"].as_array().unwrap().len(), 0);
    assert!(a["recheck_skills"].is_null());

    // AGENTS.md carries the block AND the proposed header.
    let agents = std::fs::read_to_string(fx.repo.join("AGENTS.md")).unwrap();
    assert!(agents.starts_with("# repo\n\n<!-- [unknown] one-line project description"));
    assert!(agents.contains("<!-- BEE:START -->\n# bee\n\nrules here\n<!-- BEE:END -->\n"));

    // CLAUDE.md, gitignore block, stubs, vendored lib/helpers/prompts.
    assert!(std::fs::read_to_string(fx.repo.join("CLAUDE.md")).unwrap().contains("@AGENTS.md"));
    let gi = std::fs::read_to_string(fx.repo.join(".gitignore")).unwrap();
    assert!(gi.starts_with("# BEE:START\n.bee/state.json\n"));
    assert!(gi.ends_with("# BEE:END\n"));

    let ga = std::fs::read_to_string(fx.repo.join(".gitattributes")).unwrap();
    assert!(ga.starts_with("# BEE:START\n"));
    assert!(ga.ends_with("# BEE:END\n"));
    for log in [
        "decisions.jsonl",
        "decisions-archive.jsonl",
        "backlog.jsonl",
        "review-candidates.jsonl",
        "deferred-queue.jsonl",
        "wave-ledger.jsonl",
    ] {
        assert!(ga.contains(&format!(".bee/{log} merge=union\n")), "{log} missing from .gitattributes");
    }
    assert!(fx.repo.join(".bee").join("bin").join("lib").join("cells.mjs").exists());
    assert!(fx.repo.join(".bee").join("bin").join("prompts").join("worker-cell.md").exists());

    // config-sample-herding D3: the annotated sample is seeded verbatim
    // (embedded from the bee repo's own .bee/config-sample.json) and carries
    // the herding key.
    let sample: Value = serde_json::from_str(
        &std::fs::read_to_string(fx.repo.join(".bee").join("config-sample.json")).unwrap(),
    )
    .unwrap();
    assert!(sample.get("herding").is_some());
    assert!(fx.repo.join("docs").join("specs").join("reading-map.md").exists());
    assert!(fx.repo.join("docs").join("history").join("learnings").join("critical-patterns.md").exists());

    // The nested expertise pattern landed at its nested path, not flattened.
    assert_eq!(
        std::fs::read_to_string(
            fx.repo.join(".bee").join("expertise").join("tests").join("patterns").join("differential-testing.md")
        )
        .unwrap(),
        "differential testing pattern\n"
    );
    assert_eq!(
        std::fs::read_to_string(fx.repo.join(".bee").join("expertise").join("tests.md")).unwrap(),
        "tests guide\n"
    );

    // Skills mirrored into all three roots, each with its stamp + sidecar.
    for rel in [".claude/skills", ".agents/skills", ".opencode/skills"] {
        let root = super::util::join_rel(&fx.repo, rel);
        assert!(root.join("bee-hive").join("SKILL.md").exists(), "{rel}");
        assert!(root.join("bee-hive").join("references").join("r.md").exists(), "{rel}");
        let stamp: Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(".bee-skills-version.json")).unwrap())
                .unwrap();
        assert_eq!(stamp["version"], VERSION);
        let sidecar: Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(".bee-render.json")).unwrap())
                .unwrap();
        assert_eq!(sidecar["schema"], "bee-render/2");
        assert_eq!(sidecar["skills"][0]["name"], "bee-hive");
    }
    assert_eq!(
        serde_json::from_str::<Value>(
            &std::fs::read_to_string(fx.repo.join(".agents").join("skills").join(".bee-render.json")).unwrap()
        )
        .unwrap()["target_runtime"],
        "codex"
    );
    assert_eq!(
        serde_json::from_str::<Value>(
            &std::fs::read_to_string(fx.repo.join(".opencode").join("skills").join(".bee-render.json")).unwrap()
        )
        .unwrap()["target_runtime"],
        "opencode"
    );

    // The ledger.
    let ledger: Value = serde_json::from_str(
        &std::fs::read_to_string(fx.repo.join(".bee").join("onboarding.json")).unwrap(),
    )
    .unwrap();
    let keys: Vec<&str> = ledger.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    assert_eq!(
        keys,
        vec![
            "schema_version",
            "bee_version",
            "managed",
            "agents_sync",
            "pi_team_offered",
            "design_role_offered",
            "created_at",
            "updated_at"
        ]
    );
    assert_eq!(ledger["schema_version"], "1.0");
    assert_eq!(ledger["bee_version"], VERSION);
    let managed_keys: Vec<&str> =
        ledger["managed"].as_object().unwrap().keys().map(|k| k.as_str()).collect();
    // 4cac0774: a fresh host declares no `statusLine`, so onboarding writes the
    // entry, vendors the script, and therefore OWNS the pair — the managed
    // ledger records its hashes from the first apply. Under the old opt-in this
    // key was absent here. A PowerShell host skips the pair, so owns nothing.
    let mut expected = vec![
        "agents_block",
        "gitignore_block",
        "gitattributes_block",
        "helpers",
        "lib",
        "expertise",
        "prompts",
    ];
    if statusline {
        expected.push("statusline");
    }
    assert_eq!(managed_keys, expected);
    // The managed hash is of the file's UTF-8 STRING content (hashFile).
    assert_eq!(
        ledger["managed"]["expertise"]["tests/patterns/differential-testing.md"],
        json!(super::util::sha256_str("differential testing pattern\n"))
    );
    assert_eq!(
        ledger["managed"]["agents_block"],
        json!(super::util::sha256_str(
            "<!-- BEE:START -->\n# bee\n\nrules here\n<!-- BEE:END -->\n"
        ))
    );

    // IDEMPOTENCE: a fresh plan is empty and a second apply changes nothing.
    let p2 = plan(&fx, &[]);
    assert_eq!(p2["status"], "up_to_date");
    assert_eq!(p2["plan"].as_array().unwrap().len(), 0);
    let before = tree_snapshot(&fx.repo);
    let a2 = apply(&fx, &[]);
    assert!(
        a2["applied"].as_array().unwrap().is_empty(),
        "a settled repo applies nothing; only the unconditional ledger rewrite runs"
    );
    assert_eq!(a2["recheck"], "up_to_date");
    let after = tree_snapshot(&fx.repo);
    // Everything except the ledger's updated_at stamp is byte-stable.
    for (path, body) in &before {
        if path.ends_with("onboarding.json") {
            continue;
        }
        assert_eq!(after.iter().find(|(p, _)| p == path).map(|(_, b)| b), Some(body), "{path}");
    }
    assert_eq!(before.len(), after.len());
}

// ── OpenCode guard plugin vendoring (opencode-support oc-13) ──────────────

#[test]
fn opencode_guard_plugin_installs_idempotently_and_repairs_drift() {
    let fx = fixture();
    write(&fx.root.join(".opencode").join("plugins").join("bee-guard.ts"), "// guard v1\n");

    let p = plan(&fx, &[]);
    assert_eq!(paths_for(&p, "plan", "copy_opencode_plugin"), vec![".opencode/plugins/bee-guard.ts"]);

    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    assert_eq!(
        std::fs::read_to_string(fx.repo.join(".opencode").join("plugins").join("bee-guard.ts")).unwrap(),
        "// guard v1\n"
    );

    // IDEMPOTENT: a settled repo plans and applies nothing further.
    let p2 = plan(&fx, &[]);
    assert!(!actions(&p2, "plan").contains(&"copy_opencode_plugin".to_string()));
    let a2 = apply(&fx, &[]);
    assert!(!actions(&a2, "applied").contains(&"copy_opencode_plugin".to_string()));

    // DRIFT: a hand-edited plugin file is repaired on the next apply, same as
    // any other vendored helper.
    write(&fx.repo.join(".opencode").join("plugins").join("bee-guard.ts"), "tampered\n");
    let p3 = plan(&fx, &[]);
    assert_eq!(paths_for(&p3, "plan", "copy_opencode_plugin"), vec![".opencode/plugins/bee-guard.ts"]);
    apply(&fx, &[]);
    assert_eq!(
        std::fs::read_to_string(fx.repo.join(".opencode").join("plugins").join("bee-guard.ts")).unwrap(),
        "// guard v1\n"
    );
}

#[test]
fn a_source_checkout_with_no_opencode_plugin_plans_nothing_for_it() {
    // The ordinary fixture() never writes .opencode/plugins/ under fx.root —
    // this is the "not every source checkout carries it yet" case, and it
    // must stay silent rather than erroring or fabricating a directory.
    let fx = fixture();
    let p = plan(&fx, &[]);
    assert!(!actions(&p, "plan").contains(&"copy_opencode_plugin".to_string()));
    assert!(!fx.repo.join(".opencode").join("plugins").exists());
}

// ── Pi guard extension vendoring (pi-support D1) ──────────────────────────

#[test]
fn pi_guard_extension_installs_idempotently_and_repairs_drift() {
    let fx = fixture();
    write(&fx.root.join(".pi").join("extensions").join("bee-guard").join("index.ts"), "index v1\n");
    write(&fx.root.join(".pi").join("extensions").join("bee-guard").join("state.ts"), "state v1\n");

    let p = plan(&fx, &[]);
    assert_eq!(
        paths_for(&p, "plan", "copy_pi_extension"),
        vec![".pi/extensions/bee-guard/index.ts", ".pi/extensions/bee-guard/state.ts"]
    );

    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    assert_eq!(
        std::fs::read_to_string(fx.repo.join(".pi").join("extensions").join("bee-guard").join("index.ts"))
            .unwrap(),
        "index v1\n"
    );
    assert_eq!(
        std::fs::read_to_string(fx.repo.join(".pi").join("extensions").join("bee-guard").join("state.ts"))
            .unwrap(),
        "state v1\n"
    );

    let p2 = plan(&fx, &[]);
    assert!(!actions(&p2, "plan").contains(&"copy_pi_extension".to_string()));
    assert!(!actions(&p2, "plan").contains(&"create_dir".to_string()));
    let a2 = apply(&fx, &[]);
    assert!(!actions(&a2, "applied").contains(&"copy_pi_extension".to_string()));

    write(&fx.repo.join(".pi").join("extensions").join("bee-guard").join("state.ts"), "tampered\n");
    let p3 = plan(&fx, &[]);
    assert_eq!(paths_for(&p3, "plan", "copy_pi_extension"), vec![".pi/extensions/bee-guard/state.ts"]);
    apply(&fx, &[]);
    assert_eq!(
        std::fs::read_to_string(fx.repo.join(".pi").join("extensions").join("bee-guard").join("state.ts"))
            .unwrap(),
        "state v1\n"
    );
}

#[test]
fn a_source_checkout_with_no_pi_extension_plans_nothing_for_it() {
    let fx = fixture();
    write(&fx.repo.join(".pi").join("extensions").join("bee-guard.ts"), "legacy\n");
    write(&fx.repo.join(".pi").join("extensions").join("bee-guard").join("orphan.ts"), "orphan\n");
    let p = plan(&fx, &[]);
    assert!(!actions(&p, "plan").contains(&"copy_pi_extension".to_string()));
    assert!(!actions(&p, "plan").contains(&"remove_pi_extension".to_string()));
    assert!(fx.repo.join(".pi").join("extensions").join("bee-guard.ts").exists());
}

#[test]
fn onboard_removes_legacy_single_file_before_copying_folder() {
    let fx = fixture();
    write(&fx.root.join(".pi").join("extensions").join("bee-guard").join("index.ts"), "index v1\n");
    write(&fx.repo.join(".pi").join("extensions").join("bee-guard.ts"), "legacy\n");

    let p = plan(&fx, &[]);
    let items = p["plan"].as_array().unwrap();
    let legacy_remove_idx = items.iter().position(|i| i["action"] == "remove_pi_extension" && i["path"] == ".pi/extensions/bee-guard.ts");
    assert!(legacy_remove_idx.is_some());
    let first_copy_idx = items.iter().position(|i| i["action"] == "copy_pi_extension");
    assert!(first_copy_idx.is_some());
    assert!(legacy_remove_idx.unwrap() < first_copy_idx.unwrap());

    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    assert!(!fx.repo.join(".pi").join("extensions").join("bee-guard.ts").exists());
    assert!(fx.repo.join(".pi").join("extensions").join("bee-guard").join("index.ts").exists());
}

#[test]
fn onboard_prunes_unshipped_ts_in_guard_folder_and_preserves_non_ts() {
    let fx = fixture();
    write(&fx.root.join(".pi").join("extensions").join("bee-guard").join("index.ts"), "index v1\n");
    write(&fx.repo.join(".pi").join("extensions").join("bee-guard").join("index.ts"), "index v1\n");
    write(&fx.repo.join(".pi").join("extensions").join("bee-guard").join("old.ts"), "old\n");
    write(&fx.repo.join(".pi").join("extensions").join("bee-guard").join("notes.md"), "notes\n");

    let p = plan(&fx, &[]);
    assert_eq!(
        paths_for(&p, "plan", "remove_pi_extension"),
        vec![".pi/extensions/bee-guard/old.ts"]
    );

    apply(&fx, &[]);
    assert!(!fx.repo.join(".pi").join("extensions").join("bee-guard").join("old.ts").exists());
    assert!(fx.repo.join(".pi").join("extensions").join("bee-guard").join("notes.md").exists());
    assert!(fx.repo.join(".pi").join("extensions").join("bee-guard").join("index.ts").exists());

    let p2 = plan(&fx, &[]);
    assert_eq!(p2["plan"].as_array().unwrap().len(), 0);
}

#[test]
fn remove_pi_extension_arm_refuses_out_of_scope_paths() {
    let fx = fixture();
    let outside = fx.repo.join("outside.ts");
    write(&outside, "survive\n");

    let engine = Engine::from_plugin_root(fx.root.clone());
    let opts = Options {
        repo_hooks: false,
        claude_md: false,
        global_skills: false,
        sync_skills: false,
        force_downgrade: false,
        plugin_source: false,
        statusline: false,
        runtime: "all".into(),
        design_packs: None,
    };
    let mut computed = super::plan::compute_plan(&engine, &fx.repo, &opts);
    computed.plan.push(json!({"action": "remove_pi_extension", "path": ".pi/extensions/../outside.ts"}));
    computed.plan.push(json!({"action": "remove_pi_extension", "path": ".pi/extensions/bee-guard/sub/foo.ts"}));

    let outcome = super::apply::apply_computed_plan(&engine, &fx.repo, &opts, computed);
    if let ApplyOutcome::Ok(ok) = outcome {
        assert!(!ok.applied.iter().any(|i| i["action"] == "remove_pi_extension"));
    }
    assert!(outside.exists());
    assert!(super::apply::remove_pi_extension_admissible(".pi/extensions/bee-guard.ts"));
    assert!(super::apply::remove_pi_extension_admissible(".pi/extensions/bee-guard/index.ts"));
    assert!(!super::apply::remove_pi_extension_admissible(".pi/extensions/../outside.ts"));
}

/// (relative path, content) for every file under a repo, sorted.
fn tree_snapshot(root: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    fn walk(dir: &Path, prefix: &str, out: &mut Vec<(String, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let rel = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
            match e.file_type() {
                Ok(ft) if ft.is_dir() => walk(&dir.join(&name), &rel, out),
                Ok(ft) if ft.is_file() => {
                    out.push((rel, std::fs::read_to_string(dir.join(&name)).unwrap_or_default()))
                }
                _ => {}
            }
        }
    }
    walk(root, "", &mut out);
    out.sort();
    out
}

// ── partially onboarded / drifted repairs ──────────────────────────────────

#[test]
fn a_drifted_repo_repairs_exactly_the_drifted_files() {
    let fx = fixture();
    apply(&fx, &[]);

    // Drift four different managed surfaces.
    write(&fx.repo.join(".bee").join("bin").join("lib").join("cells.mjs"), "tampered\n");
    write(&fx.repo.join(".claude").join("skills").join("bee-hive").join("SKILL.md"), "tampered\n");
    write(&fx.repo.join(".bee").join("expertise").join("tests").join("patterns").join("differential-testing.md"), "tampered\n");
    let agents = std::fs::read_to_string(fx.repo.join("AGENTS.md")).unwrap();
    write(&fx.repo.join("AGENTS.md"), &agents.replace("rules here", "TAMPERED"));

    let p = plan(&fx, &[]);
    assert_eq!(p["status"], "changes_needed");
    let acts = actions(&p, "plan");
    assert!(acts.contains(&"update_agents_block".to_string()));
    assert!(acts.contains(&"copy_lib".to_string()));
    assert!(acts.contains(&"copy_expertise".to_string()));
    assert!(acts.contains(&"sync_skill".to_string()));
    // Untouched surfaces are NOT replanned.
    assert!(!acts.contains(&"create_claude_md".to_string()));
    assert!(!acts.contains(&"create_gitignore_block".to_string()));
    assert!(!acts.contains(&"create_stub".to_string()));
    // Only the drifted expertise file, not its sibling.
    assert_eq!(
        paths_for(&p, "plan", "copy_expertise"),
        vec![".bee/expertise/tests/patterns/differential-testing.md".to_string()]
    );
    // Only the claude root drifted; the agents root stays clean.
    let sync_targets: Vec<&str> = p["plan"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["action"] == "sync_skill")
        .map(|i| i["target"].as_str().unwrap())
        .collect();
    assert_eq!(sync_targets, vec!["repo-claude"]);

    let a = apply(&fx, &[]);
    assert_eq!(a["recheck"], "up_to_date");
    assert_eq!(
        std::fs::read_to_string(fx.repo.join(".bee").join("bin").join("lib").join("cells.mjs")).unwrap(),
        "// cells\n"
    );
    assert!(std::fs::read_to_string(fx.repo.join("AGENTS.md")).unwrap().contains("rules here"));
}

#[test]
fn a_tampered_host_version_marker_blocks_never_forceably() {
    let fx = fixture();
    apply(&fx, &[]);
    // R6 CUTOVER: the host_helpers version marker moved from
    // `.bee/bin/lib/state.mjs` (deleted with the vendored Node lib) to
    // `.bee/onboarding.json`'s `bee_version`. The GUARD is unchanged and this
    // test still proves it: a host marker that EXISTS but whose version cannot
    // be read is "unknown" — refuse, and never forceable (D3 / review P1-1).
    write(
        &fx.repo.join(".bee").join("onboarding.json"),
        "{\"schema_version\": \"1.0\", \"bee_version\": \"tampered\"}\n",
    );
    let p = plan(&fx, &[]);
    assert_eq!(p["status"], "blocked_downgrade");
    assert!(p["reason"].as_str().unwrap().contains("version unresolvable for host_helpers"));
    assert!(p["reason"].as_str().unwrap().contains("never forceable"));
    assert_eq!(p["versions"]["host_helpers"], "unknown");

    for argv in [
        vec!["--repo-root", fx.repo.to_str().unwrap(), "--json", "--apply"],
        vec!["--repo-root", fx.repo.to_str().unwrap(), "--json", "--apply", "--force-downgrade"],
    ] {
        let (code, a) = run_with_root(&fx.root, &argv);
        assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::from(1)), "{argv:?}");
        assert_eq!(a["status"], "blocked_downgrade");
        assert!(a.get("host_items").is_none(), "a non-forceable refusal never invites a force");
        assert!(
            std::fs::read_to_string(fx.repo.join(".bee").join("onboarding.json"))
                .unwrap()
                .contains("\"bee_version\": \"tampered\""),
            "zero mutations on a refused apply"
        );
    }
}

#[test]
fn a_partially_onboarded_repo_only_fills_the_gaps() {
    let fx = fixture();
    // Hand-place a few artifacts the way a half-finished install would.
    write(&fx.repo.join("AGENTS.md"), "# repo\n\nMy own project prose.\n");
    write(&fx.repo.join("CLAUDE.md"), "# Rules\n\n@AGENTS.md\n");
    write(&fx.repo.join(".gitignore"), "node_modules/\n");
    write(&fx.repo.join(".gitattributes"), "*.png binary\n");

    let p = plan(&fx, &[]);
    let acts = actions(&p, "plan");
    assert!(acts.contains(&"append_agents_block".to_string()));
    assert!(!acts.contains(&"propose_agents_header".to_string()), "existing prose suppresses it");
    assert!(!acts.contains(&"create_claude_md".to_string()));
    assert!(!acts.contains(&"append_claude_md_import".to_string()));
    assert!(acts.contains(&"append_gitignore_block".to_string()));
    assert!(acts.contains(&"append_gitattributes_block".to_string()));

    apply(&fx, &[]);
    let agents = std::fs::read_to_string(fx.repo.join("AGENTS.md")).unwrap();
    assert!(agents.starts_with("# repo\n\nMy own project prose.\n\n<!-- BEE:START -->"));
    let gi = std::fs::read_to_string(fx.repo.join(".gitignore")).unwrap();
    assert!(gi.starts_with("node_modules/\n\n# BEE:START\n"));
    let ga = std::fs::read_to_string(fx.repo.join(".gitattributes")).unwrap();
    assert!(ga.starts_with("*.png binary\n\n# BEE:START\n.bee/decisions.jsonl merge=union\n"));
    assert_eq!(plan(&fx, &[])["plan"].as_array().unwrap().len(), 0);
}

fn gitattributes_block() -> String {
    super::merge::render_gitattributes_block()
}

#[test]
fn onboarding_writes_the_gitattributes_block_once_and_keeps_host_lines() {
    let fx = fixture();
    write(&fx.repo.join(".gitattributes"), "*.png binary\n");

    let p = plan(&fx, &[]);
    assert_eq!(paths_for(&p, "plan", "append_gitattributes_block"), vec![".gitattributes"]);
    apply(&fx, &[]);

    let ga = std::fs::read_to_string(fx.repo.join(".gitattributes")).unwrap();
    assert_eq!(ga, format!("*.png binary\n\n{}\n", gitattributes_block().trim_end_matches('\n')));

    assert_eq!(plan(&fx, &[])["plan"].as_array().unwrap().len(), 0);
    let ga2 = std::fs::read_to_string(fx.repo.join(".gitattributes")).unwrap();
    assert!(ga2.starts_with("*.png binary\n"));
    assert_eq!(ga2, ga);

    write(&fx.repo.join(".gitattributes"), &format!("{ga}*.zip binary\n"));
    let p3 = plan(&fx, &[]);
    assert_eq!(p3["plan"].as_array().unwrap().len(), 0, "a host line after the block is not drift");
}

#[test]
fn a_drifted_gitattributes_block_is_rewritten_and_only_inside_the_markers() {
    let fx = fixture();
    write(&fx.repo.join(".gitattributes"), "*.png binary\n");
    apply(&fx, &[]);

    let ga = std::fs::read_to_string(fx.repo.join(".gitattributes")).unwrap();
    write(&fx.repo.join(".gitattributes"), &ga.replace("merge=union", "merge=ours"));

    let p = plan(&fx, &[]);
    assert_eq!(paths_for(&p, "plan", "update_gitattributes_block"), vec![".gitattributes"]);
    apply(&fx, &[]);
    let repaired = std::fs::read_to_string(fx.repo.join(".gitattributes")).unwrap();
    assert!(repaired.starts_with("*.png binary\n"));
    assert!(!repaired.contains("merge=ours"));
}

#[test]
fn a_decoy_gitattributes_marker_line_is_never_adopted() {
    let fx = fixture();
    write(&fx.repo.join(".gitattributes"), "# BEE:START custom notes\n*.png binary\n");
    let p = plan(&fx, &[]);
    assert_eq!(paths_for(&p, "plan", "append_gitattributes_block"), vec![".gitattributes"]);
    apply(&fx, &[]);
    let ga = std::fs::read_to_string(fx.repo.join(".gitattributes")).unwrap();
    assert!(ga.starts_with("# BEE:START custom notes\n*.png binary\n\n# BEE:START\n"));
}

fn git_ok(cwd: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git must be on PATH for the gitattributes merge fixture");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn two_branches_appending_to_the_decision_log_merge_cleanly_after_onboarding() {
    let fx = fixture();
    apply(&fx, &[]);
    git_ok(&fx.repo, &["init", "-q", "-b", "main", "."]);
    git_ok(&fx.repo, &["config", "core.autocrlf", "false"]);
    git_ok(&fx.repo, &["config", "user.email", "bee@example.com"]);
    git_ok(&fx.repo, &["config", "user.name", "bee"]);
    git_ok(&fx.repo, &["add", "-A"]);
    git_ok(&fx.repo, &["commit", "-qm", "onboard"]);

    write(&fx.repo.join(".bee").join("decisions.jsonl"), "{\"id\":\"main\"}\n");
    git_ok(&fx.repo, &["add", ".bee/decisions.jsonl"]);
    git_ok(&fx.repo, &["commit", "-qm", "main append"]);

    git_ok(&fx.repo, &["checkout", "-q", "-b", "side", "HEAD~1"]);
    write(&fx.repo.join(".bee").join("decisions.jsonl"), "{\"id\":\"side\"}\n");
    git_ok(&fx.repo, &["add", ".bee/decisions.jsonl"]);
    git_ok(&fx.repo, &["commit", "-qm", "side append"]);

    git_ok(&fx.repo, &["checkout", "-q", "main"]);
    git_ok(&fx.repo, &["merge", "--no-edit", "side"]);

    let merged = std::fs::read_to_string(fx.repo.join(".bee").join("decisions.jsonl")).unwrap();
    assert_eq!(merged, "{\"id\":\"main\"}\n{\"id\":\"side\"}\n");
    assert_eq!(git_ok(&fx.repo, &["status", "--porcelain"]), "", "a union merge leaves nothing staged");
}

// ── --repo-hooks and foreign settings preservation ─────────────────────────

#[test]
fn repo_hooks_wires_both_projections_and_preserves_foreign_entries() {
    let fx = fixture();
    // A repo that already has its OWN hook entries and its own settings keys.
    write(
        &fx.repo.join(".claude").join("settings.json"),
        r#"{
  "permissions": { "allow": ["Bash(ls:*)"] },
  "hooks": {
    "PreToolUse": [
      { "matcher": "Bash", "hooks": [{ "type": "command", "command": "node ./tools/my-guard.mjs" }] }
    ],
    "Notification": [
      { "hooks": [{ "type": "command", "command": "say hi" }] }
    ]
  }
}
"#,
    );

    let p = plan(&fx, &["--repo-hooks"]);
    let acts = actions(&p, "plan");
    assert!(acts.contains(&"copy_repo_hook".to_string()));
    assert!(acts.contains(&"merge_repo_hook_settings".to_string()));
    assert!(acts.contains(&"merge_codex_hooks".to_string()));

    apply(&fx, &["--repo-hooks"]);

    let settings: Value = serde_json::from_str(
        &std::fs::read_to_string(fx.repo.join(".claude").join("settings.json")).unwrap(),
    )
    .unwrap();
    // Foreign top-level key survives, in place.
    assert_eq!(settings["permissions"]["allow"][0], "Bash(ls:*)");
    // Foreign hook entries survive, bee entries are appended after them.
    assert_eq!(settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "node ./tools/my-guard.mjs");
    assert_eq!(settings["hooks"]["PreToolUse"].as_array().unwrap().len(), 5);
    assert_eq!(settings["hooks"]["Notification"][0]["hooks"][0]["command"], "say hi");
    // A .bak was taken.
    assert!(fx.repo.join(".claude").join("settings.json.bak").exists());
    // 2-space JSON with a trailing newline.
    let raw = std::fs::read_to_string(fx.repo.join(".claude").join("settings.json")).unwrap();
    assert!(raw.ends_with("}\n"));
    assert!(raw.contains("\n  \"hooks\": {"));

    // CUTOVER: a repo with no vendored binary yet gets the runtime-detecting
    // loop plus a VISIBLE fail-open, never a `node …bee-prompt-context.mjs`
    // command with nothing behind it.
    let cmd = settings["hooks"]["UserPromptSubmit"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap();
    assert!(!cmd.contains(".mjs"), "{cmd}");
    assert!(cmd.contains("exec \"$b\" hook prompt-context"), "{cmd}");
    assert!(cmd.contains("bee: hook binary missing"), "{cmd}");
    // Codex projection landed too.
    assert!(fx.repo.join(".codex").join("hooks.json").exists());
    assert!(fx.repo.join(".bee").join("bin").join("hooks").join("bee-write-guard.mjs").exists());

    // The ledger records the hook map, and the opt-in is sticky.
    let ledger: Value = serde_json::from_str(
        &std::fs::read_to_string(fx.repo.join(".bee").join("onboarding.json")).unwrap(),
    )
    .unwrap();
    assert!(ledger["managed"]["repo_hooks"]["bee-write-guard.mjs"].is_string());
    assert!(ledger["managed"]["repo_hooks"][".codex/hooks.json"].is_string());
    // A LATER run WITHOUT the flag still keeps hooks current (sticky opt-in).
    let p2 = plan(&fx, &[]);
    assert_eq!(p2["status"], "up_to_date");
    assert_eq!(p2["plan"].as_array().unwrap().len(), 0);
}

#[test]
fn malformed_settings_json_refuses_the_hooks_merge_with_zero_mutations() {
    let fx = fixture();
    let settings_path = fx.repo.join(".claude").join("settings.json");
    let original = r#"{"model": "opus", "hooks": {"#; // truncated / invalid JSON
    write(&settings_path, original);

    let a = apply(&fx, &["--repo-hooks"]);
    assert_eq!(a["status"], "blocked_hooks_merge");
    let reason = a["reason"].as_str().unwrap();
    assert!(reason.contains("settings.json"), "{reason}");
    assert!(reason.contains("not valid JSON"), "{reason}");
    // Zero mutations: the malformed file survives byte-for-byte, and nothing
    // else the apply would have written landed either.
    assert_eq!(std::fs::read_to_string(&settings_path).unwrap(), original);
    assert!(!fx.repo.join(".codex").join("hooks.json").exists());
    assert!(!fx.repo.join(".bee").join("onboarding.json").exists());
}

#[test]
fn non_object_hooks_key_refuses_instead_of_silently_dropping_it() {
    let fx = fixture();
    let settings_path = fx.repo.join(".claude").join("settings.json");
    let original = r#"{"model": "opus", "hooks": "not-an-object"}"#;
    write(&settings_path, original);

    let a = apply(&fx, &["--repo-hooks"]);
    assert_eq!(a["status"], "blocked_hooks_merge");
    let reason = a["reason"].as_str().unwrap();
    assert!(reason.contains("settings.json"), "{reason}");
    assert!(reason.contains("non-object \"hooks\""), "{reason}");
    assert_eq!(std::fs::read_to_string(&settings_path).unwrap(), original);
}

#[test]
fn non_array_event_value_refuses_instead_of_silently_dropping_it() {
    let fx = fixture();
    let settings_path = fx.repo.join(".claude").join("settings.json");
    let original = r#"{"hooks": {"PreToolUse": "not-an-array"}}"#;
    write(&settings_path, original);

    let a = apply(&fx, &["--repo-hooks"]);
    assert_eq!(a["status"], "blocked_hooks_merge");
    let reason = a["reason"].as_str().unwrap();
    assert!(reason.contains("settings.json"), "{reason}");
    assert!(reason.contains("non-array \"hooks.PreToolUse\""), "{reason}");
    assert_eq!(std::fs::read_to_string(&settings_path).unwrap(), original);
}

#[test]
fn hook_wiring_does_not_depend_on_a_vendored_binary_and_never_stacks() {
    let fx = fixture();
    apply(&fx, &["--repo-hooks"]);
    let read_command = |fx: &Fixture| -> Value {
        let settings: Value = serde_json::from_str(
            &std::fs::read_to_string(fx.repo.join(".claude").join("settings.json")).unwrap(),
        )
        .unwrap();
        settings["hooks"]["UserPromptSubmit"][0]["hooks"][0]["command"].clone()
    };
    let before = read_command(&fx);
    // Vendor the Rust binary, then re-onboard. The wiring resolves the binary
    // at HOOK time, so its presence at ONBOARD time changes nothing — and it
    // must not, or the same settings file would be wrong when read from a
    // linked worktree that has no binary of its own.
    write(&fx.repo.join(".bee").join("bin").join("bee.exe"), "MZ");
    let p = plan(&fx, &["--repo-hooks"]);
    assert_eq!(p["plan"].as_array().unwrap().len(), 0, "vendoring must not re-wire");
    assert_eq!(read_command(&fx), before);
    let cmd = before.as_str().unwrap();
    assert!(cmd.contains("$CLAUDE_PROJECT_DIR/.bee/bin/bee.exe"), "{cmd}");
    assert!(cmd.contains("--git-common-dir"), "the main-checkout arm: {cmd}");
    let settings: Value = serde_json::from_str(
        &std::fs::read_to_string(fx.repo.join(".claude").join("settings.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        settings["hooks"]["UserPromptSubmit"].as_array().unwrap().len(),
        2,
        "the node-shaped entry is REPLACED, never stacked beside the two bee rows \
(prompt context, activity)"
    );
}

// ── expertise removal ──────────────────────────────────────────────────────

#[test]
fn removing_a_nested_expertise_pattern_from_source_cleans_the_host() {
    let fx = fixture();
    apply(&fx, &[]);
    let vendored = fx
        .repo
        .join(".bee")
        .join("expertise")
        .join("tests")
        .join("patterns")
        .join("differential-testing.md");
    assert!(vendored.exists());

    std::fs::remove_file(
        fx.root.join("expertise").join("tests").join("patterns").join("differential-testing.md"),
    )
    .unwrap();

    let p = plan(&fx, &[]);
    assert_eq!(
        paths_for(&p, "plan", "remove_expertise"),
        vec![".bee/expertise/tests/patterns/differential-testing.md".to_string()]
    );
    apply(&fx, &[]);
    assert!(!vendored.exists(), "the nested orphan is actually cleaned");
    assert!(fx.repo.join(".bee").join("expertise").join("tests.md").exists());
    assert_eq!(plan(&fx, &[])["plan"].as_array().unwrap().len(), 0);
}

// ── refusals ───────────────────────────────────────────────────────────────

#[test]
fn a_disagreeing_release_tuple_blocks_with_zero_mutations() {
    let fx = fixture();
    write(&fx.root.join(".codex-plugin").join("plugin.json"), "{\"version\":\"1.0.0\"}");
    let p = plan(&fx, &[]);
    assert_eq!(p["status"], "blocked_no_source");
    assert!(p["reason"].as_str().unwrap().contains("tuple members disagree"));
    assert!(p["bee_version"].is_null());

    let (code, a) = run_with_root(
        &fx.root,
        &["--repo-root", fx.repo.to_str().unwrap(), "--json", "--apply"],
    );
    assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::from(1)));
    assert_eq!(a["status"], "blocked_no_source");
    assert!(!fx.repo.join(".bee").exists(), "a refused apply mutates nothing");
    assert!(!fx.repo.join("AGENTS.md").exists());
}

#[test]
fn an_older_source_refuses_a_downgrade_and_force_overrides_it() {
    let fx = fixture();
    apply(&fx, &[]);
    // Age the source below the installed host. R6 CUTOVER: the SOURCE version
    // is the plugin manifests now, and the HOST version is
    // .bee/onboarding.json's bee_version (written by the apply above), so
    // ageing the source means editing exactly these two files.
    write(&fx.root.join(".claude-plugin").join("plugin.json"), "{\"version\":\"1.0.0\"}");
    write(&fx.root.join(".codex-plugin").join("plugin.json"), "{\"version\":\"1.0.0\"}");
    // Drift one vendored lib module too, so the forceable refusal has a
    // copy_lib item to name as its blast radius.
    write(&fx.root.join("packages").join("bee").join("lib").join("cells.mjs"), "// cells v1\n");

    let p = plan(&fx, &[]);
    assert_eq!(p["status"], "blocked_downgrade");
    assert!(p["reason"].as_str().unwrap().contains("source 1.0.0 is older than"));
    assert_eq!(p["versions"]["installed_skills"], VERSION);

    let (code, a) = run_with_root(
        &fx.root,
        &["--repo-root", fx.repo.to_str().unwrap(), "--json", "--apply"],
    );
    assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::from(1)));
    assert_eq!(a["status"], "blocked_downgrade");
    // Forceable refusals name their blast radius.
    assert!(a["host_items"].as_array().unwrap().iter().any(|i| i["action"] == "copy_lib"));
    // The host is untouched by the refusal.
    assert_eq!(
        std::fs::read_to_string(fx.repo.join(".bee").join("bin").join("lib").join("cells.mjs")).unwrap(),
        "// cells\n"
    );

    let (code, forced) = run_with_root(
        &fx.root,
        &["--repo-root", fx.repo.to_str().unwrap(), "--json", "--apply", "--force-downgrade"],
    );
    assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::SUCCESS));
    assert_eq!(forced["status"], "applied");
    assert_eq!(forced["forced_downgrade"], true);
    assert_eq!(forced["versions"]["installed_skills"], VERSION);
    assert_eq!(
        std::fs::read_to_string(fx.repo.join(".bee").join("bin").join("lib").join("cells.mjs")).unwrap(),
        "// cells v1\n"
    );
}

// ── plugin-source / runtime gating ─────────────────────────────────────────

#[test]
fn plugin_source_skips_skill_sync_and_codex_runtime_adds_the_hybrid_hooks() {
    let fx = fixture();
    let p = plan(&fx, &["--plugin-source", "--runtime", "claude"]);
    let acts = actions(&p, "plan");
    assert!(!acts.contains(&"sync_skill".to_string()), "plugin-first skips projection");
    assert!(!acts.contains(&"merge_codex_hooks".to_string()));
    assert_eq!(p["skills"]["targets"].as_array().unwrap().len(), 0);

    let p = plan(&fx, &["--plugin-source", "--runtime", "codex"]);
    let acts = actions(&p, "plan");
    assert!(acts.contains(&"merge_codex_hooks".to_string()), "codex hybrid always wires hooks");
    assert!(acts.contains(&"copy_repo_hook".to_string()));

    apply(&fx, &["--plugin-source", "--runtime", "both"]);
    let ledger: Value = serde_json::from_str(
        &std::fs::read_to_string(fx.repo.join(".bee").join("onboarding.json")).unwrap(),
    )
    .unwrap();
    assert!(ledger["managed"]["codex_hooks"].is_object(), "a DISTINCT key from repo_hooks");
    assert!(ledger["managed"].get("repo_hooks").is_none());
    assert!(!fx.repo.join(".claude").join("skills").exists());
}

// ── notices ────────────────────────────────────────────────────────────────

#[test]
fn notices_surface_detected_commands_and_a_stale_advisor_key() {
    let fx = fixture();
    write(&fx.repo.join("package.json"), r#"{"scripts":{"test":"vitest"}}"#);
    let p = plan(&fx, &[]);
    let notices = p["notices"].as_array().unwrap();
    assert!(notices[0].as_str().unwrap().contains("Detected candidates: test: npm test"));

    apply(&fx, &[]);
    let cfg = fx.repo.join(".bee").join("config.json");
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
    v.as_object_mut().unwrap().insert("advisor".into(), json!({"mode": "x"}));
    std::fs::write(&cfg, v.to_string()).unwrap();
    let p = plan(&fx, &[]);
    assert!(p["notices"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n.as_str().unwrap().starts_with("advisor mode was removed in 0.1.23")));
}

// ── R6 cutover: stale vendored artifacts are REACHED, not stranded ────────
//
// Before the cutover, onboarding could remove a vendored LIB module that left
// the source (3c, ledger-derived) but had no equivalent for vendored HELPERS
// outside a hand-maintained RETIRED_HELPERS list, and none at all for vendored
// repo HOOKS. Deleting the whole `.mjs` tree at once turns both gaps into
// permanent litter on every host that ever onboarded — dead files at exactly
// the paths `.codex/hooks.json` and AGENTS.md still point agents at. These two
// tests are the proof that the new actions fire, and that they fire only where
// they are allowed to.

/// The source stops shipping a helper -> the host's copy is planned for
/// removal and actually unlinked, WITHOUT the name appearing in any list.
#[test]
fn a_helper_the_source_stopped_shipping_is_removed_from_the_host() {
    let fx = fixture();
    apply(&fx, &[]);
    let vendored = fx.repo.join(".bee").join("bin").join("bee.mjs");
    assert!(vendored.exists(), "fixture must vendor the helper first");

    // The R6 move: the helper leaves the SOURCE tree.
    std::fs::remove_file(fx.root.join("packages").join("bee").join("bee.mjs")).unwrap();

    let p = plan(&fx, &[]);
    let removals: Vec<&Value> = p["plan"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["action"] == "remove_helper")
        .collect();
    assert_eq!(
        removals.len(),
        1,
        "exactly one stale helper is owed, derived from the ledger: {:?}",
        p["plan"]
    );
    assert_eq!(removals[0]["path"], ".bee/bin/bee.mjs");

    apply(&fx, &[]);
    assert!(!vendored.exists(), "the stale helper must be gone from the host");

    // Idempotent: a second plan owes nothing (the ledger no longer records it).
    let again = plan(&fx, &[]);
    assert!(
        !actions(&again, "plan").contains(&"remove_helper".to_string()),
        "a removal that already happened must not be re-planned forever"
    );
}

/// The same, for vendored repo hooks — and a proof the applier's containment
/// refuses to unlink the BINARY that lives beside them.
#[test]
fn a_hook_the_source_stopped_shipping_is_removed_from_the_host() {
    let fx = fixture();
    apply(&fx, &["--repo-hooks"]);
    let vendored = fx.repo.join(".bee").join("bin").join("hooks").join("bee-write-guard.mjs");
    assert!(vendored.exists(), "fixture must vendor the hook first");

    // The R6 move: the hook leaves the SOURCE tree.
    std::fs::remove_file(fx.root.join("packages").join("bee").join("hooks").join("bee-write-guard.mjs"))
        .unwrap();

    let p = plan(&fx, &["--repo-hooks"]);
    let removals: Vec<&Value> = p["plan"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["action"] == "remove_repo_hook")
        .collect();
    assert_eq!(removals.len(), 1, "exactly one stale hook is owed: {:?}", p["plan"]);
    assert_eq!(removals[0]["path"], ".bee/bin/hooks/bee-write-guard.mjs");

    apply(&fx, &["--repo-hooks"]);
    assert!(!vendored.exists(), "the stale hook must be gone from the host");
    // Its siblings are untouched — this is a diff, not a tree wipe.
    assert!(fx.repo.join(".bee").join("bin").join("hooks").join("bee-model-guard.mjs").exists());

    let again = plan(&fx, &["--repo-hooks"]);
    assert!(!actions(&again, "plan").contains(&"remove_repo_hook".to_string()));
}

// ── human (non-JSON) rendering ─────────────────────────────────────────────

#[test]
fn human_emit_shape_matches_the_engine() {
    let payload = json!({
        "repo_root": "/r",
        "status": "changes_needed",
        "plan": [{"action": "copy_lib", "path": ".bee/bin/lib/x.mjs"}],
        "reason": "because",
        "versions": {"source": "1.0.0", "host_helpers": "absent", "installed_skills": "absent"},
        "skills": {"skipped": [{"skill": "bee-hive", "target": "repo-claude", "reason": "sym"}]},
        "notices": ["hello"],
    });
    // Exercised for panics/shape; stdout capture is the harness's job.
    emit(&payload, false);
    emit(&payload, true);
}

// ── engine root resolution refusal wording ─────────────────────────────────

#[test]
fn no_engine_message_names_the_invocation_root_and_the_missing_template_path() {
    let err = super::source::LocateError {
        invocation_root: PathBuf::from("/host/repo"),
        missing_template: PathBuf::from("/host/repo/packages/bee/AGENTS.block.md"),
        repo_root_candidate: None,
    };
    let message = no_engine_message(&err);
    assert!(message.contains("/host/repo"), "must name the invocation root: {message}");
    assert!(
        message.contains("/host/repo/packages/bee/AGENTS.block.md"),
        "must name the missing template path: {message}"
    );
}

// ── .bee/verify/ → the runtime skill homes (verification-ships-to-hosts D3) ──
//
// The negative case is the point of this block, so it is written from the HOST
// repository's point of view: a team hand-wrote `verify-payments/` into their
// Claude skill home, bee never generated it, and a routine `bee onboard
// --apply` must leave those bytes exactly as it found them. It is deliberately
// NOT written from the guard's point of view — a "foreign verify skill is
// removed" test would certify the hazard as intended behavior, and the
// fixture bytes are identical whether the directory is a stale render or a
// person's work.

/// Every file under `dir` as `(POSIX relative path, bytes)`, sorted. Two of
/// these compared is BYTE identity, not "the directory is still there".
fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(dir: &Path, prefix: &str, out: &mut Vec<(String, Vec<u8>)>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        let mut names: Vec<String> =
            rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        names.sort();
        for name in names {
            let p = dir.join(&name);
            let rel = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
            if p.is_dir() {
                out.push((format!("{rel}/"), Vec::new()));
                walk(&p, &rel, out);
            } else {
                out.push((rel, std::fs::read(&p).unwrap_or_default()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, "", &mut out);
    out
}

/// The runtime skill homes, read from `REPO_SKILL_TARGETS` through the same
/// helper the planner uses — never a hand-written list, so a fourth runtime
/// joining that table widens these assertions on its own.
fn skill_homes(repo: &Path) -> Vec<(String, PathBuf)> {
    super::plan::verify_render_targets(repo).into_iter().map(|(_, root, rel)| (rel, root)).collect()
}

fn verify_actions(payload: &Value, key: &str, action: &str) -> Vec<Value> {
    payload[key].as_array().unwrap().iter().filter(|i| i["action"] == action).cloned().collect()
}

#[test]
fn a_host_authored_verify_skill_is_byte_identical_after_apply() {
    let fx = fixture();

    // The host repo, described the way its owner would: someone on this team
    // wrote a verification skill for the payments service and committed it
    // into the Claude skill home. bee did not generate it, and it is not
    // under `.bee/verify/`.
    let host = fx.repo.join(".claude").join("skills").join("verify-payments");
    write(&host.join("SKILL.md"), "---\nname: verify-payments\n---\n\ndrive payments\n");
    write(&host.join("control-payments"), "#!/usr/bin/env bash\necho payments\n");
    write(&host.join("references").join("feature-map.md"), "checkout -> tests/checkout\n");
    let before = snapshot(&host);
    assert_eq!(before.len(), 4, "fixture must have three files and one directory");

    // The render path is LIVE during this apply, not dormant: the repo also
    // carries a generated skill bee really does render.
    write(
        &fx.repo.join(".bee").join("verify").join("verify-bee").join("SKILL.md"),
        "drive bee\n",
    );

    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    assert!(
        fx.repo.join(".claude").join("skills").join("verify-bee").join("SKILL.md").exists(),
        "the generated skill must really have been rendered"
    );

    assert_eq!(snapshot(&host), before, "host-authored verify-payments must be untouched");
    // No item in the whole applied plan even NAMES it — not to write it, and
    // above all not to remove it.
    for item in a["applied"].as_array().unwrap() {
        assert!(
            !item.to_string().contains("verify-payments"),
            "no plan item may name a host-authored skill: {item}"
        );
    }
}

#[test]
fn an_absent_verify_root_plans_nothing_and_never_blocks_bee_skill_sync() {
    let fx = fixture();
    assert!(!fx.repo.join(".bee").join("verify").exists());

    let p = plan(&fx, &[]);
    assert!(super::plan::compute_verify_skill_items(&fx.repo).is_empty());
    assert!(!actions(&p, "plan").contains(&"copy_verify_skill".to_string()));
    assert!(actions(&p, "plan").contains(&"sync_skill".to_string()), "bee's own sync is untouched");

    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    assert_eq!(a["recheck"], "up_to_date");
    assert!(fx.repo.join(".claude").join("skills").join("bee-hive").join("SKILL.md").exists());
}

#[test]
fn an_empty_or_non_directory_verify_root_plans_nothing_and_bee_still_syncs() {
    let fx = fixture();
    let root = fx.repo.join(".bee").join("verify");

    std::fs::create_dir_all(&root).unwrap();
    assert!(super::plan::compute_verify_skill_items(&fx.repo).is_empty(), "empty root");
    assert!(actions(&plan(&fx, &[]), "plan").contains(&"sync_skill".to_string()));

    std::fs::remove_dir(&root).unwrap();
    write(&root, "someone put a file here\n");
    assert!(super::plan::compute_verify_skill_items(&fx.repo).is_empty(), "root is a file");

    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    assert!(fx.repo.join(".claude").join("skills").join("bee-hive").join("SKILL.md").exists());
    assert_eq!(
        std::fs::read_to_string(&root).unwrap(),
        "someone put a file here\n",
        "the refused root is never rewritten either"
    );
}

#[cfg(unix)]
#[test]
fn an_unreadable_verify_root_plans_nothing_and_bee_still_syncs() {
    use std::os::unix::fs::PermissionsExt;
    let fx = fixture();
    let root = fx.repo.join(".bee").join("verify");
    write(&root.join("verify-app").join("SKILL.md"), "drive app\n");
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o000)).unwrap();
    if std::fs::read_dir(&root).is_ok() {
        // Running as root: mode 0 is not a real denial here, so the case this
        // test exists for cannot be staged. Restore and stop.
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }

    assert!(super::plan::compute_verify_skill_items(&fx.repo).is_empty());
    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    assert!(fx.repo.join(".claude").join("skills").join("bee-hive").join("SKILL.md").exists());

    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn one_generated_skill_renders_into_every_runtime_skill_home() {
    let fx = fixture();
    let src = fx.repo.join(".bee").join("verify").join("verify-app");
    write(
        &src.join("SKILL.md"),
        "drive app\n<!-- bee:only claude -->\nclaude only\n<!-- bee:end -->\ntail\n",
    );
    write(&src.join("scripts").join("control-app"), "#!/usr/bin/env bash\necho app\n");

    let homes = skill_homes(&fx.repo);
    let p = plan(&fx, &[]);
    let planned = paths_for(&p, "plan", "copy_verify_skill");
    assert_eq!(planned.len(), homes.len(), "one item per runtime home: {planned:?}");

    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    for (rel_root, root) in &homes {
        let dir = root.join("verify-app");
        assert!(dir.join("scripts").join("control-app").exists(), "{rel_root}");
        let body = std::fs::read_to_string(dir.join("SKILL.md")).unwrap();
        assert!(!body.contains("bee:only"), "marker lines are stripped in {rel_root}");
        assert!(body.contains("drive app") && body.contains("tail"), "{rel_root}");
        if rel_root == ".claude/skills" {
            assert!(body.contains("claude only"), "the claude home keeps its block");
        } else {
            assert!(!body.contains("claude only"), "{rel_root} must drop the claude-only block");
        }
    }
}

#[test]
fn rendering_a_generated_skill_twice_yields_an_identical_tree_and_plan() {
    let fx = fixture();
    write(
        &fx.repo.join(".bee").join("verify").join("verify-app").join("SKILL.md"),
        "drive app\n",
    );

    let first = apply(&fx, &[]);
    assert_eq!(first["status"], "applied");
    let homes = skill_homes(&fx.repo);
    let after_first: Vec<Vec<(String, Vec<u8>)>> =
        homes.iter().map(|(_, root)| snapshot(&root.join("verify-app"))).collect();

    let again = plan(&fx, &[]);
    assert!(
        paths_for(&again, "plan", "copy_verify_skill").is_empty(),
        "a settled render plans nothing: {:?}",
        again["plan"]
    );

    let second = apply(&fx, &[]);
    assert_eq!(second["status"], "applied");
    assert!(verify_actions(&second, "applied", "copy_verify_skill").is_empty());
    let after_second: Vec<Vec<(String, Vec<u8>)>> =
        homes.iter().map(|(_, root)| snapshot(&root.join("verify-app"))).collect();
    assert_eq!(after_first, after_second, "the second apply changed the tree");
}

#[test]
fn an_updated_source_rewrites_the_render_and_never_deletes_what_it_did_not_write() {
    let fx = fixture();
    let src = fx.repo.join(".bee").join("verify").join("verify-app");
    write(&src.join("SKILL.md"), "drive app v1\n");
    apply(&fx, &[]);

    // The host drops a file INSIDE a rendered skill. Nothing prunes it: it is
    // not drift, and the next apply must not plan a write because of it.
    let rendered = fx.repo.join(".claude").join("skills").join("verify-app");
    write(&rendered.join("NOTES.md"), "my notes\n");
    assert!(paths_for(&plan(&fx, &[]), "plan", "copy_verify_skill").is_empty());

    // Now the source really changes.
    write(&src.join("SKILL.md"), "drive app v2\n");
    let planned = paths_for(&plan(&fx, &[]), "plan", "copy_verify_skill");
    assert_eq!(planned.len(), skill_homes(&fx.repo).len());

    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    for (rel_root, root) in skill_homes(&fx.repo) {
        assert_eq!(
            std::fs::read_to_string(root.join("verify-app").join("SKILL.md")).unwrap(),
            "drive app v2\n",
            "{rel_root}"
        );
    }
    assert_eq!(
        std::fs::read_to_string(rendered.join("NOTES.md")).unwrap(),
        "my notes\n",
        "an update never removes a file bee did not write"
    );
}

#[test]
fn a_host_edited_projection_is_reported_blocked_and_never_overwritten() {
    let fx = fixture();
    let src = fx.repo.join(".bee").join("verify").join("verify-app");
    write(&src.join("SKILL.md"), "drive app v1\n");
    apply(&fx, &[]);

    let ledger = read_json(&fx.repo.join(".bee").join("onboarding.json"));
    assert!(
        ledger["managed"]["verify_skills"][".claude/skills/verify-app"]["SKILL.md"].is_string(),
        "apply records the rendered hash it wrote: {ledger}"
    );

    let claude = fx.repo.join(".claude").join("skills").join("verify-app");
    write(&claude.join("SKILL.md"), "drive app v1\nhost note\n");
    write(&src.join("SKILL.md"), "drive app v2\n");

    let p = plan(&fx, &[]);
    let blocked = verify_actions(&p, "plan", "blocked_verify_skill");
    assert_eq!(blocked.len(), 1, "only the edited home is blocked: {blocked:?}");
    assert_eq!(blocked[0]["skill"], "verify-app");
    assert_eq!(blocked[0]["path"], ".claude/skills/verify-app");
    let reason = blocked[0]["reason"].as_str().unwrap();
    assert!(reason.contains("edits bee did not write (SKILL.md)"), "{reason}");
    assert!(reason.contains("FIX: copy the edits into .bee/verify/verify-app"), "{reason}");
    assert!(
        !paths_for(&p, "plan", "copy_verify_skill").contains(&".claude/skills/verify-app".into()),
        "the edited projection is never planned for a write: {:?}",
        p["plan"]
    );
    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    assert_eq!(
        std::fs::read_to_string(claude.join("SKILL.md")).unwrap(),
        "drive app v1\nhost note\n",
        "the host's edit survives the apply"
    );
    assert_eq!(a["recheck"], "up_to_date", "a blocked item never fails install.sh");
    let recheck = verify_actions(&a, "recheck_plan", "blocked_verify_skill");
    assert_eq!(recheck.len(), 1, "the recheck still reports the edit: {recheck:?}");
    for (rel_root, root) in skill_homes(&fx.repo) {
        if rel_root == ".claude/skills" {
            continue;
        }
        assert_eq!(
            std::fs::read_to_string(root.join("verify-app").join("SKILL.md")).unwrap(),
            "drive app v2\n",
            "{rel_root} carries no host edit and is still refreshed"
        );
    }
}

#[test]
fn an_untouched_projection_is_still_refreshed_after_a_source_change() {
    let fx = fixture();
    let src = fx.repo.join(".bee").join("verify").join("verify-app");
    write(&src.join("SKILL.md"), "drive app v1\n");
    apply(&fx, &[]);

    write(&src.join("SKILL.md"), "drive app v2\n");
    let p = plan(&fx, &[]);
    assert!(verify_actions(&p, "plan", "blocked_verify_skill").is_empty(), "{:?}", p["plan"]);
    assert_eq!(
        paths_for(&p, "plan", "copy_verify_skill").len(),
        skill_homes(&fx.repo).len(),
        "every home is refreshed: {:?}",
        p["plan"]
    );

    apply(&fx, &[]);
    for (rel_root, root) in skill_homes(&fx.repo) {
        assert_eq!(
            std::fs::read_to_string(root.join("verify-app").join("SKILL.md")).unwrap(),
            "drive app v2\n",
            "{rel_root}"
        );
    }
}

#[test]
fn a_blocked_verify_skill_item_alone_never_asks_for_changes() {
    let only_blocked = [json!({
        "action": "blocked_verify_skill",
        "skill": "verify-app",
        "path": ".claude/skills/verify-app",
        "scope": "installed",
        "target": "repo-claude",
        "reason": "installed .claude/skills/verify-app has edits bee did not write (SKILL.md)"
    })];
    assert!(!super::plan::core_changes_needed(&only_blocked));
    assert!(!super::plan::core_changes_needed(&[]));
}

#[test]
fn with_no_record_a_projection_newer_than_its_source_is_blocked() {
    let fx = fixture();
    let src = fx.repo.join(".bee").join("verify").join("verify-app");
    write(&src.join("SKILL.md"), "drive app v1\n");
    apply(&fx, &[]);

    let ledger_path = fx.repo.join(".bee").join("onboarding.json");
    let mut ledger = read_json(&ledger_path);
    ledger["managed"].as_object_mut().unwrap().remove("verify_skills");
    write(&ledger_path, &format!("{}\n", jsjson::stringify_pretty(&ledger)));

    let claude = fx.repo.join(".claude").join("skills").join("verify-app");
    write(&claude.join("SKILL.md"), "drive app v1\nhost note\n");
    let ahead = std::time::SystemTime::now() + std::time::Duration::from_secs(60);
    filetime::set_file_mtime(
        claude.join("SKILL.md"),
        filetime::FileTime::from_system_time(ahead),
    )
    .unwrap();

    write(&src.join("SKILL.md"), "drive app v2\n");
    let blocked = verify_actions(&plan(&fx, &[]), "plan", "blocked_verify_skill");
    assert_eq!(blocked.len(), 1, "the mtime fallback blocks: {blocked:?}");
    assert_eq!(blocked[0]["path"], ".claude/skills/verify-app");
}

#[test]
fn a_malformed_generated_skill_reports_only_its_own_item_and_bee_still_syncs() {
    let fx = fixture();
    let root = fx.repo.join(".bee").join("verify");
    write(&root.join("verify-bad").join("SKILL.md"), "text\n<!-- bee:only claude -->\nunclosed\n");
    write(&root.join("verify-good").join("SKILL.md"), "good\n");

    let p = plan(&fx, &[]);
    // Host bytes can never raise bee's whole-tree refusal: that gate reads the
    // ENGINE's skill tree, and this one reports one item and moves on.
    assert_ne!(p["skills"]["status"], "blocked_render");
    assert!(actions(&p, "plan").contains(&"sync_skill".to_string()));

    let blocked = verify_actions(&p, "plan", "blocked_verify_skill");
    assert_eq!(blocked.len(), 1, "only the malformed skill is reported: {blocked:?}");
    assert_eq!(blocked[0]["skill"], "verify-bad");
    assert!(blocked[0]["reason"].as_str().unwrap().contains("malformed bee:only markers"));
    assert_eq!(
        paths_for(&p, "plan", "copy_verify_skill").len(),
        skill_homes(&fx.repo).len(),
        "the healthy sibling still renders"
    );

    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    assert!(fx.repo.join(".claude").join("skills").join("bee-hive").join("SKILL.md").exists());
    assert!(fx.repo.join(".claude").join("skills").join("verify-good").exists());
    assert!(!fx.repo.join(".claude").join("skills").join("verify-bad").exists());
    let skipped = a["skills"]["skipped"].as_array().unwrap();
    assert!(
        skipped.iter().any(|s| s["skill"] == "verify-bad"),
        "the skip is reported loudly: {skipped:?}"
    );
}

#[cfg(unix)]
#[test]
fn a_symlinked_verify_root_is_refused_and_never_followed() {
    let fx = fixture();
    let elsewhere = fx.repo.join("elsewhere");
    write(&elsewhere.join("verify-app").join("SKILL.md"), "drive app\n");
    std::fs::create_dir_all(fx.repo.join(".bee")).unwrap();
    std::os::unix::fs::symlink(&elsewhere, fx.repo.join(".bee").join("verify")).unwrap();

    let refusal = super::plan::verify_root_refusal(&fx.repo).unwrap();
    assert!(refusal.contains("symlink"), "{refusal}");
    assert!(super::plan::compute_verify_skill_items(&fx.repo).is_empty());

    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    for (rel_root, root) in skill_homes(&fx.repo) {
        assert!(!root.join("verify-app").exists(), "{rel_root} must stay clean");
    }
}

#[cfg(unix)]
#[test]
fn a_verify_root_that_resolves_onto_a_skill_home_is_refused() {
    let fx = fixture();
    let verify = fx.repo.join(".bee").join("verify");
    write(&verify.join("verify-app").join("SKILL.md"), "drive app\n");
    // The Claude skill home IS the verify root: source and target would be
    // one tree, and every apply would render a skill into itself.
    std::fs::create_dir_all(fx.repo.join(".claude")).unwrap();
    std::os::unix::fs::symlink(&verify, fx.repo.join(".claude").join("skills")).unwrap();

    let refusal = super::plan::verify_root_refusal(&fx.repo).unwrap();
    assert!(refusal.contains(".claude/skills"), "the refusal names the home: {refusal}");
    assert!(super::plan::compute_verify_skill_items(&fx.repo).is_empty());
}

// ── the default status display (4cac0774) ──────────────────────────────────
//
// This whole section is new coverage. Under the old opt-in shape the vendoring
// path had NO live test at all — the knowledge doc carried it as an Open Gap —
// so the cases below close that gap AND cover the reversal to opt-out. What
// they hold, in one line: bee writes the entry when the host declares none,
// never touches one the host already declares, never rewrites a settings file
// it cannot parse, and converges in a single apply.

/// Pin the fixture repo to a POSIX host. Onboarding skips the status display
/// on a PowerShell host (the script is bash), and with no `host_shell` the
/// running machine decides — so without this pin, a Windows run of this
/// section tests the skip instead of the display.
fn posix_fixture() -> Fixture {
    let fx = fixture();
    write(&fx.repo.join(".bee").join("config.json"), "{\"host_shell\":\"posix\"}\n");
    fx
}

fn written_command(fx: &Fixture) -> String {
    let settings: Value = serde_json::from_str(
        &std::fs::read_to_string(fx.repo.join(".claude").join("settings.json")).unwrap(),
    )
    .unwrap();
    settings["statusLine"]["command"].as_str().unwrap().to_string()
}

fn vendored(fx: &Fixture) -> PathBuf {
    fx.repo.join(".claude").join("statusline-command.sh")
}

fn settings_path(fx: &Fixture) -> PathBuf {
    fx.repo.join(".claude").join("settings.json")
}

/// The canonical bytes this fixture's source ships.
const FIXTURE_SCRIPT: &str = "#!/bin/sh\necho hi\n";

/// A host settings file declaring the project-level entry by hand — the shape
/// that WAS the only way in before 4cac0774.
fn write_project_level_entry(fx: &Fixture) {
    write(
        &settings_path(fx),
        &format!(
            "{}\n",
            jsjson::stringify_pretty(&json!({
                "statusLine": {
                    "type": "command",
                    "command": hooks_wiring::STATUSLINE_COMMAND
                }
            }))
        ),
    );
}

// ── the pre-existing gap: vendoring, drift, healing ────────────────────────

#[test]
fn a_project_level_entry_plans_the_copy_and_apply_lands_the_canonical_bytes() {
    let fx = posix_fixture();
    write_project_level_entry(&fx);

    let p = plan(&fx, &[]);
    assert_eq!(
        paths_for(&p, "plan", "copy_statusline"),
        vec![".claude/statusline-command.sh".to_string()]
    );
    // The host already declares the entry, so nothing writes to settings.json.
    assert!(!actions(&p, "plan").contains(&"merge_statusline_settings".to_string()));

    apply(&fx, &[]);
    assert_eq!(std::fs::read_to_string(vendored(&fx)).unwrap(), FIXTURE_SCRIPT);
}

#[test]
fn a_drifted_vendored_script_is_replanned_and_healed() {
    let fx = posix_fixture();
    write_project_level_entry(&fx);
    apply(&fx, &[]);

    write(&vendored(&fx), "#!/bin/sh\necho LOCAL EDIT\n");
    assert_eq!(
        paths_for(&plan(&fx, &[]), "plan", "copy_statusline"),
        vec![".claude/statusline-command.sh".to_string()]
    );

    apply(&fx, &[]);
    assert_eq!(std::fs::read_to_string(vendored(&fx)).unwrap(), FIXTURE_SCRIPT);
}

#[test]
fn a_deleted_vendored_script_with_the_entry_present_is_replanned() {
    let fx = posix_fixture();
    write_project_level_entry(&fx);
    apply(&fx, &[]);

    std::fs::remove_file(vendored(&fx)).unwrap();
    assert_eq!(
        paths_for(&plan(&fx, &[]), "plan", "copy_statusline"),
        vec![".claude/statusline-command.sh".to_string()]
    );
    apply(&fx, &[]);
    assert!(vendored(&fx).exists());
}

#[test]
fn the_managed_ledger_carries_the_statusline_hash_only_when_bee_owns_the_entry() {
    let owned = posix_fixture();
    apply(&owned, &[]);
    let ledger: Value = serde_json::from_str(
        &std::fs::read_to_string(owned.repo.join(".bee").join("onboarding.json")).unwrap(),
    )
    .unwrap();
    assert!(ledger["managed"]["statusline"]["statusline-command.sh"].is_string());

    let disowned = posix_fixture();
    apply(&disowned, &["--no-statusline"]);
    let ledger: Value = serde_json::from_str(
        &std::fs::read_to_string(disowned.repo.join(".bee").join("onboarding.json")).unwrap(),
    )
    .unwrap();
    assert!(ledger["managed"].get("statusline").is_none());
}

// ── new to opt-out: the write itself ───────────────────────────────────────

#[test]
fn an_absent_settings_file_is_created_with_the_canonical_entry_and_the_script() {
    let fx = posix_fixture();
    assert!(!settings_path(&fx).exists());

    let acts = actions(&plan(&fx, &[]), "plan");
    assert!(acts.contains(&"merge_statusline_settings".to_string()));
    assert!(acts.contains(&"copy_statusline".to_string()));

    apply(&fx, &[]);
    assert_eq!(written_command(&fx), hooks_wiring::STATUSLINE_COMMAND);
    assert_eq!(std::fs::read_to_string(vendored(&fx)).unwrap(), FIXTURE_SCRIPT);
    // Nothing to back up: there was no file before.
    assert!(!fx.repo.join(".claude").join("settings.json.bak").exists());
}

/// THE ROUND TRIP. If the command bee writes does not read back as a
/// project-level entry, the host silently un-adopts on the very next run: no
/// re-vendoring, the managed key dropped, and nothing anywhere says so.
#[test]
fn the_entry_bee_writes_reads_back_as_a_project_level_entry() {
    let fx = posix_fixture();
    apply(&fx, &[]);
    assert!(
        hooks_wiring::statusline_opt_in(&fx.repo),
        "the written command must satisfy the detector: {}",
        written_command(&fx)
    );
    assert!(!hooks_wiring::statusline_settings_absent(&fx.repo));
}

#[test]
fn a_settings_file_without_the_key_gains_exactly_one_key_and_keeps_the_rest() {
    let fx = posix_fixture();
    write(
        &settings_path(&fx),
        "{\n  \"model\": \"opus\",\n  \"permissions\": { \"allow\": [\"Bash(ls:*)\"] }\n}\n",
    );
    let original = std::fs::read_to_string(settings_path(&fx)).unwrap();

    apply(&fx, &[]);

    let settings: Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path(&fx)).unwrap()).unwrap();
    let keys: Vec<&str> = settings.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    assert_eq!(keys, vec!["model", "permissions", "statusLine"], "one key added, in place");
    assert_eq!(settings["model"], "opus");
    assert_eq!(settings["permissions"]["allow"][0], "Bash(ls:*)");
    // The file existed, so a backup was taken — and it holds the ORIGINAL.
    let bak = std::fs::read_to_string(fx.repo.join(".claude").join("settings.json.bak")).unwrap();
    assert_eq!(bak, original);
    // 2-space JSON with a trailing newline, like every other settings write.
    assert!(std::fs::read_to_string(settings_path(&fx)).unwrap().ends_with("}\n"));
}

#[test]
fn apply_then_replan_is_a_no_op_which_is_what_the_installer_recheck_asserts() {
    let fx = posix_fixture();
    apply(&fx, &[]);
    let p = plan(&fx, &[]);
    assert_eq!(p["status"], "up_to_date");
    assert_eq!(p["plan"].as_array().unwrap().len(), 0);
}

// ── new to opt-out: everything that must NOT happen ────────────────────────

#[test]
fn an_existing_project_level_entry_is_preserved_while_the_worktree_tools_are_added() {
    let fx = posix_fixture();
    write_project_level_entry(&fx);
    let original = std::fs::read_to_string(settings_path(&fx)).unwrap();

    apply(&fx, &[]);

    let before: Value = serde_json::from_str(&original).unwrap();
    let v: Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path(&fx)).unwrap()).unwrap();
    assert_eq!(v["statusLine"], before["statusLine"]);
    assert_eq!(v["permissions"]["allow"], json!(["EnterWorktree", "ExitWorktree"]));
    assert_eq!(
        std::fs::read_to_string(fx.repo.join(".claude").join("settings.json.bak")).unwrap(),
        original
    );
    // The script IS still vendored — the host asked for it.
    assert!(vendored(&fx).exists());
}

#[test]
fn an_existing_user_level_entry_is_preference_and_nothing_is_vendored() {
    let fx = posix_fixture();
    write(
        &settings_path(&fx),
        "{\n  \"statusLine\": { \"type\": \"command\", \"command\": \"bash ~/.claude/mine.sh\" }\n}\n",
    );
    let original: Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path(&fx)).unwrap()).unwrap();

    let acts = actions(&plan(&fx, &[]), "plan");
    assert!(!acts.contains(&"merge_statusline_settings".to_string()));
    assert!(!acts.contains(&"copy_statusline".to_string()));

    apply(&fx, &[]);
    let v: Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path(&fx)).unwrap()).unwrap();
    assert_eq!(v["statusLine"], original["statusLine"], "a user-level entry is a preference");
    assert!(!vendored(&fx).exists());
}

#[test]
fn a_non_object_statusline_value_is_preference_too() {
    // A present key of ANY shape is a declared preference.
    for body in [
        "{\n  \"statusLine\": \"my-line\"\n}\n",
        "{\n  \"statusLine\": null\n}\n",
        "{\n  \"statusLine\": false\n}\n",
    ] {
        let fx = posix_fixture();
        write(&settings_path(&fx), body);
        let original: Value = serde_json::from_str(body).unwrap();
        apply(&fx, &[]);
        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(settings_path(&fx)).unwrap()).unwrap();
        assert_eq!(v["statusLine"], original["statusLine"], "left alone: {body}");
        assert!(!vendored(&fx).exists(), "nothing vendored: {body}");
    }
}

/// The case that decides whether a default is safe to ship: a host whose
/// settings file does not parse is left alone AND the run still succeeds.
/// A refusal here would turn one broken file into a blocked install.
#[test]
fn an_unparseable_settings_file_is_left_alone_and_never_blocks_the_run() {
    let fx = posix_fixture();
    let broken = "{ this is not json";
    write(&settings_path(&fx), broken);

    let acts = actions(&plan(&fx, &[]), "plan");
    assert!(!acts.contains(&"merge_statusline_settings".to_string()));
    assert!(!acts.contains(&"copy_statusline".to_string()));

    let a = apply(&fx, &[]);
    assert_ne!(a["status"], "blocked_hooks_merge", "a broken host file must not block a default");
    assert_eq!(std::fs::read_to_string(settings_path(&fx)).unwrap(), broken);
    assert!(!vendored(&fx).exists());
}

#[test]
fn a_settings_file_whose_root_is_not_an_object_is_left_alone() {
    let fx = posix_fixture();
    write(&settings_path(&fx), "[]\n");

    let acts = actions(&plan(&fx, &[]), "plan");
    assert!(!acts.contains(&"merge_statusline_settings".to_string()));
    assert!(!acts.contains(&"copy_statusline".to_string()));

    apply(&fx, &[]);
    assert_eq!(std::fs::read_to_string(settings_path(&fx)).unwrap(), "[]\n");
}

// ── the two ways out ───────────────────────────────────────────────────────

#[test]
fn no_statusline_writes_no_status_entry_vendors_nothing_and_still_converges() {
    let fx = posix_fixture();
    let a = apply(&fx, &["--no-statusline"]);
    assert_ne!(a["status"], "blocked_hooks_merge");
    let v: Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path(&fx)).unwrap()).unwrap();
    assert!(v.get("statusLine").is_none());
    assert!(!vendored(&fx).exists());

    let p = plan(&fx, &["--no-statusline"]);
    assert_eq!(p["status"], "up_to_date");
    assert_eq!(p["plan"].as_array().unwrap().len(), 0);
}

#[test]
fn no_statusline_on_a_host_that_already_carries_the_entry_leaves_it_and_the_script_alone() {
    let fx = posix_fixture();
    write_project_level_entry(&fx);
    let original: Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path(&fx)).unwrap()).unwrap();

    apply(&fx, &["--no-statusline"]);

    let v: Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path(&fx)).unwrap()).unwrap();
    assert_eq!(v["statusLine"], original["statusLine"]);
    assert!(!vendored(&fx).exists());
    assert_eq!(plan(&fx, &["--no-statusline"])["status"], "up_to_date");
}

#[test]
fn the_project_config_opt_out_behaves_exactly_like_the_flag() {
    let fx = fixture();
    write(
        &fx.repo.join(".bee").join("config.json"),
        "{\n  \"host_shell\": \"posix\",\n  \"statusline\": false\n}\n",
    );

    apply(&fx, &[]);

    let v: Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path(&fx)).unwrap()).unwrap();
    assert!(v.get("statusLine").is_none());
    assert!(!vendored(&fx).exists());
    let ledger: Value = serde_json::from_str(
        &std::fs::read_to_string(fx.repo.join(".bee").join("onboarding.json")).unwrap(),
    )
    .unwrap();
    assert!(ledger["managed"].get("statusline").is_none());
}

// ── the two writers of one file ────────────────────────────────────────────

/// Backup-of-a-backup. Both the hooks merge and the statusline write target
/// `.claude/settings.json`; if each takes its own `.bak`, the second copies the
/// already-modified file over the pristine one and `scripts/install.sh`'s
/// promise that `.bak` restores the pre-bee state is quietly broken.
#[test]
fn repo_hooks_and_the_statusline_default_share_one_backup_holding_the_original() {
    let fx = posix_fixture();
    let original = "{\n  \"model\": \"opus\"\n}\n";
    write(&settings_path(&fx), original);

    apply(&fx, &["--repo-hooks"]);

    let bak = std::fs::read_to_string(fx.repo.join(".claude").join("settings.json.bak")).unwrap();
    assert_eq!(bak, original, ".bak must hold the PRE-BEE file, not an intermediate one");

    let settings: Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path(&fx)).unwrap()).unwrap();
    let keys: Vec<&str> = settings.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    assert_eq!(keys, vec!["model", "hooks", "statusLine", "permissions"], "all three writers landed");
}

#[test]
fn a_malformed_settings_file_blocks_the_hooks_merge_without_vendoring_anything() {
    let fx = posix_fixture();
    write(&settings_path(&fx), r#"{"model": "opus", "hooks": {"#);

    let a = apply(&fx, &["--repo-hooks"]);
    assert_eq!(a["status"], "blocked_hooks_merge");
    // Zero mutations means the script too — the statusline step must not have
    // slipped a file in before the refusal.
    assert!(!vendored(&fx).exists());
}

/// D1 (pi-stage-dispatch psd-4):
/// A source with claude, codex, and pi blocks renders codex and pi text into the
/// agents root, only claude text into the claude root, and no pi text into the
/// opencode root. Marker-free files stay byte-identical everywhere.
#[test]
fn skills_with_claude_codex_and_pi_blocks_render_per_d1_into_managed_roots() {
    let fx = fixture();
    let src = fx.root.join("skills").join("bee-test");
    write(
        &src.join("SKILL.md"),
        "preamble\n<!-- bee:only claude -->\nclaude block text\n<!-- bee:end -->\n<!-- bee:only codex -->\ncodex block text\n<!-- bee:end -->\n<!-- bee:only pi -->\npi block text\n<!-- bee:end -->\npostscript\n",
    );
    write(
        &src.join("references").join("plain.md"),
        "marker-free plain text across all targets\n",
    );

    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");

    // 1. .agents/skills (repo-agents): keeps codex and pi blocks, strips claude
    let agents_body = std::fs::read_to_string(
        fx.repo.join(".agents").join("skills").join("bee-test").join("SKILL.md"),
    )
    .unwrap();
    assert!(agents_body.contains("codex block text"), "agents root must keep codex block");
    assert!(agents_body.contains("pi block text"), "agents root must keep pi block");
    assert!(!agents_body.contains("claude block text"), "agents root must strip claude block");
    assert!(!agents_body.contains("bee:only"), "markers must be stripped");

    // 2. .claude/skills (repo-claude): keeps claude, strips codex and pi
    let claude_body = std::fs::read_to_string(
        fx.repo.join(".claude").join("skills").join("bee-test").join("SKILL.md"),
    )
    .unwrap();
    assert!(claude_body.contains("claude block text"), "claude root must keep claude block");
    assert!(!claude_body.contains("codex block text"), "claude root must strip codex block");
    assert!(!claude_body.contains("pi block text"), "claude root must strip pi block");
    assert!(!claude_body.contains("bee:only"), "markers must be stripped");

    // 3. .opencode/skills (repo-opencode): strips claude, codex, and pi
    let opencode_body = std::fs::read_to_string(
        fx.repo.join(".opencode").join("skills").join("bee-test").join("SKILL.md"),
    )
    .unwrap();
    assert!(!opencode_body.contains("claude block text"), "opencode root must strip claude block");
    assert!(!opencode_body.contains("codex block text"), "opencode root must strip codex block");
    assert!(!opencode_body.contains("pi block text"), "opencode root must strip pi block");
    assert!(!opencode_body.contains("bee:only"), "markers must be stripped");

    // 4. Marker-free file is byte-identical in all three managed roots
    let src_plain = std::fs::read(src.join("references").join("plain.md")).unwrap();
    let agents_plain = std::fs::read(
        fx.repo.join(".agents").join("skills").join("bee-test").join("references").join("plain.md"),
    )
    .unwrap();
    let claude_plain = std::fs::read(
        fx.repo.join(".claude").join("skills").join("bee-test").join("references").join("plain.md"),
    )
    .unwrap();
    let opencode_plain = std::fs::read(
        fx.repo.join(".opencode").join("skills").join("bee-test").join("references").join("plain.md"),
    )
    .unwrap();
    assert_eq!(agents_plain, src_plain, "marker-free file must be byte-identical in agents root");
    assert_eq!(claude_plain, src_plain, "marker-free file must be byte-identical in claude root");
    assert_eq!(opencode_plain, src_plain, "marker-free file must be byte-identical in opencode root");

    // 5. Sidecar schema and target_runtime
    let agents_sidecar: Value = serde_json::from_str(
        &std::fs::read_to_string(
            fx.repo.join(".agents").join("skills").join(super::templates::RENDER_SIDECAR),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(agents_sidecar["schema"], "bee-render/2");
    assert_eq!(agents_sidecar["target_runtime"], "codex");
}

fn allow_list(fx: &Fixture) -> Vec<String> {
    let settings: Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path(fx)).unwrap()).unwrap();
    settings["permissions"]["allow"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_virgin_settings_file_gains_both_worktree_tools_in_permissions_allow() {
    let fx = posix_fixture();
    assert!(!settings_path(&fx).exists());

    let acts = actions(&plan(&fx, &[]), "plan");
    assert!(acts.contains(&"merge_worktree_tool_allow".to_string()));

    apply(&fx, &[]);

    assert_eq!(allow_list(&fx), vec!["EnterWorktree".to_string(), "ExitWorktree".to_string()]);
}

#[test]
fn an_existing_allow_list_is_appended_add_only_and_keeps_foreign_keys() {
    let fx = posix_fixture();
    write(
        &settings_path(&fx),
        "{\n  \"model\": \"opus\",\n  \"permissions\": {\n    \"allow\": [\"Bash(ls:*)\"],\n    \"deny\": [\"Read\"]\n  },\n  \"zeta\": 1\n}\n",
    );

    apply(&fx, &["--no-statusline"]);

    let v: Value =
        serde_json::from_str(&std::fs::read_to_string(settings_path(&fx)).unwrap()).unwrap();
    let keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    assert_eq!(keys, vec!["model", "permissions", "zeta"], "top-level keys untouched");
    assert_eq!(v["model"], "opus");
    assert_eq!(v["zeta"], 1);
    assert_eq!(v["permissions"]["deny"], json!(["Read"]), "deny survives");
    assert_eq!(
        v["permissions"]["allow"],
        json!(["Bash(ls:*)", "EnterWorktree", "ExitWorktree"]),
        "only the missing names are appended, in place"
    );
}

#[test]
fn settings_that_already_hold_both_names_plan_nothing_and_write_no_backup() {
    let fx = posix_fixture();
    write(
        &settings_path(&fx),
        "{\n  \"permissions\": { \"allow\": [\"EnterWorktree\", \"ExitWorktree\"] }\n}\n",
    );
    let original = std::fs::read_to_string(settings_path(&fx)).unwrap();

    let acts = actions(&plan(&fx, &["--no-statusline"]), "plan");
    assert!(
        !acts.contains(&"merge_worktree_tool_allow".to_string()),
        "both names present must not replan"
    );
    assert!(!actions(&plan(&fx, &[]), "plan").contains(&"merge_worktree_tool_allow".to_string()));

    apply(&fx, &["--no-statusline"]);
    assert_eq!(std::fs::read_to_string(settings_path(&fx)).unwrap(), original);
    assert!(!fx.repo.join(".claude").join("settings.json.bak").exists());
}

#[test]
fn an_unparseable_settings_file_is_never_written_by_the_worktree_projection() {
    let fx = posix_fixture();
    let broken = "{ this is not json";
    write(&settings_path(&fx), broken);

    let acts = actions(&plan(&fx, &["--no-statusline"]), "plan");
    assert!(!acts.contains(&"merge_worktree_tool_allow".to_string()));

    let a = apply(&fx, &["--no-statusline"]);
    assert_ne!(a["status"], "blocked_hooks_merge");
    assert_eq!(std::fs::read_to_string(settings_path(&fx)).unwrap(), broken);
}

#[test]
fn an_unmergeable_permissions_shape_is_left_alone_and_still_converges() {
    for body in [
        "{\n  \"permissions\": \"not-an-object\"\n}\n",
        "{\n  \"permissions\": { \"allow\": \"not-an-array\" }\n}\n",
    ] {
        let fx = posix_fixture();
        write(&settings_path(&fx), body);

        let acts = actions(&plan(&fx, &["--no-statusline"]), "plan");
        assert!(!acts.contains(&"merge_worktree_tool_allow".to_string()), "{body}");

        apply(&fx, &["--no-statusline"]);
        assert_eq!(std::fs::read_to_string(settings_path(&fx)).unwrap(), body);
        assert_eq!(plan(&fx, &["--no-statusline"])["status"], "up_to_date", "{body}");
    }
}

#[test]
fn a_second_run_after_apply_reports_up_to_date_for_the_worktree_projection() {
    let fx = posix_fixture();
    apply(&fx, &[]);

    let p = plan(&fx, &[]);
    assert_eq!(p["status"], "up_to_date");
    assert!(
        !actions(&p, "plan").contains(&"merge_worktree_tool_allow".to_string()),
        "a settled projection replans nothing"
    );
    let a = apply(&fx, &[]);
    assert_eq!(a["recheck"], "up_to_date");
}
