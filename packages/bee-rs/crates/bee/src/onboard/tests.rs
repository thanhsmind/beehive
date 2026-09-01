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
        "create_runtime_file", // .bee/config-sample.json (config-sample-herding D3)
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
    write(&fx.root.join(".pi").join("extensions").join("bee-guard.ts"), "// pi guard v1\n");

    let p = plan(&fx, &[]);
    assert_eq!(paths_for(&p, "plan", "copy_pi_extension"), vec![".pi/extensions/bee-guard.ts"]);

    let a = apply(&fx, &[]);
    assert_eq!(a["status"], "applied");
    assert_eq!(
        std::fs::read_to_string(fx.repo.join(".pi").join("extensions").join("bee-guard.ts"))
            .unwrap(),
        "// pi guard v1\n"
    );

    // IDEMPOTENT: a settled repo plans and applies nothing further.
    let p2 = plan(&fx, &[]);
    assert!(!actions(&p2, "plan").contains(&"copy_pi_extension".to_string()));
    let a2 = apply(&fx, &[]);
    assert!(!actions(&a2, "applied").contains(&"copy_pi_extension".to_string()));

    // DRIFT: a hand-edited extension file is repaired on the next apply — the
    // belt is an ENFORCEMENT surface, so a tampered copy is never left in
    // place, exactly as the OpenCode plugin above.
    write(&fx.repo.join(".pi").join("extensions").join("bee-guard.ts"), "tampered\n");
    let p3 = plan(&fx, &[]);
    assert_eq!(paths_for(&p3, "plan", "copy_pi_extension"), vec![".pi/extensions/bee-guard.ts"]);
    apply(&fx, &[]);
    assert_eq!(
        std::fs::read_to_string(fx.repo.join(".pi").join("extensions").join("bee-guard.ts"))
            .unwrap(),
        "// pi guard v1\n"
    );
}

#[test]
fn a_source_checkout_with_no_pi_extension_plans_nothing_for_it() {
    // The ordinary fixture() never writes .pi/extensions/ under fx.root —
    // this is the "not every source checkout carries it yet" case, and it
    // must stay silent rather than erroring or fabricating a directory.
    let fx = fixture();
    let p = plan(&fx, &[]);
    assert!(!actions(&p, "plan").contains(&"copy_pi_extension".to_string()));
    assert!(!fx.repo.join(".pi").join("extensions").exists());
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
    // 1 foreign + bee's three PreToolUse rows (write guard, model guard,
    // activity).
    assert_eq!(settings["hooks"]["PreToolUse"].as_array().unwrap().len(), 4);
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
