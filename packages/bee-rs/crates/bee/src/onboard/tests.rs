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

    // Release identity tuple: runtime marker + both plugin manifests.
    write(
        &root.join("packages").join("bee").join("lib").join("state.mjs"),
        &format!("export const BEE_VERSION = '{VERSION}';\n"),
    );
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
    // The engine's own script — Engine::locate's marker, and the file whose
    // presence proves this really is a bee checkout.
    write(&root.join("packages").join("bee").join("scripts").join("onboard_bee.mjs"), "// engine\n");

    // A second lib module, so drift can be exercised WITHOUT touching
    // state.mjs — which is also the host's version marker, and tampering it
    // legitimately blocks the run (see version_marker_tamper_blocks below).
    write(&root.join("packages").join("bee").join("lib").join("cells.mjs"), "// cells\n");
    write(&root.join("packages").join("bee").join("AGENTS.block.md"), "# bee\n\nrules here\n\n\n");
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
        "--runtime must be claude, codex, or both (got: rust)"
    );
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
        "create_dir",
        "create_dir",
        "copy_helper",
    ];
    assert_eq!(acts[..expected_head.len()], expected_head[..]);
    assert!(acts.contains(&"create_gitignore_block".to_string()));
    assert!(acts.contains(&"create_claude_md".to_string()));
    assert!(acts.contains(&"write_onboarding".to_string()));
    assert!(acts.contains(&"sync_skill".to_string()));

    // The nested expertise pattern file is planned by its POSIX-relative name.
    let expertise = paths_for(&p, "plan", "copy_expertise");
    assert_eq!(
        expertise,
        vec![
            ".bee/expertise/tests.md".to_string(),
            ".bee/expertise/tests/patterns/differential-testing.md".to_string()
        ]
    );

    // Both in-repo skill roots are resolved, in stable order, fresh.
    let targets = p["skills"]["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0]["kind"], "repo-claude");
    assert_eq!(targets[0]["mode"], "fresh");
    assert_eq!(targets[1]["kind"], "repo-agents");
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
    assert!(fx.repo.join(".bee").join("bin").join("lib").join("state.mjs").exists());
    assert!(fx.repo.join(".bee").join("bin").join("prompts").join("worker-cell.md").exists());
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

    // Skills mirrored into both roots, each with its stamp + sidecar.
    for rel in [".claude/skills", ".agents/skills"] {
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

    // The ledger.
    let ledger: Value = serde_json::from_str(
        &std::fs::read_to_string(fx.repo.join(".bee").join("onboarding.json")).unwrap(),
    )
    .unwrap();
    let keys: Vec<&str> = ledger.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    assert_eq!(
        keys,
        vec!["schema_version", "bee_version", "managed", "agents_sync", "created_at", "updated_at"]
    );
    assert_eq!(ledger["schema_version"], "1.0");
    assert_eq!(ledger["bee_version"], VERSION);
    let managed_keys: Vec<&str> =
        ledger["managed"].as_object().unwrap().keys().map(|k| k.as_str()).collect();
    assert_eq!(
        managed_keys,
        vec!["agents_block", "gitignore_block", "helpers", "lib", "expertise", "prompts"]
    );
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
    // .bee/bin/lib/state.mjs is the host_helpers version marker: a tree that
    // EXISTS but whose version cannot be read is "unknown" — refuse, and
    // never forceable (D3 / review P1-1).
    write(&fx.repo.join(".bee").join("bin").join("lib").join("state.mjs"), "tampered\n");
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
        assert_eq!(
            std::fs::read_to_string(fx.repo.join(".bee").join("bin").join("lib").join("state.mjs")).unwrap(),
            "tampered\n",
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

    let p = plan(&fx, &[]);
    let acts = actions(&p, "plan");
    assert!(acts.contains(&"append_agents_block".to_string()));
    assert!(!acts.contains(&"propose_agents_header".to_string()), "existing prose suppresses it");
    assert!(!acts.contains(&"create_claude_md".to_string()));
    assert!(!acts.contains(&"append_claude_md_import".to_string()));
    assert!(acts.contains(&"append_gitignore_block".to_string()));

    apply(&fx, &[]);
    let agents = std::fs::read_to_string(fx.repo.join("AGENTS.md")).unwrap();
    assert!(agents.starts_with("# repo\n\nMy own project prose.\n\n<!-- BEE:START -->"));
    let gi = std::fs::read_to_string(fx.repo.join(".gitignore")).unwrap();
    assert!(gi.starts_with("node_modules/\n\n# BEE:START\n"));
    assert_eq!(plan(&fx, &[])["plan"].as_array().unwrap().len(), 0);
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
    assert_eq!(settings["hooks"]["PreToolUse"].as_array().unwrap().len(), 3);
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
fn a_vendored_binary_flips_hook_wiring_without_stacking_duplicates() {
    let fx = fixture();
    apply(&fx, &["--repo-hooks"]);
    // Vendor the Rust binary, then re-onboard.
    write(&fx.repo.join(".bee").join("bin").join("bee.exe"), "MZ");
    let p = plan(&fx, &["--repo-hooks"]);
    assert!(actions(&p, "plan").contains(&"merge_repo_hook_settings".to_string()));
    apply(&fx, &["--repo-hooks"]);
    let settings: Value = serde_json::from_str(
        &std::fs::read_to_string(fx.repo.join(".claude").join("settings.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        settings["hooks"]["UserPromptSubmit"][0]["hooks"][0]["command"],
        "\"$CLAUDE_PROJECT_DIR\"/.bee/bin/bee.exe hook prompt-context"
    );
    assert_eq!(
        settings["hooks"]["UserPromptSubmit"].as_array().unwrap().len(),
        1,
        "the node-shaped entry is REPLACED, never stacked beside the binary one"
    );
    assert_eq!(plan(&fx, &["--repo-hooks"])["plan"].as_array().unwrap().len(), 0);
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
    // Age the source below the installed host runtime.
    for f in [
        fx.root.join("packages").join("bee").join("lib").join("state.mjs"),
    ] {
        write(&f, "export const BEE_VERSION = '1.0.0';\n");
    }
    write(&fx.root.join(".claude-plugin").join("plugin.json"), "{\"version\":\"1.0.0\"}");
    write(&fx.root.join(".codex-plugin").join("plugin.json"), "{\"version\":\"1.0.0\"}");

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
        std::fs::read_to_string(fx.repo.join(".bee").join("bin").join("lib").join("state.mjs")).unwrap(),
        format!("export const BEE_VERSION = '{VERSION}';\n")
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
        std::fs::read_to_string(fx.repo.join(".bee").join("bin").join("lib").join("state.mjs")).unwrap(),
        "export const BEE_VERSION = '1.0.0';\n"
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
