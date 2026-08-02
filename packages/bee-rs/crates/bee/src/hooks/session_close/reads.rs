// the deferred corrupt-JSON warnings, pre-flight, JS helpers and state reads
//
// Split out of the single 2.8k-line hooks/session_close.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, ReadJson};
use crate::hooks::adapter::{emit_hook_output, encode_block, log_crash, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson::{self, js_to_string};
use crate::state::{bypass_level, read_config_raw};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ─── deferred corrupt-JSON warnings ────────────────────────────────────────
// This hook buffers every byte it emits so a delegating run stays silent, so
// the readJson warning cannot be printed at read time. It is queued here and
// written by flush(), ahead of the advisory's own stderr.

thread_local! {
    static CORRUPT_JSON_WARNINGS: std::cell::RefCell<Vec<String>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Queues the sentence crate::fsutil::warn_corrupt_json would have printed.
pub(crate) fn queue_corrupt_json_warning(file: &Path) {
    let reason = {
        // Same derivation as fsutil.rs's private corrupt_json_reason.
        match std::fs::read(file) {
            Err(_) => "the file could not be read".to_string(),
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
                match serde_json::from_str::<Value>(text) {
                    Ok(_) => "invalid JSON".to_string(), // raced: it parses now
                    Err(e) if e.line() > 0 => {
                        format!("invalid JSON at line {} column {}", e.line(), e.column())
                    }
                    Err(_) => "invalid JSON".to_string(),
                }
            }
        }
    };
    CORRUPT_JSON_WARNINGS.with(|q| {
        q.borrow_mut().push(format!(
            "bee: could not parse JSON at {} — {reason}. Using fallback; fix the file.\n",
            file.display()
        ))
    });
}

/// read_json plus the queued warning: Corrupt collapses onto Missing, which is
/// what readJson's `null` fallback meant to every caller in this hook.
pub(crate) fn read_json_failopen(file: &Path) -> ReadJson {
    match read_json(file) {
        ReadJson::Corrupt => {
            queue_corrupt_json_warning(file);
            ReadJson::Missing
        }
        other => other,
    }
}

pub(crate) fn take_corrupt_json_warnings() -> String {
    CORRUPT_JSON_WARNINGS.with(|q| q.borrow_mut().drain(..).collect::<Vec<_>>().join(""))
}

pub(crate) fn clear_corrupt_json_warnings() {
    CORRUPT_JSON_WARNINGS.with(|q| q.borrow_mut().clear());
}

// ─── pre-flight (the two remaining delegate classes) ───────────────────────

/// Returns the merged tracked+overlay config (read_config_raw warns and reads
/// a corrupt file as absent, so its Err arm is unreachable) after screening the
/// one non-V8 delegate this hook has left: an inject cache that PARSES to a
/// non-object, whose spread/assignment is JS exotica.
pub(crate) fn preflight(root: &Path) -> Result<Map<String, Value>, ()> {
    let bee = root.join(".bee");
    let config = read_config_raw(root).map_err(|_| ())?;
    for cache in [bee.join("cache").join("inject-cache.json"), bee.join(".inject-cache.json")] {
        // Deliberately plain read_json: a corrupt cache is NOT screened here
        // (read_inject_cache warns for it), so this probe never double-warns.
        if let ReadJson::Parsed(v) = read_json(&cache) {
            if !matches!(v, Value::Object(_) | Value::Null | Value::Bool(false)) {
                return Err(());
            }
        }
    }
    Ok(config)
}

// ─── shared JS helpers ─────────────────────────────────────────────────────

pub(crate) fn js_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

pub(crate) fn get_session_id(payload: &Map<String, Value>) -> Option<String> {
    payload
        .get("session_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// claims.mjs requireId / state.mjs requireLaneFeature shape rule.
pub(crate) fn is_plain_id(id: &str) -> bool {
    !id.contains('/') && !id.contains('\\') && !id.contains("..")
}

pub(crate) fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

/// Date.parse for the ISO shapes bee writes (toISOString / date-only).
pub(crate) fn js_date_parse(text: &str) -> Option<f64> {
    let t = text.trim();
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(t) {
        return Some(dt.timestamp_millis() as f64);
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d") {
        let dt = d.and_hms_opt(0, 0, 0)?;
        return Some(dt.and_utc().timestamp_millis() as f64);
    }
    None
}

pub(crate) fn js_date_parse_value(v: &Value) -> Option<f64> {
    js_date_parse(&js_to_string(v))
}

/// new Date(ms).toISOString() — millisecond ISO, Z suffix. Err on invalid.
pub(crate) fn ms_to_iso(ms: f64) -> Result<String, ()> {
    if !ms.is_finite() || ms.abs() > 8.64e15 {
        return Err(());
    }
    let dt = chrono::DateTime::from_timestamp_millis(ms.trunc() as i64).ok_or(())?;
    Ok(dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

pub(crate) fn js_number(v: &Value) -> f64 {
    match v {
        Value::Null => 0.0,
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => {
            let t = s.trim();
            if t.is_empty() {
                0.0
            } else {
                t.parse::<f64>().unwrap_or(f64::NAN)
            }
        }
        _ => f64::NAN,
    }
}

/// readJsonl — skip corrupt lines, never fail.
pub(crate) fn read_jsonl(file: &Path) -> Vec<Value> {
    let Ok(bytes) = std::fs::read(file) else { return Vec::new() };
    let text = String::from_utf8_lossy(&bytes);
    text.split(['\n'])
        .map(|l| l.trim_end_matches('\r').trim())
        .filter(|l| !l.is_empty())
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .collect()
}

/// A JS-Set primitive key (SameValueZero over the truthy primitives bee logs).
pub(crate) fn primitive_key(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(format!("s:{s}")),
        Value::Number(n) => Some(format!("n:{}", jsjson::js_f64_to_string(n.as_f64()?))),
        Value::Bool(b) => Some(format!("b:{b}")),
        _ => None,
    }
}

// ─── state / pipeline reads ────────────────────────────────────────────────

pub(crate) struct Record {
    pub(crate) phase: Value,
    pub(crate) mode: Value,
    pub(crate) gates: Map<String, Value>,
}

/// state.mjs readState, reduced to the phase/mode/gates slice this hook uses.
///
/// Inlined rather than delegated to `crate::state::read_state_brief` because
/// that reader PRINTS its corrupt-JSON warning immediately, and this hook must
/// QUEUE its warnings — a delegating run (PreCompact) has to emit zero bytes
/// first. Both readers agree on the outcome: readJson's `null` fallback →
/// defaultState(). The exotic-gates arm below still delegates here, where
/// state.rs now spreads it natively; this hook keeps the delegate because it
/// still has a live delegate path to take.
pub(crate) fn read_state_record(root: &Path) -> Result<Record, Flow> {
    let defaults = || Record {
        phase: Value::String("idle".into()),
        mode: Value::Null,
        gates: default_gates(),
    };
    let file = root.join(".bee").join("state.json");
    let ReadJson::Parsed(Value::Object(state)) = read_json_failopen(&file) else {
        // Missing, corrupt (warned) or a non-object parse — all defaultState().
        return Ok(defaults());
    };
    // approved_gates: { ...defaults, ...(state.approved_gates || {}) }.
    let gates = match state.get("approved_gates") {
        None | Some(Value::Null) | Some(Value::Bool(false)) => default_gates(),
        Some(Value::String(s)) if s.is_empty() => default_gates(),
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => default_gates(),
        Some(Value::Object(overlay)) => {
            let mut merged = default_gates();
            for (k, v) in overlay {
                merged.insert(k.clone(), v.clone());
            }
            merged
        }
        // A truthy non-object spreads exotic key sets in JS — delegate.
        Some(_) => return Err(Flow::Delegate),
    };
    let mut phase = state.get("phase").cloned().unwrap_or(Value::String("idle".into()));
    if phase == Value::String("validating".into()) {
        phase = Value::String("planning".into()); // coerceLegacyPhase (D13)
    }
    Ok(Record { phase, mode: state.get("mode").cloned().unwrap_or(Value::Null), gates })
}

pub(crate) fn default_gates() -> Map<String, Value> {
    let mut m = Map::new();
    for g in ["context", "shape", "execution", "review"] {
        m.insert(g.into(), Value::Bool(false));
    }
    m
}

/// state.mjs laneRecordFrom — null when not a lane record for this feature.
pub(crate) fn lane_record_from(feature: &str, parsed: &Value) -> Result<Option<Record>, Flow> {
    let Value::Object(obj) = parsed else { return Ok(None) };
    if obj.get("feature").and_then(Value::as_str) != Some(feature) {
        return Ok(None);
    }
    let gates = match obj.get("approved_gates") {
        None | Some(Value::Null) | Some(Value::Bool(false)) => default_gates(),
        Some(Value::String(s)) if s.is_empty() => default_gates(),
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => default_gates(),
        Some(Value::Object(overlay)) => {
            let mut merged = default_gates();
            for (k, v) in overlay {
                merged.insert(k.clone(), v.clone());
            }
            merged
        }
        Some(_) => return Err(Flow::Delegate), // JS spread exotica
    };
    let mut phase = obj.get("phase").cloned().unwrap_or(Value::String("idle".into()));
    if phase == Value::String("validating".into()) {
        phase = Value::String("planning".into()); // coerceLegacyPhase (D13)
    }
    Ok(Some(Record {
        phase,
        mode: obj.get("mode").cloned().unwrap_or(Value::Null),
        gates,
    }))
}

pub(crate) enum Pipeline {
    Ok { record: Record },
    /// LANE_INVALID / LANE_MISSING / LANE_CORRUPT — callers on the Stop path
    /// only ever branch on ok-ness and fall back to readState.
    Refused,
}

/// state.mjs controlRootFor: mainRoot for a linked worktree, root otherwise;
/// linked-invalid THROWS (WorktreeLinkInvalidError) in the Node lib.
pub(crate) fn control_root(root: &Path, ctx: &HookContext) -> PathBuf {
    if ctx.worktree_resolution == "linked-valid" {
        ctx.control_root.clone().unwrap_or_else(|| root.to_path_buf())
    } else {
        root.to_path_buf()
    }
}

pub(crate) fn resolve_pipeline(
    root: &Path,
    ctx: &HookContext,
    session_id: Option<&str>,
    stderr: &mut String,
) -> Result<Pipeline, Flow> {
    let Some(sid) = session_id else {
        return Ok(Pipeline::Ok { record: read_state_record(root)? });
    };
    if ctx.worktree_resolution == "linked-invalid" {
        // state.mjs resolveRootsCore throws WorktreeLinkInvalidError here.
        return Err(Flow::Crash(format!(
            "WorktreeLinkInvalidError: linked worktree link is invalid ({})",
            root.join(".git").display()
        )));
    }
    let ctl = control_root(root, ctx);
    // readSession (fail-open id validation).
    if !is_plain_id(sid) {
        return Ok(Pipeline::Ok { record: read_state_record(root)? });
    }
    let session_file = ctl.join(".bee").join("sessions").join(format!("{sid}.json"));
    // Corrupt → warned, then reads as "no session" (readJson's null fallback
    // through readSession's `!session || session.id !== id` guard).
    let session = match read_json_failopen(&session_file) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => unreachable!("read_json_failopen never returns Corrupt"),
        ReadJson::Parsed(Value::Object(m)) => {
            if m.get("id").and_then(Value::as_str) == Some(sid) {
                Some(m)
            } else {
                None
            }
        }
        ReadJson::Parsed(_) => None,
    };
    let Some(session) = session else {
        return Ok(Pipeline::Ok { record: read_state_record(root)? });
    };
    let bound = session
        .get("lane")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);
    let Some(bound) = bound else {
        return Ok(Pipeline::Ok { record: read_state_record(root)? });
    };
    if !is_plain_id(&bound) {
        return Ok(Pipeline::Refused); // LANE_INVALID
    }
    let lane_file = ctl.join(".bee").join("lanes").join(format!("{bound}.json"));
    if !lane_file.exists() {
        return Ok(Pipeline::Refused); // LANE_MISSING
    }
    // A corrupt lane gets BOTH of Node's lines: readJson's (queued above by
    // read_json_failopen) and then readLane's own, via the None arm below —
    // laneRecordFrom(null) is null just like a shape-wrong record.
    match read_json_failopen(&lane_file) {
        ReadJson::Corrupt => unreachable!("read_json_failopen never returns Corrupt"),
        ReadJson::Missing => {
            // Vanished mid-read, or corrupt: laneRecordFrom(null) → readLane's
            // warn → LANE_CORRUPT. (A genuinely absent file was already
            // short-circuited by the exists() check above.)
            let rel = format!(".bee{s}lanes{s}{bound}.json", s = std::path::MAIN_SEPARATOR);
            stderr.push_str(&format!(
                "readLane: skipping corrupt lane record \"{rel}\" for display — mutations \
through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \
\"git checkout -- {rel}\").\n"
            ));
            Ok(Pipeline::Refused)
        }
        ReadJson::Parsed(parsed) => match lane_record_from(&bound, &parsed)? {
            Some(record) => Ok(Pipeline::Ok { record }),
            None => {
                // readLane's deterministic console.warn (corrupt lane shape).
                let rel = format!(".bee{s}lanes{s}{bound}.json", s = std::path::MAIN_SEPARATOR);
                stderr.push_str(&format!(
                    "readLane: skipping corrupt lane record \"{rel}\" for display — mutations \
through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \
\"git checkout -- {rel}\").\n"
                ));
                Ok(Pipeline::Refused) // LANE_CORRUPT
            }
        },
    }
}

/// state.mjs readHandoff, truthiness only. A corrupt HANDOFF.json warns and
/// reads as absent — so the "hive door open" mid-phase warning still fires,
/// which is the conservative half of readJson's fail-open here.
pub(crate) fn read_handoff_truthy(root: &Path) -> Result<bool, Flow> {
    match read_json_failopen(&root.join(".bee").join("HANDOFF.json")) {
        ReadJson::Missing => Ok(false),
        ReadJson::Corrupt => unreachable!("read_json_failopen never returns Corrupt"),
        ReadJson::Parsed(v) => Ok(js_truthy(&v)),
    }
}

// ─── inject.mjs dedup cache ────────────────────────────────────────────────

pub(crate) fn inject_cache_path(root: &Path) -> PathBuf {
    root.join(".bee").join("cache").join("inject-cache.json")
}

pub(crate) fn legacy_inject_cache_path(root: &Path) -> PathBuf {
    root.join(".bee").join(".inject-cache.json")
}

/// readJson(new, null) || readJson(legacy, {}) || {} — object caches only. A
/// corrupt cache warns and reads as absent, so the chain falls through to the
/// legacy file and then to `{}` (the nudge simply re-injects once). Non-object
/// parses still delegate; they are screened in preflight and re-checked here.
pub(crate) fn read_inject_cache(root: &Path) -> Result<Map<String, Value>, Flow> {
    for (file, missing_falls_through) in
        [(inject_cache_path(root), true), (legacy_inject_cache_path(root), false)]
    {
        match read_json_failopen(&file) {
            ReadJson::Corrupt => unreachable!("read_json_failopen never returns Corrupt"),
            ReadJson::Missing => {
                if !missing_falls_through {
                    return Ok(Map::new());
                }
            }
            ReadJson::Parsed(Value::Object(m)) => return Ok(m),
            // A parsed falsy value (null/false) falls through like "missing";
            // any other non-object shape is JS exotica — delegate.
            ReadJson::Parsed(Value::Null) | ReadJson::Parsed(Value::Bool(false)) => {
                if !missing_falls_through {
                    return Ok(Map::new());
                }
            }
            ReadJson::Parsed(_) => return Err(Flow::Delegate),
        }
    }
    Ok(Map::new())
}

pub(crate) fn should_inject(root: &Path, key: &str, hash: &str) -> Result<bool, Flow> {
    let cache = read_inject_cache(root)?;
    let Some(entry) = cache.get(key) else { return Ok(true) };
    if !js_truthy(entry) {
        return Ok(true);
    }
    if entry.get("hash").and_then(Value::as_str) != Some(hash) {
        return Ok(true);
    }
    let at = entry.get("at").cloned().unwrap_or(Value::Null);
    let Some(last_ms) = js_date_parse_value(&at) else { return Ok(true) };
    Ok(now_ms() - last_ms > INJECT_INTERVAL_MS)
}

pub(crate) fn mark_injected(root: &Path, key: &str, hash: &str) -> Result<(), Flow> {
    let mut cache = read_inject_cache(root)?;
    let mut entry = Map::new();
    entry.insert("hash".into(), Value::String(hash.to_string()));
    entry.insert("at".into(), Value::String(now_iso()));
    cache.insert(key.to_string(), Value::Object(entry));
    let _ = crate::fsutil::write_json_atomic(&inject_cache_path(root), &Value::Object(cache));
    crate::fsutil::remove_file_if_exists(&legacy_inject_cache_path(root));
    Ok(())
}

// ─── decisions.mjs newest active decision ──────────────────────────────────

/// activeDecisions(root, {recent:1})[0] reduced to (id, date). The tag
/// overlay (dp-5) never touches id/date, so it is irrelevant here.
pub(crate) fn newest_active_decision(root: &Path) -> Option<(Value, Value)> {
    let events = read_jsonl(&root.join(".bee").join("decisions.jsonl"));
    let mut superseded: HashSet<String> = HashSet::new();
    let mut redacted: HashSet<String> = HashSet::new();
    for event in &events {
        if event.get("type").and_then(Value::as_str) == Some("supersede") {
            if let Some(target) = event.get("supersedes").filter(|v| js_truthy(v)) {
                if let Some(k) = primitive_key(target) {
                    superseded.insert(k);
                }
            }
        }
        if event.get("type").and_then(Value::as_str) == Some("redact") {
            if let Some(target) = event.get("redacts").filter(|v| js_truthy(v)) {
                if let Some(k) = primitive_key(target) {
                    redacted.insert(k);
                }
            }
        }
    }
    let mut newest: Option<&Value> = None;
    for event in &events {
        let ty = event.get("type").and_then(Value::as_str);
        if !matches!(ty, Some("decide") | Some("supersede")) {
            continue;
        }
        let id_key = event.get("id").and_then(primitive_key);
        if let Some(k) = &id_key {
            if superseded.contains(k) || redacted.contains(k) {
                continue;
            }
        }
        newest = Some(event);
    }
    newest.map(|e| {
        (e.get("id").cloned().unwrap_or(Value::Null), e.get("date").cloned().unwrap_or(Value::Null))
    })
}
