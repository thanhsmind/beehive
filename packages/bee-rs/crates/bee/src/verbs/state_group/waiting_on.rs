// `state waiting-on set` / `state waiting-on clear` — the agent-facing verbs
// awaiting-human ah-4 exists to add.
//
// WHY THIS FILE EXISTS. ah-1 built the mark (`build_waiting_on`,
// `waiting_on_is_live`, the workflow-record and default-state setters) and
// folded it into `run_state`'s derivation; ah-2 gave it three live ways to
// end (the `UserPromptSubmit` hook, an explicit clear, and D4's dual-condition
// stale reap) but — by its own recorded deviation — wired none of it to a CLI
// verb, because no cell in the feature's shape declared the routing files
// (this one, `mod.rs`, `../../generated/registry_payload.json`) that wiring
// would touch. Nothing in ah-1/ah-2 is reimplemented here: this module is
// ONLY flag parsing, D3's target resolution, and the two calls into the
// store functions those cells already built and tested.
//
// TARGET RESOLUTION (D3). "Session-scoped first, feature-scoped only when a
// feature happens to be live" is exactly the precedence
// `resolve_handoff_workflow_id` (sessions.rs) already established for this
// crate's other feature-optional verb: explicit `--lane` names a feature
// that MUST have a live workflow (refuses if not, reusing
// `lane_missing_refusal`/`bound_lane_missing_refusal` so the wording matches
// every other lane-routed verb); `--no-lane` forces the default record
// outright; otherwise the calling session's bound lane, else the default
// record's own live feature; otherwise (D3's named gap) no feature resolves
// at all and the caller targets the default `.bee/state.json` record
// directly. `resolve_waiting_on_target` mirrors that shape rather than
// reusing the handoff resolver verbatim, because the handoff resolver's own
// refusal text names "state handoff" — a caller here needs its OWN verb name
// in the message.
//
// PROJECTION SYNC. ah-1's setters and ah-2's clearers write ONLY the
// workflow record (self-locking `update_workflow`) — never the lane/default
// record's own copy. "Record wins, projection is rebuilt from it": a reader
// of `bee status --json` (status_full/build.rs, ah-3) reads `waiting_on`
// straight off `.bee/state.json`, which only carries a workflow's fields
// once `apply_workflow_d1_fields` runs during a projection rebuild. Every
// feature-scoped set/clear here therefore rebuilds that feature's lane
// projection AND the default state projection (the second is a safe no-op
// whenever the default record's own `feature` does not match — see
// `rebuild_state_projection_reporting`'s own branching) so the mark is
// visible immediately, not just after some unrelated later mutation.

#![allow(unused_imports)]

use super::*;
use crate::fsutil::{
    append_jsonl, ensure_dir, read_json, warn_corrupt_json, write_json_atomic, ReadJson,
};
use crate::jsjson;
use crate::lock::{self, AcquireOnce, LockGuard};
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
    write_mailbox_handoff, MailboxAdopt, clear_default_state_waiting_on, clear_workflow_waiting_on,
    set_workflow_waiting_on, waiting_on_is_live,
};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
use std::time::Instant;

// ─── target resolution (D3) ─────────────────────────────────────────────────

/// D3's precedence, resolved to a `(workflow_id, feature)` pair. `None` is
/// D3's own case: no feature is live, so the caller targets the default
/// `.bee/state.json` record directly.
pub(crate) fn resolve_waiting_on_target(
    root: &Path,
    verb: &str,
    lane_feature: Option<&str>,
    no_lane: bool,
    session_id_flag: Option<&str>,
) -> Result<Option<(String, String)>, Err2> {
    let workflows = list_workflows(root)?;
    if workflows.is_empty() {
        return Ok(None); // C1: no workflow records anywhere.
    }
    // R3 (mcl-1): `state waiting-on clear`'s own help text already promises
    // "a no-op, never a refusal" once the named feature's workflow is
    // CLOSED — a closed workflow still HAS a record, it is simply no longer
    // LIVE, so the clear verb widens both lane-naming branches below to fall
    // back to the newest record for that feature regardless of status.
    // `state waiting-on set` never widens (you cannot mark a closed lane as
    // waiting): its refusal stays exactly `find_live_workflow`-only, byte-
    // identical wording
    // (`waiting_on_target_refuses_an_explicit_lane_naming_no_live_workflow`,
    // tests.rs). Branching on the literal verb string mirrors how every
    // refusal message here already embeds `verb` for its own text — the two
    // call sites (`run_waiting_on_set`/`run_waiting_on_clear`) pass their own
    // fixed strings, never a caller-supplied value.
    let clear_verb = verb == "state waiting-on clear";
    // This verb targets the WORKFLOW record directly (never a `.bee/lanes/*`
    // file), so its own refusal names that store rather than reusing
    // `lane_missing_refusal`/`bound_lane_missing_refusal` — those two are
    // phrased for `resolve_mutation_target`'s lane-FILE check and would point
    // a caller at the wrong path. Wording mirrors
    // `resolve_handoff_workflow_id`'s own live-workflow refusals
    // (sessions.rs), the closest existing precedent for a verb resolving a
    // live workflow rather than a lane record.
    if let Some(f) = lane_feature {
        if let Some(wf) = find_live_workflow(&workflows, f) {
            return Ok(Some((wf_id(wf), f.to_string())));
        }
        if clear_verb {
            if let Some(wf) = newest_workflow_for_feature(&workflows, f)? {
                return Ok(Some((wf_id(&wf), f.to_string())));
            }
            // R3: narrowed — the "and status !== closed" clause is dropped
            // because this refusal now fires only when NO record at all
            // (live or closed) names the feature.
            return Err(Err2::Msg(format!(
                "{verb}: refused \u{2014} --lane \"{f}\" names no live workflow (no .bee/runtime/workflows/*/state.json with feature \"{f}\"). FIX: start it first (\"state start-feature --feature {f} --as-lane\"), or omit --lane."
            )));
        }
        return Err(Err2::Msg(format!(
            "{verb}: refused \u{2014} --lane \"{f}\" names no live workflow (no .bee/runtime/workflows/*/state.json with feature \"{f}\" and status !== closed). FIX: start it first (\"state start-feature --feature {f} --as-lane\"), or omit --lane."
        )));
    }
    if no_lane {
        return Ok(None);
    }
    let (sid, bound) = session_binding(session_id_flag, root)?;
    if let Some(bound) = bound {
        if let Some(wf) = find_live_workflow(&workflows, &bound) {
            return Ok(Some((wf_id(wf), bound)));
        }
        if clear_verb {
            if let Some(wf) = newest_workflow_for_feature(&workflows, &bound)? {
                return Ok(Some((wf_id(&wf), bound)));
            }
        }
        return Err(Err2::Msg(format!(
            "{verb}: refused \u{2014} calling session \"{}\" is bound to lane \"{bound}\" but no live workflow names it. FIX: start the lane, unbind the session, or pass --no-lane to target the default record explicitly.",
            sid_disp(&sid)
        )));
    }
    let default_record = read_state_strict(root)?;
    if let Some(v) = default_record.get("feature") {
        if truthy(v) {
            let f = js_disp(v);
            if let Some(wf) = find_live_workflow(&workflows, &f) {
                return Ok(Some((wf_id(wf), f)));
            }
        }
    }
    Ok(None) // nothing resolves — D3's session-scoped default-record case
}

/// R3 (mcl-1): the CLEAR verb's own widened fallback — the newest workflow
/// record naming `feature`, regardless of status (`workflows_list_sort`'s
/// own newest-`created_at`-first, stable order), so a closed workflow's
/// stale mark still has a clear path. Only the clear-verb branches in
/// `resolve_waiting_on_target` above ever call this; the setter keeps
/// refusing through `find_live_workflow` alone (you cannot mark a closed
/// lane as waiting).
fn newest_workflow_for_feature(
    workflows: &[Map<String, Value>],
    feature: &str,
) -> Result<Option<Map<String, Value>>, Err2> {
    let want = Value::String(feature.to_string());
    let mut matches: Vec<Map<String, Value>> = workflows
        .iter()
        .filter(|w| w.get("feature").unwrap_or(&Value::Null) == &want)
        .cloned()
        .collect();
    if matches.is_empty() {
        return Ok(None);
    }
    workflows_list_sort(&mut matches)?;
    Ok(matches.into_iter().next())
}

/// R4 (mcl-1): the lane-file counterpart of `clear_default_state_waiting_on`'s
/// own pair-clearing (decision f9fd9d46) — called from `run_waiting_on_clear`
/// only when `rebuild_lane_projection_reporting` just answered
/// non-authoritative (R3's own closed-workflow fallback, where the live-only
/// projection sync is correctly a no-op and the `.bee/lanes/<feature>.json`
/// copy is left exactly as stuck as before this fix). Guarded the identical
/// way f9fd9d46 already established: `run_state` is nulled ONLY on the
/// literal "awaiting-approval" value, so a record-derived "done"/"shaping"
/// (or any other lane whose run_state genuinely still means something else)
/// is never clobbered. No lane file at all, or a lane copy that already
/// reads clear on both fields, is itself a no-op — no lock taken, no write.
/// Factored here rather than inlined so cells mcl-2 (worktree merge) and
/// mcl-3 (close) — landing next — can call it too.
pub(crate) fn clear_lane_waiting_on_pair(root: &Path, feature: &str) -> Result<(), Err2> {
    let Some(mut lane) = read_lane_strict(root, feature)? else {
        return Ok(());
    };
    let waiting_on_live = waiting_on_is_live(lane.get("waiting_on"));
    let run_state_stuck = lane.get("run_state") == Some(&json!("awaiting-approval"));
    if !waiting_on_live && !run_state_stuck {
        return Ok(()); // true no-op: the lane copy already reads clear
    }
    lane.insert("waiting_on".into(), Value::Null);
    if run_state_stuck {
        lane.insert("run_state".into(), Value::Null);
    }
    write_lane(root, &lane)
}

/// The setter's own refusal when no session resolves at all — distinct from
/// `build_waiting_on`'s "session is required" (that fires on an EMPTY string
/// the caller passed through; this fires when nothing could be resolved to
/// pass through in the first place).
fn no_session_refusal(verb: &str) -> String {
    format!(
        "{verb}: refused \u{2014} no session resolves for the mark's owning session (D4's stale-expiry reap reclaims against it). FIX: pass --session-id explicitly, set BEE_SESSION_ID/CLAUDE_CODE_SESSION_ID, or ensure exactly one live session record exists."
    )
}

// ─── state waiting-on set ───────────────────────────────────────────────────

pub(crate) fn run_waiting_on_set(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["kind", "subject", "session-id", "lane", "no-lane"]) {
        return None;
    }
    let ctx = match go("state waiting-on set", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let kind = match require_flag(&flags, "kind") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let subject = match require_flag(&flags, "subject") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let (lane_feature, no_lane) = match mutation_lane_selector(&flags, "state waiting-on set") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let session_flag = flag_value(&flags, "session-id");
        let session = match resolve_session_id(session_flag.as_deref(), &ctx.root)? {
            Some(s) => s,
            None => return Ok(Out::Thrown(no_session_refusal("state waiting-on set"))),
        };
        let target = resolve_waiting_on_target(
            &ctx.root,
            "state waiting-on set",
            lane_feature.as_deref(),
            no_lane,
            session_flag.as_deref(),
        )?;
        match target {
            Some((id, feature)) => {
                let record = set_workflow_waiting_on(&ctx.root, &id, &kind, &subject, &session)?;
                rebuild_lane_projection(&ctx.root, &feature)?;
                rebuild_state_projection(&ctx.root)?;
                Ok(Out::Emit(
                    Value::Object(record),
                    format!(
                        "Marked waiting on \"{subject}\" (kind \"{kind}\", feature \"{feature}\")."
                    ),
                    0,
                ))
            }
            None => {
                let record = set_default_state_waiting_on(&ctx.root, &kind, &subject, &session)?;
                Ok(Out::Emit(
                    Value::Object(record),
                    format!(
                        "Marked waiting on \"{subject}\" (kind \"{kind}\", no active feature)."
                    ),
                    0,
                ))
            }
        }
    })();
    finish(&ctx, out)
}

// ─── state waiting-on clear ─────────────────────────────────────────────────

pub(crate) fn run_waiting_on_clear(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["session-id", "lane", "no-lane"]) {
        return None;
    }
    let ctx = match go("state waiting-on clear", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let (lane_feature, no_lane) = match mutation_lane_selector(&flags, "state waiting-on clear") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let session_flag = flag_value(&flags, "session-id");
        let target = resolve_waiting_on_target(
            &ctx.root,
            "state waiting-on clear",
            lane_feature.as_deref(),
            no_lane,
            session_flag.as_deref(),
        )?;
        match target {
            Some((id, feature)) => {
                let record = clear_workflow_waiting_on(&ctx.root, &id)?;
                // R4 (mcl-1): `rebuild_lane_projection` only writes when the
                // resolved workflow is still LIVE (projections.rs's own
                // `find_live_workflow` gate) — correctly a no-op for R3's
                // closed-workflow fallback. `clear_lane_waiting_on_pair`
                // covers exactly that gap, directly on the lane file, so a
                // closed-workflow clear still leaves `waiting_on`+`run_state`
                // null on the copy an operator actually reads.
                let proj = rebuild_lane_projection_reporting(&ctx.root, &feature)?;
                if !proj.authoritative {
                    clear_lane_waiting_on_pair(&ctx.root, &feature)?;
                }
                rebuild_state_projection(&ctx.root)?;
                Ok(Out::Emit(
                    Value::Object(record),
                    format!("Cleared any live wait (feature \"{feature}\")."),
                    0,
                ))
            }
            None => {
                let record = clear_default_state_waiting_on(&ctx.root)?;
                Ok(Out::Emit(
                    Value::Object(record),
                    "Cleared any live wait (no active feature).".to_string(),
                    0,
                ))
            }
        }
    })();
    finish(&ctx, out)
}
