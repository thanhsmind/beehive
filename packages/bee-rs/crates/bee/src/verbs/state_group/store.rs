// the state store, the phase rules, sessions, claims and the legacy handoff file
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
    acquire_named_lock, acquire_workflow_lock, adopt_mailbox_handoff, build_waiting_on,
    create_workflow, find_live_workflow, NewWorkflow,
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

// ─── state store (lib/state.mjs readStateStrict / readState / writeState) ──

pub(crate) fn state_path(root: &Path) -> PathBuf {
    root.join(".bee").join("state.json")
}

pub(crate) fn default_gates() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("context".into(), json!(false));
    m.insert("shape".into(), json!(false));
    m.insert("execution".into(), json!(false));
    m.insert("review".into(), json!(false));
    m
}

pub(crate) fn default_state() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("schema_version".into(), json!("1.0"));
    m.insert("phase".into(), json!("idle"));
    m.insert("feature".into(), Value::Null);
    m.insert("mode".into(), Value::Null);
    m.insert("approved_gates".into(), Value::Object(default_gates()));
    m.insert("workers".into(), json!([]));
    m.insert("summary".into(), json!(""));
    m.insert(
        "next_action".into(),
        json!("No active bee work — awaiting a user request."),
    );
    // D1/D3 (awaiting-human): the wait mark, session-scoped first — this is
    // where it lives when no feature is active (D3), and the shape
    // `apply_workflow_d1_fields` copies a live workflow's own mark into once
    // a feature exists. Null is "no live wait", matching the workflow
    // record's own default.
    m.insert("waiting_on".into(), Value::Null);
    m
}

/// The Rust-native gate merge (D2, docs/history/js-parity-cleanup/CONTEXT.md)
/// lives beside `default_gates` in state.rs; re-exported here so every
/// state_group caller reaches it through this module, unchanged.
pub(crate) use crate::state::spread_gates;

/// coerceLegacyPhase applied to the merged map's phase slot (D13).
pub(crate) fn coerce_legacy_phase(m: &mut Map<String, Value>) -> Ex<()> {
    let Some(phase) = m.get("phase") else { return Ok(()) };
    if matches!(phase, Value::String(s) if s == "validating") {
        m.insert("phase".into(), json!("planning"));
    } else if !matches!(phase, Value::String(_)) && js_disp(phase) == "validating" {
        // hasOwnProperty coerces the property key — an exotic value whose JS
        // string form is "validating" would coerce in Node; delegate.
        return Err(Exotic);
    }
    Ok(())
}

/// `{ ...defaultState(), ...parsed }` + gates merge + phase coercion — shared
/// by readState and readStateStrict.
pub(crate) fn merge_state_with_defaults(parsed: &Map<String, Value>) -> Ex<Map<String, Value>> {
    let mut merged = default_state();
    for (k, v) in parsed {
        merged.insert(k.clone(), v.clone());
    }
    let gates = spread_gates(parsed.get("approved_gates"));
    merged.insert("approved_gates".into(), Value::Object(gates));
    coerce_legacy_phase(&mut merged)?;
    Ok(merged)
}

/// readStateStrict — the three typed refuse-to-rebuild errors are DETERMINISTIC
/// bytes (they embed only path + code strings) and are served natively.
pub(crate) fn read_state_strict(root: &Path) -> Result<Map<String, Value>, Err2> {
    let file = state_path(root);
    let file_disp = file.display().to_string();
    let rel = rel_bee(&["state.json"]);
    let tail_read = "The bee CLI refuses to rebuild state from defaults when it cannot read the existing file — that could silently clobber real state (gates, workers, feature).";
    let tail_corrupt_a = "The bee CLI refuses to rebuild state from defaults over a present-but-corrupt file — that would silently clobber real state (gates, workers, feature) while reporting success.";
    let fix = format!("FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\"), then retry.");
    let bytes = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(default_state()),
        Err(e) => {
            // CUTOVER. Node interpolated `err.code` — a libuv errno string —
            // and this port reproduced only EISDIR/EPERM, delegating the rest.
            // The refusal, its typed shape and its exit code are unchanged;
            // only the parenthetical is now ours, and EVERY read failure gets
            // one instead of half of them delegating.
            return Err(Err2::Msg(format!(
                "readStateStrict: could not read \"{file_disp}\" ({}). {tail_read} {fix}",
                io_read_reason(&file, &e)
            )));
        }
    };
    // Node reads utf8 (lossy) and parses WITHOUT a BOM strip (unlike readJson).
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let parsed = match parse_json_v8(&text)? {
        ParsedJson::Parsed(v) => v,
        ParsedJson::Unparseable => {
            return Err(Err2::Msg(format!(
                "readStateStrict: \"{file_disp}\" exists but is not valid JSON. {tail_corrupt_a} {fix}"
            )));
        }
    };
    let obj = match &parsed {
        Value::Object(m) => m.clone(),
        other => {
            let found = match other {
                Value::Array(_) => "an array",
                Value::Null => "object", // typeof null
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Bool(_) => "boolean",
                Value::Object(_) => unreachable!(),
            };
            return Err(Err2::Msg(format!(
                "readStateStrict: \"{file_disp}\" exists but is not a JSON object (found {found}). {tail_corrupt_a} {fix}"
            )));
        }
    };
    merge_state_with_defaults(&obj).map_err(|_| Err2::Ex)
}

/// readState — the fail-open peek. A corrupt file WARNS (readJson's own
/// sentence, our wording) and reads as `defaultState()`, exactly the fallback
/// `readJson(statePath, null)` handed Node's `!state` guard.
pub(crate) fn read_state_peek(root: &Path) -> Ex<Map<String, Value>> {
    match read_json(&state_path(root)) {
        ReadJson::Missing => Ok(default_state()),
        ReadJson::Corrupt => {
            warn_corrupt_json(&state_path(root));
            Ok(default_state())
        }
        ReadJson::Parsed(v) => match js_numberify(&v)? {
            Value::Object(m) => merge_state_with_defaults(&m),
            _ => Ok(default_state()),
        },
    }
}

pub(crate) fn write_state(root: &Path, state: &Map<String, Value>) -> Result<(), Err2> {
    write_json_atomic(&state_path(root), &Value::Object(state.clone())).map_err(|_| Err2::Ex)
}

/// D1/D3 (awaiting-human): the setter for the SESSION-SCOPED case — mark the
/// default `.bee/state.json` record as waiting on the human when no live
/// workflow claims the current feature (D3's named gap: a question asked
/// before any feature exists still needs a home). Shares its typed refusal
/// with the workflow-record setter via `build_waiting_on` (writes nothing on
/// an unknown kind or an empty subject/session) — one vocabulary, two
/// storage locations. `run_state` has no gates/cells to derive from at this
/// layer (no workflow record exists), so a live mark here writes
/// "awaiting-approval" directly, the same value the record path derives —
/// D1's "ONE state" holds across both.
#[allow(dead_code)] // not yet wired to a CLI verb — ah-3 owns the read/command surface
pub(crate) fn set_default_state_waiting_on(
    root: &Path,
    kind: &str,
    subject: &str,
    session: &str,
) -> Result<Map<String, Value>, Err2> {
    let mark = build_waiting_on(kind, subject, session)?;
    let guard = acquire_named_lock(root, "state")?;
    let out = (|| -> Result<Map<String, Value>, Err2> {
        let mut current = read_state_strict(root)?;
        current.insert("waiting_on".into(), mark);
        current.insert("run_state".into(), json!("awaiting-approval"));
        write_state(root, &current)?;
        Ok(current)
    })();
    drop(guard);
    out
}

// ─── phase rules (state.mjs, chain-integrity door — PURE) ──────────────────

pub(crate) struct Transition {
    pub(crate) ok: bool,
    pub(crate) reason: String,
    /// transition.waivedCompounding — production reads it only on the close
    /// path, which this port delegates (the waiver's decision-logging lives in
    /// decisions.mjs); kept so the door tests pin the full Node contract.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) waived_compounding: bool,
}

/// checkPhaseTransition (state.mjs:116-166) — the chain-integrity tail guard.
pub(crate) fn check_phase_transition(
    from: Option<&Value>,
    to: &str,
    rec: &Map<String, Value>,
    waive_compounding: bool,
) -> Ex<Transition> {
    let refuse = |reason: String| Transition { ok: false, reason, waived_compounding: false };
    if to == "compounding" {
        return Ok(refuse(
            "set: phase \"compounding\" is not settable directly — it is produced only by RECORDING a real scribing run, never by asserting one. FIX: run `bee state scribing-run --feature <f> --areas \"<a,b>\" --next-action \"<n>\"`, which stamps last_scribing_run and advances the phase for you.".to_string(),
        ));
    }
    // `current !== 'compounding'` — strict, so only the exact string passes.
    let current_is_compounding = matches!(from, Some(Value::String(s)) if s == "compounding");
    if to == "compounding-complete" && !current_is_compounding {
        let current = disp_or_idle(from);
        return Ok(refuse(format!(
            "set: phase \"compounding-complete\" may only be entered from \"compounding\" (current: \"{current}\"). That name asserts scribing ran, compounding ran, AND the compounding run was RECORDED — not merely asserted; setting it from \"{current}\" claims work that did not happen and shuts the intake gate on a feature that never closed. FIX: close the chain in order — bee-capturing (`state scribing-run`), then bee-capturing (`state compounding-run`)."
        )));
    }
    if to == "compounding-complete" {
        let run = rec.get("last_compounding_run");
        let scribing = rec.get("last_scribing_run");
        let parse_at = |v: Option<&Value>| -> Ex<Option<f64>> {
            match v {
                Some(v) if truthy(v) => match jget(v, "at") {
                    Some(Value::String(_)) => date_parse_val(jget(v, "at")),
                    _ => Ok(None), // typeof run.at !== 'string' → NaN
                },
                _ => Ok(None),
            }
        };
        let run_at = parse_at(run)?;
        let scribing_at = parse_at(scribing)?;
        let run_truthy = run.map(truthy).unwrap_or(false);
        let scribing_truthy = scribing.map(truthy).unwrap_or(false);
        let fresh = run_truthy
            && scribing_truthy
            && matches!((run_at, scribing_at), (Some(r), Some(s)) if r >= s)
            && opt_strict_eq(
                run.and_then(|v| jget(v, "feature")),
                scribing.and_then(|v| jget(v, "feature")),
            );
        if !fresh && !waive_compounding {
            // (scribing && scribing.feature) || rec.feature || '<f>'
            let cand1 = if scribing_truthy {
                scribing.and_then(|v| jget(v, "feature"))
            } else {
                None
            };
            let feature_name = [cand1, rec.get("feature")]
                .into_iter()
                .flatten()
                .find(|v| truthy(v))
                .map(js_disp)
                .unwrap_or_else(|| "<f>".to_string());
            return Ok(refuse(format!(
                "set: phase \"compounding-complete\" refused — no fresh compounding run recorded for feature \"{feature_name}\" (last_compounding_run must exist, with an \"at\" timestamp at or after last_scribing_run.at, for the same feature). FIX: run `bee state compounding-run --feature {feature_name} --learnings <path>`, then retry. If compounding genuinely needs no separate recorded run here, retry with --waive-compounding — it is permitted, but it logs a decision naming the feature."
            )));
        }
        return Ok(Transition { ok: true, reason: String::new(), waived_compounding: !fresh && waive_compounding });
    }
    Ok(Transition { ok: true, reason: String::new(), waived_compounding: false })
}

/// checkScribingRunPhase — None = ok, Some(reason) = refuse.
pub(crate) fn check_scribing_run_phase(from: Option<&Value>) -> Option<String> {
    if matches!(from, Some(Value::String(s)) if SCRIBING_RUN_FROM.contains(&s.as_str())) {
        return None;
    }
    let current = disp_or_idle(from);
    Some(format!(
        "scribing-run: refused from phase \"{current}\" — a scribing run records the spec sync for work that has been EXECUTED. Legal from: swarming, reviewing, scribing. FIX: if execution really is done, the phase should say so; if it is not, there is nothing to scribe yet."
    ))
}

/// checkCompoundingRunPhase — legal only from "compounding".
pub(crate) fn check_compounding_run_phase(from: Option<&Value>) -> Option<String> {
    if matches!(from, Some(Value::String(s)) if s == "compounding") {
        return None;
    }
    let current = disp_or_idle(from);
    Some(format!(
        "compounding-run: refused from phase \"{current}\" — a compounding run records the durable-learnings sync for a feature whose scribing has already run. Legal from: compounding. FIX: if compounding really is underway, the phase should already say so (`state scribing-run` is what produces it); if it is not, there is nothing to compound yet."
    ))
}

// ─── sessions (lib/claims.mjs — control root == root on the native path) ───

pub(crate) fn sessions_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("sessions")
}

/// claims.mjs requireId — thrown Errors, byte-identical.
pub(crate) fn require_id(value: &str, label: &str) -> Result<String, Err2> {
    let id = js_trim(value);
    if id.is_empty() {
        return Err(Err2::Msg(format!("{label} is required.")));
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(Err2::Msg(format!("{label} must be a plain id (no path separators).")));
    }
    Ok(id.to_string())
}

/// readSession — fail-open (malformed id / missing → None). A corrupt record
/// WARNS and reads as "no session", the fallback `readJson(file, null)` fed
/// Node's `!session` guard.
pub(crate) fn read_session(root: &Path, session_id: &str) -> Ex<Option<Map<String, Value>>> {
    let id = js_trim(session_id);
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains("..") {
        return Ok(None);
    }
    let file = sessions_dir(root).join(format!("{id}.json"));
    match read_json(&file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => {
            warn_corrupt_json(&file);
            Ok(None)
        }
        ReadJson::Parsed(v) => match js_numberify(&v)? {
            Value::Object(m) => {
                // session.id !== String(sessionId).trim() → null
                if matches!(m.get("id"), Some(Value::String(s)) if s == id) {
                    Ok(Some(m))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        },
    }
}

pub(crate) fn list_session_records(root: &Path) -> Ex<Vec<Map<String, Value>>> {
    let entries = match std::fs::read_dir(sessions_dir(root)) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".json") else { continue };
        if let Some(record) = read_session(root, stem)? {
            out.push(record);
        }
    }
    Ok(out)
}

/// claims.mjs heartbeatStale (default 900s window).
pub(crate) fn heartbeat_stale(session: &Map<String, Value>, now: f64) -> Ex<bool> {
    match date_parse_val(session.get("last_heartbeat"))? {
        None => Ok(true),
        Some(ms) => Ok(ms + HEARTBEAT_STALE_SECONDS * 1000.0 <= now),
    }
}

pub(crate) fn env_nonempty(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(v) if !js_trim(&v).is_empty() => Some(js_trim(&v).to_string()),
        _ => None,
    }
}

/// claims.mjs resolveSessionId({flag, root}) — the explicit flag wins, then
/// the env chain, then single-live-session adoption.
pub(crate) fn resolve_session_id(flag: Option<&str>, root: &Path) -> Ex<Option<String>> {
    if let Some(f) = flag {
        if !js_trim(f).is_empty() {
            return Ok(Some(js_trim(f).to_string()));
        }
    }
    resolve_session_id_no_flag(root)
}

/// resolveSessionId({root}) — env chain, then single-live-session adoption.
pub(crate) fn resolve_session_id_no_flag(root: &Path) -> Ex<Option<String>> {
    if let Some(v) = env_nonempty("BEE_SESSION_ID") {
        return Ok(Some(v));
    }
    if let Some(v) = env_nonempty("CLAUDE_CODE_SESSION_ID") {
        return Ok(Some(v));
    }
    let now = now_ms();
    let mut fresh: Vec<Map<String, Value>> = Vec::new();
    for r in list_session_records(root)? {
        if !heartbeat_stale(&r, now)? {
            fresh.push(r);
        }
    }
    if fresh.len() == 1 {
        if let Some(Value::String(id)) = fresh[0].get("id") {
            return Ok(Some(id.clone()));
        }
    }
    Ok(None)
}

/// The `(sessionId, boundLane)` pair every resolution seam shares:
/// `session && typeof session.lane === 'string' ? session.lane.trim() : ''`.
pub(crate) fn session_binding(flag: Option<&str>, root: &Path) -> Ex<(Option<String>, Option<String>)> {
    let Some(sid) = resolve_session_id(flag, root)? else { return Ok((None, None)) };
    let Some(sess) = read_session(root, &sid)? else { return Ok((Some(sid), None)) };
    let bound = match sess.get("lane") {
        Some(Value::String(l)) if !js_trim(l).is_empty() => Some(js_trim(l).to_string()),
        _ => None,
    };
    Ok((Some(sid), bound))
}

/// claims.mjs SESSIONS_LOCK_NAME bounded acquire (15 × 20ms, acquire-once).
pub(crate) fn acquire_sessions_lock(root: &Path) -> Option<lock::LockGuard> {
    for attempt in 0..15u32 {
        match lock::acquire_store_lock_once(root, "sessions") {
            AcquireOnce::Acquired(guard) => return Some(guard),
            AcquireOnce::Busy { .. } => {
                if attempt + 1 < 15 {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
            }
        }
    }
    None
}

// ─── claims (readClaim / adoptClaim + the per-claim gate file) ─────────────

pub(crate) fn claims_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("claims")
}

/// readClaim — `!claim || typeof claim !== 'object'` → null. Arrays pass
/// typeof in JS; the callers here treat them as Exotic when spread. A corrupt
/// file WARNS and reads as "no claim" (readJson's `null` fallback).
pub(crate) fn read_claim(root: &Path, cell: &str) -> Ex<Option<Value>> {
    let file = claims_dir(root).join(format!("{cell}.json"));
    match read_json(&file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => {
            warn_corrupt_json(&file);
            Ok(None)
        }
        ReadJson::Parsed(v) => match js_numberify(&v)? {
            v @ (Value::Object(_) | Value::Array(_)) => Ok(Some(v)),
            _ => Ok(None),
        },
    }
}

/// Removes the `<cell>.adopting` gate file on drop (releaseGate, force:true).
pub(crate) struct GateGuard(pub(crate) PathBuf);

impl Drop for GateGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub(crate) enum AdoptOutcome {
    Fail { reason: String },
    Adopted { claim: Map<String, Value>, previous_owner: Option<Value> },
}

/// adoptClaim (claims.mjs) — in-place atomic rewrite under the exclusive
/// gate; fence_epoch bumps by exactly 1 in the same write.
pub(crate) fn adopt_claim(root: &Path, cell_id: &str, new_session_id: &str) -> Result<AdoptOutcome, Err2> {
    let cell = require_id(cell_id, "cell id")?;
    let session = require_id(new_session_id, "session id")?;
    ensure_dir(&claims_dir(root)).map_err(|_| Err2::Ex)?;
    let gate_path = claims_dir(root).join(format!("{cell}.adopting"));
    let now = now_iso();
    let gate_body = format!(
        "{}\n",
        jsjson::stringify(&json!({"pid": std::process::id(), "at": now}))
    );
    match std::fs::OpenOptions::new().write(true).create_new(true).open(&gate_path) {
        Ok(mut f) => {
            use std::io::Write;
            f.write_all(gate_body.as_bytes()).map_err(|_| Err2::Ex)?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Ok(AdoptOutcome::Fail {
                reason: format!(
                    "claim \"{cell}\" is gated by another in-flight adopt/sweep — retry later, never wait on the gate."
                ),
            });
        }
        Err(_) => return Err(Err2::Ex),
    }
    let _gate = GateGuard(gate_path); // releaseGate via finally, incl. delegate paths
    let claim = read_claim(root, &cell)?;
    let m = match claim {
        None => {
            return Ok(AdoptOutcome::Fail {
                reason: format!("cell \"{cell}\" has no claim to adopt."),
            });
        }
        Some(Value::Array(_)) => return Err(Err2::Ex), // JS array spread exotica
        Some(Value::Object(m)) => m,
        Some(_) => unreachable!("read_claim yields only objects/arrays"),
    };
    let previous = m.get("session").cloned();
    // Number.isFinite(claim.fence_epoch) ? it : 1 — js_numberify keeps numbers finite.
    let previous_epoch = match m.get("fence_epoch") {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(1.0),
        _ => 1.0,
    };
    let mut adopted = m.clone();
    adopted.insert("session".into(), json!(session));
    adopted.insert("claimed_at".into(), json!(now)); // fresh ownership renews the TTL clock
    match &previous {
        // adopted_from: undefined is DROPPED by JSON.stringify — remove the key.
        None => {
            adopted.shift_remove("adopted_from");
        }
        Some(prev) => {
            adopted.insert("adopted_from".into(), prev.clone());
        }
    }
    adopted.insert("adopted_at".into(), json!(now));
    let epoch = serde_json::Number::from_f64(previous_epoch + 1.0).ok_or(Err2::Ex)?;
    adopted.insert("fence_epoch".into(), Value::Number(epoch));
    write_json_atomic(&claims_dir(root).join(format!("{cell}.json")), &Value::Object(adopted.clone()))
        .map_err(|_| Err2::Ex)?;
    Ok(AdoptOutcome::Adopted { claim: adopted, previous_owner: previous })
}

// ─── handoff (lib/state.mjs legacy single-file path — C1 only) ─────────────

pub(crate) fn handoff_path(root: &Path) -> PathBuf {
    root.join(".bee").join("HANDOFF.json")
}

/// readHandoff — fail-open read; an object gets `kind` normalized
/// (missing/unknown → 'pause'). Non-object values return as-is in Node; the
/// callers here delegate on truthy non-objects.
pub(crate) fn read_handoff(root: &Path) -> Ex<Option<Value>> {
    match read_json(&handoff_path(root)) {
        ReadJson::Missing => Ok(None),
        // Corrupt: warn, then `readJson(..., null)`'s fallback flows straight
        // through Node's `!handoff` guard and is RETURNED as-is — null.
        ReadJson::Corrupt => {
            warn_corrupt_json(&handoff_path(root));
            Ok(None)
        }
        ReadJson::Parsed(v) => {
            let v = js_numberify(&v)?;
            match v {
                Value::Object(mut m) => {
                    let normalized = if matches!(m.get("kind"), Some(Value::String(s)) if s == "planned-next")
                    {
                        "planned-next"
                    } else {
                        "pause"
                    };
                    m.insert("kind".into(), json!(normalized));
                    Ok(Some(Value::Object(m)))
                }
                other => Ok(Some(other)),
            }
        }
    }
}

/// A previous-cell id that Node's path.join would treat as a relative
/// filename but Rust's Path::join could interpret differently (drive letters,
/// absolute prefixes, NULs) delegates instead of risking a different read.
pub(crate) fn cell_path_modelable(id: &str) -> bool {
    !(id.contains(':') || id.starts_with('/') || id.starts_with('\\') || id.contains('\0'))
}

/// writeHandoff (state.mjs) — the strict, guarded legacy writer.
pub(crate) fn write_handoff(root: &Path, input: &Map<String, Value>, kind: &str) -> Result<Map<String, Value>, Err2> {
    let now = now_iso();
    if kind == "pause" {
        let mut record = Map::new();
        for (k, v) in input {
            if k != "kind" {
                record.insert(k.clone(), v.clone());
            }
        }
        record.insert("kind".into(), json!("pause"));
        record.insert("written_at".into(), json!(now));
        write_json_atomic(&handoff_path(root), &Value::Object(record.clone())).map_err(|_| Err2::Ex)?;
        return Ok(record);
    }
    // planned-next: every precondition is READ before the single write.
    let get_trim = |key: &str| -> String {
        match input.get(key) {
            Some(Value::String(s)) => js_trim(s).to_string(),
            _ => String::new(),
        }
    };
    let writer_session = get_trim("writer_session");
    let previous_cell = get_trim("previous_cell");
    let next_cell = get_trim("next_cell");
    if writer_session.is_empty() || previous_cell.is_empty() || next_cell.is_empty() {
        return Err(Err2::Msg(
            "writeHandoff: a planned-next handoff requires non-empty writer_session, previous_cell, and next_cell (D1) — FIX: pass all three.".to_string(),
        ));
    }
    if !cell_path_modelable(&previous_cell) {
        return Err(Err2::Ex);
    }
    let prev_file = root.join(".bee").join("cells").join(format!("{previous_cell}.json"));
    let previous = match read_json(&prev_file) {
        ReadJson::Missing => None,
        // Corrupt: warn, then readJson's `null` fallback — the cell reads as
        // absent, so the refusal below reports status "missing", unchanged.
        ReadJson::Corrupt => {
            warn_corrupt_json(&prev_file);
            None
        }
        ReadJson::Parsed(v) => Some(js_numberify(&v)?),
    };
    let prev_capped = previous
        .as_ref()
        .map(|v| truthy(v) && matches!(jget(v, "status"), Some(Value::String(s)) if s == "capped"))
        .unwrap_or(false);
    if !prev_capped {
        // `${previous?.status ?? 'missing'}` — nullish only.
        let status_disp = match previous.as_ref().and_then(|v| jget(v, "status")) {
            None | Some(Value::Null) => "missing".to_string(),
            Some(v) => js_disp(v),
        };
        return Err(Err2::Msg(format!(
            "writeHandoff: refused — previous cell \"{previous_cell}\" is not capped (found status \"{status_disp}\"). A planned-next handoff may only follow a capped cell. FIX: finish \"{previous_cell}\" first (bee cells finish), then retry."
        )));
    }
    // readClaim(controlRootFor(root), nextCell) — claimPath's requireId throws.
    let next_id = require_id(&next_cell, "cell id")?;
    let claim = read_claim(root, &next_id)?;
    let claim_owned = claim
        .as_ref()
        .map(|c| opt_strict_eq(jget(c, "session"), Some(&Value::String(writer_session.clone()))))
        .unwrap_or(false);
    if !claim_owned {
        let found = match &claim {
            None => "no claim".to_string(),
            Some(c) => format!(
                "owner \"{}\"",
                match jget(c, "session") {
                    Some(v) => js_disp(v),
                    None => "undefined".to_string(),
                }
            ),
        };
        return Err(Err2::Msg(format!(
            "writeHandoff: refused — next cell \"{next_cell}\" has no claim owned by writer session \"{writer_session}\" (found {found}). The next cell must already be claimed by the writing session before a planned-next handoff carries it. FIX: claim \"{next_cell}\" as session \"{writer_session}\" first (bee cells claim), then retry."
        )));
    }
    let mut record = input.clone();
    record.insert("kind".into(), json!("planned-next"));
    record.insert("writer_session".into(), json!(writer_session));
    record.insert("previous_cell".into(), json!(previous_cell));
    record.insert("next_cell".into(), json!(next_cell));
    record.insert("written_at".into(), json!(now));
    write_json_atomic(&handoff_path(root), &Value::Object(record.clone())).map_err(|_| Err2::Ex)?;
    Ok(record)
}

pub(crate) enum HandoffAdopt {
    Fail { reason: String },
    Ok { claim: Map<String, Value>, previous_owner: Option<Value>, next_cell: String },
}

/// adoptHandoff (state.mjs) — clear-after-adopt with idempotent recovery.
pub(crate) fn adopt_handoff(root: &Path, session_id: &str) -> Result<HandoffAdopt, Err2> {
    let handoff = read_handoff(root)?;
    let handoff = match handoff {
        None => {
            return Ok(HandoffAdopt::Fail { reason: "no .bee/HANDOFF.json to adopt.".to_string() })
        }
        Some(v) if !truthy(&v) => {
            return Ok(HandoffAdopt::Fail { reason: "no .bee/HANDOFF.json to adopt.".to_string() })
        }
        Some(Value::Object(m)) => m,
        Some(_) => return Err(Err2::Ex), // truthy non-object — JS property exotica
    };
    // read_handoff normalized kind, so a non-planned-next always reads "pause".
    if !matches!(handoff.get("kind"), Some(Value::String(s)) if s == "planned-next") {
        return Ok(HandoffAdopt::Fail {
            reason: format!(
                "handoff kind \"{}\" is not \"planned-next\" — a pause handoff is never adopted, it must be surfaced and WAITED on (D1).",
                handoff.get("kind").map(js_disp).unwrap_or_default()
            ),
        });
    }
    let next_cell = match handoff.get("next_cell") {
        Some(Value::String(s)) => js_trim(s).to_string(),
        _ => String::new(),
    };
    if next_cell.is_empty() {
        return Ok(HandoffAdopt::Fail {
            reason: "planned-next handoff has no next_cell to adopt.".to_string(),
        });
    }
    match adopt_claim(root, &next_cell, session_id)? {
        AdoptOutcome::Fail { reason } => Ok(HandoffAdopt::Fail { reason }),
        AdoptOutcome::Adopted { claim, previous_owner } => {
            let _ = std::fs::remove_file(handoff_path(root)); // rmSync force:true
            Ok(HandoffAdopt::Ok { claim, previous_owner, next_cell })
        }
    }
}
