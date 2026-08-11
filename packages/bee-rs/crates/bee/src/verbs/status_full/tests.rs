// Split out of the single 7k-line verbs/status_full.rs. Code unchanged; only module placement and item visibility moved.
//
// Moved verbatim out of the parent file's `#[cfg(test)] mod tests`,
// indentation and all: the fixtures are raw strings whose leading
// whitespace is content.

// The parent module's own `use` block travels with the tests: they reach
// for names mod.rs no longer imports now that the code using them lives
// in sibling modules.
#![allow(unused_imports)]

use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, LinkedRoots, RootsWt};
use crate::state::{bypass_level, read_config_raw};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Value};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use crate::version::BEE_VERSION;
    use super::*;

    /// An ORDINARY-checkout context (`linked: None`) — the shape every
    /// pre-existing fixture below has always had.
    fn ctx_for(root: &Path) -> Ctx {
        Ctx {
            root: root.to_path_buf(),
            cwd: root.to_path_buf(),
            linked: None,
            stderr: RefCell::new(Vec::new()),
        }
    }

    fn write(root: &Path, rel: &str, content: &str) {
        let file = root.join(rel);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }

    fn sha256_str(s: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(s.as_bytes());
        format!("{:x}", h.finalize())
    }

    // ── CUTOVER: corrupt JSON on the snapshot path ─────────────────────────

    /// `rj` used to bail the whole snapshot to Node on any corrupt file. It
    /// now warns (buffered) and returns readJson's `null` fallback, so the
    /// status still builds — with defaults where the file was unreadable,
    /// exactly the shape Node produced from that fallback.
    #[test]
    fn a_corrupt_state_file_warns_and_falls_back_to_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(root, ".bee/state.json", "{broken");
        let ctx = ctx_for(root);
        let state = read_state_full(&ctx).unwrap();
        assert_eq!(state.get("phase"), Some(&json!("idle")), "defaultState()");
        assert_eq!(state.get("feature"), Some(&Value::Null));
        let warns = ctx.stderr.borrow();
        assert_eq!(warns.len(), 1, "exactly one warning per read: {warns:?}");
        assert!(warns[0].starts_with("bee: could not parse JSON at "), "{warns:?}");
        assert!(warns[0].ends_with("Using fallback; fix the file."), "{warns:?}");
        assert!(!warns[0].contains("Unexpected token"), "no V8 text: {warns:?}");
    }

    /// The whole snapshot survives a corrupt config/handoff/onboarding, and
    /// `--json` still renders. Previously any one of these returned None.
    #[test]
    fn build_status_survives_every_corrupt_file_on_the_read_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(root, ".bee/state.json", r#"{"phase":"idle"}"#);
        write(root, ".bee/config.json", "{broken");
        write(root, ".bee/HANDOFF.json", "{broken");
        let mut ctx = ctx_for(root);
        let status = build_status(&mut ctx, false).expect("the snapshot must still build");
        assert_eq!(status.get("phase"), Some(&json!("idle")));
        assert_eq!(status.get("handoff"), Some(&Value::Null), "readJson's null fallback");
        assert_eq!(status.get("gate_bypass_level"), Some(&json!("off")), "no config -> off");
        assert!(
            ctx.stderr.borrow().iter().any(|l| l.starts_with("bee: could not parse JSON at ")),
            "the corrupt reads are reported: {:?}",
            ctx.stderr.borrow()
        );
    }

    /// readLane's two lines, in Node's order: readJson's own warning first
    /// (its null fallback is what makes laneRecordFrom answer null), then
    /// readLane's skipping-corrupt-lane-record line.
    #[test]
    fn a_corrupt_lane_record_emits_both_of_nodes_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/lanes/feat-x.json", "{broken");
        let mut ctx = ctx_for(root);
        assert_eq!(read_lane(&mut ctx, "feat-x").unwrap(), None);
        let warns = ctx.stderr.borrow();
        assert_eq!(warns.len(), 2, "{warns:?}");
        assert!(warns[0].starts_with("bee: could not parse JSON at "), "{warns:?}");
        assert!(
            warns[1].starts_with("readLane: skipping corrupt lane record "),
            "{warns:?}"
        );
    }

    #[test]
    fn runtime_drift_detects_content_missing_and_extra() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/bin/lib/a.mjs", "aaa");
        write(root, ".bee/bin/lib/extra.mjs", "zzz");
        write(root, ".bee/bin/helper.mjs", "hhh");
        let onboarding = json!({
            "bee_version": BEE_VERSION,
            "managed": {
                "lib": { "a.mjs": sha256_str("aaa"), "gone.mjs": "deadbeef" },
                "helpers": { "helper.mjs": "not-the-hash" }
            }
        });
        let ctx = ctx_for(root);
        let (drift, detail) = compute_runtime_drift(&ctx, &onboarding);
        assert!(drift);
        assert_eq!(
            detail,
            vec![
                ".bee/bin/lib/gone.mjs (missing)".to_string(),
                ".bee/bin/helper.mjs".to_string(),
                ".bee/bin/lib/extra.mjs (extra)".to_string(),
            ]
        );
        // Clean ledger: no drift.
        let clean = json!({
            "bee_version": BEE_VERSION,
            "managed": {
                "lib": { "a.mjs": sha256_str("aaa"), "extra.mjs": sha256_str("zzz") },
                "helpers": { "helper.mjs": sha256_str("hhh") }
            }
        });
        let (drift, detail) = compute_runtime_drift(&ctx, &clean);
        assert!(!drift);
        assert!(detail.is_empty());
        // Version-only drift with a legacy (no managed map) ledger.
        let legacy = json!({ "bee_version": "0.0.1" });
        let (drift, detail) = compute_runtime_drift(&ctx, &legacy);
        assert!(drift);
        assert!(detail.is_empty());
    }

    #[test]
    fn workers_derive_from_heartbeat_joined_claims() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let fresh = to_iso(now_ms());
        let stale = "2020-01-01T00:00:00.000Z";
        write(
            root,
            ".bee/sessions/live-1.json",
            &format!(r#"{{"id":"live-1","started_at":"{fresh}","last_heartbeat":"{fresh}","lane":"feat-x"}}"#),
        );
        write(
            root,
            ".bee/sessions/dead-1.json",
            &format!(r#"{{"id":"dead-1","started_at":"{stale}","last_heartbeat":"{stale}"}}"#),
        );
        write(
            root,
            ".bee/claims/cell-7.json",
            &format!(r#"{{"cell":"cell-7","session":"live-1","claimed_at":"{fresh}","ttl_seconds":3600}}"#),
        );
        // An expired claim for the same session must not win.
        write(
            root,
            ".bee/claims/cell-1.json",
            r#"{"cell":"cell-1","session":"live-1","claimed_at":"2020-01-01T00:00:00.000Z","ttl_seconds":1}"#,
        );
        let rows = active_workers(&ctx_for(root), root, None).ok().unwrap();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.get("session_id"), Some(&json!("live-1")));
        assert_eq!(row.get("lane"), Some(&json!("feat-x")));
        assert_eq!(row.get("cell"), Some(&json!("cell-7")));
        // Excluding the live session leaves zero rows.
        assert!(active_workers(&ctx_for(root), root, Some("live-1")).ok().unwrap().is_empty());
    }

    #[test]
    fn lanes_summary_vs_full() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            ".bee/lanes/alpha.json",
            r#"{"feature":"alpha","phase":"swarming","approved_gates":{"context":true}}"#,
        );
        write(
            root,
            ".bee/lanes/beta.json",
            r#"{"feature":"beta","phase":"idle"}"#,
        );
        let mut ctx = ctx_for(root);
        let rows = build_lane_rows(&mut ctx).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("feature"), Some(&json!("alpha")));
        assert_eq!(rows[0].get("bound_sessions"), Some(&json!([])));
        // Full row render.
        let row_text = format_lane_row(&Value::Object(rows[0].clone()));
        assert_eq!(
            row_text,
            "alpha [swarming] context=approved shape=pending execution=pending review=pending"
        );
        // Summary: no live session -> active null, counts + ids over all.
        let summary = build_lane_summary(&mut ctx).unwrap();
        assert_eq!(summary.get("active"), Some(&Value::Null));
        assert_eq!(summary.get("counts"), Some(&json!({"swarming": 1, "idle": 1})));
        assert_eq!(summary.get("ids"), Some(&json!(["alpha", "beta"])));
        let line = format_lane_summary_line(&Value::Object(summary)).unwrap();
        assert_eq!(line, "Lanes: 2 other lane(s) [swarming=1 idle=1] (ids: alpha, beta)");
    }

    #[test]
    fn staleness_warnings_fire_in_node_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // No commands recorded; stale advisor key; version mismatch; stale
        // handoff; unknown phase.
        write(root, ".bee/onboarding.json", r#"{"bee_version":"0.9.0"}"#);
        write(root, ".bee/config.json", r#"{"advisor":"x"}"#);
        write(
            root,
            ".bee/HANDOFF.json",
            r#"{"written_at":"2020-01-01T00:00:00.000Z","kind":"pause"}"#,
        );
        write(root, ".bee/state.json", r#"{"phase":"vibing"}"#);
        let mut ctx = ctx_for(root);
        let status = build_status(&mut ctx, false).unwrap();
        let warnings: Vec<String> = status
            .get("staleness_warnings")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .map(|w| w.as_str().unwrap().to_string())
            .collect();
        assert!(warnings[0].starts_with("No standard commands recorded"));
        assert_eq!(
            warnings[1],
            format!("Onboarding installed bee 0.9.0 but plugin is {BEE_VERSION} — re-run onboarding.")
        );
        assert_eq!(
            warnings[2],
            "HANDOFF.json is older than 7 days (written 2020-01-01T00:00:00.000Z)."
        );
        assert_eq!(warnings[3], STALE_ADVISOR_KEY_WARNING);
        assert!(warnings[4].starts_with("Unknown phase \"vibing\""));
        assert_eq!(warnings.len(), 5);
        // HANDOFF present wins the recommendation.
        assert_eq!(
            status.get("recommended_next"),
            Some(&json!("HANDOFF present — present it to the user and WAIT. Never auto-resume."))
        );
    }

    #[test]
    fn orient_recommended_next_selection_and_packet() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(root, ".bee/config.json", r#"{"commands":{"test":"npm t"}}"#);
        write(
            root,
            ".bee/state.json",
            r#"{"phase":"swarming","feature":"f1","mode":"standard","approved_gates":{"context":true,"shape":true,"execution":true}}"#,
        );
        write(
            root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f1","status":"open","lane":"standard","title":"t"}"#,
        );
        std::fs::create_dir_all(root.join("docs").join("history").join("f1")).unwrap();
        write(root, "docs/history/f1/CONTEXT.md", "# ctx");
        let mut ctx = ctx_for(root);
        let packet = build_orient(&mut ctx).unwrap();
        // exec approved + one ready cell -> ready recommendation + command.
        let next = packet.get("next").unwrap();
        assert_eq!(
            vget(next, "action"),
            Some(&json!("1 ready cell(s): c-1 — orchestrator assigns them."))
        );
        assert_eq!(vget(next, "skill"), Some(&json!("bee-swarming")));
        assert_eq!(vget(next, "command"), Some(&json!("bee cells ready --json")));
        let decisions = packet.get("decisions").unwrap();
        assert_eq!(vget(decisions, "context_md"), Some(&json!("docs/history/f1/CONTEXT.md")));
        assert_eq!(vget(decisions, "active_count"), Some(&json!(0)));
        let work = packet.get("work").unwrap();
        assert_eq!(vget(work, "ready"), Some(&json!(["c-1"])));
        assert_eq!(vget(work, "blockers"), Some(&json!([])));
        // Text renderer.
        let text = render_orient_text(&packet);
        let lines: Vec<&str> = text.split('\n').collect();
        assert_eq!(
            lines[0],
            "where: phase=swarming feature=f1 mode=standard gates=true/true/true/false bypass=off"
        );
        assert_eq!(lines[1], "decisions: 0 active | context: docs/history/f1/CONTEXT.md");
        assert_eq!(lines[2], "work: open=1 claimed=0 capped=0 | ready: c-1");
        assert_eq!(lines[3], "skill: bee-swarming");
        assert_eq!(lines[4], "next: 1 ready cell(s): c-1 — orchestrator assigns them.");
    }

    #[test]
    fn status_text_renderer_minimal_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(root, ".bee/config.json", r#"{"commands":{"test":"npm t"},"gate_bypass":true}"#);
        write(root, ".bee/state.json", r#"{"phase":"idle"}"#);
        let mut ctx = ctx_for(root);
        let status = build_status(&mut ctx, false).unwrap();
        let text = render_status_text(&status);
        let lines: Vec<&str> = text.split('\n').collect();
        assert_eq!(lines[0], format!("bee status (plugin v{BEE_VERSION})"));
        assert_eq!(lines[1], format!("Onboarding: installed (bee {BEE_VERSION})"));
        assert_eq!(lines[2], "Phase: idle | Mode: none | Feature: none");
        assert_eq!(
            lines[3],
            "Gates: context=pending shape=pending execution=pending review=pending"
        );
        assert_eq!(lines[4], bypass_banner("normal"));
        assert_eq!(lines[5], "Handoff: none");
        assert_eq!(
            lines[6],
            "Cells: open=0 claimed=0 capped=0 blocked=0 archived=0 (total capped=0)"
        );
        assert_eq!(lines[7], "Standard commands: test=npm t");
        assert_eq!(lines[8], "Active reservations: 0");
        assert_eq!(lines[9], "Active workers: 0");
        assert_eq!(lines[10], "Critical patterns file: absent");
        assert!(lines[11].starts_with("Models (claude): generation=sonnet extraction=haiku review=opus"));
        // opencode-support oc-14: opencode now carries a built-in default
        // too (the free `opencode/*` provider names oc-14 bakes into every
        // rendered `.opencode/agent/bee-*.md`), so its line prints even
        // unconfigured — same as claude's line above.
        assert_eq!(
            lines[12],
            "Models (opencode): generation=opencode/big-pickle extraction=opencode/ling-3.0-tiny-free review=opencode/nemotron-3-ultra-free"
        );
        // Idle repo with no next_action override -> defaultState's line.
        assert_eq!(
            *lines.last().unwrap(),
            "Recommended next: No active bee work — awaiting a user request."
        );
        // JSON shape spot-checks.
        assert_eq!(status.get("gate_bypass"), Some(&json!(true)));
        assert_eq!(status.get("gate_bypass_level"), Some(&json!("normal")));
        assert_eq!(status.get("ship_visibility"), Some(&json!("off")));
        assert_eq!(status.get("pbi"), Some(&Value::Null));
    }

    /// opencode-support oc-13: a configured `models.opencode` prints its own
    /// line right after claude's, same slot order, no ceiling note (ceiling
    /// is a claude-specific concept — decisions 0012/0015/0021).
    #[test]
    fn status_text_renderer_prints_a_configured_opencode_models_line() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(
            root,
            ".bee/config.json",
            r#"{"commands":{"test":"npm t"},"models":{"opencode":{"generation":"opencode/big-pickle"}}}"#,
        );
        write(root, ".bee/state.json", r#"{"phase":"idle"}"#);
        let mut ctx = ctx_for(root);
        let status = build_status(&mut ctx, false).unwrap();
        let text = render_status_text(&status);
        let lines: Vec<&str> = text.split('\n').collect();
        assert!(lines[10].starts_with("Models (claude): generation=sonnet extraction=haiku review=opus"));
        assert_eq!(
            lines[11],
            "Models (opencode): generation=opencode/big-pickle extraction=opencode/ling-3.0-tiny-free review=opencode/nemotron-3-ultra-free"
        );
    }

    #[test]
    fn locale_compare_matches_measured_node_behavior() {
        // Measured with Node localeCompare('en', {numeric:true}) / ('en').
        let cases_numeric = [
            ("1710-2", "1710-10", Ordering::Less),
            ("01", "1", Ordering::Equal),
            ("a-b", "ab", Ordering::Less),
            ("es-1", "ES-1", Ordering::Less),
            ("_", "-", Ordering::Less),
            ("-", ".", Ordering::Less),
            (".", "0", Ordering::Less),
            ("0", "a", Ordering::Less),
            ("a", "A", Ordering::Less),
            ("A", "b", Ordering::Less),
        ];
        for (a, b, expected) in cases_numeric {
            assert_eq!(locale_cmp(a, b, true), expected, "numeric {a} vs {b}");
        }
        assert_eq!(locale_cmp("1710-2", "1710-10", false), Ordering::Greater);
    }

    #[test]
    fn js_date_parse_iso_shapes() {
        assert_eq!(js_date_parse("2026-07-29T08:17:26.986Z"), 1785313046986.0);
        assert_eq!(js_date_parse("2026-07-29"), 1785283200000.0);
        assert!(js_date_parse("garbage").is_nan());
        assert!(js_date_parse("2026-02-31").is_nan());
        assert_eq!(to_iso(1785313046986.0), "2026-07-29T08:17:26.986Z");
    }

    #[test]
    fn datamark_neutralizes_text() {
        assert_eq!(datamark(Some(&json!("plain text"))), "«plain text»");
        assert_eq!(datamark(Some(&json!("a ``` b"))), "«a  b»");
        assert_eq!(datamark(Some(&json!("keep `` two"))), "«keep `` two»");
        assert_eq!(datamark(Some(&json!("x <system foo> y </user>"))), "«x  y»");
        assert_eq!(datamark(Some(&json!("no <systemic> tag"))), "«no <systemic> tag»");
        assert_eq!(datamark(None), "«»");
        assert_eq!(datamark(Some(&json!("  padded \u{0007} "))), "«padded»");
    }

    #[test]
    fn orient_decision_line_caps_at_160_chars() {
        let long = "x".repeat(200);
        let capped = orient_decision_line(Some(&json!(long)));
        assert_eq!(capped.chars().count(), 160); // 157 + '...'
        assert!(capped.ends_with("..."));
        let short = orient_decision_line(Some(&json!("first line\nsecond")));
        assert_eq!(short, "first line");
        // Decision D3: the cap counts CHARS, not UTF-16 units — 160 astral
        // chars (320 UTF-16 units) fit under the char cap untouched.
        let astral = "🐝".repeat(160);
        assert_eq!(orient_decision_line(Some(&json!(astral))), astral);
    }

    #[test]
    fn lease_rows_render_as_reservations() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            ".bee/runtime/leases/paths/abc.json",
            r#"{"resource":"path:src/a.rs","mode":"write","workflow_id":"c-1","session_id":"s-1","workspace_id":"agent:worker-1","epoch":1,"acquired_at":"2026-07-29T00:00:00.000Z","expires_at":"2026-07-29T01:00:00.000Z","kind":"lease"}"#,
        );
        let ctx = ctx_for(root);
        let rows = list_reservations(&ctx, false);
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.get("agent"), Some(&json!("worker-1")));
        assert_eq!(row.get("cell"), Some(&json!("c-1")));
        assert_eq!(row.get("path"), Some(&json!("src/a.rs")));
        assert_eq!(row.get("ttl_seconds"), Some(&json!(3600)));
        assert_eq!(row.get("released_at"), Some(&Value::Null));
        assert_eq!(row.get("session"), Some(&json!("s-1")));
        // Expired by now -> filtered out of activeOnly, still listed raw.
        assert!(list_reservations(&ctx, true).is_empty());
    }

    // ── linked worktrees, over REAL `git worktree add` fixtures ────────────
    //
    // Every expectation below was pinned against Node on the SAME fixture
    // shape before it was written here (twin-fixture byte-diff of
    // `status --json` / `orient --json` from inside each checkout, with
    // BEE_JS_ENTRY sabotaged so bee.exe could not have delegated).

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

    /// A real main checkout with two real linked worktrees: `wt-granted`
    /// (registered in MAIN's grant registry, so it owns its own store) and
    /// `wt-ungranted` (unregistered, so it shares main's).
    fn worktree_fixture(tmp: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let main = tmp.join("main");
        std::fs::create_dir_all(&main).unwrap();
        write(&main, ".bee/onboarding.json", "{}");
        write(&main, "f.txt", "x");
        git(&main, &["init", "-q", "-b", "main", "."]);
        git(&main, &["config", "user.email", "a@b.c"]);
        git(&main, &["config", "user.name", "t"]);
        git(&main, &["add", "-A"]);
        git(&main, &["commit", "-qm", "init"]);
        let granted = tmp.join("wt-granted");
        let ungranted = tmp.join("wt-ungranted");
        git(&main, &["worktree", "add", "-q", granted.to_str().unwrap(), "-b", "wt/granted"]);
        git(&main, &["worktree", "add", "-q", ungranted.to_str().unwrap(), "-b", "wt/ungranted"]);
        write(&main, ".bee/runtime/worktree-grants.json", "{\"wt-granted\": true}\n");
        write(&granted, ".bee/onboarding.json", "{}");
        (main, granted, ungranted)
    }

    /// Build the Ctx `run()` would build standing in `cwd`.
    fn ctx_at(cwd: &Path) -> Ctx {
        match resolve_store_root_worktree(cwd) {
            RootsWt::Go(r) => Ctx {
                root: r.root,
                cwd: cwd.to_path_buf(),
                linked: r.linked,
                stderr: RefCell::new(Vec::new()),
            },
            _ => panic!("expected a resolvable root at {}", cwd.display()),
        }
    }

    /// bee.mjs ungrantedWorktreeNotice: present ONLY inside an ungranted
    /// linked worktree. The main checkout and a granted worktree both omit
    /// the key entirely (GH #30) — this is the exact status shape whose loss
    /// blocked the routing flip.
    #[test]
    fn worktree_notice_fires_only_inside_an_ungranted_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, ungranted) = worktree_fixture(tmp.path());

        assert_eq!(ungranted_worktree_notice(&ctx_for(&main)), None);
        assert_eq!(ungranted_worktree_notice(&ctx_at(&main)), None);
        assert_eq!(ungranted_worktree_notice(&ctx_at(&granted)), None);
        let notice = ungranted_worktree_notice(&ctx_at(&ungranted)).expect("notice");
        assert_eq!(notice, UNGRANTED_WORKTREE_NOTICE);
        assert!(notice.starts_with("⚠ This linked worktree is UNGRANTED"));
        assert!(notice.ends_with("from inside it."));

        // And it lands in the payload under the right key, only there.
        let mut c = ctx_at(&ungranted);
        let status = build_status(&mut c, false).expect("status");
        assert_eq!(status.get("worktree_notice"), Some(&json!(notice)));
        let mut c = ctx_at(&granted);
        assert!(!build_status(&mut c, false).unwrap().contains_key("worktree_notice"));
        let mut c = ctx_at(&main);
        assert!(!build_status(&mut c, false).unwrap().contains_key("worktree_notice"));
    }

    /// state.mjs controlRootFor: sessions/claims/workers are CONTROL plane —
    /// from inside a granted worktree they must resolve onto MAIN's store,
    /// never the worktree's own. (An ungranted worktree's `root` already IS
    /// main, so it agrees trivially.)
    #[test]
    fn control_root_re_roots_onto_main_from_a_granted_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, ungranted) = worktree_fixture(tmp.path());
        // Identity, not spelling: `main_root` comes out of the gitdir chain
        // (git's own writing) while the fixture holds tempdir()'s, and on a
        // Windows runner those are the long and 8.3-short forms of one path.
        let n = |p: &Path| {
            let c = dunce::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
            normalize_abs_lexical(&c.to_string_lossy())
        };

        assert_eq!(n(&control_root_for(&mut ctx_at(&main)).unwrap()), n(&main));
        assert_eq!(n(&control_root_for(&mut ctx_at(&granted)).unwrap()), n(&main));
        assert_eq!(n(&control_root_for(&mut ctx_at(&ungranted)).unwrap()), n(&main));
        // The store root itself is NOT re-rooted: it is the worktree's own
        // when granted, main's when not.
        assert_eq!(n(&ctx_at(&granted).root), n(&granted));
        assert_eq!(n(&ctx_at(&ungranted).root), n(&main));

        // A live session written into MAIN's store only is visible from the
        // granted worktree's status through that control root.
        let now = to_iso(now_ms());
        write(
            &main,
            ".bee/sessions/sess-live.json",
            &format!("{{\"id\":\"sess-live\",\"started_at\":\"{now}\",\"last_heartbeat\":\"{now}\"}}"),
        );
        let ctrl = control_root_for(&mut ctx_at(&granted)).unwrap();
        let workers = active_workers(&ctx_at(&granted), &ctrl, None).unwrap();
        assert_eq!(workers.len(), 1);
        assert_eq!(workers[0].get("session_id"), Some(&json!("sess-live")));
        // Reading the worktree's own root instead would find nothing — this
        // is exactly the bug the `controlRoot == root` assumption caused.
        assert!(active_workers(&ctx_at(&granted), &granted, None).unwrap().is_empty());
    }

    /// reservations.mjs's own cycle-safe control-root walk (LEASE files) also
    /// answers mainRoot inside a granted worktree — from the git link alone,
    /// with no grant registry involved.
    #[test]
    fn reservations_control_root_follows_the_git_link() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, ungranted) = worktree_fixture(tmp.path());
        // Identity, not spelling: `main_root` comes out of the gitdir chain
        // (git's own writing) while the fixture holds tempdir()'s, and on a
        // Windows runner those are the long and 8.3-short forms of one path.
        let n = |p: &Path| {
            let c = dunce::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
            normalize_abs_lexical(&c.to_string_lossy())
        };
        assert_eq!(n(&reservations_control_root(&ctx_at(&main))), n(&main));
        assert_eq!(n(&reservations_control_root(&ctx_at(&granted))), n(&main));
        assert_eq!(n(&reservations_control_root(&ctx_at(&ungranted))), n(&main));
        // findMainRoot fails OPEN: a link it cannot validate answers `root`.
        let orphan = tmp.path().join("orphan");
        std::fs::create_dir_all(&orphan).unwrap();
        std::fs::write(orphan.join(".git"), "gitdir: nowhere").unwrap();
        let ctx = Ctx { root: orphan.clone(), cwd: orphan.clone(), linked: None, stderr: RefCell::new(Vec::new()) };
        assert_eq!(n(&reservations_control_root(&ctx)), n(&orphan));
    }

    /// bee.mjs readWorktreeBranch over a real `git worktree add` HEAD.
    #[test]
    fn worktree_branch_reads_the_linked_head() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, _granted, _ungranted) = worktree_fixture(tmp.path());
        assert_eq!(read_worktree_branch(&main, "wt-granted").as_deref(), Some("wt/granted"));
        assert_eq!(read_worktree_branch(&main, "no-such-id"), None);
        // Detached HEAD (a bare sha) is null, not the sha.
        std::fs::write(
            main.join(".git").join("worktrees").join("wt-granted").join("HEAD"),
            "0123456789abcdef0123456789abcdef01234567\n",
        )
        .unwrap();
        assert_eq!(read_worktree_branch(&main, "wt-granted"), None);
    }

    /// bee.mjs orientWorktreeContext, both halves, over the real fixture.
    #[test]
    fn orient_worktree_context_serves_both_halves() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, ungranted) = worktree_fixture(tmp.path());

        // Inside the GRANTED worktree: the merge-back packet. `feature` is
        // whatever status resolved, `branch` comes from the linked HEAD.
        let mut status = JMap::new();
        status.insert("feature".into(), json!("demo"));
        let block = orient_worktree_context(&mut ctx_at(&granted), &status)
            .unwrap()
            .expect("worktree block inside a granted worktree");
        assert_eq!(block.get("location"), Some(&json!("worktree")));
        assert_eq!(block.get("id"), Some(&json!("wt-granted")));
        assert_eq!(block.get("feature"), Some(&json!("demo")));
        assert_eq!(block.get("branch"), Some(&json!("wt/granted")));
        assert_eq!(
            block.get("merge_command"),
            Some(&json!("bee worktree merge --id wt-granted"))
        );
        // The text render takes the non-'main' branch.
        let mut packet = JMap::new();
        packet.insert("worktree".into(), Value::Object(block.clone()));
        packet.insert("where".into(), json!({"phase":"idle","feature":"demo","mode":null,"gates":{},"gate_bypass_level":"off"}));
        packet.insert("decisions".into(), json!({"context_md":null,"active_count":0,"recent":[]}));
        packet.insert("work".into(), json!({"cells":{"open":0,"claimed":0,"capped":0},"ready":[],"blockers":[]}));
        packet.insert("next".into(), json!({"action":"a","skill":"bee-hive","command":null}));
        assert!(render_orient_text(&packet).contains(
            "worktree: wt-granted (branch wt/granted) — merge back from main with bee worktree merge --id wt-granted"
        ));

        // Inside the UNGRANTED worktree: no block at all.
        assert!(orient_worktree_context(&mut ctx_at(&ungranted), &status)
            .unwrap()
            .is_none());

        // From MAIN with a code-touching lane whose feature lives in the
        // granted worktree: the "go there" block.
        write(&granted, ".bee/runtime/worktree-identity.json", "{\"feature\":\"demo\"}");
        let mut status_main = JMap::new();
        status_main.insert("feature".into(), json!("demo"));
        status_main.insert("route".into(), json!({"lane": "small"}));
        let block = orient_worktree_context(&mut ctx_at(&main), &status_main)
            .unwrap()
            .expect("main-side worktree block");
        assert_eq!(block.get("location"), Some(&json!("main")));
        assert_eq!(block.get("id"), Some(&json!("wt-granted")));
        assert!(tpl(block.get("guidance")).starts_with("open your session at "));
        // A docs lane is exempt -> no block, byte-unchanged orient.
        status_main.insert("route".into(), json!({"lane": "docs"}));
        assert!(orient_worktree_context(&mut ctx_at(&main), &status_main)
            .unwrap()
            .is_none());
    }

    /// The whole orient packet from inside a granted worktree carries the
    /// `worktree` key — the exact block whose loss was the measured C2 break
    /// that kept this routing flip parked.
    #[test]
    fn orient_packet_carries_the_worktree_block_inside_a_granted_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let (_main, granted, ungranted) = worktree_fixture(tmp.path());
        let packet = build_orient(&mut ctx_at(&granted)).expect("orient");
        let block = packet.get("worktree").expect("worktree block");
        assert_eq!(vget(block, "location"), Some(&json!("worktree")));
        assert_eq!(vget(block, "id"), Some(&json!("wt-granted")));
        // next.command stays orient's own (only the 'main' location overrides).
        assert!(!packet.contains_key("worktree_notice"));
        // The ungranted worktree's orient has no block.
        assert!(!build_orient(&mut ctx_at(&ungranted))
            .expect("orient")
            .contains_key("worktree"));
    }

    // ── irf-1 (PBI p-9c48a67c read-side residue) — `bee status` counts scope
    //    to the granted island's own feature ───────────────────────────────
    //
    // `git worktree add` checks out `.bee/cells` in FULL (it is
    // git-tracked), and ips-1's prune-on-register pass only ever removes
    // UNTRACKED foreign-feature files — a TRACKED one legitimately rides
    // along on disk forever. `list_cells`/`list_cells_including_archive`
    // must never surface it in a status count.

    /// A cell fixture written straight into `.bee/cells/<id>.json`.
    fn write_status_cell(root: &Path, id: &str, feature: &str, status: &str) {
        write(
            root,
            &format!(".bee/cells/{id}.json"),
            &format!(r#"{{"id":"{id}","feature":"{feature}","status":"{status}","title":"t"}}"#),
        );
    }

    /// RED before irf-1: a fresh granted island legitimately holds another
    /// feature's tracked cell file — `list_cells` (the shared enumerator
    /// every status count reads through, `build.rs`'s own `list_cells(ctx,
    /// None, None)`) must scope it out with no explicit feature filter.
    #[test]
    fn list_cells_in_a_granted_island_never_surfaces_a_foreign_features_residue() {
        let tmp = tempfile::tempdir().unwrap();
        let (_main, granted, _ungranted) = worktree_fixture(tmp.path());
        write(&granted, ".bee/runtime/worktree-identity.json", "{\"feature\":\"feat-a\"}");
        write_status_cell(&granted, "a-1", "feat-a", "open");
        write_status_cell(&granted, "b-1", "feat-b", "open");

        let ids: Vec<String> = list_cells(&ctx_at(&granted), None, None)
            .unwrap()
            .iter()
            .map(|c| tpl(vget(c, "id")))
            .collect();
        assert_eq!(ids, vec!["a-1"], "feature B's residue must never be counted from the island");
    }

    /// Same residue, the archive-aware door (`list_cells_including_archive`,
    /// the debt-door counters' own read) — "Archived cells: same rule".
    #[test]
    fn list_cells_including_archive_in_a_granted_island_scopes_to_its_own_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let (_main, granted, _ungranted) = worktree_fixture(tmp.path());
        write(&granted, ".bee/runtime/worktree-identity.json", "{\"feature\":\"feat-a\"}");
        write_status_cell(&granted, "a-1", "feat-a", "capped");
        write_status_cell(&granted, "b-1", "feat-b", "capped");
        write(
            &granted,
            ".bee/cells/archive/feat-a/a-arch.json",
            r#"{"id":"a-arch","feature":"feat-a","status":"capped","title":"t"}"#,
        );
        write(
            &granted,
            ".bee/cells/archive/feat-b/b-arch.json",
            r#"{"id":"b-arch","feature":"feat-b","status":"capped","title":"t"}"#,
        );

        let ids: Vec<String> = list_cells_including_archive(&ctx_at(&granted), None, Some("capped"))
            .unwrap()
            .iter()
            .map(|c| tpl(vget(c, "id")))
            .collect();
        let mut ids = ids;
        ids.sort();
        assert_eq!(ids, vec!["a-1", "a-arch"], "feature B's live AND archived cells stay hidden");
    }

    /// The whole `bee status` cell-count payload agrees: only feature A's
    /// counts show from inside its own granted island.
    #[test]
    fn status_cell_counts_from_a_granted_island_never_include_a_foreign_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let (_main, granted, _ungranted) = worktree_fixture(tmp.path());
        write(&granted, ".bee/runtime/worktree-identity.json", "{\"feature\":\"feat-a\"}");
        write_status_cell(&granted, "a-1", "feat-a", "open");
        write_status_cell(&granted, "b-1", "feat-b", "open");
        write_status_cell(&granted, "b-2", "feat-b", "open");

        let mut ctx = ctx_at(&granted);
        let status = build_status(&mut ctx, false).expect("status");
        assert_eq!(status.get("cells").and_then(|c| c.get("open")), Some(&json!(1)));
    }

    /// The UNGRANTED worktree shares main's store (`root == main_root`) —
    /// unfiltered, byte-identical to before this filter existed.
    #[test]
    fn list_cells_in_an_ungranted_worktree_stays_unscoped() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, _granted, ungranted) = worktree_fixture(tmp.path());
        write_status_cell(&main, "a-1", "feat-a", "open");
        write_status_cell(&main, "b-1", "feat-b", "open");

        let ids: Vec<String> = list_cells(&ctx_at(&ungranted), None, None)
            .unwrap()
            .iter()
            .map(|c| tpl(vget(c, "id")))
            .collect();
        assert_eq!(ids, vec!["a-1", "b-1"]);
    }

    /// The MAIN store itself, several features at once — pinned
    /// byte-identical: `island_feature_scope` never engages outside a
    /// GRANTED worktree island.
    #[test]
    fn list_cells_at_the_main_store_shows_every_feature_unfiltered() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_status_cell(root, "a-1", "feat-a", "open");
        write_status_cell(root, "b-1", "feat-b", "open");
        write_status_cell(root, "c-1", "feat-c", "open");

        let ids: Vec<String> = list_cells(&ctx_for(root), None, None)
            .unwrap()
            .iter()
            .map(|c| tpl(vget(c, "id")))
            .collect();
        assert_eq!(ids, vec!["a-1", "b-1", "c-1"]);
    }

    // ── recovery (recovery.mjs) ────────────────────────────────────────────
    //
    // Ported from packages/bee/tests/test_recovery.mjs. The Node oracle
    // injects `now`; this port reads `now_ms()` internally, so every fixture
    // timestamp below is anchored to the wall clock instead.

    /// `detectCrashCandidates` reads BEE_SESSION_ID / CLAUDE_CODE_SESSION_ID
    /// from the PROCESS environment (claims.mjs resolveSessionId). One test
    /// has to set it, and cargo runs the others on parallel threads of the
    /// same process — so every test that calls `detect_crash_candidates`
    /// takes this lock, not just the one that writes.
    fn session_env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// test_recovery.mjs cleanEndEvents: the stop/turn/last-prompt trio plus
    /// the trailing bookkeeping events a clean stop is allowed to emit.
    fn clean_end_events(t0: f64) -> Vec<Value> {
        vec![
            json!({"type":"user","timestamp":to_iso(t0),"message":{"role":"user","content":[{"type":"text","text":"go"}]}}),
            json!({"type":"assistant","timestamp":to_iso(t0 + 1000.0),"message":{"role":"assistant"}}),
            json!({"type":"system","subtype":"stop_hook_summary","timestamp":to_iso(t0 + 1100.0)}),
            json!({"type":"system","subtype":"turn_duration","durationMs":5000,"timestamp":to_iso(t0 + 1105.0)}),
            json!({"type":"last-prompt","lastPrompt":"hi","leafUuid":"x"}),
            json!({"type":"ai-title","aiTitle":"demo"}),
            json!({"type":"mode","mode":"normal"}),
        ]
    }

    /// test_recovery.mjs dirtyEndEvents: ends mid-turn, no trio at all.
    fn dirty_end_events(t0: f64) -> Vec<Value> {
        vec![
            json!({"type":"user","timestamp":to_iso(t0),"message":{"role":"user","content":[{"type":"text","text":"go"}]}}),
            json!({"type":"assistant","timestamp":to_iso(t0 + 1000.0),"message":{"role":"assistant"}}),
        ]
    }

    fn write_jsonl_file(file: &Path, events: &[Value]) {
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        let body = if events.is_empty() {
            String::new()
        } else {
            let lines: Vec<String> =
                events.iter().map(|e| serde_json::to_string(e).unwrap()).collect();
            format!("{}\n", lines.join("\n"))
        };
        std::fs::write(file, body).unwrap();
    }

    /// test_recovery.mjs writeSessionRecord.
    fn write_session_record(
        root: &Path,
        id: &str,
        started_at: &str,
        last_heartbeat: &str,
        lane: Option<&str>,
        transcript_path: Option<&str>,
    ) {
        let mut m = JMap::new();
        m.insert("id".into(), json!(id));
        m.insert("started_at".into(), json!(started_at));
        m.insert("last_heartbeat".into(), json!(last_heartbeat));
        if let Some(l) = lane {
            m.insert("lane".into(), json!(l));
        }
        if let Some(t) = transcript_path {
            m.insert("transcript_path".into(), json!(t));
        }
        write(
            root,
            &format!(".bee/sessions/{id}.json"),
            &serde_json::to_string(&Value::Object(m)).unwrap(),
        );
    }

    /// test_recovery.mjs writeLaneRecord.
    fn write_lane_record(root: &Path, feature: &str, phase: &str) {
        write(
            root,
            &format!(".bee/lanes/{feature}.json"),
            &serde_json::to_string(&json!({
                "schema_version": "1.0",
                "feature": feature,
                "mode": "standard",
                "phase": phase,
                "approved_gates": {"context": true, "shape": true, "execution": true, "review": false},
                "summary": "",
                "next_action": "",
            }))
            .unwrap(),
        );
    }

    /// test_recovery.mjs writeClaim.
    fn write_claim_record(root: &Path, cell_id: &str, session_id: &str, claimed_at: &str) {
        write(
            root,
            &format!(".bee/claims/{cell_id}.json"),
            &serde_json::to_string(&json!({
                "cell": cell_id,
                "session": session_id,
                "ttl_seconds": 3600,
                "claimed_at": claimed_at,
                "acquired_at": claimed_at,
            }))
            .unwrap(),
        );
    }

    /// test_recovery.mjs writeDecision (appends — the store is a ledger).
    fn append_decision(root: &Path, id: &str, date: &str) {
        let file = root.join(".bee").join("decisions.jsonl");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        let line = serde_json::to_string(&json!({
            "id": id, "type": "decide", "date": date, "decision": "x", "rationale": "y",
            "alternatives": null, "scope": "repo", "source": "user", "confidence": null,
        }))
        .unwrap();
        let mut prev = std::fs::read_to_string(&file).unwrap_or_default();
        prev.push_str(&line);
        prev.push('\n');
        std::fs::write(file, prev).unwrap();
    }

    /// test_recovery.mjs writeCaptureStub.
    fn append_capture_stub(root: &Path, id: &str, at: &str, lane: Option<&str>) {
        let file = root.join(".bee").join("capture-queue.jsonl");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        let line = serde_json::to_string(&json!({
            "kind": "stub", "id": id, "at": at, "outcome": "x",
            "dids": [], "area": null, "files": [],
            "lane": lane.map(Value::from).unwrap_or(Value::Null),
        }))
        .unwrap();
        let mut prev = std::fs::read_to_string(&file).unwrap_or_default();
        prev.push_str(&line);
        prev.push('\n');
        std::fs::write(file, prev).unwrap();
    }

    /// Pairs with `append_capture_stub` — a `flush` row for the id, so the
    /// stub it targets drops out of `pendingCaptureStubs`' stub-minus-flush
    /// membership.
    fn append_capture_flush(root: &Path, id: &str, at: &str) {
        let file = root.join(".bee").join("capture-queue.jsonl");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        let line = serde_json::to_string(&json!({
            "kind": "flush", "id": id, "at": at, "into": null,
        }))
        .unwrap();
        let mut prev = std::fs::read_to_string(&file).unwrap_or_default();
        prev.push_str(&line);
        prev.push('\n');
        std::fs::write(file, prev).unwrap();
    }

    const MS_PER_DAY: f64 = 24.0 * 60.0 * 60.0 * 1000.0;

    /// A minimal orient fixture: onboarding at the running version (no
    /// drift warning) and an idle, feature-less state, so `work.blockers`
    /// starts empty and only the capture-queue escalation under test can
    /// populate it.
    fn orient_blockers(root: &Path) -> Vec<String> {
        write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(root, ".bee/state.json", r#"{"phase":"idle"}"#);
        let mut ctx = ctx_for(root);
        let packet = build_orient(&mut ctx).unwrap();
        vget(packet.get("work").unwrap(), "blockers")
            .and_then(Value::as_array)
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    }

    /// counter-teeth D2: "pending" for the capture-queue escalation is
    /// exactly `capture_queue_summary`'s stub-minus-flush membership — a
    /// flushed stub feeds neither the count threshold nor the age
    /// threshold, even when it would otherwise trip both (old AND part of
    /// a large batch). Proves the escalation reuses that membership
    /// instead of re-deriving its own.
    #[test]
    fn capture_queue_pending_excludes_flushed_stubs_from_blocker_escalation() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let now = now_ms();
        // Old enough to trip the age threshold on its own — but flushed.
        let old_at = to_iso(now - 10.0 * MS_PER_DAY);
        append_capture_stub(root, "old-flushed", &old_at, None);
        append_capture_flush(root, "old-flushed", &to_iso(now));
        // A handful of fresh, unflushed stubs — well under the count
        // threshold on their own.
        for i in 0..3 {
            append_capture_stub(root, &format!("fresh-{i}"), &to_iso(now - (i as f64) * 1000.0), None);
        }
        assert_eq!(
            orient_blockers(root),
            Vec::<String>::new(),
            "a flushed stub must not feed the count OR the age escalation"
        );
    }

    /// D2 boundary: 9 pending fresh stubs stays an offer (status carries
    /// the count; `work.blockers` stays empty) — one short of the
    /// CAPTURE_QUEUE_BLOCKER_MIN_PENDING threshold.
    #[test]
    fn capture_queue_offer_at_nine_pending_fresh_stubs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let now = now_ms();
        for i in 0..9 {
            append_capture_stub(root, &format!("s{i}"), &to_iso(now - (i as f64) * 1000.0), None);
        }
        assert_eq!(orient_blockers(root), Vec::<String>::new());
    }

    /// D2 boundary: the 10th pending stub crosses
    /// CAPTURE_QUEUE_BLOCKER_MIN_PENDING and moves the capture-queue line
    /// into `work.blockers[]`.
    #[test]
    fn capture_queue_blocker_at_ten_pending_stubs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let now = now_ms();
        for i in 0..10 {
            append_capture_stub(root, &format!("s{i}"), &to_iso(now - (i as f64) * 1000.0), None);
        }
        let blockers = orient_blockers(root);
        assert_eq!(blockers.len(), 1, "{blockers:?}");
        assert!(
            blockers[0].starts_with("capture queue: 10 pending stub(s)"),
            "{blockers:?}"
        );
    }

    /// D2 age boundary: a single pending stub older than
    /// CAPTURE_QUEUE_BLOCKER_MAX_AGE_DAYS (7) is a blocker on its own,
    /// nowhere near the count threshold; the same single stub within 7
    /// days stays an offer.
    #[test]
    fn capture_queue_blocker_when_oldest_pending_stub_exceeds_seven_days() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let now = now_ms();

        // Within the window: no blocker.
        append_capture_stub(root, "recent", &to_iso(now - 6.0 * MS_PER_DAY), None);
        assert_eq!(orient_blockers(root), Vec::<String>::new(), "6 days old must stay an offer");

        // A fresh root past the same threshold: blocker.
        let tmp2 = tempfile::tempdir().unwrap();
        let root2 = tmp2.path();
        append_capture_stub(root2, "stale", &to_iso(now - 8.0 * MS_PER_DAY), None);
        let blockers = orient_blockers(root2);
        assert_eq!(blockers.len(), 1, "{blockers:?}");
        assert!(
            blockers[0].starts_with("capture queue: 1 pending stub(s)"),
            "{blockers:?}"
        );
    }

    /// D3 (kf-2): `bee orient` names an unapplied promote proposal in
    /// `work.blockers[]`, same report-only voice as the scribing-debt and
    /// capture-queue lines just above it — the feature, its own count
    /// clause, and its file path, never a refusal or an exit-code change.
    #[test]
    fn orient_names_an_unapplied_promote_proposal() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            "docs/history/f3/promote-proposals.md",
            "promote proposal for work item \"f3\" (docs/history/f3/CONTEXT.md) — 2 capped cell(s): b-1, b-2\nanchor: history — docs/history/f3/CONTEXT.md\n",
        );
        let blockers = orient_blockers(root);
        assert_eq!(blockers.len(), 1, "{blockers:?}");
        assert!(
            blockers[0]
                == "promote proposal unapplied: 1 feature(s) — f3 (2 capped cell(s), docs/history/f3/promote-proposals.md)",
            "{blockers:?}"
        );

        // A compounding run at or after the file's own mtime silences it.
        let mtime_ms = std::fs::metadata(root.join("docs/history/f3/promote-proposals.md"))
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;
        write(
            root,
            ".bee/logs/scribing-runs.jsonl",
            &format!(
                "{{\"ts\":\"{}\",\"feature\":\"f3\",\"areas\":[]}}\n",
                to_iso(mtime_ms + 1000.0)
            ),
        );
        assert_eq!(orient_blockers(root), Vec::<String>::new());
    }

    /// test_recovery.mjs writeCappedCell.
    fn write_capped_cell(root: &Path, id: &str, feature: &str, capped_at: &str) {
        write(
            root,
            &format!(".bee/cells/{id}.json"),
            &serde_json::to_string(
                &json!({"id": id, "feature": feature, "status": "capped", "trace": {"capped_at": capped_at}}),
            )
            .unwrap(),
        );
    }

    /// test_recovery.mjs hasCleanEndTrio, all five rows of its truth table.
    #[test]
    fn clean_end_trio_truth_table() {
        let t0 = 1_785_313_046_986.0;
        // trio + tolerated trailing bookkeeping (queue/ai-title/mode).
        assert!(has_clean_end_trio(&clean_end_events(t0)));
        // entirely absent (mid-turn tail).
        assert!(!has_clean_end_trio(&dirty_end_events(t0)));
        // stop_hook_summary + turn_duration alone is NOT the full trio.
        let no_last_prompt: Vec<Value> = clean_end_events(t0)
            .into_iter()
            .filter(|e| {
                !(str_eq(vget(e, "type"), "last-prompt")
                    || str_eq(vget(e, "type"), "ai-title")
                    || str_eq(vget(e, "type"), "mode"))
            })
            .collect();
        assert!(!has_clean_end_trio(&no_last_prompt));
        // a conversational event AFTER the trio reopens the turn.
        let mut followed = clean_end_events(t0);
        followed.push(json!({"type":"user","timestamp":to_iso(t0 + 5000.0),"message":{}}));
        assert!(!has_clean_end_trio(&followed));
        // empty tail.
        assert!(!has_clean_end_trio(&[]));
    }

    /// test_recovery.mjs readTranscriptTail: the bounded window drops the
    /// truncated first line, malformed lines are skipped, and a missing or
    /// empty file is [] rather than a throw.
    #[test]
    fn transcript_tail_is_a_bounded_window_over_well_formed_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        // missing / empty.
        assert!(
            read_transcript_tail(&dir.join("nope.jsonl"), DEFAULT_TAIL_MAX_BYTES)
                .unwrap()
                .is_empty()
        );
        std::fs::write(dir.join("empty.jsonl"), "").unwrap();
        assert!(
            read_transcript_tail(&dir.join("empty.jsonl"), DEFAULT_TAIL_MAX_BYTES)
                .unwrap()
                .is_empty()
        );

        // Window smaller than the padding forces a mid-line start.
        let pad = serde_json::to_string(&json!({"type":"assistant","pad":"x".repeat(500)})).unwrap();
        let mut lines: Vec<String> = (0..400).map(|_| pad.clone()).collect();
        lines.push(
            serde_json::to_string(&json!({"type":"user","marker":"TAIL_EVENT_1"})).unwrap(),
        );
        lines.push(
            serde_json::to_string(&json!({"type":"assistant","marker":"TAIL_EVENT_2"})).unwrap(),
        );
        let big = dir.join("big.jsonl");
        std::fs::write(&big, format!("{}\n", lines.join("\n"))).unwrap();
        let tail = read_transcript_tail(&big, 600).unwrap();
        let markers: Vec<String> = tail
            .iter()
            .filter_map(|e| vget(e, "marker").and_then(|m| m.as_str()).map(str::to_string))
            .collect();
        assert!(markers.contains(&"TAIL_EVENT_1".to_string()));
        assert!(markers.contains(&"TAIL_EVENT_2".to_string()));
        // The truncated leading fragment never survives as an entry, and the
        // window really is bounded: 400×~520B of padding cannot fit in 600B.
        assert!(tail.iter().all(|e| matches!(e, Value::Object(_))));
        assert!(tail.len() < 10, "window kept {} events, expected a handful", tail.len());
        // Control: the same file read with a window bigger than the file
        // returns every line, proving the small window is what dropped them.
        assert_eq!(read_transcript_tail(&big, 10_000_000).unwrap().len(), 402);

        // Malformed lines inside the window are skipped, order preserved.
        let malformed = dir.join("malformed.jsonl");
        std::fs::write(
            &malformed,
            "{\"type\":\"user\",\"marker\":\"ok1\"}\n{not valid json\n{\"type\":\"assistant\",\"marker\":\"ok2\"}\n",
        )
        .unwrap();
        let tail = read_transcript_tail(&malformed, DEFAULT_TAIL_MAX_BYTES).unwrap();
        assert_eq!(tail.len(), 2);
        assert_eq!(vget(&tail[0], "marker"), Some(&json!("ok1")));
        assert_eq!(vget(&tail[1], "marker"), Some(&json!("ok2")));
    }

    /// test_recovery.mjs lastDurableSettlement — max across decisions,
    /// capture stubs and cell traces, read through the same store readers
    /// `detect_crash_candidates` uses.
    #[test]
    fn last_durable_settlement_maxes_across_stores_and_scopes_by_lane() {
        let read_stores = |ctx: &Ctx| -> (Vec<Value>, Vec<Value>, Vec<Value>) {
            (
                active_decisions(ctx, None),
                read_jsonl(&ctx.root.join(".bee").join("capture-queue.jsonl")),
                list_cells(ctx, None, None).unwrap(),
            )
        };
        let base = 1_785_313_046_986.0;

        // (a) global: the newest of the three sources wins (capture stub).
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        append_decision(root, "d1", &to_iso(base));
        append_capture_stub(root, "c1", &to_iso(base + 2000.0), None);
        write_capped_cell(root, "feat-1", "feat", &to_iso(base + 1000.0));
        let ctx = ctx_for(root);
        let (d, c, cells) = read_stores(&ctx);
        assert_eq!(
            last_durable_settlement(None, &d, &c, &cells).map(to_iso),
            Some(to_iso(base + 2000.0))
        );

        // (b) lane scoping filters stubs and cells; decisions stay GLOBAL.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        append_decision(root, "d1", &to_iso(base + 9000.0));
        append_capture_stub(root, "c-mine", &to_iso(base + 1000.0), Some("mine"));
        append_capture_stub(root, "c-other", &to_iso(base + 5000.0), Some("other"));
        write_capped_cell(root, "mine-1", "mine", &to_iso(base + 2000.0));
        write_capped_cell(root, "other-1", "other", &to_iso(base + 6000.0));
        let ctx = ctx_for(root);
        let (d, c, cells) = read_stores(&ctx);
        let lane = json!("mine");
        assert_eq!(
            last_durable_settlement(Some(&lane), &d, &c, &cells).map(to_iso),
            Some(to_iso(base + 9000.0)),
            "the unscoped decision is still counted; the +5000/+6000 rows belong to lane \"other\""
        );
        // Control: unscoped, the "other" cell at +6000 is the max — so the
        // scoped answer above really was a filter, not a coincidence.
        assert_eq!(
            last_durable_settlement(None, &d, &c, &cells).map(to_iso),
            Some(to_iso(base + 9000.0))
        );
        let other = json!("other");
        assert_eq!(
            last_durable_settlement(Some(&other), &d, &c, &cells).map(to_iso),
            Some(to_iso(base + 9000.0))
        );

        // (c) nothing settled anywhere -> None (caller falls back to
        // started_at), lane-scoped or not.
        let tmp = tempfile::tempdir().unwrap();
        let ctx = ctx_for(tmp.path());
        let (d, c, cells) = read_stores(&ctx);
        assert_eq!(last_durable_settlement(None, &d, &c, &cells), None);
        assert_eq!(last_durable_settlement(Some(&lane), &d, &c, &cells), None);
    }

    /// Seeds the canonical "this session crashed" fixture: one heartbeat-stale
    /// session bound to a non-terminal lane, whose dirty (mid-turn) transcript
    /// is reached through the session record's own stored `transcript_path`
    /// (recovery.mjs/perf.mjs D5 Codex bridge). The encoded-layout root that
    /// the Node oracle uses cannot be created on win32 — see
    /// `crash_candidate_resolves_through_the_encoded_layout_root` — so the
    /// stored-path arm carries the ladder here.
    ///
    /// Returns (projects_root — deliberately absent, transcript path).
    fn seed_crash_fixture(root: &Path, sid: &str, now: f64) -> (String, PathBuf) {
        let transcript = root.join("transcripts").join(format!("{sid}.jsonl"));
        write_jsonl_file(&transcript, &dirty_end_events(now - 500_000.0));
        write_session_record(
            root,
            sid,
            &to_iso(now - 2_000_000.0),
            &to_iso(now - 1_000_000.0), // > the 900s staleness law
            Some("feat-lane"),
            Some(&transcript.to_string_lossy()),
        );
        write_lane_record(root, "feat-lane", "swarming");
        (
            root.join("projects").to_string_lossy().into_owned(),
            transcript,
        )
    }

    /// test_recovery.mjs detectCrashCandidates exclusion ladder. Every rung is
    /// one mutation away from the SAME fixture that does produce a candidate,
    /// so no rung can pass vacuously.
    #[test]
    fn crash_candidate_exclusion_ladder_each_against_its_firing_control() {
        let _guard = session_env_lock();
        let now = now_ms();
        let sid = "sess-ladder";

        // ── the control: this fixture IS a crash candidate ────────────────
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let (projects_root, transcript) = seed_crash_fixture(root, sid, now);
        let out = detect_crash_candidates(&mut ctx_for(root), &projects_root).unwrap();
        assert_eq!(out.len(), 1, "control must fire");
        assert_eq!(vget(&out[0], "session_id"), Some(&json!(sid)));
        assert_eq!(vget(&out[0], "lane"), Some(&json!("feat-lane")));
        assert_eq!(vget(&out[0], "work_signal"), Some(&json!("lane")));
        assert_eq!(
            vget(&out[0], "transcript"),
            Some(&json!(transcript.to_string_lossy())),
            "resolved from the stored transcript_path, never a layout guess"
        );
        assert_eq!(
            vget(&out[0], "runtime"),
            Some(&Value::Null),
            "no scanned root prefixes the stored path -> runtime unknown"
        );
        assert_eq!(
            vget(&out[0], "since"),
            Some(&json!(to_iso(now - 2_000_000.0))),
            "no durable settlement anywhere -> since falls back to started_at"
        );

        // ── rung 1: the CURRENT live session is never its own candidate ───
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let (projects_root, _) = seed_crash_fixture(root, sid, now);
        let prev = std::env::var_os("BEE_SESSION_ID");
        unsafe { std::env::set_var("BEE_SESSION_ID", sid) };
        let out = detect_crash_candidates(&mut ctx_for(root), &projects_root);
        unsafe {
            match &prev {
                Some(v) => std::env::set_var("BEE_SESSION_ID", v),
                None => std::env::remove_var("BEE_SESSION_ID"),
            }
        }
        assert!(out.unwrap().is_empty(), "the resolved current session is excluded");

        // ── rung 2: a fresh heartbeat is not stale ────────────────────────
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let (projects_root, transcript) = seed_crash_fixture(root, sid, now);
        write_session_record(
            root,
            sid,
            &to_iso(now - 2_000_000.0),
            &to_iso(now - 60_000.0), // 60s old, inside the 900s law
            Some("feat-lane"),
            Some(&transcript.to_string_lossy()),
        );
        assert!(
            detect_crash_candidates(&mut ctx_for(root), &projects_root)
                .unwrap()
                .is_empty()
        );

        // ── rung 3: a clean-end tail beats even a live-looking lane ───────
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let (projects_root, transcript) = seed_crash_fixture(root, sid, now);
        write_jsonl_file(&transcript, &clean_end_events(now - 500_000.0));
        assert!(
            detect_crash_candidates(&mut ctx_for(root), &projects_root)
                .unwrap()
                .is_empty()
        );

        // ── rung 4: no transcript at all -> nothing proves an abrupt stop ──
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let (projects_root, transcript) = seed_crash_fixture(root, sid, now);
        std::fs::remove_file(&transcript).unwrap();
        assert!(
            detect_crash_candidates(&mut ctx_for(root), &projects_root)
                .unwrap()
                .is_empty()
        );

        // ── rung 5: a TERMINAL-phase lane with no other signal ────────────
        // Settlement is newer than the transcript's last activity, so the
        // transcript_activity arm cannot fire either.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let (projects_root, _) = seed_crash_fixture(root, sid, now);
        write_capped_cell(root, "feat-lane-1", "feat-lane", &to_iso(now - 1000.0));
        write_lane_record(root, "feat-lane", "compounding-complete");
        assert!(
            detect_crash_candidates(&mut ctx_for(root), &projects_root)
                .unwrap()
                .is_empty()
        );
        // Same store, same transcript, only the phase flipped back -> fires.
        write_lane_record(root, "feat-lane", "swarming");
        let out = detect_crash_candidates(&mut ctx_for(root), &projects_root).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(vget(&out[0], "work_signal"), Some(&json!("lane")));
        assert_eq!(
            vget(&out[0], "since"),
            Some(&json!(to_iso(now - 1000.0))),
            "the capped cell is now the durable settlement the window is measured from"
        );
        // "idle" is the other terminal phase.
        write_lane_record(root, "feat-lane", "idle");
        assert!(
            detect_crash_candidates(&mut ctx_for(root), &projects_root)
                .unwrap()
                .is_empty()
        );
    }

    /// test_recovery.mjs detectCrashCandidates: the three positive work_signal
    /// arms, each with the store state that produced it.
    #[test]
    fn crash_candidate_work_signal_arms() {
        let _guard = session_env_lock();
        let now = now_ms();
        let stale = to_iso(now - 1_000_000.0);
        let started = to_iso(now - 2_000_000.0);

        // (a) claimed_cells — a laneless session holding an ACTIVE claim.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sid = "sess-claims";
        let transcript = root.join("transcripts").join("t.jsonl");
        write_jsonl_file(&transcript, &dirty_end_events(now - 500_000.0));
        write_session_record(
            root,
            sid,
            &started,
            &stale,
            None,
            Some(&transcript.to_string_lossy()),
        );
        write_claim_record(root, "some-cell-1", sid, &to_iso(now - 60_000.0));
        let projects_root = root.join("projects").to_string_lossy().into_owned();
        let out = detect_crash_candidates(&mut ctx_for(root), &projects_root).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(vget(&out[0], "work_signal"), Some(&json!("claimed_cells")));
        assert_eq!(vget(&out[0], "lane"), Some(&Value::Null));
        // Control: expire that same claim (ttl already elapsed) and the arm
        // goes quiet — the transcript here is older than started_at is not,
        // so transcript_activity still carries it; assert the SIGNAL changed.
        write(
            root,
            ".bee/claims/some-cell-1.json",
            &serde_json::to_string(&json!({
                "cell": "some-cell-1", "session": sid, "ttl_seconds": 1,
                "claimed_at": to_iso(now - 60_000.0), "acquired_at": to_iso(now - 60_000.0),
            }))
            .unwrap(),
        );
        let out = detect_crash_candidates(&mut ctx_for(root), &projects_root).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(
            vget(&out[0], "work_signal"),
            Some(&json!("transcript_activity")),
            "an expired claim is not a work signal"
        );

        // (b) transcript_activity + the GLOBAL settlement window (D3): a
        // laneless session whose transcript moved after the last decision.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sid = "sess-laneless";
        let transcript = root.join("transcripts").join("t.jsonl");
        write_jsonl_file(&transcript, &dirty_end_events(now - 500_000.0));
        write_session_record(
            root,
            sid,
            &started,
            &stale,
            None,
            Some(&transcript.to_string_lossy()),
        );
        append_decision(root, "d1", &to_iso(now - 1_500_000.0));
        let projects_root = root.join("projects").to_string_lossy().into_owned();
        let out = detect_crash_candidates(&mut ctx_for(root), &projects_root).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(vget(&out[0], "work_signal"), Some(&json!("transcript_activity")));
        assert_eq!(
            vget(&out[0], "since"),
            Some(&json!(to_iso(now - 1_500_000.0))),
            "the candidate carries the global settlement it was measured against"
        );
        // Control: move the settlement PAST the transcript's last activity and
        // the arm stops firing — the comparison is real, not a constant.
        append_decision(root, "d2", &to_iso(now - 1000.0));
        assert!(
            detect_crash_candidates(&mut ctx_for(root), &projects_root)
                .unwrap()
                .is_empty()
        );
    }

    /// test_recovery.mjs: "zero stale sessions never touches the
    /// decisions/capture/cells stores". Node spies on fs; the Rust port is
    /// probed with a TRIPWIRE instead — a corrupt cell file whose READ is
    /// observable. CUTOVER: that read used to BAIL the snapshot, so the
    /// tripwire was an `Ex::Bail`; it now warns and falls back, so the
    /// tripwire is the warning line itself. Same proof, louder evidence.
    #[test]
    fn zero_stale_sessions_never_reads_the_settlement_stores() {
        let _guard = session_env_lock();
        let now = now_ms();
        let sid = "sess-fresh-only";

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let transcript = root.join("transcripts").join("t.jsonl");
        write_jsonl_file(&transcript, &dirty_end_events(now - 500_000.0));
        append_decision(root, "d1", &to_iso(now - 1_500_000.0));
        append_capture_stub(root, "c1", &to_iso(now - 1_500_000.0), None);
        // The tripwire: any read of the cells store warns about this file.
        write(root, ".bee/cells/tripwire.json", "{not json");
        let tripped = |ctx: &Ctx| -> bool {
            ctx.stderr.borrow().iter().any(|l| l.contains("tripwire.json"))
        };
        let projects_root = root.join("projects").to_string_lossy().into_owned();

        // Fresh heartbeat -> the fast path returns before the stores.
        write_session_record(
            root,
            sid,
            &to_iso(now - 2_000_000.0),
            &to_iso(now - 60_000.0),
            None,
            Some(&transcript.to_string_lossy()),
        );
        let mut fresh_ctx = ctx_for(root);
        assert!(
            detect_crash_candidates(&mut fresh_ctx, &projects_root).unwrap().is_empty(),
            "fresh heartbeat -> no candidates"
        );
        assert!(!tripped(&fresh_ctx), "fresh heartbeat -> the cells store is never read");

        // Control: the identical fixture with a STALE heartbeat reaches the
        // shared-store block and trips the wire.
        write_session_record(
            root,
            sid,
            &to_iso(now - 2_000_000.0),
            &to_iso(now - 1_000_000.0),
            None,
            Some(&transcript.to_string_lossy()),
        );
        let mut stale_ctx = ctx_for(root);
        assert!(
            detect_crash_candidates(&mut stale_ctx, &projects_root).is_ok(),
            "a corrupt cell no longer bails the snapshot"
        );
        assert!(
            tripped(&stale_ctx),
            "a stale session MUST reach the cells store — otherwise the fast-path assertion above proves nothing"
        );
    }

    /// test_recovery.mjs scanTranscriptRoots: the config arms, including the
    /// exactly-one-warning contract for a bad configured root.
    #[test]
    fn scan_transcript_roots_config_arms() {
        // (a) no config -> the Claude default root alone, and a MISSING
        // default root never warns (the pre-existing D2 silent no-op).
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let projects_root = root.join("projects").to_string_lossy().into_owned();
        let mut ctx = ctx_for(root);
        let roots = scan_transcript_roots(&mut ctx, &projects_root).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].runtime, "claude");
        assert_eq!(roots[0].path, projects_root);
        assert!(!roots[0].scanned);
        assert_eq!(roots[0].reason.as_deref(), Some("ENOENT"));
        assert!(ctx.stderr.borrow().is_empty(), "stderr was {:?}", ctx.stderr.borrow());

        // (b) a healthy configured root is scanned, silently.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let projects_root = root.join("projects");
        std::fs::create_dir_all(&projects_root).unwrap();
        let extra = root.join("codex-sessions");
        std::fs::create_dir_all(&extra).unwrap();
        write(
            root,
            ".bee/config.json",
            &serde_json::to_string(&json!({"recovery": {"transcript_roots": [
                {"runtime": "codex", "path": extra.to_string_lossy()}
            ]}}))
            .unwrap(),
        );
        let mut ctx = ctx_for(root);
        let roots =
            scan_transcript_roots(&mut ctx, &projects_root.to_string_lossy()).unwrap();
        assert_eq!(roots.len(), 2);
        assert!(roots[0].scanned);
        assert_eq!(roots[1].runtime, "codex");
        assert_eq!(roots[1].path, extra.to_string_lossy());
        assert!(roots[1].scanned);
        assert!(roots[1].reason.is_none());
        assert!(ctx.stderr.borrow().is_empty(), "stderr was {:?}", ctx.stderr.borrow());

        // (c) a missing CONFIGURED root degrades to scanned:false + reason,
        // with exactly one warning naming the offending path.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let projects_root = root.join("projects");
        std::fs::create_dir_all(&projects_root).unwrap();
        let missing = root.join("does-not-exist-codex-root");
        write(
            root,
            ".bee/config.json",
            &serde_json::to_string(&json!({"recovery": {"transcript_roots": [
                {"runtime": "codex", "path": missing.to_string_lossy()}
            ]}}))
            .unwrap(),
        );
        let mut ctx = ctx_for(root);
        let roots =
            scan_transcript_roots(&mut ctx, &projects_root.to_string_lossy()).unwrap();
        assert_eq!(roots.len(), 2);
        assert!(!roots[1].scanned);
        assert_eq!(roots[1].reason.as_deref(), Some("ENOENT"));
        assert_eq!(ctx.stderr.borrow().len(), 1, "stderr was {:?}", ctx.stderr.borrow());
        assert!(ctx.stderr.borrow()[0].contains(&*missing.to_string_lossy()));
        assert!(ctx.stderr.borrow()[0].contains("recovery.transcript_roots"));

        // (d) malformed entries (missing runtime/path, non-object junk) are
        // ignored silently — only the Claude default survives.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let projects_root = root.join("projects").to_string_lossy().into_owned();
        write(
            root,
            ".bee/config.json",
            &serde_json::to_string(&json!({"recovery": {"transcript_roots": [
                {"runtime": "codex"}, {"path": "/no-runtime"}, "just-a-string", 42, null
            ]}}))
            .unwrap(),
        );
        let mut ctx = ctx_for(root);
        let roots = scan_transcript_roots(&mut ctx, &projects_root).unwrap();
        assert_eq!(roots.len(), 1);
        assert!(ctx.stderr.borrow().is_empty(), "stderr was {:?}", ctx.stderr.borrow());
    }

    /// test_recovery.mjs: a configured extra-runtime root that PREFIXES the
    /// stored transcript path tags the candidate with that runtime.
    #[test]
    fn crash_candidate_is_tagged_with_the_runtime_whose_root_held_the_transcript() {
        let _guard = session_env_lock();
        let now = now_ms();
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sid = "sess-codex-crash";

        let codex_root = root.join("codex-sessions");
        let transcript = codex_root.join("2026").join("07").join(format!("rollout-{sid}.jsonl"));
        write_jsonl_file(&transcript, &dirty_end_events(now - 500_000.0));
        write_session_record(
            root,
            sid,
            &to_iso(now - 2_000_000.0),
            &to_iso(now - 1_000_000.0),
            Some("feat-codex"),
            Some(&transcript.to_string_lossy()),
        );
        write_lane_record(root, "feat-codex", "swarming");
        let projects_root = root.join("projects").to_string_lossy().into_owned();

        // Without the config the transcript still resolves (stored path), but
        // no scanned root prefixes it -> runtime null.
        let out = detect_crash_candidates(&mut ctx_for(root), &projects_root).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(vget(&out[0], "runtime"), Some(&Value::Null));

        // Declaring the root that actually holds it flips the tag.
        write(
            root,
            ".bee/config.json",
            &serde_json::to_string(&json!({"recovery": {"transcript_roots": [
                {"runtime": "codex", "path": codex_root.to_string_lossy()}
            ]}}))
            .unwrap(),
        );
        let out = detect_crash_candidates(&mut ctx_for(root), &projects_root).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(vget(&out[0], "runtime"), Some(&json!("codex")));
        assert_eq!(
            vget(&out[0], "transcript"),
            Some(&json!(transcript.to_string_lossy()))
        );
    }

    /// The cutover fix proved: an absolute root's encoded name is now a legal
    /// directory component on every host, drive letter included. Before the
    /// fix this assertion could not be made at all — the name kept the drive
    /// colon, NTFS took it as an alternate data stream on the parent, and
    /// `create_dir_all`/`Path::exists` both LIED about it (Ok / true) while the
    /// PARENT silently became a file. The only truthful probe enumerates the
    /// parent, which is exactly what this test does.
    #[test]
    fn encoded_project_dir_is_a_nameable_directory_component() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let encoded = encode_project_dir(&root.to_string_lossy());
        assert!(!encoded.contains(':'), "drive colon must be encoded away: {encoded}");
        let parent = tmp.path().join("projects");
        std::fs::create_dir_all(parent.join(&encoded)).unwrap();
        let listed = std::fs::read_dir(&parent)
            .unwrap()
            .filter_map(Result::ok)
            .any(|e| e.file_name().to_string_lossy() == encoded);
        assert!(listed, "encoded project dir must really exist: {encoded}");
    }

    /// A Windows absolute path encodes to the spelling Claude Code itself
    /// writes — `D--projects-…`, not Node's illegal `D:-projects-…`.
    #[test]
    fn encode_project_dir_matches_claude_codes_own_spelling() {
        assert_eq!(
            encode_project_dir("D:\\projects\\tools\\AI\\harness"),
            "D--projects-tools-AI-harness"
        );
        assert_eq!(encode_project_dir("/home/u/p.roj"), "-home-u-p-roj");
    }

    /// test_recovery.mjs: with NO stored transcript_path the resolver falls
    /// back to perf.mjs layout math — `<projectsRoot>/<encodeProjectDir(root)>/
    /// <sid>.jsonl` — and the candidate is tagged runtime "claude".
    ///
    /// Formerly capability-skipped on win32: `encodeProjectDir` mapped only
    /// [\\/.] to '-', so an absolute Windows root kept its drive colon and the
    /// encoded directory was unnameable on NTFS. The cutover fix encodes ':'
    /// too, so the case runs everywhere now.
    #[test]
    fn crash_candidate_resolves_through_the_encoded_layout_root() {
        let _guard = session_env_lock();
        let now = now_ms();
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sid = "sess-layout";

        let encoded_name = encode_project_dir(&root.to_string_lossy());
        let projects_root = root.join("projects");
        let encoded = projects_root.join(&encoded_name);
        std::fs::create_dir_all(&encoded).unwrap();
        write_jsonl_file(
            &encoded.join(format!("{sid}.jsonl")),
            &dirty_end_events(now - 500_000.0),
        );
        write_session_record(
            root,
            sid,
            &to_iso(now - 2_000_000.0),
            &to_iso(now - 1_000_000.0),
            Some("feat-layout"),
            None, // no stored path: force the layout math
        );
        write_lane_record(root, "feat-layout", "swarming");
        let out =
            detect_crash_candidates(&mut ctx_for(root), &projects_root.to_string_lossy())
                .unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(vget(&out[0], "runtime"), Some(&json!("claude")));
        assert_eq!(vget(&out[0], "work_signal"), Some(&json!("lane")));
        assert!(
            tpl(vget(&out[0], "transcript")).ends_with(&format!("{sid}.jsonl")),
            "resolved through the encoded layout directory"
        );
    }

    // ── contention summary (bee.mjs buildContentionSummary) ────────────────
    //
    // Ported from packages/bee/tests/test_contention_status.mjs.

    /// The minimal repo `status` needs: onboarding marker + phase.
    fn contention_root(root: &Path) {
        write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(root, ".bee/state.json", r#"{"phase":"idle"}"#);
    }

    fn contention_record(
        ts: &str,
        lock_name: &str,
        lock_wait_ms: i64,
        holder: Option<&str>,
        caller: Option<&str>,
        result: &str,
    ) -> String {
        serde_json::to_string(&json!({
            "ts": ts,
            "lock_name": lock_name,
            "lock_wait_ms": lock_wait_ms,
            "holder_session": holder.map(Value::from).unwrap_or(Value::Null),
            "caller_session": caller.map(Value::from).unwrap_or(Value::Null),
            "workflow_id": null, "workspace_id": null, "resource": null,
            "result": result,
        }))
        .unwrap()
    }

    /// test_contention_status.mjs (1)+(2): a seeded log produces the JSON
    /// aggregates and a text line; an absent log omits the key entirely.
    #[test]
    fn contention_summary_aggregates_a_seeded_log_and_is_absent_without_one() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        contention_root(root);
        write(
            root,
            ".bee/logs/contention.jsonl",
            &format!(
                "{}\n{}\n{}\n{}\n",
                contention_record("2026-07-24T10:00:00.000Z", "sessions", 50, Some("sess-a"), Some("sess-b"), "busy"),
                contention_record("2026-07-24T10:00:01.000Z", "sessions", 120, Some("sess-a"), Some("sess-c"), "busy"),
                contention_record("2026-07-24T10:00:02.000Z", "worktree-admin", 900, Some("sess-d"), Some("sess-e"), "busy"),
                // 'acquired' must never count as contention.
                contention_record("2026-07-24T10:00:03.000Z", "sessions", 0, None, Some("sess-f"), "acquired"),
            ),
        );
        let mut ctx = ctx_for(root);
        let status = build_status(&mut ctx, false).unwrap();
        let c = status.get("contention").expect("contention key");
        assert_eq!(vget(c, "busy_count"), Some(&json!(3)));
        assert_eq!(
            vget(c, "top_locks"),
            Some(&json!([
                {"lock_name": "sessions", "busy_count": 2},
                {"lock_name": "worktree-admin", "busy_count": 1}
            ]))
        );
        assert_eq!(vget(c, "worst_wait_ms"), Some(&json!(900)));
        assert_eq!(vget(c, "worst_wait_lock"), Some(&json!("worktree-admin")));
        let recent = vget(c, "recent_busy").and_then(|v| v.as_array()).unwrap();
        assert_eq!(recent.len(), 3, "the 'acquired' row is not in recent_busy");
        assert_eq!(
            recent[0],
            json!({
                "ts": "2026-07-24T10:00:02.000Z",
                "lock_name": "worktree-admin",
                "holder_session": "sess-d",
                "caller_session": "sess-e",
                "lock_wait_ms": 900
            }),
            "recent_busy is newest-first"
        );

        // The text renderer carries the data, not just the label.
        let line = render_status_text(&status)
            .lines()
            .find(|l| l.starts_with("Contention:"))
            .expect("a Contention line")
            .to_string();
        assert!(line.contains("3 LOCK_BUSY event(s)"), "{line}");
        assert!(line.contains("sessions×2"), "{line}");
        assert!(line.contains("worktree-admin×1"), "{line}");
        assert!(line.contains("900ms"), "{line}");
        assert!(line.contains("\"worktree-admin\""), "{line}");

        // Absent log -> the key is omitted entirely, and status still builds.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        contention_root(root);
        let mut ctx = ctx_for(root);
        let status = build_status(&mut ctx, false).unwrap();
        assert!(!status.contains_key("contention"));
        assert!(!render_status_text(&status).contains("Contention:"));
    }

    /// test_contention_status.mjs (3): malformed lines are skipped while
    /// well-formed busy events still count.
    #[test]
    fn contention_summary_skips_malformed_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        contention_root(root);
        write(
            root,
            ".bee/logs/contention.jsonl",
            &format!(
                "not json at all\n{}\n{{\"truncated\": tr\n",
                contention_record("2026-07-24T10:05:00.000Z", "claims", 42, Some("sess-x"), Some("sess-y"), "busy")
            ),
        );
        let ctx = ctx_for(root);
        let c = build_contention_summary(&ctx).unwrap().expect("summary");
        assert_eq!(c.get("busy_count"), Some(&json!(1)));
        assert_eq!(
            c.get("top_locks"),
            Some(&json!([{"lock_name": "claims", "busy_count": 1}]))
        );

        // A log with only non-busy rows yields no summary at all.
        write(
            root,
            ".bee/logs/contention.jsonl",
            &format!(
                "{}\n",
                contention_record("2026-07-24T10:05:00.000Z", "claims", 0, None, Some("s"), "acquired")
            ),
        );
        assert!(build_contention_summary(&ctx_for(root)).unwrap().is_none());
    }

    /// test_contention_status.mjs (4): the read is a bounded tail window.
    /// Stronger than the Node oracle — a well-formed busy record is placed
    /// BEFORE the window, so a full-file scan would report busy_count 2.
    #[test]
    fn contention_summary_reads_only_the_tail_window() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        contention_root(root);
        let head = contention_record(
            "2026-07-24T09:00:00.000Z",
            "head-lock",
            5000,
            Some("sess-old"),
            Some("sess-older"),
            "busy",
        );
        let tail = contention_record(
            "2026-07-24T10:10:00.000Z",
            "tail-lock",
            77,
            Some("sess-tail-holder"),
            Some("sess-tail-caller"),
            "busy",
        );
        let garbage = format!("garbage-not-json-{}\n", "x".repeat(200));
        let mut body = String::new();
        body.push_str(&head);
        body.push('\n');
        while body.len() < (CONTENTION_TAIL_MAX_BYTES as usize) * 4 {
            body.push_str(&garbage);
        }
        body.push_str(&tail);
        body.push('\n');
        write(root, ".bee/logs/contention.jsonl", &body);

        let c = build_contention_summary(&ctx_for(root)).unwrap().expect("summary");
        assert_eq!(
            c.get("busy_count"),
            Some(&json!(1)),
            "the head record sits outside the {CONTENTION_TAIL_MAX_BYTES}-byte window"
        );
        assert_eq!(
            c.get("top_locks"),
            Some(&json!([{"lock_name": "tail-lock", "busy_count": 1}]))
        );
        assert_eq!(c.get("worst_wait_ms"), Some(&json!(77)));
        assert_eq!(c.get("worst_wait_lock"), Some(&json!("tail-lock")));

        // Control: the same two records with no padding between them — both
        // are inside the window, so both count. This is what proves the
        // assertion above measured the window and not a parse failure.
        write(root, ".bee/logs/contention.jsonl", &format!("{head}\n{tail}\n"));
        let c = build_contention_summary(&ctx_for(root)).unwrap().expect("summary");
        assert_eq!(c.get("busy_count"), Some(&json!(2)));
        assert_eq!(c.get("worst_wait_ms"), Some(&json!(5000)));
        assert_eq!(c.get("worst_wait_lock"), Some(&json!("head-lock")));
    }

    // ── config validation (state.mjs validateModelsConfig / drift) ─────────
    //
    // Ported from scripts/tests/test_config_validate.mjs. The Rust `Problem`
    // has no `flag` field (Node's rows carry one), so flag-level assertions
    // read the rendered message instead.

    fn codes(problems: &[Problem]) -> Vec<&'static str> {
        problems.iter().map(|p| p.code).collect()
    }

    #[test]
    fn valid_models_configs_produce_zero_problems() {
        for config in [
            json!({"models": {"claude": {"extraction": "haiku", "generation": "sonnet", "review": "opus"}}}),
            json!({"models": {"claude": {"generation": {"model": "sonnet", "effort": "medium"}}}}),
            json!({"models": {"claude": {"generation": {"kind": "cli", "command": "codex exec --json -m gpt-5.3-codex -s read-only", "promptVia": "stdin"}}}}),
            json!({"models": {"claude": {"advisor": {"kind": "cli", "command": "codex exec -m gpt-5.6-sol -s read-only -", "promptVia": "stdin"}}}}),
            json!({"models": {"codex": {"generation": {"kind": "native", "model": "gpt-5.5", "effort": "high", "fork_turns": "none", "agent_type": "worker"}}}}),
            json!({"models": {"codex": {"advisor": {
                "primary": {"kind": "native", "model": "gpt-5.5", "effort": "high"},
                "fallback": {"kind": "cli", "command": "codex exec -m gpt-5.5 -s read-only -", "promptVia": "stdin"},
                "fallback_policy": "explicit-only"
            }}}}),
            json!({}),
            json!({"hooks": {}}),
        ] {
            let problems = validate_models_config(Some(&config));
            assert!(
                problems.is_empty(),
                "{config} produced {:?}",
                problems.iter().map(|p| (p.code, &p.message)).collect::<Vec<_>>()
            );
        }
        // No config file at all is normal, never a problem row.
        assert!(validate_models_config(None).is_empty());
    }

    #[test]
    fn cli_tier_problem_codes_fire_for_each_defect() {
        // cli-malformed: three ways to be cli-shaped and invalid.
        for bad in [
            json!({"command": "codex exec"}),           // missing kind:"cli"
            json!({"kind": "cli", "command": ""}),      // empty command
            json!({"kind": "cli"}),                     // no command at all
        ] {
            let problems =
                validate_models_config(Some(&json!({"models": {"claude": {"generation": bad}}})));
            assert!(codes(&problems).contains(&"cli-malformed"), "{:?}", codes(&problems));
        }

        // cli-prompt-transport-missing: a trailing "-" is a shell convention,
        // never a declared transport. Must NOT be read as cli-malformed.
        let problems = validate_models_config(Some(&json!({"models": {"claude": {
            "generation": {"kind": "cli", "command": "codex exec -m gpt-5 -s read-only -"}
        }}})));
        assert_eq!(codes(&problems), vec!["cli-prompt-transport-missing"]);

        // cli-unsafe-flag: every alias in the blocklist, individually.
        for flag in UNSAFE_CLI_FLAGS {
            let problems = validate_models_config(Some(&json!({"models": {"claude": {
                "generation": {"kind": "cli", "command": format!("some-cli exec {flag} --other-flag"), "promptVia": "stdin"}
            }}})));
            let unsafe_rows: Vec<&Problem> =
                problems.iter().filter(|p| p.code == "cli-unsafe-flag").collect();
            assert_eq!(unsafe_rows.len(), 1, "flag {flag}: {:?}", codes(&problems));
            assert!(unsafe_rows[0].message.contains(flag), "row must name {flag}");
            assert_eq!(unsafe_rows[0].runtime, Some("claude"));
            assert_eq!(unsafe_rows[0].slot, Some("generation"));
        }
        // Two aliases in one command -> one row each.
        let problems = validate_models_config(Some(&json!({"models": {"codex": {
            "review": {"kind": "cli", "command": format!("some-cli exec {} {}", UNSAFE_CLI_FLAGS[0], UNSAFE_CLI_FLAGS[1]), "promptVia": "stdin"}
        }}})));
        let unsafe_rows: Vec<&Problem> =
            problems.iter().filter(|p| p.code == "cli-unsafe-flag").collect();
        assert_eq!(unsafe_rows.len(), 2);
        assert!(unsafe_rows.iter().any(|p| p.message.contains(UNSAFE_CLI_FLAGS[0])));
        assert!(unsafe_rows.iter().any(|p| p.message.contains(UNSAFE_CLI_FLAGS[1])));

        // Advice-class (advisor/review) write-granting tokens.
        for token in ADVICE_CLASS_WRITABLE_TOKENS {
            let problems = validate_models_config(Some(&json!({"models": {"claude": {
                "advisor": {"kind": "cli", "command": format!("codex exec -m gpt-5 {token} -"), "promptVia": "stdin"}
            }}})));
            assert!(
                problems
                    .iter()
                    .any(|p| p.code == "cli-advice-slot-writable" && p.message.contains(token)),
                "token {token}: {:?}",
                codes(&problems)
            );
            // ...and the SAME token on a non-advice slot is clean. The
            // discriminator: "-s workspace-write" is on neither blocklist for
            // generation, so this is slot scoping, not a second code firing.
            let problems = validate_models_config(Some(&json!({"models": {"claude": {
                "generation": {"kind": "cli", "command": format!("codex exec -m gpt-5 {token} -"), "promptVia": "stdin"}
            }}})));
            assert!(!codes(&problems).contains(&"cli-advice-slot-writable"));
        }
        // danger-full-access on an advice slot trips BOTH blocklists.
        let problems = validate_models_config(Some(&json!({"models": {"claude": {
            "advisor": {"kind": "cli", "command": "codex exec -m gpt-5 -s danger-full-access -", "promptVia": "stdin"}
        }}})));
        assert!(codes(&problems).contains(&"cli-unsafe-flag"));
        assert!(codes(&problems).contains(&"cli-advice-slot-writable"));
    }

    #[test]
    fn native_and_composite_tier_problem_codes() {
        let p = |v: Value| validate_models_config(Some(&json!({"models": {"codex": {"advisor": v}}})));
        let g = |v: Value| validate_models_config(Some(&json!({"models": {"codex": {"generation": v}}})));

        // A native override with no model is native-model-missing, NOT
        // cli-malformed (the native branch runs before looksLikeCli).
        let problems = g(json!({"kind": "native"}));
        assert!(codes(&problems).contains(&"native-model-missing"));
        assert!(!codes(&problems).contains(&"cli-malformed"));
        // fork_turns other than "none".
        assert!(
            codes(&g(json!({"kind": "native", "model": "gpt-5.5", "fork_turns": "full"})))
                .contains(&"native-fork-turns-unknown")
        );
        // Composite with no fallback_policy (silent native->cli is forbidden).
        assert!(
            codes(&p(json!({
                "primary": {"kind": "native", "model": "gpt-5.5"},
                "fallback": {"kind": "cli", "command": "codex exec -m gpt-5.5 -s read-only -"}
            })))
            .contains(&"composite-fallback-policy-missing")
        );
        // Composite whose primary is not a native override.
        assert!(
            codes(&p(json!({
                "primary": {"model": "gpt-5.5"},
                "fallback": {"kind": "cli", "command": "x"},
                "fallback_policy": "explicit-only"
            })))
            .contains(&"composite-primary-malformed")
        );
        // Composite whose cli fallback is malformed.
        assert!(
            codes(&p(json!({
                "primary": {"kind": "native", "model": "gpt-5.5"},
                "fallback": {"kind": "cli"},
                "fallback_policy": "explicit-only"
            })))
            .contains(&"composite-fallback-malformed")
        );
    }

    #[test]
    fn malformed_config_input_reports_rows_instead_of_throwing() {
        for bad in [
            Value::Null,
            json!("a string"),
            json!(42),
            json!(true),
            json!(["array", "config"]),
        ] {
            assert!(
                codes(&validate_models_config(Some(&bad))).contains(&"config-malformed"),
                "{bad}"
            );
        }
        // `models` of the wrong type, and a runtime of the wrong type.
        assert!(
            codes(&validate_models_config(Some(&json!({"models": "not-an-object"}))))
                .contains(&"config-malformed")
        );
        assert!(
            codes(&validate_models_config(Some(
                &json!({"models": {"claude": "not-an-object"}})
            )))
            .contains(&"runtime-malformed")
        );
        // The discriminator between "no file yet" and "file is null".
        assert!(validate_models_config(None).is_empty());
    }

    fn write_agent_file(root: &Path, agent: &str, frontmatter: &str) {
        write(
            root,
            &format!(".claude/agents/{agent}.md"),
            &format!("---\n{frontmatter}\n---\n\nBody text, not parsed by the drift check.\n"),
        );
    }

    /// scripts/tests/test_config_validate.mjs validateAgentFilesDrift.
    #[test]
    fn agent_file_drift_findings() {
        // (a) a rendered file whose model no longer matches the tier.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_agent_file(root, "bee-gather", "name: bee-gather\nmodel: opus");
        let cfg = json!({"models": {"claude": {"generation": "sonnet"}}});
        let problems = validate_agent_files_drift(&ctx_for(root), Some(&cfg));
        assert_eq!(codes(&problems), vec!["agent-file-drift"]);
        assert_eq!(problems[0].agent, Some("bee-gather"));
        assert_eq!(problems[0].slot, Some("generation"));
        assert!(problems[0].message.contains("model: \"opus\""));
        assert!(problems[0].message.contains("is \"sonnet\""));

        // (b) matching files across all three agents -> clean.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_agent_file(root, "bee-gather", "name: bee-gather\nmodel: sonnet");
        write_agent_file(root, "bee-extract", "name: bee-extract\nmodel: haiku");
        write_agent_file(root, "bee-review", "name: bee-review\nmodel: opus");
        let cfg =
            json!({"models": {"claude": {"generation": "sonnet", "extraction": "haiku", "review": "opus"}}});
        assert!(validate_agent_files_drift(&ctx_for(root), Some(&cfg)).is_empty());

        // (c) no agent files at all -> absent is clean.
        let tmp = tempfile::tempdir().unwrap();
        let cfg = json!({"models": {"claude": {"generation": "sonnet"}}});
        assert!(validate_agent_files_drift(&ctx_for(tmp.path()), Some(&cfg)).is_empty());

        // (d) a stale file under a now cli-shaped slot is flagged, never
        // silently accepted.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_agent_file(root, "bee-gather", "name: bee-gather\nmodel: sonnet");
        let cfg = json!({"models": {"claude": {"generation": {"kind": "cli", "command": "codex exec -m gpt-5.5 -s read-only -"}}}});
        let problems = validate_agent_files_drift(&ctx_for(root), Some(&cfg));
        assert_eq!(codes(&problems), vec!["agent-file-drift"]);
        assert!(problems[0].message.contains("cli-shaped or unconfigured"));

        // (e) unparseable frontmatter is its own code, never a throw.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".claude/agents/bee-extract.md", "not even frontmatter, just plain text\n");
        let cfg = json!({"models": {"claude": {"extraction": "haiku"}}});
        let problems = validate_agent_files_drift(&ctx_for(root), Some(&cfg));
        assert_eq!(codes(&problems), vec!["agent-file-malformed"]);
        assert_eq!(problems[0].agent, Some("bee-extract"));

        // (f) an explicitly null review slot falls back to generation
        // (decision 0021), mirroring resolveTier.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_agent_file(root, "bee-review", "name: bee-review\nmodel: sonnet");
        let cfg = json!({"models": {"claude": {"generation": "sonnet", "review": null}}});
        assert!(validate_agent_files_drift(&ctx_for(root), Some(&cfg)).is_empty());
        // Control: the same null-review config with a file declaring the
        // seeded review default drifts, proving the fallback really moved the
        // expectation to generation.
        write_agent_file(root, "bee-review", "name: bee-review\nmodel: opus");
        assert_eq!(
            codes(&validate_agent_files_drift(&ctx_for(root), Some(&cfg))),
            vec!["agent-file-drift"]
        );

        // (g) no config on disk at all -> resolves against the seeded
        // defaults (generation=sonnet), no throw.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_agent_file(root, "bee-gather", "name: bee-gather\nmodel: sonnet");
        assert!(validate_agent_files_drift(&ctx_for(root), None).is_empty());
    }

    fn write_opencode_agent_file(root: &Path, agent: &str, frontmatter: &str) {
        write(
            root,
            &format!(".opencode/agent/{agent}.md"),
            &format!("---\n{frontmatter}\n---\n\nBody text, not parsed by the drift check.\n"),
        );
    }

    /// opencode-support oc-13/oc-14: `.opencode/agent/` joins `.claude/agents/`
    /// in the same drift check, checked against `models.opencode` instead of
    /// `models.claude`. Before oc-14 the two roots needed different
    /// unconfigured-slot wording — opencode's files were hand-authored (oc-11),
    /// so "re-run onboarding" would have been a promise bee could not keep.
    /// oc-14 renders `.opencode/agent/` the same way `.claude/agents/` is
    /// rendered, so both roots now share one verdict shape (see the drift
    /// check's own doc comment).
    #[test]
    fn opencode_agent_file_drift_findings() {
        // (a) no models.opencode config at all -> clean: the file's declared
        // model matches the built-in default (AGENT_TIER_DEFAULTS_OPENCODE),
        // same reasoning that already applied to claude's haiku/sonnet/opus.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_opencode_agent_file(root, "bee-gather", "model: opencode/big-pickle");
        assert!(validate_agent_files_drift(&ctx_for(root), None).is_empty());
        assert!(
            validate_agent_files_drift(&ctx_for(root), Some(&json!({"models": {}}))).is_empty()
        );

        // (b) a REAL configured mismatch is still caught, worded the same as
        // the claude side now — the file IS onboarding-rendered (oc-14).
        let cfg = json!({"models": {"opencode": {"generation": "opencode/deepseek-v4-flash-free"}}});
        let problems = validate_agent_files_drift(&ctx_for(root), Some(&cfg));
        assert_eq!(codes(&problems), vec!["agent-file-drift"]);
        assert_eq!(problems[0].agent, Some("bee-gather"));
        assert!(problems[0].message.contains("model: \"opencode/big-pickle\""));
        assert!(problems[0].message.contains("is \"opencode/deepseek-v4-flash-free\""));
        assert!(problems[0].message.contains("re-run onboarding to re-render it"));
        assert!(!problems[0].message.contains("hand-authored"));

        // (c) matching config across all three opencode agents -> clean.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_opencode_agent_file(root, "bee-gather", "model: opencode/big-pickle");
        write_opencode_agent_file(root, "bee-extract", "model: opencode/ling-3.0-tiny-free");
        write_opencode_agent_file(root, "bee-review", "model: opencode/nemotron-3-ultra-free");
        let cfg = json!({"models": {"opencode": {
            "generation": "opencode/big-pickle",
            "extraction": "opencode/ling-3.0-tiny-free",
            "review": "opencode/nemotron-3-ultra-free"
        }}});
        assert!(validate_agent_files_drift(&ctx_for(root), Some(&cfg)).is_empty());

        // (d) claude and opencode roots are checked independently — a claude
        // drift never contaminates a clean opencode root and vice versa.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_agent_file(root, "bee-gather", "name: bee-gather\nmodel: opus");
        write_opencode_agent_file(root, "bee-gather", "model: opencode/big-pickle");
        let cfg = json!({"models": {
            "claude": {"generation": "sonnet"},
            "opencode": {"generation": "opencode/big-pickle"}
        }});
        let problems = validate_agent_files_drift(&ctx_for(root), Some(&cfg));
        assert_eq!(codes(&problems), vec!["agent-file-drift"]);
        assert!(problems[0].message.starts_with(".claude/agents/bee-gather.md"));

        // (e) a stale opencode file under a now cli-shaped slot is flagged
        // too — the same "this file should not exist" verdict claude's (d)
        // case proves, now symmetric across both rendered roots.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_opencode_agent_file(root, "bee-gather", "model: opencode/big-pickle");
        let cfg = json!({"models": {"opencode": {"generation": {"kind": "cli", "command": "opencode run -"}}}});
        let problems = validate_agent_files_drift(&ctx_for(root), Some(&cfg));
        assert_eq!(codes(&problems), vec!["agent-file-drift"]);
        assert!(problems[0].message.contains("cli-shaped or unconfigured"));
    }

    /// The point of the cell (scripts/tests/test_config_validate.mjs header):
    /// a malformed cli tier is LOUD. `normalizeTierValue` drops it and the
    /// seeded default silently survives — the only thing that tells anyone is
    /// the staleness warning `status` emits.
    #[test]
    fn a_malformed_cli_tier_is_loud_not_silently_reverted() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(root, ".bee/state.json", r#"{"phase":"idle"}"#);
        // kind:"cli" missing -> normalizeTierValue returns undefined.
        write(
            root,
            ".bee/config.json",
            r#"{"commands":{"test":"t"},"models":{"claude":{"generation":{"command":"codex exec"}}}}"#,
        );
        let mut ctx = ctx_for(root);
        let status = build_status(&mut ctx, false).unwrap();
        // The silent revert really happened...
        assert_eq!(
            status.get("models").and_then(|m| vget(m, "claude")).and_then(|c| vget(c, "generation")),
            Some(&json!("sonnet")),
            "the seeded default survived — this is exactly what the warning exists for"
        );
        // ...and it was announced.
        let warnings: Vec<String> = status
            .get("staleness_warnings")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .map(|w| tpl(Some(w)))
            .collect();
        let hit = warnings
            .iter()
            .find(|w| w.starts_with("config validate [cli-malformed]"))
            .unwrap_or_else(|| panic!("no cli-malformed warning in {warnings:?}"));
        assert!(hit.contains(" models.claude.generation:"), "{hit}");
        assert!(hit.contains("silently reverts to the seeded default"), "{hit}");

        // Control: the same tier, now well-formed, is adopted AND silent.
        write(
            root,
            ".bee/config.json",
            r#"{"commands":{"test":"t"},"models":{"claude":{"generation":{"kind":"cli","command":"codex exec","promptVia":"stdin"}}}}"#,
        );
        let mut ctx = ctx_for(root);
        let status = build_status(&mut ctx, false).unwrap();
        assert_eq!(
            status.get("models").and_then(|m| vget(m, "claude")).and_then(|c| vget(c, "generation")),
            Some(&json!({"kind": "cli", "command": "codex exec"}))
        );
        let warnings = status.get("staleness_warnings").and_then(|v| v.as_array()).unwrap();
        assert!(
            !warnings.iter().any(|w| tpl(Some(w)).starts_with("config validate")),
            "{warnings:?}"
        );
    }

    // ── ship_visibility (state.mjs shipVisibility) ─────────────────────────
    //
    // Ported from scripts/tests/test_ship_visibility.mjs (the status --json
    // half; the buildSessionPreamble half lives in inject.mjs, not here).

    #[test]
    fn ship_visibility_surfaces_draft_pr_and_warns_once_on_an_unrecognized_value() {
        let build = |config: Option<&str>| -> (JMap, Vec<String>) {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path();
            write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
            write(root, ".bee/state.json", r#"{"phase":"idle"}"#);
            if let Some(c) = config {
                write(root, ".bee/config.json", c);
            }
            let mut ctx = ctx_for(root);
            let status = build_status(&mut ctx, false).unwrap();
            (status, ctx.stderr.borrow().clone())
        };

        // draft-pr survives; nothing is written to stderr about it.
        let (status, stderr) = build(Some(r#"{"ship_visibility":"draft-pr"}"#));
        assert_eq!(status.get("ship_visibility"), Some(&json!("draft-pr")));
        assert!(!stderr.iter().any(|l| l.contains("ship_visibility")), "{stderr:?}");

        // An unrecognized value normalizes to "off" AND says so, once, by name.
        let (status, stderr) = build(Some(r#"{"ship_visibility":"launch-the-rocket"}"#));
        assert_eq!(status.get("ship_visibility"), Some(&json!("off")));
        let warns: Vec<&String> =
            stderr.iter().filter(|l| l.contains("ship_visibility")).collect();
        assert_eq!(warns.len(), 1, "{stderr:?}");
        assert_eq!(
            warns[0],
            "config: unrecognized ship_visibility \"launch-the-rocket\" in .bee/config.json — normalized to \"off\". Allowed: off, draft-pr."
        );

        // Explicit "off" and an absent key are the same silent shape.
        for config in [Some(r#"{"ship_visibility":"off"}"#), Some("{}"), None] {
            let (status, stderr) = build(config);
            assert_eq!(status.get("ship_visibility"), Some(&json!("off")), "{config:?}");
            assert!(
                !stderr.iter().any(|l| l.contains("ship_visibility")),
                "{config:?} -> {stderr:?}"
            );
        }
    }

    // ── the review block's cross-run git cache ─────────────────────────────
    //
    // The port kept `derive_candidate_status`'s in-process memo and dropped
    // the on-disk one Node wrote, so every `status` / `orient` re-spawned the
    // same `merge-base` and `rev-list` queries. Measured on the bee repo (89
    // candidates, win32): orient 850ms with candidates, 280ms without — two
    // thirds of the session-start path spent re-asking answered questions.
    //
    // These four pin the properties that make replaying those answers safe.
    // Each POISONS the cache with an answer the git fixture contradicts, so a
    // pass that ignored the cache and a pass that honoured it cannot report
    // the same counts — the alternative (asserting two identical runs agree)
    // is green whether or not a single byte is ever read back.

    /// A candidate whose head is an ancestor of an approved session's head,
    /// at HEAD. Derives to `reviewed` with no cache in play.
    fn review_git_fixture(tmp: &Path) -> (PathBuf, String, String) {
        let root = tmp.join("repo");
        std::fs::create_dir_all(&root).unwrap();
        write(&root, ".bee/onboarding.json", "{}");
        write(&root, "f.txt", "one");
        git(&root, &["init", "-q", "-b", "main", "."]);
        git(&root, &["config", "user.email", "a@b.c"]);
        git(&root, &["config", "user.name", "t"]);
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-qm", "one"]);
        let first = git_out(&root, &["rev-parse", "HEAD"]);
        write(&root, "f.txt", "two");
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-qm", "two"]);
        let second = git_out(&root, &["rev-parse", "HEAD"]);

        write(
            &root,
            ".bee/review-candidates.jsonl",
            &format!(
                "{}\n",
                json!({
                    "id": "cand-1", "type": "candidate", "feature": "f",
                    "head": first, "mode": "standard", "cells": ["c1"],
                })
            ),
        );
        write(
            &root,
            ".bee/reviews/s1.json",
            &json!({
                "id": "s1",
                "head": second,
                "included": [{ "type": "cell", "id": "c1" }],
                "decision": { "status": "approved" },
            })
            .to_string(),
        );
        (root, first, second)
    }

    fn git_out(cwd: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git must be on PATH for the review-cache fixtures");
        assert!(out.status.success(), "git {args:?} failed");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn poisoned_cache(head: &str, candidate_head: &str, session_head: &str) -> Value {
        json!({
            "schema": "review-git-cache/1",
            "head": head,
            "covered": {},
            // The fixture's git says TRUE. A pass that reads this file reports
            // `unreviewed`; a pass that re-derives reports `reviewed`.
            "covered_gen": {
                format!("{candidate_head} {session_head}"): {
                    "covered": false, "unresolved": false,
                },
            },
            "since": {},
        })
    }

    fn review_counts(root: &Path) -> Value {
        let mut ctx = ctx_for(root);
        let block = build_review_block(&mut ctx).expect("the review block never bails here");
        Value::Object(block)["candidates"].clone()
    }

    #[test]
    fn a_cold_review_pass_derives_from_git_and_leaves_a_cache_at_head() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, first, second) = review_git_fixture(tmp.path());
        let head = git_out(&root, &["rev-parse", "HEAD"]);

        assert_eq!(review_counts(&root)["reviewed"], json!(1));

        let cached = match crate::fsutil::read_json(&git_cache_path(&root)) {
            crate::fsutil::ReadJson::Parsed(v) => v,
            crate::fsutil::ReadJson::Missing => panic!("the pass must leave a cache behind"),
            crate::fsutil::ReadJson::Corrupt => panic!("the cache it leaves must be parseable"),
        };
        assert_eq!(cached["schema"], json!("review-git-cache/1"));
        assert_eq!(cached["head"], json!(head), "the cache is keyed on HEAD");
        assert_eq!(
            cached["covered_gen"][format!("{first} {second}")]["covered"],
            json!(true),
            "the answer git actually gave must be what is written down"
        );
    }

    #[test]
    fn a_warm_cache_at_the_same_head_is_replayed_instead_of_re_spawning_git() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, first, second) = review_git_fixture(tmp.path());
        let head = git_out(&root, &["rev-parse", "HEAD"]);
        crate::fsutil::write_json_atomic(
            &git_cache_path(&root),
            &poisoned_cache(&head, &first, &second),
        )
        .unwrap();

        // If the cache were decorative this would still be `reviewed`.
        let counts = review_counts(&root);
        assert_eq!(counts["unreviewed"], json!(1), "the cached answer must win");
        assert_eq!(counts["reviewed"], json!(0));
    }

    #[test]
    fn a_cache_written_at_another_head_is_discarded_rather_than_replayed() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, first, second) = review_git_fixture(tmp.path());
        // `commits_since` counts `<ref>..HEAD`, so an answer set derived at a
        // different HEAD is not merely old, it can be wrong. Same file, one
        // field changed — the only thing under test is the HEAD guard.
        crate::fsutil::write_json_atomic(
            &git_cache_path(&root),
            &poisoned_cache(&"0".repeat(40), &first, &second),
        )
        .unwrap();

        assert_eq!(
            review_counts(&root)["reviewed"],
            json!(1),
            "a foreign-HEAD cache must be re-derived, not trusted"
        );
    }

    #[test]
    fn a_corrupt_or_hand_edited_cache_costs_a_slower_pass_and_nothing_else() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, first, second) = review_git_fixture(tmp.path());
        let head = git_out(&root, &["rev-parse", "HEAD"]);

        // Unparseable.
        std::fs::create_dir_all(git_cache_path(&root).parent().unwrap()).unwrap();
        std::fs::write(git_cache_path(&root), b"{not json").unwrap();
        assert_eq!(review_counts(&root)["reviewed"], json!(1));

        // Parseable, but an entry whose shape this reader does not recognise
        // is skipped rather than coerced into an answer.
        let mut bad = poisoned_cache(&head, &first, &second);
        bad["covered_gen"][format!("{first} {second}")]["covered"] = json!("nope");
        crate::fsutil::write_json_atomic(&git_cache_path(&root), &bad).unwrap();
        assert_eq!(review_counts(&root)["reviewed"], json!(1));
    }

    // ── debt-door-archive dda-2: the parity test the feature exists for ────
    //
    // Four independent copies of "walk the store, count unpaid
    // behavior_change debt" live in this tree (drivers::scribing_debt behind
    // `bee close`'s door, this module's own scribing_debt/global_scribing_debt
    // behind `bee status`, hooks::chain_nudge::scribing_debt behind the
    // chain-nudge hook line, and hooks::session_preamble::scribing_debt/
    // global_scribing_debt behind the session-start line). dda-1 made the
    // first archive-aware; dda-2 made the rest. This fixture is the one
    // place all four are driven together, over the exact shape the whole
    // feature is about: a hot cell, an archived cell, and one id that exists
    // in BOTH places (live-copy-wins dedup must still count it once).

    /// A single feature with a hot capped `behavior_change` cell, an
    /// archived one, and a duplicate id present in both the hot store and
    /// the archive slot.
    fn parity_fixture(base: &Path) -> (PathBuf, &'static str) {
        let root = base.join("repo");
        let feature = "parity-feat";
        std::fs::create_dir_all(root.join(".bee/cells/archive").join(feature)).unwrap();
        std::fs::create_dir_all(root.join(".bee/logs")).unwrap();
        std::fs::write(
            root.join(".bee/onboarding.json"),
            format!(r#"{{"bee_version":"{BEE_VERSION}","completed":true}}"#),
        )
        .unwrap();
        std::fs::write(
            root.join(".bee/state.json"),
            format!(r#"{{"phase":"executing","feature":"{feature}","gates":{{}}}}"#),
        )
        .unwrap();
        let cell = |rel: &str, id: &str| {
            std::fs::write(
                root.join(rel),
                format!(
                    r#"{{"id":"{id}","feature":"{feature}","status":"capped","title":"t",
                    "trace":{{"behavior_change":true,"capped_at":"2024-01-01T00:00:00Z"}}}}"#
                ),
            )
            .unwrap();
        };
        // Hot only.
        cell(".bee/cells/hot-1.json", "hot-1");
        // Archived only — the case `list_cells`'s active-only scan cannot see.
        cell(&format!(".bee/cells/archive/{feature}/arch-1.json"), "arch-1");
        // Present in BOTH: the live copy must win and the id must count once.
        cell(".bee/cells/dup-1.json", "dup-1");
        cell(&format!(".bee/cells/archive/{feature}/dup-1.json"), "dup-1");
        (root, feature)
    }

    #[test]
    fn the_four_scribing_debt_counters_agree_over_hot_archived_and_duplicate_cells() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, feature) = parity_fixture(tmp.path());
        let want_count = 3usize;
        let want_ids = |ids: Vec<String>| {
            let mut ids = ids;
            ids.sort();
            assert_eq!(ids, vec!["arch-1", "dup-1", "hot-1"], "ids: {ids:?}");
        };

        // 1. `bee close`'s door (verbs::drivers::scribing_debt, dda-1).
        let close_debt = crate::verbs::drivers::scribing_debt(&root, feature).unwrap();
        assert_eq!(close_debt.count, want_count, "drivers::scribing_debt");
        want_ids(close_debt.ids.iter().map(|v| jsjson::js_to_string(v)).collect());

        // 2. `bee status`'s per-feature debt (this module).
        let mut ctx = ctx_for(&root);
        let status_debt = scribing_debt(&mut ctx).unwrap();
        assert_eq!(status_debt["count"], json!(want_count), "status_full::scribing_debt");
        want_ids(
            status_debt["cells"]
                .as_array()
                .unwrap()
                .iter()
                .map(jsjson::js_to_string)
                .collect(),
        );

        // 3. the chain-nudge hook line (hooks::chain_nudge::scribing_debt).
        let (chain_count, chain_ids) = crate::hooks::chain_nudge::scribing_debt(&root).unwrap();
        assert_eq!(chain_count, want_count, "chain_nudge::scribing_debt");
        want_ids(chain_ids);

        // 4. the session-start debt line (hooks::session_preamble).
        let (preamble_count, preamble_ids) = crate::hooks::session_preamble::scribing_debt(&root);
        assert_eq!(preamble_count, want_count, "session_preamble::scribing_debt");
        want_ids(preamble_ids.iter().map(jsjson::js_to_string).collect());
    }

    /// The global/orphan sweeps (`bee status`'s and the session-preamble
    /// hook's) must see the same archived+duplicated cells as their
    /// per-feature siblings above — an orphaned feature's cells are exactly
    /// the ones most likely to already be archived.
    #[test]
    fn the_two_global_orphan_sweeps_agree_over_hot_archived_and_duplicate_cells() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, feature) = parity_fixture(tmp.path());

        let mut ctx = ctx_for(&root);
        let status_global = global_scribing_debt(&mut ctx).unwrap();
        assert_eq!(status_global["count"], json!(3), "status_full::global_scribing_debt");
        let status_ids: Vec<String> = status_global["features"][0]["cells"]
            .as_array()
            .unwrap()
            .iter()
            .map(jsjson::js_to_string)
            .collect();
        let mut status_ids_sorted = status_ids.clone();
        status_ids_sorted.sort();
        assert_eq!(status_ids_sorted, vec!["arch-1", "dup-1", "hot-1"]);
        assert_eq!(status_global["features"][0]["feature"], json!(feature));

        let (preamble_count, preamble_features) =
            crate::hooks::session_preamble::global_scribing_debt(&root);
        assert_eq!(preamble_count, 3, "session_preamble::global_scribing_debt");
        assert_eq!(preamble_features.len(), 1);
        assert_eq!(preamble_features[0].0, feature);
        let mut preamble_ids: Vec<String> =
            preamble_features[0].1.iter().map(jsjson::js_to_string).collect();
        preamble_ids.sort();
        assert_eq!(preamble_ids, vec!["arch-1", "dup-1", "hot-1"]);
    }
