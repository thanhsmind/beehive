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
        assert!(text.ends_with("resume cleanly."));
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

    #[test]
    fn rollup_and_upsert_roundtrip_in_isolated_perf_dir() {
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
