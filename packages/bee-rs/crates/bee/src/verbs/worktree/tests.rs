// Split out of the single 4.2k-line verbs/worktree.rs. Code unchanged; only module placement and item visibility moved.
//
// Moved verbatim out of the parent file's inline module, indentation
// and all: a moved inline module is the same child of the same parent,
// so no path changes, and the fixtures inside are raw strings whose
// leading whitespace is content.

// The parent module's own `use` block travels with the tests: they reach
// for names mod.rs no longer imports now that the code using them lives
// in sibling modules.
#![allow(unused_imports)]

use crate::registry::check_manifest_drift;
use crate::roots::{resolve_roots_core, Resolution};
use crate::verbs::reservations::{js_numberify, js_trim, now_iso, parse_flags, FlagV, Flags};
use crate::verbs::workspace_store as ws;
use crate::verbs::{emit_no_root_error, record_timing};
use crate::{jsjson, lock};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
use std::time::Instant;
    use super::*;

    fn map_of(pairs: &[(&str, Value)]) -> Map<String, Value> {
        let mut m = Map::new();
        for (k, v) in pairs {
            m.insert((*k).to_string(), v.clone());
        }
        m
    }

    /// The registry file Node writes: 2-space JSON + a trailing newline,
    /// insertion order preserved (pinned against writeGrantsFileAtomic).
    #[test]
    fn grants_file_bytes_match_node() {
        let tmp = tempfile::tempdir().unwrap();
        let store = tmp.path().join(".bee");
        let grants = map_of(&[("wt-b", json!(true)), ("wt-a", json!(true))]);
        write_grants_file_atomic(&store, &grants).unwrap();
        let text = std::fs::read_to_string(grants_file(&store)).unwrap();
        assert_eq!(text, "{\n  \"wt-b\": true,\n  \"wt-a\": true\n}\n");
        // The tmp file never survives a successful write.
        assert!(!store.join("runtime").join("worktree-grants.json.tmp").exists());
    }

    #[test]
    fn read_grants_strict_matches_node_for_the_shapes_it_serves() {
        let tmp = tempfile::tempdir().unwrap();
        let store = tmp.path().join(".bee");
        // Missing file -> {}.
        assert_eq!(read_grants_strict(&store), Some(Map::new()));
        std::fs::create_dir_all(store.join("runtime")).unwrap();
        // A parsed non-object -> {} (Node's `typeof parsed === 'object'`).
        std::fs::write(grants_file(&store), "5").unwrap();
        assert_eq!(read_grants_strict(&store), Some(Map::new()));
        std::fs::write(grants_file(&store), "null").unwrap();
        assert_eq!(read_grants_strict(&store), Some(Map::new()));
        // A real registry round-trips in file order.
        std::fs::write(grants_file(&store), "{\"b\":true,\"a\":false}").unwrap();
        let got = read_grants_strict(&store).unwrap();
        assert_eq!(got.keys().collect::<Vec<_>>(), vec!["b", "a"]);
        // Unparseable / array -> delegate.
        std::fs::write(grants_file(&store), "{oops").unwrap();
        assert_eq!(read_grants_strict(&store), None);
        std::fs::write(grants_file(&store), "[true]").unwrap();
        assert_eq!(read_grants_strict(&store), None);
    }

    /// bootstrapWorktreeStore's two shapes, including the idempotence rule:
    /// an existing state.json is never overwritten, and the creation identity
    /// is written BEFORE that early return (so an adopted worktree gets one).
    #[test]
    fn bootstrap_shapes_match_node() {
        let tmp = tempfile::tempdir().unwrap();
        let main_store = tmp.path().join("main").join(".bee");
        std::fs::create_dir_all(&main_store).unwrap();
        std::fs::write(main_store.join("onboarding.json"), "{\"bee_version\":\"x\"}").unwrap();
        let wt = tmp.path().join("wt-a");
        std::fs::create_dir_all(&wt).unwrap();

        let first = bootstrap_worktree_store(&wt, &main_store, "demo").unwrap();
        assert_eq!(first.get("created"), Some(&Value::Bool(true)));
        assert_eq!(
            first.keys().collect::<Vec<_>>(),
            vec!["created", "worktreeStoreRoot", "onboarding", "config", "identity", "state"]
        );
        assert_eq!(first["onboarding"]["copied"], Value::Bool(true));
        assert_eq!(first["config"]["copied"], Value::Bool(false));
        assert_eq!(first["config"]["reason"], json!("main store has no config.json"));
        assert_eq!(first["identity"]["written"], Value::Bool(true));
        let state = std::fs::read_to_string(wt.join(".bee").join("state.json")).unwrap();
        assert!(state.starts_with("{\n  \"schema_version\": \"1.0\",\n  \"phase\": \"idle\","));
        assert!(state.ends_with("}\n"));

        // Re-running never clobbers state.json or the creation identity.
        std::fs::write(wt.join(".bee").join("state.json"), "{\"phase\":\"swarming\"}").unwrap();
        let second = bootstrap_worktree_store(&wt, &main_store, "renamed").unwrap();
        assert_eq!(second.get("created"), Some(&Value::Bool(false)));
        assert_eq!(second["reason"], json!("state.json already exists"));
        assert_eq!(
            second["identity"]["reason"],
            json!("creation identity already recorded — never overwritten")
        );
        assert_eq!(second["onboarding"]["reason"], json!("onboarding.json already exists"));
        let identity =
            std::fs::read_to_string(wt.join(".bee").join("runtime").join("worktree-identity.json"))
                .unwrap();
        assert!(identity.contains("\"feature\": \"demo\""));
        assert_eq!(
            std::fs::read_to_string(wt.join(".bee").join("state.json")).unwrap(),
            "{\"phase\":\"swarming\"}"
        );
    }

    /// p-9c48a67c: `bootstrap_worktree_store` copies only the granted
    /// feature's cells — a fixture main store carries feature A's and B's
    /// cells (live and archived); bootstrapping for "a" must leave the
    /// island holding A's alone, B's entirely absent, and must never mutate
    /// the main store itself.
    #[test]
    fn bootstrap_copies_only_the_granted_features_cells() {
        let tmp = tempfile::tempdir().unwrap();
        let main_store = tmp.path().join("main").join(".bee");
        let main_cells = main_store.join("cells");
        std::fs::create_dir_all(&main_cells).unwrap();
        std::fs::write(main_cells.join("a-1.json"), "{\"id\":\"a-1\",\"feature\":\"a\",\"status\":\"open\"}")
            .unwrap();
        std::fs::write(
            main_cells.join("b-1.json"),
            "{\"id\":\"b-1\",\"feature\":\"b\",\"status\":\"claimed\"}",
        )
        .unwrap();
        let main_archive_a = main_cells.join("archive").join("a");
        let main_archive_b = main_cells.join("archive").join("b");
        std::fs::create_dir_all(&main_archive_a).unwrap();
        std::fs::create_dir_all(&main_archive_b).unwrap();
        std::fs::write(main_archive_a.join("a-0.json"), "{\"id\":\"a-0\",\"feature\":\"a\",\"status\":\"capped\"}")
            .unwrap();
        std::fs::write(main_archive_b.join("b-0.json"), "{\"id\":\"b-0\",\"feature\":\"b\",\"status\":\"capped\"}")
            .unwrap();
        std::fs::write(main_cells.join("archive").join("summary.json"), "{\"a\":{\"capped\":1}}").unwrap();
        // Snapshot the main store's bytes before bootstrapping.
        let before: Vec<(PathBuf, Vec<u8>)> = [
            main_cells.join("a-1.json"),
            main_cells.join("b-1.json"),
            main_archive_a.join("a-0.json"),
            main_archive_b.join("b-0.json"),
            main_cells.join("archive").join("summary.json"),
        ]
        .into_iter()
        .map(|p| (p.clone(), std::fs::read(&p).unwrap()))
        .collect();

        let wt = tmp.path().join("wt-a");
        std::fs::create_dir_all(&wt).unwrap();
        let result = bootstrap_worktree_store(&wt, &main_store, "a").unwrap();
        assert_eq!(result.get("created"), Some(&Value::Bool(true)));

        let wt_cells = wt.join(".bee").join("cells");
        assert!(wt_cells.join("a-1.json").exists());
        assert!(!wt_cells.join("b-1.json").exists(), "feature b's live cell must be absent from the island");
        assert!(wt_cells.join("archive").join("a").join("a-0.json").exists());
        assert!(
            !wt_cells.join("archive").join("b").exists(),
            "feature b's archive dir must be absent from the island"
        );

        // Main store byte-identical after.
        for (path, bytes) in before {
            assert_eq!(std::fs::read(&path).unwrap(), bytes, "main store mutated at {}", path.display());
        }
    }

    /// Tiny git helpers for the ips-1 tracked-ness fixtures below — no
    /// `bee` CLI involved, just enough `git` to give `sync_worktree_cells`'s
    /// `git ls-files` a real index to consult.
    fn git_init(dir: &Path) {
        assert!(std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["init", "-q"])
            .status()
            .unwrap()
            .success());
    }

    fn git_add_commit(dir: &Path, paths: &[&str]) {
        assert!(std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .arg("add")
            .args(paths)
            .status()
            .unwrap()
            .success());
        assert!(std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.email=bee@example.com",
                "-c",
                "user.name=bee",
                "commit",
                "-q",
                "-m",
                "fixture"
            ])
            .status()
            .unwrap()
            .success());
    }

    /// ips-1 P1 fix, case (a): the WORKTREE's own `.bee/cells` already holds
    /// a wholesale checkout of every feature's cells (`git worktree add`
    /// checks out `.bee/cells` in full because it's git-tracked) — this is
    /// the actual production bug shape, not just an absent-and-filled one.
    /// A TRACKED foreign-feature cell (main's committed history riding along
    /// in this checkout) must survive the prune; only an UNTRACKED foreign
    /// stray is removed.
    #[test]
    fn bootstrap_prunes_only_untracked_foreign_cells_already_checked_out_in_the_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let main_store = tmp.path().join("main").join(".bee");
        std::fs::create_dir_all(main_store.join("cells")).unwrap();

        let wt = tmp.path().join("wt-a");
        let wt_cells = wt.join(".bee").join("cells");
        std::fs::create_dir_all(&wt_cells).unwrap();
        std::fs::write(wt_cells.join("a-1.json"), "{\"id\":\"a-1\",\"feature\":\"a\",\"status\":\"open\"}")
            .unwrap();
        // A foreign cell that IS tracked — checked out by `git worktree add`,
        // never staged for deletion.
        std::fs::write(wt_cells.join("b-1.json"), "{\"id\":\"b-1\",\"feature\":\"b\",\"status\":\"open\"}")
            .unwrap();
        let wt_archive_b = wt_cells.join("archive").join("b");
        std::fs::create_dir_all(&wt_archive_b).unwrap();
        std::fs::write(wt_archive_b.join("b-0.json"), "{\"id\":\"b-0\",\"feature\":\"b\",\"status\":\"capped\"}")
            .unwrap();
        git_init(&wt);
        git_add_commit(&wt, &[".bee/cells/b-1.json", ".bee/cells/archive/b/b-0.json"]);

        // A foreign cell that is NOT tracked — never committed, a plain
        // stray sitting next to the tracked ones. Written after the commit
        // so `git ls-files` never saw it.
        std::fs::write(wt_cells.join("c-1.json"), "{\"id\":\"c-1\",\"feature\":\"c\",\"status\":\"open\"}")
            .unwrap();
        let wt_archive_c = wt_cells.join("archive").join("c");
        std::fs::create_dir_all(&wt_archive_c).unwrap();
        std::fs::write(wt_archive_c.join("c-0.json"), "{\"id\":\"c-0\",\"feature\":\"c\",\"status\":\"capped\"}")
            .unwrap();

        bootstrap_worktree_store(&wt, &main_store, "a").unwrap();

        assert!(wt_cells.join("a-1.json").exists(), "the granted feature's already-checked-out cell stays");
        assert!(wt_cells.join("b-1.json").exists(), "a TRACKED foreign cell stays — it is main's history");
        assert!(
            wt_cells.join("archive").join("b").join("b-0.json").exists(),
            "a TRACKED foreign archive file stays"
        );
        assert!(!wt_cells.join("c-1.json").exists(), "an UNTRACKED foreign cell is still pruned");
        assert!(
            !wt_cells.join("archive").join("c").exists(),
            "an UNTRACKED foreign archive dir is still pruned"
        );
    }

    /// ips-1 P1 pin: the exact failure mode caught pre-merge — a fresh
    /// island bootstrapped from a real `git worktree add` checkout must
    /// leave `git status --porcelain` EMPTY. Before this fix, the prune arm
    /// deleted the foreign feature's tracked `.bee/cells` files unconditionally,
    /// manufacturing tracked deletions that a later `worktree merge` would
    /// replay onto main and wipe the cell archive.
    #[test]
    fn bootstrap_from_a_real_worktree_checkout_leaves_git_status_clean() {
        let tmp = tempfile::tempdir().unwrap();
        let main = tmp.path().join("main");
        std::fs::create_dir_all(&main).unwrap();
        git_init(&main);
        // Runtime-only bee paths are gitignored in the real repo too — same
        // shape here so bootstrap's OWN writes (state.json, the creation
        // identity under runtime/) don't show up as untracked noise and
        // mask the deletion this test exists to catch.
        std::fs::write(main.join(".gitignore"), ".bee/state.json\n.bee/runtime/\n").unwrap();

        let main_cells = main.join(".bee").join("cells");
        std::fs::create_dir_all(main_cells.join("archive").join("b")).unwrap();
        std::fs::write(main_cells.join("a-1.json"), "{\"id\":\"a-1\",\"feature\":\"a\",\"status\":\"open\"}")
            .unwrap();
        std::fs::write(main_cells.join("b-1.json"), "{\"id\":\"b-1\",\"feature\":\"b\",\"status\":\"open\"}")
            .unwrap();
        std::fs::write(
            main_cells.join("archive").join("b").join("b-0.json"),
            "{\"id\":\"b-0\",\"feature\":\"b\",\"status\":\"capped\"}",
        )
        .unwrap();
        git_add_commit(&main, &["."]);

        let wt = tmp.path().join("wt-a");
        assert!(std::process::Command::new("git")
            .arg("-C")
            .arg(&main)
            .args(["worktree", "add", "-q", "-b", "wt-a-branch"])
            .arg(&wt)
            .status()
            .unwrap()
            .success());

        let main_store = main.join(".bee");
        bootstrap_worktree_store(&wt, &main_store, "a").unwrap();

        let wt_cells = wt.join(".bee").join("cells");
        assert!(wt_cells.join("a-1.json").exists(), "granted feature's cell stays");
        assert!(wt_cells.join("b-1.json").exists(), "tracked foreign cell survives the prune");
        assert!(
            wt_cells.join("archive").join("b").join("b-0.json").exists(),
            "tracked foreign archive file survives the prune"
        );

        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(&wt)
            .args(["status", "--porcelain"])
            .output()
            .unwrap();
        assert!(status.status.success());
        assert!(
            status.stdout.is_empty(),
            "bootstrap must manufacture zero git changes in the island, got: {}",
            String::from_utf8_lossy(&status.stdout)
        );
    }

    /// git unavailable / not a repo: the prune arm fails safe and deletes
    /// nothing, foreign or not — the fill arm and the zero-cells case are
    /// untouched by this switch (they never asked git anything).
    #[test]
    fn bootstrap_prune_fails_safe_when_worktree_root_is_not_a_git_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let main_store = tmp.path().join("main").join(".bee");
        std::fs::create_dir_all(main_store.join("cells")).unwrap();

        let wt = tmp.path().join("wt-a"); // deliberately never `git init`-ed
        let wt_cells = wt.join(".bee").join("cells");
        std::fs::create_dir_all(&wt_cells).unwrap();
        std::fs::write(wt_cells.join("b-1.json"), "{\"id\":\"b-1\",\"feature\":\"b\",\"status\":\"open\"}")
            .unwrap();

        bootstrap_worktree_store(&wt, &main_store, "a").unwrap();

        assert!(
            wt_cells.join("b-1.json").exists(),
            "git unavailable/not-a-repo fails safe — nothing is pruned"
        );
    }

    /// A feature with zero cells in the main store bootstraps a clean, empty
    /// `cells` dir rather than erroring or leaving the dir absent.
    #[test]
    fn bootstrap_with_no_matching_cells_makes_an_empty_cells_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let main_store = tmp.path().join("main").join(".bee");
        std::fs::create_dir_all(&main_store).unwrap(); // no cells/ at all

        let wt = tmp.path().join("wt-empty");
        std::fs::create_dir_all(&wt).unwrap();
        bootstrap_worktree_store(&wt, &main_store, "empty-feature").unwrap();

        let wt_cells = wt.join(".bee").join("cells");
        assert!(wt_cells.is_dir());
        assert_eq!(std::fs::read_dir(&wt_cells).unwrap().count(), 0);
    }

    // ── review B-P1-1: a symlinked cell-store path refuses sync entirely ────

    /// review B-P1-1 fixture (a): the ISLAND's `.bee/cells` is itself a
    /// SYMLINK to a victim directory full of `*.json` files — a home-dir
    /// config dir stands in for the transcript's real demo target. Before
    /// this fix the tracked-set shield came back empty (`git ls-files` never
    /// sees paths under a symlink) and the prune arm deleted every `*.json`
    /// it found in the symlink's TARGET, outside the store entirely. This
    /// must be RED against the pre-fix code: the victim directory has to
    /// stay byte-identical, and the bootstrap report has to name the skip.
    #[test]
    fn bootstrap_refuses_a_symlinked_island_cells_dir_before_any_prune() {
        if !symlink_capable() {
            eprintln!("SKIP (env-limited: {SYMLINK_CAP}) — symlinked island .bee/cells refusal");
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let main_store = tmp.path().join("main").join(".bee");
        std::fs::create_dir_all(main_store.join("cells")).unwrap();

        // The victim: a directory full of `*.json` with nothing to do with
        // bee's cell store.
        let victim = tmp.path().join("victim-config");
        std::fs::create_dir_all(&victim).unwrap();
        std::fs::write(victim.join("settings.json"), "{\"unrelated\":true}").unwrap();
        std::fs::write(victim.join("keys.json"), "{\"token\":\"secret\"}").unwrap();
        let before: Vec<(PathBuf, Vec<u8>)> = std::fs::read_dir(&victim)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| (e.path(), std::fs::read(e.path()).unwrap()))
            .collect();
        assert_eq!(before.len(), 2);

        let wt = tmp.path().join("wt-a");
        std::fs::create_dir_all(wt.join(".bee")).unwrap();
        let wt_cells = wt.join(".bee").join("cells");
        symlink_dir(&victim.to_string_lossy(), &wt_cells).unwrap();

        let report = bootstrap_worktree_store(&wt, &main_store, "a").unwrap();

        for (path, bytes) in &before {
            assert_eq!(
                &std::fs::read(path).unwrap(),
                bytes,
                "victim dir must stay byte-identical at {}",
                path.display()
            );
        }
        assert_eq!(std::fs::read_dir(&victim).unwrap().count(), 2, "victim dir must keep every file");

        let sync = report.get("cellsSync").expect("bootstrap report must name the symlink skip");
        assert_eq!(sync.get("skipped"), Some(&Value::Bool(true)));
        assert_eq!(sync.get("path"), Some(&json!(p(&wt_cells))));
        assert!(
            sync.get("reason").and_then(Value::as_str).unwrap().to_lowercase().contains("symlink"),
            "{sync:?}"
        );

        // Never deleted through: the symlink itself is still there.
        assert!(std::fs::symlink_metadata(&wt_cells).unwrap().file_type().is_symlink());
    }

    /// review B-P1-1 fixture (b): the MAIN store's `cells` source dir is the
    /// symlink instead. The whole sync (prune AND fill) is refused before it
    /// ever reads through it, and the island's own `.bee/cells` is left
    /// absent rather than partially filled.
    #[test]
    fn bootstrap_refuses_a_symlinked_main_store_cells_dir_before_any_prune() {
        if !symlink_capable() {
            eprintln!("SKIP (env-limited: {SYMLINK_CAP}) — symlinked main store cells refusal");
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let main_store = tmp.path().join("main").join(".bee");
        std::fs::create_dir_all(&main_store).unwrap();

        let victim = tmp.path().join("victim-config-2");
        std::fs::create_dir_all(&victim).unwrap();
        std::fs::write(victim.join("settings.json"), "{\"unrelated\":true}").unwrap();
        let before = std::fs::read(victim.join("settings.json")).unwrap();

        let main_cells = main_store.join("cells");
        symlink_dir(&victim.to_string_lossy(), &main_cells).unwrap();

        let wt = tmp.path().join("wt-a");
        std::fs::create_dir_all(&wt).unwrap();

        let report = bootstrap_worktree_store(&wt, &main_store, "a").unwrap();

        assert_eq!(std::fs::read(victim.join("settings.json")).unwrap(), before);
        let sync = report.get("cellsSync").expect("bootstrap report must name the symlink skip");
        assert_eq!(sync.get("skipped"), Some(&Value::Bool(true)));
        assert_eq!(sync.get("path"), Some(&json!(p(&main_cells))));

        // The skip is whole-function: the island's own `.bee/cells` was
        // never even created, let alone partially filled from the symlink.
        let wt_cells = wt.join(".bee").join("cells");
        assert!(
            !wt_cells.exists(),
            "a symlinked source must skip before creating the destination dir at all"
        );
    }

    /// review B-P1-1 fixture (c): the ordinary, non-symlinked path is
    /// unaffected by the new guard — no `cellsSync` skip in the report, and
    /// the existing wsh/ips fixtures above (byte-for-byte prune/fill,
    /// `git status --porcelain` clean) stay green untouched.
    #[test]
    fn bootstrap_cell_sync_runs_normally_when_nothing_is_symlinked() {
        let tmp = tempfile::tempdir().unwrap();
        let main_store = tmp.path().join("main").join(".bee");
        let main_cells = main_store.join("cells");
        std::fs::create_dir_all(&main_cells).unwrap();
        std::fs::write(main_cells.join("a-1.json"), "{\"id\":\"a-1\",\"feature\":\"a\",\"status\":\"open\"}")
            .unwrap();

        let wt = tmp.path().join("wt-a");
        std::fs::create_dir_all(&wt).unwrap();
        let report = bootstrap_worktree_store(&wt, &main_store, "a").unwrap();

        assert!(report.get("cellsSync").is_none(), "a plain directory run must never report a symlink skip");
        assert!(wt.join(".bee").join("cells").join("a-1.json").exists());
    }

    /// resolveWorktreeFeature's preference order (issues-46-53 D4): the
    /// IMMUTABLE creation slug wins, and its absence degrades EXACTLY to the
    /// pre-fix `state.feature` behavior rather than refusing.
    #[test]
    fn worktree_feature_prefers_the_immutable_creation_slug() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path();
        std::fs::create_dir_all(wt.join(".bee").join("runtime")).unwrap();

        // Neither file: unknown, and the branch check falls back to the shape.
        let none = resolve_worktree_feature(wt);
        assert_eq!(none.feature, None);
        assert!(wt_branch_shaped("wt/demo-2"));
        assert!(!wt_branch_shaped("wt/Demo"));
        assert!(!wt_branch_shaped("feature/demo"));

        // Only state.json: the legacy degradation path.
        std::fs::write(wt.join(".bee").join("state.json"), "{\"feature\":\"renamed\"}").unwrap();
        let legacy = resolve_worktree_feature(wt);
        assert_eq!(legacy.feature.as_deref(), Some("renamed"));
        assert_eq!(legacy.created, None);

        // Both: the creation slug wins, and BOTH are reported so the refusal
        // can name the field that drifted.
        std::fs::write(
            wt.join(".bee").join("runtime").join("worktree-identity.json"),
            "{\"feature\":\"original\"}",
        )
        .unwrap();
        let both = resolve_worktree_feature(wt);
        assert_eq!(both.feature.as_deref(), Some("original"));
        assert_eq!(both.created.as_deref(), Some("original"));
        assert_eq!(both.state_feature.as_deref(), Some("renamed"));

        // A corrupt file is "unknown", never a crash — the .mjs reads these
        // with a bare JSON.parse in a try, not fsutil's warning readJson.
        std::fs::write(
            wt.join(".bee").join("runtime").join("worktree-identity.json"),
            "{oops",
        )
        .unwrap();
        assert_eq!(resolve_worktree_feature(wt).created, None);
    }

    /// gitStatusPorcelain's failure message is deterministic even when git
    /// never launched — including the literal "exit null" `${result.status}`
    /// renders. This is what makes every downstream `.stdout.trim()` site
    /// unreachable (module header, blocker (c)).
    #[test]
    fn git_status_failure_message_is_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        // Not a git repo at all: git launches and exits non-zero.
        let err = git_status_porcelain(tmp.path()).unwrap_err();
        assert!(
            err.starts_with(&format!("\"git status --porcelain\" failed in {}: ", p(tmp.path()))),
            "{err}"
        );
        // The never-launched shape renders "exit null" through the same chain.
        let never = GitOut { status: None, stdout: None, stderr: None };
        assert_eq!(never.fail_text(), "exit null");
    }

    /// The verify child is Node's `shell: true`, so a shell builtin runs and
    /// its exit code comes back verbatim — and `output_tail` is the LAST 30
    /// lines of stdout-then-stderr concatenated.
    #[test]
    fn verify_child_captures_status_and_output() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(shell_launchable());
        let green = run_verify_child("exit 0", tmp.path(), &|| {}, 30_000.0);
        assert!(green.ran);
        assert_eq!(green.status, Some(0));

        let red = run_verify_child("echo RED-TAIL& exit 7", tmp.path(), &|| {}, 30_000.0);
        assert_eq!(red.status, Some(7));
        assert!(red.combined.contains("RED-TAIL"), "{:?}", red.combined);

        // The tick fires while a slow child runs (integration-queue's
        // processor-lease heartbeat depends on exactly this).
        let ticks = std::sync::atomic::AtomicUsize::new(0);
        let slow = if cfg!(windows) {
            "ping -n 2 127.0.0.1 > NUL"
        } else {
            "sleep 1"
        };
        let out = run_verify_child(
            slow,
            tmp.path(),
            &|| {
                ticks.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            },
            60.0,
        );
        assert_eq!(out.status, Some(0));
        assert!(ticks.load(std::sync::atomic::Ordering::SeqCst) > 0, "the renewal tick must fire");
    }

    /// performCleanup's refusal shapes carry Node's exact key ORDER — the
    /// bytes `--json` prints and the twin diff pins.
    #[test]
    fn cleanup_check_failure_keeps_nodes_key_order() {
        let tmp = tempfile::tempdir().unwrap();
        let out = perform_cleanup(tmp.path(), tmp.path(), "wt/demo", "wt-demo", false);
        assert_eq!(out.keys().collect::<Vec<_>>(), vec!["ok", "code", "reason"]);
        assert_eq!(out["ok"], Value::Bool(false));
        assert_eq!(out["code"], json!("WORKTREE_MERGE_CLEANUP_CHECK_FAILED"));
    }

    /// The dirty-tree refusal carries a fourth `status` key after `reason` —
    /// the smallest real state that reaches it is a git repo with a single
    /// untracked file, no commit required.
    #[test]
    fn cleanup_dirty_worktree_keeps_nodes_key_order() {
        let tmp = tempfile::tempdir().unwrap();
        let worktree_root = tmp.path().join("wt");
        std::fs::create_dir_all(&worktree_root).unwrap();
        git_ok(&worktree_root, &["init", "-q", "-b", "main", "."]);
        git_ok(&worktree_root, &["config", "user.email", "a@b.c"]);
        git_ok(&worktree_root, &["config", "user.name", "t"]);
        // An untracked file at a tracked path — `git status --porcelain` is
        // non-empty; `main_root` is unreached on this branch, so reusing
        // `worktree_root` for it is fine.
        std::fs::write(worktree_root.join("f.txt"), "x").unwrap();

        let out = perform_cleanup(&worktree_root, &worktree_root, "wt/demo", "wt-demo", false);
        assert_eq!(
            out.keys().collect::<Vec<_>>(),
            vec!["ok", "code", "reason", "status"]
        );
        assert_eq!(out["ok"], Value::Bool(false));
        assert_eq!(out["code"], json!("WORKTREE_MERGE_CLEANUP_DIRTY"));
    }

    /// The `git worktree remove` failure shape has the same three keys as the
    /// check-failed shape but a different `reason` source — reached with a
    /// clean, standalone git repo standing in for the worktree root (so the
    /// dirty check passes) and a `main_root` that is deliberately not a git
    /// repository at all, so `git worktree remove` fails to launch against
    /// it without ever touching a real worktree relationship.
    #[test]
    fn cleanup_remove_failure_keeps_nodes_key_order() {
        let tmp = tempfile::tempdir().unwrap();
        let worktree_root = tmp.path().join("clean-repo");
        std::fs::create_dir_all(&worktree_root).unwrap();
        std::fs::write(worktree_root.join("f.txt"), "x").unwrap();
        git_ok(&worktree_root, &["init", "-q", "-b", "main", "."]);
        git_ok(&worktree_root, &["config", "user.email", "a@b.c"]);
        git_ok(&worktree_root, &["config", "user.name", "t"]);
        git_ok(&worktree_root, &["add", "-A"]);
        git_ok(&worktree_root, &["commit", "-qm", "init"]);

        let main_root = tmp.path().join("not-a-repo");
        std::fs::create_dir_all(&main_root).unwrap();

        let out = perform_cleanup(&main_root, &worktree_root, "wt/demo", "wt-demo", false);
        assert_eq!(out.keys().collect::<Vec<_>>(), vec!["ok", "code", "reason"]);
        assert_eq!(out["ok"], Value::Bool(false));
        assert_eq!(out["code"], json!("WORKTREE_MERGE_CLEANUP_REMOVE_FAILED"));
    }

    /// The branch-delete failure shape puts `removed: true` in a MIDDLE slot
    /// (`ok, code, removed, reason`) — no uniform refusal formatter would
    /// reproduce that. Reached with a real `git worktree add` fixture whose
    /// branch carries a commit main never merged, so `git branch -d` refuses
    /// after the worktree directory is already gone.
    #[test]
    fn cleanup_branch_delete_failure_keeps_nodes_key_order() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "demo", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();

        // A commit on the branch that main never merges, so `git branch -d`
        // refuses — the tree is clean again once this is committed.
        std::fs::write(wt.join("f.txt"), "unmerged").unwrap();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        git_ok(&wt, &["commit", "-qam", "unmerged work"]);

        let out = perform_cleanup(&main, &wt, &created.branch, &created.id, false);
        assert_eq!(
            out.keys().collect::<Vec<_>>(),
            vec!["ok", "code", "removed", "reason"]
        );
        assert_eq!(out["ok"], Value::Bool(false));
        assert_eq!(
            out["code"],
            json!("WORKTREE_MERGE_CLEANUP_BRANCH_DELETE_FAILED")
        );
        assert_eq!(out["removed"], Value::Bool(true));
    }

    /// attachCleanupOutcome's `cleanup=false` branch NEVER runs anything — it
    /// attaches the suggestion (decision D8b: "never prompt"). D1/D1a: the
    /// caller now passes the EFFECTIVE decision (`--no-cleanup`, a `false`
    /// `worktree_cleanup_on_merge`, or the hardcoded ALREADY_UP_TO_DATE
    /// case), never a raw `--cleanup` flag — this test used to be named for
    /// the flag's absence; it now covers this branch by its value instead.
    /// The suggested command drops `--cleanup` (a no-op now): re-running
    /// merge WITHOUT `--no-cleanup` is what actually cleans up.
    #[test]
    fn cleanup_false_only_suggests() {
        let tmp = tempfile::tempdir().unwrap();
        let mut result = Map::new();
        attach_cleanup_outcome(&mut result, tmp.path(), tmp.path(), "wt/demo", "wt-demo", false, false);
        assert_eq!(
            result["cleanup_suggested_command"],
            json!("bee worktree merge --id wt-demo --json")
        );
        assert!(!result.contains_key("cleanup"));
    }

    // ── D1/D1a: cleanup by default ──────────────────────────────────────────

    /// `worktree_cleanup_on_merge`'s config half, read the same way
    /// `archive_on_close_enabled` reads `cells_archive_on_close`
    /// (close.rs:827) — absent means ON — but unlike that helper, a
    /// present-but-non-boolean value is REFUSED (`None`), never silently
    /// read as ON.
    #[test]
    fn worktree_cleanup_on_merge_config_matches_archive_on_close_enabled_except_it_validates() {
        let tmp = tempfile::tempdir().unwrap();
        // No .bee/config.json at all -> on.
        assert_eq!(worktree_cleanup_on_merge_config(tmp.path()), Some(true));

        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(
            tmp.path().join(".bee").join("config.json"),
            "{\"worktree_cleanup_on_merge\": true}",
        )
        .unwrap();
        assert_eq!(worktree_cleanup_on_merge_config(tmp.path()), Some(true));

        std::fs::write(
            tmp.path().join(".bee").join("config.json"),
            "{\"worktree_cleanup_on_merge\": false}",
        )
        .unwrap();
        assert_eq!(worktree_cleanup_on_merge_config(tmp.path()), Some(false));

        // A typo'd non-boolean value refuses rather than defaulting to ON —
        // the divergence from archive_on_close_enabled's own pattern.
        std::fs::write(
            tmp.path().join(".bee").join("config.json"),
            "{\"worktree_cleanup_on_merge\": \"no\"}",
        )
        .unwrap();
        assert_eq!(worktree_cleanup_on_merge_config(tmp.path()), None);
    }

    /// Truth 1 + truth 3: with no `--no-cleanup` flag and no config, the
    /// merge cleans up by default; a `false` `worktree_cleanup_on_merge`
    /// beats the absent flag.
    #[test]
    fn resolve_cleanup_on_merge_defaults_on_and_config_off_beats_the_absent_flag() {
        let tmp = tempfile::tempdir().unwrap();
        // No flags at all (no --no-cleanup), no config -> cleanup runs (D1).
        assert_eq!(resolve_cleanup_on_merge(tmp.path(), false), Some(true));

        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(
            tmp.path().join(".bee").join("config.json"),
            "{\"worktree_cleanup_on_merge\": false}",
        )
        .unwrap();
        // The flag was never passed (no_cleanup_flag = false) — config alone
        // beats the absent flag.
        assert_eq!(resolve_cleanup_on_merge(tmp.path(), false), Some(false));
    }

    /// Truth 1, end to end: a merge with no `--no-cleanup` flag and no config
    /// resolves cleanup ON, and running the merge with that DEFAULT-computed
    /// value actually removes the worktree directory and deletes its branch
    /// — the same call shape `run_merge` makes when handed no flags at all.
    #[test]
    fn merge_with_no_flags_cleans_up_by_default() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "demo", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();
        std::fs::write(wt.join("f.txt"), "y").unwrap();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        git_ok(&wt, &["commit", "-qam", "work"]);

        // No --no-cleanup flag, no config -> the same value run_merge would
        // compute for a plain "bee worktree merge --id <id>" invocation.
        let cleanup = resolve_cleanup_on_merge(&main, false).unwrap();
        assert!(cleanup, "no flags, no config -> cleanup runs by default (D1)");

        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, None, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert_eq!(answer.result["cleanup"]["ok"], Value::Bool(true));
        assert!(!wt.exists(), "the worktree directory is gone");
        let branches = std::process::Command::new("git")
            .args(["branch", "--list", &created.branch])
            .current_dir(&main)
            .output()
            .unwrap();
        assert!(
            String::from_utf8_lossy(&branches.stdout).trim().is_empty(),
            "the branch is gone"
        );
    }

    /// Truth 2: `--no-cleanup` resolves to cleanup=false, and a real merge
    /// with that value leaves the worktree directory and its branch standing.
    #[test]
    fn no_cleanup_flag_resolves_off_and_leaves_the_worktree_and_branch_standing() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "demo", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();
        std::fs::write(wt.join("f.txt"), "y").unwrap();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        git_ok(&wt, &["commit", "-qam", "work"]);

        let cleanup = resolve_cleanup_on_merge(&main, true).unwrap();
        assert!(!cleanup, "--no-cleanup opts this merge out (D1a)");

        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, None, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert!(!answer.result.contains_key("cleanup"));
        assert_eq!(
            answer.result["cleanup_suggested_command"],
            json!(format!("bee worktree merge --id {} --json", created.id))
        );
        assert!(wt.exists(), "the worktree directory still stands");
        let branches = std::process::Command::new("git")
            .args(["branch", "--list", &created.branch])
            .current_dir(&main)
            .output()
            .unwrap();
        assert!(
            !String::from_utf8_lossy(&branches.stdout).trim().is_empty(),
            "the branch still stands"
        );
    }

    /// Truth 4 (D1a): a non-boolean `--no-cleanup` value is REFUSED outright
    /// — never ignored, never defaulted to either outcome. This is the CLI
    /// gate `run_merge` checks BEFORE anything is resolved or mutated
    /// (`bool_flag_ok(&flags, "no-cleanup")`); a caller that fails it returns
    /// `None` from `run_merge`, which `main.rs`'s router turns into a
    /// non-zero "unsupported command shape" exit having removed nothing.
    #[test]
    fn no_cleanup_with_a_non_boolean_value_is_refused_not_defaulted() {
        let (flags, _json) = parse_flags(&["--id", "wt-demo", "--no-cleanup=yes"]).unwrap();
        assert!(
            !bool_flag_ok(&flags, "no-cleanup"),
            "a non-boolean --no-cleanup must refuse outright, not resolve to either outcome"
        );
        // The bare/true/false shapes all still validate fine.
        let (bare, _) = parse_flags(&["--no-cleanup"]).unwrap();
        assert!(bool_flag_ok(&bare, "no-cleanup"));
        assert!(bool_flag_true(&bare, "no-cleanup"));
        let (explicit_false, _) = parse_flags(&["--no-cleanup=false"]).unwrap();
        assert!(bool_flag_ok(&explicit_false, "no-cleanup"));
        assert!(!bool_flag_true(&explicit_false, "no-cleanup"));
    }

    /// Truth 5 (D1a): the ALREADY_UP_TO_DATE arm removes nothing — even when
    /// called with the default-computed `cleanup = true`, because merging
    /// nothing is not a real merge. A second worktree merged with zero new
    /// commits on its branch is the smallest fixture that reaches this arm.
    #[test]
    fn already_up_to_date_merge_removes_nothing_even_with_cleanup_on() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "demo", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();
        // Nothing new committed on the branch — main already contains it.

        let cleanup = resolve_cleanup_on_merge(&main, false).unwrap();
        assert!(cleanup, "no flag, no config -> cleanup would run on a REAL merge (D1)");

        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, None, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["code"], json!("ALREADY_UP_TO_DATE"));
        assert!(
            !answer.result.contains_key("cleanup"),
            "D1a: the up-to-date arm never runs cleanup, regardless of the caller's cleanup value"
        );
        assert_eq!(
            answer.result["cleanup_suggested_command"],
            json!(format!("bee worktree merge --id {} --json", created.id))
        );
        assert!(wt.exists(), "the worktree directory is untouched");
        let branches = std::process::Command::new("git")
            .args(["branch", "--list", &created.branch])
            .current_dir(&main)
            .output()
            .unwrap();
        assert!(
            !String::from_utf8_lossy(&branches.stdout).trim().is_empty(),
            "the branch is untouched"
        );
    }

    /// B-P1-2: pins `--no-gpg-sign` on the merge commit (phases.rs's
    /// `merge_finish`). A `main` repo with `commit.gpgsign true` AND
    /// `gpg.program` pointed at a stub that always fails must still let the
    /// merge commit land — proving the merge, which runs while the
    /// `worktree-admin` lock is held, can never block on a signing prompt.
    /// Without `--no-gpg-sign` this turns red: the commit would ask the
    /// failing stub to sign, `git commit` would exit non-zero, and
    /// `merge_feature_worktree` would return `WORKTREE_MERGE_COMMIT_FAILED`
    /// instead of `merged: true` (verified by hand: temporarily dropping the
    /// flag from `phases.rs` flips this assertion red, restored after).
    #[cfg(unix)]
    #[test]
    fn merge_commit_lands_with_gpgsign_true_and_a_failing_signer() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "demo", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();
        std::fs::write(wt.join("f.txt"), "y").unwrap();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        git_ok(&wt, &["commit", "-qam", "work"]);

        // A "gpg" that always fails, wired in on `main` — the merge commit
        // runs there, not in the worktree.
        let stub = tmp.path().join("gpg-stub.sh");
        std::fs::write(&stub, "#!/bin/sh\nexit 1\n").unwrap();
        let mut perms = std::fs::metadata(&stub).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&stub, perms).unwrap();
        git_ok(&main, &["config", "commit.gpgsign", "true"]);
        git_ok(&main, &["config", "gpg.program", stub.to_str().unwrap()]);

        let cleanup = resolve_cleanup_on_merge(&main, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, None, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    /// releaseAllForHolder marks every unreleased row for the holder and
    /// leaves everyone else's — and never rewrites the file when nothing
    /// changed (worktree-holds.mjs's own "only write when something changed").
    #[test]
    fn release_all_for_holder_is_holder_scoped() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp
            .path()
            .join(".bee")
            .join("runtime")
            .join("cross-worktree-holds.json");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(
            &file,
            serde_json::to_string_pretty(&json!({"holds": [
                {"holder": "wt-a", "path": "src/x", "released_at": null},
                {"holder": "wt-b", "path": "src/y", "released_at": null},
                {"holder": "wt-a", "path": "src/z", "released_at": "2020-01-01T00:00:00.000Z"},
            ]}))
            .unwrap(),
        )
        .unwrap();
        release_all_for_holder(tmp.path(), "wt-a");
        let after: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        let holds = after["holds"].as_array().unwrap();
        assert!(holds[0]["released_at"].is_string(), "the holder's row is released");
        assert!(holds[1]["released_at"].is_null(), "another holder is untouched");
        assert_eq!(holds[2]["released_at"], json!("2020-01-01T00:00:00.000Z"));

        // Nothing to release: the file is left byte-identical.
        let before = std::fs::read_to_string(&file).unwrap();
        release_all_for_holder(tmp.path(), "wt-nobody");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
    }

    // ── the lifted teardown helper (D3, D3a) ────────────────────────────────

    /// `run_unregister` wires to `teardown_worktree(.., remove: None)` — the
    /// registry half alone. A worktree that was registered (grant + workspace
    /// record, exactly what `run_register` leaves behind) has BOTH gone after
    /// one call, closing the orphan `unregister` used to leave.
    #[test]
    fn teardown_worktree_registry_half_clears_grant_and_workspace_record() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path().join("main");
        let main_store_root = main_root.join(".bee");
        std::fs::create_dir_all(&main_store_root).unwrap();

        let id = "wt-demo";
        let mut grants = Map::new();
        grants.insert(id.to_string(), Value::Bool(true));
        write_grants_file_atomic(&main_store_root, &grants).unwrap();

        ws::register_workspace(
            &main_root,
            ws::RegisterSpec {
                id,
                kind: "worktree",
                root: &p(&tmp.path().join("wt")),
                branch: Some("wt/demo"),
                base_sha: None,
            },
            "2020-01-01T00:00:00.000Z",
        )
        .unwrap();
        let record_file = ws::workspace_path(&main_root, id).unwrap();
        assert!(record_file.exists(), "the fixture must start with a workspace record");

        assert!(teardown_worktree(&main_root, id, None).is_ok());

        let after = read_grants_strict(&main_store_root).unwrap();
        assert!(!after.contains_key(id), "the grant entry must be gone");
        assert!(!record_file.exists(), "the workspace record must be gone");
    }

    /// `remove: None` never reaches the directory/branch steps at all — the
    /// registry-only path removes no directory, however it is spelled.
    #[test]
    fn teardown_worktree_registry_half_touches_no_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let worktree_root = tmp.path().join("wt");
        std::fs::create_dir_all(&worktree_root).unwrap();
        assert!(teardown_worktree(tmp.path(), "wt-demo", None).is_ok());
        assert!(worktree_root.exists(), "no directory removal was requested");
    }

    /// The self-delete guard, isolated from the real process cwd: it panics
    /// when the (injected) current directory sits inside the removal root,
    /// and passes cleanly when it does not.
    #[test]
    fn directory_removal_guard_rejects_its_own_cwd() {
        let tmp = tempfile::tempdir().unwrap();
        let worktree_root = tmp.path().join("wt");
        let nested = worktree_root.join("nested");
        std::fs::create_dir_all(&nested).unwrap();
        let elsewhere = tmp.path().join("elsewhere");
        std::fs::create_dir_all(&elsewhere).unwrap();

        // Outside the root, and the root itself as cwd: both refused? No —
        // only "inside or equal to" is unsafe. `worktree_root` itself and
        // `nested` are both inside (starts_with includes equality); the
        // sibling directory is not.
        assert_directory_removal_is_safe(Some(&elsewhere), &worktree_root);
        assert_directory_removal_is_safe(None, &worktree_root);

        let panicked = std::panic::catch_unwind(|| {
            assert_directory_removal_is_safe(Some(&nested), &worktree_root);
        });
        assert!(panicked.is_err(), "a cwd nested inside the removal root must panic");

        let panicked_at_root = std::panic::catch_unwind(|| {
            assert_directory_removal_is_safe(Some(&worktree_root), &worktree_root);
        });
        assert!(panicked_at_root.is_err(), "cwd equal to the removal root must panic too");
    }

    // ── worktree-companion-hook, over REAL `git worktree add` fixtures ─────
    //
    // The mount is a real symlink and win32 denies symlink creation without
    // SeCreateSymbolicLinkPrivilege, so every test that must CREATE one probes
    // the capability and skips LOUDLY, naming it. The tests that only need a
    // mount to EXIST use a plain untracked file instead: the dirty-check
    // exclusion and the teardown unlink are indifferent to the node type
    // (porcelain collapses a symlink-to-directory and a plain file into the
    // same "?? <top-level>/" summary line, which is the whole reason the
    // exclusion has to be a git pathspec), so the companion merge path runs
    // natively here on any host.

    const SYMLINK_CAP: &str = "symlink creation denied — needs SeCreateSymbolicLinkPrivilege \
(Developer Mode or an elevated shell)";

    fn symlink_capable() -> bool {
        use std::sync::OnceLock;
        static CAP: OnceLock<bool> = OnceLock::new();
        *CAP.get_or_init(|| {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("t");
            std::fs::create_dir(&target).unwrap();
            symlink_dir(&target.to_string_lossy(), &dir.path().join("l")).is_ok()
        })
    }

    fn git_ok(cwd: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git must be on PATH for the worktree fixtures");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// A MAIN checkout with one commit and a host-shaped `.gitignore`: the
    /// whole `.bee` store is ignored EXCEPT the companion marker, which is the
    /// real-world shape COMPANION_MARKER_REL's own comment describes (it sits
    /// outside the gitignored `.bee/runtime/` prefix, so it is untracked AND
    /// not ignored — and therefore has to be excluded by pathspec).
    fn main_repo(tmp: &Path) -> PathBuf {
        let main = tmp.join("main");
        std::fs::create_dir_all(main.join(".bee")).unwrap();
        std::fs::write(main.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        std::fs::write(
            main.join(".gitignore"),
            ".bee/*\n!.bee/companion-session.json\n",
        )
        .unwrap();
        std::fs::write(main.join("f.txt"), "x").unwrap();
        git_ok(&main, &["init", "-q", "-b", "main", "."]);
        git_ok(&main, &["config", "user.email", "a@b.c"]);
        git_ok(&main, &["config", "user.name", "t"]);
        git_ok(&main, &["add", "-A"]);
        git_ok(&main, &["commit", "-qm", "init"]);
        main
    }

    /// A shell command that prints `file`'s bytes verbatim — the portable way
    /// to give `commands.worktree_companion_start` a fixed JSON stdout through
    /// the same `shell: true` spawn production uses.
    fn cat_command(file: &Path) -> String {
        if cfg!(windows) {
            format!("type \"{}\"", file.to_string_lossy())
        } else {
            format!("cat \"{}\"", file.to_string_lossy())
        }
    }

    /// The whole `--with-companion` creation path: the configured child runs,
    /// its declared worktreePath is mounted as a real symlink at the
    /// configured relative mount, and the marker records all three fields.
    #[test]
    fn companion_start_mounts_the_declared_path_and_writes_the_marker() {
        if !symlink_capable() {
            eprintln!("SKIP (env-limited: {SYMLINK_CAP}) — worktree new --with-companion mounts and marks");
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let companion = tmp.path().join("shared-checkout");
        std::fs::create_dir_all(&companion).unwrap();
        let payload = tmp.path().join("payload.json");
        std::fs::write(
            &payload,
            jsjson::stringify(&json!({
                "worktreePath": companion.to_string_lossy(),
                "sessionId": "sess-1",
            })),
        )
        .unwrap();

        let command = cat_command(&payload);
        let mut lock_busy = None;
        let created = create_feature_worktree(
            &main,
            "demo",
            None,
            CompanionSpec {
                start_command: Some(&command),
                mount_path: Some("vendor/companion"),
            },
            &mut lock_busy,
        )
        .unwrap_or_else(|e| match e {
            CErr::Refuse(m) => panic!("refused: {m}"),
            CErr::Ex => panic!("delegated"),
        });

        assert_eq!(
            created.companion.get("sessionId"),
            Some(&json!("sess-1")),
            "{:?}",
            created.companion
        );
        assert_eq!(created.companion.get("mountPath"), Some(&json!("vendor/companion")));

        // The mount is a real link to the declared path.
        let mount = created.worktree_root.join("vendor").join("companion");
        assert!(std::fs::symlink_metadata(&mount).unwrap().file_type().is_symlink());
        assert_eq!(
            dunce::canonicalize(&mount).unwrap(),
            dunce::canonicalize(&companion).unwrap()
        );

        // The marker is Node's bytes: 2-space JSON + a trailing newline.
        let marker = std::fs::read_to_string(companion_marker_file(&created.worktree_root)).unwrap();
        assert!(marker.ends_with("}\n"), "{marker}");
        let parsed: Value = serde_json::from_str(&marker).unwrap();
        assert_eq!(
            parsed.as_object().unwrap().keys().collect::<Vec<_>>(),
            vec!["sessionId", "worktreePath", "mountPath"]
        );
    }

    /// A companion start failure fires AFTER `git worktree add`, so it enters
    /// the post-add rollback ladder: worktree gone, branch gone, grant gone,
    /// and the typed refusal names the child's exit. Needs no symlink — the
    /// child dies before the mount is ever created.
    #[test]
    fn a_failed_companion_start_rolls_the_worktree_back() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let err = create_feature_worktree(
            &main,
            "demo",
            None,
            CompanionSpec {
                start_command: Some("exit 7"),
                mount_path: Some("vendor/companion"),
            },
            &mut lock_busy,
        )
        .err()
        .expect("a failing companion start must refuse");
        let CErr::Refuse(message) = err else {
            panic!("a companion start failure must never delegate")
        };
        assert!(
            message.starts_with("[WORKTREE_POST_ADD_FAILED] "),
            "{message}"
        );
        assert!(
            message.contains("commands.worktree_companion_start failed (exit 7): (no output)"),
            "{message}"
        );
        assert!(message.contains("it has been rolled back"), "{message}");

        // The ladder unwound in Node's order: nothing survives.
        assert!(!tmp.path().join("main--wt--demo").exists());
        assert_eq!(
            read_grants_strict(&main.join(".bee")).unwrap(),
            Map::new(),
            "the grant is rolled back"
        );
        let branches = run_git(&main, &["branch", "--list", "wt/demo"]);
        assert_eq!(js_trim(&branches.stdout.unwrap_or_default()), "");
    }

    /// Unparseable child stdout is a typed post-add refusal too. Node's
    /// parenthetical carries V8's JSON.parse message and this port carries
    /// serde's (the module header's documented divergence); every other byte —
    /// including the raw-stdout tail — is Node's.
    #[test]
    fn unparseable_companion_stdout_refuses_with_the_raw_tail() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let payload = tmp.path().join("payload.txt");
        std::fs::write(&payload, "not json at all").unwrap();
        let mut lock_busy = None;
        let err = create_feature_worktree(
            &main,
            "demo",
            None,
            CompanionSpec {
                start_command: Some(&cat_command(&payload)),
                mount_path: Some("vendor/companion"),
            },
            &mut lock_busy,
        )
        .err()
        .unwrap();
        let CErr::Refuse(message) = err else { panic!("must not delegate") };
        assert!(message.contains(
            "commands.worktree_companion_start must print JSON with a \"worktreePath\" field to stdout — got unparseable output ("
        ), "{message}");
        assert!(message.contains("Raw stdout: not json at all"), "{message}");
        assert!(!tmp.path().join("main--wt--demo").exists());
    }

    /// JSON that parses but carries no usable worktreePath is a FULLY
    /// deterministic refusal — `JSON.stringify(parsed)` and nothing else.
    #[test]
    fn companion_stdout_without_a_worktree_path_refuses_deterministically() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let payload = tmp.path().join("payload.json");
        std::fs::write(&payload, "{\"sessionId\":\"s\"}").unwrap();
        let mut lock_busy = None;
        let err = create_feature_worktree(
            &main,
            "demo",
            None,
            CompanionSpec {
                start_command: Some(&cat_command(&payload)),
                mount_path: Some("vendor/companion"),
            },
            &mut lock_busy,
        )
        .err()
        .unwrap();
        let CErr::Refuse(message) = err else { panic!("must not delegate") };
        assert!(message.contains(
            "commands.worktree_companion_start's JSON output must include a non-empty \"worktreePath\" string — got {\"sessionId\":\"s\"}."
        ), "{message}");
    }

    /// The two zero-mutation companion config refusals, with their exact
    /// `[CODE] …` bytes — neither ever reaches `git worktree add`.
    #[test]
    fn companion_config_refusals_are_zero_mutation() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;

        let only_one = create_feature_worktree(
            &main,
            "demo",
            None,
            CompanionSpec { start_command: Some("true"), mount_path: None },
            &mut lock_busy,
        )
        .err()
        .unwrap();
        let CErr::Refuse(message) = only_one else { panic!() };
        assert_eq!(
            message,
            "[WORKTREE_COMPANION_CONFIG_INCOMPLETE] commands.worktree_companion_start and commands.worktree_companion_mount must both be configured to use --with-companion — only one was found."
        );

        for bad in ["/abs/mount", "..\\escape", "a/../b"] {
            let mut lock_busy = None;
            let err = create_feature_worktree(
                &main,
                "demo",
                None,
                CompanionSpec { start_command: Some("true"), mount_path: Some(bad) },
                &mut lock_busy,
            )
            .err()
            .unwrap();
            let CErr::Refuse(message) = err else { panic!() };
            assert!(
                message.starts_with("[WORKTREE_COMPANION_CONFIG_INVALID] "),
                "{bad}: {message}"
            );
        }
        assert!(!tmp.path().join("main--wt--demo").exists(), "zero mutation");
    }

    /// The companion MERGE path, end to end, on a real worktree: the mount and
    /// the marker are both excluded from the dirty-check by pathspec (without
    /// that the merge refuses WORKTREE_MERGE_WORKTREE_DIRTY), the configured
    /// `_end` command runs, both are unlinked, and `companion` rides the result.
    #[test]
    fn a_companion_worktree_merges_and_tears_the_mount_down() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "demo", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();

        // Real work on the branch, so the merge has something to do.
        std::fs::write(wt.join("f.txt"), "y").unwrap();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        git_ok(&wt, &["commit", "-qam", "work"]);

        // A companion mount + marker, exactly as runCompanionStart leaves them.
        // A plain untracked file stands in for the symlink (see the block
        // comment above) so this runs on every host.
        std::fs::create_dir_all(wt.join("vendor")).unwrap();
        std::fs::write(wt.join("vendor").join("companion"), "mount").unwrap();
        std::fs::write(
            companion_marker_file(&wt),
            format!(
                "{}\n",
                jsjson::stringify_pretty(&json!({
                    "sessionId": "sess-1",
                    "worktreePath": tmp.path().join("shared").to_string_lossy(),
                    "mountPath": "vendor/companion",
                }))
            ),
        )
        .unwrap();

        // Both are genuinely dirt without the exclusion — prove it, so the
        // pathspec below is doing real work.
        assert!(is_tree_dirty(&wt).unwrap(), "the mount+marker read as dirty");

        let answer = merge_feature_worktree(&main, &created.id, false, None, Some("exit 0"), None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert_eq!(
            answer.result["companion"],
            json!({ "ended": true, "sessionId": "sess-1" }),
            "an ended-cleanly companion carries no `warning` key"
        );
        // `companion` sits directly after `verify`, before the cleanup keys.
        let keys: Vec<&String> = answer.result.keys().collect();
        let vi = keys.iter().position(|k| *k == "verify").unwrap();
        assert_eq!(keys[vi + 1], "companion");

        assert!(!wt.join("vendor").join("companion").exists(), "mount unlinked");
        assert!(!companion_marker_file(&wt).exists(), "marker unlinked");

        let lines = merge_text_lines(&created.id, &main, &answer);
        assert!(
            lines.iter().any(|l| l == "  companion: ended (session sess-1)."),
            "{lines:?}"
        );
    }

    /// With no `commands.worktree_companion_end` configured, teardown still
    /// removes the mount (so the merge is never blocked) but says so LOUDLY.
    #[test]
    fn teardown_without_an_end_command_warns_and_still_unlinks() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(wt.join(".bee")).unwrap();
        std::fs::create_dir_all(wt.join("vendor")).unwrap();
        std::fs::write(wt.join("vendor").join("companion"), "mount").unwrap();
        let marker = json!({"sessionId": Value::Null, "mountPath": "vendor/companion"});
        std::fs::write(companion_marker_file(&wt), jsjson::stringify(&marker)).unwrap();

        let out = teardown_companion_if_present(tmp.path(), &wt, None, Some(&marker)).unwrap();
        assert_eq!(out["ended"], Value::Bool(false));
        assert_eq!(out["sessionId"], Value::Null);
        assert!(
            jsjson::js_to_string(&out["warning"]).starts_with(
                "a companion marker exists on this worktree but commands.worktree_companion_end is not configured"
            ),
            "{out:?}"
        );
        assert!(!wt.join("vendor").join("companion").exists());
        assert!(!companion_marker_file(&wt).exists());
    }

    /// A failing `_end` command never blocks the merge — the mount still goes,
    /// and the failure rides the result as a warning with the child's exit.
    #[test]
    fn a_failed_end_command_warns_but_never_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(wt.join(".bee")).unwrap();
        std::fs::write(wt.join("mount"), "m").unwrap();
        let marker = json!({"sessionId": "sess-9", "mountPath": "mount"});
        std::fs::write(companion_marker_file(&wt), jsjson::stringify(&marker)).unwrap();

        let out =
            teardown_companion_if_present(tmp.path(), &wt, Some("exit 3"), Some(&marker)).unwrap();
        assert_eq!(out["ended"], Value::Bool(false));
        assert_eq!(out["sessionId"], json!("sess-9"));
        assert!(
            jsjson::js_to_string(&out["warning"])
                .starts_with("commands.worktree_companion_end failed (exit 3): (no output) — "),
            "{out:?}"
        );
        assert!(!wt.join("mount").exists());
    }

    /// The `<id>` substitution is JS `String.replace` with a STRING pattern:
    /// first occurrence only, `$`-patterns honored.
    #[test]
    fn the_end_command_substitutes_the_session_id_like_js() {
        assert_eq!(js_replace_first("end <id> then <id>", "<id>", "s1"), "end s1 then <id>");
        assert_eq!(js_replace_first("end <id>", "<id>", ""), "end ");
        assert_eq!(js_replace_first("no token", "<id>", "s1"), "no token");
        // $$ -> literal $, $& -> the matched text, $` / $' -> the surroundings.
        assert_eq!(js_replace_first("a<id>b", "<id>", "$$"), "a$b");
        assert_eq!(js_replace_first("a<id>b", "<id>", "$&"), "a<id>b");
        assert_eq!(js_replace_first("a<id>b", "<id>", "$`|$'"), "aa|bb");
        // $1 has no capture group behind a string pattern — left literal.
        assert_eq!(js_replace_first("a<id>b", "<id>", "$1"), "a$1b");
    }

    /// The dirty-check exclusion has to be a git PATHSPEC: porcelain collapses
    /// an untracked nested path to its top-level directory, so text-filtering
    /// for `vendor/companion` would never match and the merge would refuse
    /// forever. This pins the behavior that makes that true.
    #[test]
    fn the_dirty_check_exclusion_is_a_pathspec_not_a_text_filter() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        std::fs::create_dir_all(main.join("vendor")).unwrap();
        std::fs::write(main.join("vendor").join("companion"), "m").unwrap();

        // Porcelain names only the TOP-LEVEL dir, never the nested path.
        let plain = git_status_porcelain(&main).unwrap();
        assert!(plain.contains("?? vendor/"), "{plain:?}");
        assert!(!plain.contains("vendor/companion"), "{plain:?}");
        assert!(is_tree_dirty(&main).unwrap());

        // The pathspec removes it at the source, nested depth and all.
        assert!(!is_tree_dirty_excluding(
            &main,
            &["vendor/companion".to_string(), companion_marker_rel()]
        )
        .unwrap());
    }

    /// A truthy marker with no string `mountPath` is an EXPLICIT NATIVE
    /// REFUSAL: the .mjs dies here with a V8 TypeError, which can neither be
    /// reproduced nor (post-cutover) delegated.
    #[test]
    fn a_marker_without_a_mount_path_is_a_typed_refusal() {
        let MErr::Thrown(message) = companion_mount_path(&json!({"sessionId": "s"})).unwrap_err()
        else {
            panic!("must be a typed refusal")
        };
        assert!(
            message.starts_with(
                "[WORKTREE_MERGE_COMPANION_MARKER_INVALID] the companion marker at .bee/companion-session.json has no usable \"mountPath\" string (got null)"
            ),
            "{message}"
        );
        assert_eq!(
            companion_mount_path(&json!({"mountPath": "m"})).map_err(|_| ()),
            Ok("m".to_string())
        );
    }

    /// A marker that is missing, unparseable, or parses FALSY all read as "no
    /// companion here" — the .mjs's bare `JSON.parse` in a try, plus every
    /// consumer's `if (!marker)` guard.
    #[test]
    fn marker_reads_match_nodes_falsy_contract() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path();
        std::fs::create_dir_all(wt.join(".bee")).unwrap();
        assert_eq!(read_companion_marker(wt), None, "missing");
        for falsy in ["{oops", "null", "false", "0", "\"\""] {
            std::fs::write(companion_marker_file(wt), falsy).unwrap();
            assert_eq!(read_companion_marker(wt), None, "{falsy}");
        }
        std::fs::write(companion_marker_file(wt), "{\"mountPath\":\"m\"}").unwrap();
        assert_eq!(
            read_companion_marker(wt),
            Some(json!({"mountPath": "m"}))
        );
    }

    /// `--queue-wait-ms` is `Number(string)`, whole — the conversion
    /// validate()'s finiteness gate and the handler's positive filter both run
    /// against.
    #[test]
    fn queue_wait_ms_uses_js_number_semantics() {
        assert_eq!(js_string_to_number("5000"), 5000.0);
        assert_eq!(js_string_to_number("  5000  "), 5000.0);
        assert_eq!(js_string_to_number("5e3"), 5000.0);
        assert_eq!(js_string_to_number("1.5"), 1.5);
        assert_eq!(js_string_to_number(".5"), 0.5);
        assert_eq!(js_string_to_number("-1"), -1.0);
        assert_eq!(js_string_to_number("0x10"), 16.0);
        assert_eq!(js_string_to_number("0b101"), 5.0);
        assert_eq!(js_string_to_number("0o17"), 15.0);
        assert_eq!(js_string_to_number(""), 0.0);
        assert_eq!(js_string_to_number("   "), 0.0);
        assert!(js_string_to_number("Infinity").is_infinite());
        assert!(js_string_to_number("-Infinity").is_infinite());
        assert!(js_string_to_number("1e400").is_infinite()); // overflow, like V8
        for nan in ["abc", "5px", "0x", "1..2", "1e", "--1", "+"] {
            assert!(js_string_to_number(nan).is_nan(), "{nan}");
        }
    }

    /// The lock file this module contends on is Node's, by name.
    #[test]
    fn grant_writes_use_nodes_lock_name() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let Ok(mut guard) = lock::acquire_store_lock(root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS)
        else {
            panic!("a fresh root's worktree-admin lock must be free")
        };
        assert!(lock::lock_file_path(root, "worktree-admin").exists());
        guard.release();
    }

    // ── prune.rs: the fail-closed classifier (wr-2, D2/D2a/D2b) ────────────
    //
    // Every keep-reason gets its own test, plus the dead case and the
    // whole-run refusal. `age_threshold_ms: 0.0` neutralizes condition (7)
    // everywhere except the test that exercises it directly.

    fn prune_fixture(tmp: &Path) -> (PathBuf, Created) {
        let main = main_repo(tmp);
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "demo", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|e| match e {
                    CErr::Refuse(m) => panic!("refused: {m}"),
                    CErr::Ex => panic!("delegated"),
                });
        (main, created)
    }

    fn head_sha(root: &Path) -> String {
        js_trim(&run_git(root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string()
    }

    /// A merged, clean, untouched, unlocked, old-enough worktree is the ONLY
    /// shape that reaches `Verdict::Dead` — every other test below proves one
    /// way to fall short of it.
    #[test]
    fn a_fully_merged_clean_old_untouched_worktree_is_dead() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);
        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(verdict.is_dead(), "{}", verdict.reason());
    }

    #[test]
    fn an_unmerged_branch_keeps() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);
        std::fs::write(created.worktree_root.join("f.txt"), "unmerged").unwrap();
        git_ok(&created.worktree_root, &["config", "user.email", "a@b.c"]);
        git_ok(&created.worktree_root, &["config", "user.name", "t"]);
        git_ok(&created.worktree_root, &["commit", "-qam", "unmerged work"]);

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(verdict.reason().contains("not provably merged"), "{}", verdict.reason());
    }

    #[test]
    fn a_detached_head_keeps() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);
        git_ok(&created.worktree_root, &["checkout", "--detach"]);

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(verdict.reason().contains("detached HEAD"), "{}", verdict.reason());
    }

    #[test]
    fn a_branch_disagreement_keeps() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);
        git_ok(&created.worktree_root, &["checkout", "-b", "other-branch"]);

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(verdict.reason().contains("disagreement"), "{}", verdict.reason());
    }

    /// A re-registered worktree carries `branch: null` (session_init.rs:436's
    /// `RegisterSpec { branch: None, .. }`) — no real worktree fixture needed,
    /// the record alone is the whole test.
    #[test]
    fn a_null_branch_keeps() {
        let tmp = tempfile::tempdir().unwrap();
        let main = tmp.path().join("main");
        std::fs::create_dir_all(&main).unwrap();
        ws::register_workspace(
            &main,
            ws::RegisterSpec { id: "wt-null", kind: "worktree", root: "/nowhere", branch: None, base_sha: None },
            "2024-01-01T00:00:00.000Z",
        )
        .unwrap();

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: "wt-null",
            base_commit: "deadbeef",
            now_ms: 0.0,
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(verdict.reason().contains("no branch (null)"), "{}", verdict.reason());
    }

    #[test]
    fn a_dirty_tree_keeps() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);
        std::fs::write(created.worktree_root.join("untracked.txt"), "x").unwrap();

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(
            verdict.reason().contains("tracked-modified or untracked"),
            "{}",
            verdict.reason()
        );
    }

    /// D8a's blind spot, closed: `.bee/HANDOFF.json` is gitignored by
    /// `main_repo`'s own `.gitignore`, so the porcelain check alone would
    /// call this tree clean.
    #[test]
    fn a_present_handoff_keeps_even_though_the_tree_reads_clean() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);
        std::fs::write(created.worktree_root.join(".bee").join("HANDOFF.json"), "{}").unwrap();
        assert!(
            !is_tree_dirty(&created.worktree_root).unwrap(),
            "gitignored, so porcelain alone reads clean"
        );

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(verdict.reason().contains("HANDOFF.json"), "{}", verdict.reason());
    }

    #[test]
    fn a_non_empty_capture_queue_keeps_even_though_the_tree_reads_clean() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);
        std::fs::write(
            created.worktree_root.join(".bee").join("capture-queue.jsonl"),
            "{\"stub\":true}\n",
        )
        .unwrap();
        assert!(
            !is_tree_dirty(&created.worktree_root).unwrap(),
            "gitignored, so porcelain alone reads clean"
        );

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(verdict.reason().contains("capture-queue.jsonl"), "{}", verdict.reason());
    }

    /// An empty capture queue is NOT precious — only a non-empty one is.
    #[test]
    fn an_empty_capture_queue_does_not_keep_on_its_own() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);
        std::fs::write(created.worktree_root.join(".bee").join("capture-queue.jsonl"), "").unwrap();

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(verdict.is_dead(), "{}", verdict.reason());
    }

    #[test]
    fn an_interrupted_rebase_keeps() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);
        let admin_dir = main.join(".git").join("worktrees").join(&created.id);
        std::fs::write(admin_dir.join("MERGE_HEAD"), base_commit.clone()).unwrap();

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(verdict.reason().contains("MERGE_HEAD"), "{}", verdict.reason());
    }

    /// D2b: liveness comes from a session record naming THIS workspace with
    /// a fresh heartbeat — never `write_owner_session`/`attached_sessions`,
    /// which the only writer hardcodes to the main workspace
    /// (state_group/policy.rs:128) and are therefore null/empty here too.
    #[test]
    fn a_live_session_naming_this_workspace_keeps() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);
        let now = chrono::Utc::now();
        let now_ms = now.timestamp_millis() as f64;
        std::fs::create_dir_all(main.join(".bee").join("sessions")).unwrap();
        std::fs::write(
            main.join(".bee").join("sessions").join("sess-live.json"),
            jsjson::stringify(&json!({
                "id": "sess-live",
                "last_heartbeat": now.to_rfc3339(),
                "workspace_id": created.id,
            })),
        )
        .unwrap();

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms,
            liveness_seconds: PRUNE_LIVENESS_SECONDS,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(verdict.reason().contains("sess-live"), "{}", verdict.reason());
    }

    /// An unreadable session record counts as LIVE — this scan cannot rule
    /// it out, so it keeps rather than silently skipping it the way the
    /// fail-open `list_session_records` scans elsewhere in this codebase do.
    #[test]
    fn an_unreadable_session_record_keeps() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);
        std::fs::create_dir_all(main.join(".bee").join("sessions")).unwrap();
        std::fs::write(main.join(".bee").join("sessions").join("corrupt.json"), "{oops").unwrap();

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: PRUNE_LIVENESS_SECONDS,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(
            verdict.reason().contains("unreadable session record counts as live"),
            "{}",
            verdict.reason()
        );
    }

    #[test]
    fn a_worktree_younger_than_the_age_threshold_keeps() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 365.0 * 24.0 * 60.0 * 60.0 * 1000.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(
            verdict.reason().contains("younger than the age threshold"),
            "{}",
            verdict.reason()
        );
    }

    // ── prune.rs: the orphan verdict (wov-1) ────────────────────────────────
    //
    // Directory gone AND branch gone — the record is the only artifact
    // left. Evaluated BEFORE the merge test on purpose: a missing branch
    // can never pass `git merge-base --is-ancestor`, so without this branch
    // a truly gone worktree would keep forever, misread as "not provably
    // merged" by a test with no ref left to ask about.

    /// Neither the directory nor the branch survives. If the merge test
    /// (condition 1) ran first, a nonexistent branch could never pass
    /// `merge-base --is-ancestor` and the verdict would come back Kept
    /// ("not provably merged") — reaching Dead here proves the orphan check
    /// fires first and short-circuits the ancestry probe entirely.
    #[test]
    fn neither_directory_nor_branch_existing_is_dead_with_no_ancestry_probe() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let base_commit = head_sha(&main);
        ws::register_workspace(
            &main,
            ws::RegisterSpec {
                id: "wt-orphan",
                kind: "worktree",
                root: &tmp.path().join("nonexistent-wt").to_string_lossy(),
                branch: Some("ghost-branch-never-created"),
                base_sha: None,
            },
            "2024-01-01T00:00:00.000Z",
        )
        .unwrap();

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: "wt-orphan",
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Dead { orphan: true, .. }), "{verdict:?}");
        assert!(
            verdict.reason().contains("the workspace record is the only artifact left"),
            "{}",
            verdict.reason()
        );
        assert!(
            !verdict.reason().contains("not provably merged"),
            "the ancestry probe must never run for a true orphan: {}",
            verdict.reason()
        );
    }

    /// The directory is gone but the branch still exists — the branch may
    /// carry commits no other ref protects, so this must still keep. It
    /// keeps today via the existing detached-HEAD condition (`current_branch`
    /// fails to read a HEAD ref from a directory that is not there), not via
    /// the new orphan branch, which the conjunction never reaches.
    #[test]
    fn a_missing_directory_with_a_live_branch_still_keeps() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let base_commit = head_sha(&main);
        assert!(branch_exists(&main, &created.branch), "fixture must start with a real branch");
        std::fs::remove_dir_all(&created.worktree_root).unwrap();
        assert!(branch_exists(&main, &created.branch), "removing the directory must not remove the branch");

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: &created.id,
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(
            !verdict.reason().contains("the workspace record is the only artifact left"),
            "a live branch must keep it out of the orphan branch: {}",
            verdict.reason()
        );
    }

    /// The directory still stands but the branch is gone — the tree may
    /// hold uncommitted or ignored work the branch never saw, so this must
    /// still keep. It keeps via the existing merge test (a branch that does
    /// not exist can never pass `merge-base --is-ancestor`), never via the
    /// new orphan branch, which the conjunction never reaches.
    #[test]
    fn a_standing_directory_with_a_missing_branch_still_keeps() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let base_commit = head_sha(&main);
        let wt_dir = tmp.path().join("wt-standing-dir");
        std::fs::create_dir_all(&wt_dir).unwrap();
        ws::register_workspace(
            &main,
            ws::RegisterSpec {
                id: "wt-branchless",
                kind: "worktree",
                root: &wt_dir.to_string_lossy(),
                branch: Some("ghost-branch-2"),
                base_sha: None,
            },
            "2024-01-01T00:00:00.000Z",
        )
        .unwrap();
        assert!(!branch_exists(&main, "ghost-branch-2"), "the branch must genuinely not exist");

        let verdict = classify_worktree(&PruneCheck {
            main_root: &main,
            id: "wt-branchless",
            base_commit: &base_commit,
            now_ms: crate::verbs::reservations::now_ms(),
            liveness_seconds: 1.0,
            age_threshold_ms: 0.0,
        });
        assert!(matches!(verdict, Verdict::Kept { .. }), "{verdict:?}");
        assert!(
            !verdict.reason().contains("the workspace record is the only artifact left"),
            "a standing directory must keep it out of the orphan branch: {}",
            verdict.reason()
        );
    }

    /// The orphan teardown removes no directory and deletes no branch: it
    /// drops the record (and grant, and holds) through the same
    /// registry-only `teardown_worktree(.., None)` call `run_unregister`
    /// already uses. Proven by the run coming back `removed`, never `kept`
    /// — attempting `git worktree remove --force` on a path that was never a
    /// working tree, or `git branch -d` on a ref that never existed, would
    /// both fail and land the id in `kept_ids` instead.
    #[test]
    fn the_orphan_teardown_removes_no_directory_and_deletes_no_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        ws::register_workspace(
            &main,
            ws::RegisterSpec {
                id: "wt-orphan-teardown",
                kind: "worktree",
                root: &tmp.path().join("nonexistent-wt-2").to_string_lossy(),
                branch: Some("ghost-branch-teardown"),
                base_sha: None,
            },
            "2024-01-01T00:00:00.000Z",
        )
        .unwrap();

        let outcome = run_prune_core(&main, false, 0.0).unwrap();

        assert_eq!(outcome.removed_ids, vec!["wt-orphan-teardown".to_string()], "{:?}", outcome.entries);
        assert!(outcome.kept_ids.is_empty(), "{:?}", outcome.entries);
        assert_eq!(outcome.reclaimed_bytes, 0, "there was never a directory to reclaim bytes from");
        assert!(
            ws::read_workspace(&main, "wt-orphan-teardown").is_err(),
            "the orphan workspace record must be dropped"
        );
    }

    /// The bug this whole module exists to retire: a `rev-list --count`
    /// failure reads as `0`, i.e. "merged", for EVERY worktree at once. Here,
    /// `main` is not even a git repository, so every git subprocess the
    /// classifier runs fails outright — and every worktree still keeps.
    #[test]
    fn a_git_failure_at_any_probe_keeps_every_worktree_classified_in_the_run() {
        let tmp = tempfile::tempdir().unwrap();
        let main = tmp.path().join("not-a-repo");
        std::fs::create_dir_all(&main).unwrap();
        for (id, branch) in [("wt-a", "wt/a"), ("wt-b", "wt/b"), ("wt-c", "wt/c")] {
            let root = tmp.path().join(id);
            std::fs::create_dir_all(&root).unwrap();
            ws::register_workspace(
                &main,
                ws::RegisterSpec {
                    id,
                    kind: "worktree",
                    root: &root.to_string_lossy(),
                    branch: Some(branch),
                    base_sha: None,
                },
                "2024-01-01T00:00:00.000Z",
            )
            .unwrap();
        }

        for id in ["wt-a", "wt-b", "wt-c"] {
            let verdict = classify_worktree(&PruneCheck {
                main_root: &main,
                id,
                base_commit: "deadbeef",
                now_ms: 0.0,
                liveness_seconds: 1.0,
                age_threshold_ms: 0.0,
            });
            assert!(matches!(verdict, Verdict::Kept { .. }), "{id}: {verdict:?}");
        }
    }

    #[test]
    fn a_base_ref_that_does_not_resolve_refuses_the_whole_run() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let err = resolve_prune_base(&main, "definitely-not-a-ref").unwrap_err();
        assert!(err.contains("does not resolve"), "{err}");
        assert!(err.contains("refusing the whole prune run"), "{err}");
    }

    #[test]
    fn a_resolvable_base_ref_returns_the_current_sha() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let head = head_sha(&main);
        let resolved = resolve_prune_base(&main, "main").unwrap();
        assert_eq!(resolved, head);
    }

    // ── prune.rs: `run_prune_core` (wr-3, D2/D5) ────────────────────────────
    //
    // Runs the testable core directly — never `run_prune`, which resolves
    // roots off `std::env::current_dir()` and is exercised only through the
    // built binary (tests/registry_dispatch.rs), the same split every other
    // `run_*` handler in this module already keeps.

    /// `--dry-run` classifies a worktree that is otherwise fully dead and
    /// removes NOTHING: the directory, the workspace record and the grant
    /// all survive the run.
    #[test]
    fn dry_run_classifies_a_dead_worktree_and_removes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());

        let outcome = run_prune_core(&main, true, 0.0).unwrap();

        assert!(outcome.dry_run);
        assert!(outcome.removed_ids.is_empty(), "{:?}", outcome.removed_ids);
        assert!(outcome.kept_ids.is_empty(), "{:?}", outcome.kept_ids);
        assert_eq!(outcome.entries.len(), 1);
        assert_eq!(outcome.entries[0]["id"], json!(created.id));
        assert_eq!(outcome.entries[0]["verdict"], json!("dead"));
        assert_eq!(outcome.entries[0]["removed"], json!(false));
        assert!(outcome.reclaimed_bytes > 0, "a real worktree directory is not zero bytes");
        assert!(
            outcome.lines.iter().any(|l| l.starts_with(&format!("{}: would remove", created.id))),
            "{:?}",
            outcome.lines
        );

        // The three artifacts a real prune would have dropped are all still
        // there — this is the whole point of `--dry-run`.
        assert!(created.worktree_root.exists(), "the worktree directory must survive a dry run");
        assert!(ws::read_workspace(&main, &created.id).is_ok(), "the workspace record must survive a dry run");
        let grants = read_grants_strict(&main.join(".bee")).unwrap();
        assert_eq!(grants.get(&created.id), Some(&Value::Bool(true)), "the grant must survive a dry run");
    }

    /// D2/D5's union enumeration: a grant-driven scan alone never reaches a
    /// GRANTLESS orphan workspace record — exactly the shape `worktree
    /// unregister` leaves behind today (D3 closes that gap in `unregister`
    /// itself; this is the sweep that also clears what already leaked).
    /// Dropping the grant here reproduces that leak directly, then proves a
    /// real (non-dry-run) prune reaches the orphan through its workspace
    /// record and removes it — clearing exactly the CONTEXT.md-promised
    /// leftovers a grants-only scan would silently skip forever.
    #[test]
    fn a_grantless_orphan_workspace_record_is_reached_and_dropped() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());

        // Simulate the exact leak D3/D3a exists to close: the grant is gone,
        // the workspace record is not.
        let store = main.join(".bee");
        let mut grants = read_grants_strict(&store).unwrap();
        assert_eq!(grants.remove(&created.id), Some(Value::Bool(true)), "fixture must start granted");
        write_grants_file_atomic(&store, &grants).unwrap();
        assert_eq!(read_grants_strict(&store).unwrap().get(&created.id), None, "grant must be gone");
        assert!(ws::read_workspace(&main, &created.id).is_ok(), "the orphan record must still be there");

        let outcome = run_prune_core(&main, false, 0.0).unwrap();

        assert_eq!(outcome.removed_ids, vec![created.id.clone()], "{:?}", outcome.entries);
        assert!(outcome.kept_ids.is_empty(), "{:?}", outcome.kept_ids);
        assert!(outcome.reclaimed_bytes > 0);
        assert!(
            !created.worktree_root.exists(),
            "a real (non-dry-run) prune must remove the directory of a reached, dead orphan"
        );
        assert!(
            ws::read_workspace(&main, &created.id).is_err(),
            "the orphan workspace record must be dropped along with the directory"
        );
    }
