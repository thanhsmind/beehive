// `state advisor-ref record` / `state advisor-ref show` — the AO3/AO13
// advisor-consult anchors, staleness check, and the two verbs that record and
// read them.
//
// Provenance: lib/state.mjs `ADVISOR_PLAN_ABSENT_SENTINEL` / `advisorPlanPath`
// / `advisorRefAnchors` / `advisorRefStale` (ported verbatim — the staleness
// rule is AO13 and carries NO TTL, by design: a ref never ages out, it only
// goes stale when one of the four anchors moves), and bee.mjs
// `handleStateAdvisorRefRecord` / `handleStateAdvisorRefShow`.
//
// `record` follows the house mutation skeleton (workers.rs's
// run_scribing_run / run_compounding_run): keys_known -> go -> require_flags
// -> mutation_lane_selector -> resolve_mutation_lock_scope -> list_workflows
// -> acquire_mutation_locks -> resolve_mutation_target -> door check -> mutate
// -> write_through_projection -> finish. The anchors are stamped by the verb
// itself from the SELECTED record's feature (never caller-supplied).
//
// `show` is read-only, so it skips the lock/mutate/write-through steps, but
// it now shares the SAME target resolution (`mutation_lane_selector` +
// `resolve_mutation_target`) record uses — the deleted JS's `show` handler
// only ever honoured an explicit `--lane`, never a session-bound lane or
// `--no-lane`; this port widens it to the standard selector every other read
// in this file already gives a caller, per this cell's own spec.
//
// NOT wired into `try_native`'s routing table yet: that match arm lives in
// set_gate.rs, out of scope for this cell (agp-1) by explicit instruction —
// the routing entry lands with the Gate 3 precondition in agp-2. Until then
// these two verbs are reachable only in-process (this file's own tests), and
// the registry keeps advertising them as `unavailable` so `--help` and the
// registry<->dispatcher contract (tests/registry_dispatch.rs) stay honest
// about what this binary actually serves.
#![allow(unused_imports)]
// `run_advisor_ref_record` / `run_advisor_ref_show` have no caller yet outside
// this file's own tests (see the note above) — silence the dead-code warning
// rather than call them from nowhere just to appease the compiler.
#![allow(dead_code)]

use super::*;
use crate::fsutil::{
    append_jsonl, ensure_dir, read_json, warn_corrupt_json, write_json_atomic, ReadJson,
};
use crate::jsjson;
use crate::lock::{self, AcquireOnce, LockGuard};
use crate::textutil::truncate_chars_head;
use crate::verbs::decisions::active_decisions;
use crate::verbs::reservations::{
    date_parse_val, finish, iso_from_ms, jget, js_disp, js_disp_opt,
    js_numberify, js_trim, keys_known, now_iso, now_ms, parse_flags, prelude, truthy,
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
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
use std::time::Instant;

const EXAMPLE_ADVISOR_REF_RECORD: &str =
    "bee state advisor-ref record --advisor gpt-5.6-sol --digest-file consult.txt --json";

// ─── anchors + staleness (lib/state.mjs, AO3/AO13, Slice 4) ────────────────
// Zero precedent: no gate anywhere checked a precondition before this. The
// advisor_ref field records that a real advisor consult happened for the
// SELECTED (default or lane) record; Gate 3 refuses high-risk execution
// approval when the ref is missing or stale. Staleness is NEVER a TTL — AO13
// bans invented time numbers. Both helpers are pure reads and never throw on
// a missing artifact: a missing plan.md hashes to the absent sentinel, so the
// only failure mode is "stale", never a crash on the gate's hot path.

pub(crate) const ADVISOR_PLAN_ABSENT_SENTINEL: &str = "absent";

/// `path.join(root, 'docs', 'history', String(feature ?? ''), 'plan.md')`.
pub(crate) fn advisor_plan_path(root: &Path, feature: &str) -> PathBuf {
    root.join("docs").join("history").join(feature).join("plan.md")
}

/// advisorRefAnchors — the verb stamps these itself at record time; the
/// caller never supplies them. `feature` is the SELECTED record's feature (a
/// lane's own feature, not the default record's — checker M1), so the
/// plan.md hashed is THAT feature's plan.
pub(crate) fn advisor_ref_anchors(root: &Path, feature: &Value) -> Map<String, Value> {
    // `newest_decision_id`: the id of the newest ACTIVE decision, null on ANY
    // error (activeDecisions' own try/catch swallowed everything).
    let newest_decision_id = match active_decisions(root, false) {
        Ok(active) => active
            .first()
            .and_then(|e| jget(e, "id"))
            .cloned()
            .unwrap_or(Value::Null),
        Err(_) => Value::Null,
    };
    // `plan_sha256`: sha256 of docs/history/<feature>/plan.md, or the literal
    // absent sentinel when the file is missing OR unreadable for any reason
    // (existsSync + readFileSync's own try/catch, collapsed into one read).
    let feature_str = match feature {
        Value::Null => String::new(),
        v => js_disp(v),
    };
    let plan_sha256 = match std::fs::read(advisor_plan_path(root, &feature_str)) {
        Ok(bytes) => {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            format!("{:x}", hasher.finalize())
        }
        Err(_) => ADVISOR_PLAN_ABSENT_SENTINEL.to_string(),
    };
    let mut anchors = Map::new();
    // `feature ?? null` — Null in, Null out; anything else rides through
    // unchanged (never coerced to a string here — only the path build above
    // coerces).
    anchors.insert("feature".into(), feature.clone());
    anchors.insert("newest_decision_id".into(), newest_decision_id);
    anchors.insert("plan_sha256".into(), json!(plan_sha256));
    anchors
}

pub(crate) struct Staleness {
    pub(crate) stale: bool,
    pub(crate) reasons: Vec<String>,
}

/// advisorRefStale — AO13 verbatim: an advisor_ref is stale if ANY of — its
/// feature differs from state.feature; the newest active decision id changed
/// since the consult; sha256(plan.md) changed; or the ref predates the most
/// recent revocation of the execution gate. A malformed/missing ref
/// (non-object, array, or null) reads as missing — stale, never a throw.
/// There is NO TTL anywhere in this function — a ref does not age out.
pub(crate) fn advisor_ref_stale(
    root: &Path,
    raw_ref: Option<&Value>,
    state: &Map<String, Value>,
) -> Staleness {
    let ref_obj = match raw_ref {
        Some(Value::Object(m)) => m,
        _ => {
            return Staleness {
                stale: true,
                reasons: vec!["no advisor_ref recorded".to_string()],
            }
        }
    };
    let mut reasons = Vec::new();
    // `state && state.feature != null ? state.feature : null` — nullish, not
    // falsy: an empty-string feature stays "" here, never coerced to null.
    let feature = match state.get("feature") {
        Some(Value::Null) | None => Value::Null,
        Some(v) => v.clone(),
    };
    let anchors = advisor_ref_anchors(root, &feature);

    let ref_feature = ref_obj.get("feature");
    let anchors_feature = anchors.get("feature");
    if !opt_strict_eq(ref_feature, anchors_feature) {
        reasons.push(format!(
            "feature changed since the consult (ref \"{}\" ≠ current \"{}\")",
            js_disp_opt(ref_feature),
            js_disp_opt(anchors_feature)
        ));
    }

    let ref_ndi = ref_obj.get("newest_decision_id");
    let anchors_ndi = anchors.get("newest_decision_id");
    if !opt_strict_eq(ref_ndi, anchors_ndi) {
        reasons.push(format!(
            "a new decision was logged since the consult (ref \"{}\" ≠ current \"{}\")",
            js_disp_opt(ref_ndi),
            js_disp_opt(anchors_ndi)
        ));
    }

    let ref_plan = ref_obj.get("plan_sha256");
    let anchors_plan = anchors.get("plan_sha256");
    if !opt_strict_eq(ref_plan, anchors_plan) {
        reasons.push("plan.md changed since the consult (sha256 mismatch)".to_string());
    }

    // `state.gate_revoked_at ? state.gate_revoked_at.execution : undefined`,
    // then `if (revokedAt && (!ref.consulted_at || String(ref.consulted_at) <
    // String(revokedAt)))`.
    let revoked_at: Option<&Value> = match state.get("gate_revoked_at") {
        Some(v) if truthy(v) => jget(v, "execution"),
        _ => None,
    };
    if let Some(revoked_at_v) = revoked_at {
        if truthy(revoked_at_v) {
            let consulted_at = ref_obj.get("consulted_at");
            let consulted_falsy = !consulted_at.map(truthy).unwrap_or(false);
            let predates = consulted_falsy || js_disp_opt(consulted_at) < js_disp(revoked_at_v);
            if predates {
                // `ref.consulted_at ?? 'never'` — nullish, not falsy: an
                // empty-string consulted_at would display as "", not "never".
                let consulted_disp = match consulted_at {
                    None | Some(Value::Null) => "never".to_string(),
                    Some(v) => js_disp(v),
                };
                reasons.push(format!(
                    "the consult predates the most recent execution-gate revocation (consulted {consulted_disp}, revoked {})",
                    js_disp(revoked_at_v)
                ));
            }
        }
    }

    Staleness { stale: !reasons.is_empty(), reasons }
}

// ─── state advisor-ref record ───────────────────────────────────────────────
// hive law 12: the Gate 3 precondition needs a state field AND a CLI verb.
// The verb stamps the staleness anchors ITSELF — the caller supplies only the
// advisor identity and a digest for audit; anchors are never caller-supplied.

pub(crate) fn run_advisor_ref_record(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["advisor", "digest-file", "lane", "no-lane"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "no-lane") {
        return None;
    }
    let ctx = match go("state advisor-ref record", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = record_body(&ctx.root, &flags);
    finish(&ctx, out)
}

/// The root-explicit body of `run_advisor_ref_record`, split out from the
/// `go()`-wrapped entry point above so it can be exercised directly against a
/// scratch root — `go()` resolves its root from `std::env::current_dir()`,
/// process-global state a parallel test suite cannot safely mutate.
fn record_body(root: &Path, flags: &Flags) -> R2<Out> {
    let values = match require_flags(
        flags,
        &[("advisor", None), ("digest-file", None)],
        EXAMPLE_ADVISOR_REF_RECORD,
    ) {
        Ok(v) => v,
        Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
        Err(Err2::Ex) => return Err(Err2::Ex),
    };
    let (advisor, digest_file) = (values[0].clone(), values[1].clone());
    let (lane_feature, no_lane) = match mutation_lane_selector(flags, "advisor-ref record") {
        Ok(v) => v,
        Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
        Err(Err2::Ex) => return Err(Err2::Ex),
    };
    let scope = resolve_mutation_lock_scope(root, lane_feature.as_deref(), no_lane)?;
    let workflows = list_workflows(root)?;
    let locks = acquire_mutation_locks(root, &scope, &workflows)?;
    let mut target =
        resolve_mutation_target(root, lane_feature.as_deref(), "advisor-ref record", no_lane)?;
    let lane_note = target.lane_note();

    // door: refuse — zero writes — when there is no active feature to
    // anchor the consult to.
    let feature = target.record().get("feature").cloned().unwrap_or(Value::Null);
    let phase = target.record().get("phase").cloned().unwrap_or(Value::Null);
    let idle_or_done = matches!(&phase, Value::String(p) if p == "idle" || p == "compounding-complete");
    if !truthy(&feature) || idle_or_done {
        let phase_disp = match &phase {
            Value::Null => "idle".to_string(),
            v => js_disp(v),
        };
        let feature_disp = match &feature {
            Value::Null => "none".to_string(),
            v => js_disp(v),
        };
        return Ok(Out::Thrown(format!(
            "advisor-ref record: refused — no active feature to anchor the consult to (phase \"{phase_disp}\", feature \"{feature_disp}\"). FIX: start a feature and reach an in-flight phase before recording an advisor consult."
        )));
    }

    // door: refuse — zero writes — when the digest file cannot be read.
    let digest_head = match std::fs::read(&digest_file) {
        Ok(bytes) => {
            let text = String::from_utf8_lossy(&bytes).into_owned();
            truncate_chars_head(&text, 500)
        }
        Err(e) => {
            return Ok(Out::Thrown(format!(
                "advisor-ref record: could not read --digest-file \"{digest_file}\" ({}). FIX: pass the path to the captured advisor consult digest.",
                io_read_reason(Path::new(&digest_file), &e)
            )));
        }
    };

    // Anchors bound to the SELECTED record's feature (M1), stamped here.
    let anchors = advisor_ref_anchors(root, &feature);
    let anchors_feature = anchors.get("feature").cloned().unwrap_or(Value::Null);
    {
        let rec = target.record_mut();
        let mut adv = Map::new();
        adv.insert("consulted_at".into(), json!(now_iso()));
        adv.insert("feature".into(), anchors.get("feature").cloned().unwrap_or(Value::Null));
        adv.insert(
            "newest_decision_id".into(),
            anchors.get("newest_decision_id").cloned().unwrap_or(Value::Null),
        );
        adv.insert(
            "plan_sha256".into(),
            anchors.get("plan_sha256").cloned().unwrap_or(Value::Null),
        );
        adv.insert("advisor".into(), json!(advisor));
        adv.insert("digest_head".into(), json!(digest_head));
        rec.insert("advisor_ref".into(), Value::Object(adv));
    }
    let record = target.record().clone();
    write_through_projection(root, &target, &record, &[])?;
    drop(locks);

    let advisor_ref = record.get("advisor_ref").cloned().unwrap_or(Value::Null);
    let text = format!(
        "Recorded advisor_ref (advisor \"{advisor}\", feature \"{}\").{lane_note}",
        js_disp(&anchors_feature)
    );
    Ok(Out::Emit(advisor_ref, text, 0))
}

// ─── state advisor-ref show ─────────────────────────────────────────────────

pub(crate) fn run_advisor_ref_show(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["lane", "no-lane"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "no-lane") {
        return None;
    }
    let ctx = match go("state advisor-ref show", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = show_body(&ctx.root, &flags);
    finish(&ctx, out)
}

/// The root-explicit body of `run_advisor_ref_show` — see `record_body`'s doc
/// for why this is split out from the `go()`-wrapped entry point.
fn show_body(root: &Path, flags: &Flags) -> R2<Out> {
    let (lane_feature, no_lane) = match mutation_lane_selector(flags, "advisor-ref show") {
        Ok(v) => v,
        Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
        Err(Err2::Ex) => return Err(Err2::Ex),
    };
    let target =
        resolve_mutation_target(root, lane_feature.as_deref(), "advisor-ref show", no_lane)?;
    let lane_note = target.lane_note();
    let state = target.record();
    let raw = state.get("advisor_ref");
    // `raw && typeof raw === 'object' && !Array.isArray(raw) ? raw : null`
    let ref_obj = match raw {
        Some(Value::Object(m)) => Some(m),
        _ => None,
    };
    let Some(ref_obj) = ref_obj else {
        return Ok(Out::Emit(Value::Null, format!("No advisor_ref recorded.{lane_note}"), 0));
    };
    let staleness = advisor_ref_stale(root, raw, state);
    let advisor_disp = js_disp_opt(ref_obj.get("advisor"));
    let feature_disp = js_disp_opt(ref_obj.get("feature"));
    let consulted_disp = js_disp_opt(ref_obj.get("consulted_at"));
    let reasons_note = if staleness.reasons.is_empty() {
        String::new()
    } else {
        format!(" ({})", staleness.reasons.join("; "))
    };
    let text = format!(
        "advisor=\"{advisor_disp}\" feature=\"{feature_disp}\" consulted_at={consulted_disp} stale={}{reasons_note}{lane_note}",
        staleness.stale
    );
    let result = json!({
        "advisor_ref": Value::Object(ref_obj.clone()),
        "stale": staleness.stale,
        "reasons": staleness.reasons.clone(),
    });
    Ok(Out::Emit(result, text, 0))
}

// ─── tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write_state_file(root: &Path, content: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("state.json"), content).unwrap();
    }

    fn flags(args: &[&str]) -> Flags {
        parse_flags(args).expect("well-formed fixture argv").0
    }

    fn plan_file(root: &Path, feature: &str, body: &str) -> String {
        let dir = root.join("docs").join("history").join(feature);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("plan.md"), body).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(body.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    // ── advisor_ref_anchors ─────────────────────────────────────────────

    #[test]
    fn anchors_hash_a_present_plan_and_sentinel_an_absent_one() {
        let tmp = tmp_root();
        let root = tmp.path();
        let want = plan_file(root, "feat-a", "the plan body");
        let anchors = advisor_ref_anchors(root, &json!("feat-a"));
        assert_eq!(anchors["plan_sha256"], json!(want));

        let anchors_missing = advisor_ref_anchors(root, &json!("feat-b"));
        assert_eq!(anchors_missing["plan_sha256"], json!(ADVISOR_PLAN_ABSENT_SENTINEL));
    }

    #[test]
    fn anchors_newest_decision_id_is_null_absent_a_decisions_store() {
        let tmp = tmp_root();
        let anchors = advisor_ref_anchors(tmp.path(), &json!("feat-a"));
        assert_eq!(anchors["newest_decision_id"], Value::Null);
    }

    // ── advisor_ref_stale: each reason independently ────────────────────

    fn fresh_state(root: &Path, feature: &str) -> Map<String, Value> {
        let mut state = Map::new();
        state.insert("feature".into(), json!(feature));
        state
    }

    fn fresh_ref(root: &Path, feature: &str) -> Map<String, Value> {
        let anchors = advisor_ref_anchors(root, &json!(feature));
        let mut r = Map::new();
        r.insert("consulted_at".into(), json!("2026-01-01T00:00:00.000Z"));
        r.insert("feature".into(), anchors["feature"].clone());
        r.insert("newest_decision_id".into(), anchors["newest_decision_id"].clone());
        r.insert("plan_sha256".into(), anchors["plan_sha256"].clone());
        r.insert("advisor".into(), json!("gpt-5.6-sol"));
        r.insert("digest_head".into(), json!("digest"));
        r
    }

    #[test]
    fn a_freshly_recorded_ref_reads_as_fresh() {
        let tmp = tmp_root();
        let root = tmp.path();
        plan_file(root, "feat-a", "v1");
        let state = fresh_state(root, "feat-a");
        let r = fresh_ref(root, "feat-a");
        let s = advisor_ref_stale(root, Some(&Value::Object(r)), &state);
        assert!(!s.stale, "{:?}", s.reasons);
        assert!(s.reasons.is_empty());
    }

    #[test]
    fn stale_reason_feature_changed() {
        let tmp = tmp_root();
        let root = tmp.path();
        let r = fresh_ref(root, "feat-a");
        let state = fresh_state(root, "feat-b"); // moved feature
        let s = advisor_ref_stale(root, Some(&Value::Object(r)), &state);
        assert!(s.stale);
        assert_eq!(s.reasons.len(), 1, "{:?}", s.reasons);
        assert!(s.reasons[0].contains("feature changed since the consult"), "{:?}", s.reasons);
    }

    #[test]
    fn stale_reason_new_decision_logged() {
        let tmp = tmp_root();
        let root = tmp.path();
        let r = fresh_ref(root, "feat-a");
        // Append a decide event AFTER the ref was computed — the active
        // decisions store now differs from what the ref captured.
        let decisions_dir = root.join(".bee");
        std::fs::create_dir_all(&decisions_dir).unwrap();
        std::fs::write(
            decisions_dir.join("decisions.jsonl"),
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-01-02T00:00:00.000Z\",\"decision\":\"x\",\"rationale\":\"y\"}\n",
        )
        .unwrap();
        let state = fresh_state(root, "feat-a");
        let s = advisor_ref_stale(root, Some(&Value::Object(r)), &state);
        assert!(s.stale);
        assert_eq!(s.reasons.len(), 1, "{:?}", s.reasons);
        assert!(
            s.reasons[0].contains("a new decision was logged since the consult"),
            "{:?}",
            s.reasons
        );
    }

    #[test]
    fn stale_reason_plan_changed() {
        let tmp = tmp_root();
        let root = tmp.path();
        plan_file(root, "feat-a", "v1");
        let r = fresh_ref(root, "feat-a");
        // Mutate plan.md AFTER the ref captured its hash.
        plan_file(root, "feat-a", "v2");
        let state = fresh_state(root, "feat-a");
        let s = advisor_ref_stale(root, Some(&Value::Object(r)), &state);
        assert!(s.stale);
        assert_eq!(s.reasons.len(), 1, "{:?}", s.reasons);
        assert!(s.reasons[0].contains("plan.md changed since the consult"), "{:?}", s.reasons);
    }

    #[test]
    fn stale_reason_predates_execution_gate_revocation() {
        let tmp = tmp_root();
        let root = tmp.path();
        let r = fresh_ref(root, "feat-a"); // consulted_at = 2026-01-01
        let mut state = fresh_state(root, "feat-a");
        let mut revoked = Map::new();
        revoked.insert("execution".into(), json!("2026-01-02T00:00:00.000Z"));
        state.insert("gate_revoked_at".into(), Value::Object(revoked));
        let s = advisor_ref_stale(root, Some(&Value::Object(r)), &state);
        assert!(s.stale);
        assert_eq!(s.reasons.len(), 1, "{:?}", s.reasons);
        assert!(
            s.reasons[0].contains("predates the most recent execution-gate revocation"),
            "{:?}",
            s.reasons
        );
    }

    #[test]
    fn a_revocation_before_the_consult_does_not_stale_it() {
        let tmp = tmp_root();
        let root = tmp.path();
        let r = fresh_ref(root, "feat-a"); // consulted_at = 2026-01-01
        let mut state = fresh_state(root, "feat-a");
        let mut revoked = Map::new();
        revoked.insert("execution".into(), json!("2025-01-01T00:00:00.000Z"));
        state.insert("gate_revoked_at".into(), Value::Object(revoked));
        let s = advisor_ref_stale(root, Some(&Value::Object(r)), &state);
        assert!(!s.stale, "{:?}", s.reasons);
    }

    #[test]
    fn a_malformed_ref_reads_as_stale_with_one_reason_never_a_crash() {
        let tmp = tmp_root();
        let root = tmp.path();
        let state = fresh_state(root, "feat-a");
        for bad in [
            Value::Array(vec![json!(1)]),
            Value::String("nope".into()),
            Value::Bool(true),
            Value::Number(1.into()),
            Value::Null,
        ] {
            let s = advisor_ref_stale(root, Some(&bad), &state);
            assert!(s.stale, "{bad:?}");
            assert_eq!(s.reasons, vec!["no advisor_ref recorded".to_string()], "{bad:?}");
        }
        // Absent entirely (None) reads the same way.
        let s = advisor_ref_stale(root, None, &state);
        assert!(s.stale);
        assert_eq!(s.reasons, vec!["no advisor_ref recorded".to_string()]);
    }

    #[test]
    fn no_ttl_a_ref_does_not_go_stale_merely_by_aging() {
        let tmp = tmp_root();
        let root = tmp.path();
        plan_file(root, "feat-a", "v1");
        let mut r = fresh_ref(root, "feat-a");
        // An implausibly old consulted_at, nothing else moved: still fresh.
        r.insert("consulted_at".into(), json!("1970-01-01T00:00:00.000Z"));
        let state = fresh_state(root, "feat-a");
        let s = advisor_ref_stale(root, Some(&Value::Object(r)), &state);
        assert!(!s.stale, "an old timestamp alone must never stale a ref: {:?}", s.reasons);
    }

    // ── run_advisor_ref_record ──────────────────────────────────────────

    fn digest_file(dir: &Path, body: &str) -> String {
        let file = dir.join("digest.txt");
        std::fs::write(&file, body).unwrap();
        file.display().to_string()
    }

    /// Unwraps `record_body`/`show_body`'s outcome down to `(is_thrown, text)`
    /// — a thrown Error and an emitted result are both legitimate, non-Exotic
    /// outcomes here; only `Err2::Ex` (an Exotic probe) panics, since none of
    /// these fixtures are exotic input.
    fn unwrap_out(out: R2<Out>) -> (bool, String) {
        match out {
            Ok(Out::Emit(_, text, _)) => (false, text),
            Ok(Out::Thrown(msg)) => (true, msg),
            Err(_) => panic!("unexpected Exotic result"),
        }
    }

    #[test]
    fn record_refuses_with_zero_writes_when_no_feature_is_active() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_state_file(root, r#"{"schema_version":"1.0","phase":"idle","feature":null}"#);
        let before = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        let digest = digest_file(root, "consult notes");
        let out = record_body(
            root,
            &flags(&["--advisor", "gpt-5.6-sol", "--digest-file", &digest]),
        );
        let (thrown, msg) = unwrap_out(out);
        assert!(thrown, "{msg}");
        assert!(msg.contains("no active feature to anchor the consult to"), "{msg}");
        let after = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        assert_eq!(before, after, "a refused record must write nothing");
    }

    #[test]
    fn record_refuses_with_a_typed_error_when_the_digest_file_is_unreadable() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_state_file(
            root,
            r#"{"schema_version":"1.0","phase":"swarming","feature":"feat-a"}"#,
        );
        let before = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        let missing = root.join("no-such-digest.txt").display().to_string();
        let out = record_body(
            root,
            &flags(&["--advisor", "gpt-5.6-sol", "--digest-file", &missing]),
        );
        let (thrown, msg) = unwrap_out(out);
        assert!(thrown, "{msg}");
        assert!(msg.contains("could not read --digest-file"), "{msg}");
        let after = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        assert_eq!(before, after, "a refused record must write nothing");
    }

    #[test]
    fn record_stamps_anchors_and_show_then_reports_fresh() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_state_file(
            root,
            r#"{"schema_version":"1.0","phase":"swarming","feature":"feat-a"}"#,
        );
        let digest = digest_file(root, "consult notes");
        let out = record_body(
            root,
            &flags(&["--advisor", "gpt-5.6-sol", "--digest-file", &digest]),
        );
        let (thrown, msg) = unwrap_out(out);
        assert!(!thrown, "{msg}");

        let show_out = show_body(root, &flags(&[]));
        let (show_thrown, show_msg) = unwrap_out(show_out);
        assert!(!show_thrown, "{show_msg}");
        assert!(show_msg.contains("stale=false"), "{show_msg}");

        let state: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee/state.json")).unwrap(),
        )
        .unwrap();
        let advisor_ref = &state["advisor_ref"];
        assert_eq!(advisor_ref["advisor"], json!("gpt-5.6-sol"));
        assert_eq!(advisor_ref["feature"], json!("feat-a"));
        assert_eq!(advisor_ref["digest_head"], json!("consult notes"));
        assert!(advisor_ref["consulted_at"].is_string());

        let s = advisor_ref_stale(root, Some(advisor_ref), state.as_object().unwrap());
        assert!(!s.stale, "just-recorded ref must read fresh: {:?}", s.reasons);
    }

    #[test]
    fn record_honours_an_explicit_lane_and_leaves_the_default_record_untouched() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_state_file(root, r#"{"schema_version":"1.0","phase":"idle","feature":null}"#);
        std::fs::create_dir_all(root.join(".bee").join("lanes")).unwrap();
        std::fs::write(
            root.join(".bee").join("lanes").join("lane-feat.json"),
            r#"{"schema_version":"1.0","feature":"lane-feat","phase":"swarming","mode":null,"approved_gates":{},"summary":"","next_action":""}"#,
        )
        .unwrap();
        let digest = digest_file(root, "lane consult");
        let default_before = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        let out = record_body(
            root,
            &flags(&[
                "--advisor",
                "gpt-5.6-sol",
                "--digest-file",
                &digest,
                "--lane",
                "lane-feat",
            ]),
        );
        let (thrown, msg) = unwrap_out(out);
        assert!(!thrown, "{msg}");

        // Default record: byte-untouched.
        let default_after = std::fs::read_to_string(root.join(".bee/state.json")).unwrap();
        assert_eq!(default_before, default_after);

        // Lane record: carries the freshly stamped ref.
        let lane: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee/lanes/lane-feat.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(lane["advisor_ref"]["feature"], json!("lane-feat"));
    }

    // ── run_advisor_ref_show ────────────────────────────────────────────

    #[test]
    fn show_reports_no_ref_recorded_when_state_carries_none() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_state_file(root, r#"{"schema_version":"1.0","phase":"idle","feature":null}"#);
        let (thrown, msg) = unwrap_out(show_body(root, &flags(&[])));
        assert!(!thrown, "{msg}");
        assert!(msg.starts_with("No advisor_ref recorded."), "{msg}");
    }

    #[test]
    fn show_targets_the_explicit_lane_over_the_default_record() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_state_file(root, r#"{"schema_version":"1.0","phase":"idle","feature":null}"#);
        std::fs::create_dir_all(root.join(".bee").join("lanes")).unwrap();
        std::fs::write(
            root.join(".bee").join("lanes").join("lane-feat.json"),
            r#"{"schema_version":"1.0","feature":"lane-feat","phase":"swarming","mode":null,"approved_gates":{},"summary":"","next_action":"","advisor_ref":{"consulted_at":"2026-01-01T00:00:00.000Z","feature":"lane-feat","newest_decision_id":null,"plan_sha256":"absent","advisor":"gpt-5.6-sol","digest_head":"d"}}"#,
        )
        .unwrap();
        let (thrown, msg) = unwrap_out(show_body(root, &flags(&["--lane", "lane-feat"])));
        assert!(!thrown, "{msg}");
        assert!(msg.contains("feature=\"lane-feat\""), "{msg}");
        assert!(msg.contains("(lane \"lane-feat\")"), "{msg}");
    }

    #[test]
    fn show_refuses_a_missing_lane() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_state_file(root, r#"{"schema_version":"1.0","phase":"idle","feature":null}"#);
        match show_body(root, &flags(&["--lane", "ghost"])) {
            Err(Err2::Msg(m)) => assert!(m.contains("lane \"ghost\" does not exist"), "{m}"),
            Ok(_) => panic!("expected a lane-missing refusal, got a successful result"),
            Err(Err2::Ex) => panic!("expected a lane-missing refusal, got an Exotic result"),
        }
    }

    #[test]
    fn record_no_lane_forces_the_default_record_even_with_a_session_bound_lane() {
        let tmp = tmp_root();
        let root = tmp.path();
        write_state_file(
            root,
            r#"{"schema_version":"1.0","phase":"swarming","feature":"feat-a"}"#,
        );
        let digest = digest_file(root, "consult notes");
        let out = record_body(
            root,
            &flags(&[
                "--advisor",
                "gpt-5.6-sol",
                "--digest-file",
                &digest,
                "--no-lane",
                "--lane",
                "feat-a",
            ]),
        );
        // `--no-lane` combined with `--lane` is a malformed call — refused,
        // never silently resolved one way or the other.
        let (thrown, msg) = unwrap_out(out);
        assert!(thrown, "{msg}");
        assert!(msg.contains("cannot be combined with --lane"), "{msg}");
    }
}
