// Split out of the single 3k-line verbs/reservations.rs. Code unchanged; only module placement and item visibility moved.
//
// Moved verbatim out of the parent file's inline module, indentation
// and all: a moved inline module is the same child of the same parent,
// so no path changes, and the fixtures inside are raw strings whose
// leading whitespace is content.

// The parent module's own `use` block travels with the tests: they reach
// for names mod.rs no longer imports now that the code using them lives
// in sibling modules.
#![allow(unused_imports)]

use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, resolve_store_root_worktree, Roots, RootsWt, StoreRoots};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
    use super::*;

    fn fixture_root() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(tmp.path().join(".bee").join("onboarding.json"), "{}\n").unwrap();
        tmp
    }

    fn params(agent: &str, cell: &str, path: &str) -> ReserveParams {
        ReserveParams {
            agent: agent.into(),
            cell: cell.into(),
            path: path.into(),
            ttl: None,
            session: None,
            kind: None,
        }
    }

    fn root_str(p: &Path) -> String {
        p.to_str().unwrap().to_string()
    }

    /// resolveHoldTopology's answer for an ORDINARY checkout — what every
    /// pre-existing fixture below has always exercised.
    fn main_topo(p: &Path) -> Option<Topo<'_>> {
        Some(Topo { main_root: p, holder: "main" })
    }

    /// Session-resolution tests assume no ambient session identity —
    /// edition-2024 env mutation is unsafe under threaded tests, so skip
    /// instead of scrubbing when the harness itself exports one.
    fn ambient_session() -> bool {
        env_nonempty("BEE_SESSION_ID").is_some() || env_nonempty("CLAUDE_CODE_SESSION_ID").is_some()
    }

    #[test]
    fn paths_overlap_vectors() {
        assert!(paths_overlap("src/api", "src/api"));
        assert!(paths_overlap("src/api", "src/api/router.ts"));
        assert!(paths_overlap("src/api/*", "src/api/router.ts"));
        assert!(paths_overlap("*", "anything"));
        assert!(!paths_overlap("", "src"));
        assert!(!paths_overlap("src/api", "src/apix"));
        assert!(paths_overlap("./src\\api/", "src/api"));
    }

    #[test]
    fn normalize_path_vectors() {
        assert_eq!(res_normalize_path("./a/b/"), "a/b");
        assert_eq!(res_normalize_path(".//a"), "a");
        assert_eq!(res_normalize_path("a\\b"), "a/b");
        assert_eq!(res_normalize_path("./"), "");
    }

    #[test]
    fn number_flag_grammar() {
        assert!(matches!(js_number_flag("60"), Ok(Some(v)) if v == 60.0));
        assert!(matches!(js_number_flag(" 5.5 "), Ok(Some(v)) if v == 5.0));
        assert!(matches!(js_number_flag("-3"), Ok(Some(v)) if v == -3.0));
        assert!(matches!(js_number_flag(".5"), Ok(None))); // parseInt NaN
        assert!(js_number_flag("0x10").is_err()); // outside modeled grammar
        assert!(js_number_flag("abc").is_err());
        assert!(js_number_flag("").is_err());
    }

    #[test]
    fn date_parse_subset() {
        assert!(matches!(
            js_date_parse("2020-01-02T03:04:05.678Z"),
            Ok(Some(v)) if v == 1577934245678.0
        ));
        let day = js_date_parse("2020-01-02").unwrap_or(None).unwrap();
        let full = js_date_parse("2020-01-02T00:00:00.000Z").unwrap_or(None).unwrap();
        assert_eq!(day, full); // date-only is UTC midnight in JS
        assert!(matches!(js_date_parse(""), Ok(None)));
        assert!(js_date_parse("July 4 2026").is_err()); // V8 legacy grammar — delegate
    }

    #[test]
    fn reserve_writes_node_shaped_lease_and_mirrored_hold() {
        if ambient_session() {
            return;
        }
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        let out = reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "cell-1", "src/x.ts"), 1);
        let Ok(Out::Emit(result, text, code)) = out else {
            panic!("expected an emit outcome");
        };
        assert_eq!(code, 0);
        assert!(text.starts_with("Reserved \"src/x.ts\" for worker-a (cell cell-1, ttl 3600s)."));
        assert_eq!(result["ok"], Value::Bool(true));
        assert_eq!(result["reservation"]["path"], "src/x.ts");
        assert_eq!(result["reservation"]["kind"], "lease");
        // Lease file exists at the sha256 of the resource key, Node-shaped.
        let file = path_lease_file(&root_s, "src/x.ts");
        let content = std::fs::read_to_string(&file).unwrap();
        assert!(content.ends_with("}\n"));
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["resource"], "path:src/x.ts");
        assert_eq!(parsed["mode"], "write");
        assert_eq!(parsed["workspace_id"], "agent:worker-a");
        assert_eq!(parsed["session_id"], SESSIONLESS_SESSION_ID);
        // Mirrored hold landed in the ledger.
        let ledger: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(tmp.path())).unwrap())
                .unwrap();
        assert_eq!(ledger["holds"][0]["holder"], "main");
        assert_eq!(ledger["holds"][0]["path"], "src/x.ts");
        assert_eq!(ledger["holds"][0]["released_at"], Value::Null);
    }

    #[test]
    fn reserve_exact_path_race_reports_conflict() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "cell-1", "src/x.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        let Ok(Out::Emit(result, text, code)) =
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-b", "cell-2", "src/x.ts"), 1)
        else {
            panic!("expected emit");
        };
        assert_eq!(code, 1);
        assert_eq!(result["ok"], Value::Bool(false));
        assert_eq!(result["conflicts"][0]["agent"], "worker-a");
        assert!(text.contains("Reservation CONFLICT"));
        assert!(text.contains("- worker-a holds \"src/x.ts\" (cell cell-1)"));
    }

    #[test]
    fn intent_kind_overlap_is_advisory_but_exact_path_still_hard() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        let mut intent = params("planner", "cell-p", "src/api/*");
        intent.kind = Some("intent".into());
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &intent, 1),
            Ok(Out::Emit(_, _, 0))
        ));
        // Broad overlap against the intent row: allowed.
        assert!(matches!(
            reserve_exec(
                main_topo(tmp.path()),
                &root_s,
                &params("worker-a", "cell-1", "src/api/router.ts"),
                1
            ),
            Ok(Out::Emit(_, _, 0))
        ));
        // Exact same resource: hard regardless of kind.
        let Ok(Out::Emit(result, _, 1)) =
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-b", "cell-2", "src/api/*"), 1)
        else {
            panic!("expected exact-path hard conflict");
        };
        assert_eq!(result["ok"], Value::Bool(false));
    }

    #[test]
    fn reserve_contends_on_the_shared_node_lock_name() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        let _held = lock::acquire_store_lock(tmp.path(), CROSS_WORKTREE_HOLDS_LOCK, 1)
            .ok()
            .unwrap();
        match reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "c", "src/x.ts"), 1) {
            Err(Err2::Msg(msg)) => {
                assert!(msg.contains("lock \"cross-worktree-holds\" busy: held by"), "{msg}");
            }
            _ => panic!("reserve under a held lock must refuse with the LockBusy message"),
        }
        // No lease was written while the lock was held.
        assert!(!path_lease_file(&root_s, "src/x.ts").exists());
    }

    #[test]
    fn invalid_kind_message_matches_node() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        let mut p = params("a", "c", "src/x.ts");
        p.kind = Some("true".into());
        match reserve_exec(main_topo(tmp.path()), &root_s, &p, 1) {
            Ok(Out::Thrown(msg)) => {
                assert_eq!(msg, "reserve: kind must be one of intent/lease (got \"true\").")
            }
            _ => panic!("expected thrown message"),
        }
    }

    #[test]
    fn release_deletes_lease_and_marks_mirrored_hold() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "cell-1", "src/x.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        let Ok(Out::Emit(result, text, 0)) = release_exec(main_topo(tmp.path()), &root_s, "worker-a", None, 1)
        else {
            panic!("expected release emit");
        };
        assert_eq!(result["released"], serde_json::json!(1.0));
        assert_eq!(result["holds_released"], serde_json::json!(1.0));
        assert_eq!(text, "Released 1 reservation(s) and 1 cross-worktree hold(s).");
        assert!(!path_lease_file(&root_s, "src/x.ts").exists());
        let ledger: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(tmp.path())).unwrap())
                .unwrap();
        assert!(ledger["holds"][0]["released_at"].is_string());
    }

    #[test]
    fn release_scoped_to_other_cell_releases_nothing() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "cell-1", "src/x.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        let Ok(Out::Emit(result, text, 0)) =
            release_exec(main_topo(tmp.path()), &root_s, "worker-a", Some("other-cell"), 1)
        else {
            panic!("expected release emit");
        };
        assert_eq!(result["released"], serde_json::json!(0.0));
        assert_eq!(text, "Released 0 reservation(s).");
        assert!(path_lease_file(&root_s, "src/x.ts").exists());
    }

    #[test]
    fn sweep_releases_expired_lease_and_hold() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        // A lease already past its expiry + a matching expired ledger row.
        let file = path_lease_file(&root_s, "src/old.ts");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(
            &file,
            "{\n  \"resource\": \"path:src/old.ts\",\n  \"mode\": \"write\",\n  \"workflow_id\": \"c\",\n  \"session_id\": \"s\",\n  \"workspace_id\": \"agent:a\",\n  \"epoch\": 0,\n  \"acquired_at\": \"2020-01-01T00:00:00.000Z\",\n  \"expires_at\": \"2020-01-01T01:00:00.000Z\",\n  \"kind\": \"lease\"\n}\n",
        )
        .unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee").join("runtime")).unwrap();
        std::fs::write(
            holds_ledger_path(tmp.path()),
            "{\n  \"holds\": [\n    {\n      \"path\": \"src/old.ts\",\n      \"holder\": \"main\",\n      \"feature\": null,\n      \"session\": null,\n      \"cell\": \"c\",\n      \"ttl_seconds\": 60,\n      \"mirrored_at\": \"2020-01-01T00:00:00.000Z\",\n      \"released_at\": null\n    }\n  ]\n}\n",
        )
        .unwrap();
        let Ok(Out::Emit(result, text, 0)) = sweep_exec(tmp.path(), &root_s, 1) else {
            panic!("expected sweep emit");
        };
        assert_eq!(result["released"], serde_json::json!(1.0));
        assert_eq!(result["holds_released"], serde_json::json!(1.0));
        assert_eq!(
            text,
            "Swept 1 expired reservation(s) and 1 expired cross-worktree hold(s)."
        );
        assert!(!file.exists());
    }

    #[test]
    /// CUTOVER (was `corrupt_ledger_or_null_hold_delegates`): a CORRUPT
    /// ledger no longer delegates — it warns and reads as the empty ledger,
    /// `readJson(file, null)`'s own fallback through Node's `!store` guard.
    /// A `null` HOLD still delegates: that one is a JS property-access crash,
    /// not a V8-message matter, and is out of the cutover's scope.
    fn corrupt_ledger_reads_empty_and_a_null_hold_still_delegates() {
        let tmp = fixture_root();
        std::fs::create_dir_all(tmp.path().join(".bee").join("runtime")).unwrap();
        std::fs::write(holds_ledger_path(tmp.path()), "{broken").unwrap();
        assert_eq!(read_holds_store(tmp.path()).ok().unwrap(), json!({"holds": []}));
        std::fs::write(holds_ledger_path(tmp.path()), "{\"holds\": [null]}").unwrap();
        assert!(read_holds_store(tmp.path()).is_err());
        // Shape-less parses read as an empty ledger, like Node's readStore —
        // which is exactly what the corrupt file now reads as, too.
        std::fs::write(holds_ledger_path(tmp.path()), "[1,2]").unwrap();
        let store = read_holds_store(tmp.path()).ok().unwrap();
        assert_eq!(store, json!({"holds": []}));
    }

    #[test]
    fn session_required_when_others_live_and_caller_unidentified() {
        if ambient_session() {
            return;
        }
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        let sessions = tmp.path().join(".bee").join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let hb = now_iso();
        // TWO live sessions: adoption (exactly-one rule) cannot fire, and
        // concurrent mode is on → typed SESSION_REQUIRED refusal.
        for id in ["other", "second"] {
            std::fs::write(
                sessions.join(format!("{id}.json")),
                format!(
                    "{{\n  \"id\": \"{id}\",\n  \"started_at\": \"{hb}\",\n  \"last_heartbeat\": \"{hb}\"\n}}\n"
                ),
            )
            .unwrap();
        }
        let Ok(Out::Emit(result, _, 1)) =
            reserve_exec(main_topo(tmp.path()), &root_s, &params("a", "c", "src/x.ts"), 1)
        else {
            panic!("expected SESSION_REQUIRED refusal");
        };
        assert_eq!(result["code"], "SESSION_REQUIRED");
        assert_eq!(result["conflicts"], json!([]));
    }

    #[test]
    fn single_live_session_is_adopted_and_stamped() {
        if ambient_session() {
            return;
        }
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        let sessions = tmp.path().join(".bee").join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let hb = now_iso();
        std::fs::write(
            sessions.join("solo.json"),
            format!(
                "{{\n  \"id\": \"solo\",\n  \"started_at\": \"{hb}\",\n  \"last_heartbeat\": \"{hb}\"\n}}\n"
            ),
        )
        .unwrap();
        let Ok(Out::Emit(result, _, 0)) =
            reserve_exec(main_topo(tmp.path()), &root_s, &params("a", "c", "src/x.ts"), 1)
        else {
            panic!("expected adopted-session reserve");
        };
        assert_eq!(result["reservation"]["session"], "solo");
    }

    #[test]
    fn parse_flags_mirrors_node() {
        let (flags, json) = parse_flags(&["--agent", "a", "--json", "--cell=c1"]).unwrap();
        assert!(json);
        assert_eq!(flags.req_str("agent"), Some("a"));
        assert_eq!(flags.req_str("cell"), Some("c1"));
        // Value flags consume the next token even when it looks like a flag.
        let (flags, json) = parse_flags(&["--agent", "--json"]).unwrap();
        assert!(!json);
        assert_eq!(flags.req_str("agent"), Some("--json"));
        // Trailing value-less flag → Node's own error → delegate.
        assert!(parse_flags(&["--agent"]).is_none());
        // Non-flag token → delegate.
        assert!(parse_flags(&["stray"]).is_none());
    }

    #[test]
    fn uuid_shape() {
        let a = pseudo_uuid_v4();
        let b = pseudo_uuid_v4();
        assert_ne!(a, b);
        assert_eq!(a.len(), 36);
        assert_eq!(&a[14..15], "4");
        assert!(matches!(&a[19..20], "8" | "9" | "a" | "b"));
    }

    // ── hold topology over REAL `git worktree add` fixtures ────────────────
    //
    // Pinned against Node on the same fixture shape first (twin-fixture
    // byte-diff of reserve/list/release/sweep from each checkout with
    // BEE_JS_ENTRY sabotaged, plus a diff of the resulting `.bee` trees).

    fn git(cwd: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git must be on PATH for the worktree fixtures");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// main + `wt-granted` (registered) + `wt-ungranted` (not).
    fn worktree_fixture(tmp: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let main = tmp.join("main");
        std::fs::create_dir_all(main.join(".bee")).unwrap();
        std::fs::write(main.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        std::fs::write(main.join("f.txt"), "x").unwrap();
        git(&main, &["init", "-q", "-b", "main", "."]);
        git(&main, &["config", "user.email", "a@b.c"]);
        git(&main, &["config", "user.name", "t"]);
        git(&main, &["add", "-A"]);
        git(&main, &["commit", "-qm", "init"]);
        let granted = tmp.join("wt-granted");
        let ungranted = tmp.join("wt-ungranted");
        git(&main, &["worktree", "add", "-q", granted.to_str().unwrap(), "-b", "wt/g"]);
        git(&main, &["worktree", "add", "-q", ungranted.to_str().unwrap(), "-b", "wt/u"]);
        std::fs::create_dir_all(main.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            main.join(".bee").join("runtime").join("worktree-grants.json"),
            "{\"wt-granted\": true}\n",
        )
        .unwrap();
        std::fs::create_dir_all(granted.join(".bee")).unwrap();
        std::fs::write(granted.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        (main, granted, ungranted)
    }

    fn roots_at(cwd: &Path) -> StoreRoots {
        match resolve_store_root_worktree(cwd) {
            RootsWt::Go(r) => r,
            _ => panic!("expected a resolvable root at {}", cwd.display()),
        }
    }

    fn nrm(p: &Path) -> String {
        p.to_string_lossy().replace('/', "\\")
    }

    /// bee.mjs resolveMainRoot + resolveHoldTopology, all three topologies.
    #[test]
    fn hold_topology_matches_node_for_every_checkout_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, ungranted) = worktree_fixture(tmp.path());

        // ORDINARY: {mainRoot: workRoot, holder: 'main'}.
        let r = roots_at(&main);
        assert_eq!(nrm(&r.main_root()), nrm(&main));
        let (m, h) = r.hold_topology().expect("ordinary always has a topology");
        assert_eq!(nrm(&m), nrm(&main));
        assert_eq!(h, "main");

        // GRANTED worktree: ledger at MAIN, holder = the git-verified id.
        let r = roots_at(&granted);
        assert_eq!(nrm(&r.root), nrm(&granted)); // its OWN store
        assert_eq!(nrm(&r.main_root()), nrm(&main));
        let (m, h) = r.hold_topology().expect("a granted worktree holds");
        assert_eq!(nrm(&m), nrm(&main));
        assert_eq!(h, "wt-granted");

        // UNGRANTED worktree: root already IS main's store, and the whole
        // cross-worktree wiring is SKIPPED (topology === null).
        let r = roots_at(&ungranted);
        assert_eq!(nrm(&r.root), nrm(&main));
        assert_eq!(nrm(&r.main_root()), nrm(&main)); // sweep still prunes main
        assert!(r.hold_topology().is_none());
    }

    /// A reserve from inside a GRANTED worktree mirrors into MAIN's ledger
    /// under the worktree's own id — and the reciprocal reserve from main is
    /// then refused with FOREIGN_HOLD.
    #[test]
    fn granted_worktree_mirrors_under_its_id_and_blocks_main() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, _ungranted) = worktree_fixture(tmp.path());
        let g = roots_at(&granted);
        let (gm, gh) = g.hold_topology().unwrap();
        let g_topo = Some(Topo { main_root: &gm, holder: &gh });
        let g_root_s = root_str(&g.root);

        assert!(matches!(
            reserve_exec(g_topo, &g_root_s, &params("wt-agent", "w1", "src/shared.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        // The mirror row lives in MAIN's ledger, holder = the worktree id.
        let ledger: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(&main)).unwrap())
                .unwrap();
        assert_eq!(ledger["holds"].as_array().unwrap().len(), 1);
        assert_eq!(ledger["holds"][0]["holder"], "wt-granted");
        assert_eq!(ledger["holds"][0]["path"], "src/shared.ts");
        // The worktree's OWN store never gets a ledger.
        assert!(!holds_ledger_path(&granted).exists());

        // MAIN now hits the foreign hold on the same path.
        let m = roots_at(&main);
        let (mm, mh) = m.hold_topology().unwrap();
        let m_topo = Some(Topo { main_root: &mm, holder: &mh });
        let m_root_s = root_str(&m.root);
        let Ok(Out::Emit(result, text, 1)) =
            reserve_exec(m_topo, &m_root_s, &params("main-agent", "c1", "src/shared.ts"), 1)
        else {
            panic!("expected a FOREIGN_HOLD refusal from main");
        };
        assert_eq!(result["code"], "FOREIGN_HOLD");
        assert_eq!(result["holder"], "wt-granted");
        assert!(text.starts_with("bee cross-worktree hold: \"src/shared.ts\" is held by checkout \"wt-granted\""));

        // Releasing from the worktree clears exactly its own mirrored rows.
        let Ok(Out::Emit(result, _, 0)) = release_exec(g_topo, &g_root_s, "wt-agent", None, 1)
        else {
            panic!("expected a clean release");
        };
        assert_eq!(result["holds_released"], 1.0);
    }

    /// An UNGRANTED worktree has NO topology: reserve takes no cross-worktree
    /// lock, writes no mirror row, and release reports zero holds — while the
    /// LEASE itself still lands in main's shared store (controlRootFor).
    #[test]
    fn ungranted_worktree_skips_the_cross_worktree_section_entirely() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, _granted, ungranted) = worktree_fixture(tmp.path());
        let r = roots_at(&ungranted);
        assert!(r.hold_topology().is_none());
        let root_s = root_str(&r.root);

        assert!(matches!(
            reserve_exec(None, &root_s, &params("u-agent", "c1", "src/only.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        // No ledger written anywhere — not in main, not in the worktree.
        assert!(!holds_ledger_path(&main).exists());
        assert!(!holds_ledger_path(&ungranted).exists());
        // But the lease DID land, in main's control root (shared store).
        let Ok(rows) = list_reservations(&root_s, true, now_ms()) else {
            panic!("listable");
        };
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].path, "src/only.ts");
        let Ok(ctrl) = control_root_for(&root_s) else { panic!("control root") };
        assert_eq!(nrm(Path::new(&ctrl)), nrm(&main));

        let Ok(Out::Emit(result, text, 0)) = release_exec(None, &root_s, "u-agent", None, 1) else {
            panic!("expected a clean release");
        };
        assert_eq!(result["released"], 1.0);
        assert_eq!(result["holds_released"], 0.0);
        assert_eq!(text, "Released 1 reservation(s).");
    }

    /// sweep uses resolveMainRoot, NOT the topology: even from an ungranted
    /// worktree it prunes MAIN's ledger, exactly as running it from main does.
    #[test]
    fn sweep_prunes_mains_ledger_from_an_ungranted_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, _granted, ungranted) = worktree_fixture(tmp.path());
        std::fs::create_dir_all(holds_ledger_path(&main).parent().unwrap()).unwrap();
        std::fs::write(
            holds_ledger_path(&main),
            r#"{"holds":[{"path":"src/old.ts","holder":"someone-else","feature":null,"session":null,"cell":"c","ttl_seconds":60,"mirrored_at":"2020-01-01T00:00:00.000Z","released_at":null}]}"#,
        )
        .unwrap();
        let r = roots_at(&ungranted);
        let Ok(Out::Emit(result, text, 0)) =
            sweep_exec(&r.main_root(), &root_str(&r.root), 1)
        else {
            panic!("expected a clean sweep");
        };
        assert_eq!(result["holds_released"], 1.0);
        assert_eq!(
            text,
            "Swept 0 expired reservation(s) and 1 expired cross-worktree hold(s)."
        );
        let ledger: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(&main)).unwrap())
                .unwrap();
        assert!(ledger["holds"][0]["released_at"].is_string());
    }

    // ── R5 test migration from packages/bee/tests/test_lease_store.mjs ─────
    //
    // WHAT IS TESTED HERE, AND WHERE THE REST LIVES — the lease store THIS
    // file ports is the SINGLE-resource path-lease half that reservations.mjs
    // drives (acquire one lease under O_EXCL, release by agent/cell, sweep by
    // expiry), so the rows below are the single-resource echoes.
    //
    // The rest of lease-store.mjs — the multi-resource batch (hash-sorted
    // acquire order, partial rollback, typed LEASE_HELD /
    // LEASE_INVALID_REQUEST), renewLease / renewLeasesBySession with the
    // LEASE_MISSING refusal, and LEASE_FENCE_STALE on BOTH renew and release
    // including the "file is never removed on a fenced refusal" property — now
    // lives in `src/lease_store.rs` with its own tests. `reserve_locked` here
    // is deliberately NOT rewired through it: this arm is byte-diffed against
    // Node through four live verbs, and rewiring it would be a behavior risk
    // with no behavior gain (see that module's header).

    /// Reads the whole path-lease directory as a set of file names, so a test
    /// can assert nothing was added or left behind.
    fn lease_files(root_s: &str) -> Vec<String> {
        let dir = Path::new(root_s)
            .join(".bee")
            .join("runtime")
            .join("leases")
            .join("paths");
        let mut names: Vec<String> = match std::fs::read_dir(&dir) {
            Ok(rd) => rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect(),
            Err(_) => Vec::new(),
        };
        names.sort();
        names
    }

    fn write_lease_file(root_s: &str, path: &str, expires_at: Value) {
        let file = path_lease_file(root_s, path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        let record = json!({
            "resource": format!("path:{}", res_normalize_path(path)),
            "mode": "write",
            "workflow_id": "c",
            "session_id": "s",
            "workspace_id": "agent:a",
            "epoch": 0,
            "acquired_at": "2020-01-01T00:00:00.000Z",
            "expires_at": expires_at,
            "kind": "lease",
        });
        std::fs::write(&file, format!("{}\n", jsjson::stringify_pretty(&record))).unwrap();
    }

    fn write_holds_ledger(root: &Path, holds: Value) {
        std::fs::create_dir_all(holds_ledger_path(root).parent().unwrap()).unwrap();
        std::fs::write(holds_ledger_path(root), jsjson::stringify(&json!({"holds": holds}))).unwrap();
    }

    /// Oracle: "sweepExpiredLeases deletes only expired leases; never-expiring
    /// (ttl<=0) leases are never swept (TTL semantics)".
    ///
    /// computeExpiresAt stores `expires_at: null` for a non-positive ttl
    /// (reserve_locked's own non-positive-ttl branch), and both the sweep and
    /// the active-only listing must read that as "never expires".
    #[test]
    fn never_expiring_leases_and_holds_survive_a_sweep_that_takes_the_expired_ones() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        write_lease_file(&root_s, "src/expired.ts", json!("2020-01-01T01:00:00.000Z"));
        write_lease_file(&root_s, "src/fresh.ts", json!("2999-01-01T00:00:00.000Z"));
        write_lease_file(&root_s, "src/forever.ts", Value::Null);
        assert_eq!(lease_files(&root_s).len(), 3);

        // ttl_seconds 0 and -5 are both "never expires"; 60s from 2020 is long
        // expired — the control that the sweep is doing anything at all.
        write_holds_ledger(
            tmp.path(),
            json!([
                {"path": "src/expired.ts", "holder": "main", "feature": null, "session": null,
                 "cell": "c", "ttl_seconds": 60, "mirrored_at": "2020-01-01T00:00:00.000Z", "released_at": null},
                {"path": "src/forever.ts", "holder": "main", "feature": null, "session": null,
                 "cell": "c", "ttl_seconds": 0, "mirrored_at": "2020-01-01T00:00:00.000Z", "released_at": null},
                {"path": "src/forever-neg.ts", "holder": "main", "feature": null, "session": null,
                 "cell": "c", "ttl_seconds": -5, "mirrored_at": "2020-01-01T00:00:00.000Z", "released_at": null},
            ]),
        );

        let Ok(Out::Emit(result, text, 0)) = sweep_exec(tmp.path(), &root_s, 1) else {
            panic!("expected sweep emit");
        };
        assert_eq!(result["released"], json!(1.0), "exactly the expired lease");
        assert_eq!(result["holds_released"], json!(1.0), "exactly the expired hold");
        assert_eq!(
            text,
            "Swept 1 expired reservation(s) and 1 expired cross-worktree hold(s)."
        );
        assert!(!path_lease_file(&root_s, "src/expired.ts").exists());
        assert!(path_lease_file(&root_s, "src/fresh.ts").exists(), "an unexpired lease survives");
        assert!(
            path_lease_file(&root_s, "src/forever.ts").exists(),
            "a never-expiring (expires_at null) lease survives"
        );
        let ledger: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(tmp.path())).unwrap()).unwrap();
        assert!(ledger["holds"][0]["released_at"].is_string(), "the expired hold was marked");
        assert_eq!(ledger["holds"][1]["released_at"], Value::Null, "ttl 0 never expires");
        assert_eq!(ledger["holds"][2]["released_at"], Value::Null, "a negative ttl never expires");

        // The active-only listing agrees, and reports the never-expiring lease
        // with reservations.mjs's ttl_seconds sentinel of 0.
        let mut rows = list_reservations(&root_s, true, now_ms()).ok().unwrap();
        rows.sort_by(|a, b| a.path.cmp(&b.path));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].path, "src/forever.ts");
        assert_eq!(rows[0].ttl_seconds, Some(0.0));
        assert_eq!(rows[1].path, "src/fresh.ts");

        // A second sweep is a no-op: nothing left is expired.
        let Ok(Out::Emit(result, _, 0)) = sweep_exec(tmp.path(), &root_s, 1) else {
            panic!("expected sweep emit");
        };
        assert_eq!(result["released"], json!(0.0));
        assert_eq!(result["holds_released"], json!(0.0));
    }

    /// The single-resource echo of the oracle's "partial acquire rolls back
    /// fully … zero residue": a reserve that loses its resource must leave the
    /// lease directory and the mirrored ledger exactly as it found them.
    #[test]
    fn a_lost_reserve_leaves_zero_residue_in_the_lease_dir_and_the_ledger() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "cell-1", "src/x.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        let files_before = lease_files(&root_s);
        // Semantic snapshot: the ledger must be unchanged as a RECORD SET, not
        // merely as bytes a rewrite happened to reproduce.
        let ledger_before: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(tmp.path())).unwrap()).unwrap();
        assert_eq!(files_before.len(), 1);
        assert_eq!(ledger_before["holds"].as_array().unwrap().len(), 1);

        // Overlap conflict on the exact same resource.
        let Ok(Out::Emit(result, _, 1)) =
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-b", "cell-2", "src/x.ts"), 1)
        else {
            panic!("expected the conflict refusal");
        };
        assert_eq!(result["ok"], Value::Bool(false));
        // Overlap conflict on a path that only OVERLAPS an existing lease.
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-b", "cell-2", "src/x.ts/inner"), 1),
            Ok(Out::Emit(_, _, 1))
        ));
        assert_eq!(lease_files(&root_s), files_before, "no lease file survived a refusal");
        let ledger_after: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(tmp.path())).unwrap()).unwrap();
        assert_eq!(ledger_after, ledger_before, "no hold row survived a refusal");

        // Control: a non-overlapping resource DOES add exactly one of each, so
        // the zero-residue assertions above are not vacuous.
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-b", "cell-2", "src/y.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        assert_eq!(lease_files(&root_s).len(), 2);
        let ledger_final: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(tmp.path())).unwrap()).unwrap();
        assert_eq!(ledger_final["holds"].as_array().unwrap().len(), 2);
    }

    /// The single-resource echo of the oracle's LEASE_INVALID_REQUEST row
    /// ("missing fields, bad type … refuses before any file is created") and
    /// of "kind defaults to lease when omitted; an explicit intent is stamped
    /// verbatim; an invalid kind refuses before any file is created".
    #[test]
    fn a_malformed_reserve_request_is_refused_before_any_lease_file_is_created() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());

        // Refusal wording is a pinned contract on this verb (bee.mjs serves
        // reserve()'s throws byte-identical) — hence the exact comparisons.
        let cases: [(ReserveParams, &str); 4] = [
            (params("  ", "cell-1", "src/x.ts"), "reserve: agent is required."),
            (params("worker-a", "", "src/x.ts"), "reserve: cell id is required."),
            (params("worker-a", "cell-1", "   "), "reserve: path is required."),
            (
                {
                    let mut p = params("worker-a", "cell-1", "src/x.ts");
                    p.kind = Some("exclusive".into());
                    p
                },
                "reserve: kind must be one of intent/lease (got \"exclusive\").",
            ),
        ];
        for (p, expected) in cases {
            match reserve_exec(main_topo(tmp.path()), &root_s, &p, 1) {
                Ok(Out::Thrown(msg)) => assert_eq!(msg, expected),
                other => panic!(
                    "expected a thrown refusal for {expected:?}, got {}",
                    match other {
                        Ok(Out::Emit(v, _, _)) => jsjson::stringify(&v),
                        Ok(Out::Thrown(m)) => m,
                        Err(_) => "an error".to_string(),
                    }
                ),
            }
            assert!(lease_files(&root_s).is_empty(), "a refused request created a lease file");
        }
        assert!(
            !holds_ledger_path(tmp.path()).exists(),
            "a refused request never writes the mirrored ledger"
        );

        // Control: the same request with every field valid succeeds, and the
        // stored record carries the DEFAULTED kind.
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "cell-1", "src/x.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        let stored: Value = serde_json::from_str(
            &std::fs::read_to_string(path_lease_file(&root_s, "src/x.ts")).unwrap(),
        )
        .unwrap();
        assert_eq!(stored["kind"], json!("lease"), "an omitted kind defaults to lease");
        // …and an explicit kind is stamped verbatim.
        let mut intent = params("planner", "cell-p", "src/api/*");
        intent.kind = Some("intent".into());
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &intent, 1),
            Ok(Out::Emit(_, _, 0))
        ));
        let stored: Value = serde_json::from_str(
            &std::fs::read_to_string(path_lease_file(&root_s, "src/api/*")).unwrap(),
        )
        .unwrap();
        assert_eq!(stored["kind"], json!("intent"));
    }

    // ── CUTOVER: corrupt-JSON reads that used to delegate ─────────────────

    /// worktree-holds.mjs readStore: a corrupt ledger reads as the EMPTY
    /// ledger, which is `readJson(file, null)`'s own fallback through Node's
    /// `!store` guard — same value, one warning of explanation.
    #[test]
    fn a_corrupt_holds_ledger_reads_as_an_empty_ledger() {
        let tmp = fixture_root();
        let ledger = holds_ledger_path(tmp.path());
        std::fs::create_dir_all(ledger.parent().unwrap()).unwrap();
        std::fs::write(&ledger, "{broken").unwrap();
        assert_eq!(read_holds_store(tmp.path()).ok().unwrap(), json!({"holds": []}));
        // A missing file answers the same thing, which is the point.
        std::fs::remove_file(&ledger).unwrap();
        assert_eq!(read_holds_store(tmp.path()).ok().unwrap(), json!({"holds": []}));
    }

    /// claims.mjs listSessionRecords: a corrupt session record is skipped and
    /// the scan continues — the readable siblings still list.
    #[test]
    fn a_corrupt_session_record_is_skipped_not_delegated() {
        let tmp = fixture_root();
        let dir = tmp.path().join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("good.json"), r#"{"id":"good"}"#).unwrap();
        std::fs::write(dir.join("bad.json"), "{broken").unwrap();
        let records = list_session_records(tmp.path().to_str().unwrap()).ok().unwrap();
        assert_eq!(records.len(), 1, "the corrupt record is skipped");
        assert_eq!(records[0].get("id"), Some(&json!("good")));
    }

    /// The `|n| >= 1e21` delegate class is retired: js_numberify accepts every
    /// finite number now that jsjson prints the full ECMA Number::toString.
    #[test]
    fn large_numbers_no_longer_delegate() {
        let v: Value = serde_json::from_str("{\"n\":1e21}").unwrap();
        assert!(js_numberify(&v).is_ok());
    }
