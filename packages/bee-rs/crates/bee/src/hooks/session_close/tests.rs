// Split out of the single 2.8k-line hooks/session_close.rs. Code unchanged; only module placement and item visibility moved.
//
// Moved verbatim out of the parent file's inline module, indentation
// and all: a moved inline module is the same child of the same parent,
// so no path changes, and the fixtures inside are raw strings whose
// leading whitespace is content.

// The parent module's own `use` block travels with the tests: they reach
// for names mod.rs no longer imports now that the code using them lives
// in sibling modules.
#![allow(unused_imports)]

use crate::fsutil::{read_json, ReadJson};
use crate::hooks::adapter::{emit_hook_output, encode_block, log_crash, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson::{self, js_to_string};
use crate::state::{bypass_level, read_config_raw};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
    use super::*;
    use serde_json::json;

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let lib = root.join(".bee").join("bin").join("lib");
        std::fs::create_dir_all(&lib).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        for name in ["state.mjs", "inject.mjs", "decisions.mjs", "capture.mjs", "knowledge.mjs", "cells.mjs", "reservations.mjs"] {
            std::fs::write(lib.join(name), "// stub\n").unwrap();
        }
        dir
    }

    fn write_json_file(path: &Path, v: &Value) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, format!("{}\n", serde_json::to_string_pretty(v).unwrap())).unwrap();
    }

    fn run_stop(root: &Path, extra: Value) -> Result<(String, Vec<String>, String), ()> {
        // Runs the advisory pipeline the way run_inner does (skipping the perf
        // refresh so tests never touch the machine-global perf store), and
        // returns (stdout, parts, stderr).
        let mut body = json!({"hook_event_name": "Stop", "cwd": root.to_string_lossy()});
        if let Value::Object(m) = extra {
            for (k, v) in m {
                body[k.as_str()] = v;
            }
        }
        let stdin = serde_json::to_string(&body).unwrap();
        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        let root = ctx.root.clone().expect("fixture root resolves");
        let session_id = get_session_id(&ctx.payload);
        clear_corrupt_json_warnings();
        let config = preflight(&root)?;
        let mut parts = Vec::new();
        let mut stderr = String::new();
        let mut stdout = String::new();
        match advisory(&root, &ctx, &config, session_id.as_deref(), &mut parts, &mut stderr) {
            Ok(AdvisoryOutcome::Block(reason)) => stdout = encode_block(&reason),
            Ok(_) => {}
            Err(Flow::Delegate) => return Err(()),
            Err(Flow::Crash(_)) => {}
        }
        // flush() writes the queued corrupt-JSON warnings ahead of `stderr`;
        // tests read them from the same string.
        Ok((stdout, parts, format!("{}{stderr}", take_corrupt_json_warnings())))
    }

    #[test]
    fn bypass_net_blocks_planning_once_then_steps_aside() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({"gate_bypass": "total"}));
        write_json_file(
            &root.join(".bee").join("state.json"),
            &json!({"phase": "planning", "mode": "standard", "approved_gates": {"execution": false}}),
        );
        let (stdout, parts, _) = run_stop(root, json!({"session_id": "s-1"})).unwrap();
        assert!(stdout.starts_with("{\"decision\":\"block\",\"reason\":\"⚡ GATE BYPASS (total): "));
        assert!(stdout.contains("mid-planning with Gate 2 (shape+execution) still pending"));
        // The net prescribes the MERGED approval, not the standalone --name
        // path: Gate 2 flips `shape` and `execution` together, so a net that
        // set only execution would leave the gate it just "approved" half open.
        assert!(stdout.contains("state gate --merge --approved true"));
        assert!(!stdout.contains("--name execution"));
        assert!(!stdout.contains("High-risk execution requires"));
        assert!(parts.is_empty());
        // loop-guard: the same (session, phase, gate, level) key degrades to advisory
        let (stdout2, _, _) = run_stop(root, json!({"session_id": "s-1"})).unwrap();
        assert_eq!(stdout2, "");
    }

    /// Gate 2 has passed only when BOTH of its components are true. A record
    /// carrying just one of them is a half-open merged gate, and the net must
    /// still fire on it — otherwise the standalone `--name` path is a hole
    /// straight through the bypass net.
    #[test]
    fn bypass_net_fires_on_a_half_open_merged_gate_and_stands_down_on_a_whole_one() {
        for half in [
            json!({"shape": true, "execution": false}),
            json!({"shape": false, "execution": true}),
        ] {
            let fx = fixture();
            let root = fx.path();
            write_json_file(&root.join(".bee").join("config.json"), &json!({"gate_bypass": "total"}));
            write_json_file(
                &root.join(".bee").join("state.json"),
                &json!({"phase": "planning", "mode": "standard", "approved_gates": half}),
            );
            let (stdout, _, _) = run_stop(root, json!({"session_id": "s-1"})).unwrap();
            assert!(
                stdout.contains("state gate --merge --approved true"),
                "half-open gate {half} did not fire the net: {stdout}"
            );
        }

        // Both components granted: the gate is whole, so the net stands down.
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({"gate_bypass": "total"}));
        write_json_file(
            &root.join(".bee").join("state.json"),
            &json!({"phase": "planning", "mode": "standard", "approved_gates": {"shape": true, "execution": true}}),
        );
        let (stdout, _, _) = run_stop(root, json!({"session_id": "s-1"})).unwrap();
        assert_eq!(stdout, "");
    }

    #[test]
    fn bypass_net_high_risk_consult_sentence_and_mode_floor() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({"gate_bypass": "full"}));
        write_json_file(
            &root.join(".bee").join("state.json"),
            &json!({"phase": "validating", "mode": "high-risk", "approved_gates": {}}),
        );
        let (stdout, _, _) = run_stop(root, json!({})).unwrap();
        // legacy 'validating' coerces to planning; full covers high-risk
        assert!(stdout.contains("High-risk execution requires a live advisor consult first"));
        // normal does NOT cover high-risk
        let fx2 = fixture();
        let root2 = fx2.path();
        write_json_file(&root2.join(".bee").join("config.json"), &json!({"gate_bypass": true}));
        write_json_file(
            &root2.join(".bee").join("state.json"),
            &json!({"phase": "planning", "mode": "high-risk", "approved_gates": {}}),
        );
        let (stdout2, _, _) = run_stop(root2, json!({})).unwrap();
        assert_eq!(stdout2, "");
    }

    #[test]
    fn mid_phase_warning_lists_cells_and_reservations() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "swarming"}));
        write_json_file(
            &root.join(".bee").join("cells").join("b.json"),
            &json!({"id": "w-10", "status": "claimed", "trace": {"worker": "worker-b"}}),
        );
        write_json_file(
            &root.join(".bee").join("cells").join("a.json"),
            &json!({"id": "w-2", "status": "claimed"}),
        );
        write_json_file(
            &root.join(".bee").join("cells").join("c.json"),
            &json!({"id": "w-3", "status": "capped"}),
        );
        write_json_file(
            &root.join(".bee").join("runtime").join("leases").join("paths").join("h1.json"),
            &json!({"resource": "path:src/api", "workflow_id": "w-2", "workspace_id": "agent:alpha", "acquired_at": "2026-01-01T00:00:00.000Z", "expires_at": null}),
        );
        let (stdout, parts, _) = run_stop(root, json!({})).unwrap();
        assert_eq!(stdout, "");
        assert_eq!(parts.len(), 1);
        let text = &parts[0];
        assert!(text.starts_with("bee session-close warning: session is ending mid-phase (phase: swarming) "));
        // numeric-aware id sort: w-2 before w-10
        assert!(text.contains("Claimed-but-uncapped cells: w-2, w-10 (worker-b)."));
        assert!(text.contains("Active reservations: alpha -> src/api (cell w-2)."));
        // three sanctioned exits: finish-and-cap, HANDOFF.json + release, or a
        // decision-0017 capture stub (p-808487c4) — the third was missing.
        assert!(text.contains(
            "Either finish and cap the work, write .bee/HANDOFF.json and release reservations \
so the next session can resume cleanly, or record a capture stub for what settled \
(bee capture add) and close cleanly."
        ));
        assert!(text.ends_with("close cleanly."));
    }

    #[test]
    fn handoff_suppresses_warning_and_expired_leases_drop() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "swarming"}));
        write_json_file(&root.join(".bee").join("HANDOFF.json"), &json!({"kind": "pause"}));
        let (_, parts, _) = run_stop(root, json!({})).unwrap();
        assert!(parts.is_empty());
        // expired lease is not "active"
        std::fs::remove_file(root.join(".bee").join("HANDOFF.json")).unwrap();
        write_json_file(
            &root.join(".bee").join("runtime").join("leases").join("paths").join("h1.json"),
            &json!({"resource": "path:src", "workflow_id": "w", "workspace_id": "agent:a", "acquired_at": "2020-01-01T00:00:00.000Z", "expires_at": "2020-01-01T01:00:00.000Z"}),
        );
        let (_, parts, _) = run_stop(root, json!({})).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(!parts[0].contains("Active reservations"));
    }

    #[test]
    fn capture_queue_nudge_counts_pending_and_dedupes() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        let queue = root.join(".bee").join("capture-queue.jsonl");
        // s2's `at` must stay recent — U3 (docs/history/knowledge-usable/
        // CONTEXT.md) escalates the nudge once the oldest PENDING stub is
        // older than the configured day threshold (default 7); a fixed
        // past date would drift stale and flip this test's wording.
        let recent = now_iso();
        std::fs::write(
            &queue,
            format!(
                "{{\"kind\":\"stub\",\"id\":\"s1\",\"at\":\"2026-01-01T00:00:00.000Z\",\"outcome\":\"x\"}}\n\
{{\"kind\":\"stub\",\"id\":\"s2\",\"at\":\"{recent}\",\"outcome\":\"y\"}}\n\
{{\"kind\":\"flush\",\"id\":\"s1\",\"at\":\"2026-01-03T00:00:00.000Z\"}}\n"
            ),
        )
        .unwrap();
        let (_, parts, _) = run_stop(root, json!({})).unwrap();
        assert!(parts.iter().any(|p| p.starts_with("bee capture queue (decision 0017): 1 settlement stub(s) are queued")));
        // deduped on the second run (same pending set, < 30 min)
        let (_, parts2, _) = run_stop(root, json!({})).unwrap();
        assert!(!parts2.iter().any(|p| p.contains("bee capture queue")));
    }

    #[test]
    fn capture_nudge_fires_when_decision_newer_than_docs() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        std::fs::create_dir_all(root.join("docs").join("specs")).unwrap();
        std::fs::write(root.join("docs").join("specs").join("area.md"), "# spec\n").unwrap();
        let recent = ms_to_iso(now_ms() + 60_000.0).unwrap(); // decision newer than the spec file
        std::fs::write(
            root.join(".bee").join("decisions.jsonl"),
            format!("{{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"{recent}\",\"decision\":\"x\"}}\n"),
        )
        .unwrap();
        let (_, parts, _) = run_stop(root, json!({})).unwrap();
        let nudge = parts.iter().find(|p| p.starts_with("bee capture nudge (decision 0003)")).unwrap();
        assert!(nudge.contains("area spec under docs/specs/")); // no-bundle variant
        // bundle variant: a concept with type frontmatter flips the wording
        let fx2 = fixture();
        let root2 = fx2.path();
        write_json_file(&root2.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root2.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        let bundle = root2.join("docs").join("knowledge").join("areas").join("x");
        std::fs::create_dir_all(&bundle).unwrap();
        std::fs::write(bundle.join("concept.md"), "---\ntype: concept\n---\nbody\n").unwrap();
        std::fs::write(
            root2.join(".bee").join("decisions.jsonl"),
            format!("{{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"{recent}\",\"decision\":\"x\"}}\n"),
        )
        .unwrap();
        // make the concept file older than the decision
        let old = filetime::FileTime::from_unix_time(1_600_000_000, 0);
        filetime::set_file_mtime(bundle.join("concept.md"), old).unwrap();
        let (_, parts2, _) = run_stop(root2, json!({})).unwrap();
        let nudge2 = parts2.iter().find(|p| p.starts_with("bee capture nudge (decision 0003)")).unwrap();
        assert!(nudge2.contains("knowledge bundle (docs/knowledge/)"));
    }

    #[test]
    fn superseded_and_redacted_decisions_are_skipped() {
        let fx = fixture();
        let root = fx.path();
        std::fs::write(
            root.join(".bee").join("decisions.jsonl"),
            concat!(
                "{\"id\":\"a\",\"type\":\"decide\",\"date\":\"2026-01-01T00:00:00.000Z\"}\n",
                "{\"id\":\"b\",\"type\":\"decide\",\"date\":\"2026-01-02T00:00:00.000Z\"}\n",
                "{\"id\":\"c\",\"type\":\"redact\",\"redacts\":\"b\",\"date\":\"2026-01-03T00:00:00.000Z\"}\n"
            ),
        )
        .unwrap();
        let (id, date) = newest_active_decision(root).unwrap();
        assert_eq!(id, json!("a"));
        assert_eq!(date, json!("2026-01-01T00:00:00.000Z"));
    }

    #[test]
    fn corrupt_state_reads_as_defaults_and_precompact_still_delegates() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        std::fs::write(root.join(".bee").join("state.json"), "{broken").unwrap();
        let (stdout, parts, stderr) = run_stop(root, json!({})).expect("must run natively");
        // defaultState() → phase idle → the decision-nudge branch, never the
        // mid-phase warning; no block; the corruption is reported once.
        assert_eq!(stdout, "");
        assert!(!parts.iter().any(|p| p.contains("hive door open")));
        // TWO lines, matching Node: bee-session-close.mjs reads state.json
        // once itself and once more through resolvePipeline's defaults().
        assert_eq!(stderr.matches("could not parse JSON at").count(), 2);
        assert!(stderr.contains("Using fallback; fix the file."));
        // PreCompact still delegates in run_inner.
        let fx2 = fixture();
        write_json_file(&fx2.path().join(".bee").join("config.json"), &json!({}));
        let body = json!({"hook_event_name": "PreCompact", "cwd": fx2.path().to_string_lossy()});
        assert!(run_inner(&[], &serde_json::to_string(&body).unwrap()).is_err());
    }

    #[test]
    fn corrupt_handoff_still_raises_the_mid_phase_warning() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(
            &root.join(".bee").join("state.json"),
            &json!({"phase": "swarming", "mode": "standard"}),
        );
        std::fs::write(root.join(".bee").join("HANDOFF.json"), "{broken").unwrap();
        let (stdout, parts, stderr) = run_stop(root, json!({})).expect("must run natively");
        assert_eq!(stdout, "");
        // readHandoff's null fallback = "no handoff" → the door-open warning.
        assert!(parts.iter().any(|p| p.contains("You are about to leave the hive door open")));
        assert_eq!(stderr.matches("could not parse JSON at").count(), 1);
    }

    #[test]
    fn corrupt_lane_record_refuses_and_falls_back_to_state() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        write_json_file(
            &root.join(".bee").join("sessions").join("s-1.json"),
            &json!({"id": "s-1", "lane": "l1"}),
        );
        std::fs::create_dir_all(root.join(".bee").join("lanes")).unwrap();
        std::fs::write(root.join(".bee").join("lanes").join("l1.json"), "{broken").unwrap();
        let (_, _, stderr) = run_stop(root, json!({"session_id": "s-1"})).expect("native");
        // Both of Node's lines, in Node's order: readJson's, then readLane's.
        let readjson_at = stderr.find("could not parse JSON at").unwrap();
        let readlane_at = stderr.find("readLane: skipping corrupt lane record").unwrap();
        assert!(readjson_at < readlane_at);
        assert_eq!(stderr.matches("could not parse JSON at").count(), 1);
    }

    #[test]
    fn corrupt_session_record_reads_as_no_session() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        std::fs::create_dir_all(root.join(".bee").join("sessions")).unwrap();
        std::fs::write(root.join(".bee").join("sessions").join("s-1.json"), "{broken").unwrap();
        let (stdout, _, stderr) = run_stop(root, json!({"session_id": "s-1"})).expect("native");
        assert_eq!(stdout, "");
        assert_eq!(stderr.matches("could not parse JSON at").count(), 1);
    }

    #[test]
    fn corrupt_cell_is_skipped_from_the_claimed_list() {
        let fx = fixture();
        let root = fx.path();
        let cells = root.join(".bee").join("cells");
        write_json_file(&cells.join("c-1.json"), &json!({"id": "c-1", "status": "claimed"}));
        std::fs::write(cells.join("bad.json"), "{broken").unwrap();
        let listed = list_claimed_cells(root).expect("must not delegate");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].get("id"), Some(&json!("c-1")));
        assert_eq!(take_corrupt_json_warnings().matches("bad.json").count(), 1);
    }

    #[test]
    fn corrupt_inject_cache_falls_through_to_empty() {
        let fx = fixture();
        let root = fx.path();
        clear_corrupt_json_warnings();
        std::fs::create_dir_all(root.join(".bee").join("cache")).unwrap();
        std::fs::write(inject_cache_path(root), "{broken").unwrap();
        // Reads as absent → `{}` → every key is due for injection again.
        let cache = read_inject_cache(root).expect("must not delegate");
        assert!(cache.is_empty());
        assert!(should_inject(root, "any-key", "h1").unwrap());
        // A non-object cache is still a delegate (JS assignment exotica).
        std::fs::write(inject_cache_path(root), "[1,2]").unwrap();
        assert!(read_inject_cache(root).is_err());
        clear_corrupt_json_warnings();
    }

    #[test]
    fn frontmatter_subset_rules() {
        assert!(frontmatter_has_type("---\ntype: concept\n---\nbody\n"));
        assert!(frontmatter_has_type("---\r\ntitle: \"x: y\"\r\ntype: note\r\n---\r\n"));
        assert!(!frontmatter_has_type("no frontmatter"));
        assert!(!frontmatter_has_type("---\ntype: concept\n")); // unclosed
        assert!(!frontmatter_has_type("---\ntype: concept\n\n---\n")); // blank line
        assert!(!frontmatter_has_type("---\ntype: true\n---\n")); // boolean type
        assert!(!frontmatter_has_type("---\ntype: \"\"\n---\n")); // empty string
        assert!(!frontmatter_has_type("---\ntype: concept\ntype: again\n---\n")); // dup
        assert!(!frontmatter_has_type("---\nnested:\n  k: v\n---\n")); // non-bee map
        assert!(frontmatter_has_type("---\ntype: concept\nbee:\n  cell: x\n---\n"));
        assert!(!frontmatter_has_type("---\ntags: [a, \"b\"\ntype: t\n---\n")); // bad list
        assert!(frontmatter_has_type("---\ntags: [a, \"b\"]\ntype: t\n---\n"));
    }

    #[test]
    fn locale_numeric_sort_matches_expected_slug_order() {
        let mut ids = vec!["w-10", "w-2", "w-1", "x-1", "a2", "a10", "A3"];
        ids.sort_by(|a, b| cmp_locale_numeric(a, b));
        assert_eq!(ids, vec!["a2", "A3", "a10", "w-1", "w-2", "w-10", "x-1"]);
    }

    #[test]
    fn perf_helpers_match_node_shapes() {
        // Cutover fix: the drive colon is encoded away too, so the name is
        // legal on NTFS (Node spelled "D:-a-b-c", a component mkdir rejects).
        assert_eq!(encode_project_dir("D:\\a\\b.c"), "D--a-b-c");
        assert_eq!(encode_project_dir("/a/b.c"), "-a-b-c");
        assert_eq!(humanize_ms(3_723_000.0), "1h2m3s");
        assert_eq!(humanize_ms(0.0), "0s");
        assert_eq!(fmt_tokens(1_234.0), "1.2k");
        assert_eq!(fmt_tokens(999.0), "999");
        assert_eq!(fmt_tokens(2_500_000.0), "2.50M");
        assert_eq!(short_model("claude-sonnet-4-20250514"), "sonnet-4");
        assert_eq!(short_model("gpt-5.5"), "gpt-5.5");
        assert_eq!(cache_pct(200.0, 50.0), "25%");
        assert_eq!(cache_pct(0.0, 0.0), "—");
        assert_eq!(project_name(&json!("D:\\x\\proj\\")), "proj");
        assert_eq!(project_name(&Value::Null), "(unknown)");
    }

    /// Serializes every test that mutates the process-global `BEEHIVE_PERF_DIR`
    /// var — `cargo test` runs test fns on multiple threads in the SAME
    /// process, so two such tests racing would each read the other's tempdir.
    fn lock_perf_env() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn rollup_and_upsert_roundtrip_in_isolated_perf_dir() {
        let _guard = lock_perf_env();
        // BEEHIVE_PERF_DIR isolates the machine-global store for this test.
        let perf = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("BEEHIVE_PERF_DIR", perf.path()) };
        let tdir = tempfile::tempdir().unwrap();
        let transcript = tdir.path().join("sess-1.jsonl");
        std::fs::write(
            &transcript,
            concat!(
                "{\"type\":\"assistant\",\"timestamp\":\"2026-01-01T00:00:00.000Z\",\"requestId\":\"r1\",\"cwd\":\"D:\\\\p\\\\demo\",\"message\":{\"model\":\"claude-sonnet-4-20250514\",\"usage\":{\"input_tokens\":10,\"output_tokens\":5,\"cache_read_input_tokens\":100}}}\n",
                "{\"type\":\"assistant\",\"timestamp\":\"2026-01-01T00:01:00.000Z\",\"requestId\":\"r1\",\"message\":{\"model\":\"claude-sonnet-4-20250514\",\"usage\":{\"input_tokens\":10,\"output_tokens\":9,\"cache_read_input_tokens\":100}}}\n",
                "{\"type\":\"system\",\"subtype\":\"turn_duration\",\"timestamp\":\"2026-01-01T00:01:01.000Z\",\"durationMs\":1500}\n"
            ),
        )
        .unwrap();
        let rollup = rollup_transcript(&transcript).unwrap();
        assert_eq!(rollup.session_id, "sess-1");
        assert_eq!(rollup.event_count, 3);
        assert_eq!(rollup.running_time_ms, 1500.0);
        // requestId dedupe keeps the larger-output record
        assert_eq!(
            jsjson::stringify(&rollup.models),
            r#"{"claude-sonnet-4-20250514":{"input":10,"output":9,"cache_write":0,"cache_read":100,"new":19,"cached":100,"total":119}}"#
        );
        let record = session_record(&rollup).unwrap();
        upsert_session_records(&[record.clone()]).unwrap();
        upsert_session_records(&[record]).unwrap(); // dedupe by session_id
        assert_eq!(read_session_records().len(), 1);
        let projects = build_matrix_from_log();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].project, "demo");
        assert_eq!(projects[0].total_tokens, 119.0);
        let html = render_matrix_html(&projects, "2026-01-01T00:00:00.000Z").unwrap();
        assert!(html.contains("<title>bee performance</title>"));
        assert!(html.contains("sonnet-4"));
        unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };
    }

    // ─── SessionEnd (close the session record) ─────────────────────────────

    #[test]
    fn session_end_marks_the_session_record_closed() {
        let fx = fixture();
        let root = fx.path();
        write_json_file(
            &root.join(".bee").join("sessions").join("s-1.json"),
            &json!({"id": "s-1", "started_at": "2026-01-01T00:00:00.000Z"}),
        );
        let body = json!({
            "hook_event_name": "SessionEnd",
            "cwd": root.to_string_lossy(),
            "session_id": "s-1",
        });
        let stdin = serde_json::to_string(&body).unwrap();
        // The SessionEnd branch returns before the perf refresh runs, so no
        // BEEHIVE_PERF_DIR isolation is needed here.
        assert_eq!(run_inner(&[], &stdin), Ok(()));
        let record: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee").join("sessions").join("s-1.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(record["status"], "closed");
        assert!(record["closed_at"].as_str().unwrap().ends_with('Z'), "{record}");
        // The rest of the record survives the write untouched.
        assert_eq!(record["started_at"], "2026-01-01T00:00:00.000Z");
    }

    #[test]
    fn stop_payload_leaves_the_session_record_untouched() {
        let fx = fixture();
        let root = fx.path();
        // BEEHIVE_PERF_DIR isolates the machine-global store the Stop path's
        // perf refresh touches — see rollup_and_upsert_roundtrip_in_isolated_
        // perf_dir above; lock_perf_env keeps the two tests from racing on it.
        let _guard = lock_perf_env();
        let perf = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("BEEHIVE_PERF_DIR", perf.path()) };
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        let record = json!({"id": "s-1", "started_at": "2026-01-01T00:00:00.000Z"});
        write_json_file(&root.join(".bee").join("sessions").join("s-1.json"), &record);
        let body = json!({
            "hook_event_name": "Stop",
            "cwd": root.to_string_lossy(),
            "session_id": "s-1",
        });
        let stdin = serde_json::to_string(&body).unwrap();
        let _ = run_inner(&[], &stdin);
        unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };
        let after: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee").join("sessions").join("s-1.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(after, record, "Stop must never touch status/closed_at");
    }

    // ─── auto-wait-mark D1/D5/D6: the Stop-only turn-end mark ──────────────
    //
    // These drive `run_inner` (the real dispatch entry every Stop payload
    // reaches), not `run_stop` above — `run_stop` calls `advisory()`
    // directly and never touches `run_inner`'s match arms, which is exactly
    // where the turn-end setter hangs (mirrors `SessionEnd`/`PreCompact`'s
    // own placement, both handled in `run_inner` before `advisory()` is ever
    // reached).

    /// `Err2`'s `Exotic` payload has no `Debug` impl, so a plain
    /// `.unwrap()` on a `Result<_, Err2>` does not compile — same helper
    /// `prompt_context.rs`'s own tests already carry for the same reason.
    fn ok<T, E>(r: Result<T, E>) -> T {
        match r {
            Ok(v) => v,
            Err(_) => panic!("unexpected error result"),
        }
    }

    /// Serializes every test that mutates the process-global
    /// `CLAUDE_CONFIG_DIR` var (`resolve_transcript_for`'s own root) — same
    /// hazard `lock_perf_env` guards for `BEEHIVE_PERF_DIR`.
    fn lock_transcript_env() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Writes a real transcript `.jsonl`, at exactly the path
    /// `resolve_transcript_for` looks for under an isolated
    /// `CLAUDE_CONFIG_DIR`: `<config>/projects/<encode_project_dir(root)>/
    /// <session>.jsonl`. `lines` are raw JSONL text, one per transcript
    /// event, oldest first.
    fn write_transcript(config_dir: &Path, root: &Path, session: &str, lines: &[&str]) {
        let dir = config_dir.join("projects").join(encode_project_dir(&root.to_string_lossy()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{session}.jsonl")), format!("{}\n", lines.join("\n"))).unwrap();
    }

    #[test]
    fn stop_with_no_live_mark_sets_a_turn_end_mark_from_the_transcripts_last_line() {
        let _perf_guard = lock_perf_env();
        let _transcript_guard = lock_transcript_env();
        let perf = tempfile::tempdir().unwrap();
        let config = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("BEEHIVE_PERF_DIR", perf.path()) };
        unsafe { std::env::set_var("CLAUDE_CONFIG_DIR", config.path()) };

        let fx = fixture();
        // resolve_roots realpaths the root (dunce::canonicalize). A Windows
        // runner's temp path carries 8.3 short components (RUNNER~1) that
        // canonicalize to a different STRING, and the transcript's
        // projects-dir name is built from that string — so the fixture has to
        // use the canonical spelling the hook itself resolves.
        let root_canon = dunce::canonicalize(fx.path()).unwrap();
        let root = root_canon.as_path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        // The turn's FINAL assistant entry is a bare tool_use (no text) —
        // the setter must scan backward past it to the entry that actually
        // carries a text block.
        write_transcript(
            config.path(),
            root,
            "s-1",
            &[
                r#"{"type":"assistant","message":{"content":[{"type":"text","text":"line one\n\nSay go and I will do it.  \n"}]}}"#,
                r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{}}]}}"#,
            ],
        );

        let body = json!({
            "hook_event_name": "Stop",
            "cwd": root.to_string_lossy(),
            "session_id": "s-1",
        });
        let stdin = serde_json::to_string(&body).unwrap();
        assert_eq!(run_inner(&[], &stdin), Ok(()));

        unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };
        unsafe { std::env::remove_var("CLAUDE_CONFIG_DIR") };

        let state: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee").join("state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(state["waiting_on"]["kind"], "turn-end");
        assert_eq!(state["waiting_on"]["subject"], "Say go and I will do it.");
        assert_eq!(state["waiting_on"]["session"], "s-1");
        assert_eq!(state["run_state"], "awaiting-approval");
    }

    /// auto-wait-mark rework: the perf-refresh rollup and the turn-end mark
    /// setter both need this Stop's transcript — `perf_refresh` resolves and
    /// reads it for the rollup, and `turn_end_subject` needs its final
    /// assistant line. Pins the fix by COUNTING `read_jsonl` calls against
    /// the transcript's own path rather than eyeballing the call graph: the
    /// old code called `read_jsonl` on this exact path twice per Stop (once
    /// inside `perf_refresh`'s rollup, once again inside `turn_end_subject`);
    /// the fix threads the one parsed event vector through both, so it must
    /// land at exactly one.
    #[test]
    fn stop_reads_the_transcript_exactly_once_per_turn() {
        let _perf_guard = lock_perf_env();
        let _transcript_guard = lock_transcript_env();
        let perf = tempfile::tempdir().unwrap();
        let config = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("BEEHIVE_PERF_DIR", perf.path()) };
        unsafe { std::env::set_var("CLAUDE_CONFIG_DIR", config.path()) };

        let fx = fixture();
        // resolve_roots realpaths the root (dunce::canonicalize). A Windows
        // runner's temp path carries 8.3 short components (RUNNER~1) that
        // canonicalize to a different STRING, and the transcript's
        // projects-dir name is built from that string — so the fixture has to
        // use the canonical spelling the hook itself resolves.
        let root_canon = dunce::canonicalize(fx.path()).unwrap();
        let root = root_canon.as_path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        write_transcript(
            config.path(),
            root,
            "s-1",
            &[r#"{"type":"assistant","message":{"content":[{"type":"text","text":"hello there"}]}}"#],
        );
        let transcript_path = config
            .path()
            .join("projects")
            .join(encode_project_dir(&root.to_string_lossy()))
            .join("s-1.jsonl");

        let body = json!({
            "hook_event_name": "Stop",
            "cwd": root.to_string_lossy(),
            "session_id": "s-1",
        });
        let stdin = serde_json::to_string(&body).unwrap();

        READ_JSONL_LOG.with(|log| log.borrow_mut().clear());
        assert_eq!(run_inner(&[], &stdin), Ok(()));
        let transcript_reads = READ_JSONL_LOG
            .with(|log| log.borrow().iter().filter(|p| **p == transcript_path).count());

        unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };
        unsafe { std::env::remove_var("CLAUDE_CONFIG_DIR") };

        assert_eq!(
            transcript_reads, 1,
            "expected exactly one std::fs::read of the transcript per Stop, got {transcript_reads}"
        );
        // The mark itself must still have been written from that one read.
        let state: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee").join("state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(state["waiting_on"]["kind"], "turn-end");
        assert_eq!(state["waiting_on"]["subject"], "hello there");
    }

    /// auto-wait-mark second rework: the same one-read guarantee, but on the
    /// path where the perf refresh FAILS *after* its read. `BEEHIVE_PERF_DIR`
    /// points at a regular file here, so `upsert_session_records`'
    /// `create_dir_all` errors — strictly after the transcript was read. The
    /// previous shape folded the events into that `Err`, `run_inner` turned
    /// it into `None`, and `turn_end_subject` then re-resolved and re-read
    /// the same file: two reads per Stop, the one thing
    /// `docs/history/auto-wait-mark/CONTEXT.md` forbids. The events now
    /// survive the failure, so this lands at exactly one read — and the mark
    /// is still written from it, with the perf error still logged.
    #[test]
    fn a_late_perf_refresh_failure_still_reads_the_transcript_exactly_once() {
        let _perf_guard = lock_perf_env();
        let _transcript_guard = lock_transcript_env();
        let perf = tempfile::tempdir().unwrap();
        let config = tempfile::tempdir().unwrap();
        // A regular FILE where the perf dir belongs: every write under it
        // fails, and the first such failure comes after the transcript read.
        let perf_file = perf.path().join("not-a-dir");
        std::fs::write(&perf_file, "").unwrap();
        unsafe { std::env::set_var("BEEHIVE_PERF_DIR", &perf_file) };
        unsafe { std::env::set_var("CLAUDE_CONFIG_DIR", config.path()) };

        let fx = fixture();
        // resolve_roots realpaths the root (dunce::canonicalize). A Windows
        // runner's temp path carries 8.3 short components (RUNNER~1) that
        // canonicalize to a different STRING, and the transcript's
        // projects-dir name is built from that string — so the fixture has to
        // use the canonical spelling the hook itself resolves.
        let root_canon = dunce::canonicalize(fx.path()).unwrap();
        let root = root_canon.as_path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        write_transcript(
            config.path(),
            root,
            "s-1",
            &[r#"{"type":"assistant","message":{"content":[{"type":"text","text":"hello there"}]}}"#],
        );
        let transcript_path = config
            .path()
            .join("projects")
            .join(encode_project_dir(&root.to_string_lossy()))
            .join("s-1.jsonl");

        // The failure is real, and it lands after the read: the same call
        // the hook makes returns Err, and hands the events back anyway.
        let refreshed = perf_refresh(root, Some("s-1"));
        assert!(refreshed.result.is_err(), "the perf refresh must fail on a file-shaped perf dir");
        assert!(
            refreshed.events.as_ref().is_some_and(|e| !e.is_empty()),
            "a failing perf refresh must still hand back the events it read"
        );

        let body = json!({
            "hook_event_name": "Stop",
            "cwd": root.to_string_lossy(),
            "session_id": "s-1",
        });
        let stdin = serde_json::to_string(&body).unwrap();

        READ_JSONL_LOG.with(|log| log.borrow_mut().clear());
        assert_eq!(run_inner(&[], &stdin), Ok(()));
        let transcript_reads = READ_JSONL_LOG
            .with(|log| log.borrow().iter().filter(|p| **p == transcript_path).count());

        unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };
        unsafe { std::env::remove_var("CLAUDE_CONFIG_DIR") };

        assert_eq!(
            transcript_reads, 1,
            "a Stop whose perf refresh fails after its read must still read the transcript once, got {transcript_reads}"
        );
        // The mark is still written, from that one read.
        let state: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee").join("state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(state["waiting_on"]["kind"], "turn-end");
        assert_eq!(state["waiting_on"]["subject"], "hello there");
    }

    #[test]
    fn a_final_assistant_text_block_that_is_blank_still_yields_a_non_empty_subject() {
        let _perf_guard = lock_perf_env();
        let _transcript_guard = lock_transcript_env();
        let perf = tempfile::tempdir().unwrap();
        let config = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("BEEHIVE_PERF_DIR", perf.path()) };
        unsafe { std::env::set_var("CLAUDE_CONFIG_DIR", config.path()) };

        let fx = fixture();
        // resolve_roots realpaths the root (dunce::canonicalize). A Windows
        // runner's temp path carries 8.3 short components (RUNNER~1) that
        // canonicalize to a different STRING, and the transcript's
        // projects-dir name is built from that string — so the fixture has to
        // use the canonical spelling the hook itself resolves.
        let root_canon = dunce::canonicalize(fx.path()).unwrap();
        let root = root_canon.as_path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        // A transcript that resolves and parses fine, but the only text
        // block it carries is whitespace-only.
        write_transcript(
            config.path(),
            root,
            "s-1",
            &[r#"{"type":"assistant","message":{"content":[{"type":"text","text":"   \n  \n"}]}}"#],
        );

        let body = json!({
            "hook_event_name": "Stop",
            "cwd": root.to_string_lossy(),
            "session_id": "s-1",
        });
        let stdin = serde_json::to_string(&body).unwrap();
        assert_eq!(run_inner(&[], &stdin), Ok(()));

        unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };
        unsafe { std::env::remove_var("CLAUDE_CONFIG_DIR") };

        let state: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee").join("state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(state["waiting_on"]["kind"], "turn-end");
        let subject = state["waiting_on"]["subject"].as_str().unwrap();
        assert!(!subject.trim().is_empty(), "subject must be non-empty: {state}");
    }

    #[test]
    fn a_missing_or_malformed_transcript_writes_no_mark_but_leaves_the_hook_native() {
        let _perf_guard = lock_perf_env();
        let _transcript_guard = lock_transcript_env();
        let perf = tempfile::tempdir().unwrap();
        let config = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("BEEHIVE_PERF_DIR", perf.path()) };
        unsafe { std::env::set_var("CLAUDE_CONFIG_DIR", config.path()) };

        // Case one: no transcript file at all (nothing written under
        // config/projects/...).
        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        let body = json!({"hook_event_name": "Stop", "cwd": root.to_string_lossy(), "session_id": "s-1"});
        let stdin = serde_json::to_string(&body).unwrap();
        assert_eq!(run_inner(&[], &stdin), Ok(()), "a missing transcript must never take the hook down");
        let state: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee").join("state.json")).unwrap(),
        )
        .unwrap();
        assert!(state.get("waiting_on").is_none() || state["waiting_on"].is_null(), "{state}");

        // Case two: a transcript file exists but carries zero parseable
        // JSONL lines.
        let fx2 = fixture();
        // Same canonical-spelling requirement as the tests above: on a raw
        // tempdir path this case never reaches the malformed-transcript
        // branch — the hook simply finds no file and the assertion passes
        // for the wrong reason.
        let root2_canon = dunce::canonicalize(fx2.path()).unwrap();
        let root2 = root2_canon.as_path();
        write_json_file(&root2.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root2.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        write_transcript(config.path(), root2, "s-2", &["not json at all"]);
        let body2 = json!({"hook_event_name": "Stop", "cwd": root2.to_string_lossy(), "session_id": "s-2"});
        let stdin2 = serde_json::to_string(&body2).unwrap();
        assert_eq!(run_inner(&[], &stdin2), Ok(()), "a malformed transcript must never take the hook down");
        let state2: Value = serde_json::from_str(
            &std::fs::read_to_string(root2.join(".bee").join("state.json")).unwrap(),
        )
        .unwrap();
        assert!(state2.get("waiting_on").is_none() || state2["waiting_on"].is_null(), "{state2}");

        unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };
        unsafe { std::env::remove_var("CLAUDE_CONFIG_DIR") };
    }

    #[test]
    fn session_end_pre_compact_and_a_non_stop_event_write_no_mark() {
        for event in ["SessionEnd", "PreCompact", "SubagentStop"] {
            let _perf_guard = lock_perf_env();
            let perf = tempfile::tempdir().unwrap();
            unsafe { std::env::set_var("BEEHIVE_PERF_DIR", perf.path()) };

            let fx = fixture();
            let root = fx.path();
            write_json_file(&root.join(".bee").join("config.json"), &json!({}));
            write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
            let body = json!({
                "hook_event_name": event,
                "cwd": root.to_string_lossy(),
                "session_id": "s-1",
            });
            let stdin = serde_json::to_string(&body).unwrap();
            let _ = run_inner(&[], &stdin); // PreCompact returns Err(()) (delegate) — expected

            unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };

            let state: Value = serde_json::from_str(
                &std::fs::read_to_string(root.join(".bee").join("state.json")).unwrap(),
            )
            .unwrap();
            assert!(
                state.get("waiting_on").is_none() || state["waiting_on"].is_null(),
                "{event}: {state}"
            );
        }
    }

    #[test]
    fn stop_never_overwrites_a_live_declared_wait() {
        let _perf_guard = lock_perf_env();
        let _transcript_guard = lock_transcript_env();
        let perf = tempfile::tempdir().unwrap();
        let config = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("BEEHIVE_PERF_DIR", perf.path()) };
        unsafe { std::env::set_var("CLAUDE_CONFIG_DIR", config.path()) };

        let fx = fixture();
        // resolve_roots realpaths the root (dunce::canonicalize). A Windows
        // runner's temp path carries 8.3 short components (RUNNER~1) that
        // canonicalize to a different STRING, and the transcript's
        // projects-dir name is built from that string — so the fixture has to
        // use the canonical spelling the hook itself resolves.
        let root_canon = dunce::canonicalize(fx.path()).unwrap();
        let root = root_canon.as_path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        write_transcript(
            config.path(),
            root,
            "s-1",
            &[r#"{"type":"assistant","message":{"content":[{"type":"text","text":"a different, later line"}]}}"#],
        );
        // A declared wait, set through the real setter — the same shape
        // `state waiting-on set` produces (D2: no provenance to tell them
        // apart, so the ONLY thing protecting it is D5's kind-blind live
        // check).
        ok(crate::verbs::state_group::set_default_state_waiting_on(
            root,
            "question",
            "should we ship the v3 index?",
            "sess-asker",
        ));

        let body = json!({
            "hook_event_name": "Stop",
            "cwd": root.to_string_lossy(),
            "session_id": "s-1",
        });
        let stdin = serde_json::to_string(&body).unwrap();
        assert_eq!(run_inner(&[], &stdin), Ok(()));

        unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };
        unsafe { std::env::remove_var("CLAUDE_CONFIG_DIR") };

        let state: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee").join("state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(state["waiting_on"]["kind"], "question");
        assert_eq!(state["waiting_on"]["subject"], "should we ship the v3 index?");
        assert_eq!(state["waiting_on"]["session"], "sess-asker");
    }

    /// Probe (auto-wait-mark plan, "Deferred To Planning" Q2): does a
    /// re-entrant Stop (Claude Code's own `stop_hook_active` continuation)
    /// double the mark? It cannot — D5's live-check already guards every
    /// write, so a second Stop before the next `UserPromptSubmit` clear
    /// finds the mark the FIRST Stop just wrote and leaves it untouched. No
    /// `stop_hook_active` guard is added; this is the proof.
    #[test]
    fn a_second_stop_before_any_clear_never_doubles_or_changes_the_mark() {
        let _perf_guard = lock_perf_env();
        let _transcript_guard = lock_transcript_env();
        let perf = tempfile::tempdir().unwrap();
        let config = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("BEEHIVE_PERF_DIR", perf.path()) };
        unsafe { std::env::set_var("CLAUDE_CONFIG_DIR", config.path()) };

        let fx = fixture();
        // resolve_roots realpaths the root (dunce::canonicalize). A Windows
        // runner's temp path carries 8.3 short components (RUNNER~1) that
        // canonicalize to a different STRING, and the transcript's
        // projects-dir name is built from that string — so the fixture has to
        // use the canonical spelling the hook itself resolves.
        let root_canon = dunce::canonicalize(fx.path()).unwrap();
        let root = root_canon.as_path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        write_transcript(
            config.path(),
            root,
            "s-1",
            &[r#"{"type":"assistant","message":{"content":[{"type":"text","text":"first stop's line"}]}}"#],
        );

        let body = json!({"hook_event_name": "Stop", "cwd": root.to_string_lossy(), "session_id": "s-1"});
        let stdin = serde_json::to_string(&body).unwrap();
        assert_eq!(run_inner(&[], &stdin), Ok(()));

        // As if the agent kept talking through a re-entrant continuation —
        // a DIFFERENT last line, so a second write would be visible.
        write_transcript(
            config.path(),
            root,
            "s-1",
            &[
                r#"{"type":"assistant","message":{"content":[{"type":"text","text":"first stop's line"}]}}"#,
                r#"{"type":"assistant","message":{"content":[{"type":"text","text":"second stop's line"}]}}"#,
            ],
        );
        assert_eq!(run_inner(&[], &stdin), Ok(()));

        unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };
        unsafe { std::env::remove_var("CLAUDE_CONFIG_DIR") };

        let state: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee").join("state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            state["waiting_on"]["subject"], "first stop's line",
            "D5 must keep the FIRST Stop's mark, never the second's"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_failing_store_write_is_logged_and_never_fails_the_stop_hook() {
        use std::os::unix::fs::PermissionsExt;

        let _perf_guard = lock_perf_env();
        let _transcript_guard = lock_transcript_env();
        let perf = tempfile::tempdir().unwrap();
        let config = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("BEEHIVE_PERF_DIR", perf.path()) };
        unsafe { std::env::set_var("CLAUDE_CONFIG_DIR", config.path()) };

        let fx = fixture();
        let root = fx.path();
        write_json_file(&root.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root.join(".bee").join("state.json"), &json!({"phase": "idle"}));
        write_transcript(
            config.path(),
            root,
            "s-1",
            &[r#"{"type":"assistant","message":{"content":[{"type":"text","text":"still waiting on you"}]}}"#],
        );

        let bee_dir = root.join(".bee");
        // Pre-create every subdirectory the rest of the hook writes into so
        // ONLY the mark's own `write_json_atomic(state.json, ..)` tmp-file
        // creation fails — same shape `prompt_context.rs`'s own failure-
        // injection test uses for the clear path.
        std::fs::create_dir_all(bee_dir.join("locks")).unwrap();
        std::fs::create_dir_all(bee_dir.join("cache")).unwrap();
        std::fs::create_dir_all(bee_dir.join("logs")).unwrap();
        let writable = std::fs::metadata(&bee_dir).unwrap().permissions();
        let mut readonly = writable.clone();
        readonly.set_mode(0o555); // r-x, no write: no new file inside .bee/
        std::fs::set_permissions(&bee_dir, readonly).unwrap();

        let body = json!({"hook_event_name": "Stop", "cwd": root.to_string_lossy(), "session_id": "s-1"});
        let stdin = serde_json::to_string(&body).unwrap();
        let outcome = run_inner(&[], &stdin);

        // Restore write access before any assertion can panic and before
        // the tempdir's own Drop tries to remove a directory it can no
        // longer write into.
        std::fs::set_permissions(&bee_dir, writable).unwrap();
        unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };
        unsafe { std::env::remove_var("CLAUDE_CONFIG_DIR") };

        assert_eq!(
            outcome,
            Ok(()),
            "a write that fails on disk must still resolve natively, never crash the hook"
        );
        let crash_log =
            std::fs::read_to_string(bee_dir.join("logs").join("hooks.jsonl")).unwrap_or_default();
        assert!(
            crash_log.contains(HOOK_NAME) && crash_log.contains("waiting_on"),
            "the failed write must be logged: {crash_log}"
        );
    }

    #[test]
    fn codex_transcript_resolution_via_stored_path() {
        let fx = fixture();
        let root = dunce::canonicalize(fx.path()).unwrap();
        let transcript_dir = tempfile::tempdir().unwrap();
        let codex_file = transcript_dir.path().join("rollout-2026-09-12T14-28-28-s-codex-1.jsonl");

        std::fs::write(
            &codex_file,
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"s-codex-1\",\"cwd\":\"/tmp\"}}\n",
        )
        .unwrap();

        let sessions_dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();
        write_json_file(
            &sessions_dir.join("s-codex-1.json"),
            &json!({
                "id": "s-codex-1",
                "transcript_path": codex_file.to_string_lossy()
            }),
        );

        let resolved = resolve_transcript_for(&root, Some("s-codex-1"));
        assert_eq!(resolved, Some(codex_file));
    }

    #[test]
    fn codex_transcript_resolution_via_fallback_dir() {
        let _transcript_guard = lock_transcript_env();
        let fx = fixture();
        let root = dunce::canonicalize(fx.path()).unwrap();

        let codex_home_dir = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("CODEX_HOME", codex_home_dir.path()) };

        let rollout_dir = codex_home_dir.path().join("sessions").join("2026").join("09").join("12");
        std::fs::create_dir_all(&rollout_dir).unwrap();
        let rollout_file = rollout_dir.join("rollout-2026-09-12T14-28-28-s-codex-fallback.jsonl");

        std::fs::write(
            &rollout_file,
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"s-codex-fallback\",\"cwd\":\"/tmp\"}}\n",
        )
        .unwrap();

        let resolved = resolve_transcript_for(&root, Some("s-codex-fallback"));
        unsafe { std::env::remove_var("CODEX_HOME") };

        assert_eq!(resolved, Some(rollout_file));
    }

    #[test]
    fn refusal_to_pick_another_sessions_newest_file() {
        let _transcript_guard = lock_transcript_env();
        let fx = fixture();
        let root = dunce::canonicalize(fx.path()).unwrap();

        let claude_config = tempfile::tempdir().unwrap();
        let codex_home = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("CLAUDE_CONFIG_DIR", claude_config.path()) };
        unsafe { std::env::set_var("CODEX_HOME", codex_home.path()) };

        // Create transcript for session-alpha in Claude projects dir
        write_transcript(
            claude_config.path(),
            &root,
            "session-alpha",
            &[r#"{"type":"session_meta","payload":{"id":"session-alpha"}}"#],
        );

        // Create transcript for session-alpha in Codex dir
        let codex_dir = codex_home.path().join("sessions").join("2026").join("09").join("12");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(
            codex_dir.join("rollout-2026-09-12T15-00-00-session-alpha.jsonl"),
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"session-alpha\"}}\n",
        )
        .unwrap();

        // Querying for session-beta (which has no transcript) must return None, NOT borrow session-alpha
        let resolved_beta = resolve_transcript_for(&root, Some("session-beta"));
        assert_eq!(resolved_beta, None, "must refuse to borrow another session's file");

        // Querying with None session_id must return None, NOT pick newest file
        let resolved_none = resolve_transcript_for(&root, None);
        assert_eq!(resolved_none, None, "must return None when session_id is None");

        // Stored transcript_path pointing to a file with mismatched identity must be refused
        let sessions_dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();
        let alpha_codex_path = codex_dir.join("rollout-2026-09-12T15-00-00-session-alpha.jsonl");
        write_json_file(
            &sessions_dir.join("session-gamma.json"),
            &json!({
                "id": "session-gamma",
                "transcript_path": alpha_codex_path.to_string_lossy()
            }),
        );
        let resolved_gamma = resolve_transcript_for(&root, Some("session-gamma"));
        assert_eq!(resolved_gamma, None, "mismatched identity in stored transcript_path must be refused");

        unsafe { std::env::remove_var("CLAUDE_CONFIG_DIR") };
        unsafe { std::env::remove_var("CODEX_HOME") };
    }

    #[test]
    fn token_usage_aggregation_without_double_counting_cumulative_totals() {
        let events = vec![
            json!({
                "type": "turn_context",
                "payload": {
                    "model": "o3-mini"
                }
            }),
            // Codex event_msg token_count request 1
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 100,
                            "output_tokens": 60,
                            "cached_input_tokens": 20,
                            "cache_write_input_tokens": 10,
                            "total_tokens": 160
                        },
                        "total_token_usage": {
                            "input_tokens": 100,
                            "output_tokens": 60,
                            "cached_input_tokens": 20,
                            "cache_write_input_tokens": 10,
                            "total_tokens": 160
                        }
                    }
                }
            }),
            // Duplicate of request 1
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 100,
                            "output_tokens": 60,
                            "cached_input_tokens": 20,
                            "cache_write_input_tokens": 10,
                            "total_tokens": 160
                        },
                        "total_token_usage": {
                            "input_tokens": 100,
                            "output_tokens": 60,
                            "cached_input_tokens": 20,
                            "cache_write_input_tokens": 10,
                            "total_tokens": 160
                        }
                    }
                }
            }),
            // Codex event_msg token_count request 2
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 40,
                            "output_tokens": 20,
                            "cached_input_tokens": 5,
                            "cache_write_input_tokens": 0,
                            "total_tokens": 60
                        },
                        "total_token_usage": {
                            "input_tokens": 140,
                            "output_tokens": 80,
                            "cached_input_tokens": 25,
                            "cache_write_input_tokens": 10,
                            "total_tokens": 220
                        }
                    }
                }
            }),
            // Codex event_msg token_count request 3
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 10,
                            "output_tokens": 5,
                            "cached_input_tokens": 0,
                            "cache_write_input_tokens": 0,
                            "total_tokens": 15
                        },
                        "total_token_usage": {
                            "input_tokens": 150,
                            "output_tokens": 85,
                            "cached_input_tokens": 25,
                            "cache_write_input_tokens": 10,
                            "total_tokens": 235
                        }
                    }
                }
            }),
        ];

        let agg = aggregate_usage(&events);
        assert_eq!(agg.models.0.len(), 1);
        let (model, acc) = &agg.models.0[0];
        assert_eq!(model, "o3-mini");
        // Normalized uncached input:
        // req 1: 100 - 20 - 10 = 70
        // req 2: 40 - 5 - 0 = 35
        // req 3: 10 - 0 - 0 = 10
        // Total uncached input: 70 + 35 + 10 = 115
        assert_eq!(acc.input, 115.0);
        // Incremental output: 60 + 20 + 5 = 85
        assert_eq!(acc.output, 85.0);
        // Incremental cached: 20 + 5 + 0 = 25
        assert_eq!(acc.cache_read, 25.0);
        // Incremental cache_write: 10 + 0 + 0 = 10
        assert_eq!(acc.cache_write, 10.0);
        // Total = 115 + 85 + 10 + 25 = 235
        assert_eq!(acc.total, 235.0);
    }

    #[test]
    fn commentary_only_and_tool_only_turn_end_subject() {
        // Commentary only
        let commentary_events = vec![
            json!({
                "type": "response_item",
                "payload": {
                    "type": "message",
                    "role": "assistant",
                    "phase": "commentary",
                    "content": [
                        {
                            "type": "output_text",
                            "text": "Reviewing tests...\nReady to verify changes."
                        }
                    ]
                }
            }),
        ];
        assert_eq!(
            turn_end_subject(Some(commentary_events)),
            Some("Ready to verify changes.".to_string())
        );

        // Commentary and final_answer: final_answer takes priority
        let both_events = vec![
            json!({
                "type": "response_item",
                "payload": {
                    "type": "message",
                    "role": "assistant",
                    "phase": "commentary",
                    "content": [{"type": "output_text", "text": "Commentary here"}]
                }
            }),
            json!({
                "type": "response_item",
                "payload": {
                    "type": "message",
                    "role": "assistant",
                    "phase": "final_answer",
                    "content": [{"type": "output_text", "text": "Final answer here"}]
                }
            }),
        ];
        assert_eq!(
            turn_end_subject(Some(both_events)),
            Some("Final answer here".to_string())
        );

        // Tool-only turn
        let tool_only_events = vec![
            json!({
                "type": "response_item",
                "payload": {
                    "type": "function_call",
                    "name": "spawn_agent",
                    "arguments": "{}"
                }
            }),
        ];
        assert_eq!(
            turn_end_subject(Some(tool_only_events)),
            Some("(turn ended)".to_string())
        );

        // Whitespace-only turn
        let ws_events = vec![
            json!({
                "type": "response_item",
                "payload": {
                    "type": "message",
                    "role": "assistant",
                    "phase": "final_answer",
                    "content": [{"type": "output_text", "text": "   \n\t  \n"}]
                }
            }),
        ];
        assert_eq!(
            turn_end_subject(Some(ws_events)),
            Some("(turn ended)".to_string())
        );
    }

    #[test]
    fn concurrent_session_isolation() {
        let _perf_guard = lock_perf_env();
        let _transcript_guard = lock_transcript_env();
        let perf = tempfile::tempdir().unwrap();
        let claude_config = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("BEEHIVE_PERF_DIR", perf.path()) };
        unsafe { std::env::set_var("CLAUDE_CONFIG_DIR", claude_config.path()) };

        let fx1 = fixture();
        let root1 = dunce::canonicalize(fx1.path()).unwrap();
        write_json_file(&root1.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root1.join(".bee").join("state.json"), &json!({"phase": "idle"}));

        let fx2 = fixture();
        let root2 = dunce::canonicalize(fx2.path()).unwrap();
        write_json_file(&root2.join(".bee").join("config.json"), &json!({}));
        write_json_file(&root2.join(".bee").join("state.json"), &json!({"phase": "idle"}));

        // Write distinct transcripts for s-1 and s-2 in their respective project roots
        write_transcript(
            claude_config.path(),
            &root1,
            "s-1",
            &[r#"{"type":"assistant","message":{"content":[{"type":"text","text":"s1 waiting for user"}]}}"#],
        );
        write_transcript(
            claude_config.path(),
            &root2,
            "s-2",
            &[r#"{"type":"assistant","message":{"content":[{"type":"text","text":"s2 waiting for approval"}]}}"#],
        );

        // Verify transcript resolution isolation
        assert_eq!(
            resolve_transcript_for(&root1, Some("s-1")),
            Some(claude_config.path().join("projects").join(encode_project_dir(&root1.to_string_lossy())).join("s-1.jsonl"))
        );
        assert_eq!(resolve_transcript_for(&root1, Some("s-2")), None);
        assert_eq!(
            resolve_transcript_for(&root2, Some("s-2")),
            Some(claude_config.path().join("projects").join(encode_project_dir(&root2.to_string_lossy())).join("s-2.jsonl"))
        );
        assert_eq!(resolve_transcript_for(&root2, Some("s-1")), None);

        // Stop s-1
        let body1 = json!({
            "hook_event_name": "Stop",
            "cwd": root1.to_string_lossy(),
            "session_id": "s-1",
        });
        assert_eq!(run_inner(&[], &serde_json::to_string(&body1).unwrap()), Ok(()));

        let state1: Value = serde_json::from_str(
            &std::fs::read_to_string(root1.join(".bee").join("state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(state1["waiting_on"]["session"], "s-1");
        assert_eq!(state1["waiting_on"]["subject"], "s1 waiting for user");

        // Stop s-2
        let body2 = json!({
            "hook_event_name": "Stop",
            "cwd": root2.to_string_lossy(),
            "session_id": "s-2",
        });
        assert_eq!(run_inner(&[], &serde_json::to_string(&body2).unwrap()), Ok(()));

        let state2: Value = serde_json::from_str(
            &std::fs::read_to_string(root2.join(".bee").join("state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(state2["waiting_on"]["session"], "s-2");
        assert_eq!(state2["waiting_on"]["subject"], "s2 waiting for approval");

        unsafe { std::env::remove_var("BEEHIVE_PERF_DIR") };
        unsafe { std::env::remove_var("CLAUDE_CONFIG_DIR") };
    }

    #[test]
    fn codex_repeated_token_count_dedup_and_multiple_requests_in_turn() {
        let events = vec![
            // Model settings
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "thread_settings_applied",
                    "thread_settings": { "model": "o3-mini" }
                }
            }),
            // Turn starts
            json!({
                "type": "event_msg",
                "payload": { "type": "task_started", "turn_id": "turn-1" }
            }),
            // First LLM request: info has no turn_id/response_id.
            // Emitted four times identically as observed in live Codex transcripts!
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 1000,
                            "cached_input_tokens": 800,
                            "output_tokens": 100,
                            "total_tokens": 1100
                        },
                        "total_token_usage": {
                            "input_tokens": 1000,
                            "cached_input_tokens": 800,
                            "output_tokens": 100,
                            "total_tokens": 1100
                        }
                    }
                }
            }),
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 1000,
                            "cached_input_tokens": 800,
                            "output_tokens": 100,
                            "total_tokens": 1100
                        },
                        "total_token_usage": {
                            "input_tokens": 1000,
                            "cached_input_tokens": 800,
                            "output_tokens": 100,
                            "total_tokens": 1100
                        }
                    }
                }
            }),
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 1000,
                            "cached_input_tokens": 800,
                            "output_tokens": 100,
                            "total_tokens": 1100
                        },
                        "total_token_usage": {
                            "input_tokens": 1000,
                            "cached_input_tokens": 800,
                            "output_tokens": 100,
                            "total_tokens": 1100
                        }
                    }
                }
            }),
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 1000,
                            "cached_input_tokens": 800,
                            "output_tokens": 100,
                            "total_tokens": 1100
                        },
                        "total_token_usage": {
                            "input_tokens": 1000,
                            "cached_input_tokens": 800,
                            "output_tokens": 100,
                            "total_tokens": 1100
                        }
                    }
                }
            }),
            // Second LLM request within the same turn: total_token_usage advances
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 1500,
                            "cached_input_tokens": 1200,
                            "output_tokens": 150,
                            "total_tokens": 1650
                        },
                        "total_token_usage": {
                            "input_tokens": 2500,
                            "cached_input_tokens": 2000,
                            "output_tokens": 250,
                            "total_tokens": 2750
                        }
                    }
                }
            }),
            // Duplicate of second request
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 1500,
                            "cached_input_tokens": 1200,
                            "output_tokens": 150,
                            "total_tokens": 1650
                        },
                        "total_token_usage": {
                            "input_tokens": 2500,
                            "cached_input_tokens": 2000,
                            "output_tokens": 250,
                            "total_tokens": 2750
                        }
                    }
                }
            }),
        ];

        let agg = aggregate_usage(&events);
        assert_eq!(agg.models.0.len(), 1);
        let (model, acc) = &agg.models.0[0];
        assert_eq!(model, "o3-mini");
        // Dedup must NOT count identical records 4 times:
        // First request: uncached input = 1000 - 800 = 200, output = 100, cached = 800
        // Second request: uncached input = 1500 - 1200 = 300, output = 150, cached = 1200
        // Total uncached input = 500
        assert_eq!(acc.input, 500.0);
        assert_eq!(acc.output, 250.0);
        assert_eq!(acc.cache_read, 2000.0);
        assert_eq!(acc.total, 2750.0);
    }

    #[test]
    fn codex_input_tokens_includes_cached_normalization() {
        // Observed live Codex record: input=202967, cached=194560, output=440, total=203407
        let events = vec![
            json!({
                "type": "turn_context",
                "payload": { "model": "o3-mini" }
            }),
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 202967,
                            "cached_input_tokens": 194560,
                            "cache_write_input_tokens": 0,
                            "output_tokens": 440,
                            "total_tokens": 203407
                        },
                        "total_token_usage": {
                            "input_tokens": 202967,
                            "cached_input_tokens": 194560,
                            "cache_write_input_tokens": 0,
                            "output_tokens": 440,
                            "total_tokens": 203407
                        }
                    }
                }
            }),
        ];
        let agg = aggregate_usage(&events);
        let (_, acc) = &agg.models.0[0];
        // Normalized uncached input = 202967 - 194560 = 8407
        assert_eq!(acc.input, 8407.0);
        assert_eq!(acc.cache_read, 194560.0);
        assert_eq!(acc.output, 440.0);
        // Total must equal raw input + output = 203407, NOT double-counting cached tokens!
        assert_eq!(acc.total, 203407.0);
    }

    #[test]
    fn codex_thread_settings_applied_nested_and_no_past_model_assignment() {
        let events = vec![
            // Session starts with model A
            json!({
                "type": "turn_context",
                "payload": { "model": "model-a" }
            }),
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": { "input_tokens": 100, "cached_input_tokens": 0, "output_tokens": 20, "total_tokens": 120 },
                        "total_token_usage": { "input_tokens": 100, "cached_input_tokens": 0, "output_tokens": 20, "total_tokens": 120 }
                    }
                }
            }),
            // Model changes mid-session via nested event_msg thread_settings_applied
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "thread_settings_applied",
                    "thread_settings": { "model": "model-b" }
                }
            }),
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": { "input_tokens": 200, "cached_input_tokens": 0, "output_tokens": 40, "total_tokens": 240 },
                        "total_token_usage": { "input_tokens": 300, "cached_input_tokens": 0, "output_tokens": 60, "total_tokens": 360 }
                    }
                }
            }),
        ];

        let agg = aggregate_usage(&events);
        assert_eq!(agg.models.0.len(), 2);
        // Past record must remain with model-a, NOT assigned to model-b!
        assert_eq!(agg.models.0[0].0, "model-a");
        assert_eq!(agg.models.0[0].1.input, 100.0);
        assert_eq!(agg.models.0[0].1.output, 20.0);
        assert_eq!(agg.models.0[1].0, "model-b");
        assert_eq!(agg.models.0[1].1.input, 200.0);
        assert_eq!(agg.models.0[1].1.output, 40.0);
    }

    #[test]
    fn codex_turn_boundary_latest_tool_only_turn_does_not_reuse_old_final() {
        // Counterexample:
        // old task_started; old final_answer OLD; old task_complete;
        // new task_started; new function_call
        let events = vec![
            json!({
                "type": "event_msg",
                "payload": { "type": "task_started", "turn_id": "turn-1" }
            }),
            json!({
                "type": "response_item",
                "payload": {
                    "type": "message",
                    "role": "assistant",
                    "phase": "final_answer",
                    "content": [{ "type": "output_text", "text": "OLD answer" }]
                }
            }),
            json!({
                "type": "event_msg",
                "payload": { "type": "task_complete", "turn_id": "turn-1", "last_agent_message": "OLD answer" }
            }),
            json!({
                "type": "event_msg",
                "payload": { "type": "task_started", "turn_id": "turn-2" }
            }),
            json!({
                "type": "response_item",
                "payload": {
                    "type": "function_call",
                    "name": "shell",
                    "arguments": "{}"
                }
            }),
        ];

        // The latest turn (turn-2) is tool-only; it must NOT return "OLD answer"!
        let subject = turn_end_subject(Some(events));
        assert_eq!(subject, Some("(turn ended)".to_string()));
    }

    #[test]
    fn codex_rollup_preserves_full_uuid_when_session_meta_missing() {
        let file = PathBuf::from("/path/to/rollout-2026-09-12T14-28-28-01a095e4-8306-70d0-bf1f-7a48da530e71.jsonl");
        let events = vec![
            json!({
                "type": "turn_context",
                "payload": { "model": "o3-mini" }
            }),
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": { "input_tokens": 10, "output_tokens": 5 },
                        "total_token_usage": { "input_tokens": 10, "output_tokens": 5 }
                    }
                }
            }),
        ];

        let rollup = rollup_from_events(&file, &events).expect("rollup must succeed");
        // Must preserve the FULL 36-character UUID, NOT truncate to "7a48da530e71"!
        assert_eq!(rollup.session_id, "01a095e4-8306-70d0-bf1f-7a48da530e71");
    }

    #[test]
    fn codex_rollup_does_not_use_response_item_payload_id_as_session_id() {
        let file = PathBuf::from("/path/to/rollout-2026-09-12T14-28-28-01a095e4-8306-70d0-bf1f-7a48da530e71.jsonl");
        let events = vec![
            json!({
                "type": "turn_context",
                "payload": { "model": "o3-mini" }
            }),
            json!({
                "type": "response_item",
                "payload": {
                    "id": "msg_123",
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "text", "text": "Hello"}]
                }
            }),
            json!({
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "last_token_usage": { "input_tokens": 10, "output_tokens": 5 },
                        "total_token_usage": { "input_tokens": 10, "output_tokens": 5 }
                    }
                }
            }),
        ];

        let rollup = rollup_from_events(&file, &events).expect("rollup must succeed");
        // Must NOT use "msg_123" from response_item payload.id!
        assert_eq!(rollup.session_id, "01a095e4-8306-70d0-bf1f-7a48da530e71");
    }

    #[test]
    fn claude_turn_end_subject_preserves_assistant_text_across_tool_result() {
        let events = vec![
            json!({
                "type": "assistant",
                "message": {
                    "content": [
                        { "type": "text", "text": "Keep existing subject" }
                    ]
                }
            }),
            json!({
                "type": "user",
                "message": {
                    "content": [
                        {
                            "type": "tool_result",
                            "tool_use_id": "tool_123",
                            "content": "some result"
                        }
                    ]
                }
            }),
            json!({
                "type": "assistant",
                "message": {
                    "content": [
                        {
                            "type": "tool_use",
                            "id": "tool_456",
                            "name": "bash",
                            "input": {}
                        }
                    ]
                }
            }),
        ];

        let subject = turn_end_subject(Some(events));
        assert_eq!(subject, Some("Keep existing subject".to_string()));
    }

    #[test]
    fn extract_validated_session_id_from_stem_safety_and_rejection() {
        // Safe UTF-8 handling: malformed Unicode slicing must not panic
        let non_boundary_stem = format!("A日{}", "x".repeat(34));
        assert_eq!(extract_validated_session_id_from_stem(&non_boundary_stem), None);

        let unicode_rollout = "rollout-2026-🎉-something";
        assert_eq!(extract_validated_session_id_from_stem(unicode_rollout), None);

        // Reject arbitrary rollout suffix without timestamp or uuid
        assert_eq!(extract_validated_session_id_from_stem("rollout-arbitrary-suffix"), None);
        assert_eq!(extract_validated_session_id_from_stem("rollout-invalid-uuid-format"), None);

        // Valid timestamped rollout with custom session slug
        assert_eq!(
            extract_validated_session_id_from_stem("rollout-2026-09-12T14-28-28-s-codex-1"),
            Some("s-codex-1".to_string())
        );

        // Valid timestamped rollout with UUID
        assert_eq!(
            extract_validated_session_id_from_stem("rollout-2026-09-12T14-28-28-01a095e4-8306-70d0-bf1f-7a48da530e71"),
            Some("01a095e4-8306-70d0-bf1f-7a48da530e71".to_string())
        );

        // Valid direct UUID rollout
        assert_eq!(
            extract_validated_session_id_from_stem("rollout-01a095e4-8306-70d0-bf1f-7a48da530e71"),
            Some("01a095e4-8306-70d0-bf1f-7a48da530e71".to_string())
        );

        // Non-rollout standard session id
        assert_eq!(
            extract_validated_session_id_from_stem("sess-1"),
            Some("sess-1".to_string())
        );
    }


