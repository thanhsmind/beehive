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

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::fsutil::{read_json, write_json_atomic};

/// The four bee gate names, in their canonical output order — mirrors
/// `state.mjs`'s exported `GATE_NAMES` (rust-port-16, CONTEXT.md D3/D9).
pub const GATE_NAMES: [&str; 4] = ["context", "shape", "execution", "review"];

/// state.mjs `BEE_VERSION` — kept in sync BY HAND at port time (D1 freeze:
/// the mjs source is the oracle for this value, never the other way
/// around). Read by [`compute_runtime_drift`]'s version-drift check.
pub const BEE_VERSION: &str = "1.18.2";

/// state.mjs `PHASES` — the phase enum's non-terminal members, in the
/// mjs source's own declared order (used verbatim in the
/// `isKnownPhase` staleness-warning message).
pub const PHASES: [&str; 9] =
    ["idle", "exploring", "planning", "validating", "swarming", "reviewing", "scribing", "compounding", "grooming"];

/// state.mjs `KNOWN_PHASES` — [`PHASES`] plus the one blessed terminal
/// alias, `"compounding-complete"`.
pub const KNOWN_PHASES: [&str; 10] = [
    "idle",
    "exploring",
    "planning",
    "validating",
    "swarming",
    "reviewing",
    "scribing",
    "compounding",
    "grooming",
    "compounding-complete",
];

/// `isKnownPhase(phase)`.
pub fn is_known_phase(phase: &str) -> bool {
    KNOWN_PHASES.contains(&phase)
}

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
            Ok(mut state) if state.feature.as_deref() == Some(trimmed) => {
                // Deviation (rust-port-20, found while oracle-diffing
                // `build_lane_rows` against the real mjs `status --json`
                // output): `laneRecordFrom` merges `defaultLaneRecord(
                // feature)` UNDER the parsed record, and that default
                // carries `created_at: null` — a lane file written before
                // that key existed (or written by hand) must still read
                // with the key PRESENT (null), never absent. The `State`
                // struct has no named `created_at` field (deliberately —
                // plain `state.json`'s own `defaultState()` has no such
                // key, and this struct is shared with `read_state`), so
                // the default is injected into `extra` here instead,
                // scoped to lane reads only.
                state.extra.entry("created_at".to_string()).or_insert(Value::Null);
                Some(state)
            }
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
    let bound = session.lane.as_deref().map(str::trim).unwrap_or("").to_string();
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

/// `writeState(root, state)` — atomic write-through, no locking of its own
/// (callers that need read-modify-write safety take `workflow:<id>` or a
/// store lock around the whole cycle; this mirrors `writeState`'s own
/// unlocked shape in the mjs source).
pub fn write_state(root: &Path, state: &State) -> io::Result<()> {
    write_json_atomic(&state_path(root), state)
}

/// `writeLane(root, lane)` — atomic write-through to
/// `.bee/lanes/<lane.feature>.json`. `lane.feature` must be set (the mjs
/// source derives the path from `lane.feature` the same way); an absent
/// feature is a caller bug, reported as an `InvalidInput` io error rather
/// than silently writing to a nonsense path.
pub fn write_lane(root: &Path, lane: &State) -> io::Result<()> {
    let feature = lane
        .feature
        .as_deref()
        .filter(|f| !f.trim().is_empty())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "writeLane: lane.feature is required."))?;
    let file = lane_path(root, feature).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    write_json_atomic(&file, lane)
}

// ─── handoff (rust-port-16) ──────────────────────────────────────────────
// Read-only surface `state-projection.mjs`'s `rebuildHandoffProjection`
// needs: the legacy single-file path/normalization, plus the per-workflow
// mailbox listing. Mutators (`writeMailboxHandoff`/`adoptMailboxHandoff`)
// are out of this cell's scope (rust-port-16's action list names only the
// state-projection.mjs rebuild verbs) and stay unported.

pub fn handoff_path(root: &Path) -> PathBuf {
    root.join(".bee").join("HANDOFF.json")
}

/// `normalizeHandoffKind(kind)` — missing/unknown -> `"pause"` (the
/// fail-safe: surface-and-wait is the safe default), `"planned-next"`
/// passes through verbatim.
pub fn normalize_handoff_kind(kind: Option<&str>) -> &'static str {
    if kind == Some("planned-next") {
        "planned-next"
    } else {
        "pause"
    }
}

/// One record from a workflow's handoff mailbox
/// (`.bee/runtime/handoffs/<workflow-id>/<seq>.json`). `seq` is derived
/// from the filename (never persisted inside the body) and `fields` is the
/// record body with `kind` already normalized, matching
/// `listHandoffMailbox`'s in-memory `{...raw, kind, seq}` shape.
#[derive(Debug, Clone)]
pub struct HandoffMailboxRecord {
    pub seq: i64,
    pub fields: Map<String, Value>,
}

pub fn handoff_mailbox_dir(root: &Path, workflow_id: &str) -> PathBuf {
    root.join(".bee").join("runtime").join("handoffs").join(workflow_id)
}

/// `Number.parseInt(str, 10)` on a `.json`-stripped filename stem: leading
/// optional sign, then leading decimal digits; trailing garbage is
/// ignored; no leading digit at all is `None` (`NaN`, which
/// `listHandoffMailbox`'s `Number.isFinite` guard then skips).
fn parse_leading_int(s: &str) -> Option<i64> {
    let bytes = s.as_bytes();
    let mut idx = 0;
    if idx < bytes.len() && (bytes[idx] == b'+' || bytes[idx] == b'-') {
        idx += 1;
    }
    let digits_start = idx;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        idx += 1;
    }
    if idx == digits_start {
        return None;
    }
    s[..idx].parse::<i64>().ok()
}

/// `listHandoffMailbox(root, workflowId)` — every record in a workflow's
/// mailbox, oldest to newest by seq. Fail-open: a missing/unreadable
/// directory reads as `[]`; a record that is missing, unparseable, or not
/// a JSON object is skipped rather than failing the whole listing.
pub fn list_handoff_mailbox(root: &Path, workflow_id: &str) -> Vec<HandoffMailboxRecord> {
    let dir = handoff_mailbox_dir(root, workflow_id);
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut records = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".json") else { continue };
        let Some(seq) = parse_leading_int(stem) else { continue };
        let raw: Value = read_json(&entry.path(), Value::Null);
        let Value::Object(mut fields) = raw else { continue };
        let kind = fields.get("kind").and_then(Value::as_str);
        fields.insert("kind".to_string(), Value::String(normalize_handoff_kind(kind).to_string()));
        records.push(HandoffMailboxRecord { seq, fields });
    }
    records.sort_by_key(|r| r.seq);
    records
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

// ─── status readers B2 (rust-port-20, CONTEXT.md D3/D5) ─────────────────────
// The remaining state.mjs readers `buildStatus` calls that rust-port-8/16
// left unported: `readOnboarding`, `readHandoff` (bare display read —
// distinct from the mailbox-projection helpers above), `bypassLevel`,
// `hasStaleAdvisorKey`, `validateModelsConfig`/`validateAgentFilesDrift`
// (config-validate, ao-2ai-1), plus the seven bee.mjs-PRIVATE helpers this
// cell's panel named explicitly: `buildLaneRows`/`buildLaneSummary`,
// `computeRuntimeDrift`, `findRepoHive`, `ungrantedWorktreeNotice` (the
// eighth and ninth — `buildRecoveryBlock`/`buildContentionSummary` — live in
// `crate::recovery`, alongside `readTranscriptTail`).

/// `readOnboarding(root)`.
pub fn read_onboarding(root: &Path) -> Value {
    read_json(&root.join(".bee").join("onboarding.json"), Value::Null)
}

/// `readHandoff(root)` — the fail-open DISPLAY read (distinct from the
/// mailbox-projection helpers above): a missing/malformed file, or any
/// non-object JSON body, passes through `readJson`'s result verbatim
/// (mirrors the mjs source's `if (!handoff || typeof handoff !== 'object'
/// || Array.isArray(handoff)) return handoff;` early return); an object
/// gets its `kind` normalized via [`normalize_handoff_kind`].
pub fn read_handoff(root: &Path) -> Value {
    let raw: Value = read_json(&handoff_path(root), Value::Null);
    match raw {
        Value::Object(mut map) => {
            let kind = map.get("kind").and_then(Value::as_str).map(str::to_string);
            map.insert("kind".to_string(), Value::String(normalize_handoff_kind(kind.as_deref()).to_string()));
            Value::Object(map)
        }
        other => other,
    }
}

/// state.mjs `BYPASS_LEVELS`.
pub const BYPASS_LEVELS: [&str; 4] = ["off", "normal", "full", "total"];

/// `bypassLevel(root)` — reads `readConfig(root).gate_bypass`, which is
/// the OVERLAY-MERGED value (`.bee/config.local.json` wins over the
/// tracked file for every namespace, gate_bypass included). Deliberately
/// NOT [`crate::config::Config::bypass_level`] (that method reads the
/// TRACKED file only — a documented, narrower scope for the guard-support
/// readers rust-port-8/9 built it for); this function reuses
/// [`crate::config::read_config_value`]'s overlay-aware merge so a repo
/// that sets `gate_bypass` only in the local overlay is read identically
/// to the real `bee.mjs status`.
pub fn bypass_level(root: &Path) -> &'static str {
    let raw = crate::config::read_config_value(root);
    match raw.get("gate_bypass") {
        Some(Value::String(s)) if s == "total" => "total",
        Some(Value::String(s)) if s == "full" => "full",
        Some(Value::Bool(true)) => "normal",
        Some(Value::String(s)) if s == "on" || s == "normal" => "normal",
        _ => "off",
    }
}

/// state.mjs `STALE_ADVISOR_KEY_WARNING`.
pub const STALE_ADVISOR_KEY_WARNING: &str = "advisor mode was removed in 0.1.23; the top-level advisor key in .bee/config.json is ignored — delete it. (This does not affect the models.<runtime>.advisor slot, which is separate and still valid.)";

/// `hasStaleAdvisorKey(root)`.
pub fn has_stale_advisor_key(root: &Path) -> bool {
    let raw: Value = read_json(&root.join(".bee").join("config.json"), Value::Null);
    matches!(&raw, Value::Object(map) if map.contains_key("advisor"))
}

// ─── config-validate (ao-2ai-1) ──────────────────────────────────────────

/// `readRawConfigForValidation(root)`'s three-way contract:
/// `Absent` = no `.bee/config.json` file at all (JS `undefined` —
/// `validateModelsConfig` reports zero problems for this case: nothing
/// configured is not an error); `Value(v)` = the file exists and was read
/// (`v` may itself be `Value::Null` when the content is malformed JSON —
/// JS `readJson(file, null)`'s fallback — which `validateModelsConfig`
/// DOES report as a problem, unlike the `Absent` case). A plain
/// `Option<Value>` cannot distinguish these two `None`-shaped JS values
/// (`undefined` vs a read `null`), hence this dedicated enum.
#[derive(Debug, Clone)]
pub enum RawConfig {
    Absent,
    Value(Value),
}

/// `readRawConfigForValidation(root)`.
pub fn read_raw_config_for_validation(root: &Path) -> RawConfig {
    let file = crate::config::config_path(root);
    if file.exists() {
        RawConfig::Value(read_json(&file, Value::Null))
    } else {
        RawConfig::Absent
    }
}

/// One row of `validateModelsConfig`'s return array. `runtime`/`slot` are
/// ALWAYS present in the mjs shape (nullable, never omitted); `flag` is
/// present only for the two flag-blocklist problem codes (`cli-unsafe-flag`,
/// `cli-advice-slot-writable`) — omitted (via `skip_serializing_if`)
/// everywhere else, matching JS's undefined-key-drop-on-JSON.stringify.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelConfigProblem {
    pub code: String,
    pub runtime: Option<String>,
    pub slot: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag: Option<String>,
}

fn problem(code: &str, runtime: Option<&str>, slot: Option<&str>, message: String) -> ModelConfigProblem {
    ModelConfigProblem {
        code: code.to_string(),
        runtime: runtime.map(str::to_string),
        slot: slot.map(str::to_string),
        message,
        flag: None,
    }
}

const MODEL_VALIDATE_SLOTS: [&str; 4] = ["extraction", "generation", "review", "advisor"];

/// ao-2b-2/AO8 — advice-class slots (every runtime): must run read-only.
pub const ADVICE_CLASS_SLOTS: [&str; 2] = ["advisor", "review"];

/// A BLOCKLIST of known-bad auto-approve/sandbox-bypass flags — NOT a
/// positive read-only guarantee (state.mjs `UNSAFE_CLI_FLAGS`).
pub const UNSAFE_CLI_FLAGS: [&str; 6] = [
    "--yolo",
    "--dangerously-skip-permissions",
    "--dangerously-bypass-approvals-and-sandbox",
    "--full-auto",
    "-s danger-full-access",
    "--sandbox danger-full-access",
];

/// A second, NARROWER blocklist layered on top of [`UNSAFE_CLI_FLAGS`] for
/// [`ADVICE_CLASS_SLOTS`] only (state.mjs `ADVICE_CLASS_WRITABLE_TOKENS`).
pub const ADVICE_CLASS_WRITABLE_TOKENS: [&str; 4] =
    ["-s workspace-write", "--sandbox workspace-write", "--sandbox=workspace-write", "danger-full-access"];

/// `JSON.stringify` of a single already-parsed JSON value — used only for
/// the `fork_turns` value interpolated into a problem message, matching
/// mjs's `${JSON.stringify(value.fork_turns)}` exactly (both produce the
/// same compact JSON text for any JSON value).
fn js_json_stringify(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

/// `validateModelsConfig(config)` (ao-2ai-1) — see the mjs source's own
/// doc comment (state.mjs:390-421) for the full check list; ported here
/// check-for-check, in the same order, against the SAME raw (never
/// normalized) config shape.
pub fn validate_models_config(raw: &RawConfig) -> Vec<ModelConfigProblem> {
    let mut problems = Vec::new();
    let config = match raw {
        RawConfig::Absent => return problems,
        RawConfig::Value(v) => v,
    };
    let Value::Object(config_obj) = config else {
        problems.push(problem(
            "config-malformed",
            None,
            None,
            ".bee/config.json content is null or not an object — models config cannot be validated; defaults apply."
                .to_string(),
        ));
        return problems;
    };
    let Some(models) = config_obj.get("models") else { return problems };
    let Value::Object(models_obj) = models else {
        problems.push(problem(
            "config-malformed",
            None,
            None,
            "`models` in .bee/config.json is present but not an object — ignored; defaults apply.".to_string(),
        ));
        return problems;
    };
    for rt in crate::config::RUNTIMES {
        let Some(src) = models_obj.get(rt) else { continue };
        let Value::Object(src_obj) = src else {
            problems.push(problem(
                "runtime-malformed",
                Some(rt),
                None,
                format!("models.{rt} is present but not an object — ignored; defaults apply."),
            ));
            continue;
        };
        for slot in MODEL_VALIDATE_SLOTS {
            let Some(value) = src_obj.get(slot) else { continue };
            if value.is_null() {
                continue;
            }
            if value.is_string() {
                continue;
            }
            let Value::Object(obj) = value else {
                problems.push(problem(
                    "slot-value-malformed",
                    Some(rt),
                    Some(slot),
                    format!("models.{rt}.{slot} is not a string, object, or null — ignored; defaults apply."),
                ));
                continue;
            };
            let is_composite =
                obj.contains_key("primary") || obj.contains_key("fallback") || obj.contains_key("fallback_policy");
            if is_composite {
                let primary_obj = obj.get("primary").and_then(Value::as_object);
                let primary_ok = primary_obj
                    .map(|p| {
                        p.get("kind").and_then(Value::as_str) == Some("native")
                            && p.get("model").and_then(Value::as_str).map(|m| !m.trim().is_empty()).unwrap_or(false)
                    })
                    .unwrap_or(false);
                if !primary_ok {
                    problems.push(problem(
                        "composite-primary-malformed",
                        Some(rt),
                        Some(slot),
                        format!(
                            "models.{rt}.{slot} is a composite (primary/fallback) but its primary is not a valid native override {{kind:\"native\", model}} — ignored; today this silently reverts to the seeded default (D2)."
                        ),
                    ));
                    continue;
                }
                let primary_obj = primary_obj.unwrap();
                if let Some(fork_turns) = primary_obj.get("fork_turns") {
                    if fork_turns.as_str() != Some("none") {
                        problems.push(problem(
                            "native-fork-turns-unknown",
                            Some(rt),
                            Some(slot),
                            format!(
                                "models.{rt}.{slot} composite primary has fork_turns:{} — only \"none\" is valid; a full-history fork rejects model overrides (E2/D2).",
                                js_json_stringify(fork_turns)
                            ),
                        ));
                    }
                }
                if obj.get("fallback_policy").and_then(Value::as_str) != Some("explicit-only") {
                    problems.push(problem(
                        "composite-fallback-policy-missing",
                        Some(rt),
                        Some(slot),
                        format!(
                            "models.{rt}.{slot} is a composite but has no fallback_policy:\"explicit-only\" — its cli fallback is silently dropped and no fallback is ever taken; silent native->cli fallback is forbidden (D1). Set fallback_policy:\"explicit-only\" to opt in."
                        ),
                    ));
                    continue;
                }
                let fb_ok = obj
                    .get("fallback")
                    .and_then(Value::as_object)
                    .map(|fb| {
                        fb.get("kind").and_then(Value::as_str) == Some("cli")
                            && fb.get("command").and_then(Value::as_str).map(|c| !c.trim().is_empty()).unwrap_or(false)
                    })
                    .unwrap_or(false);
                if !fb_ok {
                    problems.push(problem(
                        "composite-fallback-malformed",
                        Some(rt),
                        Some(slot),
                        format!(
                            "models.{rt}.{slot} composite declares fallback_policy:\"explicit-only\" but its fallback is not a valid cli executor {{kind:\"cli\", command}} — the fallback is silently dropped; fix or remove it (D2)."
                        ),
                    ));
                }
                continue;
            }
            if obj.get("kind").and_then(Value::as_str) == Some("native") {
                let model_ok = obj.get("model").and_then(Value::as_str).map(|m| !m.trim().is_empty()).unwrap_or(false);
                if !model_ok {
                    problems.push(problem(
                        "native-model-missing",
                        Some(rt),
                        Some(slot),
                        format!(
                            "models.{rt}.{slot} is a native override (kind:\"native\") but has no non-empty model — the exact catalog model id is required; today this silently reverts to the seeded default (D2)."
                        ),
                    ));
                    continue;
                }
                if let Some(fork_turns) = obj.get("fork_turns") {
                    if fork_turns.as_str() != Some("none") {
                        problems.push(problem(
                            "native-fork-turns-unknown",
                            Some(rt),
                            Some(slot),
                            format!(
                                "models.{rt}.{slot} native override has fork_turns:{} — only \"none\" is valid; a full-history fork rejects model overrides (E2/D2).",
                                js_json_stringify(fork_turns)
                            ),
                        ));
                    }
                }
                continue;
            }
            let looks_like_cli = obj.contains_key("kind") || obj.contains_key("command");
            if looks_like_cli {
                let kind_ok = obj.get("kind").and_then(Value::as_str) == Some("cli");
                let command = obj.get("command").and_then(Value::as_str).unwrap_or("");
                let command_ok = !command.trim().is_empty();
                if !kind_ok || !command_ok {
                    problems.push(problem(
                        "cli-malformed",
                        Some(rt),
                        Some(slot),
                        format!(
                            "models.{rt}.{slot} looks like a cli executor but is missing kind:\"cli\" or a non-empty command — today this silently reverts to the seeded default; fix or remove it (W-e)."
                        ),
                    ));
                    continue;
                }
                let transport_ok =
                    obj.get("promptVia").and_then(Value::as_str).map(|p| !p.trim().is_empty()).unwrap_or(false);
                if !transport_ok {
                    problems.push(problem(
                        "cli-prompt-transport-missing",
                        Some(rt),
                        Some(slot),
                        format!(
                            "models.{rt}.{slot} is a cli executor with no declared prompt transport — set promptVia (e.g. \"stdin\") so the prompt reliably reaches it; never inferred from the command string (B2)."
                        ),
                    ));
                }
                for flag in UNSAFE_CLI_FLAGS {
                    if command.contains(flag) {
                        let mut p = problem(
                            "cli-unsafe-flag",
                            Some(rt),
                            Some(slot),
                            format!(
                                "models.{rt}.{slot} command contains \"{flag}\" — a known auto-approve/sandbox-bypass flag; remove it (B6/B7). This is a blocklist of KNOWN-BAD flags, not a positive read-only guarantee."
                            ),
                        );
                        p.flag = Some(flag.to_string());
                        problems.push(p);
                    }
                }
                if ADVICE_CLASS_SLOTS.contains(&slot) {
                    for token in ADVICE_CLASS_WRITABLE_TOKENS {
                        if command.contains(token) {
                            let mut p = problem(
                                "cli-advice-slot-writable",
                                Some(rt),
                                Some(slot),
                                format!(
                                    "models.{rt}.{slot} is an advice-class cli slot (advisor/review must run read-only, AO8) and its command contains \"{token}\" — a known write-granting sandbox token; remove it. This is a blocklist of KNOWN write-granting tokens, not a positive read-only guarantee."
                                ),
                            );
                            p.flag = Some(token.to_string());
                            problems.push(p);
                        }
                    }
                }
                continue;
            }
            let model_ok = obj.get("model").and_then(Value::as_str).map(|m| !m.trim().is_empty()).unwrap_or(false);
            if !model_ok {
                problems.push(problem(
                    "model-shape-malformed",
                    Some(rt),
                    Some(slot),
                    format!(
                        "models.{rt}.{slot} is an object but neither a valid cli executor nor a valid {{model}} shape — ignored; today this silently reverts to the seeded default."
                    ),
                ));
            }
        }
    }
    problems
}

// ─── W3 agent-file drift (ao-3b-2, AO12) ─────────────────────────────────

const AGENT_FILE_TIER: [(&str, &str); 3] =
    [("bee-gather", "generation"), ("bee-extract", "extraction"), ("bee-review", "review")];

/// One row of `validateAgentFilesDrift`'s return array.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AgentFileProblem {
    pub code: String,
    pub agent: String,
    pub slot: String,
    pub message: String,
}

/// `readAgentFileModel(file)`'s three outcomes, ported as an enum rather
/// than mjs's `{found, model}` shape (Rust's type system already
/// distinguishes "file missing" from "file present, no model line" more
/// cleanly than a two-field record with an optional-within-optional).
enum AgentFileModel {
    NotFound,
    Found(Option<String>),
}

/// Locates the first `\n---` (optionally `\r`-preceded) after the
/// frontmatter opener, mirroring the non-greedy `[\s\S]*?` capture in
/// `/^---\r?\n([\s\S]*?)\r?\n---/` — the first occurrence IS the
/// non-greedy match. Returns the END index of the captured frontmatter
/// body (excluding the closing delimiter's leading `\r?\n`).
fn find_frontmatter_terminator(s: &str) -> Option<usize> {
    let pos = s.find("\n---")?;
    let end = if pos > 0 && s.as_bytes()[pos - 1] == b'\r' { pos - 1 } else { pos };
    Some(end)
}

/// `readAgentFileModel(file)`: a read failure is [`AgentFileModel::NotFound`];
/// otherwise [`AgentFileModel::Found`] carrying `Some(model)` when a
/// `model:` frontmatter line was found, `None` when the file has no
/// frontmatter (or no `model:` line inside it) — mjs's `model: undefined`.
fn read_agent_file_model(file: &Path) -> AgentFileModel {
    let raw = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(_) => return AgentFileModel::NotFound,
    };
    let after_open = raw.strip_prefix("---\r\n").or_else(|| raw.strip_prefix("---\n"));
    let Some(after_open) = after_open else { return AgentFileModel::Found(None) };
    let Some(end) = find_frontmatter_terminator(after_open) else { return AgentFileModel::Found(None) };
    let frontmatter = &after_open[..end];
    for line in frontmatter.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if let Some(rest) = line.strip_prefix("model:") {
            return AgentFileModel::Found(Some(rest.trim().to_string()));
        }
    }
    AgentFileModel::Found(None)
}

/// `validateAgentFilesDrift(root, rawConfig)`. `rawConfig` is the SAME
/// [`RawConfig`] shape [`validate_models_config`] takes; only its `.models`
/// key (when `rawConfig` is a present object) feeds
/// [`crate::config::normalize_models`] — matching the mjs source's
/// "normalized here through the same `normalizeModels()` readConfig
/// itself uses" comment.
pub fn validate_agent_files_drift(root: &Path, raw: &RawConfig) -> Vec<AgentFileProblem> {
    let mut problems = Vec::new();
    let raw_models = match raw {
        RawConfig::Value(Value::Object(map)) => map.get("models").cloned().unwrap_or(Value::Null),
        _ => Value::Null,
    };
    let models = crate::config::normalize_models(&raw_models);
    for (agent_name, slot) in AGENT_FILE_TIER {
        let file = root.join(".claude").join("agents").join(format!("{agent_name}.md"));
        let file_model = match read_agent_file_model(&file) {
            AgentFileModel::NotFound => continue,
            AgentFileModel::Found(m) => m,
        };
        let Some(file_model) = file_model else {
            problems.push(AgentFileProblem {
                code: "agent-file-malformed".to_string(),
                agent: agent_name.to_string(),
                slot: slot.to_string(),
                message: format!(
                    ".claude/agents/{agent_name}.md has no readable \"model:\" frontmatter line — cannot check drift; re-run onboarding to re-render it."
                ),
            });
            continue;
        };
        let mut value = models.get("claude").and_then(|c| c.get(slot)).cloned();
        if matches!(value, None | Some(Value::Null)) && slot == "review" {
            value = models.get("claude").and_then(|c| c.get("generation")).cloned();
        }
        let expected: Option<String> = match &value {
            Some(Value::String(s)) => Some(s.clone()),
            Some(Value::Object(o)) => o.get("model").and_then(Value::as_str).map(str::to_string),
            _ => None,
        };
        match expected {
            None => {
                problems.push(AgentFileProblem {
                    code: "agent-file-drift".to_string(),
                    agent: agent_name.to_string(),
                    slot: slot.to_string(),
                    message: format!(
                        ".claude/agents/{agent_name}.md declares model: \"{file_model}\" but the {slot} slot is now cli-shaped or unconfigured (no model name) — re-run onboarding to remove the stale file."
                    ),
                });
            }
            Some(exp) if exp != file_model => {
                problems.push(AgentFileProblem {
                    code: "agent-file-drift".to_string(),
                    agent: agent_name.to_string(),
                    slot: slot.to_string(),
                    message: format!(
                        ".claude/agents/{agent_name}.md declares model: \"{file_model}\" but the configured {slot} model is \"{exp}\" — re-run onboarding to re-render it."
                    ),
                });
            }
            _ => {}
        }
    }
    problems
}

// ─── lanes (bee.mjs-private buildLaneRows/buildLaneSummary) ──────────────

/// `listLanes(root)`.
pub fn list_lanes(root: &Path) -> Vec<State> {
    let entries = match fs::read_dir(lanes_dir(root)) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut lanes = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".json") else { continue };
        if let Some(record) = read_lane(root, stem) {
            lanes.push(record);
        }
    }
    lanes
}

/// `defaultLaneRecord(feature)`'s key order (state.mjs:1619-1630).
///
/// This, NOT the shared [`State`] struct's field order, is the emitted key
/// order of a lane record: `laneRecordFrom` (state.mjs:1638) builds
/// `{...defaultLaneRecord(feature), ...parsed}`, and a JS spread over an
/// existing key OVERWRITES IN PLACE rather than reordering — so every key
/// the default declares keeps the default's position, and only genuinely
/// new keys from `parsed` are appended after them.
///
/// `State` declares `phase` before `feature`/`mode` because that is
/// `defaultState()`'s order for `state.json`; the lane record's default
/// puts `feature, mode` before `phase`. Serializing the shared struct and
/// hoping the order matched emitted
/// `{schema_version, phase, feature, mode, …}` against mjs's
/// `{schema_version, feature, mode, phase, …}` — a D3 byte-compat break
/// (rust-port-15) that stayed invisible until the parity harness gained a
/// pinned session id and a `--lanes-full` leg, because until then no full
/// lane record was ever serialized on the status path at all.
const LANE_RECORD_KEY_ORDER: [&str; 8] = [
    "schema_version",
    "feature",
    "mode",
    "phase",
    "approved_gates",
    "summary",
    "next_action",
    "created_at",
];

/// `buildLaneRows(root)` — every lane record plus `bound_sessions` (the
/// ids of every session currently bound to it). `sessions` is read from
/// `root` directly (bee.mjs's own `buildLaneRows` does NOT re-root through
/// `controlRootFor` — unlike several claims/session reads elsewhere in
/// this cell's scope).
pub fn build_lane_rows(root: &Path) -> Vec<Value> {
    let lanes = list_lanes(root);
    let sessions = crate::claims::list_session_records(root);
    let mut bound_by: HashMap<String, Vec<String>> = HashMap::new();
    for session in &sessions {
        if let Some(lane) = session.lane.as_deref() {
            if !lane.is_empty() {
                bound_by.entry(lane.to_string()).or_default().push(session.id.clone());
            }
        }
    }
    lanes
        .iter()
        .map(|lane| {
            let serialized = serde_json::to_value(lane).unwrap_or(Value::Null);
            let src = serialized.as_object().cloned().unwrap_or_default();
            let feature = lane.feature.clone().unwrap_or_default();
            let bound = bound_by.get(&feature).cloned().unwrap_or_default();

            // Rebuild the row in `defaultLaneRecord`'s key order rather
            // than reordering the struct's serialization in place. Building
            // it explicitly also removes the old `workers` deletion
            // entirely: `workers` is a `state.json`-only field
            // (`defaultState()`), `defaultLaneRecord` has no such key, and
            // no lane-mutation path in the mjs source ever writes one — so
            // the honest fix is never to emit it, not to emit and remove
            // it. (The removal it replaces was also a `Map::remove`, which
            // under this crate's required `preserve_order` feature aliases
            // `swap_remove` — serde_json 1.0.150 `src/map.rs:156-165` —
            // and would have dropped the LAST key into the vacated slot.)
            let mut row = Map::new();
            for key in LANE_RECORD_KEY_ORDER {
                if let Some(value) = src.get(key) {
                    row.insert(key.to_string(), value.clone());
                }
            }
            // Then any key the record carries that the default does not
            // declare, in the record's own order — mirroring the spread's
            // append-only behavior for genuinely new keys.
            for (key, value) in &src {
                if key == "workers" || LANE_RECORD_KEY_ORDER.contains(&key.as_str()) {
                    continue;
                }
                row.insert(key.clone(), value.clone());
            }
            row.insert("bound_sessions".to_string(), json!(bound));
            Value::Object(row)
        })
        .collect()
}

/// `buildLaneSummary(root)` — the payload-size-motivated summary
/// (lpsp-2): the ACTIVE lane in full (the one `resolved_session_id` is
/// bound to) plus counts-by-phase and bare ids for every OTHER lane.
/// `control_root`/`resolved_session_id` are caller-supplied (this cell's
/// established pattern: `resolveSessionId`'s env/root-inference chain and
/// `controlRootFor`'s git-worktree walk are binary-crate concerns, D5
/// hermetic-fixture prohibition) rather than re-derived here.
pub fn build_lane_summary(root: &Path, control_root: &Path, resolved_session_id: Option<&str>) -> Value {
    let lanes = build_lane_rows(root);
    if lanes.is_empty() {
        return json!({"active": Value::Null, "counts": {}, "ids": Vec::<Value>::new()});
    }
    let mut active: Option<Value> = None;
    if let Some(sid) = resolved_session_id {
        if let Some(session) = crate::claims::read_session(control_root, sid) {
            if let Some(lane_name) = session.lane.as_deref().filter(|l| !l.is_empty()) {
                active = lanes.iter().find(|l| l.get("feature").and_then(Value::as_str) == Some(lane_name)).cloned();
            }
        }
    }
    let active_feature = active.as_ref().and_then(|a| a.get("feature")).and_then(Value::as_str).map(str::to_string);
    let rest: Vec<&Value> = match &active_feature {
        Some(af) => lanes.iter().filter(|l| l.get("feature").and_then(Value::as_str) != Some(af.as_str())).collect(),
        None => lanes.iter().collect(),
    };
    let mut counts: Map<String, Value> = Map::new();
    for l in &rest {
        let phase = l.get("phase").and_then(Value::as_str).unwrap_or("idle").to_string();
        let current = counts.get(&phase).and_then(Value::as_i64).unwrap_or(0);
        counts.insert(phase, json!(current + 1));
    }
    let ids: Vec<Value> = rest.iter().map(|l| l.get("feature").cloned().unwrap_or(Value::Null)).collect();
    json!({"active": active.unwrap_or(Value::Null), "counts": counts, "ids": ids})
}

// ─── runtime drift (codex-harness-hardening 1c) ──────────────────────────

/// `computeRuntimeDrift(root, onboardingRaw)`.
pub fn compute_runtime_drift(root: &Path, onboarding_raw: &Value) -> Value {
    let version_drift =
        onboarding_raw.get("bee_version").and_then(Value::as_str).map(|v| v != BEE_VERSION).unwrap_or(false);
    let managed = onboarding_raw.get("managed").filter(|m| m.is_object());
    let Some(managed) = managed else {
        return json!({"drift": version_drift, "detail": Vec::<String>::new()});
    };
    let mut detail: Vec<String> = Vec::new();
    {
        let mut check_group = |recorded: Option<&Value>, rel_dir: &str| {
            let Some(Value::Object(map)) = recorded else { return };
            for (name, recorded_hash) in map {
                let (abs, rel_posix) = if rel_dir.is_empty() {
                    (root.join(".bee").join("bin").join(name), format!(".bee/bin/{name}"))
                } else {
                    (root.join(".bee").join("bin").join(rel_dir).join(name), format!(".bee/bin/{rel_dir}/{name}"))
                };
                match crate::fsutil::hash_file(&abs) {
                    Ok(live) => {
                        if Some(live.as_str()) != recorded_hash.as_str() {
                            detail.push(rel_posix);
                        }
                    }
                    Err(_) => detail.push(format!("{rel_posix} (missing)")),
                }
            }
        };
        check_group(managed.get("lib"), "lib");
        check_group(managed.get("helpers"), "");
    }
    if let Some(Value::Object(lib_map)) = managed.get("lib") {
        if let Ok(entries) = fs::read_dir(root.join(".bee").join("bin").join("lib")) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.ends_with(".mjs") && !lib_map.contains_key(&name) {
                    detail.push(format!(".bee/bin/lib/{name} (extra)"));
                }
            }
        }
    }
    json!({"drift": version_drift || !detail.is_empty(), "detail": detail})
}

// ─── source identity + worktree notice (DIST-04/SRC-01, GH#30) ──────────

/// `findRepoHive(root)`.
pub fn find_repo_hive(root: &Path) -> Option<PathBuf> {
    let candidates: [Vec<&str>; 3] = [vec!["skills"], vec![".claude", "skills"], vec![".agents", "skills"]];
    for segs in candidates {
        let mut hive = root.to_path_buf();
        for seg in segs {
            hive.push(seg);
        }
        hive.push("bee-hive");
        if hive.exists() {
            return Some(hive);
        }
    }
    None
}

/// Caller-supplied `resolveRoots()`-shaped classification (rust-port-20,
/// mirrors `guards.rs`'s `WriteTopology` pattern and rust-port-16's
/// control-root-parameter convention): [`ungranted_worktree_notice`] never
/// re-derives this itself — the binary crate's own git-worktree walk
/// (`queen-bee::adapter::resolve_roots`, a binary-crate concern per this
/// cell's own action note on `controlRootFor`) is the source. Field names
/// mirror `queen-bee::adapter::Roots`; a bee-core -> queen-bee dependency
/// would invert the crate graph, so this is a parallel, minimal shape
/// rather than a shared type.
#[derive(Debug, Clone)]
pub struct WorktreeRootsView {
    pub worktree_resolution: String,
    pub store_root: Option<PathBuf>,
    pub main_root: Option<PathBuf>,
}

/// `ungrantedWorktreeNotice(root)` — messaging only (per the mjs source's
/// own comment): never affects store-root selection, only whether to
/// print a notice about the selection the caller already made.
pub fn ungranted_worktree_notice(view: &WorktreeRootsView) -> Option<String> {
    if view.worktree_resolution != "linked-valid" {
        return None;
    }
    let (Some(store_root), Some(main_root)) = (&view.store_root, &view.main_root) else {
        return None;
    };
    if store_root != main_root {
        return None;
    }
    Some("⚠ This linked worktree is UNGRANTED — it SHARES the main checkout's store (same feature/phase/claims; no isolation). To work an isolated feature: run \"bee worktree new --feature <slug>\" from the main checkout. To grant isolation to THIS existing worktree instead: run \"bee worktree register --feature <slug>\" from inside it.".to_string())
}

// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
