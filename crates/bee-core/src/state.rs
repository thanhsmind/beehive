//! state — reader for the projected `.bee/state.json`, ported from
//! `.bee/bin/lib/state.mjs`'s `readState`/`defaultState`/`gateApproved`
//! (rust-port-8, CONTEXT.md D3). Read-only, zero subprocess (D5).
//!
//! `.bee/bin/lib/state.mjs` is FROZEN for the duration of the rust-port
//! feature (D1). This reader mirrors `readState`'s fail-open posture
//! exactly: a missing or unparseable `state.json` reads as
//! [`default_state`], never a panic or an error — the same posture the
//! mjs source documents as load-bearing (hooks and `bee.mjs status` read
//! this file constantly and must never throw on a corrupt file mid-session).
//! `readStateStrict`'s throwing variant (the CLI mutation path) is out of
//! scope here — this cell's consumers (rust-port-9..12) are read-only
//! guard-support checks, never state mutators.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::fsutil::read_json;

pub fn state_path(root: &Path) -> PathBuf {
    root.join(".bee").join("state.json")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovedGates {
    #[serde(default)]
    pub context: bool,
    #[serde(default)]
    pub shape: bool,
    #[serde(default)]
    pub execution: bool,
    #[serde(default)]
    pub review: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for ApprovedGates {
    fn default() -> Self {
        ApprovedGates {
            context: false,
            shape: false,
            execution: false,
            review: false,
            extra: Map::new(),
        }
    }
}

/// The projected `.bee/state.json` shape: `{phase, feature, mode,
/// approved_gates, workers, summary, next_action, ...}`. Every field this
/// reader does not name explicitly survives round-trip via `extra` (D3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default = "default_phase")]
    pub phase: String,
    #[serde(default)]
    pub feature: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub approved_gates: ApprovedGates,
    #[serde(default)]
    pub workers: Vec<Value>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub next_action: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_schema_version() -> String {
    "1.0".to_string()
}

fn default_phase() -> String {
    "idle".to_string()
}

/// `defaultState()` — the fresh/idle skeleton every fail-open read falls
/// back to.
pub fn default_state() -> State {
    State {
        schema_version: default_schema_version(),
        phase: default_phase(),
        feature: None,
        mode: None,
        approved_gates: ApprovedGates::default(),
        workers: Vec::new(),
        summary: String::new(),
        next_action: "No active bee work — awaiting a user request.".to_string(),
        extra: Map::new(),
    }
}

/// `readState(root)`: a missing/malformed `state.json` falls open to
/// [`default_state`]; a present-but-partial object is merged over the
/// defaults field-by-field (matching mjs's `{...defaultState(), ...state}`
/// plus the `approved_gates` sub-merge), so an old record missing a newer
/// gate name still reads that gate as `false` rather than losing the whole
/// record.
pub fn read_state(root: &Path) -> State {
    let raw: Value = read_json(&state_path(root), Value::Null);
    if !raw.is_object() {
        return default_state();
    }
    match serde_json::from_value::<State>(raw) {
        Ok(state) => state,
        Err(_) => default_state(),
    }
}

// ─── lanes + pipeline resolution (rust-port-9) ──────────────────────────────
// Ports of state.mjs's `lanesDir`/`lanePath`/`readLane`/`resolvePipeline`,
// the slice guards.mjs's `checkWrite` needs through `resolveWriteRecord`:
// a bound sessionId is governed by its lane record, an unbound/unknown
// session by the default record, and an unresolvable binding is a typed
// refusal reason — never a silent fallback to the default pipeline.

pub fn lanes_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("lanes")
}

/// `requireLaneFeature` + `lanePath`: the feature becomes a filename under
/// `.bee/lanes/`, so path separators and `..` are bad arguments (Err with
/// the mjs error message verbatim — resolvePipeline interpolates it).
pub fn lane_path(root: &Path, feature: &str) -> Result<PathBuf, String> {
    let trimmed = feature.trim();
    if trimmed.is_empty() {
        return Err("lane feature is required.".to_string());
    }
    if trimmed.contains('\\') || trimmed.contains('/') || trimmed.contains("..") {
        return Err("lane feature must be a plain id (no path separators).".to_string());
    }
    Ok(lanes_dir(root).join(format!("{trimmed}.json")))
}

/// `readLane` (fail-open DISPLAY read): `None` when missing or corrupt; a
/// present-but-corrupt record is WARNED to stderr exactly like the mjs
/// source's console.warn, then read as `None`. `laneRecordFrom`'s
/// feature-match guard is preserved: a record whose `feature` field does not
/// match the requested lane is corrupt, never trusted.
pub fn read_lane(root: &Path, feature: &str) -> Option<State> {
    let file = lane_path(root, feature).ok()?;
    if !file.exists() {
        return None;
    }
    let trimmed = feature.trim();
    let raw: Value = read_json(&file, Value::Null);
    let record = if raw.is_object() {
        match serde_json::from_value::<State>(raw) {
            Ok(state) if state.feature.as_deref() == Some(trimmed) => Some(state),
            _ => None,
        }
    } else {
        None
    };
    if record.is_none() {
        let rel = format!(".bee/lanes/{trimmed}.json");
        eprintln!(
            "readLane: skipping corrupt lane record \"{rel}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
        );
    }
    record
}

/// The `{ok: true}` arm of `resolvePipeline`.
pub struct PipelineResolution {
    /// `'default'` or `'lane'` (msn-21: lane flows opt out of the
    /// workspace-ownership check in guards.mjs `checkWrite`).
    pub source: &'static str,
    pub record: State,
}

/// Port of state.mjs `resolvePipeline(root, {sessionId})`. `Err(reason)`
/// carries the typed refusal reason (LANE_INVALID / LANE_MISSING /
/// LANE_CORRUPT) verbatim — the caller (`check_write`) prefixes it with
/// "bee lane guard: ".
pub fn resolve_pipeline(root: &Path, session_id: Option<&str>) -> Result<PipelineResolution, String> {
    let defaults = || PipelineResolution { source: "default", record: read_state(root) };
    let Some(sid) = session_id.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(defaults());
    };
    // Sessions and lanes are coordination-store state; the caller passes the
    // control root (mjs re-derives it via controlRootFor — the hook's
    // topology already resolved it once, msn-21).
    let Some(session) = crate::claims::read_session(root, sid) else {
        return Ok(defaults());
    };
    let bound = session
        .extra
        .get("lane")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if bound.is_empty() {
        return Ok(defaults());
    }
    let file = match lane_path(root, &bound) {
        Ok(f) => f,
        Err(err) => {
            return Err(format!(
                "session \"{id}\" is bound to lane \"{bound}\", which is not a valid lane name ({err}) — never guessed back to the default pipeline. FIX: rebind or unbind the session (claims.mjs bindSessionLane/unbindSessionLane).",
                id = session.id,
            ));
        }
    };
    if !file.exists() {
        return Err(format!(
            "session \"{id}\" is bound to lane \"{bound}\" but .bee/lanes/{bound}.json does not exist — resolution never guesses back to the default pipeline. FIX: start the lane (startFeature with lane mode) or unbind the session.",
            id = session.id,
        ));
    }
    let Some(record) = read_lane(root, &bound) else {
        return Err(format!(
            "session \"{id}\" is bound to lane \"{bound}\" but its record is corrupt — display never guesses and mutations must refuse. FIX: inspect/restore .bee/lanes/{bound}.json, then retry.",
            id = session.id,
        ));
    };
    Ok(PipelineResolution { source: "lane", record })
}

/// `gateApproved(state, gateName)` for the four named gates.
pub fn gate_approved(state: &State, gate_name: &str) -> bool {
    match gate_name {
        "context" => state.approved_gates.context,
        "shape" => state.approved_gates.shape,
        "execution" => state.approved_gates.execution,
        "review" => state.approved_gates.review,
        other => matches!(state.approved_gates.extra.get(other), Some(Value::Bool(true))),
    }
}


// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
