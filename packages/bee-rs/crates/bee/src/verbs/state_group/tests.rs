// Split out of the single 6.1k-line verbs/state_group.rs. Code unchanged; only module placement and item visibility moved.
//
// Moved verbatim out of the parent file's `#[cfg(test)] mod tests`,
// indentation and all: the fixtures are raw strings whose leading
// whitespace is content.

// The parent module's own `use` block travels with the tests: they reach
// for names mod.rs no longer imports now that the code using them lives
// in sibling modules.
#![allow(unused_imports)]

use crate::fsutil::{
    append_jsonl, ensure_dir, read_json, warn_corrupt_json, write_json_atomic, ReadJson,
};
use crate::jsjson;
use crate::lock::{self, AcquireOnce, LockGuard};
use crate::verbs::reservations::{
    date_parse_val, finish, iso_from_ms, jget, js_disp, js_disp_opt,
    js_numberify, js_strict_eq, js_trim, keys_known, now_iso, now_ms, parse_flags, prelude, truthy,
    Ctx, Err2, Ex, Exotic, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::reservations::{list_reservations, paths_overlap, rebuild_reservations_projection};
use crate::verbs::workspace_store as ws;
use crate::verbs::workflow_store::{
    acquire_named_lock, acquire_workflow_lock, adopt_mailbox_handoff, create_workflow,
    find_live_workflow, NewWorkflow,
    gates_patch_from_record, lane_lock_name, lane_path, list_lanes, list_workflows,
    newest_open_handoff_mailbox_record, projection_lock_name, read_lane_display, read_lane_strict,
    rebuild_handoff_projection, rebuild_handoff_projection_reporting, rebuild_lane_projection,
    rebuild_lane_projection_reporting, rebuild_state_projection,
    rebuild_state_projection_reporting, update_workflow, update_workflow_assuming_lock,
    update_workflow_assuming_lock_with, wf_id, workflows_list_sort, write_lane,
    write_mailbox_handoff, MailboxAdopt,
};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
use std::time::Instant;
    use super::*;
    use crate::verbs::workflow_store::{
        lanes_dir, list_handoff_mailbox, workflow_state_path, workflows_dir,
    };

    fn tmp_root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    /// Err2/Exotic carry no Debug impl (reservations.rs owns them) — a local
    /// expect keeps `Result` unwrapping panics readable without editing it.
    fn ok<T, E>(r: Result<T, E>) -> T {
        match r {
            Ok(v) => v,
            Err(_) => panic!("unexpected error result"),
        }
    }

    fn write_state_file(root: &Path, content: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("state.json"), content).unwrap();
    }

    fn obj(s: &str) -> Map<String, Value> {
        match serde_json::from_str::<Value>(s).unwrap() {
            Value::Object(m) => m,
            _ => panic!("expected object"),
        }
    }

    // ── state start-feature ───────────────────────────────────────────────

    /// seedLegacyWorkflows is once-per-repo and materializes the LIVE legacy
    /// records only — never the feature this same call is about to write.
    #[test]
    fn seed_legacy_workflows_is_gated_and_carries_gates_over() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_state_file(
            root,
            r#"{"schema_version":"1.0","phase":"swarming","feature":"legacy-feat","mode":"standard",
                "approved_gates":{"context":true,"shape":false,"execution":false,"review":false},
                "summary":"legacy summary","next_action":"legacy next"}"#,
        );
        // A TERMINAL lane is never seeded; a live one is.
        std::fs::create_dir_all(lanes_dir(root)).unwrap();
        for (feature, phase) in [("dead-lane", "compounding-complete"), ("live-lane", "planning")] {
            std::fs::write(
                lanes_dir(root).join(format!("{feature}.json")),
                format!(
                    "{{\"schema_version\":\"1.0\",\"feature\":\"{feature}\",\"phase\":\"{phase}\",\"mode\":null,\"approved_gates\":{{}},\"summary\":\"\",\"next_action\":\"\"}}"
                ),
            )
            .unwrap();
        }

        ok(seed_legacy_workflows(root, root));
        let seeded = ok(list_workflows(root));
        let mut features: Vec<String> = seeded.iter().map(|w| js_disp_opt(w.get("feature"))).collect();
        features.sort();
        assert_eq!(features, vec!["legacy-feat", "live-lane"]);

        // The default record's gates rode across as
        // `{approved, approved_for_plan_rev: null}` per gate.
        let legacy = seeded
            .iter()
            .find(|w| js_disp_opt(w.get("feature")) == "legacy-feat")
            .unwrap();
        assert_eq!(legacy["gates"]["context"]["approved"], Value::Bool(true));
        assert_eq!(legacy["gates"]["shape"]["approved"], Value::Bool(false));
        assert_eq!(legacy["gates"]["context"]["approved_for_plan_rev"], Value::Null);
        assert_eq!(legacy["mode"], json!("standard"));
        assert_eq!(legacy["summary"], json!("legacy summary"));

        // Never re-seed once ANY record exists: a second call is a no-op even
        // after a new live lane appears.
        std::fs::write(
            lanes_dir(root).join("later-lane.json"),
            r#"{"schema_version":"1.0","feature":"later-lane","phase":"planning","mode":null,"approved_gates":{},"summary":"","next_action":""}"#,
        )
        .unwrap();
        ok(seed_legacy_workflows(root, root));
        assert_eq!(ok(list_workflows(root)).len(), 2);
    }

    /// ensureWorkflowRecordForFeature is idempotent BY FEATURE — never a
    /// second record, and never a silent overwrite of the live one's phase.
    #[test]
    fn ensure_workflow_record_is_idempotent_by_feature() {
        let tmp = tmp_root();
        let root = tmp.path();
        ok(ensure_workflow_record_for_feature(
            root, "feat-a", "planning", Some("standard"), Some(&json!("s")), Some(&json!("n")), None,
        ));
        let first = ok(list_workflows(root));
        assert_eq!(first.len(), 1);
        assert_eq!(first[0]["phase"], json!("planning"));

        // Same feature, different phase: adopted, not duplicated or moved.
        ok(ensure_workflow_record_for_feature(
            root, "feat-a", "swarming", None, None, None, None,
        ));
        let second = ok(list_workflows(root));
        assert_eq!(second.len(), 1);
        assert_eq!(second[0]["phase"], json!("planning"));

        // An UNKNOWN phase degrades to 'idle' rather than refusing.
        ok(ensure_workflow_record_for_feature(root, "feat-b", "nope", None, None, None, None));
        let third = ok(list_workflows(root));
        let b = third.iter().find(|w| js_disp_opt(w.get("feature")) == "feat-b").unwrap();
        assert_eq!(b["phase"], json!("idle"));

        // A CLOSED record for the feature is not "live" — a new one is made.
        let id = wf_id(b);
        let mut patch = Map::new();
        patch.insert("status".into(), json!("closed"));
        ok(update_workflow(root, &id, patch));
        ok(ensure_workflow_record_for_feature(root, "feat-b", "planning", None, None, None, None));
        assert_eq!(ok(list_workflows(root)).len(), 3);
    }

    /// closeWorkflowsForFeature closes BY FEATURE — every live record whose
    /// feature differs from `keep`, and is idempotent on a second call.
    #[test]
    fn close_workflows_for_feature_keeps_only_the_named_one() {
        let tmp = tmp_root();
        let root = tmp.path();
        for feature in ["keep-me", "drop-a", "drop-b"] {
            ok(ensure_workflow_record_for_feature(root, feature, "planning", None, None, None, None));
        }
        ok(close_workflows_for_feature(root, Some("keep-me")));
        let after = ok(list_workflows(root));
        for record in &after {
            let expected = if js_disp_opt(record.get("feature")) == "keep-me" { "active" } else { "closed" };
            assert_eq!(js_disp_opt(record.get("status")), expected);
        }
        // Idempotent.
        ok(close_workflows_for_feature(root, Some("keep-me")));
        assert_eq!(
            ok(list_workflows(root)).iter().filter(|r| js_disp_opt(r.get("status")) != "closed").count(),
            1
        );
        // `keep: None` is a full wind-down.
        ok(close_workflows_for_feature(root, None));
        assert!(ok(list_workflows(root)).iter().all(|r| js_disp_opt(r.get("status")) == "closed"));
    }

    /// The two shared workflow preconditions name the conflicting record /
    /// cells, byte-for-byte.
    #[test]
    fn workflow_preconditions_name_the_conflict() {
        let workflows = vec![
            obj(r#"{"id":"wf-1","feature":"taken","phase":"swarming","status":"active"}"#),
            obj(r#"{"id":"wf-2","feature":"done","phase":"idle","status":"closed"}"#),
        ];
        assert_eq!(check_no_live_workflow_for_feature(&workflows, "free"), None);
        assert_eq!(check_no_live_workflow_for_feature(&workflows, "done"), None);
        assert_eq!(
            check_no_live_workflow_for_feature(&workflows, "taken").unwrap(),
            "startFeature: refused — a live workflow already exists for feature \"taken\" (workflow wf-1, phase \"swarming\", status \"active\"). FIX: close or resolve that workflow before starting a new one for the same feature."
        );

        let cells = vec![
            obj(r#"{"id":"c-1","feature":"f","status":"claimed"}"#),
            obj(r#"{"id":"c-2","feature":"f","status":"open"}"#),
            obj(r#"{"id":"c-3","feature":"g","status":"claimed"}"#),
        ];
        assert_eq!(check_no_same_feature_claimed_cells("g2", &cells), None);
        assert_eq!(
            check_no_same_feature_claimed_cells("f", &cells).unwrap(),
            "startFeature: refused — feature \"f\" already has claimed cell(s): c-1. FIX: cap or drop them first (bee cells cap / bee cells drop)."
        );
    }

    /// The isolate-notice marker path replaces BOTH separators, so a session
    /// id that looks like a path can never escape the notices directory.
    #[test]
    fn isolate_notice_marker_is_path_safe() {
        let tmp = tmp_root();
        let marker = isolate_notice_marker_path(tmp.path(), "a/b\\c");
        assert_eq!(marker.file_name().unwrap().to_str().unwrap(), "a_b_c.json");
        assert!(marker.starts_with(tmp.path().join(".bee").join("runtime").join("notices")));
    }

    /// activeWorkers is a DERIVED view: live-heartbeat sessions joined with
    /// their first active claim, with the calling session excluded (C3).
    #[test]
    fn active_workers_joins_live_sessions_to_claims() {
        let tmp = tmp_root();
        let root = tmp.path();
        std::fs::create_dir_all(sessions_dir(root)).unwrap();
        std::fs::create_dir_all(claims_dir(root)).unwrap();
        let fresh = now_iso();
        for id in ["live-1", "mine"] {
            std::fs::write(
                sessions_dir(root).join(format!("{id}.json")),
                format!("{{\"id\":\"{id}\",\"last_heartbeat\":\"{fresh}\"}}"),
            )
            .unwrap();
        }
        std::fs::write(
            sessions_dir(root).join("stale.json"),
            "{\"id\":\"stale\",\"last_heartbeat\":\"2020-01-01T00:00:00.000Z\"}",
        )
        .unwrap();
        std::fs::write(
            claims_dir(root).join("c-1.json"),
            format!("{{\"cell\":\"c-1\",\"session\":\"live-1\",\"claimed_at\":\"{fresh}\",\"ttl_seconds\":3600}}"),
        )
        .unwrap();
        // An EXPIRED claim never contributes a cell.
        std::fs::write(
            claims_dir(root).join("c-2.json"),
            "{\"cell\":\"c-2\",\"session\":\"mine\",\"claimed_at\":\"2020-01-01T00:00:00.000Z\",\"ttl_seconds\":1}",
        )
        .unwrap();

        let all = ok(active_workers(root, None));
        let mut rows: Vec<(String, String)> = all
            .iter()
            .map(|w| (w.session_id.clone(), js_disp_opt(w.cell.as_ref())))
            .collect();
        rows.sort();
        assert_eq!(
            rows,
            vec![("live-1".to_string(), "c-1".to_string()), ("mine".to_string(), "undefined".to_string())]
        );

        // C3: the calling session's own heartbeat is never "another" worker.
        let others = ok(active_workers(root, Some("mine")));
        assert_eq!(others.len(), 1);
        assert_eq!(others[0].session_id, "live-1");
    }

    /// listAllCellsForStart reads objects only. CUTOVER: a CORRUPT cell used
    /// to be Exotic (Node's readJson warned with a V8 message); it is now
    /// warned about and SKIPPED, which is the same list `readJson`'s `null`
    /// fallback produced in Node — so the preflight no longer delegates.
    #[test]
    fn cells_for_start_skips_non_objects_and_warns_past_a_corrupt_record() {
        let tmp = tmp_root();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee").join("cells")).unwrap();
        std::fs::write(root.join(".bee/cells/c-1.json"), "{\"id\":\"c-1\"}").unwrap();
        std::fs::write(root.join(".bee/cells/c-2.json"), "[1,2]").unwrap();
        std::fs::write(root.join(".bee/cells/notes.txt"), "ignored").unwrap();
        assert_eq!(ok(list_all_cells_for_start(root)).len(), 1);
        std::fs::write(root.join(".bee/cells/c-3.json"), "{oops").unwrap();
        let cells = ok(list_all_cells_for_start(root));
        assert_eq!(cells.len(), 1, "the corrupt record is skipped, the good one survives");
        assert_eq!(cells[0].get("id"), Some(&json!("c-1")));
        assert!(preflight(root, root).is_ok(), "a corrupt cell no longer delegates");
    }

    /// The lone-surrogate class: V8's JSON.parse accepted `"\uD800"`, serde
    /// refuses it, and no Rust `String` can hold it — so it is CORRUPT and
    /// takes each caller's corrupt branch instead of delegating.
    #[test]
    fn lone_surrogate_escapes_are_corrupt_not_delegated() {
        assert!(matches!(parse_json_v8("{\"a\":\"\\uD800\"}"), Ok(ParsedJson::Unparseable)));
        let tmp = tmp_root();
        write_state_file(tmp.path(), "{\"phase\":\"\\uD800\"}");
        // The fail-open peek falls back to defaults, as any corrupt file does.
        assert_eq!(ok(read_state_peek(tmp.path())).get("phase"), Some(&json!("idle")));
        // The strict read still REFUSES, with the unparseable message.
        match read_state_strict(tmp.path()) {
            Err(Err2::Msg(m)) => assert!(m.contains("exists but is not valid JSON"), "{m}"),
            _ => panic!("expected the unparseable refusal"),
        }
    }

    /// Every fail-open reader warns ONCE and takes readJson's own fallback.
    #[test]
    fn corrupt_reads_fail_open_with_one_warning_each() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_state_file(root, "{broken");
        assert_eq!(ok(read_state_peek(root)).get("phase"), Some(&json!("idle")));

        std::fs::create_dir_all(root.join(".bee").join("sessions")).unwrap();
        std::fs::write(root.join(".bee/sessions/s-1.json"), "{broken").unwrap();
        assert!(ok(read_session(root, "s-1")).is_none());

        std::fs::create_dir_all(root.join(".bee").join("claims")).unwrap();
        std::fs::write(root.join(".bee/claims/c-1.json"), "{broken").unwrap();
        assert!(ok(read_claim(root, "c-1")).is_none());

        std::fs::write(handoff_path(root), "{broken").unwrap();
        assert!(ok(read_handoff(root)).is_none());
    }

    /// The read-failure refusal keeps its typed shape; only the parenthetical
    /// is ours now, and a DIRECTORY in the file's place no longer delegates.
    #[test]
    fn read_state_strict_unreadable_names_an_engine_free_category() {
        let tmp = tmp_root();
        std::fs::create_dir_all(tmp.path().join(".bee").join("state.json")).unwrap();
        match read_state_strict(tmp.path()) {
            Err(Err2::Msg(m)) => {
                assert!(m.starts_with("readStateStrict: could not read "), "{m}");
                assert!(m.contains("(the path is a directory)"), "{m}");
                assert!(!m.contains("EISDIR"), "no libuv errno string: {m}");
            }
            _ => panic!("expected the unreadable refusal"),
        }
    }

    // ── gate-door rules (checkPhaseTransition) ────────────────────────────

    #[test]
    fn compounding_is_never_settable_directly() {
        let t = check_phase_transition(Some(&json!("swarming")), "compounding", &Map::new(), false)
            .ok()
            .unwrap();
        assert!(!t.ok);
        assert!(t.reason.starts_with(
            "set: phase \"compounding\" is not settable directly — it is produced only by RECORDING"
        ));
    }

    #[test]
    fn compounding_complete_requires_compounding_phase() {
        let t = check_phase_transition(
            Some(&json!("swarming")),
            "compounding-complete",
            &Map::new(),
            false,
        )
        .ok()
        .unwrap();
        assert!(!t.ok);
        assert!(t.reason.contains(
            "may only be entered from \"compounding\" (current: \"swarming\")"
        ));
        // Falsy phase reads as idle.
        let t2 = check_phase_transition(None, "compounding-complete", &Map::new(), false)
            .ok()
            .unwrap();
        assert!(t2.reason.contains("(current: \"idle\")"));
    }

    #[test]
    fn compounding_complete_requires_fresh_recorded_run() {
        let rec = obj(
            r#"{"feature":"f1","last_scribing_run":{"feature":"f1","at":"2026-07-01T00:00:00.000Z"}}"#,
        );
        let t = check_phase_transition(Some(&json!("compounding")), "compounding-complete", &rec, false)
            .ok()
            .unwrap();
        assert!(!t.ok);
        assert!(t.reason.contains("no fresh compounding run recorded for feature \"f1\""));
        // Waived: passes with waived_compounding flagged.
        let t2 = check_phase_transition(Some(&json!("compounding")), "compounding-complete", &rec, true)
            .ok()
            .unwrap();
        assert!(t2.ok && t2.waived_compounding);
        // Stale run (before the scribing stamp) is not fresh.
        let rec3 = obj(
            r#"{"last_scribing_run":{"feature":"f1","at":"2026-07-02T00:00:00.000Z"},"last_compounding_run":{"feature":"f1","at":"2026-07-01T00:00:00.000Z"}}"#,
        );
        let t3 = check_phase_transition(Some(&json!("compounding")), "compounding-complete", &rec3, false)
            .ok()
            .unwrap();
        assert!(!t3.ok);
        // Fresh same-feature run at-or-after the scribing stamp passes clean.
        let rec4 = obj(
            r#"{"last_scribing_run":{"feature":"f1","at":"2026-07-01T00:00:00.000Z"},"last_compounding_run":{"feature":"f1","at":"2026-07-01T00:00:00.000Z"}}"#,
        );
        let t4 = check_phase_transition(Some(&json!("compounding")), "compounding-complete", &rec4, false)
            .ok()
            .unwrap();
        assert!(t4.ok && !t4.waived_compounding);
        // A mismatched feature is never fresh.
        let rec5 = obj(
            r#"{"last_scribing_run":{"feature":"f1","at":"2026-07-01T00:00:00.000Z"},"last_compounding_run":{"feature":"other","at":"2026-07-03T00:00:00.000Z"}}"#,
        );
        let t5 = check_phase_transition(Some(&json!("compounding")), "compounding-complete", &rec5, false)
            .ok()
            .unwrap();
        assert!(!t5.ok);
    }

    #[test]
    fn backward_moves_and_idle_stay_permissive() {
        for to in ["idle", "exploring", "planning", "swarming", "grooming"] {
            let t = check_phase_transition(Some(&json!("reviewing")), to, &Map::new(), false)
                .ok()
                .unwrap();
            assert!(t.ok, "transition to {to} must be permissive");
        }
    }

    #[test]
    fn scribing_and_compounding_run_doors() {
        assert!(check_scribing_run_phase(Some(&json!("swarming"))).is_none());
        let refuse = check_scribing_run_phase(Some(&json!("idle"))).unwrap();
        assert!(refuse.contains("refused from phase \"idle\""));
        assert!(refuse.contains("Legal from: swarming, reviewing, scribing."));
        assert!(check_compounding_run_phase(Some(&json!("compounding"))).is_none());
        let refuse2 = check_compounding_run_phase(Some(&json!("swarming"))).unwrap();
        assert!(refuse2.contains("compounding-run: refused from phase \"swarming\""));
    }

    // ── readStateStrict's typed errors ────────────────────────────────────

    #[test]
    fn read_state_strict_missing_yields_defaults() {
        let tmp = tmp_root();
        let state = read_state_strict(tmp.path()).ok().unwrap();
        assert_eq!(
            jsjson::stringify(&Value::Object(state)),
            r#"{"schema_version":"1.0","phase":"idle","feature":null,"mode":null,"approved_gates":{"context":false,"shape":false,"execution":false,"review":false},"workers":[],"summary":"","next_action":"No active bee work — awaiting a user request."}"#
        );
    }

    #[test]
    fn read_state_strict_unparseable_message_is_exact() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), "{broken");
        let file = tmp.path().join(".bee").join("state.json");
        match read_state_strict(tmp.path()) {
            Err(Err2::Msg(m)) => {
                assert_eq!(
                    m,
                    format!(
                        "readStateStrict: \"{}\" exists but is not valid JSON. The bee CLI refuses to rebuild state from defaults over a present-but-corrupt file — that would silently clobber real state (gates, workers, feature) while reporting success. FIX: inspect/restore the file (e.g. \"git checkout -- .bee{}state.json\"), then retry.",
                        file.display(),
                        MAIN_SEPARATOR
                    )
                );
            }
            _ => panic!("expected the unparseable refusal"),
        }
    }

    #[test]
    fn read_state_strict_non_object_names_the_found_type() {
        let cases = [
            ("[1,2]", "an array"),
            ("null", "object"),
            ("42", "number"),
            ("\"x\"", "string"),
            ("true", "boolean"),
        ];
        for (content, found) in cases {
            let tmp = tmp_root();
            write_state_file(tmp.path(), content);
            match read_state_strict(tmp.path()) {
                Err(Err2::Msg(m)) => {
                    assert!(
                        m.contains(&format!("exists but is not a JSON object (found {found})")),
                        "content {content}: {m}"
                    );
                }
                _ => panic!("expected the non-object refusal for {content}"),
            }
        }
    }

    #[test]
    fn read_state_strict_merges_defaults_and_coerces_legacy_phase() {
        let tmp = tmp_root();
        write_state_file(
            tmp.path(),
            r#"{"phase":"validating","feature":"f1","approved_gates":{"shape":true},"extra":1}"#,
        );
        let state = read_state_strict(tmp.path()).ok().unwrap();
        assert_eq!(state.get("phase"), Some(&json!("planning")));
        assert_eq!(state.get("feature"), Some(&json!("f1")));
        assert_eq!(
            jsjson::stringify(state.get("approved_gates").unwrap()),
            r#"{"context":false,"shape":true,"execution":false,"review":false}"#
        );
        // File keys override in the default slot order; extras append.
        let keys: Vec<&String> = state.keys().collect();
        assert_eq!(keys.last().unwrap().as_str(), "extra");
    }

    // ── worker add / prune ────────────────────────────────────────────────

    #[test]
    fn worker_mutate_add_writes_node_shaped_state() {
        let tmp = tmp_root();
        let out = worker_mutate(tmp.path(), |workers| {
            workers.push(json!({"nickname": "w1", "cell": "c1", "tier": Value::Null, "status": Value::Null}));
            Ok("Added worker \"w1\" (cell c1).".to_string())
        });
        match out {
            Ok(Out::Emit(result, text, 0)) => {
                assert_eq!(text, "Added worker \"w1\" (cell c1).");
                let workers = result.get("workers").unwrap();
                assert_eq!(
                    jsjson::stringify(workers),
                    r#"[{"nickname":"w1","cell":"c1","tier":null,"status":null}]"#
                );
            }
            _ => panic!("expected emit"),
        }
        let bytes = std::fs::read_to_string(tmp.path().join(".bee").join("state.json")).unwrap();
        assert!(bytes.ends_with("\n"));
        assert!(bytes.contains("\"nickname\": \"w1\""));
        // The store lock was released.
        assert!(!lock::lock_file_path(tmp.path(), "state").exists());
    }

    #[test]
    fn worker_transient_suffix_and_keep_rules() {
        assert_eq!(worker_transient_suffix_len("c1.prompt.md"), Some(10));
        assert_eq!(worker_transient_suffix_len("c1.result.json"), Some(12));
        assert_eq!(worker_transient_suffix_len("c1.out12.log"), Some(10));
        assert_eq!(worker_transient_suffix_len("c1.out.log"), Some(8));
        assert_eq!(worker_transient_suffix_len("c1.log"), Some(4));
        assert_eq!(worker_transient_suffix_len("c1.txt"), None);
        // Leftmost match: the whole dotted tail is the suffix for a dotted id.
        assert_eq!(worker_transient_suffix_len("a.result.md"), Some(10));
        // Empty stem: the name IS the suffix.
        assert_eq!(worker_transient_suffix_len(".log"), Some(4));
        // Prefix keep-check: "<id>" or "<id>.<anything>", never a mis-stem.
        let keep = vec!["cell.a".to_string()];
        assert!(kept_by_keep_set("cell.a.log", &keep));
        assert!(kept_by_keep_set("cell.a", &keep));
        assert!(!kept_by_keep_set("cell.ab.log", &keep));
    }

    #[test]
    fn prune_keep_set_protects_non_capped_and_corrupt_cells() {
        let tmp = tmp_root();
        write_state_file(
            tmp.path(),
            r#"{"workers":[{"nickname":"w1","cell":"c-keep"},null,"junk"]}"#,
        );
        let cells = tmp.path().join(".bee").join("cells");
        std::fs::create_dir_all(&cells).unwrap();
        std::fs::write(cells.join("open.json"), r#"{"status":"open"}"#).unwrap();
        std::fs::write(cells.join("capped.json"), r#"{"status":"capped"}"#).unwrap();
        std::fs::write(cells.join("corrupt.json"), "{nope").unwrap();
        let keep = ok(read_prune_keep_set(tmp.path()));
        assert!(keep.contains(&"c-keep".to_string()));
        assert!(keep.contains(&"open".to_string()));
        assert!(keep.contains(&"corrupt".to_string()));
        assert!(!keep.contains(&"capped".to_string()));
    }

    #[test]
    fn prune_refuses_malformed_workers() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"workers":"not-an-array"}"#);
        match read_prune_keep_set(tmp.path()) {
            Err(Err2::Msg(m)) => assert!(m.starts_with("worker prune: state.workers is not an array")),
            _ => panic!("expected the fails-closed refusal"),
        }
    }

    // ── handoff kinds ─────────────────────────────────────────────────────

    #[test]
    fn pause_handoff_round_trips_with_kind_and_written_at() {
        let tmp = tmp_root();
        let mut input = Map::new();
        input.insert("kind".into(), json!("pause"));
        input.insert("feature".into(), json!("f1"));
        input.insert("cell".into(), json!("wip-1"));
        let record = ok(write_handoff(tmp.path(), &input, "pause"));
        let keys: Vec<&String> = record.keys().collect();
        assert_eq!(keys, ["feature", "cell", "kind", "written_at"]);
        assert_eq!(record.get("kind"), Some(&json!("pause")));
        // readHandoff normalizes kind (already pause) and reads it back.
        let read = ok(read_handoff(tmp.path())).unwrap();
        assert_eq!(jget(&read, "cell"), Some(&json!("wip-1")));
        // A kindless record on disk reads as pause (the fail-safe).
        std::fs::write(
            handoff_path(tmp.path()),
            "{\n  \"cell\": \"x\"\n}\n",
        )
        .unwrap();
        let read2 = ok(read_handoff(tmp.path())).unwrap();
        assert_eq!(jget(&read2, "kind"), Some(&json!("pause")));
    }

    #[test]
    fn planned_next_refuses_uncapped_previous_and_unowned_claim() {
        let tmp = tmp_root();
        let mut input = Map::new();
        input.insert("kind".into(), json!("planned-next"));
        input.insert("writer_session".into(), json!("sess-w"));
        input.insert("previous_cell".into(), json!("prev"));
        input.insert("next_cell".into(), json!("next"));
        // No previous cell record at all → "missing".
        match write_handoff(tmp.path(), &input, "planned-next") {
            Err(Err2::Msg(m)) => assert!(m.contains(
                "previous cell \"prev\" is not capped (found status \"missing\")"
            )),
            _ => panic!("expected the uncapped refusal"),
        }
        // Capped previous but no claim on next → "no claim".
        let cells = tmp.path().join(".bee").join("cells");
        std::fs::create_dir_all(&cells).unwrap();
        std::fs::write(cells.join("prev.json"), r#"{"status":"capped"}"#).unwrap();
        match write_handoff(tmp.path(), &input, "planned-next") {
            Err(Err2::Msg(m)) => {
                assert!(m.contains("next cell \"next\" has no claim owned by writer session \"sess-w\" (found no claim)"))
            }
            _ => panic!("expected the unowned-claim refusal"),
        }
        // Claim owned by someone else → owner "..." in the refusal.
        let claims = claims_dir(tmp.path());
        std::fs::create_dir_all(&claims).unwrap();
        std::fs::write(claims.join("next.json"), r#"{"cell":"next","session":"other"}"#).unwrap();
        match write_handoff(tmp.path(), &input, "planned-next") {
            Err(Err2::Msg(m)) => assert!(m.contains("(found owner \"other\")")),
            _ => panic!("expected the owner refusal"),
        }
        // Correctly owned claim → the record writes with the stamp fields.
        std::fs::write(claims.join("next.json"), r#"{"cell":"next","session":"sess-w"}"#).unwrap();
        let record = ok(write_handoff(tmp.path(), &input, "planned-next"));
        assert_eq!(record.get("kind"), Some(&json!("planned-next")));
        assert!(record.contains_key("written_at"));
    }

    #[test]
    fn adopt_refuses_pause_and_adopts_planned_next_with_fence_bump() {
        let tmp = tmp_root();
        // Pause handoff is never adopted.
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(
            handoff_path(tmp.path()),
            r#"{"kind":"pause","cell":"x"}"#,
        )
        .unwrap();
        match ok(adopt_handoff(tmp.path(), "sess-new")) {
            HandoffAdopt::Fail { reason } => assert_eq!(
                reason,
                "handoff kind \"pause\" is not \"planned-next\" — a pause handoff is never adopted, it must be surfaced and WAITED on (D1)."
            ),
            _ => panic!("expected the pause refusal"),
        }
        // planned-next with an owned claim adopts: fence bumps, handoff clears.
        std::fs::write(
            handoff_path(tmp.path()),
            r#"{"kind":"planned-next","writer_session":"sess-w","previous_cell":"prev","next_cell":"next"}"#,
        )
        .unwrap();
        let claims = claims_dir(tmp.path());
        std::fs::create_dir_all(&claims).unwrap();
        std::fs::write(
            claims.join("next.json"),
            r#"{"cell":"next","session":"sess-w","fence_epoch":1}"#,
        )
        .unwrap();
        match ok(adopt_handoff(tmp.path(), "sess-new")) {
            HandoffAdopt::Ok { claim, previous_owner, next_cell } => {
                assert_eq!(next_cell, "next");
                assert_eq!(previous_owner, Some(json!("sess-w")));
                assert_eq!(claim.get("session"), Some(&json!("sess-new")));
                assert_eq!(claim.get("adopted_from"), Some(&json!("sess-w")));
                assert_eq!(claim.get("fence_epoch"), Some(&json!(2.0)));
            }
            HandoffAdopt::Fail { reason } => panic!("unexpected refusal: {reason}"),
        }
        assert!(!handoff_path(tmp.path()).exists(), "handoff cleared after adopt");
        // The gate file was released.
        assert!(!claims.join("next.adopting").exists());
        // No handoff left → NO_HANDOFF.
        match ok(adopt_handoff(tmp.path(), "sess-new")) {
            HandoffAdopt::Fail { reason } => assert_eq!(reason, "no .bee/HANDOFF.json to adopt."),
            _ => panic!("expected NO_HANDOFF"),
        }
    }

    #[test]
    fn adopt_gate_held_is_a_typed_refusal() {
        let tmp = tmp_root();
        let claims = claims_dir(tmp.path());
        std::fs::create_dir_all(&claims).unwrap();
        std::fs::write(claims.join("next.json"), r#"{"cell":"next","session":"a"}"#).unwrap();
        std::fs::write(claims.join("next.adopting"), "{}").unwrap();
        match ok(adopt_claim(tmp.path(), "next", "sess-new")) {
            AdoptOutcome::Fail { reason } => assert!(reason.contains("gated by another in-flight adopt/sweep")),
            _ => panic!("expected GATE_HELD"),
        }
        // The pre-existing (foreign) gate file must survive our failed attempt.
        assert!(claims.join("next.adopting").exists());
    }

    // ── the lane/workflow seam (the R6 "C1 gate" is gone) ─────────────────

    fn write_workflow(root: &Path, id: &str, body: Value) {
        let dir = workflows_dir(root).join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("state.json"), serde_json::to_string(&body).unwrap()).unwrap();
    }

    fn read_workflow_file(root: &Path, id: &str) -> Value {
        serde_json::from_str(&std::fs::read_to_string(workflow_state_path(root, id)).unwrap())
            .unwrap()
    }

    fn write_lane_file(root: &Path, feature: &str, content: &str) {
        std::fs::create_dir_all(lanes_dir(root)).unwrap();
        std::fs::write(lanes_dir(root).join(format!("{feature}.json")), content).unwrap();
    }

    /// The session id the resolver will actually look for. resolveSessionId's
    /// env chain (BEE_SESSION_ID / CLAUDE_CODE_SESSION_ID) OUTRANKS single-live
    /// -session adoption, and a Claude Code test runner really does export
    /// CLAUDE_CODE_SESSION_ID — so a fixture that hard-codes "sess-1" would be
    /// invisible to the very code under test. Ask the resolver instead.
    fn fixture_session_id(root: &Path) -> String {
        ok(resolve_session_id_no_flag(root)).unwrap_or_else(|| "sess-1".to_string())
    }

    fn write_session(root: &Path, id: &str, lane: Option<&str>) {
        let dir = sessions_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        let mut rec = Map::new();
        rec.insert("id".into(), json!(id));
        rec.insert("last_heartbeat".into(), json!(now_iso()));
        if let Some(l) = lane {
            rec.insert("lane".into(), json!(l));
        }
        std::fs::write(
            dir.join(format!("{id}.json")),
            jsjson::stringify(&Value::Object(rec)),
        )
        .unwrap();
    }

    #[test]
    fn mutation_scope_follows_lane_then_session_binding_then_default_feature() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"default-f"}"#);
        // Explicit --lane always wins.
        let s = ok(resolve_mutation_lock_scope(tmp.path(), Some("lane-a"), false));
        assert_eq!(s.feature.as_deref(), Some("lane-a"));
        assert!(s.lane);
        assert_eq!(projection_lock_name(s.lane, s.feature.as_deref()), "lane:lane-a");
        // --no-lane forces the default record AND skips session resolution.
        let s = ok(resolve_mutation_lock_scope(tmp.path(), None, true));
        assert!(s.feature.is_none() && !s.lane);
        // A bound session targets its lane.
        let sid = fixture_session_id(tmp.path());
        write_session(tmp.path(), &sid, Some("lane-b"));
        let s = ok(resolve_mutation_lock_scope(tmp.path(), None, false));
        assert_eq!(s.feature.as_deref(), Some("lane-b"));
        assert!(s.lane);
        // Unbound: the default record's own feature, lane = false.
        write_session(tmp.path(), &sid, None);
        let s = ok(resolve_mutation_lock_scope(tmp.path(), None, false));
        assert_eq!(s.feature.as_deref(), Some("default-f"));
        assert!(!s.lane);
        assert_eq!(projection_lock_name(s.lane, s.feature.as_deref()), "state");
    }

    #[test]
    fn lane_resolution_refusals_are_byte_exact_and_never_guess_back() {
        let tmp = tmp_root();
        match resolve_mutation_target(tmp.path(), Some("ghost"), "set", false) {
            Err(Err2::Msg(m)) => assert_eq!(
                m,
                "set: refused — lane \"ghost\" does not exist (no .bee/lanes/ghost.json). FIX: start it first (\"state start-feature --feature ghost --as-lane\"), then retry."
            ),
            _ => panic!("expected the LANE_MISSING refusal"),
        }
        let sid = fixture_session_id(tmp.path());
        write_session(tmp.path(), &sid, Some("ghost"));
        match resolve_mutation_target(tmp.path(), None, "gate", false) {
            Err(Err2::Msg(m)) => assert_eq!(
                m,
                format!("gate: refused — calling session \"{sid}\" is bound to lane \"ghost\" but no .bee/lanes/ghost.json exists; resolution never guesses back to the default record. FIX: start the lane (\"state start-feature --feature ghost --as-lane\"), unbind the session, or pass --no-lane to target the default record explicitly.")
            ),
            _ => panic!("expected the bound-lane refusal"),
        }
        // --no-lane is the documented escape back to the default record.
        assert!(matches!(
            ok(resolve_mutation_target(tmp.path(), None, "gate", true)),
            Target::Default { .. }
        ));
        // A present-but-corrupt lane record refuses instead of defaulting.
        write_lane_file(tmp.path(), "ghost", "{nope");
        assert!(matches!(
            resolve_mutation_target(tmp.path(), Some("ghost"), "set", false),
            Err(Err2::Msg(_))
        ));
    }

    #[test]
    fn lane_mutation_writes_through_the_live_workflow_record() {
        let tmp = tmp_root();
        write_lane_file(
            tmp.path(),
            "f1",
            r#"{"feature":"f1","phase":"planning","created_at":"2026-01-01T00:00:00.000Z"}"#,
        );
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"planning",
                   "plan_rev":2,"created_at":"2026-01-02T00:00:00.000Z"}),
        );
        let mut target = ok(resolve_mutation_target(tmp.path(), Some("f1"), "gate", false));
        assert_eq!(target.selected_record(), "lane \"f1\"");
        assert_eq!(target.lane_note(), " (lane \"f1\")");
        {
            let rec = target.record_mut();
            let mut gates = default_gates();
            gates.insert("execution".into(), json!(true));
            rec.insert("approved_gates".into(), Value::Object(gates));
            rec.insert("phase".into(), json!("swarming"));
        }
        let record = target.record().clone();
        let stamps = vec![("execution".to_string(), json!(2))];
        ok(write_through_projection(tmp.path(), &target, &record, &stamps));
        // The WORKFLOW record took the D1 fields and the plan-rev stamp…
        let wf = read_workflow_file(tmp.path(), "wf-1");
        assert_eq!(wf["phase"], json!("swarming"));
        assert_eq!(wf["feature"], json!("f1"), "identity never patched");
        assert_eq!(
            wf["gates"]["execution"],
            json!({"approved":true,"approved_for_plan_rev":2})
        );
        assert_eq!(
            wf["gates"]["context"],
            json!({"approved":false,"approved_for_plan_rev":null})
        );
        // …and the lane projection was rebuilt FROM it ("record wins").
        let lane = ok(read_lane_strict(tmp.path(), "f1")).unwrap();
        assert_eq!(lane.get("phase"), Some(&json!("swarming")));
        assert_eq!(
            jsjson::stringify(lane.get("approved_gates").unwrap()),
            r#"{"context":false,"shape":false,"execution":true,"review":false}"#
        );
        assert_eq!(lane.get("created_at"), Some(&json!("2026-01-01T00:00:00.000Z")));
        // A lane mutation never touches .bee/state.json.
        assert!(!state_path(tmp.path()).exists());
    }

    #[test]
    fn a_plan_rev_bump_flips_the_stamped_gate_in_the_same_projection() {
        let tmp = tmp_root();
        write_lane_file(tmp.path(), "f1", r#"{"feature":"f1","phase":"planning"}"#);
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"planning","plan_rev":1,
                   "created_at":"2026-01-01T00:00:00.000Z",
                   "gates":{"execution":{"approved":true,"approved_for_plan_rev":1},
                            "context":{"approved":true,"approved_for_plan_rev":null}}}),
        );
        // Before the bump, execution projects effective.
        let lane = ok(rebuild_lane_projection(tmp.path(), "f1")).unwrap();
        assert_eq!(
            jsjson::stringify(lane.get("approved_gates").unwrap()),
            r#"{"context":true,"shape":false,"execution":true,"review":false}"#
        );
        let updated = ok(update_workflow_assuming_lock_with(tmp.path(), "wf-1", |current| {
            let base = current.get("plan_rev").and_then(Value::as_f64).unwrap_or(0.0);
            let mut patch = Map::new();
            patch.insert(
                "plan_rev".into(),
                Value::Number(serde_json::Number::from_f64(base + 1.0).unwrap()),
            );
            Ok(patch)
        }));
        assert_eq!(jsjson::stringify(updated.get("plan_rev").unwrap()), "2");
        let lane = ok(rebuild_lane_projection(tmp.path(), "f1")).unwrap();
        // execution was stamped for rev 1 → ineffective at rev 2; context is
        // rev-immune (never stamped) and survives.
        assert_eq!(
            jsjson::stringify(lane.get("approved_gates").unwrap()),
            r#"{"context":true,"shape":false,"execution":false,"review":false}"#
        );
    }

    #[test]
    fn default_mutation_routes_through_its_own_live_workflow() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"f1","workers":[]}"#);
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"planning",
                   "plan_rev":0,"summary":"","next_action":"",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        let mut target = ok(resolve_mutation_target(tmp.path(), None, "set", true));
        assert_eq!(target.selected_record(), "default state");
        assert_eq!(target.lane_note(), "");
        target.record_mut().insert("phase".into(), json!("swarming"));
        target.record_mut().insert("summary".into(), json!("S"));
        let record = target.record().clone();
        ok(write_through_projection(tmp.path(), &target, &record, &[]));
        let wf = read_workflow_file(tmp.path(), "wf-1");
        assert_eq!(wf["phase"], json!("swarming"));
        assert_eq!(wf["summary"], json!("S"));
        // state.json is the rebuilt projection of that same record.
        let st = ok(read_state_strict(tmp.path()));
        assert_eq!(st.get("phase"), Some(&json!("swarming")));
        assert_eq!(st.get("summary"), Some(&json!("S")));
    }

    #[test]
    fn a_feature_swap_bypasses_workflow_routing_and_writes_state_directly() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"old"}"#);
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"old","status":"active","phase":"planning",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        let mut target = ok(resolve_mutation_target(tmp.path(), None, "set", true));
        target.record_mut().insert("feature".into(), json!("new"));
        let record = target.record().clone();
        ok(write_through_projection(tmp.path(), &target, &record, &[]));
        // state.json took the swap…
        assert_eq!(ok(read_state_strict(tmp.path())).get("feature"), Some(&json!("new")));
        // …and the OLD feature's workflow record is completely untouched.
        let wf = read_workflow_file(tmp.path(), "wf-1");
        assert_eq!(wf["feature"], json!("old"));
        assert_eq!(wf["phase"], json!("planning"));
    }

    #[test]
    fn mutation_locks_follow_the_global_order_and_the_projection_scope() {
        let tmp = tmp_root();
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        let workflows = ok(list_workflows(tmp.path()));
        let scope = Scope { feature: Some("f1".to_string()), lane: true };
        let locks = ok(acquire_mutation_locks(tmp.path(), &scope, &workflows));
        assert!(lock::lock_file_path(tmp.path(), "workflow:wf-1").exists());
        assert!(lock::lock_file_path(tmp.path(), "lane:f1").exists());
        // A lane mutation must NOT serialize against .bee/state.json's writers.
        assert!(!lock::lock_file_path(tmp.path(), "state").exists());
        drop(locks);
        assert!(!lock::lock_file_path(tmp.path(), "workflow:wf-1").exists());
        // A default-record mutation with a live workflow: workflow:<id> + 'state'.
        let scope = Scope { feature: Some("f1".to_string()), lane: false };
        let locks = ok(acquire_mutation_locks(tmp.path(), &scope, &workflows));
        assert!(lock::lock_file_path(tmp.path(), "workflow:wf-1").exists());
        assert!(lock::lock_file_path(tmp.path(), "state").exists());
        drop(locks);
        // C1 fallback (no live workflow): the single 'state' hold, lane or not.
        let scope = Scope { feature: Some("nolane".to_string()), lane: true };
        let locks = ok(acquire_mutation_locks(tmp.path(), &scope, &workflows));
        assert!(lock::lock_file_path(tmp.path(), "state").exists());
        assert!(!lock::lock_file_path(tmp.path(), "lane:nolane").exists());
        drop(locks);
    }

    #[test]
    fn handoff_workflow_resolution_covers_c1_lane_session_and_default() {
        let tmp = tmp_root();
        // C1: zero workflow records → the legacy single-file path.
        assert!(ok(resolve_handoff_workflow_id(tmp.path(), None, None)).is_none());
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        assert_eq!(
            ok(resolve_handoff_workflow_id(tmp.path(), Some("f1"), None)).as_deref(),
            Some("wf-1")
        );
        match resolve_handoff_workflow_id(tmp.path(), Some("ghost"), None) {
            Err(Err2::Msg(m)) => assert_eq!(
                m,
                "state handoff: refused — --lane \"ghost\" names no live workflow (no .bee/runtime/workflows/*/state.json with feature \"ghost\" and status !== closed). FIX: start it first (\"state start-feature --feature ghost --as-lane\"), or omit --lane."
            ),
            _ => panic!("expected the --lane refusal"),
        }
        // The default record's own feature resolves last.
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"f1"}"#);
        assert_eq!(
            ok(resolve_handoff_workflow_id(tmp.path(), None, None)).as_deref(),
            Some("wf-1")
        );
        // A bound session naming no live workflow refuses loudly (the
        // --session-id FLAG outranks the env chain, so "sess-1" is safe here).
        write_session(tmp.path(), "sess-1", Some("ghost"));
        match resolve_handoff_workflow_id(tmp.path(), None, Some("sess-1")) {
            Err(Err2::Msg(m)) => assert_eq!(
                m,
                "state handoff: refused — calling session \"sess-1\" is bound to lane \"ghost\" but no live workflow names it. FIX: start the lane, unbind the session, or pass --lane explicitly."
            ),
            _ => panic!("expected the bound-session refusal"),
        }
        // A CLOSED workflow is not live.
        write_workflow(
            tmp.path(),
            "wf-2",
            json!({"id":"wf-2","feature":"ghost","status":"closed",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        assert!(resolve_handoff_workflow_id(tmp.path(), Some("ghost"), None).is_err());
    }

    #[test]
    fn a_workflow_carrying_repo_routes_handoffs_to_the_mailbox() {
        let tmp = tmp_root();
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"f1"}"#);
        let wid = ok(resolve_handoff_workflow_id(tmp.path(), None, None)).unwrap();
        let mut input = Map::new();
        input.insert("kind".into(), json!("pause"));
        input.insert("feature".into(), json!("f1"));
        let record = ok(write_mailbox_handoff(tmp.path(), &wid, &input, None));
        assert_eq!(record.get("workflow_id"), Some(&json!("wf-1")));
        assert_eq!(record.get("seq"), Some(&json!(1)));
        // The mailbox is the source of truth…
        assert_eq!(ok(list_handoff_mailbox(tmp.path(), "wf-1")).len(), 1);
        // …and the legacy file is its projection.
        ok(rebuild_handoff_projection(tmp.path()));
        let legacy = ok(read_handoff(tmp.path())).unwrap();
        assert_eq!(jget(&legacy, "kind"), Some(&json!("pause")));
        assert!(jget(&legacy, "workflow_id").is_none(), "mailbox-only field stripped");
    }

    #[test]
    fn workflows_close_guards_the_active_feature() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"active-f"}"#);
        write_workflow(
            tmp.path(),
            "wf-active",
            json!({"id":"wf-active","feature":"active-f","status":"active",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        write_workflow(
            tmp.path(),
            "wf-stale",
            json!({"id":"wf-stale","feature":"stale-f","status":"active",
                   "created_at":"2026-01-02T00:00:00.000Z"}),
        );
        let active = ok(resolve_active_feature_for_workflows_close(tmp.path()));
        assert_eq!(active.unwrap().as_deref(), Some("active-f"));
        // A resolution FAILURE is distinguishable from "idle" (F5).
        let sid = fixture_session_id(tmp.path());
        write_session(tmp.path(), &sid, Some("ghost"));
        let failed = ok(resolve_active_feature_for_workflows_close(tmp.path()));
        assert!(failed.is_err(), "a bound-but-missing lane is a failure, not null");
        assert!(workflows_close_unresolved_active_tail("R").starts_with("Underlying resolution failure: R\n"));
    }

    // ── lanes ─────────────────────────────────────────────────────────────

    #[test]
    fn lane_records_merge_defaults_and_reject_mismatched_features() {
        let tmp = tmp_root();
        let lanes = lanes_dir(tmp.path());
        std::fs::create_dir_all(&lanes).unwrap();
        std::fs::write(
            lanes.join("f1.json"),
            r#"{"feature":"f1","phase":"swarming","approved_gates":{"shape":true}}"#,
        )
        .unwrap();
        std::fs::write(lanes.join("f2.json"), r#"{"feature":"OTHER"}"#).unwrap();
        let rows = list_lanes(tmp.path()).ok().unwrap();
        assert_eq!(rows.len(), 1, "mismatched-feature record is skipped (warned)");
        let lane = &rows[0];
        assert_eq!(lane.get("feature"), Some(&json!("f1")));
        assert_eq!(lane.get("phase"), Some(&json!("swarming")));
        assert_eq!(
            jsjson::stringify(lane.get("approved_gates").unwrap()),
            r#"{"context":false,"shape":true,"execution":false,"review":false}"#
        );
        assert_eq!(lane.get("created_at"), Some(&Value::Null));
    }

    // ── state rebuild-projections (R6) ────────────────────────────────────

    fn seed_workflow(root: &Path, id: &str, feature: &str, status: &str) {
        let dir = workflows_dir(root).join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("state.json"),
            format!(
                r#"{{"id":"{id}","feature":"{feature}","status":"{status}","phase":"exploring","mode":null,"plan_rev":0,"summary":"s","next_action":"n","route":null,"created_at":"2026-01-01T00:00:00.000Z"}}"#
            ),
        )
        .unwrap();
    }

    #[test]
    fn rebuild_all_projections_reports_every_record_in_nodes_key_order() {
        let tmp = tmp_root();
        // C1: zero workflow records — state/handoff non-authoritative, while
        // reservations is ALWAYS authoritative with a concrete count.
        let empty = ok(rebuild_all_projections(tmp.path()));
        let keys: Vec<&str> = match &empty {
            Value::Object(m) => m.keys().map(String::as_str).collect(),
            _ => panic!("object"),
        };
        assert_eq!(keys, vec!["state", "handoff", "reservations", "lanes"]);
        assert_eq!(
            jget(&empty, "state").and_then(|s| jget(s, "authoritative")),
            Some(&json!(false))
        );
        assert_eq!(
            jget(&empty, "handoff").and_then(|h| jget(h, "authoritative")),
            Some(&json!(false))
        );
        assert_eq!(
            jsjson::stringify(jget(&empty, "reservations").unwrap()),
            r#"{"authoritative":true,"count":0}"#
        );
        assert_eq!(jget(&empty, "lanes"), Some(&json!([])));
        assert!(tmp.path().join(".bee").join("reservations.json").exists());

        // One active + one closed record: only the active one is rebuilt.
        seed_workflow(tmp.path(), "wf-1", "alpha", "active");
        seed_workflow(tmp.path(), "wf-2", "gone", "closed");
        let full = ok(rebuild_all_projections(tmp.path()));
        let lanes = match jget(&full, "lanes") {
            Some(Value::Array(a)) => a.clone(),
            _ => panic!("lanes array"),
        };
        assert_eq!(lanes.len(), 1);
        assert_eq!(jget(&lanes[0], "authoritative"), Some(&json!(true)));
        assert_eq!(jget(&lanes[0], "source"), Some(&json!("wf-1")));
        // The idle-bootstrap branch adopts the newest ACTIVE record.
        assert_eq!(
            jget(&full, "state").and_then(|s| jget(s, "source")),
            Some(&json!("wf-1"))
        );
        assert!(lanes_dir(tmp.path()).join("alpha.json").exists());
    }

    #[test]
    fn rebuild_projections_lane_locks_are_sorted_deduped_and_state_first() {
        // The lock ORDER is what keeps two concurrent rebuilds deadlock-free:
        // "state" first, then every active lane name sorted + de-duplicated.
        let mut lane_locks: Vec<String> = ["zeta", "alpha", "zeta", "mid"]
            .iter()
            .map(|f| lane_lock_name(f))
            .collect();
        js_sort(&mut lane_locks);
        lane_locks.dedup();
        assert_eq!(lane_locks, vec!["lane:alpha", "lane:mid", "lane:zeta"]);
    }

    // ── state route (R6) ──────────────────────────────────────────────────

    fn route_flags(pairs: &[(&str, &str)]) -> Flags {
        Flags(pairs.iter().map(|(k, v)| ((*k).to_string(), FlagV::S((*v).to_string()))).collect())
    }

    #[test]
    fn route_set_validation_names_every_bad_value() {
        let f = route_flags(&[
            ("class", "nope"),
            ("lane", "weird"),
            ("flags", "bogus,auth"),
            ("files", "x"),
        ]);
        let message = match ok(validate_route_set_flags(&f)) {
            Err(m) => m,
            Ok(_) => panic!("expected a refusal"),
        };
        assert!(message.starts_with(
            "route --set: invalid flag(s): --class \"nope\" (must be one of feature, bugfix, docs"
        ));
        assert!(message.contains(
            "--lane \"weird\" (must be one of docs, tiny, small, spike, standard, high-risk)"
        ));
        assert!(message.contains("--flags \"bogus,auth\" names invalid flag(s) bogus (legal set: auth,"));
        assert!(message.contains("--files \"x\" (must be a non-negative integer)"));
        assert!(message.ends_with(&format!("Example: {EXAMPLE_ROUTE}")));

        // Missing is a different clause; `--flags ""` is zero flags, not missing.
        let missing = match ok(validate_route_set_flags(&Flags(Vec::new()))) {
            Err(m) => m,
            Ok(_) => panic!("expected a refusal"),
        };
        assert!(missing.contains("missing required flag(s): --class, --lane, --flags, --files"));
        let zero = route_flags(&[("class", "docs"), ("lane", "docs"), ("flags", ""), ("files", "0")]);
        let route = match ok(validate_route_set_flags(&zero)) {
            Ok(r) => r,
            Err(m) => panic!("unexpected refusal: {m}"),
        };
        assert_eq!(route.get("flags"), Some(&json!([])));
        assert_eq!(route.get("rationale"), Some(&Value::Null));
        // Key order is the object literal's.
        let keys: Vec<&str> = route.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            vec!["class", "lane", "flags", "product_files", "rationale", "updated_at"]
        );
    }

    #[test]
    fn route_files_uses_number_not_parseint() {
        // Number("2.5") === 2.5 (NOT parseInt's 2) → not an integer → refused.
        let f = route_flags(&[("class", "docs"), ("lane", "docs"), ("flags", ""), ("files", "2.5")]);
        assert!(matches!(ok(validate_route_set_flags(&f)), Err(m) if m.contains("--files \"2.5\"")));
        assert_eq!(js_number_full("2.5").ok().unwrap(), Some(2.5));
        assert_eq!(js_number_full("  7 ").ok().unwrap(), Some(7.0));
        assert_eq!(js_number_full("   ").ok().unwrap(), Some(0.0)); // Number("  ") === 0
        assert_eq!(js_number_full("x").ok().unwrap(), None); // NaN
        assert_eq!(js_number_full("7px").ok().unwrap(), None);
        assert!(js_number_full("0x10").is_err()); // modeled by Node only
        assert!(js_number_full("Infinity").is_err());
    }

    #[test]
    fn route_worktree_block_follows_is_code_touching_lane() {
        assert!(!is_code_touching_lane("docs", true));
        assert!(!is_code_touching_lane("", true));
        assert!(!is_code_touching_lane("tiny", false));
        assert!(is_code_touching_lane("tiny", true));
        for lane in ["small", "spike", "standard", "high-risk"] {
            assert!(is_code_touching_lane(lane, false));
        }
        let tmp = tmp_root();
        let root = tmp.path(); // no grants registry at all -> the ordinary arm
        assert!(route_worktree_block(root, None, "standard", true).is_none());
        assert!(route_worktree_block(root, Some("f1"), "docs", false).is_none());
        let block = route_worktree_block(root, Some("f1"), "standard", true).unwrap();
        assert_eq!(jget(&block, "required"), Some(&json!(true)));
        assert_eq!(
            jget(&block, "command"),
            Some(&json!("bee worktree new --feature f1"))
        );
        assert!(js_disp_opt(jget(&block, "notice")).starts_with(
            "\u{26a0} WORKTREE-FIRST: lane \"standard\" is code-touching and this is the MAIN checkout."
        ));
    }

    // ── ct-1 (D5): the granted-worktree arm of `route --set` ───────────────
    //
    // The retired Node arm: `code_touching && any_granted_worktree` used to
    // bail the WHOLE verb with `Err2::Ex` before any lock — surfacing as a
    // misleading "unsupported argument shape" refusal — whenever ANY grant
    // was `true`, even one for a completely different feature. Ported here:
    // a foreign grant has zero effect (req. 1); the TARGET feature's own
    // grant redirects the `worktree` block to "continue at the existing
    // worktree" (req. 2); no path bails anymore (req. 3).

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

    /// A real main checkout with two real linked `git worktree add` trees:
    /// `wt-target`, granted and identified as feature "target-feat", and
    /// `wt-foreign`, granted and identified as feature "foreign-feat" — so a
    /// query for "target-feat" hits its own grant while every other feature
    /// (including "foreign-feat" queried on its own, or any third name) must
    /// see the grants registry as if it did not carry a matching entry.
    fn route_worktree_fixture(tmp: &Path) -> PathBuf {
        let main = tmp.join("main");
        std::fs::create_dir_all(&main).unwrap();
        std::fs::write(main.join("f.txt"), "x").unwrap();
        git(&main, &["init", "-q", "-b", "main", "."]);
        git(&main, &["config", "user.email", "a@b.c"]);
        git(&main, &["config", "user.name", "t"]);
        git(&main, &["add", "-A"]);
        git(&main, &["commit", "-qm", "init"]);
        let target = tmp.join("wt-target");
        let foreign = tmp.join("wt-foreign");
        git(&main, &["worktree", "add", "-q", target.to_str().unwrap(), "-b", "wt/target-feat"]);
        git(&main, &["worktree", "add", "-q", foreign.to_str().unwrap(), "-b", "wt/foreign-feat"]);
        std::fs::create_dir_all(main.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            main.join(".bee").join("runtime").join("worktree-grants.json"),
            "{\"wt-target\": true, \"wt-foreign\": true}\n",
        )
        .unwrap();
        std::fs::create_dir_all(target.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            target.join(".bee").join("runtime").join("worktree-identity.json"),
            "{\"feature\":\"target-feat\"}",
        )
        .unwrap();
        std::fs::create_dir_all(foreign.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            foreign.join(".bee").join("runtime").join("worktree-identity.json"),
            "{\"feature\":\"foreign-feat\"}",
        )
        .unwrap();
        main
    }

    /// Requirement 1: a grant for a DIFFERENT feature must not affect the
    /// call at all — proceed exactly as the no-grant path (the "create a
    /// worktree" notice), never a decline, and never mention the foreign id.
    #[test]
    fn route_worktree_block_ignores_a_foreign_features_grant() {
        let tmp = tmp_root();
        let main = route_worktree_fixture(tmp.path());
        // "other-feat" holds no grant of its own; "wt-foreign" is granted for
        // an unrelated feature and must be invisible to this call.
        let block = route_worktree_block(&main, Some("other-feat"), "standard", true)
            .expect("still a code-touching feature -> still a block");
        assert_eq!(
            jget(&block, "command"),
            Some(&json!("bee worktree new --feature other-feat"))
        );
        let notice = js_disp_opt(jget(&block, "notice"));
        assert!(notice.starts_with(
            "\u{26a0} WORKTREE-FIRST: lane \"standard\" is code-touching and this is the MAIN checkout."
        ));
        assert!(!notice.contains("wt-foreign"), "notice: {notice}");
    }

    /// Requirement 2: a grant for the TARGET feature routes the `worktree`
    /// block to the EXISTING granted worktree — "continue there" — instead
    /// of telling the caller to create a new one.
    #[test]
    fn route_worktree_block_names_the_existing_worktree_for_a_granted_feature() {
        let tmp = tmp_root();
        let main = route_worktree_fixture(tmp.path());
        let block = route_worktree_block(&main, Some("target-feat"), "standard", true)
            .expect("a granted feature is still code-touching -> still a block");
        assert_ne!(
            jget(&block, "command"),
            Some(&json!("bee worktree new --feature target-feat")),
            "must not tell the caller to create a worktree that already exists"
        );
        let notice = js_disp_opt(jget(&block, "notice"));
        assert!(notice.contains("wt-target"), "notice: {notice}");
        assert!(
            notice.to_lowercase().contains("already"),
            "notice must say the worktree already exists: {notice}"
        );
    }

    /// Requirement 3: no path through this arm ever declines (`Err2::Ex`)
    /// anymore. `route_worktree_block` is exactly what `run_route` calls to
    /// build the response's `worktree` key, so "always answers `Some` for a
    /// code-touching lane with a feature" — no matter which grants exist —
    /// pins the fix at the boundary `run_route` actually consumes. Before
    /// ct-1 this same shape (ANY grant `true`) made the WHOLE verb decline
    /// with a misleading "unsupported argument shape" refusal.
    #[test]
    fn route_worktree_block_never_declines_under_any_grant_shape() {
        let tmp = tmp_root();
        let main = route_worktree_fixture(tmp.path());
        let bare = tmp_root(); // zero grants at all
        for (root, feature) in [
            (bare.path(), "no-grants-anywhere"),
            (main.as_path(), "other-feat"),   // grants exist, none match
            (main.as_path(), "foreign-feat"), // matches wt-foreign's OWN grant
            (main.as_path(), "target-feat"),  // matches wt-target's OWN grant
        ] {
            assert!(
                route_worktree_block(root, Some(feature), "standard", true).is_some(),
                "feature {feature} must still get a worktree block, never a decline"
            );
        }
    }

    #[test]
    fn other_live_work_present_ignores_idle_and_stale_peers() {
        let tmp = tmp_root();
        let sessions = tmp.path().join(".bee").join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let stamp = now_iso();
        // The record standing in for "me" must carry whatever session id
        // resolution will ACTUALLY return here, not the literal "self".
        //
        // `resolve_session_id_no_flag` reads BEE_SESSION_ID, then
        // CLAUDE_CODE_SESSION_ID, and only falls back to single-live-session
        // adoption when neither is set. bee's own primary runtime always
        // exports CLAUDE_CODE_SESSION_ID — so under a hard-coded "self" this
        // test silently became a THREE-session store (me, self, peer) with
        // nothing excluded, and the final "a stale peer never counts"
        // assertion read the still-live `self` as other live work. It failed
        // for everyone running the suite from inside Claude Code, which is
        // most of the time, and passed only in a bare shell.
        let me = resolve_session_id_no_flag(tmp.path())
            .ok()
            .flatten()
            .unwrap_or_else(|| "self".to_string());
        std::fs::write(
            sessions.join("self.json"),
            format!(
                r#"{{"id":"{me}","started_at":"{stamp}","last_heartbeat":"{stamp}","lane":null}}"#
            ),
        )
        .unwrap();
        // A live peer on the DEFAULT record, which is idle → no live work.
        std::fs::write(
            sessions.join("peer.json"),
            format!(
                r#"{{"id":"peer","started_at":"{stamp}","last_heartbeat":"{stamp}","lane":null}}"#
            ),
        )
        .unwrap();
        assert!(!ok(other_live_work_present(tmp.path())));
        // Move the default record off idle → the peer now counts.
        write_state_file(tmp.path(), r#"{"phase":"swarming","feature":"f1"}"#);
        assert!(ok(other_live_work_present(tmp.path())));
        // A STALE peer never counts.
        std::fs::write(
            sessions.join("peer.json"),
            r#"{"id":"peer","started_at":"2020-01-01T00:00:00.000Z","last_heartbeat":"2020-01-01T00:00:00.000Z","lane":null}"#,
        )
        .unwrap();
        assert!(!ok(other_live_work_present(tmp.path())));
    }

    // ── state route --set re-lane transition validation (D5, hook-teeth bh-5) ──
    //
    // scout-and-ticks.md, "Re-lane checkpoint": downward moves only travel
    // the standard->small->tiny ladder, one step or more but always
    // downward; at most one demotion per feature EVER (a `demoted_at`
    // stamp on the route object persists it); `high-risk` never demotes;
    // any hard-gate flag in the NEW --flags set blocks a demotion. Any
    // upward move or same-lane re-record is always allowed.

    fn route_with(lane: &str) -> Map<String, Value> {
        obj(&format!(r#"{{"class":"feature","lane":"{lane}","flags":[],"product_files":1}}"#))
    }

    fn route_with_demoted_at(lane: &str, stamp: &str) -> Map<String, Value> {
        let mut r = route_with(lane);
        r.insert("demoted_at".into(), json!(stamp));
        r
    }

    #[test]
    fn triage_ladder_rank_orders_only_the_three_demotable_lanes() {
        assert!(triage_ladder_rank("tiny") < triage_ladder_rank("small"));
        assert!(triage_ladder_rank("small") < triage_ladder_rank("standard"));
        // docs/spike/high-risk are off this numeric ladder entirely — the
        // high-risk case is an absolute rule of its own, and docs/spike
        // moves are never classified as a ladder demotion by this rule.
        for lane in ["docs", "spike", "high-risk", ""] {
            assert_eq!(triage_ladder_rank(lane), None, "lane {lane}");
        }
    }

    #[test]
    fn a_first_demotion_stamps_demoted_at_and_a_second_ever_refuses() {
        // standard -> small: within-threshold, zero hard-gate flags, no
        // prior demotion -> allowed, stamps a fresh demoted_at.
        let stamp = match validate_route_lane_transition(&route_with("standard"), "small", &[]) {
            Ok(Some(s)) => s,
            other => panic!("expected a fresh demoted_at stamp, got {other:?}"),
        };
        assert!(!stamp.is_empty());

        // A second demotion attempt on a route that already carries that
        // stamp -> refused, naming the once-per-feature rule, even though
        // this is a DIFFERENT step (small -> tiny) on the SAME feature.
        let existing = route_with_demoted_at("small", &stamp);
        let message = match validate_route_lane_transition(&existing, "tiny", &[]) {
            Err(m) => m,
            Ok(v) => panic!("expected a refusal, got Ok({v:?})"),
        };
        assert!(
            message.contains("at most one demotion per feature, ever"),
            "message: {message}"
        );
        assert!(message.contains(&stamp), "message must cite the first stamp: {message}");
    }

    #[test]
    fn multi_step_downward_demotion_is_allowed_as_a_single_demotion() {
        // "one step or more but always downward" — standard -> tiny
        // directly is still just ONE demotion, not two.
        let existing = route_with("standard");
        match validate_route_lane_transition(&existing, "tiny", &[]) {
            Ok(Some(_)) => {}
            other => panic!("expected a stamped single demotion, got {other:?}"),
        }
    }

    #[test]
    fn hard_gate_flag_in_the_new_flags_blocks_demotion() {
        let existing = route_with("standard");
        let flags = vec!["auth".to_string()];
        let message = match validate_route_lane_transition(&existing, "small", &flags) {
            Err(m) => m,
            Ok(v) => panic!("expected a refusal, got Ok({v:?})"),
        };
        assert!(
            message.contains("a hard-gate flag can never demote"),
            "message: {message}"
        );
        assert!(message.contains("auth"), "message: {message}");

        // Reuses validate_route_set_flags's own vocabulary — every entry
        // in HARD_GATE_ROUTE_FLAGS must be a legal --flags value.
        for f in HARD_GATE_ROUTE_FLAGS {
            assert!(ROUTE_FLAG_VALUES.contains(&f), "hard-gate flag {f} not in ROUTE_FLAG_VALUES");
        }

        // A demotion with a flag OUTSIDE the hard-gate set is untouched.
        let benign = vec!["multi-domain".to_string()];
        match validate_route_lane_transition(&existing, "small", &benign) {
            Ok(Some(_)) => {}
            other => panic!("expected an allowed demotion, got {other:?}"),
        }
    }

    #[test]
    fn high_risk_never_demotes_regardless_of_target_lane_or_flags() {
        let existing = route_with("high-risk");
        for target in ["standard", "small", "tiny", "docs", "spike"] {
            let message = match validate_route_lane_transition(&existing, target, &[]) {
                Err(m) => m,
                Ok(v) => panic!("expected a refusal for high-risk -> {target}, got Ok({v:?})"),
            };
            assert!(
                message.contains("high-risk lanes never demote"),
                "message for {target}: {message}"
            );
        }
    }

    #[test]
    fn upward_moves_and_off_ladder_moves_are_always_allowed() {
        // Upward, even across multiple rungs.
        assert_eq!(
            ok2(validate_route_lane_transition(&route_with("tiny"), "standard", &[])),
            None
        );
        // Promoting INTO high-risk from anywhere is always allowed.
        assert_eq!(
            ok2(validate_route_lane_transition(&route_with("small"), "high-risk", &[])),
            None
        );
        // A move touching docs/spike is off the standard/small/tiny
        // ladder entirely and is never classified as a ladder demotion.
        assert_eq!(
            ok2(validate_route_lane_transition(&route_with("standard"), "docs", &[])),
            None
        );
        assert_eq!(
            ok2(validate_route_lane_transition(&route_with("docs"), "standard", &[])),
            None
        );
    }

    #[test]
    fn same_lane_re_record_is_allowed_and_carries_demotion_history_forward() {
        // No prior demotion -> stays None.
        assert_eq!(
            ok2(validate_route_lane_transition(&route_with("standard"), "standard", &[])),
            None
        );
        // A prior demotion's stamp survives an unrelated same-lane
        // re-record UNCHANGED (never re-stamped, never dropped).
        let existing = route_with_demoted_at("small", "2020-01-01T00:00:00.000Z");
        assert_eq!(
            ok2(validate_route_lane_transition(&existing, "small", &[])),
            Some("2020-01-01T00:00:00.000Z".to_string())
        );
    }

    #[test]
    fn a_promotion_after_a_demotion_still_carries_the_demotion_history_forward() {
        // The once-per-feature limit is "ever": a later promotion must not
        // erase the fact that this feature already spent its one
        // demotion, so a future re-demotion attempt still refuses.
        let existing = route_with_demoted_at("small", "2020-01-01T00:00:00.000Z");
        assert_eq!(
            ok2(validate_route_lane_transition(&existing, "standard", &[])),
            Some("2020-01-01T00:00:00.000Z".to_string())
        );
    }

    /// `Result<Option<String>, String>` unwrapped for the assert_eq! call
    /// sites above — panics with the refusal text on an unexpected Err,
    /// which is exactly what a stray refusal in an "always allowed" case
    /// should do.
    fn ok2(r: Result<Option<String>, String>) -> Option<String> {
        match r {
            Ok(v) => v,
            Err(m) => panic!("unexpected refusal: {m}"),
        }
    }
