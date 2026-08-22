// `state lanes`, `state session …` and `state handoff …`
//
// Split out of the single 6.1k-line verbs/state_group.rs. Code unchanged; only module placement and item visibility moved.
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
    write_mailbox_handoff, MailboxAdopt,
};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
use std::time::Instant;

// ─── state lanes / session list ────────────────────────────────────────────

pub(crate) fn run_lanes(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match go("state lanes", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let lanes = list_lanes(&ctx.root)?;
        let sessions = list_session_records(&ctx.root)?;
        // boundBy: lane feature -> bound session ids (session.lane, string+truthy).
        let mut bound_by: Vec<(String, Vec<Value>)> = Vec::new();
        for session in &sessions {
            let Some(Value::String(lane)) = session.get("lane") else { continue };
            if lane.is_empty() {
                continue;
            }
            let id = session.get("id").cloned().unwrap_or(Value::Null);
            match bound_by.iter_mut().find(|(k, _)| k == lane) {
                Some((_, ids)) => ids.push(id),
                None => bound_by.push((lane.clone(), vec![id])),
            }
        }
        let mut rows: Vec<Value> = Vec::new();
        let mut lines: Vec<String> = Vec::new();
        for lane in &lanes {
            let feature = match lane.get("feature") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            let bound: Vec<Value> = bound_by
                .iter()
                .find(|(k, _)| *k == feature)
                .map(|(_, ids)| ids.clone())
                .unwrap_or_default();
            let mut row = lane.clone();
            row.insert("bound_sessions".into(), Value::Array(bound.clone()));
            let gates_obj = lane.get("approved_gates");
            let gates = GATE_NAMES
                .iter()
                .map(|g| {
                    let approved = gates_obj
                        .and_then(|v| jget(v, g))
                        .map(truthy)
                        .unwrap_or(false);
                    format!("{g}={}", if approved { "approved" } else { "pending" })
                })
                .collect::<Vec<_>>()
                .join(" ");
            let bindings_note = if bound.is_empty() {
                String::new()
            } else {
                format!(
                    " sessions={}",
                    bound.iter().map(js_disp).collect::<Vec<_>>().join(",")
                )
            };
            lines.push(format!(
                "{} [{}] {gates}{bindings_note}",
                js_disp(lane.get("feature").unwrap_or(&Value::Null)),
                js_disp(lane.get("phase").unwrap_or(&Value::Null)),
            ));
            rows.push(Value::Object(row));
        }
        let text = if rows.is_empty() {
            "No lane records.".to_string()
        } else {
            lines.join("\n")
        };
        Ok(Out::Emit(Value::Array(rows), text, 0))
    })();
    finish(&ctx, out)
}

/// The window an `activity.at` stamp stays "live" for (D4, decision
/// 2d4e3900). It is NOT a heartbeat constant: the heartbeat answers "is this
/// session's process still around", this answers "did the agent do anything
/// just now", and the two ages are read independently.
pub(crate) const SESSION_SIGNAL_WINDOW_SECONDS: f64 = 90.0;

/// D4 (decision 2d4e3900) — read-time liveness for one session record.
///
/// - `None` — the session is over (`status` `"dead"` or `"closed"`); there is
///   nothing left to be live about, so callers emit no signal rather than a
///   misleading `no_signal`.
/// - `Some("live")` — the activity hook stamped `activity.at` inside the
///   window.
/// - `Some("no_signal")` — no `activity` object at all, or its `at` is at
///   least `SESSION_SIGNAL_WINDOW_SECONDS` old. An `at` that will not parse
///   reads the same way: bee's own hook writes that field as an ISO-8601
///   stamp, so an unreadable one is a damaged record, never a live one.
///
/// Pure and DERIVED: every caller computes it at read time from the record it
/// already holds, and no writer ever persists the result — a stored `signal`
/// would be a stale claim about the present.
pub(crate) fn session_signal(record: &Map<String, Value>, now_ms_v: f64) -> Option<&'static str> {
    if matches!(record.get("status"), Some(Value::String(s)) if s == "dead" || s == "closed") {
        return None;
    }
    let Some(Value::Object(activity)) = record.get("activity") else {
        return Some("no_signal");
    };
    // date_parse_val: Ok(None) = NaN (absent/unparseable), Err = a string only
    // V8's legacy parser would take. Both are "not a stamp we can trust".
    let Some(at) = date_parse_val(activity.get("at")).ok().flatten() else {
        return Some("no_signal");
    };
    if !at.is_finite() || at + SESSION_SIGNAL_WINDOW_SECONDS * 1000.0 <= now_ms_v {
        return Some("no_signal");
    }
    Some("live")
}

pub(crate) fn run_session_list(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match go("state session list", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let mut sessions = list_session_records(&ctx.root)?;
        // D4: `signal` is DERIVED here, once, against a single `now` so every
        // row in one listing is judged against the same instant — and it is
        // added to the OUTGOING copy only. Nothing on this path writes a
        // session file, so the field can never reach disk.
        let now = now_ms();
        for session in sessions.iter_mut() {
            let signal = match session_signal(session, now) {
                Some(s) => json!(s),
                None => Value::Null,
            };
            session.insert("signal".into(), signal);
        }
        let lines: Vec<String> = sessions
            .iter()
            .map(|s| {
                let lane_note = match s.get("lane") {
                    Some(Value::String(l)) if !l.is_empty() => format!("-> lane \"{l}\""),
                    _ => "(unbound)".to_string(),
                };
                let disp = |key: &str| match s.get(key) {
                    Some(v) => js_disp(v),
                    None => "undefined".to_string(),
                };
                // A closed or dead session shows "-": it has no signal to
                // read, which is not the same claim as "no_signal".
                let signal = match s.get("signal") {
                    Some(Value::String(v)) => v.as_str(),
                    _ => "-",
                };
                format!(
                    "{} {lane_note} | started {} | heartbeat {} | signal {signal}",
                    disp("id"),
                    disp("started_at"),
                    disp("last_heartbeat")
                )
            })
            .collect();
        let text = if sessions.is_empty() {
            "No session records.".to_string()
        } else {
            lines.join("\n")
        };
        let result = Value::Array(sessions.into_iter().map(Value::Object).collect());
        Ok(Out::Emit(result, text, 0))
    })();
    finish(&ctx, out)
}

// ─── state session bind / unbind ───────────────────────────────────────────

/// `true` when `lane` names no lane record — the one condition `session bind`
/// refuses on. A present-but-unreadable record is NOT this condition:
/// `read_lane_strict` raises its own typed refusal, which propagates unchanged
/// rather than being flattened into "the lane does not exist".
pub(crate) fn bind_lane_missing(root: &Path, lane: &str) -> Result<bool, Err2> {
    Ok(read_lane_strict(root, lane)?.is_none())
}

pub(crate) fn run_session_bind(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["session-id", "lane"]) {
        return None;
    }
    let ctx = match go("state session bind", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let session_id_raw = match require_flag(&flags, "session-id") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let lane_raw = match require_flag(&flags, "lane") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        // bindSessionLane: requireId runs BEFORE the lock is touched at all.
        let session = match require_id(&session_id_raw, "session id") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let lane = match require_id(&lane_raw, "lane feature") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        // The lane a bind names must ALREADY exist. Every other lane-resolving
        // seam refuses a binding that names no `.bee/lanes/<f>.json` (ledger's
        // bound_lane_missing_refusal, the hook's own lane guard), so a bind
        // that accepts one only writes a record the rest of the CLI is obliged
        // to reject — and the guard then stands in front of the very unbind
        // those refusals name as the FIX. Checked BEFORE the lock, so the
        // refusal costs no lock and mutates nothing.
        if bind_lane_missing(&ctx.root, &lane)? {
            return Ok(Out::Thrown(lane_missing_refusal("session bind", &lane)));
        }
        let Some(guard) = acquire_sessions_lock(&ctx.root) else {
            return Ok(Out::Thrown(format!(
                "session bind: session \"{session}\" bind to lane \"{lane}\" could not acquire the sessions lock after 15 bounded attempts — never waited unboundedly."
            )));
        };
        let record = read_session(&ctx.root, &session)?;
        let Some(mut record) = record else {
            return Ok(Out::Thrown(format!(
                "session bind: session \"{session}\" has no record to bind to lane \"{lane}\"."
            )));
        };
        record.insert("lane".into(), json!(lane));
        write_json_atomic(
            &sessions_dir(&ctx.root).join(format!("{session}.json")),
            &Value::Object(record.clone()),
        )
        .map_err(|_| Err2::Ex)?;
        drop(guard);
        Ok(Out::Emit(
            Value::Object(record),
            format!("Session \"{session_id_raw}\" bound to lane \"{lane_raw}\"."),
            0,
        ))
    })();
    finish(&ctx, out)
}

pub(crate) fn run_session_unbind(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["session-id"]) {
        return None;
    }
    let ctx = match go("state session unbind", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let session_id_raw = match require_flag(&flags, "session-id") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let session = match require_id(&session_id_raw, "session id") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let Some(guard) = acquire_sessions_lock(&ctx.root) else {
            return Ok(Out::Thrown(format!(
                "session unbind: session \"{session}\" unbind could not acquire the sessions lock after 15 bounded attempts — never waited unboundedly."
            )));
        };
        let record = read_session(&ctx.root, &session)?;
        let Some(mut record) = record else {
            return Ok(Out::Thrown(format!(
                "session unbind: session \"{session}\" has no record to unbind."
            )));
        };
        // `const { lane, ...unbound } = record` — the key is OMITTED entirely.
        record.shift_remove("lane");
        write_json_atomic(
            &sessions_dir(&ctx.root).join(format!("{session}.json")),
            &Value::Object(record.clone()),
        )
        .map_err(|_| Err2::Ex)?;
        drop(guard);
        Ok(Out::Emit(
            Value::Object(record),
            format!("Session \"{session_id_raw}\" unbound from its lane."),
            0,
        ))
    })();
    finish(&ctx, out)
}

/// The plain flag→env resolver `state session release` targets its own
/// session with: `--session-id` wins, then `BEE_SESSION_ID`, then
/// `CLAUDE_CODE_SESSION_ID` — mirrors claims.rs's `resolve_session_flag_env`
/// exactly, deliberately WITHOUT `resolve_session_id`'s single-live-session
/// adoption fallback: a release names its own session, never guesses one
/// from the sessions directory.
fn resolve_release_session_id(flag: Option<&str>) -> Option<String> {
    if let Some(f) = flag {
        if !js_trim(f).is_empty() {
            return Some(js_trim(f).to_string());
        }
    }
    env_nonempty("BEE_SESSION_ID").or_else(|| env_nonempty("CLAUDE_CODE_SESSION_ID"))
}

/// `bee state session release` — marks an OPEN session `status: "closed"`,
/// `released: true` so it stops holding write-guard/concurrency locks
/// instantly instead of waiting out the 900s heartbeat-stale window, while
/// the record itself stays open for the split revival: `state_sync.rs`'s
/// PostToolUse heartbeat leaves a `released` mark intact (so the release
/// command's own trailing hook call can't immediately undo it), while
/// `prompt_context.rs`'s UserPromptSubmit revival DOES clear it — the user
/// speaking again is the re-engage signal.
pub(crate) fn run_session_release(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["session-id"]) {
        return None;
    }
    let ctx = match go("state session release", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let session_id_flag = flag_value(&flags, "session-id");
        let Some(session_raw) = resolve_release_session_id(session_id_flag.as_deref()) else {
            return Ok(Out::Thrown(
                "state session release: no session id resolved — pass --session-id, or run \
                 inside a session exporting BEE_SESSION_ID / CLAUDE_CODE_SESSION_ID."
                    .to_string(),
            ));
        };
        let session = match require_id(&session_raw, "session id") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let Some(guard) = acquire_sessions_lock(&ctx.root) else {
            return Ok(Out::Thrown(format!(
                "state session release: session \"{session}\" release could not acquire the sessions lock after 15 bounded attempts — never waited unboundedly."
            )));
        };
        let record = read_session(&ctx.root, &session)?;
        let Some(mut record) = record else {
            drop(guard);
            return Ok(Out::Emit(
                json!({"id": session, "released": false, "reason": "no_session_record"}),
                format!("Session \"{session}\" has no record — nothing to release."),
                0,
            ));
        };
        let closed_at = now_iso();
        record.insert("status".into(), json!("closed"));
        record.insert("closed_at".into(), json!(closed_at));
        record.insert("released".into(), json!(true));
        write_json_atomic(
            &sessions_dir(&ctx.root).join(format!("{session}.json")),
            &Value::Object(record),
        )
        .map_err(|_| Err2::Ex)?;
        drop(guard);
        Ok(Out::Emit(
            json!({"id": session, "released": true, "closed_at": closed_at}),
            format!("Session \"{session}\" released — marked closed until the user returns."),
            0,
        ))
    })();
    finish(&ctx, out)
}

// ─── state handoff write / adopt / show ────────────────────────────────────
//
// multisession-native-15 (D5): each verb first resolves WHICH workflow it
// targets and, when one resolves, reads/writes/adopts THAT workflow's own
// mailbox (.bee/runtime/handoffs/<workflow-id>/NNNN.json) instead of the
// single legacy .bee/HANDOFF.json — every mailbox mutation then rebuilds the
// legacy file as a display projection. A repo with zero workflow records (C1),
// or a call where nothing resolves, keeps the legacy single-file path.

/// bee.mjs resolveHandoffWorkflowId — explicit --lane > the calling session's
/// bound lane > the DEFAULT record's own live workflow > null. A --lane or a
/// bound session naming NO live workflow refuses loudly (never guesses back).
pub(crate) fn resolve_handoff_workflow_id(
    root: &Path,
    lane_feature: Option<&str>,
    session_id_flag: Option<&str>,
) -> Result<Option<String>, Err2> {
    let workflows = list_workflows(root)?;
    if workflows.is_empty() {
        return Ok(None); // C1: no workflow records anywhere.
    }
    if let Some(f) = lane_feature {
        return match find_live_workflow(&workflows, f) {
            Some(wf) => Ok(Some(wf_id(wf))),
            None => Err(Err2::Msg(format!(
                "state handoff: refused \u{2014} --lane \"{f}\" names no live workflow (no .bee/runtime/workflows/*/state.json with feature \"{f}\" and status !== closed). FIX: start it first (\"state start-feature --feature {f} --as-lane\"), or omit --lane."
            ))),
        };
    }
    let (sid, bound) = session_binding(session_id_flag, root)?;
    if let Some(bound) = bound {
        return match find_live_workflow(&workflows, &bound) {
            Some(wf) => Ok(Some(wf_id(wf))),
            None => Err(Err2::Msg(format!(
                "state handoff: refused \u{2014} calling session \"{}\" is bound to lane \"{bound}\" but no live workflow names it. FIX: start the lane, unbind the session, or pass --lane explicitly.",
                sid_disp(&sid)
            ))),
        };
    }
    let default_record = read_state_strict(root)?;
    if let Some(v) = default_record.get("feature") {
        if truthy(v) {
            if let Some(wf) = find_live_workflow(&workflows, &js_disp(v)) {
                return Ok(Some(wf_id(wf)));
            }
        }
    }
    Ok(None) // nothing resolves — the legacy single-file path handles this call
}

pub(crate) fn run_handoff_write(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(
        &flags,
        &[
            "kind", "writer-session", "previous-cell", "next-cell", "cell", "files", "done",
            "remaining", "feature", "phase", "mode", "next-action", "lane", "target-role",
            "session-id",
        ],
    ) {
        return None;
    }
    // validate(): `kind` is REQUIRED (and therefore enum-enforced) — a
    // missing/out-of-enum kind is the generic STDOUT refusal → delegate.
    let kind = match flags.get("kind") {
        Some(FlagV::S(s)) if s == "planned-next" || s == "pause" => s.clone(),
        _ => return None,
    };
    let ctx = match go("state handoff write", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let lane = match optional_lane_flag(&flags, "state handoff write") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let target_role = flag_string(&flags, "target-role");
        let mut input = Map::new();
        input.insert("kind".into(), json!(kind));
        if let Some(v) = flag_string(&flags, "feature") {
            input.insert("feature".into(), json!(v));
        }
        if let Some(v) = flag_string(&flags, "phase") {
            input.insert("phase".into(), json!(v));
        }
        if let Some(v) = flag_string(&flags, "mode") {
            input.insert("mode".into(), json!(v));
        }
        if let Some(v) = flag_string(&flags, "next-action") {
            input.insert("next_action".into(), json!(v));
        }
        if kind == "planned-next" {
            for (flag, key) in [
                ("writer-session", "writer_session"),
                ("previous-cell", "previous_cell"),
                ("next-cell", "next_cell"),
            ] {
                match require_flag(&flags, flag) {
                    Ok(v) => {
                        input.insert(key.into(), json!(v));
                    }
                    Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
                    Err(Err2::Ex) => return Err(Err2::Ex),
                }
            }
        } else {
            if let Some(v) = flag_string(&flags, "cell") {
                input.insert("cell".into(), json!(v));
            }
            for (flag, key) in [("files", "files"), ("done", "done"), ("remaining", "remaining")] {
                if let Some(v) = flag_string(&flags, flag) {
                    let list: Vec<Value> = split_list(&v).into_iter().map(|s| json!(s)).collect();
                    input.insert(key.into(), Value::Array(list));
                }
            }
        }
        let workflow_id = match resolve_handoff_workflow_id(
            &ctx.root,
            lane.as_deref(),
            flag_value(&flags, "session-id").as_deref(),
        ) {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        if let Some(wid) = workflow_id {
            let record = match write_mailbox_handoff(
                &ctx.root,
                &wid,
                &input,
                target_role.as_deref(),
            ) {
                Ok(r) => r,
                Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
                Err(Err2::Ex) => return Err(Err2::Ex),
            };
            rebuild_handoff_projection(&ctx.root)?;
            let kind_disp = js_disp_opt(record.get("kind"));
            let seq_disp = js_disp_opt(record.get("seq"));
            let text = format!(
                "Wrote \"{kind_disp}\" handoff to workflow \"{wid}\" mailbox (seq {seq_disp})."
            );
            return Ok(Out::Emit(Value::Object(record), text, 0));
        }
        // Legacy single-file path (C1).
        let record = match write_handoff(&ctx.root, &input, &kind) {
            Ok(r) => r,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let kind_disp = js_disp(record.get("kind").unwrap_or(&Value::Null));
        Ok(Out::Emit(
            Value::Object(record),
            format!("Wrote \"{kind_disp}\" handoff."),
            0,
        ))
    })();
    finish(&ctx, out)
}

pub(crate) fn run_handoff_adopt(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["session-id", "lane", "target-role"]) {
        return None;
    }
    // validate(): `session-id` is REQUIRED — missing/empty delegates.
    if !matches!(flags.get("session-id"), Some(FlagV::S(s)) if !s.is_empty()) {
        return None;
    }
    let ctx = match go("state handoff adopt", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let session_id = match require_flag(&flags, "session-id") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let lane = match optional_lane_flag(&flags, "state handoff adopt") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let target_role = flag_string(&flags, "target-role");
        let workflow_id =
            match resolve_handoff_workflow_id(&ctx.root, lane.as_deref(), Some(&session_id)) {
                Ok(v) => v,
                Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
                Err(Err2::Ex) => return Err(Err2::Ex),
            };
        if let Some(wid) = workflow_id {
            let adopted = match adopt_mailbox_handoff(
                &ctx.root,
                &wid,
                &session_id,
                target_role.as_deref(),
            ) {
                Ok(v) => v,
                Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
                Err(Err2::Ex) => return Err(Err2::Ex),
            };
            return match adopted {
                MailboxAdopt::Fail { reason } => {
                    Ok(Out::Thrown(format!("state handoff adopt: {reason}")))
                }
                MailboxAdopt::Ok { claim, previous_owner, next_cell, workflow_id, seq } => {
                    rebuild_handoff_projection(&ctx.root)?;
                    let mut result = Map::new();
                    result.insert("ok".into(), json!(true));
                    result.insert("claim".into(), claim.unwrap_or(Value::Null));
                    if let Some(prev) = previous_owner {
                        // undefined is dropped by JSON.stringify.
                        result.insert("previous_owner".into(), prev);
                    }
                    result.insert("next_cell".into(), json!(next_cell));
                    result.insert("workflow_id".into(), json!(workflow_id));
                    result.insert("seq".into(), json!(seq));
                    let text = format!(
                        "Adopted the handoff's carried claim on \"{next_cell}\" into session \"{session_id}\" (workflow \"{wid}\"); handoff cleared."
                    );
                    Ok(Out::Emit(Value::Object(result), text, 0))
                }
            };
        }
        // Legacy single-file path (C1).
        match adopt_handoff(&ctx.root, &session_id) {
            Err(Err2::Msg(m)) => Ok(Out::Thrown(m)), // requireId's own throws, unprefixed
            Err(Err2::Ex) => Err(Err2::Ex),
            Ok(HandoffAdopt::Fail { reason }) => {
                Ok(Out::Thrown(format!("state handoff adopt: {reason}")))
            }
            Ok(HandoffAdopt::Ok { claim, previous_owner, next_cell }) => {
                let mut result = Map::new();
                result.insert("ok".into(), json!(true));
                result.insert("claim".into(), Value::Object(claim));
                if let Some(prev) = previous_owner {
                    result.insert("previous_owner".into(), prev);
                }
                result.insert("next_cell".into(), json!(next_cell));
                let text = format!(
                    "Adopted the handoff's carried claim on \"{next_cell}\" into session \"{session_id}\"; handoff cleared."
                );
                Ok(Out::Emit(Value::Object(result), text, 0))
            }
        }
    })();
    finish(&ctx, out)
}

pub(crate) fn run_handoff_show(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["lane", "target-role", "session-id"]) {
        return None;
    }
    let ctx = match go("state handoff show", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let lane = match optional_lane_flag(&flags, "state handoff show") {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let target_role = flag_string(&flags, "target-role");
        let workflow_id = match resolve_handoff_workflow_id(
            &ctx.root,
            lane.as_deref(),
            flag_value(&flags, "session-id").as_deref(),
        ) {
            Ok(v) => v,
            Err(Err2::Msg(m)) => return Ok(Out::Thrown(m)),
            Err(Err2::Ex) => return Err(Err2::Ex),
        };
        let handoff: Option<Value> = match &workflow_id {
            Some(wid) => {
                newest_open_handoff_mailbox_record(&ctx.root, wid, target_role.as_deref())?
                    .map(Value::Object)
            }
            None => read_handoff(&ctx.root)?,
        };
        let m = match handoff {
            None => return Ok(Out::Emit(Value::Null, "No handoff.".to_string(), 0)),
            Some(v) if !truthy(&v) => {
                return Ok(Out::Emit(Value::Null, "No handoff.".to_string(), 0))
            }
            Some(Value::Object(m)) => m,
            Some(_) => return Err(Err2::Ex), // truthy non-object — JS property exotica
        };
        // `${h.feature ?? 'unknown'}` — nullish coalescing per field.
        let field = |key: &str| match m.get(key) {
            None | Some(Value::Null) => "unknown".to_string(),
            Some(v) => js_disp(v),
        };
        let text = format!(
            "kind={} feature={} phase={} mode={}",
            js_disp(m.get("kind").unwrap_or(&Value::Null)),
            field("feature"),
            field("phase"),
            field("mode"),
        );
        Ok(Out::Emit(Value::Object(m), text, 0))
    })();
    finish(&ctx, out)
}


// ─── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn record(json_text: &str) -> Map<String, Value> {
        match serde_json::from_str::<Value>(json_text).unwrap() {
            Value::Object(m) => m,
            _ => panic!("expected an object"),
        }
    }

    /// One fixed `now`, so every case reads the same clock the production
    /// path reads: a stamp `age_secs` old against it.
    const NOW_MS: f64 = 1_700_000_000_000.0;

    /// Exotic carries no Debug (reservations.rs owns it), so the local
    /// expect keeps a fixture panic readable — the same shape the sibling
    /// suite's `ok` helper uses.
    fn iso(ms: f64) -> String {
        match iso_from_ms(ms) {
            Ok(s) => s,
            Err(_) => panic!("fixture timestamp is inside the JS Date range"),
        }
    }

    fn with_activity_age(age_secs: f64) -> Map<String, Value> {
        let at = iso(NOW_MS - age_secs * 1000.0);
        record(&format!(
            "{{\"id\":\"s1\",\"last_heartbeat\":\"{at}\",\"activity\":{{\"state\":\"working\",\"event\":\"PreToolUse\",\"at\":\"{at}\"}}}}"
        ))
    }

    /// D4: inside the 90 s window the agent is live. 89 s is the last second
    /// that still counts.
    #[test]
    fn session_signal_reads_live_inside_the_window() {
        assert_eq!(session_signal(&with_activity_age(89.0), NOW_MS), Some("live"));
    }

    /// D4: past the window the record says nothing about now — `no_signal`,
    /// never "gone". 91 s is the first second outside it.
    #[test]
    fn session_signal_reads_no_signal_past_the_window() {
        assert_eq!(session_signal(&with_activity_age(91.0), NOW_MS), Some("no_signal"));
    }

    /// A session the activity hook has never written for is indistinguishable
    /// from one that went quiet: both read `no_signal`.
    #[test]
    fn session_signal_reads_no_signal_without_an_activity_object() {
        let plain = record("{\"id\":\"s1\",\"last_heartbeat\":\"2026-08-22T00:00:00.000Z\"}");
        assert_eq!(session_signal(&plain, NOW_MS), Some("no_signal"));
    }

    /// A finished session carries NO signal — `None`, so callers emit null
    /// instead of asserting `no_signal` about a session that ended on purpose.
    /// Fresh activity does not change that: `status` decides first.
    #[test]
    fn session_signal_is_absent_for_a_dead_or_closed_session() {
        for status in ["dead", "closed"] {
            let mut ended = with_activity_age(1.0);
            ended.insert("status".into(), json!(status));
            assert_eq!(session_signal(&ended, NOW_MS), None, "status {status}");
        }
    }
}
