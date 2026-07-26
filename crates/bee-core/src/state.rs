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
