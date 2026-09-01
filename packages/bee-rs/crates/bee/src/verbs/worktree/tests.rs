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
use std::collections::HashSet;
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

    /// review B-P2-2: `run_register`'s own slug gate matches
    /// `create.rs`'s `feature_slug_ok` — `worktree register` never routes
    /// through `create_feature_worktree`'s validation (only `worktree new`
    /// does), so this is the ONLY thing standing between an unvalidated
    /// `--feature` value and `bootstrap_worktree_store`'s feature-keyed path
    /// joins (`archive/<feature>`). A path-traversal or absolute value must
    /// be refused BY NAME, never delegated or silently accepted.
    #[test]
    fn register_feature_refusal_matches_feature_slug_ok() {
        for feature in ["demo", "demo-2", "a", "0abc", "trailing-hyphen-"] {
            assert!(feature_slug_ok(feature), "{feature}");
            assert_eq!(
                register_feature_refusal(feature),
                None,
                "a valid slug must never be refused: {feature}"
            );
        }
        for feature in ["../../etc", "/etc/passwd", "Demo", "-leading-hyphen", "has space"] {
            assert!(!feature_slug_ok(feature), "{feature}");
            let refusal = register_feature_refusal(feature)
                .unwrap_or_else(|| panic!("a non-slug feature must be refused: {feature}"));
            assert!(
                refusal.contains(feature),
                "the refusal must name the exact value, got {refusal:?} for {feature}"
            );
        }
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
            // srg-2: `binary` rides beside `config` on BOTH return paths.
            vec!["created", "worktreeStoreRoot", "onboarding", "config", "binary", "identity", "state"]
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

    /// review D-P2-1: `git_tracked_cells`'s fail-CLOSED contract, pinned at
    /// the pure-parse layer rather than through a real `git` process — an
    /// entry that does not itself carry the `.bee/cells/` prefix must fail
    /// the WHOLE lookup, never just get skipped and leave the tracked set
    /// silently under-populated (an under-populated set reads exactly like
    /// "nothing is tracked", which is the shape that makes the prune arm
    /// above delete everything).
    #[test]
    fn tracked_cells_output_fails_closed_on_an_unexpected_entry() {
        let well_formed = b".bee/cells/a-1.json\0.bee/cells/archive/a/a-0.json\0";
        let set = parse_git_ls_files_cells_output(well_formed).expect("well-formed output must parse");
        assert_eq!(
            set,
            HashSet::from([String::from("a-1.json"), String::from("archive/a/a-0.json")])
        );

        // One entry that never carries the `.bee/cells/` prefix at all.
        let unexpected = b".bee/cells/a-1.json\0.bee/config.json\0";
        assert_eq!(
            parse_git_ls_files_cells_output(unexpected),
            None,
            "an entry outside .bee/cells/ must fail the whole lookup closed, not just be skipped"
        );

        // Empty output (nothing tracked under the pathspec) still parses to
        // an empty set — this is not the failure shape.
        assert_eq!(parse_git_ls_files_cells_output(b""), Some(HashSet::new()));
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

    /// review D-P3-1: `GIT_CEILING_DIRECTORIES` is process environment, so a
    /// bare `set_var` around one assertion would leak into every other test
    /// that spawns `git` while it was set. Scoped to exactly the life of one
    /// test: `new` records whatever value (or absence) was already there and
    /// pins the ceiling to `dir`; `Drop` puts it straight back — a caller can
    /// never forget to unwind it, even on a panicking assert. Lifted from
    /// `verbs/drivers/close.rs`'s own `GitCeilingGuard` (P3-4) rather than
    /// shared, because that struct is private to close.rs's own test module
    /// and this cell's scope does not touch close.rs beyond its fixture seed
    /// line — keep the two in sync by hand if either changes.
    struct GitCeilingGuard {
        prior: Option<std::ffi::OsString>,
    }

    impl GitCeilingGuard {
        fn new(dir: &Path) -> Self {
            let prior = std::env::var_os("GIT_CEILING_DIRECTORIES");
            // SAFETY: no other thread reads/writes this specific var across
            // this guard's lifetime — nothing else in this crate consults
            // GIT_CEILING_DIRECTORIES, and it exists only to steer this one
            // test's own `git` child processes.
            unsafe { std::env::set_var("GIT_CEILING_DIRECTORIES", dir) };
            GitCeilingGuard { prior }
        }
    }

    impl Drop for GitCeilingGuard {
        fn drop(&mut self) {
            // SAFETY: see `new` above.
            match self.prior.take() {
                Some(v) => unsafe { std::env::set_var("GIT_CEILING_DIRECTORIES", v) },
                None => unsafe { std::env::remove_var("GIT_CEILING_DIRECTORIES") },
            }
        }
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

        // review D-P3-1: pin the ceiling to the tempdir's own parent for the
        // life of this test — a TMPDIR that happens to sit under a real git
        // checkout must never let `git ls-files` walk up into that enclosing
        // repo and answer with a real (but irrelevant) tracked set instead of
        // the "not a repo" shape this test exists to pin.
        let _ceiling = GitCeilingGuard::new(tmp.path());

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

    /// review B-P2-7 / D-P3-1 fixture: the ISLAND's `archive/<feature>`
    /// subdir itself — the exact join `fs::copy` writes through at the
    /// bottom of `sync_worktree_cells` — is a SYMLINK to a victim directory.
    /// Before this fix `dest_archive.join(feature)` was never in the checked
    /// set (only its parent, `cells/archive`, was), so `create_dir_all`
    /// no-op'd on the existing link and `fs::copy` landed straight in the
    /// link's target. This must be RED against the pre-fix code: the victim
    /// stays byte-identical, and the whole sync is refused before ANY
    /// prune/fill runs (same whole-function skip the other B-P1-1 fixtures
    /// pin), not just the archive-fill step.
    #[test]
    fn bootstrap_refuses_a_symlinked_feature_archive_subdir_before_any_copy() {
        if !symlink_capable() {
            eprintln!("SKIP (env-limited: {SYMLINK_CAP}) — symlinked feature archive subdir refusal");
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let main_store = tmp.path().join("main").join(".bee");
        let src_feature_archive = main_store.join("cells").join("archive").join("a");
        std::fs::create_dir_all(&src_feature_archive).unwrap();
        std::fs::write(
            src_feature_archive.join("a-0.json"),
            "{\"id\":\"a-0\",\"feature\":\"a\",\"status\":\"capped\"}",
        )
        .unwrap();

        // The victim: a directory with its own file, nothing to do with bee.
        let victim = tmp.path().join("victim-feature-archive");
        std::fs::create_dir_all(&victim).unwrap();
        std::fs::write(victim.join("keep.json"), "{\"unrelated\":true}").unwrap();

        let wt = tmp.path().join("wt-a");
        let wt_archive = wt.join(".bee").join("cells").join("archive");
        std::fs::create_dir_all(&wt_archive).unwrap();
        let wt_feature_archive = wt_archive.join("a");
        symlink_dir(&victim.to_string_lossy(), &wt_feature_archive).unwrap();

        let report = bootstrap_worktree_store(&wt, &main_store, "a").unwrap();

        assert_eq!(std::fs::read_dir(&victim).unwrap().count(), 1, "victim dir must keep only its own file");
        assert!(!victim.join("a-0.json").exists(), "fs::copy must never write into the symlink's target");
        assert!(
            std::fs::symlink_metadata(&wt_feature_archive).unwrap().file_type().is_symlink(),
            "the symlink itself must be left untouched"
        );

        let sync = report.get("cellsSync").expect("bootstrap report must name the symlink skip");
        assert_eq!(sync.get("skipped"), Some(&Value::Bool(true)));
        assert_eq!(sync.get("path"), Some(&json!(p(&wt_feature_archive))));
    }

    /// review D-P3-2: an untracked foreign cell/archive file actually pruned
    /// gets named in the bootstrap report's `pruned` list; the common
    /// nothing-pruned case omits the key entirely rather than reporting `[]`.
    #[test]
    fn bootstrap_reports_pruned_file_names_only_when_something_was_removed() {
        let tmp = tempfile::tempdir().unwrap();
        let main_store = tmp.path().join("main").join(".bee");
        std::fs::create_dir_all(main_store.join("cells")).unwrap();

        let wt = tmp.path().join("wt-a");
        let wt_cells = wt.join(".bee").join("cells");
        std::fs::create_dir_all(&wt_cells).unwrap();
        std::fs::write(wt_cells.join("a-1.json"), "{\"id\":\"a-1\",\"feature\":\"a\",\"status\":\"open\"}")
            .unwrap();
        let wt_archive_c = wt_cells.join("archive").join("c");
        std::fs::create_dir_all(&wt_archive_c).unwrap();
        std::fs::write(wt_archive_c.join("c-0.json"), "{\"id\":\"c-0\",\"feature\":\"c\",\"status\":\"capped\"}")
            .unwrap();
        // An untracked foreign top-level cell and archive file — both
        // prunable, since `git_init` below never commits either.
        std::fs::write(wt_cells.join("c-1.json"), "{\"id\":\"c-1\",\"feature\":\"c\",\"status\":\"open\"}")
            .unwrap();
        git_init(&wt);

        let result = bootstrap_worktree_store(&wt, &main_store, "a").unwrap();
        let pruned = result.get("pruned").and_then(Value::as_array).expect("pruned must be reported");
        let names: Vec<&str> = pruned.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(names.contains(&"c-1.json"), "{names:?}");
        assert!(names.contains(&"archive/c/c-0.json"), "{names:?}");
        assert_eq!(names.len(), 2, "{names:?}");
        assert!(wt_cells.join("a-1.json").exists(), "the granted feature's own cell must stay");

        // The nothing-pruned case: the key is absent entirely.
        let wt2 = tmp.path().join("wt-b");
        std::fs::create_dir_all(&wt2).unwrap();
        let result2 = bootstrap_worktree_store(&wt2, &main_store, "a").unwrap();
        assert!(result2.get("pruned").is_none(), "an empty prune must omit the key rather than report []");
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

    /// review B-P2-7 / D-P3-1: `worktree new`'s own result/text must carry
    /// the SAME `cellsSync` skip note `worktree register` already surfaces —
    /// pinned at the pure `new_result_and_text` layer (no cwd, no real
    /// `create_feature_worktree` call needed) so a `Created` whose bootstrap
    /// map carries `cellsSync` is enough to prove it.
    #[test]
    fn new_result_and_text_carries_the_cells_sync_skip_note() {
        let created = Created {
            id: "wt-demo".to_string(),
            worktree_root: PathBuf::from("/tmp/wt-demo"),
            branch: "wt/demo".to_string(),
            base_ref: None,
            base_ref_sha: None,
            bootstrap: map_of(&[
                ("created", json!(true)),
                ("worktreeStoreRoot", json!("/tmp/wt-demo/.bee")),
                (
                    "cellsSync",
                    json!({
                        "skipped": true,
                        "path": "/tmp/wt-demo/.bee/cells",
                        "reason": "refusing to sync .bee/cells through a symlinked path",
                    }),
                ),
            ]),
            companion: Value::Null,
            skills_sync: json!({ "applied": true }),
        };
        let (result, text) = new_result_and_text("demo", &created, "next step text");

        let sync = result.get("cellsSync").expect("result must carry the bootstrap map's cellsSync");
        assert_eq!(sync["skipped"], Value::Bool(true));
        assert!(
            text.contains(
                "cells sync skipped — /tmp/wt-demo/.bee/cells: refusing to sync .bee/cells through a symlinked path"
            ),
            "{text}"
        );
    }

    /// The common case: no `cellsSync` in the bootstrap map means neither the
    /// result nor the text carries any skip note.
    #[test]
    fn new_result_and_text_carries_no_cells_sync_key_when_nothing_was_skipped() {
        let created = Created {
            id: "wt-demo".to_string(),
            worktree_root: PathBuf::from("/tmp/wt-demo"),
            branch: "wt/demo".to_string(),
            base_ref: None,
            base_ref_sha: None,
            bootstrap: map_of(&[("created", json!(true)), ("worktreeStoreRoot", json!("/tmp/wt-demo/.bee"))]),
            companion: Value::Null,
            skills_sync: json!({ "applied": true }),
        };
        let (result, text) = new_result_and_text("demo", &created, "next step text");
        assert!(result.get("cellsSync").is_none());
        assert!(!text.contains("cells sync skipped"));
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

    /// D7/D8 (docs/history/test-doctrine/CONTEXT.md, td-3): a capped cell,
    /// written straight into the MAIN checkout's `.bee/cells/` the same way
    /// `bee cells finish` would leave it — `feature_proof_check`'s own read
    /// path (`verbs/cells/proof.rs`), not a fixture-only shape.
    fn write_capped_cell(main: &Path, id: &str, feature: &str, report: Option<Value>) {
        let mut cell = json!({
            "id": id,
            "feature": feature,
            "status": "capped",
        });
        if let Some(report) = report {
            cell["trace"] = json!({ "report": report });
        }
        let dir = main.join(".bee").join("cells");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), cell.to_string()).unwrap();
    }

    /// Same live-store shape `write_capped_cell` leaves, plus
    /// `trace.files_changed` — the field `feature_touched_files`
    /// (drivers/close.rs) reads to learn which paths a capped cell of this
    /// feature actually touched. kss-1: this is how a merging feature earns
    /// a `docs/knowledge/` path into its own scoped auto-commit.
    fn write_capped_cell_with_files(main: &Path, id: &str, feature: &str, files: &[&str]) {
        let cell = json!({
            "id": id,
            "feature": feature,
            "status": "capped",
            "trace": { "files_changed": files },
        });
        let dir = main.join(".bee").join("cells");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), cell.to_string()).unwrap();
    }

    fn valid_proof_report() -> Value {
        json!({
            "outcome": "did the thing",
            "commit": "abc123",
            "files": ["src/a.rs"],
            "tests": "cargo test -p bee — green — touched a.rs",
            "deviations": [],
        })
    }

    /// D7/D8: a feature whose every capped cell carries a valid D8 proof
    /// line merges straight through — no `commands.test` spawn, no verify
    /// child, just the door reading `trace.report.tests`.
    #[test]
    fn a_fully_proofed_merge_proceeds_to_normal_finish() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "proofed", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();
        std::fs::write(wt.join("f.txt"), "y").unwrap();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        git_ok(&wt, &["commit", "-qam", "work"]);

        write_capped_cell(&main, "proofed-1", "proofed", Some(valid_proof_report()));

        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert_eq!(answer.result["verify"], json!("proven (1 cell(s))"));
    }

    /// D4 (docs/history/proof-strength-and-expiry/CONTEXT.md): a cap whose
    /// recorded proof was taken before the merge base — main moved and the
    /// branch took it in afterwards — is NAMED by the `proof-stale`
    /// advisory, in the JSON result and in the printed text, and the merge
    /// lands anyway. The two silent shapes ride the same setup, because
    /// they are only meaningful beside a row that DID fire: a cap recording
    /// the `"none"` commit sentinel, and one recording a sha this repo does
    /// not have (a git call that cannot answer). Fail open: a warning that
    /// is wrong is worse than no warning.
    #[test]
    fn a_proof_taken_before_the_merge_base_is_named_and_the_merge_still_lands() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "aged", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        std::fs::write(wt.join("f.txt"), "y").unwrap();
        git_ok(&wt, &["commit", "-qam", "work"]);
        let proved_at = head_sha(&wt);

        // Main moves on, and the branch takes main in — which is what walks
        // the merge base forward, past the commit the cap was proven at.
        std::fs::write(main.join("m.txt"), "m").unwrap();
        git_ok(&main, &["add", "m.txt"]);
        git_ok(&main, &["commit", "-qm", "main moves"]);
        git_ok(&wt, &["merge", "--no-edit", "main"]);

        let report_at = |commit: &str| {
            json!({
                "outcome": "did the thing",
                "commit": commit,
                "files": ["src/a.rs"],
                "tests": "cargo test -p bee — green:unit — touched a.rs",
                "deviations": [],
            })
        };
        write_capped_cell(&main, "aged-1", "aged", Some(report_at(&proved_at)));
        write_capped_cell(&main, "aged-2", "aged", Some(report_at("none")));
        write_capped_cell(&main, "aged-3", "aged", Some(report_at("abc123")));

        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("the advisory must never refuse a merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true), "the merge lands anyway");
        let stale = &answer.result["proof_stale"];
        assert_eq!(stale["cells"], json!(["aged-1"]), "{:?}", answer.result);
        assert!(
            jsjson::js_to_string(&stale["message"]).contains("aged-1"),
            "the advisory names the cell: {stale:?}"
        );
        // It is PRINTED, not just carried in the JSON result.
        let printed = merge_text_lines(&created.id, &main, &answer).join("\n");
        assert!(printed.contains("proof-stale: "), "{printed}");
        assert!(printed.contains("aged-1"), "{printed}");
    }

    /// D4: the ordinary merge — every cap was proven on the line that
    /// descends from the merge base — says nothing at all. An advisory that
    /// fires on a clean merge is noise, so this is the case that matters
    /// most: no key in the result, no line in the text.
    #[test]
    fn a_proof_that_descends_from_the_merge_base_is_silent() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "fresh", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        std::fs::write(wt.join("f.txt"), "y").unwrap();
        git_ok(&wt, &["commit", "-qam", "work"]);

        write_capped_cell(
            &main,
            "fresh-1",
            "fresh",
            Some(json!({
                "outcome": "did the thing",
                "commit": head_sha(&wt),
                "files": ["src/a.rs"],
                "tests": "cargo test -p bee — green:unit — touched a.rs",
                "deviations": [],
            })),
        );

        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert!(
            answer.result.get("proof_stale").is_none(),
            "a fresh proof must say nothing: {:?}",
            answer.result
        );
        let printed = merge_text_lines(&created.id, &main, &answer).join("\n");
        assert!(!printed.contains("proof-stale"), "{printed}");
    }

    /// D7/D8: a capped cell that carries a `trace.report` but no VALID D8
    /// proof line (here: an empty `tests` string) refuses the merge —
    /// zero-mutation, naming the cell — before `git merge` ever runs.
    #[test]
    fn a_present_but_empty_proof_refuses_the_merge_naming_the_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "unproofed", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();
        std::fs::write(wt.join("f.txt"), "y").unwrap();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        git_ok(&wt, &["commit", "-qam", "work"]);

        write_capped_cell(
            &main,
            "unproofed-1",
            "unproofed",
            Some(json!({
                "outcome": "did the thing",
                "commit": "abc123",
                "files": [],
                "tests": "",
                "deviations": [],
            })),
        );

        let pre_merge_head =
            js_trim(&run_git(&main, &["rev-parse", "HEAD"]).stdout.unwrap_or_default())
                .to_string();
        let message = match merge_feature_worktree(&main, &created.id, false, None, true, None) {
            Ok(answer) => panic!(
                "a present-but-empty proof line must refuse, never merge silently: {:?}",
                answer.result
            ),
            Err(MErr::Thrown(m)) => m,
            Err(MErr::Ex) => panic!("merge delegated instead of refusing"),
        };
        assert!(message.starts_with("[WORKTREE_MERGE_PROOF_DEBT] "), "{message}");
        assert!(message.contains("unproofed-1"), "{message}");
        // Zero mutation: HEAD on main never moved, and the worktree stands.
        let head_after =
            js_trim(&run_git(&main, &["rev-parse", "HEAD"]).stdout.unwrap_or_default())
                .to_string();
        assert_eq!(head_after, pre_merge_head, "a refused merge must never touch main");
        assert!(wt.exists(), "the worktree stands — nothing was torn down");
    }

    /// D7/D8: a capped cell with NO `trace.report` at all — a legacy cap
    /// from before `--report` was required — passes ungated, and the
    /// merge's own `verify` field names it as legacy rather than claiming
    /// something was proven.
    #[test]
    fn a_legacy_report_less_cap_merges_with_a_named_note() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "legacy", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();
        std::fs::write(wt.join("f.txt"), "y").unwrap();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        git_ok(&wt, &["commit", "-qam", "work"]);

        write_capped_cell(&main, "legacy-1", "legacy", None);

        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "a report-less legacy cap must never block a merge: {:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert_eq!(
            answer.result["verify"],
            json!("unchecked (1 legacy cap(s), no proof line)")
        );
    }

    // ── slp-dissent-stop-and-ask sd-5: the merge door's dissent precondition ──
    //
    // a2affcba names BOTH doors, and `bee worktree merge` had no cell-debt
    // precondition but proof debt. These cases sit beside the proof cases
    // above on purpose: same slot, same zero-mutation posture, same feature.
    //
    // Two of them invert the proof twins deliberately — the dissent helper
    // counts a cell in ANY status (a blocker dissent parks its cell as
    // `blocked`, never `capped`), and the refusal carries a named escape
    // that the escape-less proof refusal does not have.

    /// A cell of `feature` carrying ONE dissent entry, written straight into
    /// the MAIN checkout's live `.bee/cells/` the same way `bee cells
    /// dissent` leaves it — `feature_dissent_debt`'s own read path
    /// (verbs/cells/dissent.rs), not a fixture-only shape. `status` is a
    /// parameter on purpose: the helper counts every status.
    ///
    /// A `capped` cell also gets a VALID proof report, because the proof
    /// door sits above the dissent door and would otherwise mask it.
    fn write_dissenting_cell(
        main: &Path,
        id: &str,
        feature: &str,
        status: &str,
        verdict: Option<&str>,
    ) {
        let mut entry = json!({
            "target": id,
            "claim": "the cell's shape is wrong",
            "alternative": "do it the other way",
            "severity": "blocker",
            "recorded_at": "2026-08-28T00:00:00.000Z",
        });
        if let Some(v) = verdict {
            let m = entry.as_object_mut().unwrap();
            m.insert("verdict".into(), json!(v));
            m.insert("verdict_reason".into(), json!("because"));
            m.insert("answered_at".into(), json!("2026-08-28T01:00:00.000Z"));
        }
        let mut trace = json!({ "dissent": [entry] });
        if status == "capped" {
            trace["report"] = valid_proof_report();
        }
        let cell = json!({
            "id": id,
            "feature": feature,
            "status": status,
            "trace": trace,
        });
        let dir = main.join(".bee").join("cells");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), cell.to_string()).unwrap();
    }

    /// The named escape: one `decisions.jsonl` row tagged `dissent-deferral`
    /// whose text names `named` — the same row shape the close door's own
    /// deferral case writes (verbs/cells/tests.rs).
    fn write_dissent_deferral(main: &Path, named: &str) {
        std::fs::write(
            main.join(".bee").join("decisions.jsonl"),
            format!(
                "{{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-08-28T00:00:00.000Z\",\"decision\":\"defer the dissent for {named}\",\"rationale\":\"r\",\"tags\":[\"dissent-deferral\"],\"scope\":\"repo\"}}\n"
            ),
        )
        .unwrap();
    }

    /// sd-5: an unanswered dissent on ANY cell of the merging feature refuses
    /// the merge, zero-mutation, naming EVERY offending cell — including a
    /// cell that was never capped, which is the normal state of a cell a
    /// blocker dissent just parked.
    #[test]
    fn an_unanswered_dissent_refuses_the_merge_naming_every_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "dissenting");
        let wt = created.worktree_root.clone();

        // NOT capped: the helper counts every status on purpose.
        write_dissenting_cell(&main, "dissenting-1", "dissenting", "blocked", None);
        // Capped AND fully proven: proves the proof door is not what refuses.
        write_dissenting_cell(&main, "dissenting-2", "dissenting", "capped", None);

        let pre_merge_head =
            js_trim(&run_git(&main, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
        let message = match merge_feature_worktree(&main, &created.id, false, None, true, None) {
            Ok(answer) => panic!(
                "an unanswered dissent must refuse, never merge silently: {:?}",
                answer.result
            ),
            Err(MErr::Thrown(m)) => m,
            Err(MErr::Ex) => panic!("merge delegated instead of refusing"),
        };
        assert!(message.starts_with("[WORKTREE_MERGE_DISSENT_DEBT] "), "{message}");
        assert!(message.contains("dissenting-1"), "the uncapped offender is named: {message}");
        assert!(message.contains("dissenting-2"), "every offender is named: {message}");
        assert!(
            message.contains("bee cells dissent-verdict"),
            "the refusal carries its own remedy: {message}"
        );
        assert!(message.contains("dissent-deferral"), "the refusal names its escape: {message}");
        // Zero mutation: HEAD on main never moved, and the worktree stands.
        let head_after =
            js_trim(&run_git(&main, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
        assert_eq!(head_after, pre_merge_head, "a refused merge must never touch main");
        assert!(wt.exists(), "the worktree stands — nothing was torn down");
    }

    /// sd-5: recording the verdict clears the precondition with NO other
    /// change to the fixture — same cell, same status, one `verdict` key.
    #[test]
    fn a_recorded_dissent_verdict_lets_the_same_merge_proceed() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "answered");
        write_dissenting_cell(&main, "answered-1", "answered", "blocked", Some("accept"));

        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("an answered dissent must not refuse: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    /// sd-5: the named escape. A logged `dissent-deferral` decision naming
    /// THIS feature lets the merge through; the same tag naming a different
    /// feature never does.
    #[test]
    fn a_logged_dissent_deferral_naming_this_feature_lets_the_merge_through() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "deferred");
        write_dissenting_cell(&main, "deferred-1", "deferred", "blocked", None);

        // The same tag naming SOMEONE ELSE never lifts this block.
        write_dissent_deferral(&main, "elsewhere");
        let message = match merge_feature_worktree(&main, &created.id, false, None, true, None) {
            Ok(answer) => panic!(
                "a deferral naming another feature must not clear this door: {:?}",
                answer.result
            ),
            Err(MErr::Thrown(m)) => m,
            Err(MErr::Ex) => panic!("merge delegated instead of refusing"),
        };
        assert!(message.starts_with("[WORKTREE_MERGE_DISSENT_DEBT] "), "{message}");

        // Naming THIS feature does.
        write_dissent_deferral(&main, "deferred");
        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("a logged deferral must clear the door: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    /// sd-5: a worktree whose identity carries NO feature merges exactly as
    /// it does today — the dissent helper is never called at all, so cells
    /// carrying unanswered dissents under some other feature slug cannot
    /// reach it. The same posture the proof door's `None` arm takes.
    #[test]
    fn a_worktree_with_no_resolvable_feature_merges_past_the_dissent_door() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "nofeat");
        let wt = created.worktree_root.clone();

        // Both identity sources removed: `resolve_worktree_feature` reads
        // `None`, and the branch check falls back to the `wt/<slug>` shape.
        let _ =
            std::fs::remove_file(wt.join(".bee").join("runtime").join("worktree-identity.json"));
        let _ = std::fs::remove_file(wt.join(".bee").join("state.json"));

        write_dissenting_cell(&main, "nofeat-1", "nofeat", "blocked", None);

        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("an unresolved feature must merge ungated: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    /// sd-5: a `WORKTREE_MERGE_DISSENT_DEBT` refusal is a zero-mutation
    /// precondition too — it writes nothing to the merging feature's lane
    /// either. The twin of
    /// `a_proof_debt_refusal_leaves_the_lane_record_byte_identical`.
    #[test]
    fn a_dissent_debt_refusal_leaves_the_lane_record_byte_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "dissenting");
        write_dissenting_cell(&main, "dissenting-1", "dissenting", "blocked", None);
        write_stranded_lane(&main, "dissenting", "scribing");
        let lane_path = main.join(".bee").join("lanes").join("dissenting.json");
        let before = std::fs::read(&lane_path).unwrap();

        let result = merge_feature_worktree(&main, &created.id, false, None, true, None);
        let Err(err) = result else { panic!("an unanswered dissent must still refuse") };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_DISSENT_DEBT"), "{msg}");

        let after = std::fs::read(&lane_path).unwrap();
        assert_eq!(before, after, "a dissent-debt refusal must not touch the lane record at all");
    }

    /// sd-5, THE POINT OF THE PHASE: ONE fixture state — a cell carrying an
    /// unanswered dissent plus ONE `dissent-deferral` row naming the feature
    /// — clears BOTH doors. `bee close`'s dissent-debt door reads
    /// NON-BLOCKING and `bee worktree merge` proceeds past its precondition,
    /// because both read the SAME two functions. An escape that worked at
    /// one door and not the other is the defect this pins.
    #[test]
    fn one_dissent_deferral_clears_both_the_close_door_and_the_merge_precondition() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "bothdoors");
        write_dissenting_cell(&main, "bothdoors-1", "bothdoors", "blocked", None);

        // Before the escape: BOTH doors say no.
        let doors = crate::verbs::drivers::build_close_report_doors(&main, "bothdoors").unwrap();
        let door = doors.iter().find(|d| d.door == "dissent-debt").unwrap();
        assert!(door.blocking, "the close door blocks first: {}", door.detail);

        write_dissent_deferral(&main, "bothdoors");

        // After it: the close door reads non-blocking...
        let doors = crate::verbs::drivers::build_close_report_doors(&main, "bothdoors").unwrap();
        let door = doors.iter().find(|d| d.door == "dissent-debt").unwrap();
        assert!(!door.blocking, "one escape, close door: {}", door.detail);
        assert!(door.detail.contains("bothdoors-1"), "the count is untouched: {}", door.detail);

        // ...and the merge walks past its precondition on the SAME state.
        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("one escape must clear the merge door too: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    // ── slp-advisor-nudge an-3: the advisor-nudge debt at the merge door ───
    //
    // Placed beside the dissent-debt merge tests above for the same reason
    // they sit beside the proof ones: same slot, same zero-mutation posture.

    /// One `advisor-nudge` mailbox row in the MAIN checkout's live store —
    /// the record-event shape `bee supervisor record` appends, read by
    /// `feature_advisor_nudge_debt` (verbs/supervisor.rs). The `feature` is
    /// the one DERIVED onto the row at record time.
    fn write_advisor_nudge_row(main: &Path, id: &str, feature: Option<&str>) {
        let dir = main.join(".bee").join("supervisor");
        std::fs::create_dir_all(&dir).unwrap();
        let mut row = json!({
            "event": "record",
            "id": id,
            "ts": "2026-08-29T00:00:00.000Z",
            "kind": "advisor-nudge",
            "signal": "struggling-loop",
            "point_key": id,
            "question": "Would an advisor read help here?",
            "target_session": "sess-1",
        });
        if let Some(feature) = feature {
            row.as_object_mut().unwrap().insert("feature".into(), json!(feature));
        }
        let path = dir.join("interventions.jsonl");
        let mut text = std::fs::read_to_string(&path).unwrap_or_default();
        text.push_str(&format!("{row}\n"));
        std::fs::write(&path, text).unwrap();
    }

    /// The per-row escape: one decision tagged `advisor-nudge` naming the row.
    fn write_advisor_nudge_clearing(main: &Path, decision_id: &str, row_id: &str) {
        std::fs::create_dir_all(main.join(".bee")).unwrap();
        let event = json!({
            "id": decision_id,
            "type": "decide",
            "date": "2026-08-29T01:00:00.000Z",
            "decision": format!("declined the advisor consult for {row_id}: the loop already broke"),
            "rationale": "r",
            "tags": ["advisor-nudge"],
            "scope": "repo",
        });
        let path = main.join(".bee").join("decisions.jsonl");
        let mut text = std::fs::read_to_string(&path).unwrap_or_default();
        text.push_str(&format!("{event}\n"));
        std::fs::write(&path, text).unwrap();
    }

    /// an-3: an unanswered advisor nudge for the merging feature refuses the
    /// merge, zero-mutation, naming every offending row — and the SAME
    /// fixture merges once one clearing decision lands. One test, because the
    /// second half is the only proof the refusal has a working remedy.
    #[test]
    fn an_unanswered_advisor_nudge_refuses_the_merge_until_a_decision_clears_it() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "nudged");
        let wt = created.worktree_root.clone();

        write_advisor_nudge_row(&main, "nud-1", Some("nudged"));
        write_advisor_nudge_row(&main, "nud-2", Some("nudged"));
        // Neither of these is this feature's debt.
        write_advisor_nudge_row(&main, "nud-3", Some("elsewhere"));
        write_advisor_nudge_row(&main, "nud-4", None);

        let pre_merge_head =
            js_trim(&run_git(&main, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
        let message = match merge_feature_worktree(&main, &created.id, false, None, true, None) {
            Ok(answer) => panic!(
                "an unanswered advisor nudge must refuse, never merge silently: {:?}",
                answer.result
            ),
            Err(MErr::Thrown(m)) => m,
            Err(MErr::Ex) => panic!("merge delegated instead of refusing"),
        };
        assert!(message.starts_with("[WORKTREE_MERGE_ADVISOR_NUDGE_DEBT] "), "{message}");
        assert!(message.contains("nud-1"), "every offender is named: {message}");
        assert!(message.contains("nud-2"), "every offender is named: {message}");
        assert!(!message.contains("nud-3"), "another feature's nudge is not named: {message}");
        assert!(!message.contains("nud-4"), "a feature-less nudge is not named: {message}");
        assert!(
            message.contains("bee decisions log"),
            "the refusal carries its own remedy: {message}"
        );
        assert!(message.contains("advisor-nudge"), "the refusal names its tag: {message}");

        // Zero mutation: HEAD on main never moved, and the worktree stands.
        let head_after =
            js_trim(&run_git(&main, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
        assert_eq!(head_after, pre_merge_head, "a refused merge must never touch main");
        assert!(wt.exists(), "the worktree stands — nothing was torn down");

        // One row answered is not enough — the escape is PER ROW.
        write_advisor_nudge_clearing(&main, "d1", "nud-1");
        let message = match merge_feature_worktree(&main, &created.id, false, None, true, None) {
            Ok(answer) => panic!("one row of two answered must still refuse: {:?}", answer.result),
            Err(MErr::Thrown(m)) => m,
            Err(MErr::Ex) => panic!("merge delegated instead of refusing"),
        };
        assert!(message.contains("nud-2"), "the row still owed is named: {message}");
        assert!(!message.contains("nud-1"), "the answered row is gone: {message}");

        // Both answered — the same merge goes through.
        write_advisor_nudge_clearing(&main, "d2", "nud-2");
        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("an answered nudge must not refuse: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    /// an-3: ONE fixture state — an unanswered nudge for the feature — is
    /// read the SAME way by `bee close`'s door and the merge precondition,
    /// and ONE clearing decision opens BOTH. A door that answers differently
    /// from its twin is the defect this pins.
    #[test]
    fn one_advisor_nudge_decision_clears_both_the_close_door_and_the_merge() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "bothnudge");
        write_advisor_nudge_row(&main, "nud-1", Some("bothnudge"));

        let doors = crate::verbs::drivers::build_close_report_doors(&main, "bothnudge").unwrap();
        let door = doors.iter().find(|d| d.door == "advisor-nudge-debt").unwrap();
        assert!(door.blocking, "the close door blocks first: {}", door.detail);

        write_advisor_nudge_clearing(&main, "d1", "nud-1");

        let doors = crate::verbs::drivers::build_close_report_doors(&main, "bothnudge").unwrap();
        let door = doors.iter().find(|d| d.door == "advisor-nudge-debt").unwrap();
        assert!(!door.blocking, "one escape, close door: {}", door.detail);

        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("one escape must clear the merge door too: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    /// an-3: a worktree whose identity carries NO feature merges ungated —
    /// the nudge helper is never called at all. The same `None` posture the
    /// proof and dissent doors above both take.
    #[test]
    fn a_worktree_with_no_resolvable_feature_merges_past_the_advisor_nudge_door() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "nonudgefeat");
        let wt = created.worktree_root.clone();
        let _ =
            std::fs::remove_file(wt.join(".bee").join("runtime").join("worktree-identity.json"));
        let _ = std::fs::remove_file(wt.join(".bee").join("state.json"));

        write_advisor_nudge_row(&main, "nud-1", Some("nonudgefeat"));

        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("an unresolved feature must merge ungated: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
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

    /// mlsg-1: a live session's cwd must never be deleted out from under it.
    /// A fresh-heartbeat session record naming this worktree's id refuses
    /// cleanup BEFORE `teardown_worktree` runs — the worktree directory is
    /// still there afterward, unlike every other refusal shape above.
    #[test]
    fn cleanup_refuses_while_a_live_session_holds_the_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "live", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();

        std::fs::create_dir_all(main.join(".bee").join("sessions")).unwrap();
        std::fs::write(
            main.join(".bee").join("sessions").join("sess-live.json"),
            jsjson::stringify(&json!({
                "id": "sess-live",
                "last_heartbeat": chrono::Utc::now().to_rfc3339(),
                "workspace_id": created.id,
            })),
        )
        .unwrap();

        let out = perform_cleanup(&main, &wt, &created.branch, &created.id, false);
        assert_eq!(out["ok"], Value::Bool(false));
        assert_eq!(out["code"], json!("WORKTREE_MERGE_CLEANUP_LIVE_SESSION"));
        assert!(wt.exists(), "the worktree directory must survive a live-session refusal");
    }

    /// mlsg-1: the same session record, but its heartbeat is well past
    /// `HEARTBEAT_STALE_SECONDS` (900s) — stale, so it does not hold the
    /// worktree and the live-session refusal must not fire.
    #[test]
    fn cleanup_ignores_a_session_with_a_stale_heartbeat() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "stale", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();

        let stale = chrono::Utc::now() - chrono::Duration::seconds(1000);
        std::fs::create_dir_all(main.join(".bee").join("sessions")).unwrap();
        std::fs::write(
            main.join(".bee").join("sessions").join("sess-stale.json"),
            jsjson::stringify(&json!({
                "id": "sess-stale",
                "last_heartbeat": stale.to_rfc3339(),
                "workspace_id": created.id,
            })),
        )
        .unwrap();

        let out = perform_cleanup(&main, &wt, &created.branch, &created.id, false);
        assert_ne!(
            out.get("code"),
            Some(&json!("WORKTREE_MERGE_CLEANUP_LIVE_SESSION"))
        );
    }

    /// ser-2: the same session record, but its status is `"closed"`
    /// (SessionEnd's clean exit) and its heartbeat is still FRESH — the
    /// closed mark itself releases the hold, independent of heartbeat
    /// timing; cleanup must proceed exactly as it does for a stale peer.
    #[test]
    fn cleanup_ignores_a_session_marked_closed() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "closed", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();

        let now = chrono::Utc::now();
        std::fs::create_dir_all(main.join(".bee").join("sessions")).unwrap();
        std::fs::write(
            main.join(".bee").join("sessions").join("sess-closed.json"),
            jsjson::stringify(&json!({
                "id": "sess-closed",
                "last_heartbeat": now.to_rfc3339(),
                "workspace_id": created.id,
                "status": "closed",
                "closed_at": now.to_rfc3339(),
            })),
        )
        .unwrap();

        let out = perform_cleanup(&main, &wt, &created.branch, &created.id, false);
        assert_eq!(out["ok"], Value::Bool(true), "{out:?}");
        assert_eq!(out["removed"], Value::Bool(true));
        assert!(!wt.exists(), "a closed-session hold must not block cleanup");
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

    /// wkm-1 (D1): `worktree_cleanup_on_merge`'s config half — unlike
    /// `archive_on_close_enabled` (close.rs:827), absent means OFF now (the
    /// worktree is kept by default), and a present-but-non-boolean value is
    /// still REFUSED (`None`), never silently read as either outcome.
    #[test]
    fn worktree_cleanup_on_merge_config_defaults_off_and_validates_the_present_shape() {
        let tmp = tempfile::tempdir().unwrap();
        // No .bee/config.json at all -> off (wkm-1's new default: keep).
        assert_eq!(worktree_cleanup_on_merge_config(tmp.path()), Some(false));

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

    /// wkm-1 (D1): with no `--cleanup`, no `--no-cleanup`, and no config,
    /// the merge KEEPS by default; an explicit `worktree_cleanup_on_merge:
    /// true` opts a merge into teardown even without the flag.
    #[test]
    fn resolve_cleanup_on_merge_defaults_off_and_config_true_opts_in() {
        let tmp = tempfile::tempdir().unwrap();
        // No flags at all, no config -> keep (D1's new default).
        assert_eq!(resolve_cleanup_on_merge(tmp.path(), false, false), Some(false));

        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(
            tmp.path().join(".bee").join("config.json"),
            "{\"worktree_cleanup_on_merge\": true}",
        )
        .unwrap();
        // The --cleanup flag was never passed — config alone opts this
        // merge into teardown.
        assert_eq!(resolve_cleanup_on_merge(tmp.path(), false, false), Some(true));
    }

    /// wkm-1 (D1): `--no-cleanup` is an explicit keep and wins over BOTH a
    /// config `true` opt-in AND a `--cleanup` flag for the same merge.
    #[test]
    fn no_cleanup_flag_wins_over_a_cleanup_flag_and_a_config_true_opt_in() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(
            tmp.path().join(".bee").join("config.json"),
            "{\"worktree_cleanup_on_merge\": true}",
        )
        .unwrap();
        assert_eq!(resolve_cleanup_on_merge(tmp.path(), false, true), Some(false));
        assert_eq!(resolve_cleanup_on_merge(tmp.path(), true, true), Some(false));
    }

    /// wkm-1 (D1), end to end: a merge with no `--cleanup`, no `--no-cleanup`,
    /// and no config resolves cleanup OFF, and running the merge with that
    /// DEFAULT-computed value KEEPS the worktree directory, its branch, and
    /// its grant/workspace registration — the same call shape `run_merge`
    /// makes when handed no flags at all — while queuing exactly one
    /// `worktree-cleanup` deferred-queue entry as the cross-check record.
    #[test]
    fn merge_with_no_flags_keeps_the_worktree_by_default_and_queues_one_entry() {
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

        // No --cleanup flag, no --no-cleanup flag, no config -> the same
        // value run_merge would compute for a plain "bee worktree merge --id
        // <id>" invocation.
        let cleanup = resolve_cleanup_on_merge(&main, false, false).unwrap();
        assert!(!cleanup, "no flags, no config -> the worktree is kept by default (D1)");

        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
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
        assert!(wt.exists(), "the worktree directory stands");
        let branches = std::process::Command::new("git")
            .args(["branch", "--list", &created.branch])
            .current_dir(&main)
            .output()
            .unwrap();
        assert!(
            !String::from_utf8_lossy(&branches.stdout).trim().is_empty(),
            "the branch stands"
        );
        // Registration (grants + workspace record) stays — prune must still
        // find the worktree later.
        let grants = read_grants_strict(&main.join(".bee")).expect("grants must still parse");
        assert!(grants.contains_key(&created.id), "the grant is still registered");

        let queued = crate::verbs::deferred_queue::items_for(&main, "worktree-cleanup", "demo");
        assert_eq!(queued.len(), 1, "exactly one worktree-cleanup entry");
        assert!(!queued[0].completed);
        // `QueuedItem` strips `files`/`reason` (nobody else needs them) —
        // read the raw event to check the entry actually names the
        // worktree root, id, and branch the reader needs to cross-check it.
        let raw = std::fs::read_to_string(main.join(".bee").join("deferred-queue.jsonl")).unwrap();
        let add_line = raw
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .find(|v| v["kind"] == json!("worktree-cleanup"))
            .expect("one worktree-cleanup add event");
        // Windows spells the same temp dir two ways (8.3 short vs long
        // form) and git resolves one while tempfile hands out the other —
        // compare path identity, not string spelling.
        let queued_files = add_line["files"].as_array().expect("files is an array");
        assert_eq!(queued_files.len(), 1, "{add_line}");
        assert!(
            crate::path_identity::canonical_paths_equal(
                Path::new(queued_files[0].as_str().unwrap()),
                &wt
            ),
            "{add_line}"
        );
        assert!(add_line["reason"].as_str().unwrap().contains(&created.id), "{add_line}");
        assert!(add_line["reason"].as_str().unwrap().contains(&created.branch), "{add_line}");
        assert!(
            add_line["reason"].as_str().unwrap().contains("bee worktree prune"),
            "{add_line}"
        );
    }

    /// wkm-1 (D1): `--cleanup` re-arms teardown — a merge with the flag on
    /// (no config, no `--no-cleanup`) actually removes the worktree
    /// directory, deletes its branch, and queues NOTHING (the entry only
    /// exists for the keep path).
    #[test]
    fn cleanup_flag_tears_down_and_queues_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would suppress cleanup while uat is pending — spell
        // "merge" explicitly since this test is about --cleanup itself.
        std::fs::write(main.join(".bee").join("config.json"), r#"{"uat_stop": "merge"}"#).unwrap();
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "demo", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();
        std::fs::write(wt.join("f.txt"), "y").unwrap();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        git_ok(&wt, &["commit", "-qam", "work"]);

        let cleanup = resolve_cleanup_on_merge(&main, true, false).unwrap();
        assert!(cleanup, "--cleanup opts this merge into teardown (D1)");

        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
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
        let queued = crate::verbs::deferred_queue::items_for(&main, "worktree-cleanup", "demo");
        assert!(queued.is_empty(), "the teardown path never queues a cross-check entry");
    }

    /// Truth 2: `--no-cleanup` resolves to cleanup=false, and a real merge
    /// with that value leaves the worktree directory and its branch
    /// standing — and, being a keep path, still queues its cross-check
    /// entry exactly like the default does.
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

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        assert!(!cleanup, "--no-cleanup opts this merge out (D1a)");

        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
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
        let queued = crate::verbs::deferred_queue::items_for(&main, "worktree-cleanup", "demo");
        assert_eq!(queued.len(), 1, "--no-cleanup is still a keep path: one entry");
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
    /// called with an explicit `--cleanup`-computed `cleanup = true`,
    /// because merging nothing is not a real merge. A second worktree
    /// merged with zero new commits on its branch is the smallest fixture
    /// that reaches this arm. It never queues a `worktree-cleanup` entry
    /// either — that entry only exists for a merge that actually merged.
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

        let cleanup = resolve_cleanup_on_merge(&main, true, false).unwrap();
        assert!(cleanup, "--cleanup would run on a REAL merge (D1)");

        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
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
        let queued = crate::verbs::deferred_queue::items_for(&main, "worktree-cleanup", "demo");
        assert!(queued.is_empty(), "nothing merged -> no cross-check entry either");
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

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    // ── trun-4: the pre-merge `.bee` (+ `docs/history/<feature>`) bookkeeping
    //    auto-commit that closes the deadlock between WORKTREE_MERGE_MAIN_DIRTY
    //    and the worktree-first guard ──────────────────────────────────────

    /// A MAIN checkout with one commit and a TRACKED `.bee/config.json` —
    /// unlike `main_repo` (whose `.gitignore` hides the whole store except
    /// the companion marker), these tests need `.bee` bookkeeping that shows
    /// up as ordinary tracked dirt, the same way a real store's
    /// `.bee/decisions.jsonl` / `.bee/cells/*.json` do.
    fn main_repo_tracking_bee(tmp: &Path) -> PathBuf {
        let main = tmp.join("main");
        std::fs::create_dir_all(main.join(".bee")).unwrap();
        std::fs::write(main.join(".bee").join("config.json"), "{}\n").unwrap();
        // Everything else `bootstrap_worktree_store` writes into a FRESH
        // worktree (`.bee/state.json`, `.bee/runtime/worktree-identity.json`)
        // is untracked bee-store scaffolding, not this cell's concern — the
        // worktree-dirty check already treats a bootstrapped, gitignored
        // `.bee` store as clean (decision D8a). Only `.bee/config.json`
        // (tracked, same as `main_repo`'s companion marker exception) is
        // exempt, so a later edit to it shows up as ordinary tracked dirt.
        std::fs::write(
            main.join(".gitignore"),
            ".bee/*\n!.bee/config.json\n",
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

    fn git_status_porcelain_str(dir: &Path) -> String {
        git_status_porcelain(dir).unwrap()
    }

    /// `git status --porcelain --untracked-files=all` as one string — the
    /// spelling that NAMES an untracked file inside an otherwise-untracked
    /// directory instead of collapsing it to `?? docs/`.
    fn git_status_untracked_all_str(dir: &Path) -> String {
        git_stdout(dir, &["status", "--porcelain", "--untracked-files=all"])
    }

    /// `git ls-files -- <path>`, trimmed: empty means the path is UNTRACKED
    /// — nothing added it to the index or to a commit.
    fn git_ls_files_str(dir: &Path, rel: &str) -> String {
        git_stdout(dir, &["ls-files", "--", rel]).trim().to_string()
    }

    /// Seed a file in main and COMMIT it, so a later `std::fs::write` to the
    /// same path leaves TRACKED-and-modified dirt instead of a brand-new
    /// untracked file. (mdp-1) That is what a peer's dirt really looks like —
    /// a capture sync dirties an existing tracked doc, it does not conjure an
    /// untracked one — and tracked dirt outside the swept roots is what this
    /// door still refuses on, unconditionally. Call it BEFORE the worktree is
    /// created: main is then clean at creation, and the branch never carries
    /// a change to this path, so the refusal cannot come from a collision.
    fn seed_tracked_file(main: &Path, rel: &str) {
        let path = main.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "seeded, already tracked\n").unwrap();
        git_ok(main, &["add", "-A"]);
        git_ok(main, &["commit", "-qm", "seed a tracked fixture file"]);
    }

    /// A read-only `git` call whose stdout an assertion needs verbatim.
    fn git_stdout(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap_or_else(|e| panic!("git {args:?} could not run in {dir:?}: {e}"));
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    /// A real worktree with one new commit on its branch — the smallest
    /// fixture that reaches the STAGED path (not ALREADY_UP_TO_DATE) in
    /// every test below.
    fn worktree_with_a_real_commit(main: &Path, feature: &str) -> crate::verbs::worktree::Created {
        let mut lock_busy = None;
        let created =
            create_feature_worktree(main, feature, None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        std::fs::write(created.worktree_root.join("f.txt"), "y").unwrap();
        git_ok(&created.worktree_root, &["config", "user.email", "a@b.c"]);
        git_ok(&created.worktree_root, &["config", "user.name", "t"]);
        git_ok(&created.worktree_root, &["commit", "-qam", "work"]);
        created
    }

    /// Truth 1: `.bee`-only dirt in main — no config override, no manual
    /// commit — merges clean. Reproduces the deadlock this cell closes: an
    /// orchestrator's normal state calls (`bee decisions log`, cell traces)
    /// leave `.bee/config.json` tracked-modified in main, and the merge must
    /// no longer refuse on that alone.
    #[test]
    fn bee_only_dirt_in_main_auto_commits_and_merge_succeeds() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");

        // The orchestrator's own bookkeeping, dirtying a TRACKED .bee file.
        std::fs::write(main.join(".bee").join("config.json"), "{\"a\": 1}\n").unwrap();
        assert!(is_tree_dirty(&main).unwrap(), "main must start dirty for this test to mean anything");

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge refused instead of auto-committing .bee dirt: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert_eq!(
            answer.result["bookkeeping_commit"]["committed"],
            Value::Bool(true),
            "{}",
            answer.result["bookkeeping_commit"]
        );
        assert!(git_status_porcelain_str(&main).is_empty(), "{}", git_status_porcelain_str(&main));
    }

    /// Truth 3 (path scoping), tested at the same grain close.rs tests its
    /// own bookkeeping commit: an unrelated STAGED file must never be swept
    /// into the `.bee`-scoped auto-commit, even though `commit_main_bookkeeping`
    /// runs `git add -A` — because that `-A` is itself pathspec-scoped.
    #[test]
    fn unrelated_staged_file_stays_staged_out_of_the_bookkeeping_commit() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        std::fs::write(main.join(".bee").join("config.json"), "{\"a\": 1}\n").unwrap();
        std::fs::write(main.join("staged.txt"), "staged dirt\n").unwrap();
        git_ok(&main, &["add", "staged.txt"]);

        let commit = commit_main_bookkeeping(
            &main,
            "Auto-commit .bee bookkeeping before merging worktree demo",
            &main_bookkeeping_roots(&main, None),
        );
        assert!(matches!(commit, MainBookkeepingCommit::Committed { .. }), "{}", commit.value());

        let status = git_status_porcelain_str(&main);
        assert!(status.contains("A  staged.txt"), "{status}");
        assert!(!status.contains("config.json"), "{status}");
    }

    /// Truth 2: a dirty path OUTSIDE `.bee/` (and outside this feature's own
    /// `docs/history/<feature>/`) still refuses, and the message names it —
    /// the auto-commit never widens past its two allowed roots. (mdp-1) The
    /// fixture is TRACKED-and-modified: tracked dirt refuses whether or not
    /// the merging branch touches it, and the untracked narrowing never
    /// reaches it.
    #[test]
    fn dirt_outside_bee_still_refuses_and_names_the_offending_path() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        seed_tracked_file(&main, "unrelated.txt");
        let created = worktree_with_a_real_commit(&main, "demo");

        std::fs::write(main.join(".bee").join("config.json"), "{\"a\": 1}\n").unwrap();
        std::fs::write(main.join("unrelated.txt"), "surprise\n").unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, true, None);
        let Err(err) = result else { panic!("a dirty path outside .bee/ must still refuse") };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_MAIN_DIRTY"), "{msg}");
        assert!(msg.contains("unrelated.txt"), "{msg}");
        // Nothing committed, nothing merged: main stays exactly as dirtied.
        assert!(git_status_porcelain_str(&main).contains("unrelated.txt"));
        assert!(git_status_porcelain_str(&main).contains("config.json"));
    }

    /// Truth 4 (this cell, D1): the refusal's scope string names all four
    /// roots the auto-commit actually sweeps — `.bee/`, `docs/decisions/`,
    /// `docs/knowledge/`, and `docs/history/<feature>/` — not the stale
    /// two-root list. A refusal that undercounts its own swept scope would
    /// mislead the reader about what still needs a manual commit.
    #[test]
    fn refusal_scope_string_names_the_widened_roots() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        seed_tracked_file(&main, "unrelated.txt");
        let created = worktree_with_a_real_commit(&main, "demo");

        std::fs::write(main.join(".bee").join("config.json"), "{\"a\": 1}\n").unwrap();
        std::fs::write(main.join("unrelated.txt"), "surprise\n").unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, true, None);
        let Err(err) = result else { panic!("a dirty path outside the swept roots must still refuse") };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains(".bee/"), "{msg}");
        assert!(msg.contains("docs/decisions/"), "{msg}");
        assert!(msg.contains("docs/knowledge/"), "{msg}");
        assert!(msg.contains("docs/history/demo/"), "{msg}");
    }

    /// Truth 1 + the docs/history root: this feature's OWN
    /// `docs/history/<feature>/` artifacts (a promote proposal `bee close`
    /// wrote, for instance) are swept into the same auto-commit as `.bee/`.
    #[test]
    fn this_features_docs_history_dirt_is_auto_committed_alongside_bee() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");

        std::fs::write(main.join(".bee").join("config.json"), "{\"a\": 1}\n").unwrap();
        std::fs::create_dir_all(main.join("docs").join("history").join("demo")).unwrap();
        std::fs::write(
            main.join("docs").join("history").join("demo").join("promote-proposals.md"),
            "proposal\n",
        )
        .unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge refused instead of auto-committing docs/history/demo: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert!(git_status_porcelain_str(&main).is_empty(), "{}", git_status_porcelain_str(&main));
    }

    /// D1 (dirty-main-conflicts): `docs/decisions/` is bee's own tracked
    /// output — `bee decisions log` writes `docs/decisions/taxonomy.json`
    /// on every run — so a dirty, already-tracked file under it joins
    /// `.bee` in the auto-commit's swept roots exactly like `docs/history`
    /// does, instead of refusing on bee's own bookkeeping.
    #[test]
    fn docs_decisions_dirt_is_auto_committed_alongside_bee() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");

        std::fs::create_dir_all(main.join("docs").join("decisions")).unwrap();
        std::fs::write(main.join("docs").join("decisions").join("taxonomy.json"), "{}\n").unwrap();
        git_ok(&main, &["add", "-A"]);
        git_ok(&main, &["commit", "-qm", "seed taxonomy.json"]);
        // Dirty the already-tracked file, the same shape `decisions log`
        // leaves behind.
        std::fs::write(main.join("docs").join("decisions").join("taxonomy.json"), "{\"a\": 1}\n").unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge refused instead of auto-committing docs/decisions: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert_eq!(
            answer.result["bookkeeping_commit"]["committed"],
            Value::Bool(true),
            "{}",
            answer.result["bookkeeping_commit"]
        );
        assert!(git_status_porcelain_str(&main).is_empty(), "{}", git_status_porcelain_str(&main));
    }

    /// kss-1: `docs/knowledge/` holds AUTHORED prose, not bookkeeping, so it
    /// is swept only when THIS feature's own capped cells recorded touching
    /// the exact path — the same read `feature_touched_files`
    /// (drivers/close.rs) already gives close's own doc-deferral scan.
    /// Recorded, dirty, already-tracked: it joins `.bee` in the auto-commit
    /// exactly like `docs/history/<feature>` does.
    #[test]
    fn docs_knowledge_dirt_recorded_by_this_features_own_cell_is_auto_committed_alongside_bee() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");

        std::fs::create_dir_all(main.join("docs").join("knowledge").join("areas")).unwrap();
        std::fs::write(
            main.join("docs").join("knowledge").join("areas").join("example.md"),
            "captured\n",
        )
        .unwrap();
        git_ok(&main, &["add", "-A"]);
        git_ok(&main, &["commit", "-qm", "seed knowledge doc"]);
        // Dirty the already-tracked file, the same shape a capture sync
        // leaves behind, and record it as touched by one of "demo"'s own
        // capped cells — the fact that earns it a scoped sweep.
        std::fs::write(
            main.join("docs").join("knowledge").join("areas").join("example.md"),
            "captured, updated\n",
        )
        .unwrap();
        write_capped_cell_with_files(&main, "demo-1", "demo", &["docs/knowledge/areas/example.md"]);

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge refused instead of auto-committing docs/knowledge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert_eq!(
            answer.result["bookkeeping_commit"]["committed"],
            Value::Bool(true),
            "{}",
            answer.result["bookkeeping_commit"]
        );
        assert!(git_status_porcelain_str(&main).is_empty(), "{}", git_status_porcelain_str(&main));
    }

    /// REAL INCIDENT, 2026-08-18: merging `uat-stop-placement` produced
    /// bookkeeping commit `7429dfda`, which swallowed a SIBLING session's
    /// capture sync to `docs/knowledge/areas/workflow-state/gates.md` —
    /// work belonging to feature `start-feature-reservation-scope`. This is
    /// the reproduction: another feature's OWN `docs/knowledge/` dirt must
    /// never be swept into THIS merge's auto-commit, modelled line-for-line
    /// on `another_features_docs_history_dirt_still_refuses_and_is_named`.
    /// Only the exact paths THIS feature's own capped cells recorded are
    /// ever in scope; anything else under `docs/knowledge/` still refuses
    /// and is named. (mdp-1) The peer's file is TRACKED-and-modified, which
    /// is what the 7429dfda incident actually was: a capture sync dirties an
    /// existing tracked doc. Tracked dirt refuses unconditionally.
    #[test]
    fn another_features_docs_knowledge_dirt_still_refuses_and_is_named() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        seed_tracked_file(&main, "docs/knowledge/areas/gates.md");
        let created = worktree_with_a_real_commit(&main, "demo");

        std::fs::write(main.join(".bee").join("config.json"), "{\"a\": 1}\n").unwrap();
        std::fs::write(
            main.join("docs").join("knowledge").join("areas").join("gates.md"),
            "a peer's in-flight capture sync\n",
        )
        .unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, true, None);
        let Err(err) = result else { panic!("another feature's docs/knowledge dirt must still refuse") };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_MAIN_DIRTY"), "{msg}");
        assert!(msg.contains("gates.md"), "{msg}");
        // Nothing committed, nothing merged: the peer's file is left exactly
        // as dirtied. (mdp-1) A TRACKED modification is never collapsed by
        // porcelain the way an untracked directory is, so plain porcelain
        // names it here.
        let after = git_status_porcelain_str(&main);
        assert!(
            after.contains("docs/knowledge/areas/gates.md"),
            "the peer's docs/knowledge file must be left untouched: {after}"
        );
    }

    /// The narrower case within a SINGLE feature: `docs/knowledge/` dirt
    /// recorded by no cell of THIS feature stays uncommitted even though the
    /// feature has other capped cells that recorded a DIFFERENT knowledge
    /// path — only the exact recorded paths are ever swept, never the whole
    /// root just because the feature touched some part of it. (mdp-1) The
    /// unrecorded file is TRACKED-and-modified, the dirt a capture sync
    /// really leaves; tracked dirt outside the swept roots refuses
    /// unconditionally.
    #[test]
    fn docs_knowledge_dirt_unrecorded_by_any_of_this_features_cells_stays_uncommitted() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        seed_tracked_file(&main, "docs/knowledge/areas/unrecorded.md");
        let created = worktree_with_a_real_commit(&main, "demo");

        write_capped_cell_with_files(&main, "demo-1", "demo", &["docs/knowledge/areas/recorded.md"]);

        std::fs::write(main.join(".bee").join("config.json"), "{\"a\": 1}\n").unwrap();
        std::fs::write(
            main.join("docs").join("knowledge").join("areas").join("unrecorded.md"),
            "dirt this feature never recorded touching\n",
        )
        .unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, true, None);
        let Err(err) = result else { panic!("unrecorded docs/knowledge dirt must still refuse") };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_MAIN_DIRTY"), "{msg}");
        assert!(msg.contains("unrecorded.md"), "{msg}");
    }

    /// kss-2, the last arm: a merge whose feature cannot be resolved
    /// (`resolve_worktree_feature` — absent for a worktree registered
    /// without bee's own creation identity) has no cell record to scope
    /// `docs/knowledge` by, so it sweeps NEITHER `docs/history` NOR
    /// `docs/knowledge` — only `.bee` and `docs/decisions`, exactly like the
    /// existing feature-less `docs/history` behavior this cell mirrors.
    /// Modelled on `bee_only_dirt_in_main_auto_commits_and_merge_succeeds`
    /// and `docs_decisions_dirt_is_auto_committed_alongside_bee`, with
    /// `make_feature_unresolvable` (the usp-7 fixture) genuinely stripping
    /// the feature instead of just dirtying an already-scoped tree.
    #[test]
    fn feature_less_merge_still_sweeps_bee_and_docs_decisions() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        make_feature_unresolvable(&created.worktree_root);
        assert_eq!(
            resolve_worktree_feature(&created.worktree_root).feature,
            None,
            "fixture must genuinely leave the feature unresolvable"
        );

        std::fs::create_dir_all(main.join("docs").join("decisions")).unwrap();
        std::fs::write(main.join("docs").join("decisions").join("taxonomy.json"), "{}\n").unwrap();
        git_ok(&main, &["add", "-A"]);
        git_ok(&main, &["commit", "-qm", "seed taxonomy.json"]);
        std::fs::write(main.join(".bee").join("config.json"), "{\"a\": 1}\n").unwrap();
        std::fs::write(main.join("docs").join("decisions").join("taxonomy.json"), "{\"a\": 1}\n").unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => {
                    panic!("a feature-less merge must still auto-commit .bee and docs/decisions dirt: {m}")
                }
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert_eq!(
            answer.result["bookkeeping_commit"]["committed"],
            Value::Bool(true),
            "{}",
            answer.result["bookkeeping_commit"]
        );
        assert!(git_status_porcelain_str(&main).is_empty(), "{}", git_status_porcelain_str(&main));

        // Truth 4: the bookkeeping commit's own subject names what it
        // actually swept — never a stale claim on `docs/knowledge`, which
        // this arm never touches. (`git log -1` would read the MERGE
        // commit that lands on top of it, not the bookkeeping commit
        // itself, so this reads the bookkeeping sha directly.)
        let sha = answer.result["bookkeeping_commit"]["sha"].as_str().unwrap();
        let subject = std::process::Command::new("git")
            .args(["show", "-s", "--format=%s", sha])
            .current_dir(&main)
            .output()
            .unwrap();
        let subject = String::from_utf8_lossy(&subject.stdout);
        assert!(subject.contains(".bee"), "{subject}");
        assert!(subject.contains("docs/decisions"), "{subject}");
        assert!(!subject.contains("docs/knowledge"), "{subject}");
    }

    /// The other half: a feature-less merge with dirty `docs/knowledge`
    /// (no cell record exists to scope it by — there is no resolvable
    /// feature to have recorded one) leaves it uncommitted and the existing
    /// dirty-main refusal names it, the same mechanism that already names
    /// an unscoped `docs/history` file today. Modelled on
    /// `another_features_docs_knowledge_dirt_still_refuses_and_is_named`.
    /// (mdp-1) The file is TRACKED-and-modified, the dirt a capture sync
    /// really leaves; tracked dirt refuses unconditionally.
    #[test]
    fn feature_less_merge_docs_knowledge_dirt_stays_uncommitted_and_is_named() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        seed_tracked_file(&main, "docs/knowledge/areas/example.md");
        let created = worktree_with_a_real_commit(&main, "demo");
        make_feature_unresolvable(&created.worktree_root);
        assert_eq!(resolve_worktree_feature(&created.worktree_root).feature, None);

        std::fs::write(main.join(".bee").join("config.json"), "{\"a\": 1}\n").unwrap();
        std::fs::write(
            main.join("docs").join("knowledge").join("areas").join("example.md"),
            "captured, but nobody can name whose feature this is\n",
        )
        .unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, true, None);
        let Err(err) = result else {
            panic!("a feature-less merge must still refuse on dirty docs/knowledge")
        };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_MAIN_DIRTY"), "{msg}");
        assert!(msg.contains("example.md"), "{msg}");
        // Nothing committed, nothing merged: the file is left exactly as
        // dirtied, same as an unscoped docs/history file is today.
        let after = git_status_porcelain_str(&main);
        assert!(
            after.contains("docs/knowledge/areas/example.md"),
            "the unscoped docs/knowledge file must be left untouched: {after}"
        );
    }

    // ── mdp-1: an untracked path in main blocks the merge only where the
    //    merging branch actually writes it ─────────────────────────────────
    //
    // These two replace `another_features_docs_history_dirt_still_refuses_
    // and_is_named`, whose fixture was an untracked
    // `docs/history/<other>/plan.md`. That test argued the REFUSAL was a
    // safety property because sweeping a peer's `docs/history/<other>/` into
    // this merge's bookkeeping commit would land their work without their
    // say-so. The argument is sound, but it is about the AUTO-COMMIT, and
    // the two are separable: do-not-commit is kept (and now PROVEN three
    // ways in the first test below, instead of assumed from the refusal),
    // while do-not-refuse is dropped, because refusing on a path this merge
    // can never write bought nothing and cost every parallel session a
    // merge. `main_bookkeeping_roots` is untouched, so the peer's file is
    // exempted from the REFUSAL and from nothing else.

    /// mdp-1, the half this cell FREES. A peer session's untracked
    /// `docs/history/<other>/` file is not something this merge can touch:
    /// the merging branch never writes that path, so no collision is
    /// possible. Refusing on it made the steady state with several live
    /// sessions "nobody merges until everybody commits" — a coordination
    /// cost paid daily for a collision that cannot happen. The merge must
    /// PROCEED, and the peer's file must still be there afterwards,
    /// untracked and uncommitted: "stop refusing" must never quietly become
    /// "sweep it into my commit".
    #[test]
    fn an_untracked_path_the_branch_never_touches_no_longer_blocks_the_merge() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");

        std::fs::write(main.join(".bee").join("config.json"), "{\"a\": 1}\n").unwrap();
        let peer = main.join("docs").join("history").join("other-feature").join("plan.md");
        std::fs::create_dir_all(peer.parent().unwrap()).unwrap();
        std::fs::write(&peer, "a peer's in-flight work\n").unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => {
                    panic!("an untracked path the branch never writes must not refuse the merge: {m}")
                }
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));

        // The do-not-commit half, proven rather than assumed — three ways.
        // Still on disk, byte-identical:
        assert!(peer.exists(), "the peer's file must survive the merge");
        assert_eq!(
            std::fs::read_to_string(&peer).unwrap(),
            "a peer's in-flight work\n",
            "the peer's file must be left byte-untouched"
        );
        // still UNTRACKED (nothing added it to the index or to a commit):
        assert!(
            git_ls_files_str(&main, "docs/history/other-feature/plan.md").is_empty(),
            "the peer's file must still be untracked after the merge"
        );
        // and still reported as uncommitted dirt by git itself:
        let after = git_status_untracked_all_str(&main);
        assert!(
            after.contains("docs/history/other-feature/plan.md"),
            "the peer's file must still be uncommitted: {after}"
        );
    }

    /// mdp-1, the half this cell KEEPS. The narrowing is a collision test,
    /// never a blanket pardon for untracked files: when the merging branch
    /// itself writes the path, the merge WOULD overwrite whatever is sitting
    /// there, so it still refuses and still names it. Without this arm the
    /// narrowing would be a way to clobber.
    #[test]
    fn an_untracked_path_the_branch_does_write_still_refuses_and_is_named() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");

        // The branch's own history adds exactly this path.
        let on_branch = created
            .worktree_root
            .join("docs")
            .join("history")
            .join("other-feature")
            .join("plan.md");
        std::fs::create_dir_all(on_branch.parent().unwrap()).unwrap();
        std::fs::write(&on_branch, "the branch's own version\n").unwrap();
        git_ok(&created.worktree_root, &["add", "-A"]);
        git_ok(&created.worktree_root, &["commit", "-qm", "the branch writes that path"]);

        std::fs::write(main.join(".bee").join("config.json"), "{\"a\": 1}\n").unwrap();
        let collides = main.join("docs").join("history").join("other-feature").join("plan.md");
        std::fs::create_dir_all(collides.parent().unwrap()).unwrap();
        std::fs::write(&collides, "uncommitted work the merge would clobber\n").unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, true, None);
        let Err(err) = result else {
            panic!("an untracked path the branch itself writes must still refuse")
        };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_MAIN_DIRTY"), "{msg}");
        assert!(msg.contains("docs/history/other-feature/plan.md"), "{msg}");
        // Nothing merged, nothing clobbered: the file is left exactly as it was.
        assert_eq!(
            std::fs::read_to_string(&collides).unwrap(),
            "uncommitted work the merge would clobber\n"
        );
    }

    /// mdp-1's sharp edge: the changed-file set is read against the branch's
    /// MERGE BASE with main, never against main's HEAD. This fixture is the
    /// one where the two spellings DISAGREE — main has moved on since the
    /// fork and DELETED `stale.txt`, which the branch still carries
    /// untouched. `git diff HEAD wt/demo` therefore names `stale.txt` (the
    /// merge RESULT differs from HEAD there), but the branch never wrote it,
    /// so an untracked `stale.txt` in main cannot be clobbered and must not
    /// refuse. Without a fixture where main moved on since the fork, both
    /// spellings agree and the test proves nothing.
    #[test]
    fn the_changed_file_set_is_read_against_the_merge_base_not_mains_head() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        std::fs::write(main.join("stale.txt"), "seeded before the fork\n").unwrap();
        git_ok(&main, &["add", "-A"]);
        git_ok(&main, &["commit", "-qm", "seed stale.txt"]);

        let created = worktree_with_a_real_commit(&main, "demo");

        // Main moves on AFTER the fork: it deletes the file and commits.
        git_ok(&main, &["rm", "-q", "stale.txt"]);
        git_ok(&main, &["commit", "-qm", "main deletes stale.txt"]);
        // …and an untracked file of the same name is left sitting in main.
        std::fs::write(main.join("stale.txt"), "a peer's fresh scratch file\n").unwrap();

        // The two spellings disagree — the whole reason this fixture exists.
        let base = git_stdout(&main, &["merge-base", "HEAD", "wt/demo"]);
        let base_spelling =
            git_stdout(&main, &["diff", "--no-renames", "--name-only", base.trim(), "wt/demo"]);
        assert!(
            !base_spelling.contains("stale.txt"),
            "the branch never wrote stale.txt: {base_spelling}"
        );
        let head_spelling =
            git_stdout(&main, &["diff", "--no-renames", "--name-only", "HEAD", "wt/demo"]);
        assert!(
            head_spelling.contains("stale.txt"),
            "the HEAD spelling must name stale.txt, or this fixture proves nothing: {head_spelling}"
        );

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("the merge-base spelling must let this merge through: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert_eq!(
            std::fs::read_to_string(main.join("stale.txt")).unwrap(),
            "a peer's fresh scratch file\n",
            "the untracked file must be left byte-untouched"
        );
    }

    /// Truth 4: warn-never-block. A `pre-commit` hook that fails SILENTLY
    /// (exit 1, nothing on either stream) drives the bookkeeping commit's
    /// own `git commit` into its failure branch — the merge must still
    /// complete green, same contract `bee close`'s own bookkeeping commit
    /// keeps. The hook fails ONCE (a marker file flips it green after) so it
    /// targets only the bookkeeping auto-commit, not the merge commit that
    /// follows it in the same repo.
    #[cfg(unix)]
    #[test]
    fn a_failing_bookkeeping_commit_only_warns_and_the_merge_still_completes_green() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");

        std::fs::write(main.join(".bee").join("config.json"), "{\"a\": 1}\n").unwrap();

        let hook_dir = main.join(".git").join("hooks");
        std::fs::create_dir_all(&hook_dir).unwrap();
        let hook = hook_dir.join("pre-commit");
        std::fs::write(
            &hook,
            "#!/bin/sh\nmarker=\"$(git rev-parse --git-dir)/fail-once-marker\"\nif [ ! -f \"$marker\" ]; then touch \"$marker\"; exit 1; fi\nexit 0\n",
        )
        .unwrap();
        let mut perms = std::fs::metadata(&hook).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook, perms).unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("a failing bookkeeping commit must warn, not refuse: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert_eq!(answer.result["bookkeeping_commit"]["committed"], Value::Bool(false));
        let reason = answer.result["bookkeeping_commit"]["reason"].as_str().unwrap_or_default();
        assert!(reason.starts_with("git_failed:"), "{reason}");
    }

    /// Truth 5: the opt-out. `worktree_merge_commit_bookkeeping: false` in
    /// `.bee/config.json` turns the auto-commit off entirely — the merge
    /// falls back to refusing on ANY dirty main exactly as it did before
    /// this cell, even for `.bee`-only dirt.
    #[test]
    fn config_opt_out_disables_the_auto_commit_and_the_merge_refuses_as_before() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo_tracking_bee(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");

        // Dirties `.bee/config.json` itself — writing the opt-out into the
        // very file that must stay uncommitted (same shape close.rs's own
        // `config_false_skips_the_commit_with_reason_config_off` test uses).
        std::fs::write(
            main.join(".bee").join("config.json"),
            "{\"worktree_merge_commit_bookkeeping\": false}\n",
        )
        .unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, true, None);
        let Err(err) = result else { panic!("the opt-out must fall back to refusing on any dirty main") };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_MAIN_DIRTY"), "{msg}");
        // The pre-cell wording, unchanged: opting out reverts to it exactly,
        // never a narrower "outside .bee/" variant.
        assert!(msg.contains("\"git status --porcelain\" is non-empty"), "{msg}");
        assert!(
            git_status_porcelain_str(&main).contains("config.json"),
            "opted-out dirt must stay uncommitted: {}",
            git_status_porcelain_str(&main)
        );
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

        let answer = merge_feature_worktree(&main, &created.id, false, Some("exit 0"), true, None)
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

    // ── registry.rs: wkm-3 — `worktree list` marks kept-merged worktrees ──

    /// The pure fold half of the marker: an `add` for `worktree-cleanup`
    /// leaves its file pending; a matching `complete` (wkm-2's prune
    /// resolution) clears it; a different `kind` is never counted even
    /// though it shares the queue file.
    #[test]
    fn pending_worktree_cleanup_roots_reads_pending_and_skips_completed() {
        let tmp = tempfile::tempdir().unwrap();
        let main_store_root = tmp.path().join(".bee");
        std::fs::create_dir_all(&main_store_root).unwrap();

        crate::verbs::deferred_queue::enqueue(
            tmp.path(),
            "worktree-cleanup",
            "demo-a",
            &[],
            &[],
            &[String::from("/wt/pending")],
            "kept per default",
        )
        .unwrap();
        let (resolved_id, _) = crate::verbs::deferred_queue::enqueue(
            tmp.path(),
            "worktree-cleanup",
            "demo-b",
            &[],
            &[],
            &[String::from("/wt/resolved")],
            "kept per default",
        )
        .unwrap();
        crate::verbs::deferred_queue::enqueue(
            tmp.path(),
            "capture",
            "demo-c",
            &[],
            &[],
            &[String::from("/wt/other-kind")],
            "not a worktree-cleanup entry",
        )
        .unwrap();
        // Fold in the resolution by hand — the same `complete` event shape
        // wkm-2's prune resolution appends (no hand-edit of the JSONL: this
        // goes through `fsutil::append_jsonl`, exactly what a real
        // `complete` writes).
        crate::fsutil::append_jsonl(
            &main_store_root.join("deferred-queue.jsonl"),
            &json!({
                "ts": now_iso(),
                "event": "complete",
                "id": resolved_id,
            }),
        )
        .unwrap();

        let roots = pending_worktree_cleanup_roots(&main_store_root);
        assert_eq!(roots, vec![PathBuf::from("/wt/pending")], "{roots:?}");
    }

    /// A missing queue file is "nothing pending", not a delegate — the same
    /// shape `read_grants_strict` gives a missing grants file.
    #[test]
    fn pending_worktree_cleanup_roots_is_empty_for_a_missing_queue_file() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(pending_worktree_cleanup_roots(&tmp.path().join(".bee")).is_empty());
    }

    /// The must_have, end to end over a real `git worktree add` fixture: an
    /// id whose worktree root is named by a pending `worktree-cleanup`
    /// entry is `true`; the same id is `false` again once that entry is
    /// marked `complete` — `resolve_worktree_by_id`'s real git-verified path
    /// resolution is exactly what `run_list` calls, so this proves the
    /// marker survives the id -> root lookup, not just a string compare.
    #[test]
    fn merged_pending_map_is_true_only_for_a_pending_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, created) = prune_fixture(tmp.path());
        let ids = vec![&created.id];

        // No entry at all: never pending.
        let before = merged_pending_map(&main, &ids);
        assert_eq!(before.get(&created.id), Some(&Value::Bool(false)), "{before:?}");

        let (id, _) = crate::verbs::deferred_queue::enqueue(
            &main,
            "worktree-cleanup",
            "demo",
            &[],
            &[],
            &[p(&created.worktree_root)],
            &format!(
                "Worktree {} (branch {}) merged into main at deadbeef and kept per default (D1) — remove it with `bee worktree prune`.",
                created.id, created.branch
            ),
        )
        .unwrap();

        let pending = merged_pending_map(&main, &ids);
        assert_eq!(pending.get(&created.id), Some(&Value::Bool(true)), "{pending:?}");

        // Resolved (wkm-2's `complete`): the marker clears again.
        crate::fsutil::append_jsonl(
            &main.join(".bee").join("deferred-queue.jsonl"),
            &json!({ "ts": now_iso(), "event": "complete", "id": id }),
        )
        .unwrap();
        let after = merged_pending_map(&main, &ids);
        assert_eq!(after.get(&created.id), Some(&Value::Bool(false)), "{after:?}");
    }

    // ── uat-gate-before-merge D1: the merge-time uat precondition ──────────

    /// A live workflow record for `feature`, naming its risk lane and
    /// whether the uat gate has been approved — the primary source both
    /// `uat_merge_precheck` reads consult first.
    fn write_live_workflow_uat(main: &Path, feature: &str, mode: &str, uat_approved: bool) {
        let id = format!("wf-{feature}-uat-test");
        let dir = main.join(".bee").join("runtime").join("workflows").join(&id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("state.json"),
            jsjson::stringify(&json!({
                "id": id,
                "feature": feature,
                "status": "active",
                "mode": mode,
                "gates": { "uat": { "approved": uat_approved } },
            })),
        )
        .unwrap();
    }

    /// A `.bee/lanes/<feature>.json` record naming only the risk lane — the
    /// fallback `uat_merge_precheck` reads when no LIVE workflow names the
    /// feature (e.g. a lane started but never turned into a running
    /// workflow, or a feature classified without `--as-lane`).
    fn write_lane_mode(main: &Path, feature: &str, mode: &str) {
        let dir = main.join(".bee").join("lanes");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{feature}.json")),
            jsjson::stringify(&json!({ "feature": feature, "mode": mode })),
        )
        .unwrap();
    }

    /// Truth 1: a standard-lane feature with an unapproved uat gate refuses,
    /// typed, and touches NOTHING — the worktree, its branch, and main's
    /// tree all stand exactly as before the call.
    #[test]
    fn merge_of_a_standard_lane_feature_without_uat_approval_refuses_zero_mutation() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", false);
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which never blocks at merge — spell "merge" explicitly to
        // exercise the merge-time door this test is about.
        std::fs::write(main.join(".bee").join("config.json"), r#"{"uat_stop": "merge"}"#).unwrap();

        let pre_merge_head =
            js_trim(&run_git(&main, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, false, None);
        let Err(err) = result else { panic!("an unapproved uat gate on a standard lane must refuse") };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_UAT_PENDING"), "{msg}");
        // The remedy names all three exits.
        assert!(msg.contains("--skip-uat"), "{msg}");
        assert!(msg.contains("bee gate --name uat --approved true"), "{msg}");
        assert!(msg.contains("uat_before_merge"), "{msg}");

        // Zero mutation: main's HEAD is unchanged, no MERGE_HEAD exists, the
        // worktree directory and branch both stand.
        let post_merge_head =
            js_trim(&run_git(&main, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
        assert_eq!(pre_merge_head, post_merge_head);
        assert!(!main.join(".git").join("MERGE_HEAD").exists());
        assert!(created.worktree_root.exists());
        let branches = std::process::Command::new("git")
            .args(["branch", "--list", &created.branch])
            .current_dir(&main)
            .output()
            .unwrap();
        assert!(!String::from_utf8_lossy(&branches.stdout).trim().is_empty());
    }

    // ── staging-lane D0 teeth #1 & #3 ───────────────────────────────────────

    /// Teeth #1: the staging worktree/branch can never land into main via
    /// `bee worktree merge`, matched by EITHER the git-internal worktree id
    /// (the surface's own `--id`) or the branch name itself — no escape
    /// flag, and the refusal touches nothing (main's HEAD, the staging
    /// worktree/branch, and its record all stand exactly as before).
    #[test]
    fn merge_refuses_the_staging_branch_by_id_and_by_branch_zero_mutation() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        // defaults-and-agent-env D2: staging is opt-in now (absent means
        // off) — this fixture is about the merge-side refusal, not the
        // staging default, so turn staging on explicitly.
        std::fs::write(main.join(".bee").join("config.json"), r#"{"staging_before_merge": true}"#).unwrap();
        worktree_with_a_real_commit(&main, "demo");
        let add = crate::verbs::staging::staging_add(&main, "demo")
            .unwrap_or_else(|e| panic!("staging add must succeed: {e}"));
        let staging_root = add.staging_worktree_root.clone();
        let staging_id = read_worktree_git_verified_id(&staging_root)
            .unwrap_or_else(|e| panic!("could not resolve the staging worktree's git id: {e}"));

        let pre_merge_head =
            js_trim(&run_git(&main, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
        let record_before = crate::verbs::staging::read_staging_record(&main).unwrap().unwrap();

        // Matched by the git-internal worktree id.
        let by_id = merge_feature_worktree(&main, &staging_id, false, None, true, None);
        match by_id {
            Err(MErr::Thrown(msg)) => {
                assert!(msg.contains("WORKTREE_MERGE_STAGING_FORBIDDEN"), "{msg}");
                assert!(msg.contains("no escape flag"), "{msg}");
                assert!(msg.contains("uat"), "{msg}");
            }
            Err(MErr::Ex) => panic!("expected a typed refusal, got MErr::Ex"),
            Ok(answer) => panic!("a staging merge by id must refuse: ok={} {:?}", answer.ok, answer.result),
        }

        // Matched by the branch name alone.
        let by_branch = merge_feature_worktree(&main, "staging", false, None, true, None);
        match by_branch {
            Err(MErr::Thrown(msg)) => assert!(msg.contains("WORKTREE_MERGE_STAGING_FORBIDDEN"), "{msg}"),
            Err(MErr::Ex) => panic!("expected a typed refusal, got MErr::Ex"),
            Ok(answer) => panic!("a staging merge by branch must refuse: ok={} {:?}", answer.ok, answer.result),
        }

        // Zero mutation across both attempts.
        let post_merge_head =
            js_trim(&run_git(&main, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
        assert_eq!(pre_merge_head, post_merge_head, "main HEAD must not move");
        assert!(!main.join(".git").join("MERGE_HEAD").exists());
        assert!(staging_root.exists(), "the staging worktree must still stand");
        let branches = std::process::Command::new("git")
            .args(["branch", "--list", "staging"])
            .current_dir(&main)
            .output()
            .unwrap();
        assert!(
            !String::from_utf8_lossy(&branches.stdout).trim().is_empty(),
            "the staging branch must still stand"
        );
        let record_after = crate::verbs::staging::read_staging_record(&main).unwrap().unwrap();
        assert_eq!(record_after.base_sha, record_before.base_sha);
        assert_eq!(record_after.staged.len(), record_before.staged.len());
    }

    /// Teeth #3 (D0a trigger 3): a real merge to main carries the rebuild
    /// nudge in its result JSON only when a staging record exists — its
    /// mere presence is what triggers the nudge, not this merge's own
    /// feature (the staged feature here is a DIFFERENT one than the one
    /// being merged).
    #[test]
    fn merge_result_carries_the_staging_rebuild_nudge_only_when_a_staging_record_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());

        // No staging record yet — the nudge must be absent.
        let created_first = worktree_with_a_real_commit(&main, "demo");
        let answer_first = merge_feature_worktree(&main, &created_first.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer_first.ok, "{:?}", answer_first.result);
        assert!(
            !answer_first.result.contains_key("staging_rebuild_suggested"),
            "no staging record yet -> no nudge: {:?}",
            answer_first.result
        );

        // Stage a (third, still-unmerged) feature so a staging record now
        // exists, then merge a SECOND feature to main. `-qam` only stages
        // MODIFICATIONS to already-tracked files, never a brand-new
        // untracked path — so every worktree below keeps writing the SAME
        // already-tracked `f.txt`, just with content unique enough that
        // `-a` always has something real to pick up (main's HEAD already
        // carries `f.txt` = "y" from the "demo" merge above).
        let mut lock_busy = None;
        let staged_third =
            create_feature_worktree(&main, "already-staged", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        std::fs::write(staged_third.worktree_root.join("f.txt"), "z1").unwrap();
        git_ok(&staged_third.worktree_root, &["config", "user.email", "a@b.c"]);
        git_ok(&staged_third.worktree_root, &["config", "user.name", "t"]);
        git_ok(&staged_third.worktree_root, &["commit", "-qam", "staged work"]);
        // defaults-and-agent-env D2: staging is opt-in now — turn it on
        // explicitly right before the first staging_add, so the earlier
        // "no staging record yet" assertion still exercises a genuinely
        // staging-off repo.
        std::fs::write(main.join(".bee").join("config.json"), r#"{"staging_before_merge": true}"#).unwrap();
        crate::verbs::staging::staging_add(&main, "already-staged")
            .unwrap_or_else(|e| panic!("staging add must succeed: {e}"));

        let created_second =
            create_feature_worktree(&main, "second", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        std::fs::write(created_second.worktree_root.join("f.txt"), "z2").unwrap();
        git_ok(&created_second.worktree_root, &["config", "user.email", "a@b.c"]);
        git_ok(&created_second.worktree_root, &["config", "user.name", "t"]);
        git_ok(&created_second.worktree_root, &["commit", "-qam", "second work"]);
        let answer_second = merge_feature_worktree(&main, &created_second.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer_second.ok, "{:?}", answer_second.result);
        assert_eq!(
            answer_second.result["staging_rebuild_suggested"],
            json!("bee staging rebuild")
        );
    }

    // ── mcl-2 (R1): the merge-time lane close ───────────────────────────────
    // A green, real merge answers the merged feature's stranded uat mark —
    // clears its lane `waiting_on`/`run_state` pair and rewrites
    // `next_action` to name the close road, never touching `phase`.

    /// A lane record with a live `waiting_on` gate mark and a stuck
    /// `run_state: awaiting-approval`, exactly the shape `bee state gate`
    /// leaves behind — the same raw-JSON shape
    /// `clear_lane_waiting_on_pair_nulls_the_stuck_pair` (state_group/tests.rs)
    /// already pins for the shared helper this cell calls.
    fn write_stranded_lane(main: &Path, feature: &str, phase: &str) {
        let dir = main.join(".bee").join("lanes");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{feature}.json")),
            format!(
                r#"{{"feature":"{feature}","phase":"{phase}","waiting_on":{{"kind":"gate","subject":"uat","asked_at":"2026-01-01T00:00:00.000Z","session":"sess-1"}},"run_state":"awaiting-approval"}}"#
            ),
        )
        .unwrap();
    }

    /// Happy path: a green real merge of a feature whose lane holds a live
    /// uat mark leaves that lane with `waiting_on` null, `run_state` no
    /// longer "awaiting-approval", `phase` byte-identical, and a
    /// `next_action` naming `bee close --feature <f>` — the same line the
    /// merge's own text output carries.
    #[test]
    fn a_green_merge_clears_the_merged_features_stranded_lane_mark_and_names_close() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which SETS the uat wait on merge rather than clearing it — spell
        // "merge" explicitly, since this test is about the clearing
        // behavior itself, not the placement default.
        std::fs::write(main.join(".bee").join("config.json"), r#"{"uat_stop": "merge"}"#).unwrap();
        let created = worktree_with_a_real_commit(&main, "demo");
        write_stranded_lane(&main, "demo", "scribing");

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));

        let lane = crate::verbs::workflow_store::read_lane_strict(&main, "demo")
            .unwrap()
            .expect("the lane record must still exist after a green merge");
        assert_eq!(lane.get("waiting_on"), Some(&Value::Null), "{lane:?}");
        assert_eq!(lane.get("run_state"), Some(&Value::Null), "{lane:?}");
        assert_eq!(
            lane.get("phase"),
            Some(&json!("scribing")),
            "merge must never write phase — a slice-1-of-3 merge would lie about a finished feature: {lane:?}"
        );
        let next_action = lane.get("next_action").and_then(Value::as_str).unwrap_or_default();
        assert!(next_action.contains("bee close --feature demo"), "{next_action}");
        assert!(next_action.starts_with("Merged into main at"), "{next_action}");

        assert_eq!(answer.result["next_action"], json!(next_action));
        let lines = merge_text_lines(&created.id, &main, &answer);
        assert!(
            lines.iter().any(|l| l.contains("bee close --feature demo")),
            "merge text must name the same close road: {lines:?}"
        );
    }

    /// Edge: an already-up-to-date merge writes no lane change at all — the
    /// `ALREADY_UP_TO_DATE` arm returns before `merge_finish`'s commit
    /// sequence (and this cell's lane write inside it) is ever reached.
    #[test]
    fn already_up_to_date_merge_leaves_the_lane_record_byte_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "demo", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        write_stranded_lane(&main, "demo", "scribing");
        let lane_path = main.join(".bee").join("lanes").join("demo.json");
        let before = std::fs::read(&lane_path).unwrap();

        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["code"], json!("ALREADY_UP_TO_DATE"));

        let after = std::fs::read(&lane_path).unwrap();
        assert_eq!(before, after, "an up-to-date merge must not touch the lane record at all");
    }

    /// Error: a lane write that cannot complete (here, a lane record too
    /// corrupt for `read_lane_strict` to parse) warns and the merge still
    /// returns green — same warn-never-block contract the bookkeeping
    /// auto-commit above keeps.
    #[test]
    fn a_failing_lane_write_only_warns_and_the_merge_still_completes_green() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        let dir = main.join(".bee").join("lanes");
        std::fs::create_dir_all(&dir).unwrap();
        let lane_path = dir.join("demo.json");
        std::fs::write(&lane_path, "not json at all").unwrap();
        let before = std::fs::read(&lane_path).unwrap();

        let answer = merge_feature_worktree(&main, &created.id, false, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("a corrupt lane must warn, not refuse the merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        let warning = answer.result["lane_close_warning"].as_str().unwrap_or_default();
        assert!(warning.contains("corrupt"), "{warning}");
        assert!(!answer.result.contains_key("next_action"), "{:?}", answer.result);

        let after = std::fs::read(&lane_path).unwrap();
        assert_eq!(before, after, "a lane write that could not even be read must leave the file untouched");
    }

    /// mcl-2 (R2): the regression the semantic judge caught. `main_repo`'s
    /// `.gitignore` (`.bee/*`) means every fixture above leaves the lane
    /// file UNTRACKED, so the post-commit dirty guard (which reads
    /// `git status --porcelain --untracked-files=no`) never sees it move —
    /// masking a real bug. Here the fixture force-adds and commits the lane
    /// file BEFORE the merge, so it is genuinely tracked; the merge's own
    /// rewrite of `next_action` is exactly the kind of tracked-file mutation
    /// the guard watches for. A correctly-ordered merge (guard reads the
    /// tree, THEN writes the lane) must still come back green with no
    /// `verify_mutated_tracked_files` warning — the guard is watching the
    /// merge commit's aftermath, not the lane write this function is about
    /// to make on its own behalf.
    #[test]
    fn a_green_merge_of_a_tracked_lane_file_emits_no_mutated_tracked_files_warning() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        // defaults-and-agent-env D1: spell "merge" explicitly — this test
        // is about the tracked-file mutation guard, not the uat-stop
        // placement default, and the absent-key default (Close) would set
        // the uat wait instead of clearing it.
        std::fs::write(main.join(".bee").join("config.json"), r#"{"uat_stop": "merge"}"#).unwrap();
        let created = worktree_with_a_real_commit(&main, "demo");
        write_stranded_lane(&main, "demo", "scribing");
        git_ok(&main, &["add", "-f", ".bee/lanes/demo.json"]);
        git_ok(&main, &["commit", "-qm", "track the demo lane"]);

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert!(
            !answer.result.contains_key("warning"),
            "a tracked lane rewrite must not trip the post-commit dirty guard: {:?}",
            answer.result
        );

        let lane = crate::verbs::workflow_store::read_lane_strict(&main, "demo")
            .unwrap()
            .expect("the lane record must still exist after a green merge");
        assert_eq!(lane.get("waiting_on"), Some(&Value::Null), "{lane:?}");
        let next_action = lane.get("next_action").and_then(Value::as_str).unwrap_or_default();
        assert!(next_action.contains("bee close --feature demo"), "{next_action}");
    }

    /// mct-1: the P3 residual an independent judge raised against
    /// merge-closes-the-lane. `close_the_lane_on_merge` rewrites a TRACKED
    /// file (`.bee/lanes/<feature>.json`, force-tracked here — the SAME
    /// fixture shape the sibling test above uses, since an untracked lane
    /// file is invisible to both the guard and this behavior, the exact
    /// blindness that hid the original defect) — a green merge must commit
    /// that rewrite in its own path-scoped commit, not leave it as dirt for
    /// main. Pins: (1) main's tree is clean after the merge, (2) the lane
    /// commit lands as a NEW, separate commit on top of the merge commit
    /// (the merge commit is never amended — still a real merge with two
    /// parents), and (3) that new commit's diff touches ONLY the lane path.
    #[test]
    fn a_green_merge_of_a_tracked_lane_file_commits_the_rewrite_and_leaves_main_clean() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_stranded_lane(&main, "demo", "scribing");
        // `main_repo`'s blanket `.bee/*` ignore (unlike the real repo's own
        // `.gitignore`, which never blanket-ignores `.bee/lanes/`) would
        // make git's own `add` refuse a re-add of this already-tracked path
        // on its later modification without an explicit `-f` every time —
        // negate it here so the fixture matches production's actual shape
        // instead of masking the lane-commit step this test exists to pin.
        std::fs::write(main.join(".gitignore"), ".bee/*\n!.bee/companion-session.json\n!.bee/lanes/\n").unwrap();
        git_ok(&main, &["add", "-A", "--", ".gitignore"]);
        git_ok(&main, &["commit", "-qm", "stop ignoring lanes"]);
        git_ok(&main, &["add", "-f", ".bee/lanes/demo.json"]);
        git_ok(&main, &["commit", "-qm", "track the demo lane"]);

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert_eq!(
            answer.result["lane_bookkeeping_commit"]["committed"],
            Value::Bool(true),
            "{}",
            answer.result["lane_bookkeeping_commit"]
        );

        // (1) main's tree is clean — the rewrite is committed, not dirt.
        assert!(git_status_porcelain_str(&main).is_empty(), "{}", git_status_porcelain_str(&main));

        // (2) the lane commit is a NEW commit on top of the merge commit:
        // HEAD has exactly one parent (an ordinary commit, not amended into
        // the merge), and HEAD~1 (the merge commit itself) still has two —
        // it was never rewritten.
        let head_parents =
            js_trim(&run_git(&main, &["rev-list", "--parents", "-1", "HEAD"]).stdout.unwrap_or_default())
                .to_string();
        assert_eq!(
            head_parents.split_whitespace().count(),
            2,
            "HEAD (the lane commit) must have exactly one parent: {head_parents}"
        );
        let merge_parents = js_trim(
            &run_git(&main, &["rev-list", "--parents", "-1", "HEAD~1"]).stdout.unwrap_or_default(),
        )
        .to_string();
        assert_eq!(
            merge_parents.split_whitespace().count(),
            3,
            "HEAD~1 (the merge commit) must still carry both its parents, unamended: {merge_parents}"
        );

        // (3) the lane commit's diff names only the lane path.
        let touched = js_trim(
            &run_git(&main, &["diff", "--name-only", "HEAD~1", "HEAD"]).stdout.unwrap_or_default(),
        )
        .to_string();
        assert_eq!(touched, ".bee/lanes/demo.json", "{touched}");
    }

    /// Truth 2a: the same standard-lane feature merges once its uat gate is
    /// approved on the live workflow record.
    #[test]
    fn merge_proceeds_once_the_live_workflow_uat_gate_is_approved() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", true);
        // defaults-and-agent-env D1: spell "merge" explicitly so this stays
        // a genuine test of the approval read at merge time, not a
        // trivial pass under the new absent-key default (Close never
        // blocks at merge regardless of approval).
        std::fs::write(main.join(".bee").join("config.json"), r#"{"uat_stop": "merge"}"#).unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge refused despite an approved uat gate: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    /// Truth 2b: `--skip-uat` bypasses an unapproved gate for JUST this
    /// merge call.
    #[test]
    fn skip_uat_flag_bypasses_an_unapproved_gate() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", false);
        // defaults-and-agent-env D1: spell "merge" explicitly so
        // --skip-uat is genuinely exercised against a door that would
        // otherwise block, rather than a Close default that never blocks
        // at merge in the first place.
        std::fs::write(main.join(".bee").join("config.json"), r#"{"uat_stop": "merge"}"#).unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("--skip-uat must bypass an unapproved gate: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    /// Truth 2c: an explicit repo-wide `uat_before_merge: false` bypasses
    /// the precondition entirely, without `--skip-uat`.
    #[test]
    fn uat_before_merge_config_false_bypasses_an_unapproved_gate() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", false);
        std::fs::write(
            main.join(".bee").join("config.json"),
            "{\"uat_before_merge\": false}",
        )
        .unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("uat_before_merge: false must bypass an unapproved gate: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    /// Truth 2d: a `tiny` lane merges without any uat approval — the door
    /// only ever applies to standard/high-risk. This exercises the LANE
    /// fallback read (`.bee/lanes/<feature>.json`, no live workflow at all).
    #[test]
    fn tiny_lane_merges_without_uat_approval() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_lane_mode(&main, "demo", "tiny");

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("a tiny lane must never require uat approval: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    /// Truth 3: a present-but-non-boolean `uat_before_merge` config value is
    /// refused outright, typed, even on an otherwise-exempt tiny lane — a
    /// typo'd config must never silently resolve to either outcome.
    #[test]
    fn uat_before_merge_config_non_boolean_is_refused_regardless_of_lane() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_lane_mode(&main, "demo", "tiny");
        std::fs::write(
            main.join(".bee").join("config.json"),
            "{\"uat_before_merge\": \"nope\"}",
        )
        .unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, false, None);
        let Err(err) = result else { panic!("a non-boolean uat_before_merge must refuse") };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_UAT_CONFIG_INVALID"), "{msg}");
        assert!(created.worktree_root.exists());
    }

    /// Truth 4: with no live workflow AND no lane record, `uat_merge_precheck`
    /// falls back to the plain default `.bee/state.json` record's
    /// `approved_gates.uat` — but ONLY when that record is presently
    /// tracking THIS feature.
    #[test]
    fn merge_proceeds_via_the_default_state_record_fallback_when_it_names_this_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        // defaults-and-agent-env D1: spell "merge" explicitly so this stays
        // a genuine test of the default-state approval fallback, not a
        // trivial pass under the new absent-key default (Close never
        // blocks at merge regardless of approval).
        std::fs::write(
            main.join(".bee").join("state.json"),
            jsjson::stringify(&json!({
                "feature": "demo",
                "mode": "standard",
                "approved_gates": { "uat": true },
            })),
        )
        .unwrap();
        std::fs::write(main.join(".bee").join("config.json"), r#"{"uat_stop": "merge"}"#).unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("the default state.json fallback must approve this feature's uat gate: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    /// Truth 4a (the safety half of Truth 4): a default state.json record
    /// naming a DIFFERENT feature's approval must never leak through as
    /// this feature's own uat approval.
    #[test]
    fn a_different_features_state_json_approval_does_not_leak_through() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        std::fs::write(
            main.join(".bee").join("state.json"),
            jsjson::stringify(&json!({
                "feature": "some-other-feature",
                "mode": "standard",
                "approved_gates": { "uat": true },
            })),
        )
        .unwrap();
        // defaults-and-agent-env D1: spell "merge" explicitly — the new
        // absent-key default (Close) never blocks at merge, which would
        // make this refusal assertion pass for the wrong reason.
        std::fs::write(main.join(".bee").join("config.json"), r#"{"uat_stop": "merge"}"#).unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, false, None);
        let Err(err) = result else {
            panic!("a different feature's approved_gates.uat must never approve THIS feature's merge")
        };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_UAT_PENDING"), "{msg}");
    }

    // `uat_before_merge_config` (the standalone config reader) is retired —
    // uat-stop-placement D1 folds its shape into `crate::uat::uat_stop_config`,
    // whose own unit tests (uat.rs) already pin absent/true/false/non-boolean
    // exactly the way this test used to.

    /// `--skip-uat` parses as a bare boolean (FLAG_ALONE_BOOLEANS) and its
    /// non-boolean shape is refused outright, the same discipline
    /// `no_cleanup_with_a_non_boolean_value_is_refused_not_defaulted` pins
    /// for `--no-cleanup`.
    #[test]
    fn skip_uat_flag_parses_as_a_bare_boolean_and_refuses_a_non_boolean_value() {
        let (bare, _) = parse_flags(&["--skip-uat"]).unwrap();
        assert!(bool_flag_ok(&bare, "skip-uat"));
        assert!(bool_flag_true(&bare, "skip-uat"));
        let (explicit_false, _) = parse_flags(&["--skip-uat=false"]).unwrap();
        assert!(bool_flag_ok(&explicit_false, "skip-uat"));
        assert!(!bool_flag_true(&explicit_false, "skip-uat"));
        let (bad, _) = parse_flags(&["--skip-uat=yes"]).unwrap();
        assert!(!bool_flag_ok(&bad, "skip-uat"), "a non-boolean --skip-uat must refuse outright");
    }

    // ── uat-stop-placement D4: the merge side of D4.1-D4.3 ──────────────────
    // D4.1's precondition and D4.2's post-merge lane write both branch on
    // the SAME resolved `uat_stop`, now exercised through the new key
    // directly (not just its `uat_before_merge` back-compat alias, which
    // Truth 2a-2d above already cover).

    fn write_uat_stop_config(main: &Path, value: &str) {
        std::fs::write(
            main.join(".bee").join("config.json"),
            jsjson::stringify(&json!({ "uat_stop": value })),
        )
        .unwrap();
    }

    /// D4.1: `uat_stop: "merge"` (today's default, spelled explicitly)
    /// refuses an unapproved standard-lane feature exactly like the
    /// `uat_before_merge` alias already does (Truth 1).
    #[test]
    fn uat_stop_merge_key_unapproved_standard_lane_refuses() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", false);
        write_uat_stop_config(&main, "merge");

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, false, None);
        let Err(err) = result else { panic!("uat_stop: merge must still refuse an unapproved gate") };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_UAT_PENDING"), "{msg}");
    }

    /// D4.1/D4.2: `uat_stop: "merge"`, approved — merges, and the
    /// merge-time lane clear (D4.2's unchanged branch) still fires.
    #[test]
    fn uat_stop_merge_key_approved_standard_lane_merges_and_clears_lane() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", true);
        write_stranded_lane(&main, "demo", "scribing");
        write_uat_stop_config(&main, "merge");

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("an approved gate under uat_stop merge must merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        let lane = crate::verbs::workflow_store::read_lane_strict(&main, "demo").unwrap().unwrap();
        assert_eq!(lane.get("waiting_on"), Some(&Value::Null), "{lane:?}");
        let next_action = lane.get("next_action").and_then(Value::as_str).unwrap_or_default();
        assert!(next_action.contains("bee close --feature demo"), "{next_action}");
    }

    /// D4.2 — the INVERSION: `uat_stop: "close"` on a standard lane with an
    /// unapproved gate does NOT refuse the merge; it SETS the lane's
    /// `waiting_on` to a "gate" mark naming "uat: demo" and points
    /// `next_action` at the reload-test-approve-or-fix road instead of
    /// `bee close`.
    #[test]
    fn uat_stop_close_unapproved_standard_lane_merges_and_sets_the_uat_wait() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", false);
        write_lane_mode(&main, "demo", "standard");
        write_uat_stop_config(&main, "close");

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("uat_stop: close must never refuse the merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));

        let lane = crate::verbs::workflow_store::read_lane_strict(&main, "demo")
            .unwrap()
            .expect("the merge must have written demo's lane");
        let waiting_on = lane.get("waiting_on").cloned().unwrap_or(Value::Null);
        assert_eq!(waiting_on.get("kind").and_then(Value::as_str), Some("gate"), "{waiting_on:?}");
        assert_eq!(waiting_on.get("subject").and_then(Value::as_str), Some("uat: demo"), "{waiting_on:?}");
        let next_action = lane.get("next_action").and_then(Value::as_str).unwrap_or_default();
        assert!(next_action.contains("reload"), "{next_action}");
        assert!(next_action.contains("approve uat"), "{next_action}");
        assert!(next_action.contains("merge again"), "{next_action}");
        assert!(!next_action.contains("bee close"), "{next_action}");
        assert_eq!(answer.result["next_action"], json!(next_action));
    }

    /// D4.2's un-inverted half: `uat_stop: "close"` with an ALREADY
    /// approved gate keeps today's clear-and-point-at-close behavior.
    #[test]
    fn uat_stop_close_approved_standard_lane_merges_and_clears_lane() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", true);
        write_stranded_lane(&main, "demo", "scribing");
        write_uat_stop_config(&main, "close");

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("an approved gate under uat_stop close must merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        let lane = crate::verbs::workflow_store::read_lane_strict(&main, "demo").unwrap().unwrap();
        assert_eq!(lane.get("waiting_on"), Some(&Value::Null), "{lane:?}");
        let next_action = lane.get("next_action").and_then(Value::as_str).unwrap_or_default();
        assert!(next_action.contains("bee close --feature demo"), "{next_action}");
    }

    /// D4.1/D4.2: `uat_stop: "off"` never refuses, and the lane write keeps
    /// today's clear behavior too — "off" means no uat stop ANYWHERE, not
    /// just at merge time.
    #[test]
    fn uat_stop_off_unapproved_standard_lane_merges_and_clears_lane() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", false);
        write_stranded_lane(&main, "demo", "scribing");
        write_uat_stop_config(&main, "off");

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("uat_stop: off must never refuse the merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        let lane = crate::verbs::workflow_store::read_lane_strict(&main, "demo").unwrap().unwrap();
        assert_eq!(lane.get("waiting_on"), Some(&Value::Null), "{lane:?}");
        let next_action = lane.get("next_action").and_then(Value::as_str).unwrap_or_default();
        assert!(next_action.contains("bee close --feature demo"), "{next_action}");
    }

    /// The trivial half of the "off" pair — an approved gate merges and
    /// clears exactly the same way.
    #[test]
    fn uat_stop_off_approved_standard_lane_merges_and_clears_lane() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", true);
        write_stranded_lane(&main, "demo", "scribing");
        write_uat_stop_config(&main, "off");

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("uat_stop: off must never refuse the merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        let lane = crate::verbs::workflow_store::read_lane_strict(&main, "demo").unwrap().unwrap();
        assert_eq!(lane.get("waiting_on"), Some(&Value::Null), "{lane:?}");
    }

    /// D2/D4.2: a `tiny` lane under `uat_stop: "close"` is exempt — the
    /// lane rule (`uat_gate_applies_to_lane`) never applies, so the merge
    /// keeps today's clear-and-point-at-close behavior even with no uat
    /// approval at all.
    #[test]
    fn uat_stop_close_tiny_lane_is_exempt_and_clears_the_lane() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_lane_mode(&main, "demo", "tiny");
        write_uat_stop_config(&main, "close");

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("a tiny lane must be exempt under uat_stop close too: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        let lane = crate::verbs::workflow_store::read_lane_strict(&main, "demo").unwrap().unwrap();
        assert!(
            !crate::verbs::workflow_store::waiting_on_is_live(lane.get("waiting_on")),
            "{lane:?}"
        );
        let next_action = lane.get("next_action").and_then(Value::as_str).unwrap_or_default();
        assert!(next_action.contains("bee close --feature demo"), "{next_action}");
    }

    /// D4.3: while the uat wait set by THIS merge is live, `--cleanup` is
    /// ignored — the merge itself still lands, but the worktree survives
    /// and the result explains why instead of tearing it down.
    #[test]
    fn uat_stop_close_pending_uat_forces_the_cleanup_flag_off() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", false);
        write_lane_mode(&main, "demo", "standard");
        write_uat_stop_config(&main, "close");

        let cleanup = resolve_cleanup_on_merge(&main, true, false).unwrap();
        assert!(cleanup, "the flag alone must resolve cleanup to true before the merge overrides it");
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("uat_stop: close must never refuse the merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        let cleanup_field = &answer.result["cleanup"];
        assert_eq!(cleanup_field["ok"], json!(false), "{cleanup_field:?}");
        assert_eq!(
            cleanup_field["code"],
            json!("WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING"),
            "{cleanup_field:?}"
        );
        assert!(created.worktree_root.exists(), "the worktree must survive a suppressed cleanup");

        let lines = merge_text_lines(&created.id, &main, &answer);
        assert!(
            lines.iter().any(|l| l.contains("WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING")),
            "the merge text must carry one visible line naming why cleanup was suppressed: {lines:?}"
        );
    }

    /// D4.3's other trigger: `worktree_cleanup_on_merge: true` in config is
    /// ignored the same way `--cleanup` is.
    #[test]
    fn uat_stop_close_pending_uat_forces_the_cleanup_config_off() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", false);
        write_lane_mode(&main, "demo", "standard");
        std::fs::write(
            main.join(".bee").join("config.json"),
            jsjson::stringify(&json!({ "uat_stop": "close", "worktree_cleanup_on_merge": true })),
        )
        .unwrap();

        let cleanup = resolve_cleanup_on_merge(&main, false, false).unwrap();
        assert!(cleanup, "the config opt-in alone must resolve cleanup to true before the merge overrides it");
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("uat_stop: close must never refuse the merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        let cleanup_field = &answer.result["cleanup"];
        assert_eq!(cleanup_field["ok"], json!(false), "{cleanup_field:?}");
        assert_eq!(
            cleanup_field["code"],
            json!("WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING"),
            "{cleanup_field:?}"
        );
        assert!(created.worktree_root.exists(), "the worktree must survive a suppressed cleanup");
    }

    /// D4.3's own reason, proven: a SECOND merge after the uat wait is live
    /// still works — the grant is still present because cleanup was
    /// suppressed, so this never trips the no-granted-worktree refusal.
    #[test]
    fn uat_stop_close_repeat_merge_after_the_uat_wait_still_works() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", false);
        write_lane_mode(&main, "demo", "standard");
        write_uat_stop_config(&main, "close");

        // `--cleanup` is requested on the FIRST merge — D4.3 must force it
        // off (the wait it just set is live), which is what keeps the
        // worktree/grant standing for the second merge below to reuse.
        let cleanup = resolve_cleanup_on_merge(&main, true, false).unwrap();
        let first = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("the first merge must land and set the uat wait: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(first.ok, "{:?}", first.result);
        assert_eq!(first.result["merged"], Value::Bool(true));
        assert_eq!(
            first.result["cleanup"]["code"],
            json!("WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING"),
            "{:?}",
            first.result
        );
        assert!(created.worktree_root.exists(), "the worktree must survive the first merge's suppressed cleanup");

        let second = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("a repeat merge with the grant still present must not refuse: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(second.ok, "{:?}", second.result);
        assert_eq!(second.result["code"], json!("ALREADY_UP_TO_DATE"));
    }

    // ── usp-5: a missing lane record must not let cleanup fire ─────────────
    // A judge observation against usp-2: `set_lane_uat_wait_on_merge` used
    // to return `uat_wait_set: false` whenever the feature had no
    // `.bee/lanes/<feature>.json` on disk, which let `merge_finish`'s
    // `effective_cleanup = cleanup && !uat_wait_set` tear the worktree down
    // out from under a user who still owed a "uat" test — asymmetric with
    // the merge-time precondition, which already fails CLOSED
    // (`uat_gate_applies_to_lane(None)` is `true`) in the exact same
    // no-lane-record case. These three cells never call `write_lane_mode`
    // or `write_stranded_lane`, so `.bee/lanes/demo.json` never exists on
    // disk before the merge — only the live workflow record
    // (`write_live_workflow_uat`) names the feature's "standard" mode and
    // its "uat" approval.

    /// The hole itself, fixed: an unapproved uat, `uat_stop: "close"`, and
    /// NO lane record on disk still suppresses `--cleanup` — the worktree,
    /// its branch, and the grant all survive, and the merge still reports
    /// the SAME suppression code the lane-record case does. The missing
    /// lane record is proven a non-event: no red merge, and no lane file
    /// fabricated by the mere act of merging.
    #[test]
    fn uat_stop_close_pending_uat_with_no_lane_record_still_suppresses_cleanup() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", false);
        write_uat_stop_config(&main, "close");
        assert!(
            !main.join(".bee").join("lanes").join("demo.json").exists(),
            "fixture must start with no lane record at all"
        );

        let cleanup = resolve_cleanup_on_merge(&main, true, false).unwrap();
        assert!(cleanup, "the flag alone must resolve cleanup to true before the merge overrides it");
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("uat_stop: close must never refuse the merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "a missing lane record must never redden the merge: {:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));

        let cleanup_field = &answer.result["cleanup"];
        assert_eq!(cleanup_field["ok"], json!(false), "{cleanup_field:?}");
        assert_eq!(
            cleanup_field["code"],
            json!("WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING"),
            "a missing lane record must fail closed the SAME way a present one does: {cleanup_field:?}"
        );
        assert!(
            created.worktree_root.exists(),
            "the worktree must survive a suppressed cleanup even with no lane record on disk"
        );
        let branches = std::process::Command::new("git")
            .args(["branch", "--list", &created.branch])
            .current_dir(&main)
            .output()
            .unwrap();
        assert!(
            !String::from_utf8_lossy(&branches.stdout).trim().is_empty(),
            "the branch must survive a suppressed cleanup"
        );
        assert!(
            !main.join(".bee").join("lanes").join("demo.json").exists(),
            "the lane write stays best-effort — a missing lane record must never be fabricated by a merge"
        );
    }

    /// The reason the suppression matters, proven with no lane record: a
    /// SECOND merge on that same worktree afterwards still runs (this repo
    /// has no new commits by then, so it lands as `ALREADY_UP_TO_DATE`)
    /// rather than hitting the no-granted-worktree refusal — which is
    /// exactly what a wrongly-torn-down worktree would have caused.
    #[test]
    fn uat_stop_close_pending_uat_with_no_lane_record_repeat_merge_still_works() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", false);
        write_uat_stop_config(&main, "close");

        let cleanup = resolve_cleanup_on_merge(&main, true, false).unwrap();
        let first = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("the first merge must land and suppress cleanup: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(first.ok, "{:?}", first.result);
        assert_eq!(
            first.result["cleanup"]["code"],
            json!("WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING"),
            "{:?}",
            first.result
        );
        assert!(
            created.worktree_root.exists(),
            "the worktree must survive the first merge's suppressed cleanup"
        );

        let second = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("a repeat merge with the grant still present must not refuse: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(second.ok, "{:?}", second.result);
        assert_eq!(second.result["code"], json!("ALREADY_UP_TO_DATE"));
    }

    /// The other half, unchanged: with the SAME no-lane-record fixture but
    /// uat APPROVED, cleanup runs normally — the fix touches only the
    /// unapproved/no-lane-record hole, never the approved path.
    #[test]
    fn uat_stop_close_approved_uat_with_no_lane_record_cleans_up_normally() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", true);
        write_uat_stop_config(&main, "close");
        assert!(
            !main.join(".bee").join("lanes").join("demo.json").exists(),
            "fixture must start with no lane record at all"
        );

        let cleanup = resolve_cleanup_on_merge(&main, true, false).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("an approved gate under uat_stop close must merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["cleanup"]["ok"], Value::Bool(true), "{:?}", answer.result);
        assert!(
            !created.worktree_root.exists(),
            "an approved uat must clean up normally even with no lane record on disk"
        );
    }

    // ── usp-7: an UNRESOLVABLE feature must fail closed too ────────────────
    // The last instance of the same asymmetry usp-3 and usp-5 already fixed
    // elsewhere: `merge_finish`'s own `uat_wait_set` used to gate the
    // `uat_merge_precheck` call behind `feature.as_deref().is_some_and(...)`,
    // which short-circuited to `false` — UNsuppressed cleanup — the instant
    // `Staged.feature` was `None`, even though `uat_merge_precheck(main_root,
    // None)` itself already answers `lane_applies: true, gate_approved:
    // false` (the SAME fail-closed shape the merge-time precondition already
    // reads via `uat_gate_applies_to_lane(None) == true`). These fixtures
    // strip BOTH of `resolve_worktree_feature`'s reads after creation so
    // `Staged.feature` genuinely resolves to `None`, the one shape that
    // exercises the hole rather than asserting around it (the branch check
    // falls back to `wt_branch_shaped` when `identity.feature` is `None`, so
    // the branch itself is left untouched and still matches).

    /// Strips the immutable creation slug and the mutable `.bee/state.json`
    /// `feature` field from an already-created worktree, so
    /// `resolve_worktree_feature` genuinely returns `feature: None` — the
    /// exact shape `worktree_feature_prefers_the_immutable_creation_slug`'s
    /// "neither file" case above proves resolves to `None`.
    fn make_feature_unresolvable(worktree_root: &Path) {
        std::fs::remove_file(
            worktree_root.join(".bee").join("runtime").join("worktree-identity.json"),
        )
        .unwrap();
        std::fs::write(worktree_root.join(".bee").join("state.json"), "{}\n").unwrap();
    }

    /// The hole itself, fixed: `uat_stop: "close"`, a feature that cannot be
    /// resolved at merge time, and `--cleanup` still suppresses — the
    /// worktree, its branch, and the grant all survive, and the merge
    /// reports the SAME suppression code the resolvable-pending case does.
    #[test]
    fn uat_stop_close_unresolvable_feature_still_suppresses_cleanup() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_uat_stop_config(&main, "close");
        make_feature_unresolvable(&created.worktree_root);
        assert_eq!(
            resolve_worktree_feature(&created.worktree_root).feature,
            None,
            "fixture must genuinely leave the feature unresolvable"
        );

        let cleanup = resolve_cleanup_on_merge(&main, true, false).unwrap();
        assert!(cleanup, "the flag alone must resolve cleanup to true before the merge overrides it");
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!(
                    "an unresolvable feature must never redden the merge under uat_stop close: {m}"
                ),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));

        let cleanup_field = &answer.result["cleanup"];
        assert_eq!(cleanup_field["ok"], json!(false), "{cleanup_field:?}");
        assert_eq!(
            cleanup_field["code"],
            json!("WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING"),
            "an unresolvable feature must fail closed the SAME way a resolvable-pending one does: {cleanup_field:?}"
        );
        assert!(
            created.worktree_root.exists(),
            "the worktree must survive a suppressed cleanup even with an unresolvable feature"
        );
        let branches = std::process::Command::new("git")
            .args(["branch", "--list", &created.branch])
            .current_dir(&main)
            .output()
            .unwrap();
        assert!(
            !String::from_utf8_lossy(&branches.stdout).trim().is_empty(),
            "the branch must survive a suppressed cleanup"
        );
    }

    /// The reason the suppression matters, proven with an unresolvable
    /// feature: a SECOND merge on that same worktree afterwards still runs
    /// (no new commits by then, so it lands as `ALREADY_UP_TO_DATE`) rather
    /// than hitting the no-granted-worktree refusal — exactly what a
    /// wrongly-torn-down worktree would have caused.
    #[test]
    fn uat_stop_close_unresolvable_feature_repeat_merge_still_works() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_uat_stop_config(&main, "close");
        make_feature_unresolvable(&created.worktree_root);

        let cleanup = resolve_cleanup_on_merge(&main, true, false).unwrap();
        let first = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("the first merge must land and suppress cleanup: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(first.ok, "{:?}", first.result);
        assert_eq!(
            first.result["cleanup"]["code"],
            json!("WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING"),
            "{:?}",
            first.result
        );
        assert!(
            created.worktree_root.exists(),
            "the worktree must survive the first merge's suppressed cleanup"
        );

        let second = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("a repeat merge with the grant still present must not refuse: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(second.ok, "{:?}", second.result);
        assert_eq!(second.result["code"], json!("ALREADY_UP_TO_DATE"));
    }

    /// Merge is unchanged: an unresolvable feature under `uat_stop: "merge"`
    /// still hits the pre-existing zero-mutation precondition
    /// (`WORKTREE_MERGE_UAT_PENDING`) exactly as before this cell — the fix
    /// touches only `merge_finish`'s cleanup decision, never the merge-time
    /// refusal.
    #[test]
    fn uat_stop_merge_unresolvable_feature_precondition_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_uat_stop_config(&main, "merge");
        make_feature_unresolvable(&created.worktree_root);

        let cleanup = resolve_cleanup_on_merge(&main, true, false).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, false, None);
        let Err(err) = result else {
            panic!("uat_stop: merge must still refuse an unresolvable feature's pending gate")
        };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_UAT_PENDING"), "{msg}");
    }

    /// Merge, skipped: with `--skip-uat`, the same unresolvable-feature
    /// worktree merges and cleans up normally — `uat_stop: "merge"` never
    /// reaches `merge_finish`'s `uat_wait_set` at all (it stays gated behind
    /// `*uat_stop == UatStop::Close`), so this cell's fix cannot suppress a
    /// cleanup outside `"close"`.
    #[test]
    fn uat_stop_merge_unresolvable_feature_skip_uat_cleans_up_normally() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_uat_stop_config(&main, "merge");
        make_feature_unresolvable(&created.worktree_root);

        let cleanup = resolve_cleanup_on_merge(&main, true, false).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("--skip-uat must still merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["cleanup"]["ok"], Value::Bool(true), "{:?}", answer.result);
        assert!(
            !created.worktree_root.exists(),
            "an unresolvable feature under uat_stop merge (skipped) must clean up normally"
        );
    }

    /// Off, unchanged: an unresolvable feature under `uat_stop: "off"` still
    /// cleans up normally — `merge_finish`'s `uat_wait_set` stays gated
    /// behind `*uat_stop == UatStop::Close`, so `"off"` never reaches it.
    #[test]
    fn uat_stop_off_unresolvable_feature_cleans_up_normally() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_uat_stop_config(&main, "off");
        make_feature_unresolvable(&created.worktree_root);

        let cleanup = resolve_cleanup_on_merge(&main, true, false).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("uat_stop off must never refuse: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["cleanup"]["ok"], Value::Bool(true), "{:?}", answer.result);
        assert!(
            !created.worktree_root.exists(),
            "an unresolvable feature under uat_stop off must clean up normally"
        );
    }

    /// `--no-cleanup` still means keep: an unresolvable feature that was
    /// never asked to clean up in the first place keeps the worktree for an
    /// entirely separate reason (`cleanup == false`), unrelated to this
    /// cell's `uat_wait_set` fix.
    #[test]
    fn uat_stop_close_unresolvable_feature_no_cleanup_flag_still_means_keep() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_uat_stop_config(&main, "close");
        make_feature_unresolvable(&created.worktree_root);

        let cleanup = resolve_cleanup_on_merge(&main, false, false).unwrap();
        assert!(!cleanup, "no --cleanup flag and no config default must resolve to false");
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("an unrequested cleanup must never refuse the merge: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert!(
            created.worktree_root.exists(),
            "--no-cleanup (no request at all) must keep the worktree regardless of uat_wait_set"
        );
    }

    // ── D5: no-write paths, preserved ────────────────────────────────────

    /// D5: a real textual conflict writes NOTHING to the merging feature's
    /// lane — `merge_stage`'s `MERGE_CONFLICT` arm returns long before
    /// `merge_finish` (and this cell's lane write inside it) is ever
    /// reached.
    #[test]
    fn a_merge_conflict_leaves_the_lane_record_byte_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_stranded_lane(&main, "demo", "scribing");
        let lane_path = main.join(".bee").join("lanes").join("demo.json");
        let before = std::fs::read(&lane_path).unwrap();

        // Diverge main on the SAME file the worktree already committed, so
        // the merge hits a real textual conflict instead of a clean one.
        std::fs::write(main.join("f.txt"), "conflict").unwrap();
        git_ok(&main, &["add", "-A", "--", "f.txt"]);
        git_ok(&main, &["commit", "-qm", "diverge on main"]);

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("a textual conflict must return Ok(ok:false), not throw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(!answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["code"], json!("MERGE_CONFLICT"));

        let after = std::fs::read(&lane_path).unwrap();
        assert_eq!(before, after, "a textual-conflict merge must not touch the lane record at all");
    }

    /// D5: a `WORKTREE_MERGE_PROOF_DEBT` refusal is a zero-mutation
    /// precondition too — it writes nothing to the merging feature's lane
    /// either.
    #[test]
    fn a_proof_debt_refusal_leaves_the_lane_record_byte_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "unproofed", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();
        std::fs::write(wt.join("f.txt"), "y").unwrap();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        git_ok(&wt, &["commit", "-qam", "work"]);
        write_capped_cell(
            &main,
            "unproofed-1",
            "unproofed",
            Some(json!({
                "outcome": "did the thing",
                "commit": "abc123",
                "files": [],
                "tests": "",
                "deviations": [],
            })),
        );
        write_stranded_lane(&main, "unproofed", "scribing");
        let lane_path = main.join(".bee").join("lanes").join("unproofed.json");
        let before = std::fs::read(&lane_path).unwrap();

        let result = merge_feature_worktree(&main, &created.id, false, None, true, None);
        let Err(err) = result else { panic!("an unproven cap must still refuse") };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_PROOF_DEBT"), "{msg}");

        let after = std::fs::read(&lane_path).unwrap();
        assert_eq!(before, after, "a proof-debt refusal must not touch the lane record at all");

    }

    // ── docs/history/uat-approval-reaches-the-door: the lane fallback ──────
    //
    // The defect: `bee gate --name uat --approved true --lane <f>` writes
    // `approved_gates.uat = true` into `.bee/lanes/<f>.json` whenever the
    // feature's workflow record is already `closed` — but neither
    // `uat_merge_precheck` (above) nor the close-time door ever read that
    // file, so the approval landed nowhere either door looked. These tests
    // exercise the NEW second source `crate::uat::uat_gate_approved` adds:
    // the lane record, consulted only once a live workflow record is absent
    // (here, explicitly `status: "closed"`, the exact shape
    // `bee state workflows close --all-but-active` produces).

    /// A `.bee/runtime/workflows/<id>/state.json` record for `feature` whose
    /// `status` is `"closed"` — `find_live_workflow` excludes it outright,
    /// so it is invisible to source 1 regardless of what its own
    /// `gates.uat.approved` says. Stamped `true` here on purpose: proves a
    /// closed record's own approval is never consulted, only its ABSENCE
    /// from the live set is what matters.
    fn write_closed_workflow_uat(main: &Path, feature: &str, mode: &str) {
        let id = format!("wf-{feature}-uat-test-closed");
        let dir = main.join(".bee").join("runtime").join("workflows").join(&id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("state.json"),
            jsjson::stringify(&json!({
                "id": id,
                "feature": feature,
                "status": "closed",
                "mode": mode,
                "gates": { "uat": { "approved": true } },
            })),
        )
        .unwrap();
    }

    /// A `.bee/lanes/<feature>.json` record naming a risk lane, with
    /// `approved_gates` set to whatever shape the caller wants to probe —
    /// `None` omits the `approved_gates` key entirely (the "lane file with
    /// no approved_gates" negative case); `Some(gates)` writes `gates`
    /// verbatim as `approved_gates` (e.g. `json!({})` for "no uat key",
    /// `json!({"uat": false})`, or a non-boolean `uat`).
    fn write_lane_approved_gates(main: &Path, feature: &str, mode: &str, gates: Option<Value>) {
        let dir = main.join(".bee").join("lanes");
        std::fs::create_dir_all(&dir).unwrap();
        let mut body = json!({ "feature": feature, "mode": mode });
        if let Some(gates) = gates {
            body.as_object_mut().unwrap().insert("approved_gates".into(), gates);
        }
        std::fs::write(dir.join(format!("{feature}.json")), jsjson::stringify(&body)).unwrap();
    }

    /// Happy path (merge side): a closed-record feature whose lane file
    /// reads `approved_gates.uat: true` merges under an explicit
    /// `uat_stop: "merge"` placement — the approval the owner recorded now
    /// reaches the door that blocks on it.
    #[test]
    fn merge_proceeds_once_a_closed_records_lane_file_reads_uat_true() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_uat_stop_config(&main, "merge");
        write_closed_workflow_uat(&main, "demo", "standard");
        write_lane_approved_gates(&main, "demo", "standard", Some(json!({ "uat": true })));

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, false, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => {
                    panic!("a lane-file uat approval on a closed-record feature must merge: {m}")
                }
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
    }

    /// Happy path (close side): the identical closed-record-plus-lane-file
    /// shape clears the close-time uat door under `uat_stop: "close"`.
    #[test]
    fn close_door_does_not_block_once_a_closed_records_lane_file_reads_uat_true() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        write_uat_stop_config(&main, "close");
        write_closed_workflow_uat(&main, "demo", "standard");
        write_lane_approved_gates(&main, "demo", "standard", Some(json!({ "uat": true })));

        let doors = crate::verbs::drivers::build_close_report_doors(&main, "demo").unwrap();
        let uat_door = doors.iter().find(|d| d.door == "uat").expect("the door must exist for a standard lane");
        assert!(!uat_door.blocking, "a lane-file uat approval on a closed-record feature must clear the door");
        assert_eq!(uat_door.detail, "clear");
    }

    /// Precedence: a live (non-closed) workflow record saying `false` beats
    /// a lane file saying `true` — the live record is consulted first and
    /// its answer stands, never overridden by a later source.
    #[test]
    fn a_live_workflow_saying_false_beats_a_lane_file_saying_true() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        // defaults-and-agent-env D1: spell "merge" explicitly — the new
        // absent-key default (Close) never blocks at merge, which would
        // make this refusal assertion pass for the wrong reason.
        std::fs::write(main.join(".bee").join("config.json"), r#"{"uat_stop": "merge"}"#).unwrap();
        let created = worktree_with_a_real_commit(&main, "demo");
        write_live_workflow_uat(&main, "demo", "standard", false);
        write_lane_approved_gates(&main, "demo", "standard", Some(json!({ "uat": true })));

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let result = merge_feature_worktree(&main, &created.id, cleanup, None, false, None);
        let Err(err) = result else {
            panic!("a live record's false must stand even though the lane file reads true")
        };
        let MErr::Thrown(msg) = err else { panic!("expected a typed refusal, got MErr::Ex") };
        assert!(msg.contains("WORKTREE_MERGE_UAT_PENDING"), "{msg}");
    }

    /// The load-bearing negative set: with the record closed and no live
    /// record, each of these lane-file shapes must still read UNAPPROVED
    /// and refuse — a wrong read here would let an unapproved merge
    /// through.
    #[test]
    fn closed_record_negative_lane_shapes_all_refuse() {
        let cases: [(&str, Option<Value>); 5] = [
            ("no lane file at all", None),
            ("approved_gates absent", Some(json!({}))),
            ("approved_gates present, uat absent", Some(json!({ "approved_gates": json!({}) }))),
            ("uat: false", Some(json!({ "approved_gates": json!({ "uat": false }) }))),
            (
                "uat present but not a boolean",
                Some(json!({ "approved_gates": json!({ "uat": "yes" }) })),
            ),
        ];

        for (label, lane_body) in cases {
            let tmp = tempfile::tempdir().unwrap();
            let main = main_repo(tmp.path());
            // defaults-and-agent-env D1: spell "merge" explicitly — the
            // new absent-key default (Close) never blocks at merge, which
            // would make this refusal assertion pass for the wrong reason.
            std::fs::write(main.join(".bee").join("config.json"), r#"{"uat_stop": "merge"}"#).unwrap();
            let created = worktree_with_a_real_commit(&main, "demo");
            write_closed_workflow_uat(&main, "demo", "standard");
            match lane_body {
                None => {} // no lane file at all
                Some(body) => {
                    let dir = main.join(".bee").join("lanes");
                    std::fs::create_dir_all(&dir).unwrap();
                    let mut full = json!({ "feature": "demo", "mode": "standard" });
                    for (k, v) in body.as_object().unwrap() {
                        full.as_object_mut().unwrap().insert(k.clone(), v.clone());
                    }
                    std::fs::write(dir.join("demo.json"), jsjson::stringify(&full)).unwrap();
                }
            }

            let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
            let result = merge_feature_worktree(&main, &created.id, cleanup, None, false, None);
            let Err(err) = result else { panic!("{label}: must still refuse as unapproved") };
            let MErr::Thrown(msg) = err else { panic!("{label}: expected a typed refusal, got MErr::Ex") };
            assert!(msg.contains("WORKTREE_MERGE_UAT_PENDING"), "{label}: {msg}");
        }
    }

    /// Exactly one resolver serves both doors: the merge-side and
    /// close-side outcomes agree on the SAME closed-record-plus-lane-file
    /// input, for both the approved and the unapproved shape.
    #[test]
    fn merge_and_close_doors_agree_on_the_same_closed_record_lane_shape() {
        for (uat, expect_approved) in [(true, true), (false, false)] {
            let tmp = tempfile::tempdir().unwrap();
            let main = main_repo(tmp.path());
            write_closed_workflow_uat(&main, "demo", "standard");
            write_lane_approved_gates(&main, "demo", "standard", Some(json!({ "uat": uat })));

            assert_eq!(
                crate::uat::uat_gate_approved(&main, "demo"),
                expect_approved,
                "uat={uat}"
            );

            write_uat_stop_config(&main, "close");
            let doors = crate::verbs::drivers::build_close_report_doors(&main, "demo").unwrap();
            let uat_door = doors.iter().find(|d| d.door == "uat").expect("door must exist for a standard lane");
            assert_eq!(!uat_door.blocking, expect_approved, "uat={uat} detail={}", uat_door.detail);
        }
    }

    // ── merge-ready-fact D2: a grant that ends removes the fact ─────────────
    //
    // `merge_ready` says "finished in its worktree, waiting for the human to
    // merge". Both ways a worktree grant can end — the merge itself, and
    // `worktree unregister` — end that wait, so both remove the fact. The
    // fact is seeded by writing the lane record directly (test setup;
    // production seeds it from the last cap).

    fn write_merge_ready_lane(main: &Path, feature: &str) {
        let dir = main.join(".bee").join("lanes");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{feature}.json")),
            format!(
                r#"{{"feature":"{feature}","phase":"scribing",
                     "merge_ready":{{"since":"2026-01-01T00:00:00.000Z","branch":"wt/{feature}",
                     "worktree_id":"wt-{feature}","uat":"pending","blocked_by":[]}}}}"#
            ),
        )
        .unwrap();
    }

    fn lane_has_merge_ready(main: &Path, feature: &str) -> bool {
        let raw = std::fs::read_to_string(
            main.join(".bee").join("lanes").join(format!("{feature}.json")),
        )
        .unwrap();
        let parsed: Value = serde_json::from_str(&raw).unwrap();
        parsed.get("merge_ready").is_some_and(|v| !v.is_null())
    }

    /// A green merge drops the fact from the merged feature's lane — and
    /// still commits that TRACKED lane file in its own path-scoped
    /// bookkeeping commit, so the removal never sits as dirt in main.
    #[test]
    fn a_green_merge_removes_the_merged_features_merge_ready_fact_and_commits_the_lane() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_merge_ready_lane(&main, "demo");
        // Track the lane file, exactly as production does — an untracked
        // lane file would hide the commit step this test also pins.
        std::fs::write(
            main.join(".gitignore"),
            ".bee/*\n!.bee/companion-session.json\n!.bee/lanes/\n",
        )
        .unwrap();
        git_ok(&main, &["add", "-A", "--", ".gitignore"]);
        git_ok(&main, &["commit", "-qm", "stop ignoring lanes"]);
        git_ok(&main, &["add", "-f", ".bee/lanes/demo.json"]);
        git_ok(&main, &["commit", "-qm", "track the demo lane"]);
        assert!(lane_has_merge_ready(&main, "demo"), "the fixture must start merge-ready");

        let cleanup = resolve_cleanup_on_merge(&main, false, true).unwrap();
        let answer = merge_feature_worktree(&main, &created.id, cleanup, None, true, None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));

        assert!(
            !lane_has_merge_ready(&main, "demo"),
            "the merge is what ends the wait — the fact goes with it"
        );
        assert_eq!(
            answer.result["lane_bookkeeping_commit"]["committed"],
            Value::Bool(true),
            "the lane rewrite must land in its own commit: {}",
            answer.result["lane_bookkeeping_commit"]
        );
        assert!(
            git_status_porcelain_str(&main).is_empty(),
            "main must be clean after the merge: {}",
            git_status_porcelain_str(&main)
        );
    }

    /// `worktree unregister` ends the grant the other way, and removes the
    /// fact the same way — resolving the feature off the worktree's own
    /// identity BEFORE teardown makes the id unresolvable. A dead id is
    /// silence, never a refusal.
    #[test]
    fn unregistering_a_worktree_removes_its_features_merge_ready_fact() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let created = worktree_with_a_real_commit(&main, "demo");
        write_merge_ready_lane(&main, "demo");

        assert!(
            clear_merge_ready_for_worktree(&main, &created.id),
            "the grant's own feature must resolve off the worktree identity"
        );
        assert!(!lane_has_merge_ready(&main, "demo"));
        assert!(
            !clear_merge_ready_for_worktree(&main, &created.id),
            "clearing twice writes nothing"
        );
        assert!(
            !clear_merge_ready_for_worktree(&main, "wt-does-not-exist"),
            "an unresolvable id is silence, never a throw"
        );
    }
