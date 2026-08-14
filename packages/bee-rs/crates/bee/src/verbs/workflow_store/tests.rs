// Split out of the single 2.7k-line verbs/workflow_store.rs. Code unchanged; only module placement and item visibility moved.
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
use crate::lock::{self, LockGuard, MAX_ATTEMPTS};
use crate::verbs::reservations::{
    date_parse_val, jget, js_disp, js_disp_opt, js_trim, now_iso, pseudo_uuid_v4,
    truthy, Err2, Ex, Exotic,
};
use crate::verbs::state_group::{
    adopt_claim, coerce_legacy_phase, default_gates, handoff_path, io_read_reason, parse_json_v8,
    read_claim, read_state_peek, spread_gates, write_state, AdoptOutcome, ParsedJson,
};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
    use super::*;

    fn tmp_root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn ok<T, E>(r: Result<T, E>) -> T {
        match r {
            Ok(v) => v,
            Err(_) => panic!("unexpected error result"),
        }
    }

    fn write_workflow(root: &Path, id: &str, body: Value) {
        let dir = workflows_dir(root).join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("state.json"), serde_json::to_string(&body).unwrap()).unwrap();
    }

    fn write_state_file(root: &Path, content: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("state.json"), content).unwrap();
    }

    fn write_lane_file(root: &Path, feature: &str, content: &str) {
        std::fs::create_dir_all(lanes_dir(root)).unwrap();
        std::fs::write(lanes_dir(root).join(format!("{feature}.json")), content).unwrap();
    }

    fn read_back(file: &Path) -> Value {
        serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap()
    }

    // ── workflow-store.mjs ────────────────────────────────────────────────

    #[test]
    fn merge_gates_defaults_overlays_and_keeps_unknown_names() {
        let merged = ok(merge_gates(
            None,
            Some(&json!({"execution": {"approved": true, "approved_for_plan_rev": 2},
                         "fifth": {"approved": true}})),
        ));
        assert_eq!(
            jsjson::stringify(&merged),
            r#"{"context":{"approved":false,"approved_for_plan_rev":null,"state":"pending","actor":null,"at":null,"reason":null,"bypass_level":null},"shape":{"approved":false,"approved_for_plan_rev":null,"state":"pending","actor":null,"at":null,"reason":null,"bypass_level":null},"execution":{"approved":true,"approved_for_plan_rev":2,"state":"approved","actor":null,"at":null,"reason":null,"bypass_level":null},"review":{"approved":false,"approved_for_plan_rev":null,"state":"pending","actor":null,"at":null,"reason":null,"bypass_level":null},"fifth":{"approved":true,"approved_for_plan_rev":null,"state":"approved","actor":null,"at":null,"reason":null,"bypass_level":null}}"#
        );
        // A patch carrying only `approved` PRESERVES the base's rev stamp —
        // and, per D3, re-derives `state` from the fresh boolean rather than
        // leaving the base's `state` stale.
        let base = ok(merge_gates(None, Some(&json!({"execution": {"approved": true, "approved_for_plan_rev": 7}}))));
        let next = ok(merge_gates(Some(&base), Some(&json!({"execution": {"approved": true}}))));
        assert_eq!(jget(&next, "execution").unwrap()["approved_for_plan_rev"], json!(7));
        assert_eq!(jget(&next, "execution").unwrap()["state"], json!("approved"));
    }

    #[test]
    fn default_gate_entry_carries_all_five_new_fields() {
        let entry = default_gate_entry();
        assert_eq!(entry["approved"], json!(false));
        assert_eq!(entry["state"], json!("pending"));
        assert_eq!(entry["actor"], Value::Null);
        assert_eq!(entry["at"], Value::Null);
        assert_eq!(entry["reason"], Value::Null);
        assert_eq!(entry["bypass_level"], Value::Null);
    }

    #[test]
    fn merge_gates_refuses_unknown_state_and_actor_values() {
        match merge_gates(None, Some(&json!({"execution": {"approved": true, "state": "waiting"}}))) {
            Err(Err2::Msg(m)) => assert!(m.contains("state must be one of pending/approved/rejected"), "{m}"),
            other => panic!("expected a typed refusal, got {other:?}"),
        }
        match merge_gates(None, Some(&json!({"execution": {"approved": true, "actor": "robot"}}))) {
            Err(Err2::Msg(m)) => assert!(m.contains("actor must be one of user/auto"), "{m}"),
            other => panic!("expected a typed refusal, got {other:?}"),
        }
        // A merge that never touches state/actor at all writes nothing bad —
        // no refusal, no new record on disk (this is a pure function, so
        // "writes nothing" is simply "returns Err before merged is used").
        assert!(ok(merge_gates(None, None)).is_object());
    }

    #[test]
    fn old_shape_gate_entry_without_state_derives_it_from_approved() {
        let tmp = tmp_root();
        write_workflow(
            tmp.path(),
            "wf-legacy",
            json!({"id":"wf-legacy","feature":"f1",
                   "gates":{"execution":{"approved":true,"approved_for_plan_rev":3},
                            "shape":{"approved":false,"approved_for_plan_rev":null}}}),
        );
        let rec = read_workflow_record(tmp.path(), "wf-legacy").ok().unwrap();
        let gates = rec.get("gates").unwrap();
        assert_eq!(jget(gates, "execution").unwrap()["state"], json!("approved"));
        assert_eq!(jget(gates, "execution").unwrap()["approved"], json!(true));
        assert_eq!(jget(gates, "shape").unwrap()["state"], json!("pending"));
        // A gate name absent from the on-disk record entirely still defaults
        // fully, including the new `state` field.
        assert_eq!(jget(gates, "context").unwrap()["state"], json!("pending"));
        assert_eq!(jget(gates, "context").unwrap()["approved"], json!(false));
    }

    #[test]
    fn plan_rev_bump_leaves_the_record_entry_approved_while_projection_goes_stale() {
        // D3's intended divergence (plan.md "One divergence is intended"):
        // the RECORD entry an approval writes stays `approved: true` /
        // `state: "approved"` — only the PROJECTED boolean
        // (`workflow_gates_to_approved_gates`) reads plan-rev staleness.
        // This is intended plan-rev invalidation, asserted here as intended,
        // never repaired.
        let gates = ok(merge_gates(
            None,
            Some(&json!({"execution": {"approved": true, "approved_for_plan_rev": 3}})),
        ));
        let entry = jget(&gates, "execution").unwrap();
        assert_eq!(entry["approved"], json!(true));
        assert_eq!(entry["state"], json!("approved"));
        // plan_rev bumped past the stamp (3 → 4): the RECORD entry above is
        // untouched by a bump (nothing here re-derives it), but the
        // PROJECTION reads false for the same gate at the bumped rev.
        assert_eq!(
            jget(&workflow_gates_to_approved_gates(Some(&gates), Some(&json!(4))), "execution"),
            Some(&json!(false))
        );
        // …while the record entry itself is still approved:true/state:approved.
        assert_eq!(entry["approved"], json!(true));
        assert_eq!(entry["state"], json!("approved"));
    }

    #[test]
    fn legacy_gates_to_workflow_gates_emits_the_new_fields_consistently() {
        use crate::verbs::state_group::legacy_gates_to_workflow_gates;
        let approved = json!({"execution": true, "shape": false});
        let gates = legacy_gates_to_workflow_gates(Some(&approved));
        assert_eq!(jget(&gates, "execution").unwrap()["approved"], json!(true));
        assert_eq!(jget(&gates, "execution").unwrap()["state"], json!("approved"));
        assert_eq!(jget(&gates, "shape").unwrap()["approved"], json!(false));
        assert_eq!(jget(&gates, "shape").unwrap()["state"], json!("pending"));
        // A gate absent from the legacy boolean map defaults to pending too.
        assert_eq!(jget(&gates, "context").unwrap()["state"], json!("pending"));
        // Feeds straight into merge_gates/create_workflow without desyncing.
        let merged = ok(merge_gates(None, Some(&gates)));
        assert_eq!(jget(&merged, "execution").unwrap()["state"], json!("approved"));
        assert_eq!(jget(&merged, "execution").unwrap()["approved"], json!(true));
    }

    #[test]
    fn read_workflow_record_merges_defaults_and_refuses_id_mismatch() {
        let tmp = tmp_root();
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        let rec = read_workflow_record(tmp.path(), "wf-1").ok().unwrap();
        assert_eq!(rec.get("phase"), Some(&json!("idle")));
        assert_eq!(rec.get("plan_rev"), Some(&json!(0)));
        assert_eq!(rec.get("status"), Some(&json!("active")));
        assert_eq!(rec.get("route"), Some(&Value::Null));
        assert!(rec.contains_key("gates"));
        write_workflow(tmp.path(), "wf-2", json!({"id":"other","feature":"f2"}));
        match read_workflow_record(tmp.path(), "wf-2") {
            Err(WfSkip(m)) => assert!(m.contains("does not match the requested workflow \"wf-2\"")),
            _ => panic!("expected the id-mismatch skip"),
        }
        // Missing record: WORKFLOW_MISSING reason.
        std::fs::create_dir_all(workflows_dir(tmp.path()).join("wf-3")).unwrap();
        match read_workflow_record(tmp.path(), "wf-3") {
            Err(WfSkip(m)) => assert!(m.starts_with("readWorkflow: no workflow record at")),
            _ => panic!("expected WORKFLOW_MISSING"),
        }
    }

    // ── trun-7: run_state, the persisted run-lifecycle vocabulary ──────────

    #[test]
    fn run_state_closed_vocabulary_accepts_null_and_refuses_unknown_values() {
        for v in RUN_STATE_VALUES {
            assert!(valid_run_state_value(&json!(v)), "{v} should be valid");
        }
        assert!(valid_run_state_value(&Value::Null), "null is the pre-migration shape");
        assert!(!valid_run_state_value(&json!("waiting-around")));
        assert!(!valid_run_state_value(&json!(true)));

        let mut patch = Map::new();
        patch.insert("run_state".into(), json!("waiting-around"));
        match check_patch_run_state(&patch) {
            Err(Err2::Msg(m)) => assert_eq!(
                m,
                "updateWorkflowAssumingLock: run_state must be one of shaping/awaiting-approval/running/blocked/done (got \"waiting-around\")."
            ),
            other => panic!("expected a typed refusal, got {other:?}"),
        }
    }

    #[test]
    fn read_workflow_record_refuses_a_corrupt_run_state_on_disk() {
        let tmp = tmp_root();
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","run_state":"vibing"}),
        );
        match read_workflow_record(tmp.path(), "wf-1") {
            Err(WfSkip(m)) => assert!(m.contains("its run_state is corrupt"), "{m}"),
            Ok(_) => panic!("expected the run_state corruption refusal"),
        }
        // A legacy record with no run_state at all reads back fine, backfilled null.
        write_workflow(tmp.path(), "wf-2", json!({"id":"wf-2","feature":"f2"}));
        let rec = read_workflow_record(tmp.path(), "wf-2").ok().unwrap();
        assert_eq!(rec.get("run_state"), Some(&Value::Null));
    }

    /// The one rule this feature exists to make visible: any gate `pending`
    /// with nothing LATER in the fixed GATE_NAMES sequence `approved` reads
    /// `awaiting-approval` — unconditionally, holding even when an earlier
    /// gate is `rejected`.
    #[test]
    fn derive_run_state_reads_awaiting_approval_whenever_a_gate_is_pending_and_none_later_is_approved() {
        let counts = CellCounts::default();

        // Fresh gates: every gate pending, nothing approved anywhere.
        let all_pending = json!(default_wf_gates());
        assert_eq!(derive_run_state("active", &all_pending, &counts), "awaiting-approval");

        // context approved, shape pending, execution/review untouched —
        // shape is the earliest pending gate and nothing later is approved.
        let mid_pending = json!({
            "context": {"state": "approved"},
            "shape": {"state": "pending"},
        });
        assert_eq!(derive_run_state("active", &mid_pending, &counts), "awaiting-approval");

        // context rejected, shape (later) still pending, execution/review
        // untouched (also pending): an earlier rejection never blocks the
        // unconditional pending-with-nothing-later-approved rule.
        let rejected_then_pending = json!({
            "context": {"state": "rejected"},
            "shape": {"state": "pending"},
        });
        assert_eq!(
            derive_run_state("active", &rejected_then_pending, &counts),
            "awaiting-approval",
            "an earlier rejected gate never overrides the unconditional pending rule"
        );
    }

    #[test]
    fn derive_run_state_reads_blocked_for_a_trailing_unresolved_rejection() {
        let counts = CellCounts::default();
        // Every gate decided; the last decision in the sequence is a
        // rejection nothing later has overturned — the existing
        // approved→rejected revocation path, named on the run.
        let gates = json!({
            "context": {"state": "approved"},
            "shape": {"state": "approved"},
            "execution": {"state": "rejected"},
            "review": {"state": "pending"},
        });
        // `review` is still pending, and nothing later than it is approved,
        // so the unconditional awaiting-approval rule fires first.
        assert_eq!(derive_run_state("active", &gates, &counts), "awaiting-approval");

        // Once every earlier gate is settled and the LAST gate in the
        // sequence is the rejection, nothing later can ever supersede it.
        let trailing_rejection = json!({
            "context": {"state": "approved"},
            "shape": {"state": "approved"},
            "execution": {"state": "approved"},
            "review": {"state": "rejected"},
        });
        assert_eq!(derive_run_state("active", &trailing_rejection, &counts), "blocked");
    }

    #[test]
    fn derive_run_state_reads_cell_counts_once_every_gate_clears() {
        let all_approved = json!({
            "context": {"state": "approved"},
            "shape": {"state": "approved"},
            "execution": {"state": "approved"},
            "review": {"state": "approved"},
        });
        let mut counts = CellCounts::default();
        assert_eq!(derive_run_state("active", &all_approved, &counts), "shaping", "no cells yet");

        counts.open = 1;
        assert_eq!(derive_run_state("active", &all_approved, &counts), "running");

        counts = CellCounts::default();
        counts.blocked = 1;
        assert_eq!(derive_run_state("active", &all_approved, &counts), "blocked");

        counts = CellCounts::default();
        counts.capped = 2;
        counts.dropped = 1;
        assert_eq!(derive_run_state("active", &all_approved, &counts), "done");

        // A closed workflow always reads done, regardless of gates or cells.
        assert_eq!(derive_run_state("closed", &all_approved, &counts), "done");
    }

    #[test]
    fn create_workflow_starts_awaiting_approval_with_every_gate_pending() {
        let tmp = tmp_root();
        let record = ok(create_workflow(tmp.path(), NewWorkflow::for_feature("f1")));
        assert_eq!(record.get("run_state"), Some(&json!("awaiting-approval")));
        // …and it landed on disk, readable back through the strict reader.
        let on_disk = ok(read_workflow_record(tmp.path(), record.get("id").unwrap().as_str().unwrap()));
        assert_eq!(on_disk.get("run_state"), Some(&json!("awaiting-approval")));
    }

    #[test]
    fn update_assuming_lock_recomputes_run_state_and_ignores_a_patched_value() {
        let tmp = tmp_root();
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        // Approve every gate directly through the patch — run_state should
        // move off awaiting-approval even though nothing named it.
        let mut patch = Map::new();
        patch.insert(
            "gates".into(),
            json!({
                "context": {"approved": true, "state": "approved"},
                "shape": {"approved": true, "state": "approved"},
                "execution": {"approved": true, "state": "approved"},
                "review": {"approved": true, "state": "approved"},
            }),
        );
        // A caller-supplied run_state is a valid vocabulary value, but the
        // write recomputes it anyway — it is never taken from the patch.
        patch.insert("run_state".into(), json!("done"));
        let next = ok(update_workflow_assuming_lock(tmp.path(), "wf-1", patch));
        assert_eq!(
            next.get("run_state"),
            Some(&json!("shaping")),
            "recomputed from status=active, all gates approved, zero cells — not the patched \"done\""
        );
    }

    // ── awaiting-human: the waiting mark (D1/D3) ────────────────────────────

    #[test]
    fn build_waiting_on_refuses_unknown_kind_empty_subject_and_empty_session() {
        match build_waiting_on("vibe", "why?", "sess-1") {
            Err(Err2::Msg(m)) => assert!(m.contains("kind must be one of gate/question"), "{m}"),
            other => panic!("expected a typed refusal, got {other:?}"),
        }
        match build_waiting_on("question", "   ", "sess-1") {
            Err(Err2::Msg(m)) => assert!(m.contains("subject is required"), "{m}"),
            other => panic!("expected a typed refusal, got {other:?}"),
        }
        match build_waiting_on("question", "why?", "") {
            Err(Err2::Msg(m)) => assert!(m.contains("session is required"), "{m}"),
            other => panic!("expected a typed refusal, got {other:?}"),
        }
        let mark = ok(build_waiting_on("question", "why is X true?", "sess-1"));
        assert_eq!(mark["kind"], json!("question"));
        assert_eq!(mark["subject"], json!("why is X true?"));
        assert_eq!(mark["session"], json!("sess-1"));
        assert!(mark.get("asked_at").is_some());
        assert!(waiting_on_is_live(Some(&mark)));
    }

    #[test]
    fn waiting_on_is_live_only_for_a_well_shaped_mark() {
        assert!(!waiting_on_is_live(None));
        assert!(!waiting_on_is_live(Some(&Value::Null)));
        assert!(!waiting_on_is_live(Some(&json!({"kind": "gate"})))); // no subject
        assert!(!waiting_on_is_live(Some(&json!({"kind": "vibe", "subject": "x"})))); // bad kind
        assert!(waiting_on_is_live(Some(&json!({"kind": "gate", "subject": "execution"}))));
    }

    /// Setting a mark makes run_state read awaiting-approval, WITH NO GATE
    /// PENDING — the mark alone is sufficient.
    #[test]
    fn set_workflow_waiting_on_makes_run_state_read_awaiting_approval_with_no_gate_pending() {
        let tmp = tmp_root();
        let record = ok(create_workflow(tmp.path(), NewWorkflow::for_feature("f1")));
        let id = record.get("id").unwrap().as_str().unwrap().to_string();
        // Approve every gate first: the baseline (no mark) no longer reads
        // awaiting-approval on its own.
        let mut approve = Map::new();
        approve.insert(
            "gates".into(),
            json!({
                "context": {"approved": true, "state": "approved"},
                "shape": {"approved": true, "state": "approved"},
                "execution": {"approved": true, "state": "approved"},
                "review": {"approved": true, "state": "approved"},
            }),
        );
        let approved = ok(update_workflow(tmp.path(), &id, approve));
        assert_eq!(approved.get("run_state"), Some(&json!("shaping")), "baseline, no wait");

        let marked = ok(set_workflow_waiting_on(tmp.path(), &id, "question", "why is X true?", "sess-1"));
        assert_eq!(marked.get("run_state"), Some(&json!("awaiting-approval")));
        assert_eq!(
            marked.get("waiting_on").and_then(|v| v.get("subject")),
            Some(&json!("why is X true?"))
        );
        assert_eq!(marked.get("waiting_on").and_then(|v| v.get("session")), Some(&json!("sess-1")));

        // Reads back the same off disk.
        let on_disk = ok(read_workflow_record(tmp.path(), &id));
        assert_eq!(on_disk.get("run_state"), Some(&json!("awaiting-approval")));
        assert!(waiting_on_is_live(on_disk.get("waiting_on")));
    }

    /// A pending gate AND a live mark together still read exactly ONE
    /// awaiting-approval, never a conflict — a fresh workflow's gates are
    /// already all pending (its own awaiting-approval condition), so marking
    /// it on top proves the two sources agree rather than disagreeing.
    #[test]
    fn a_pending_gate_and_a_live_waiting_on_mark_together_read_exactly_one_awaiting_approval() {
        let tmp = tmp_root();
        let record = ok(create_workflow(tmp.path(), NewWorkflow::for_feature("f1")));
        assert_eq!(record.get("run_state"), Some(&json!("awaiting-approval")), "gate condition alone");
        let id = record.get("id").unwrap().as_str().unwrap().to_string();
        let marked = ok(set_workflow_waiting_on(tmp.path(), &id, "gate", "shape", "sess-1"));
        assert_eq!(marked.get("run_state"), Some(&json!("awaiting-approval")), "both sources, one value");
    }

    #[test]
    fn set_workflow_waiting_on_refuses_an_unknown_kind_and_writes_nothing() {
        let tmp = tmp_root();
        let record = ok(create_workflow(tmp.path(), NewWorkflow::for_feature("f1")));
        let id = record.get("id").unwrap().as_str().unwrap().to_string();
        match set_workflow_waiting_on(tmp.path(), &id, "vibe", "why?", "sess-1") {
            Err(Err2::Msg(m)) => assert!(m.contains("kind must be one of gate/question"), "{m}"),
            other => panic!("expected a typed refusal, got {other:?}"),
        }
        match set_workflow_waiting_on(tmp.path(), &id, "question", "", "sess-1") {
            Err(Err2::Msg(m)) => assert!(m.contains("subject is required"), "{m}"),
            other => panic!("expected a typed refusal, got {other:?}"),
        }
        // Nothing was written — the record still has no live mark.
        let on_disk = ok(read_workflow_record(tmp.path(), &id));
        assert_eq!(on_disk.get("waiting_on"), Some(&Value::Null));
    }

    #[test]
    fn check_patch_waiting_on_refuses_a_malformed_patch_and_writes_nothing() {
        let tmp = tmp_root();
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        let mut patch = Map::new();
        patch.insert("waiting_on".into(), json!({"kind": "gate"})); // no subject
        match update_workflow_assuming_lock(tmp.path(), "wf-1", patch) {
            Err(Err2::Msg(m)) => assert!(m.contains("waiting_on must be null or an object"), "{m}"),
            other => panic!("expected a typed refusal, got {other:?}"),
        }
        let on_disk = ok(read_workflow_record(tmp.path(), "wf-1"));
        assert_eq!(on_disk.get("waiting_on"), Some(&Value::Null), "the refused patch wrote nothing");
    }

    // ── awaiting-human: clearing (D2, ah-2) ─────────────────────────────────

    #[test]
    fn clear_workflow_waiting_on_clears_a_live_mark_and_recomputes_run_state() {
        let tmp = tmp_root();
        let record = ok(create_workflow(tmp.path(), NewWorkflow::for_feature("f1")));
        let id = record.get("id").unwrap().as_str().unwrap().to_string();
        // Approve every gate so, once the mark is gone, run_state falls all
        // the way through to a value the gate condition alone can't fake.
        let mut approve = Map::new();
        approve.insert(
            "gates".into(),
            json!({
                "context": {"approved": true, "state": "approved"},
                "shape": {"approved": true, "state": "approved"},
                "execution": {"approved": true, "state": "approved"},
                "review": {"approved": true, "state": "approved"},
            }),
        );
        ok(update_workflow(tmp.path(), &id, approve));
        ok(set_workflow_waiting_on(tmp.path(), &id, "question", "why is X true?", "sess-1"));
        let marked = ok(read_workflow_record(tmp.path(), &id));
        assert_eq!(marked.get("run_state"), Some(&json!("awaiting-approval")));

        let cleared = ok(clear_workflow_waiting_on(tmp.path(), &id));
        assert_eq!(cleared.get("waiting_on"), Some(&Value::Null));
        assert_eq!(cleared.get("run_state"), Some(&json!("shaping")), "recomputed, mark gone");
        let on_disk = ok(read_workflow_record(tmp.path(), &id));
        assert_eq!(on_disk.get("waiting_on"), Some(&Value::Null));
    }

    #[test]
    fn clear_workflow_waiting_on_is_a_no_op_when_nothing_is_live() {
        let tmp = tmp_root();
        let record = ok(create_workflow(tmp.path(), NewWorkflow::for_feature("f1")));
        let id = record.get("id").unwrap().as_str().unwrap().to_string();
        // No mark was ever set — clearing must not refuse.
        let cleared = ok(clear_workflow_waiting_on(tmp.path(), &id));
        assert_eq!(cleared.get("waiting_on"), Some(&Value::Null));
    }

    #[test]
    fn clear_default_state_waiting_on_clears_a_live_mark() {
        let tmp = tmp_root();
        ok(crate::verbs::state_group::set_default_state_waiting_on(
            tmp.path(),
            "question",
            "why?",
            "sess-1",
        ));
        let marked = ok(crate::verbs::state_group::read_state_strict(tmp.path()));
        assert!(waiting_on_is_live(marked.get("waiting_on")));
        assert_eq!(marked.get("run_state"), Some(&json!("awaiting-approval")));

        let cleared = ok(clear_default_state_waiting_on(tmp.path()));
        assert_eq!(cleared.get("waiting_on"), Some(&Value::Null));
        let on_disk = ok(crate::verbs::state_group::read_state_strict(tmp.path()));
        assert_eq!(on_disk.get("waiting_on"), Some(&Value::Null));
    }

    #[test]
    fn clear_default_state_waiting_on_is_a_no_op_when_nothing_is_live() {
        let tmp = tmp_root();
        // No .bee/state.json at all yet — clearing must not refuse or create one
        // with a bogus mark.
        let cleared = ok(clear_default_state_waiting_on(tmp.path()));
        assert_eq!(cleared.get("waiting_on"), Some(&Value::Null));
    }

    // ── awaiting-human: stale expiry (D4, ah-2) ─────────────────────────────

    const WAITING_ON_OLD_ASKED_AT: &str = "2020-01-01T00:00:00.000Z";

    fn write_session_fixture(root: &Path, id: &str, last_heartbeat: &str) {
        let dir = crate::verbs::cells::sessions_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        let rec = json!({"id": id, "started_at": last_heartbeat, "last_heartbeat": last_heartbeat});
        std::fs::write(dir.join(format!("{id}.json")), jsjson::stringify_pretty(&rec)).unwrap();
    }

    /// Pure predicate: age past the threshold is necessary but, on its own,
    /// never sufficient — the trap D4 exists to close.
    #[test]
    fn waiting_on_expired_requires_both_conditions_age_alone_never_expires() {
        let now = crate::verbs::reservations::now_ms();
        let old_mark = json!({
            "kind": "question", "subject": "x", "session": "s",
            "asked_at": WAITING_ON_OLD_ASKED_AT,
        });
        let fresh_mark = json!({
            "kind": "question", "subject": "x", "session": "s",
            "asked_at": ok(crate::verbs::reservations::iso_from_ms(now)),
        });
        assert!(waiting_on_age_expired(&old_mark, now), "old enough on its own");
        assert!(!waiting_on_age_expired(&fresh_mark, now), "not old enough");

        // The trap: age past threshold, heartbeat FRESH (owner_heartbeat_stale
        // = false) — the mark SURVIVES.
        assert!(
            !waiting_on_expired(&old_mark, now, false),
            "age alone must never expire a mark whose owning session is plainly alive"
        );
        // Both conditions hold — the mark expires.
        assert!(waiting_on_expired(&old_mark, now, true));
        // Heartbeat stale but the mark itself is still fresh — never expires.
        assert!(!waiting_on_expired(&fresh_mark, now, true));
    }

    #[test]
    fn waiting_on_age_expired_tolerates_a_missing_or_unparseable_asked_at() {
        let now = crate::verbs::reservations::now_ms();
        assert!(!waiting_on_age_expired(&json!({"kind": "gate", "subject": "x"}), now));
        assert!(!waiting_on_age_expired(&json!({"asked_at": "not a date"}), now));
    }

    /// The must-have this cell exists to prove: a workflow's waiting_on mark
    /// whose age is past the threshold but whose owning session's heartbeat
    /// is FRESH survives a reap untouched.
    #[test]
    fn reap_stale_waiting_on_survives_a_live_owner_session() {
        let tmp = tmp_root();
        let record = ok(create_workflow(tmp.path(), NewWorkflow::for_feature("f1")));
        let id = record.get("id").unwrap().as_str().unwrap().to_string();
        ok(set_workflow_waiting_on(tmp.path(), &id, "question", "why?", "live-sess"));
        // Back-date asked_at past the threshold by hand-editing the record
        // (build_waiting_on always stamps "now").
        let mut patch = Map::new();
        patch.insert(
            "waiting_on".into(),
            json!({"kind": "question", "subject": "why?", "session": "live-sess", "asked_at": WAITING_ON_OLD_ASKED_AT}),
        );
        ok(update_workflow(tmp.path(), &id, patch));
        let fresh = ok(crate::verbs::reservations::iso_from_ms(crate::verbs::reservations::now_ms()));
        write_session_fixture(tmp.path(), "live-sess", &fresh);

        reap_stale_waiting_on(tmp.path(), crate::verbs::reservations::now_ms());

        let on_disk = ok(read_workflow_record(tmp.path(), &id));
        assert!(
            waiting_on_is_live(on_disk.get("waiting_on")),
            "a plainly alive owner session must never lose its mark to age alone: {on_disk:?}"
        );
    }

    /// The other half: the SAME old mark, but its owning session's heartbeat
    /// has also gone stale — the reap clears it.
    #[test]
    fn reap_stale_waiting_on_clears_a_mark_whose_owner_session_is_dead() {
        let tmp = tmp_root();
        let record = ok(create_workflow(tmp.path(), NewWorkflow::for_feature("f1")));
        let id = record.get("id").unwrap().as_str().unwrap().to_string();
        ok(set_workflow_waiting_on(tmp.path(), &id, "question", "why?", "dead-sess"));
        let mut patch = Map::new();
        patch.insert(
            "waiting_on".into(),
            json!({"kind": "question", "subject": "why?", "session": "dead-sess", "asked_at": WAITING_ON_OLD_ASKED_AT}),
        );
        ok(update_workflow(tmp.path(), &id, patch));
        write_session_fixture(tmp.path(), "dead-sess", WAITING_ON_OLD_ASKED_AT); // stale heartbeat too

        reap_stale_waiting_on(tmp.path(), crate::verbs::reservations::now_ms());

        let on_disk = ok(read_workflow_record(tmp.path(), &id));
        assert_eq!(on_disk.get("waiting_on"), Some(&Value::Null), "dead owner: reaped");
    }

    /// D3's session-scoped case reaps exactly like the feature-scoped one.
    #[test]
    fn reap_stale_waiting_on_covers_the_default_state_record_too() {
        let tmp = tmp_root();
        ok(crate::verbs::state_group::set_default_state_waiting_on(
            tmp.path(),
            "question",
            "why?",
            "dead-sess",
        ));
        let mut current = ok(crate::verbs::state_group::read_state_strict(tmp.path()));
        current.insert(
            "waiting_on".into(),
            json!({"kind": "question", "subject": "why?", "session": "dead-sess", "asked_at": WAITING_ON_OLD_ASKED_AT}),
        );
        ok(write_state(tmp.path(), &current));
        write_session_fixture(tmp.path(), "dead-sess", WAITING_ON_OLD_ASKED_AT);

        reap_stale_waiting_on(tmp.path(), crate::verbs::reservations::now_ms());

        let on_disk = ok(crate::verbs::state_group::read_state_strict(tmp.path()));
        assert_eq!(on_disk.get("waiting_on"), Some(&Value::Null), "dead owner: reaped");
    }

    // ── listWorkflows skip tolerance (R6 blocker, now native) ─────────────

    /// The three ordinary skips, each with the reason bytes `read_workflow_
    /// record` hands `console.warn`. Named here so the warn-stream tests and
    /// the reason-shape tests cannot drift apart.
    fn seed_the_three_ordinary_skips(root: &Path) -> Vec<String> {
        let dir = workflows_dir(root);
        // (1) directory present, no state.json → WORKFLOW_MISSING
        std::fs::create_dir_all(dir.join("wf-missing")).unwrap();
        // (2) present but not a JSON object
        std::fs::create_dir_all(dir.join("wf-array")).unwrap();
        std::fs::write(dir.join("wf-array").join("state.json"), "[1,2]").unwrap();
        // (3) present, an object, but its id names someone else
        write_workflow(root, "wf-wrongid", json!({"id":"somebody-else","feature":"f"}));
        vec![
            format!(
                "readWorkflow: no workflow record at \"{}\". FIX: createWorkflow first, or check the id.",
                workflow_state_path(root, "wf-missing").display()
            ),
            format!(
                "readWorkflow: \"{}\" exists but is not a JSON object (found an array).",
                workflow_state_path(root, "wf-array").display()
            ),
            format!(
                "readWorkflow: \"{}\" exists but its id field (\"somebody-else\") does not match the \
requested workflow \"wf-wrongid\" — never trusted. FIX: inspect/restore the file (e.g. \"git \
checkout -- {}\").",
                workflow_state_path(root, "wf-wrongid").display(),
                format!(".bee{0}runtime{0}workflows{0}wf-wrongid{0}state.json", MAIN_SEPARATOR)
            ),
        ]
    }

    #[test]
    fn list_workflows_skips_the_three_ordinary_shapes_and_keeps_the_readable_ones() {
        let tmp = tmp_root();
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        assert_eq!(ok(list_workflows(tmp.path())).len(), 1);

        let reasons = seed_the_three_ordinary_skips(tmp.path());
        // Every skip is tolerated: the listing still returns the good record.
        let listed = ok(list_workflows(tmp.path()));
        assert_eq!(listed.len(), 1, "the readable record survives every skip");
        assert_eq!(wf_id(&listed[0]), "wf-1");

        // ...and each skip reason is the exact WorkflowStoreError message.
        for (id, reason) in ["wf-missing", "wf-array", "wf-wrongid"].iter().zip(&reasons) {
            match read_workflow_record(tmp.path(), id) {
                Err(WfSkip(m)) => assert_eq!(&m, reason, "reason bytes for {id}"),
                _ => panic!("expected an ordinary (native) skip for {id}"),
            }
        }

        // A non-directory entry is skipped SILENTLY by Node — no warn at all.
        std::fs::write(workflows_dir(tmp.path()).join("README"), "x").unwrap();
        assert_eq!(ok(list_workflows(tmp.path())).len(), 1);
    }

    #[test]
    fn the_warn_line_is_console_warns_own_shape() {
        let tmp = tmp_root();
        let reasons = seed_the_three_ordinary_skips(tmp.path());
        assert_eq!(
            skip_warn_line("wf-missing", &reasons[0]),
            format!(
                "listWorkflows: skipping unreadable workflow \"wf-missing\" — readWorkflow: no \
workflow record at \"{}\". FIX: createWorkflow first, or check the id.",
                workflow_state_path(tmp.path(), "wf-missing").display()
            )
        );
    }

    /// CUTOVER (was `only_the_two_v8_worded_arms_still_delegate`). The two
    /// residue arms are native: each is an ordinary skip with a reason we
    /// author, the listing survives, and nothing routes back to Node.
    #[test]
    fn the_two_residue_arms_are_ordinary_native_skips() {
        let tmp = tmp_root();
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));

        // (1) unparseable JSON — was `(${err.message})`, a V8 parse message.
        let dir = workflows_dir(tmp.path()).join("wf-badjson");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("state.json"), "{not json").unwrap();
        match read_workflow_record(tmp.path(), "wf-badjson") {
            Err(WfSkip(m)) => {
                assert_eq!(
                    m,
                    format!(
                        "readWorkflow: \"{}\" exists but is not valid JSON. The bee CLI refuses to \
rebuild a workflow from defaults over a present-but-corrupt file — that would silently clobber real \
state (gates, phase) while reporting success. FIX: inspect/restore the file (e.g. \"git checkout -- \
{}\").",
                        workflow_state_path(tmp.path(), "wf-badjson").display(),
                        format!(".bee{0}runtime{0}workflows{0}wf-badjson{0}state.json", MAIN_SEPARATOR)
                    )
                );
                assert!(!m.contains("Unexpected token"), "no V8 text: {m}");
            }
            _ => panic!("expected a native skip"),
        }
        let listed = ok(list_workflows(tmp.path()));
        assert_eq!(listed.len(), 1, "the readable record survives the corrupt one");
        assert_eq!(wf_id(&listed[0]), "wf-1");

        // (2) present-but-unreadable — was `(${err.code})`, a libuv errno
        // string. A directory in place of state.json reaches it portably.
        std::fs::remove_dir_all(&dir).unwrap();
        let d2 = workflows_dir(tmp.path()).join("wf-eisdir");
        std::fs::create_dir_all(d2.join("state.json")).unwrap();
        match read_workflow_record(tmp.path(), "wf-eisdir") {
            Err(WfSkip(m)) => {
                assert!(m.starts_with("readWorkflow: could not read "), "{m}");
                assert!(m.contains("(the path is a directory)"), "{m}");
                assert!(!m.contains("EISDIR") && !m.contains("EACCES"), "no errno string: {m}");
                assert!(m.contains("refuses to guess at a workflow record it cannot read"), "{m}");
            }
            _ => panic!("expected a native skip"),
        }
        let listed = ok(list_workflows(tmp.path()));
        assert_eq!(listed.len(), 1);
        assert_eq!(wf_id(&listed[0]), "wf-1");
    }

    /// CUTOVER (was `a_delegating_scan_emits_no_warn_before_it_bails`). The
    /// pre-pass existed only to keep a delegating scan silent; with nothing
    /// left to decide it is gone, and each bad record warns exactly ONCE per
    /// call — the corrupt and unreadable ones alongside the ordinary three.
    #[test]
    fn every_bad_record_warns_exactly_once_per_scan() {
        let tmp = tmp_root();
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        let ordinary = seed_the_three_ordinary_skips(tmp.path());
        let bad = workflows_dir(tmp.path()).join("wf-badjson");
        std::fs::create_dir_all(&bad).unwrap();
        std::fs::write(bad.join("state.json"), "{not json").unwrap();
        let unreadable = workflows_dir(tmp.path()).join("wf-eisdir");
        std::fs::create_dir_all(unreadable.join("state.json")).unwrap();

        let listed = ok(list_workflows(tmp.path()));
        assert_eq!(listed.len(), 1, "one good record survives five skips");
        assert_eq!(wf_id(&listed[0]), "wf-1");

        // One warn line per bad record, in the same sentence Node used.
        let mut lines: Vec<String> = Vec::new();
        for id in ["wf-missing", "wf-array", "wf-wrongid", "wf-badjson", "wf-eisdir"] {
            match read_workflow_record(tmp.path(), id) {
                Err(WfSkip(m)) => lines.push(skip_warn_line(id, &m)),
                _ => panic!("expected a skip for {id}"),
            }
        }
        assert_eq!(lines.len(), 5);
        for (line, id) in lines.iter().zip([
            "wf-missing", "wf-array", "wf-wrongid", "wf-badjson", "wf-eisdir",
        ]) {
            assert!(
                line.starts_with(&format!("listWorkflows: skipping unreadable workflow \"{id}\" — ")),
                "{line}"
            );
        }
        // The three ordinary reasons are byte-unchanged by the cutover.
        for (line, reason) in lines.iter().take(3).zip(&ordinary) {
            assert!(line.ends_with(reason.as_str()), "{line}");
        }
    }

    /// state.mjs readLane (display): a corrupt file used to delegate. It now
    /// reads as "no lane" — readJson's `null` fallback — after printing the
    /// two lines Node printed.
    #[test]
    fn a_corrupt_lane_record_reads_as_no_lane() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = lanes_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("feat-x.json"), "{broken").unwrap();
        assert!(ok(read_lane_display(root, "feat-x")).is_none());
        // The strict sibling still REFUSES, with the same typed message.
        match read_lane_strict(root, "feat-x") {
            Err(Err2::Msg(m)) => {
                assert!(m.contains("exists but is corrupt"), "{m}");
                assert!(!m.contains("Unexpected token"), "no V8 text: {m}");
            }
            _ => panic!("expected the corrupt-lane refusal"),
        }
        // A readable lane beside it is unaffected.
        std::fs::write(dir.join("feat-y.json"), r#"{"feature":"feat-y","phase":"planning"}"#).unwrap();
        let lane = ok(read_lane_display(root, "feat-y")).unwrap();
        assert_eq!(lane.get("phase"), Some(&json!("planning")));
    }

    /// readLaneStrict's unreadable arm: the refusal keeps its shape and exit
    /// path, with an engine-free category in place of the libuv errno code.
    #[test]
    fn read_lane_strict_unreadable_names_an_engine_free_category() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(lanes_dir(root).join("feat-d.json")).unwrap();
        match read_lane_strict(root, "feat-d") {
            Err(Err2::Msg(m)) => {
                assert!(m.starts_with("readLaneStrict: could not read lane record "), "{m}");
                assert!(m.contains("(the path is a directory)"), "{m}");
                assert!(!m.contains("EISDIR"), "no errno string: {m}");
            }
            _ => panic!("expected the unreadable refusal"),
        }
    }

    /// listHandoffMailbox: a corrupt record is skipped (readJson's `null`
    /// fallback fails Node's `!raw` guard) and the rest of the mailbox lists.
    #[test]
    fn a_corrupt_mailbox_record_is_skipped_not_delegated() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = handoff_mailbox_dir(root, "wf-1").unwrap();
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("0001.json"), r#"{"kind":"pause","note":"first"}"#).unwrap();
        std::fs::write(dir.join("0002.json"), "{broken").unwrap();
        std::fs::write(dir.join("0003.json"), r#"{"kind":"pause","note":"third"}"#).unwrap();
        let records = ok(list_handoff_mailbox(root, "wf-1"));
        assert_eq!(records.len(), 2, "the corrupt record is skipped");
        assert_eq!(records[0].get("seq"), Some(&json!(1)));
        assert_eq!(records[1].get("seq"), Some(&json!(3)));
    }

    #[test]
    fn update_assuming_lock_protects_identity_and_merges_gates() {
        let tmp = tmp_root();
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","created_at":"2026-01-01T00:00:00.000Z",
                   "phase":"planning","plan_rev":2,
                   "gates":{"execution":{"approved":true,"approved_for_plan_rev":2}}}),
        );
        let mut patch = Map::new();
        patch.insert("id".into(), json!("hacked"));
        patch.insert("feature".into(), json!("hacked"));
        patch.insert("created_at".into(), json!("hacked"));
        patch.insert("phase".into(), json!("swarming"));
        patch.insert("gates".into(), json!({"execution":{"approved":false}}));
        let next = ok(update_workflow_assuming_lock(tmp.path(), "wf-1", patch));
        assert_eq!(next.get("id"), Some(&json!("wf-1")));
        assert_eq!(next.get("feature"), Some(&json!("f1")));
        assert_eq!(next.get("created_at"), Some(&json!("2026-01-01T00:00:00.000Z")));
        assert_eq!(next.get("phase"), Some(&json!("swarming")));
        // approved flipped; the rev stamp survived (mergeGates one level deep).
        let gates = next.get("gates").unwrap();
        assert_eq!(jget(gates, "execution").unwrap()["approved"], json!(false));
        // JSON.parse yields JS numbers, so js_numberify makes every parsed
        // number an f64 — jsjson prints 2.0 back as "2", byte-identically.
        assert_eq!(jget(gates, "execution").unwrap()["approved_for_plan_rev"], json!(2.0));
        // …and it landed on disk.
        let on_disk = read_back(&workflow_state_path(tmp.path(), "wf-1"));
        assert_eq!(on_disk["phase"], json!("swarming"));
    }

    #[test]
    fn update_assuming_lock_refuses_bad_status() {
        let tmp = tmp_root();
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        let mut patch = Map::new();
        patch.insert("status".into(), json!("zombie"));
        match update_workflow_assuming_lock(tmp.path(), "wf-1", patch) {
            Err(Err2::Msg(m)) => assert_eq!(
                m,
                "updateWorkflowAssumingLock: status must be one of active/paused/closed (got \"zombie\")."
            ),
            _ => panic!("expected the status refusal"),
        }
    }

    #[test]
    fn update_assuming_lock_refuses_bad_run_state() {
        let tmp = tmp_root();
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        let mut patch = Map::new();
        patch.insert("run_state".into(), json!("zombie"));
        match update_workflow_assuming_lock(tmp.path(), "wf-1", patch) {
            Err(Err2::Msg(m)) => assert_eq!(
                m,
                "updateWorkflowAssumingLock: run_state must be one of shaping/awaiting-approval/running/blocked/done (got \"zombie\")."
            ),
            _ => panic!("expected the run_state refusal"),
        }
    }

    #[test]
    fn workflow_lock_name_matches_node_and_is_per_id() {
        let tmp = tmp_root();
        let g = ok(acquire_workflow_lock(tmp.path(), "wf-a"));
        // A DIFFERENT id is a distinct lock file (sanitizeLockName hashes ':').
        let g2 = ok(acquire_workflow_lock(tmp.path(), "wf-b"));
        assert!(lock::lock_file_path(tmp.path(), "workflow:wf-a").exists());
        assert!(lock::lock_file_path(tmp.path(), "workflow:wf-b").exists());
        assert_ne!(
            lock::lock_file_path(tmp.path(), "workflow:wf-a"),
            lock::lock_file_path(tmp.path(), "workflow:wf-b")
        );
        drop(g);
        drop(g2);
        assert!(!lock::lock_file_path(tmp.path(), "workflow:wf-a").exists());
        assert_eq!(projection_lock_name(true, Some("f1")), "lane:f1");
        assert_eq!(projection_lock_name(false, Some("f1")), "state");
        assert_eq!(projection_lock_name(true, None), "state");
    }

    // ── projections ───────────────────────────────────────────────────────

    #[test]
    fn gates_project_plan_rev_effective_approval() {
        let gates = json!({
            "context": {"approved": true, "approved_for_plan_rev": null},
            "shape": {"approved": true},
            "execution": {"approved": true, "approved_for_plan_rev": 3},
            "review": {"approved": false},
        });
        // plan_rev 3: the stamped execution gate is effective.
        assert_eq!(
            jsjson::stringify(&workflow_gates_to_approved_gates(Some(&gates), Some(&json!(3)))),
            r#"{"context":true,"shape":true,"execution":true,"review":false}"#
        );
        // plan_rev 4 (a bump): execution goes ineffective, the rest are immune.
        assert_eq!(
            jsjson::stringify(&workflow_gates_to_approved_gates(Some(&gates), Some(&json!(4)))),
            r#"{"context":true,"shape":true,"execution":false,"review":false}"#
        );
    }

    #[test]
    fn newest_active_workflow_skips_closed_and_terminal_and_breaks_ties_by_id() {
        let mk = |id: &str, status: &str, phase: &str, at: &str| -> Map<String, Value> {
            match json!({"id":id,"feature":id,"status":status,"phase":phase,"created_at":at}) {
                Value::Object(m) => m,
                _ => unreachable!(),
            }
        };
        let wfs = vec![
            mk("wf-a", "active", "planning", "2026-01-01T00:00:00.000Z"),
            mk("wf-z", "active", "planning", "2026-01-01T00:00:00.000Z"),
            mk("wf-newer", "closed", "planning", "2026-05-01T00:00:00.000Z"),
            mk("wf-term", "active", "compounding-complete", "2026-06-01T00:00:00.000Z"),
        ];
        let picked = ok(pick_newest_active_workflow(&wfs)).unwrap();
        // Same created_at → id DESCENDING wins.
        assert_eq!(picked.get("id"), Some(&json!("wf-z")));
        let none: Vec<Map<String, Value>> = vec![mk("wf-c", "closed", "idle", "2026-01-01T00:00:00.000Z")];
        assert!(ok(pick_newest_active_workflow(&none)).is_none());
    }

    /// The C1 fallback and the two authoritative branches, pinned against the
    /// EXACT fixtures src/hooks/state_sync.rs's own projection tests use
    /// (`rebuild_with_zero_workflows_is_overrides_only`,
    /// `rebuild_idle_bootstrap_adopts_newest_active_workflow`,
    /// `rebuild_feature_match_projects_gates_with_plan_rev`). state_sync.rs's
    /// copies of these functions are module-private and that file is outside
    /// this cell's touchable set, so this test is the standing proof that the
    /// two ports agree — the only difference being the overrides the hook
    /// always passes and the CLI never does (with none, a non-authoritative
    /// branch writes nothing at all).
    #[test]
    fn agrees_with_state_sync_port_on_shared_fixtures() {
        // (1) zero workflow records — pure no-op, D1 fields untouched.
        let tmp = tmp_root();
        write_state_file(
            tmp.path(),
            r#"{"schema_version":"1.0","phase":"swarming","feature":"f1","extra":42}"#,
        );
        ok(rebuild_state_projection(tmp.path()));
        let out = read_back(&tmp.path().join(".bee").join("state.json"));
        assert_eq!(out["phase"], json!("swarming"));
        assert_eq!(out["extra"], json!(42));
        assert!(out.get("cells").is_none(), "no overrides → nothing added");

        // (2) idle bootstrap adopts the newest ACTIVE workflow; a gate stamped
        // for a rev the record is not at projects false.
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"idle"}"#);
        write_workflow(
            tmp.path(),
            "wf-old",
            json!({"id":"wf-old","feature":"f-old","status":"active","phase":"planning",
                   "mode":"standard","plan_rev":0,"summary":"s","next_action":"n",
                   "created_at":"2026-01-01T00:00:00.000Z",
                   "gates":{"shape":{"approved":true,"approved_for_plan_rev":0}}}),
        );
        write_workflow(
            tmp.path(),
            "wf-new",
            json!({"id":"wf-new","feature":"f-new","status":"active","phase":"swarming",
                   "mode":"standard","plan_rev":1,"summary":"s2","next_action":"n2",
                   "created_at":"2026-02-01T00:00:00.000Z",
                   "gates":{"execution":{"approved":true,"approved_for_plan_rev":2}}}),
        );
        ok(rebuild_state_projection(tmp.path()));
        let out = read_back(&tmp.path().join(".bee").join("state.json"));
        assert_eq!(out["feature"], json!("f-new"));
        assert_eq!(out["phase"], json!("swarming"));
        assert_eq!(
            out["approved_gates"],
            json!({"context":false,"shape":false,"execution":false,"review":false})
        );

        // (3) feature-matched branch, pass-through fields survive.
        let tmp = tmp_root();
        write_state_file(
            tmp.path(),
            r#"{"phase":"planning","feature":"f1","workers":["w"]}"#,
        );
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"swarming",
                   "mode":"lane","plan_rev":3,"summary":"sum","next_action":"next",
                   "created_at":"2026-01-01T00:00:00.000Z",
                   "gates":{"context":{"approved":true,"approved_for_plan_rev":null},
                            "execution":{"approved":true,"approved_for_plan_rev":3}}}),
        );
        ok(rebuild_state_projection(tmp.path()));
        let out = read_back(&tmp.path().join(".bee").join("state.json"));
        assert_eq!(out["phase"], json!("swarming"));
        assert_eq!(out["mode"], json!("lane"));
        assert_eq!(out["summary"], json!("sum"));
        assert_eq!(out["workers"], json!(["w"]));
        assert_eq!(
            out["approved_gates"],
            json!({"context":true,"shape":false,"execution":true,"review":false})
        );
    }

    /// CRITICAL, plan.md's named trap: `apply_workflow_d1_fields` copies a
    /// FIXED field list into `.bee/state.json`. A new record field does NOT
    /// reach the projection unless it is named in that list — so a rebuild
    /// round-trip alone (byte-identical) proves nothing about a field that
    /// was never copied at all. This test asserts the projection ACTUALLY
    /// CARRIES `run_state`, not merely that a rebuild is idempotent.
    #[test]
    fn apply_workflow_d1_fields_carries_run_state_into_the_state_projection() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"f1"}"#);
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"swarming",
                   "mode":"standard","plan_rev":1,"summary":"s","next_action":"n",
                   "created_at":"2026-01-01T00:00:00.000Z","run_state":"running",
                   "gates":{"execution":{"approved":true,"approved_for_plan_rev":1,"state":"approved"}}}),
        );
        ok(rebuild_state_projection(tmp.path()));
        let out = read_back(&tmp.path().join(".bee").join("state.json"));
        assert_eq!(out["run_state"], json!("running"), "apply_workflow_d1_fields must copy run_state");

        // Also directly on the function, in case a future caller stops
        // going through rebuild_state_projection: the field-copy itself.
        let mut next = Map::new();
        let wf = ok(read_workflow_record(tmp.path(), "wf-1"));
        apply_workflow_d1_fields(&mut next, &wf);
        assert_eq!(next.get("run_state"), Some(&json!("running")));
    }

    /// CRITICAL, same trap, this feature's own field: `apply_workflow_d1_fields`
    /// must ALSO carry `waiting_on`, or the mark reaches the record but never
    /// `.bee/state.json` — invisible to `bee status --json`. This test starts
    /// with a state.json that has NO mark at all and asserts one appears
    /// after a rebuild, rather than merely checking a rebuild is idempotent
    /// (which would pass vacuously on an omitted field).
    #[test]
    fn apply_workflow_d1_fields_carries_waiting_on_into_the_state_projection() {
        let tmp = tmp_root();
        // No `waiting_on` key at all on the starting state.json.
        write_state_file(tmp.path(), r#"{"phase":"planning","feature":"f1"}"#);
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"swarming",
                   "mode":"standard","plan_rev":1,"summary":"s","next_action":"n",
                   "created_at":"2026-01-01T00:00:00.000Z","run_state":"awaiting-approval",
                   "waiting_on":{"kind":"question","subject":"why is X true?",
                                 "asked_at":"2026-08-14T00:00:00.000Z","session":"sess-1"},
                   "gates":{"execution":{"approved":true,"approved_for_plan_rev":1,"state":"approved"}}}),
        );
        let before = read_back(&tmp.path().join(".bee").join("state.json"));
        assert!(before.get("waiting_on").is_none(), "starts with no mark at all");

        ok(rebuild_state_projection(tmp.path()));
        let out = read_back(&tmp.path().join(".bee").join("state.json"));
        assert_eq!(
            out["waiting_on"],
            json!({"kind":"question","subject":"why is X true?",
                   "asked_at":"2026-08-14T00:00:00.000Z","session":"sess-1"}),
            "apply_workflow_d1_fields must copy waiting_on"
        );

        // Also directly on the function, the same belt-and-suspenders check
        // run_state's own sibling test applies.
        let mut next = Map::new();
        let wf = ok(read_workflow_record(tmp.path(), "wf-1"));
        apply_workflow_d1_fields(&mut next, &wf);
        assert_eq!(next.get("waiting_on").and_then(|v| v.get("subject")), Some(&json!("why is X true?")));
    }

    #[test]
    fn state_projection_is_a_noop_when_no_live_workflow_names_the_feature() {
        let tmp = tmp_root();
        write_state_file(tmp.path(), r#"{"phase":"swarming","feature":"f1"}"#);
        write_workflow(
            tmp.path(),
            "wf-other",
            json!({"id":"wf-other","feature":"other","status":"active","phase":"idle",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        ok(rebuild_state_projection(tmp.path()));
        let out = read_back(&tmp.path().join(".bee").join("state.json"));
        assert_eq!(out["phase"], json!("swarming"), "untouched");
        // A CLOSED workflow naming the feature is also no authority.
        write_workflow(
            tmp.path(),
            "wf-closed",
            json!({"id":"wf-closed","feature":"f1","status":"closed","phase":"idle",
                   "created_at":"2026-01-01T00:00:00.000Z"}),
        );
        ok(rebuild_state_projection(tmp.path()));
        assert_eq!(
            read_back(&tmp.path().join(".bee").join("state.json"))["phase"],
            json!("swarming")
        );
    }

    #[test]
    fn lane_projection_rebuilds_from_the_record_and_keeps_ad_hoc_fields() {
        let tmp = tmp_root();
        write_lane_file(
            tmp.path(),
            "f1",
            r#"{"schema_version":"1.0","feature":"f1","phase":"idle","created_at":"2026-01-01T00:00:00.000Z","last_scribing_run":{"feature":"f1"}}"#,
        );
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"f1","status":"active","phase":"swarming",
                   "mode":"standard","plan_rev":1,"summary":"S","next_action":"N",
                   "created_at":"2026-03-03T00:00:00.000Z",
                   "gates":{"shape":{"approved":true,"approved_for_plan_rev":1}}}),
        );
        let lane = ok(rebuild_lane_projection(tmp.path(), "f1")).unwrap();
        assert_eq!(lane.get("phase"), Some(&json!("swarming")));
        assert_eq!(lane.get("summary"), Some(&json!("S")));
        // created_at keeps the LANE's original identity timestamp.
        assert_eq!(lane.get("created_at"), Some(&json!("2026-01-01T00:00:00.000Z")));
        // Ad hoc lane-only fields pass through.
        assert!(lane.contains_key("last_scribing_run"));
        assert_eq!(
            jsjson::stringify(lane.get("approved_gates").unwrap()),
            r#"{"context":false,"shape":true,"execution":false,"review":false}"#
        );
        // No live workflow → no-op, existing record returned.
        let none = ok(rebuild_lane_projection(tmp.path(), "nolane"));
        assert!(none.is_none());
    }

    #[test]
    fn lane_projection_seeds_created_at_from_the_record_when_no_file_exists() {
        let tmp = tmp_root();
        write_workflow(
            tmp.path(),
            "wf-1",
            json!({"id":"wf-1","feature":"fresh","status":"active","phase":"planning",
                   "created_at":"2026-04-04T00:00:00.000Z"}),
        );
        let lane = ok(rebuild_lane_projection(tmp.path(), "fresh")).unwrap();
        let keys: Vec<&str> = lane.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "schema_version",
                "feature",
                "mode",
                "phase",
                "approved_gates",
                "summary",
                "next_action",
                "run_state",
                "waiting_on",
                "created_at"
            ]
        );
        assert_eq!(lane.get("created_at"), Some(&json!("2026-04-04T00:00:00.000Z")));
    }

    // ── lanes ─────────────────────────────────────────────────────────────

    #[test]
    fn lane_strict_refuses_corrupt_and_returns_none_for_missing() {
        let tmp = tmp_root();
        assert!(ok(read_lane_strict(tmp.path(), "nope")).is_none());
        write_lane_file(tmp.path(), "bad", "{not json");
        match read_lane_strict(tmp.path(), "bad") {
            Err(Err2::Msg(m)) => {
                assert!(m.contains("exists but is corrupt (not a JSON object naming feature \"bad\")"));
                assert!(!m.contains("Unexpected token"), "no V8 text in the refusal");
            }
            _ => panic!("expected the corrupt refusal"),
        }
        // A record naming a DIFFERENT feature is corrupt too.
        write_lane_file(tmp.path(), "mismatch", r#"{"feature":"other"}"#);
        assert!(matches!(read_lane_strict(tmp.path(), "mismatch"), Err(Err2::Msg(_))));
        // Bad names throw requireLaneFeature's own message.
        match read_lane_strict(tmp.path(), "a/b") {
            Err(Err2::Msg(m)) => assert_eq!(m, "lane feature must be a plain id (no path separators)."),
            _ => panic!("expected the name refusal"),
        }
        // Healthy record merges the per-feature defaults.
        write_lane_file(tmp.path(), "f1", r#"{"feature":"f1","phase":"swarming"}"#);
        let rec = ok(read_lane_strict(tmp.path(), "f1")).unwrap();
        assert_eq!(rec.get("phase"), Some(&json!("swarming")));
        assert_eq!(rec.get("schema_version"), Some(&json!("1.0")));
        assert_eq!(
            jsjson::stringify(rec.get("approved_gates").unwrap()),
            r#"{"context":false,"shape":false,"execution":false,"review":false}"#
        );
    }

    #[test]
    fn write_lane_round_trips_through_the_feature_filename() {
        let tmp = tmp_root();
        let mut lane = default_lane_record("f1");
        lane.insert("phase".into(), json!("reviewing"));
        ok(write_lane(tmp.path(), &lane));
        let back = ok(read_lane_strict(tmp.path(), "f1")).unwrap();
        assert_eq!(back.get("phase"), Some(&json!("reviewing")));
    }

    // ── handoff mailbox ───────────────────────────────────────────────────

    fn capped_cell_and_claim(root: &Path, cell: &str, session: &str) {
        let cells = root.join(".bee").join("cells");
        std::fs::create_dir_all(&cells).unwrap();
        std::fs::write(cells.join("prev.json"), r#"{"status":"capped"}"#).unwrap();
        let claims = root.join(".bee").join("claims");
        std::fs::create_dir_all(&claims).unwrap();
        std::fs::write(
            claims.join(format!("{cell}.json")),
            format!(r#"{{"cell":"{cell}","session":"{session}","fence_epoch":1}}"#),
        )
        .unwrap();
    }

    /// D4 fixture: stamp a session record's `source` the way session-init
    /// (bh-4) now persists it, without going through the hook.
    fn write_session_source(root: &Path, id: &str, source: &str) {
        let dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            serde_json::to_string(&json!({"id": id, "source": source})).unwrap(),
        )
        .unwrap();
    }

    fn planned_next_input() -> Map<String, Value> {
        let mut input = Map::new();
        input.insert("kind".into(), json!("planned-next"));
        input.insert("feature".into(), json!("f1"));
        input.insert("writer_session".into(), json!("sess-w"));
        input.insert("previous_cell".into(), json!("prev"));
        input.insert("next_cell".into(), json!("next"));
        input
    }

    #[test]
    fn mailbox_write_assigns_seq_clears_the_prior_open_record_and_scopes_by_role() {
        let tmp = tmp_root();
        capped_cell_and_claim(tmp.path(), "next", "sess-w");
        let input = planned_next_input();
        let r1 = ok(write_mailbox_handoff(tmp.path(), "wf-1", &input, None));
        assert_eq!(r1.get("seq"), Some(&json!(1)));
        assert_eq!(r1.get("id"), Some(&json!("wf-1-0001")));
        assert_eq!(r1.get("status"), Some(&json!("open")));
        assert_eq!(r1.get("claim_epoch"), Some(&json!(1.0))); // parsed → f64, prints "1"
        assert_eq!(r1.get("from_session"), Some(&json!("sess-w")));
        // Node's exact key order for a planned-next mailbox record.
        let keys: Vec<&str> = r1.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "id", "workflow_id", "target_role", "status", "written_at", "feature", "kind",
                "writer_session", "from_session", "previous_cell", "next_cell", "claim_epoch",
                "seq"
            ]
        );
        // A second write to the SAME (workflow, role) clears the first.
        let r2 = ok(write_mailbox_handoff(tmp.path(), "wf-1", &input, None));
        assert_eq!(r2.get("seq"), Some(&json!(2)));
        let all = ok(list_handoff_mailbox(tmp.path(), "wf-1"));
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].get("status"), Some(&json!("cleared")));
        assert_eq!(all[1].get("status"), Some(&json!("open")));
        // A DIFFERENT role gets its own slot and never touches the other.
        let r3 = ok(write_mailbox_handoff(tmp.path(), "wf-1", &input, Some("reviewer")));
        assert_eq!(r3.get("target_role"), Some(&json!("reviewer")));
        let all = ok(list_handoff_mailbox(tmp.path(), "wf-1"));
        assert_eq!(all[1].get("status"), Some(&json!("open")), "unscoped slot survives");
        let newest_unscoped = ok(newest_open_handoff_mailbox_record(tmp.path(), "wf-1", None)).unwrap();
        assert_eq!(record_seq(&newest_unscoped), 2);
        let newest_reviewer =
            ok(newest_open_handoff_mailbox_record(tmp.path(), "wf-1", Some("reviewer"))).unwrap();
        assert_eq!(record_seq(&newest_reviewer), 3);
    }

    #[test]
    fn mailbox_write_refuses_uncapped_previous_and_unowned_claim() {
        let tmp = tmp_root();
        let input = planned_next_input();
        match write_mailbox_handoff(tmp.path(), "wf-1", &input, None) {
            Err(Err2::Msg(m)) => {
                assert!(m.contains("previous cell \"prev\" is not capped (found status \"missing\")"))
            }
            _ => panic!("expected the uncapped refusal"),
        }
        let cells = tmp.path().join(".bee").join("cells");
        std::fs::create_dir_all(&cells).unwrap();
        std::fs::write(cells.join("prev.json"), r#"{"status":"capped"}"#).unwrap();
        match write_mailbox_handoff(tmp.path(), "wf-1", &input, None) {
            Err(Err2::Msg(m)) => assert!(m.contains("(found no claim)")),
            _ => panic!("expected the unowned-claim refusal"),
        }
        // Nothing was written on either refusal.
        assert!(ok(list_handoff_mailbox(tmp.path(), "wf-1")).is_empty());
    }

    #[test]
    fn mailbox_adopt_moves_the_claim_bumps_the_fence_and_clears() {
        let tmp = tmp_root();
        capped_cell_and_claim(tmp.path(), "next", "sess-w");
        ok(write_mailbox_handoff(tmp.path(), "wf-1", &planned_next_input(), None));
        match ok(adopt_mailbox_handoff(tmp.path(), "wf-1", "sess-new", None)) {
            MailboxAdopt::Ok { claim, previous_owner, next_cell, workflow_id, seq } => {
                assert_eq!(next_cell, "next");
                assert_eq!(workflow_id, "wf-1");
                assert_eq!(seq, 1);
                assert_eq!(previous_owner, Some(json!("sess-w")));
                let claim = claim.unwrap();
                assert_eq!(jget(&claim, "session"), Some(&json!("sess-new")));
                assert_eq!(jget(&claim, "fence_epoch"), Some(&json!(2.0)));
            }
            MailboxAdopt::Fail { reason } => panic!("unexpected refusal: {reason}"),
        }
        let all = ok(list_handoff_mailbox(tmp.path(), "wf-1"));
        assert_eq!(all[0].get("status"), Some(&json!("cleared")));
        assert_eq!(all[0].get("adopted_by"), Some(&json!("sess-new")));
        // A second adopt finds nothing open/adopted left.
        match ok(adopt_mailbox_handoff(tmp.path(), "wf-1", "sess-new", None)) {
            MailboxAdopt::Fail { reason } => {
                assert_eq!(reason, "no open handoff in workflow \"wf-1\"'s mailbox to adopt.")
            }
            _ => panic!("expected NO_HANDOFF"),
        }
    }

    #[test]
    fn mailbox_adopt_refuses_a_pause_record() {
        let tmp = tmp_root();
        let mut input = Map::new();
        input.insert("kind".into(), json!("pause"));
        input.insert("cell".into(), json!("wip"));
        let rec = ok(write_mailbox_handoff(tmp.path(), "wf-1", &input, None));
        // Node's exact key order for a pause mailbox record.
        let keys: Vec<&str> = rec.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            ["id", "workflow_id", "target_role", "status", "written_at", "cell", "kind", "from_session", "seq"]
        );
        match ok(adopt_mailbox_handoff(tmp.path(), "wf-1", "s", None)) {
            MailboxAdopt::Fail { reason } => assert!(reason.starts_with("handoff kind \"pause\" is not \"planned-next\"")),
            _ => panic!("expected NOT_PLANNED_NEXT"),
        }
    }

    // ── D4 fresh-session-boundary gate on `state handoff adopt` (bh-4) ─────

    #[test]
    fn handoff_adopt_source_gate_classifies_eligible_and_blocked_sources() {
        let tmp = tmp_root();
        for eligible in ["startup", "clear"] {
            write_session_source(tmp.path(), "s1", eligible);
            assert!(
                handoff_adopt_source_refusal(tmp.path(), "s1").is_none(),
                "source={eligible} must be adopt-eligible"
            );
        }
        for blocked in ["resume", "compact"] {
            write_session_source(tmp.path(), "s1", blocked);
            let reason = handoff_adopt_source_refusal(tmp.path(), "s1")
                .unwrap_or_else(|| panic!("source={blocked} must refuse adoption (D4)"));
            assert!(
                reason.starts_with(HANDOFF_ADOPT_SOURCE_REFUSAL_PREFIX),
                "refusal must carry the pinned prefix: {reason}"
            );
            assert!(reason.contains(blocked), "{reason}");
        }
        // No session record at all, and a record predating D4's `source`
        // field: both eligible, fail-open — a missing source is a warning,
        // never a refusal.
        assert!(handoff_adopt_source_refusal(tmp.path(), "no-such-session").is_none());
        let dir = tmp.path().join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("s2.json"), r#"{"id":"s2"}"#).unwrap();
        assert!(handoff_adopt_source_refusal(tmp.path(), "s2").is_none());
    }

    #[test]
    fn mailbox_adopt_refuses_a_resumed_or_compacted_session_and_leaves_the_handoff_open() {
        let tmp = tmp_root();
        capped_cell_and_claim(tmp.path(), "next", "sess-w");
        ok(write_mailbox_handoff(tmp.path(), "wf-1", &planned_next_input(), None));
        write_session_source(tmp.path(), "sess-new", "compact");
        match ok(adopt_mailbox_handoff(tmp.path(), "wf-1", "sess-new", None)) {
            MailboxAdopt::Fail { reason } => {
                assert!(reason.starts_with(HANDOFF_ADOPT_SOURCE_REFUSAL_PREFIX), "{reason}");
            }
            MailboxAdopt::Ok { .. } => panic!("a compacted session must never adopt (D4)"),
        }
        // Nothing moved: the handoff is still open, the claim untouched.
        let all = ok(list_handoff_mailbox(tmp.path(), "wf-1"));
        assert_eq!(all[0].get("status"), Some(&json!("open")));
        let claim: Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".bee").join("claims").join("next.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(claim["session"], json!("sess-w"), "the claim never moved");
    }

    #[test]
    fn mailbox_adopt_proceeds_for_a_fresh_session_boundary() {
        let tmp = tmp_root();
        capped_cell_and_claim(tmp.path(), "next", "sess-w");
        ok(write_mailbox_handoff(tmp.path(), "wf-1", &planned_next_input(), None));
        write_session_source(tmp.path(), "sess-new", "startup");
        match ok(adopt_mailbox_handoff(tmp.path(), "wf-1", "sess-new", None)) {
            MailboxAdopt::Ok { next_cell, .. } => assert_eq!(next_cell, "next"),
            MailboxAdopt::Fail { reason } => panic!("unexpected refusal: {reason}"),
        }
    }

    #[test]
    fn handoff_projection_picks_the_newest_open_record_and_strips_mailbox_fields() {
        let tmp = tmp_root();
        capped_cell_and_claim(tmp.path(), "next", "sess-w");
        write_workflow(tmp.path(), "wf-1", json!({"id":"wf-1","feature":"f1"}));
        write_workflow(tmp.path(), "wf-2", json!({"id":"wf-2","feature":"f2"}));
        ok(write_mailbox_handoff(tmp.path(), "wf-1", &planned_next_input(), None));
        ok(rebuild_handoff_projection(tmp.path()));
        let projected = read_back(&handoff_path(tmp.path()));
        assert_eq!(projected["kind"], json!("planned-next"));
        assert_eq!(projected["next_cell"], json!("next"));
        for stripped in ["seq", "status", "id", "workflow_id", "target_role", "from_session"] {
            assert!(projected.get(stripped).is_none(), "{stripped} must be stripped");
        }
        // Adopting clears the only open record → the legacy file is removed.
        ok(adopt_mailbox_handoff(tmp.path(), "wf-1", "sess-new", None));
        ok(rebuild_handoff_projection(tmp.path()));
        assert!(!handoff_path(tmp.path()).exists());
    }

    #[test]
    fn handoff_projection_is_a_noop_at_zero_workflow_records() {
        let tmp = tmp_root();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(handoff_path(tmp.path()), r#"{"kind":"pause"}"#).unwrap();
        ok(rebuild_handoff_projection(tmp.path()));
        assert!(handoff_path(tmp.path()).exists(), "C1: legacy file untouched");
    }

    #[test]
    fn workflows_list_sort_is_newest_created_first() {
        let mk = |id: &str, at: Value| -> Map<String, Value> {
            match json!({"id": id, "created_at": at}) {
                Value::Object(m) => m,
                _ => unreachable!(),
            }
        };
        let mut records = vec![
            mk("a", json!("2026-01-01T00:00:00.000Z")),
            mk("b", json!("2026-05-01T00:00:00.000Z")),
            mk("c", Value::Null),
        ];
        ok(workflows_list_sort(&mut records));
        let ids: Vec<String> = records.iter().map(|r| js_disp_opt(r.get("id"))).collect();
        assert_eq!(ids, ["b", "a", "c"]);
    }

    #[test]
    fn gates_patch_carries_stamps_only_for_the_named_gates() {
        let mut updated = Map::new();
        updated.insert(
            "approved_gates".into(),
            json!({"context": true, "shape": true, "execution": true, "review": false}),
        );
        let stamps = vec![
            ("shape".to_string(), json!(4)),
            ("execution".to_string(), json!(4)),
        ];
        assert_eq!(
            jsjson::stringify(&gates_patch_from_record(&updated, &stamps)),
            r#"{"context":{"approved":true,"state":"approved"},"shape":{"approved":true,"state":"approved","approved_for_plan_rev":4},"execution":{"approved":true,"state":"approved","approved_for_plan_rev":4},"review":{"approved":false,"state":"pending"}}"#
        );
        // No stamps at all: every gate carries `approved` (+ `state`) only,
        // so mergeGates preserves whatever rev each entry already had.
        assert_eq!(
            jsjson::stringify(&gates_patch_from_record(&updated, &[])),
            r#"{"context":{"approved":true,"state":"approved"},"shape":{"approved":true,"state":"approved"},"execution":{"approved":true,"state":"approved"},"review":{"approved":false,"state":"pending"}}"#
        );
    }

    #[test]
    fn gates_patch_derives_rejected_state_from_gate_revoked_at() {
        // A gate that was never approved stays `pending`; a gate the
        // execution-component revocation path stamped `gate_revoked_at` for
        // reads `rejected` — the invariant `approved == (state == "approved")`
        // holds in both cases, and the two are told apart.
        let mut updated = Map::new();
        updated.insert(
            "approved_gates".into(),
            json!({"context": false, "shape": false, "execution": false, "review": false}),
        );
        updated.insert("gate_revoked_at".into(), json!({"execution": "2026-08-14T00:00:00.000Z"}));
        let gates = gates_patch_from_record(&updated, &[]);
        assert_eq!(jget(&gates, "execution").unwrap()["approved"], json!(false));
        assert_eq!(jget(&gates, "execution").unwrap()["state"], json!("rejected"));
        assert_eq!(
            jget(&gates, "context").unwrap()["state"],
            json!("pending"),
            "never touched stays pending, not rejected"
        );
    }

    // ── R5 test migration: createWorkflow + locking + absent-store ─────────
    //
    // The createWorkflow rows of test_workflow_store.mjs (full-schema write,
    // refusal to overwrite an existing id, refusal when id === feature) now
    // have a Rust counterpart — see § createWorkflow below.
    //
    // The oracle's "listWorkflows … tolerant of an unreadable entry
    // (skip+report, never throws)" row is now ported too — see § listWorkflows
    // skip tolerance above. CUTOVER: the last two reasons (a V8 parse message
    // and a libuv errno string) are native, so NOTHING here delegates;
    // `the_two_residue_arms_are_ordinary_native_skips` and
    // `every_bad_record_warns_exactly_once_per_scan` pin both.

    // ══ createWorkflow (workflow-store.mjs) ════════════════════════════════

    /// Oracle: "createWorkflow writes the full schema and readWorkflow reads
    /// it back unchanged". Create and read must be BYTE-symmetric (mv-4), so
    /// the key order is asserted, not just the values.
    #[test]
    fn create_writes_the_full_schema_and_reads_back_identical() {
        let tmp = tmp_root();
        let root = tmp.path();
        let record = create_workflow(
            root,
            NewWorkflow {
                feature: Some("  billing-refunds  "),
                phase: Some(json!("planning")),
                mode: Some(json!("swarm")),
                plan_rev: Some(json!(2)),
                gates: Some(json!({"shape": {"approved": true}})),
                summary: Some(json!("s")),
                next_action: Some(json!("n")),
                status: Some("paused"),
                id: Some("wf-explicit"),
            },
        )
        .expect("create");

        let keys: Vec<&str> = record.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            vec![
                "mode", "phase", "plan_rev", "summary", "next_action", "status", "route",
                "run_state", "waiting_on", "id", "feature", "gates", "created_at"
            ],
            "a JS re-assignment keeps a key's original position — only id/feature/gates/created_at \
             append; run_state (trun-7) and waiting_on (awaiting-human) are base defaults, so both \
             sit with route, not appended"
        );
        assert_eq!(record["id"], json!("wf-explicit"));
        assert_eq!(record["feature"], json!("billing-refunds"), "the feature slug is trimmed");
        assert_eq!(record["phase"], json!("planning"));
        assert_eq!(record["mode"], json!("swarm"));
        assert_eq!(record["plan_rev"], json!(2));
        assert_eq!(record["status"], json!("paused"));
        assert_eq!(record["route"], Value::Null, "baseWorkflowDefaults' route survives");
        assert!(record["created_at"].as_str().unwrap().ends_with('Z'));
        // mergeGates(defaultGates(), overrides): the override is one level
        // deep over the default entry, and every GATE_NAME is still present.
        // D3 extends the default entry with `state` (+ actor/at/reason/
        // bypass_level); the overlaid `shape` entry's `approved:true` derives
        // `state:"approved"` since the override never named `state` itself.
        assert_eq!(
            jsjson::stringify(&record["gates"]),
            r#"{"context":{"approved":false,"approved_for_plan_rev":null,"state":"pending","actor":null,"at":null,"reason":null,"bypass_level":null},"shape":{"approved":true,"approved_for_plan_rev":null,"state":"approved","actor":null,"at":null,"reason":null,"bypass_level":null},"execution":{"approved":false,"approved_for_plan_rev":null,"state":"pending","actor":null,"at":null,"reason":null,"bypass_level":null},"review":{"approved":false,"approved_for_plan_rev":null,"state":"pending","actor":null,"at":null,"reason":null,"bypass_level":null}}"#
        );

        // The record is on disk at .bee/runtime/workflows/<id>/state.json.
        // This used to be byte-for-byte a live `node` run of
        // workflow-store.mjs createWorkflow (ORACLE-PINNED); D3 extends the
        // gate entry schema past what the Node oracle ever wrote, so the
        // fixture below is now pinned against THIS shape, not the oracle's.
        let file = workflow_state_path(root, "wf-explicit");
        let on_disk = std::fs::read_to_string(&file)
            .unwrap()
            .replace(record["created_at"].as_str().unwrap(), "<now>");
        assert_eq!(
            on_disk,
            "{\n  \"mode\": \"swarm\",\n  \"phase\": \"planning\",\n  \"plan_rev\": 2,\n  \"summary\": \"s\",\n  \"next_action\": \"n\",\n  \"status\": \"paused\",\n  \"route\": null,\n  \"run_state\": \"awaiting-approval\",\n  \"waiting_on\": null,\n  \"id\": \"wf-explicit\",\n  \"feature\": \"billing-refunds\",\n  \"gates\": {\n    \"context\": {\n      \"approved\": false,\n      \"approved_for_plan_rev\": null,\n      \"state\": \"pending\",\n      \"actor\": null,\n      \"at\": null,\n      \"reason\": null,\n      \"bypass_level\": null\n    },\n    \"shape\": {\n      \"approved\": true,\n      \"approved_for_plan_rev\": null,\n      \"state\": \"approved\",\n      \"actor\": null,\n      \"at\": null,\n      \"reason\": null,\n      \"bypass_level\": null\n    },\n    \"execution\": {\n      \"approved\": false,\n      \"approved_for_plan_rev\": null,\n      \"state\": \"pending\",\n      \"actor\": null,\n      \"at\": null,\n      \"reason\": null,\n      \"bypass_level\": null\n    },\n    \"review\": {\n      \"approved\": false,\n      \"approved_for_plan_rev\": null,\n      \"state\": \"pending\",\n      \"actor\": null,\n      \"at\": null,\n      \"reason\": null,\n      \"bypass_level\": null\n    }\n  },\n  \"created_at\": \"<now>\"\n}\n"
        );
        // … and readWorkflowRecord round-trips it with no drift at all.
        let read_back = read_workflow_record(root, "wf-explicit").ok().expect("readable");
        assert_eq!(jsjson::stringify(&Value::Object(read_back)), jsjson::stringify(&Value::Object(record)));
    }

    /// Oracle: "createWorkflow defaults every optional field" — and the
    /// generated id is never the feature slug.
    #[test]
    fn create_defaults_every_optional_field_and_generates_a_wf_prefixed_id() {
        let tmp = tmp_root();
        let root = tmp.path();
        let record = create_workflow(root, NewWorkflow::for_feature("f1")).expect("create");
        assert_eq!(record["phase"], json!("idle"));
        assert_eq!(record["mode"], Value::Null);
        assert_eq!(record["plan_rev"], json!(0));
        assert_eq!(record["summary"], json!(""));
        assert_eq!(record["next_action"], json!(""));
        assert_eq!(record["status"], json!("active"));
        let id = record["id"].as_str().unwrap();
        assert!(id.starts_with("wf-"), "{id}");
        assert_eq!(id.len(), 11, "wf- plus 4 bytes of hex: {id}");
        assert_ne!(id, "f1");
        assert!(record["gates"]["context"]["approved"] == json!(false));

        // Two creates never collide, and both are listed.
        let second = create_workflow(root, NewWorkflow::for_feature("f2")).expect("create");
        assert_ne!(second["id"], record["id"]);
        assert_eq!(ok(list_workflows(root)).len(), 2);
    }

    /// Oracle: "createWorkflow refuses to overwrite an existing record", "…
    /// refuses when id === feature (D1)", plus the id/feature/status
    /// validation ladder. Every refusal's bytes are pinned.
    #[test]
    fn create_refuses_on_every_invalid_shape_and_never_overwrites() {
        let tmp = tmp_root();
        let root = tmp.path();

        let msg = |r: Result<Map<String, Value>, Err2>| match r {
            Err(Err2::Msg(m)) => m,
            Err(Err2::Ex) => panic!("expected a typed refusal, got Exotic"),
            Ok(_) => panic!("expected a refusal"),
        };

        // feature is required — checked AFTER requireWorkflowId, so a valid
        // explicit id plus a blank feature reports the feature refusal.
        for feature in [None, Some(""), Some("   ")] {
            let mut opts = NewWorkflow::for_feature("x");
            opts.feature = feature;
            opts.id = Some("wf-a");
            assert_eq!(msg(create_workflow(root, opts)), "createWorkflow: feature is required.");
        }
        // requireWorkflowId fires first for a path-shaped explicit id.
        let mut opts = NewWorkflow::for_feature("f");
        opts.id = Some("  ");
        assert_eq!(msg(create_workflow(root, opts)), "workflow id is required.");
        let mut opts = NewWorkflow::for_feature("f");
        opts.id = Some("a/b");
        assert_eq!(
            msg(create_workflow(root, opts)),
            "workflow id \"a/b\" must be a plain id (no path separators) — it becomes a directory name under .bee/runtime/workflows/."
        );
        // D1: the id may never be the feature slug.
        let mut opts = NewWorkflow::for_feature("wf-thing");
        opts.id = Some("wf-thing");
        assert_eq!(
            msg(create_workflow(root, opts)),
            "createWorkflow: workflow id \"wf-thing\" must not equal the feature slug \"wf-thing\" — ids are \
generated identifiers, never feature slugs (CONTEXT.md D1: a feature can reopen or run competing \
attempts, so identity must never collide with the human-chosen name). FIX: pass an explicit id distinct \
from the feature, or omit id to let one be generated."
        );
        let mut opts = NewWorkflow::for_feature("f");
        opts.id = Some("wf-a");
        opts.status = Some("archived");
        assert_eq!(
            msg(create_workflow(root, opts)),
            "createWorkflow: status must be one of active/paused/closed (got \"archived\")."
        );
        // Not one of those refusals wrote anything.
        assert!(!workflows_dir(root).exists(), "a refused create never touches the store");

        // The overwrite refusal — reached AFTER the `workflow:<id>` lock, so
        // it must be native (campaign rule 2).
        let mut opts = NewWorkflow::for_feature("f");
        opts.id = Some("wf-dup");
        create_workflow(root, opts).expect("first create");
        let before = std::fs::read_to_string(workflow_state_path(root, "wf-dup")).unwrap();
        let mut opts = NewWorkflow::for_feature("other-feature");
        opts.id = Some("wf-dup");
        opts.status = Some("closed");
        assert_eq!(
            msg(create_workflow(root, opts)),
            format!(
                "createWorkflow: a workflow record already exists at \"{}\" — createWorkflow never overwrites an \
existing record. FIX: use updateWorkflow, or generate a fresh id.",
                workflow_state_path(root, "wf-dup").display()
            )
        );
        assert_eq!(
            std::fs::read_to_string(workflow_state_path(root, "wf-dup")).unwrap(),
            before,
            "the existing record is byte-identical after the refusal"
        );
    }

    /// createWorkflow takes `workflow:<id>` for its whole body — the same lock
    /// name updateWorkflow uses, so a racing create and update on one id
    /// serialize. Proven by holding the lock externally.
    #[test]
    fn create_takes_the_workflow_id_lock_for_its_whole_body() {
        let tmp = tmp_root();
        let root = tmp.path();
        let held = lock::acquire_store_lock(root, "workflow:wf-locked", 1)
            .unwrap_or_else(|b| panic!("precondition: {}", b.message()));
        let mut opts = NewWorkflow::for_feature("f");
        opts.id = Some("wf-locked");
        match create_workflow(root, opts) {
            Err(Err2::Msg(m)) => assert!(
                m.starts_with("lock \"workflow:wf-locked\" busy: held by "),
                "expected LOCK_BUSY, got {m}"
            ),
            other => panic!(
                "create must be denied under a held workflow lock, got {}",
                match other {
                    Ok(_) => "a record".to_string(),
                    Err(Err2::Ex) => "Exotic".to_string(),
                    Err(Err2::Msg(m)) => m,
                }
            ),
        }
        assert!(!workflow_state_path(root, "wf-locked").exists());
        drop(held);
        // Control: with the lock free the very same create succeeds.
        let mut opts = NewWorkflow::for_feature("f");
        opts.id = Some("wf-locked");
        assert!(create_workflow(root, opts).is_ok());
        // A DIFFERENT id is never blocked by another id's lock.
        let other = lock::acquire_store_lock(root, "workflow:wf-locked", 1)
            .unwrap_or_else(|b| panic!("precondition: {}", b.message()));
        let mut opts = NewWorkflow::for_feature("g");
        opts.id = Some("wf-sibling");
        assert!(create_workflow(root, opts).is_ok(), "sibling ids hash to distinct lock files");
        drop(other);
    }

    /// Oracle: "listWorkflows on an absent .bee/runtime/workflows/ directory
    /// returns an empty, non-throwing result".
    #[test]
    fn list_workflows_over_an_absent_store_is_empty_and_creates_nothing() {
        let tmp = tmp_root();
        let root = tmp.path();
        assert!(!workflows_dir(root).exists());
        assert!(ok(list_workflows(root)).is_empty(), "no workflows dir -> empty list");
        assert!(
            !workflows_dir(root).exists(),
            "listWorkflows never creates the directory as a side effect of listing"
        );
        // An EXISTING but empty store is the same answer, still without
        // inventing entries.
        std::fs::create_dir_all(workflows_dir(root)).unwrap();
        assert!(ok(list_workflows(root)).is_empty());
        // Control: the enumeration is not simply always-empty.
        write_workflow(root, "wf-1", json!({"id":"wf-1","feature":"f1"}));
        let listed = ok(list_workflows(root));
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].get("id"), Some(&json!("wf-1")));
    }

    /// Oracle: "updateWorkflowAssumingLock (multisession-native-10, C4):
    /// succeeds through an externally-held workflow:<id> lock that DENIES
    /// updateWorkflow itself — proves it takes no lock of its own".
    ///
    /// The negative control runs the real self-locking form against the real
    /// retry loop, so it spends MAX_ATTEMPTS × RETRY_DELAY (~5s) before
    /// reporting busy; Node's oracle short-circuits it with {maxAttempts: 1},
    /// which this port does not expose.
    #[test]
    fn update_assuming_lock_writes_through_an_externally_held_workflow_lock() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_workflow(root, "wf-1", json!({"id":"wf-1","feature":"assuming-lock-deadlock-proof"}));

        let held = lock::acquire_store_lock(root, "workflow:wf-1", 1)
            .unwrap_or_else(|b| panic!("precondition: the test must hold workflow:wf-1 — {}", b.message()));

        // Negative control: the SELF-LOCKING form is denied while that same
        // lock is held — the shape that would deadlock a caller already
        // inside its own withWorkflowLock hold.
        let mut patch = Map::new();
        patch.insert("phase".into(), json!("planning"));
        match update_workflow(root, "wf-1", patch) {
            Err(Err2::Msg(m)) => assert!(
                m.starts_with("lock \"workflow:wf-1\" busy: held by "),
                "expected the LOCK_BUSY refusal, got {m}"
            ),
            other => panic!("updateWorkflow must be denied under a held lock, got {}", match other {
                Ok(_) => "Ok".to_string(),
                Err(_) => "a non-message error".to_string(),
            }),
        }
        // …and the denial wrote nothing.
        assert_eq!(read_back(&workflow_state_path(root, "wf-1")).get("phase"), None);

        // The real proof: the assuming-lock form succeeds THROUGH the same
        // held lock, because it never tries to acquire it.
        let mut patch = Map::new();
        patch.insert("phase".into(), json!("planning"));
        let updated = ok(update_workflow_assuming_lock(root, "wf-1", patch));
        assert_eq!(updated.get("phase"), Some(&json!("planning")));
        assert_eq!(read_back(&workflow_state_path(root, "wf-1"))["phase"], json!("planning"));

        // Release, and the self-locking form works again — so the denial
        // above was the lock, not a broken record.
        drop(held);
        let mut patch = Map::new();
        patch.insert("phase".into(), json!("swarming"));
        assert_eq!(ok(update_workflow(root, "wf-1", patch)).get("phase"), Some(&json!("swarming")));
    }

    /// Oracle: "withWorkflowLock is a thin named wrapper: two ids run their
    /// bodies without either blocking the other".
    ///
    /// Node proves independence by interleaving two async bodies; the Rust
    /// wrapper is an RAII guard, so the same property is proved without a
    /// scheduler: with `workflow:wf-p` held, a DIFFERENT id is granted on its
    /// very first attempt while the SAME id is denied on its first attempt.
    #[test]
    fn workflow_locks_for_two_ids_never_block_each_other() {
        let tmp = tmp_root();
        let root = tmp.path();
        let held_p = ok(acquire_workflow_lock(root, "wf-p"));

        // Independent name: granted with zero retries, while wf-p is held.
        let held_q = lock::acquire_store_lock(root, "workflow:wf-q", 1)
            .unwrap_or_else(|b| panic!("a distinct id must not queue behind wf-p — {}", b.message()));
        // Control: the SAME name is refused on that same single attempt, so
        // the grant above is independence and not a disabled lock.
        let same = lock::acquire_store_lock(root, "workflow:wf-p", 1);
        assert!(same.is_err(), "workflow:wf-p must be denied while it is held");
        // A third, still-distinct id is likewise granted with both held.
        let held_r = lock::acquire_store_lock(root, "workflow:wf-r", 1)
            .unwrap_or_else(|b| panic!("a third distinct id must not queue — {}", b.message()));

        assert!(lock::lock_file_path(root, "workflow:wf-p").exists());
        assert!(lock::lock_file_path(root, "workflow:wf-q").exists());
        assert!(lock::lock_file_path(root, "workflow:wf-r").exists());
        drop(held_q);
        drop(held_r);
        drop(held_p);
        // Every guard released its own file and only its own.
        for id in ["wf-p", "wf-q", "wf-r"] {
            assert!(!lock::lock_file_path(root, &format!("workflow:{id}")).exists());
        }
        // …and wf-p is takeable again once released.
        ok(lock::acquire_store_lock(root, "workflow:wf-p", 1));
    }
