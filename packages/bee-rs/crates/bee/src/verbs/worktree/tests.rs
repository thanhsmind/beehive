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

    /// attachCleanupOutcome without the flag NEVER runs anything — it attaches
    /// the suggestion (decision D8b: "never prompt").
    #[test]
    fn cleanup_without_the_flag_only_suggests() {
        let tmp = tempfile::tempdir().unwrap();
        let mut result = Map::new();
        attach_cleanup_outcome(&mut result, tmp.path(), tmp.path(), "wt/demo", "wt-demo", false, false);
        assert_eq!(
            result["cleanup_suggested_command"],
            json!("bee worktree merge --id wt-demo --cleanup --json")
        );
        assert!(!result.contains_key("cleanup"));
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
