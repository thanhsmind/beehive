//! config — reader for `.bee/config.json`, ported from
//! `.bee/bin/lib/state.mjs`'s `readConfig`/`bypassLevel` (rust-port-8,
//! CONTEXT.md D3). Read-only, zero subprocess (D5): this module never
//! shells out and never writes `.bee/config.json` or its local overlay.
//!
//! `.bee/bin/lib/state.mjs` is FROZEN for the duration of the rust-port
//! feature (D1) — this module mirrors its config-reading semantics, not
//! the full config-mutation surface (`bee config set/unset`, which stays
//! mjs-only for now). Every struct round-trips unknown fields via
//! `#[serde(flatten)]` (D3), so a repo running a newer/older mjs config
//! shape than this reader knows about never loses data on a read-modify-
//! write cycle — matching the `fsutil_oracle.rs` flatten pattern
//! established by rust-port-5.
//!
//! One deliberate scope narrowing vs. the full mjs `readConfig`: this
//! reader does NOT implement the `.bee/config.local.json` overlay merge
//! (state.mjs's `mergeConfigOverlay`) — it reads the tracked
//! `.bee/config.json` only. The overlay is a machine-local escape hatch for
//! `guards.*`/`hooks.*` namespaces (state.mjs's `LOCAL_ONLY_CONFIG_NAMESPACES`
//! comment); the guard-support readers this cell delivers (rust-port-9..12's
//! consumers) read the tracked file's hooks/gate_bypass/models, which is
//! sufficient for the read-only checks named in this cell. A future cell
//! that needs overlay-aware reads should add it here rather than
//! reimplementing readConfig a second time.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::fsutil::read_json;

/// Decision 0012 (state.mjs `CONFIGURABLE_TIERS`): the two model tiers a
/// repo may override per runtime.
pub const CONFIGURABLE_TIERS: [&str; 2] = ["extraction", "generation"];
/// Decision 0021 P16 (state.mjs `CONFIGURABLE_SLOTS`): tiers plus the
/// `review` role slot — `review` falls back to `generation` when unset.
pub const CONFIGURABLE_SLOTS: [&str; 3] = ["extraction", "generation", "review"];
/// Decision D2 (advisor feature, state.mjs `MODEL_NORMALIZE_SLOTS`):
/// `CONFIGURABLE_SLOTS` plus `advisor` — `advisor` is normalized alongside
/// the configurable slots but is deliberately NOT one of them (it is not a
/// tier and never falls back to `generation`).
pub const MODEL_NORMALIZE_SLOTS: [&str; 4] = ["extraction", "generation", "review", "advisor"];
/// state.mjs `RUNTIMES` — the two dispatcher runtimes a `models` map may key by.
pub const RUNTIMES: [&str; 2] = ["claude", "codex"];
/// state.mjs `DEFAULT_HOOKS` — every hook name defaults to enabled; a
/// tracked config.json only needs to list the ones it disables.
pub const DEFAULT_HOOKS: [(&str, bool); 7] = [
    ("session-init", true),
    ("prompt-context", true),
    ("write-guard", true),
    ("state-sync", true),
    ("chain-nudge", true),
    ("session-close", true),
    ("tools-logger", true),
];
/// state.mjs `BYPASS_LEVELS`.
pub const BYPASS_LEVELS: [&str; 4] = ["off", "normal", "full", "total"];
/// state.mjs `EFFORT_LEVELS` (decision 0021 P17) — the per-slot reasoning
/// effort values `normalizeTierValue` accepts; an out-of-list effort string
/// is silently dropped, never propagated.
pub const EFFORT_LEVELS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

pub fn config_path(root: &Path) -> PathBuf {
    root.join(".bee").join("config.json")
}

/// `.bee/config.json`, typed for the fields guard-support readers need
/// (hooks toggles, `gate_bypass`, the full `models` map) with every other
/// key preserved verbatim via `extra` (D3 round-trip).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Per-hook enable/disable toggles. Unlisted hooks default to enabled —
    /// use [`Config::hook_enabled`] rather than indexing this map directly,
    /// matching state.mjs's `hookEnabled` (`config.hooks[name] !== false`).
    #[serde(default)]
    pub hooks: Map<String, Value>,
    /// Raw `gate_bypass` value (bool, string level, or absent/null) — use
    /// [`Config::bypass_level`] for the normalized level, matching
    /// state.mjs's `bypassLevel`.
    #[serde(default)]
    pub gate_bypass: Value,
    /// The full `models` map, kept as raw JSON: per-runtime, per-slot
    /// entries are one of a bare model-name string, `{model, effort}`, a
    /// native/cli executor object, or a composite `{primary, fallback,
    /// fallback_policy}` — CONFIGURABLE_NORMALIZE_SLOTS-shaped validation
    /// lives in state.mjs and is out of scope for a read-only reader.
    #[serde(default)]
    pub models: Value,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hooks: Map::new(),
            gate_bypass: Value::Null,
            models: Value::Null,
            extra: Map::new(),
        }
    }
}

impl Config {
    /// `hookEnabled(root, name)`: a hook is enabled unless explicitly set
    /// to `false` — absent/unlisted/non-boolean all read as enabled.
    pub fn hook_enabled(&self, name: &str) -> bool {
        !matches!(self.hooks.get(name), Some(Value::Bool(false)))
    }

    /// `bypassLevel(root)`: normalizes the raw `gate_bypass` value into one
    /// of the four levels. Legacy boolean `true` maps to `"normal"`;
    /// anything else unrecognized maps to `"off"`.
    pub fn bypass_level(&self) -> &'static str {
        match &self.gate_bypass {
            Value::String(s) if s == "total" => "total",
            Value::String(s) if s == "full" => "full",
            Value::Bool(true) => "normal",
            Value::String(s) if s == "on" || s == "normal" => "normal",
            _ => "off",
        }
    }

    /// The advisor model override for a runtime/slot, e.g.
    /// `models.claude.advisor`, as raw JSON (string | object | null | absent).
    pub fn model_slot(&self, runtime: &str, slot: &str) -> Option<&Value> {
        self.models.get(runtime).and_then(|r| r.get(slot))
    }
}

/// state.mjs `localConfigPath` — the per-machine, gitignored overlay sibling
/// of the tracked `.bee/config.json`.
pub fn local_config_path(root: &Path) -> PathBuf {
    root.join(".bee").join("config.local.json")
}

/// Port of state.mjs `mergeConfigOverlay` — deep-merge `overlay` OVER `base`
/// (overlay wins on every conflict; plain objects merge key-by-key; arrays
/// REPLACE wholesale; a scalar overlay value replaces the base value).
pub fn merge_config_overlay(base: &Value, overlay: &Value) -> Value {
    match overlay {
        Value::Array(a) => Value::Array(a.clone()),
        Value::Object(overlay_map) => {
            let base_map = match base {
                Value::Object(m) => m.clone(),
                _ => Map::new(),
            };
            let mut out = base_map.clone();
            for (key, value) in overlay_map {
                let merged = match (base_map.get(key), value) {
                    (Some(bv @ Value::Object(_)), Value::Object(_)) => merge_config_overlay(bv, value),
                    (_, Value::Array(a)) => Value::Array(a.clone()),
                    _ => value.clone(),
                };
                out.insert(key.clone(), merged);
            }
            Value::Object(out)
        }
        _ => base.clone(),
    }
}

/// The RAW merged config value (tracked `.bee/config.json` with the
/// `.bee/config.local.json` overlay winning), for callers that need
/// namespaces outside the typed [`Config`] shape — rust-port-9's guard
/// checks read `guards.*` here, a LOCAL_ONLY namespace whose live values a
/// host writes into the overlay (state.mjs `LOCAL_ONLY_CONFIG_NAMESPACES`),
/// so an overlay-blind read would miss e.g. a `guards.idle_gate` opt-out.
/// Missing/malformed files fall open exactly like state.mjs `readConfig`:
/// a non-object tracked file reads as `{}`, an absent/malformed overlay
/// leaves the tracked object untouched.
pub fn read_config_value(root: &Path) -> Value {
    let raw: Value = read_json(&config_path(root), Value::Null);
    let tracked = if raw.is_object() { raw } else { Value::Object(Map::new()) };
    let overlay: Value = read_json(&local_config_path(root), Value::Null);
    if overlay.is_object() {
        merge_config_overlay(&tracked, &overlay)
    } else {
        tracked
    }
}

/// Reads `.bee/config.json` under `root`, matching state.mjs's
/// `readConfig` for the fields this cell's readers need: a missing or
/// malformed file falls open to `Config::default()` (fail-open, same
/// posture as [`crate::fsutil::read_json`]'s fallback semantics), never a
/// panic or an error return.
pub fn read_config(root: &Path) -> Config {
    let raw: Value = read_json(&config_path(root), Value::Null);
    if !raw.is_object() {
        return Config::default();
    }
    serde_json::from_value::<Config>(raw).unwrap_or_default()
}


// ─── model-tier resolution (rust-port-10, CONTEXT.md D2/D7) ────────────────
// Port of state.mjs's `normalizeTierValue`/`normalizeModels`/`resolveTier`/
// `resolveAdvisor`/`modelForTier` (decisions 0012/0019/0021, advisor
// feature D2) — the tier-resolution half of the guard-support reader this
// module already provides (`read_config`/`read_config_value` above read the
// RAW `models` map; these reproduce state.mjs's DEFAULTING + shape
// normalization on top of it, which `dispatch_guard::evaluate_dispatch`
// depends on for byte-identical deny/allow verdicts against the mjs oracle).
// `.bee/bin/lib/state.mjs` stays FROZEN (D1); conformance lives in
// `crates/queen-bee/tests/modelguard_conformance.rs`.

/// state.mjs `DEFAULT_MODELS` — applied BEFORE any configured override:
/// claude defaults to named models per slot; codex has no per-agent model
/// switch by default (every slot budget/cap-fallback). Neither runtime
/// defaults an `advisor` slot — it stays absent unless configured.
fn default_models() -> Value {
    json!({
        "claude": { "extraction": "haiku", "generation": "sonnet", "review": "opus" },
        "codex": { "extraction": Value::Null, "generation": Value::Null, "review": Value::Null },
    })
}

/// Port of `normalizeTierValue(value)`: `None` means "undefined — the
/// caller keeps whatever default it already had"; `Some(Value::Null)` means
/// an EXPLICIT null override (budget/cap fallback, distinct from "absent").
/// Every other `Some` is one of the four normalized leaf shapes: a bare
/// model-name string, `{kind:"cli", command}`, `{kind:"native", model,
/// effort?, fork_turns?, agent_type?}`, `{primary, fallback_policy?,
/// fallback?}`, or `{model, effort?}`. An invalid/unrecognized shape
/// normalizes to `None` (undefined), exactly like the mjs source.
fn normalize_tier_value(value: Option<&Value>) -> Option<Value> {
    let value = value?;
    if let Some(s) = value.as_str() {
        let trimmed = s.trim();
        return if trimmed.is_empty() { None } else { Some(Value::String(trimmed.to_string())) };
    }
    if value.is_null() {
        return Some(Value::Null);
    }
    let Value::Object(obj) = value else { return None };

    if obj.get("kind").and_then(Value::as_str) == Some("cli") {
        let command = obj.get("command").and_then(Value::as_str).map(str::trim).unwrap_or("");
        return if command.is_empty() { None } else { Some(json!({ "kind": "cli", "command": command })) };
    }
    if obj.get("kind").and_then(Value::as_str) == Some("native") {
        return normalize_native_leaf(obj).map(Value::Object);
    }
    if let Some(primary) = obj.get("primary") {
        let primary_valid = primary
            .get("kind")
            .and_then(Value::as_str)
            .map(|k| k == "native")
            .unwrap_or(false)
            && primary.get("model").and_then(Value::as_str).map(|m| !m.trim().is_empty()).unwrap_or(false);
        if primary_valid {
            let Value::Object(primary_obj) = primary else { unreachable!("primary_valid implies an object") };
            let normalized_primary = normalize_native_leaf(primary_obj).expect("primary_valid implies a valid native leaf");
            let mut out = Map::new();
            out.insert("primary".to_string(), Value::Object(normalized_primary));
            if obj.get("fallback_policy").and_then(Value::as_str) == Some("explicit-only") {
                out.insert("fallback_policy".to_string(), Value::String("explicit-only".to_string()));
                if let Some(Value::Object(fb)) = obj.get("fallback") {
                    if fb.get("kind").and_then(Value::as_str) == Some("cli") {
                        let command = fb.get("command").and_then(Value::as_str).map(str::trim).unwrap_or("");
                        if !command.is_empty() {
                            out.insert(
                                "fallback".to_string(),
                                json!({ "kind": "cli", "command": command }),
                            );
                        }
                    }
                }
            }
            return Some(Value::Object(out));
        }
    }
    if obj.get("kind").is_none() {
        if let Some(model) = obj.get("model").and_then(Value::as_str).map(str::trim).filter(|m| !m.is_empty()) {
            let mut out = Map::new();
            out.insert("model".to_string(), Value::String(model.to_string()));
            if let Some(effort) = obj.get("effort").and_then(Value::as_str).map(str::trim) {
                if EFFORT_LEVELS.contains(&effort) {
                    out.insert("effort".to_string(), Value::String(effort.to_string()));
                }
            }
            return Some(Value::Object(out));
        }
    }
    None
}

/// Shared native-leaf normalization (`{kind:"native", model, effort?,
/// fork_turns?, agent_type?}`) for both the direct `kind:"native"` branch
/// and the `primary` slot of a composite — mirrors `normalizeTierValue`'s
/// two call sites of the same inline logic. `None` when `model` is missing
/// or blank (an invalid native leaf never normalizes).
fn normalize_native_leaf(obj: &Map<String, Value>) -> Option<Map<String, Value>> {
    let model = obj.get("model").and_then(Value::as_str).map(str::trim).filter(|m| !m.is_empty())?;
    let mut out = Map::new();
    out.insert("kind".to_string(), Value::String("native".to_string()));
    out.insert("model".to_string(), Value::String(model.to_string()));
    if let Some(effort) = obj.get("effort").and_then(Value::as_str).map(str::trim) {
        if EFFORT_LEVELS.contains(&effort) {
            out.insert("effort".to_string(), Value::String(effort.to_string()));
        }
    }
    if obj.get("fork_turns").and_then(Value::as_str).map(str::trim) == Some("none") {
        out.insert("fork_turns".to_string(), Value::String("none".to_string()));
    }
    if let Some(agent_type) = obj.get("agent_type").and_then(Value::as_str).map(str::trim).filter(|a| !a.is_empty()) {
        out.insert("agent_type".to_string(), Value::String(agent_type.to_string()));
    }
    Some(out)
}

/// Port of `normalizeModels(raw)`: seeds [`default_models`], then overrides
/// each `(runtime, slot)` pair present and valid in `raw` — a repo that
/// configures nothing gets the plain claude/codex defaults; an `advisor`
/// slot appears only when the repo configured one (neither default carries
/// it).
fn normalize_models(raw: &Value) -> Value {
    let mut out = default_models();
    if let Value::Object(raw_map) = raw {
        for rt in RUNTIMES {
            let Some(Value::Object(src)) = raw_map.get(rt) else { continue };
            let Some(target) = out.get_mut(rt) else { continue };
            for slot in MODEL_NORMALIZE_SLOTS {
                if let Some(normalized) = normalize_tier_value(src.get(slot)) {
                    target[slot] = normalized;
                }
            }
        }
    }
    out
}

/// The DEFAULTED + normalized `models` map for `root` — `read_config_value`
/// (tracked + local-overlay merge) run through [`normalize_models`], the
/// exact input `resolveTier`/`resolveAdvisor` read in the mjs source.
fn normalized_models(root: &Path) -> Value {
    let raw_models = read_config_value(root).get("models").cloned().unwrap_or(Value::Null);
    normalize_models(&raw_models)
}

/// Resolved shape of a configurable tier/advisor slot (decisions
/// 0019/0021/D2) — the Rust twin of state.mjs `resolveTier`/`resolveAdvisor`'s
/// returned `{type, ...}` records.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedTier {
    /// `{type:'inherit'}` — ceiling: omit the model param, inherit the
    /// session model.
    Inherit,
    /// `{type:'model', model, effort?}` — a named per-agent model switch.
    Model { model: String, effort: Option<String> },
    /// `{type:'budget'}` — no per-agent switch; enforce via prompt budget.
    Budget,
    /// `{type:'cli', command}` — an external executor (gather-purpose only).
    Cli { command: String },
    /// `{type:'native', model, effort?, fork_turns, agent_type, fallback?}`
    /// — a codex spawn_agent V2 model-override leaf (D2).
    Native { model: String, effort: Option<String>, fork_turns: String, agent_type: String, fallback: Option<String> },
    /// `{type:'refused', reason, slot, fix}` — a cli-shaped slot resolved
    /// for `Cell` purpose (an in-family subagent cannot BE the external CLI).
    Refused { reason: String, slot: String, fix: String },
}

/// `resolveTier`'s optional 4th `purpose` parameter (AO12/B1): `Cell`
/// (default) refuses a cli-shaped slot; only an explicit `Gather` resolves
/// it to a dispatchable `{type:'cli', command}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResolvePurpose {
    #[default]
    Cell,
    Gather,
}

/// A normalized leaf's shape, independent of which caller (resolve_tier /
/// resolve_advisor) is interpreting it — factors out the shared parse so
/// neither caller re-derives the other's logic (mirrors how both mjs
/// functions share the same leaf-shape checks in the same order: cli,
/// native, primary/composite, plain model).
enum NormalizedLeaf {
    Model { model: String, effort: Option<String> },
    Cli { command: String },
    Native { model: String, effort: Option<String>, fork_turns: String, agent_type: String, fallback: Option<String> },
    None,
}

fn parse_normalized_leaf(value: &Value) -> NormalizedLeaf {
    if let Some(s) = value.as_str() {
        return NormalizedLeaf::Model { model: s.to_string(), effort: None };
    }
    let Value::Object(obj) = value else { return NormalizedLeaf::None };

    if obj.get("kind").and_then(Value::as_str) == Some("cli") {
        let command = obj.get("command").and_then(Value::as_str).unwrap_or_default().to_string();
        return NormalizedLeaf::Cli { command };
    }
    if obj.get("kind").and_then(Value::as_str) == Some("native") {
        return native_leaf_from(obj, None);
    }
    if let Some(primary) = obj.get("primary") {
        if let NormalizedLeaf::Native { model, effort, fork_turns, agent_type, .. } = parse_normalized_leaf(primary) {
            let mut fallback = None;
            if obj.get("fallback_policy").and_then(Value::as_str) == Some("explicit-only") {
                if let Some(fb) = obj.get("fallback") {
                    if fb.get("kind").and_then(Value::as_str) == Some("cli") {
                        fallback = fb.get("command").and_then(Value::as_str).map(str::to_string);
                    }
                }
            }
            return NormalizedLeaf::Native { model, effort, fork_turns, agent_type, fallback };
        }
    }
    if let Some(model) = obj.get("model").and_then(Value::as_str) {
        let effort = obj.get("effort").and_then(Value::as_str).map(str::to_string);
        return NormalizedLeaf::Model { model: model.to_string(), effort };
    }
    NormalizedLeaf::None
}

fn native_leaf_from(obj: &Map<String, Value>, fallback: Option<String>) -> NormalizedLeaf {
    NormalizedLeaf::Native {
        model: obj.get("model").and_then(Value::as_str).unwrap_or_default().to_string(),
        effort: obj.get("effort").and_then(Value::as_str).map(str::to_string),
        fork_turns: obj.get("fork_turns").and_then(Value::as_str).unwrap_or("none").to_string(),
        agent_type: obj.get("agent_type").and_then(Value::as_str).unwrap_or("worker").to_string(),
        fallback,
    }
}

/// Port of `resolveTier(root, slot, runtime, purpose)`. `slot` outside
/// [`CONFIGURABLE_SLOTS`] coerces to `"generation"`; `runtime` outside
/// [`RUNTIMES`] coerces to `"claude"` — matching the mjs fail-safe
/// coercions exactly.
pub fn resolve_tier(root: &Path, slot: &str, runtime: &str, purpose: ResolvePurpose) -> ResolvedTier {
    if slot == "ceiling" {
        return ResolvedTier::Inherit;
    }
    let models = normalized_models(root);
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    let slot_name = if CONFIGURABLE_SLOTS.contains(&slot) { slot } else { "generation" };

    let mut leaf = models.get(rt).and_then(|r| r.get(slot_name)).cloned();
    if matches!(leaf, None | Some(Value::Null)) && slot_name == "review" {
        leaf = models.get(rt).and_then(|r| r.get("generation")).cloned(); // review falls back to generation
    }
    let leaf = match leaf {
        None | Some(Value::Null) => return ResolvedTier::Budget,
        Some(v) => v,
    };

    match parse_normalized_leaf(&leaf) {
        NormalizedLeaf::Model { model, effort } => ResolvedTier::Model { model, effort },
        NormalizedLeaf::Cli { command } => {
            if purpose == ResolvePurpose::Gather {
                ResolvedTier::Cli { command }
            } else {
                ResolvedTier::Refused {
                    reason: "cli_tier_gather_only".to_string(),
                    slot: slot_name.to_string(),
                    fix: "declare {for:\"gather\"} for a read-only gather; cli cell execution stays refused until a cell-execution dogfood is green (plan 2A/W9)".to_string(),
                }
            }
        }
        NormalizedLeaf::Native { model, effort, fork_turns, agent_type, fallback } => {
            ResolvedTier::Native { model, effort, fork_turns, agent_type, fallback }
        }
        NormalizedLeaf::None => ResolvedTier::Budget,
    }
}

/// Port of `modelForTier(root, tier, runtime)`: the resolved model NAME, or
/// `None` when the runtime cannot per-agent switch models for this tier
/// (ceiling/budget/cli/native/refused all degrade to `None` here — callers
/// that need the richer shape use [`resolve_tier`] directly).
pub fn model_for_tier(root: &Path, tier: &str, runtime: &str) -> Option<String> {
    match resolve_tier(root, tier, runtime, ResolvePurpose::Cell) {
        ResolvedTier::Model { model, .. } => Some(model),
        _ => None,
    }
}

/// Port of `resolveAdvisor(root, runtime)`. Deliberately NOT routed through
/// [`resolve_tier`]: `advisor` is not a [`CONFIGURABLE_SLOTS`] member, so
/// `resolve_tier` would silently coerce it to `"generation"` — the one
/// behavior this function must never exhibit. `None` unambiguously means
/// "no advisor" (unset, explicit null, or an invalid/unrecognized shape);
/// unlike [`resolve_tier`] this never produces `Budget` or `Refused` — a
/// cli-shaped advisor always resolves to `Cli` regardless of purpose (the
/// mjs source has no gather/cell distinction for the advisor slot).
pub fn resolve_advisor(root: &Path, runtime: &str) -> Option<ResolvedTier> {
    let models = normalized_models(root);
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    let leaf = match models.get(rt).and_then(|r| r.get("advisor")).cloned() {
        None | Some(Value::Null) => return None,
        Some(v) => v,
    };
    match parse_normalized_leaf(&leaf) {
        NormalizedLeaf::Model { model, effort } => Some(ResolvedTier::Model { model, effort }),
        NormalizedLeaf::Cli { command } => Some(ResolvedTier::Cli { command }),
        NormalizedLeaf::Native { model, effort, fork_turns, agent_type, fallback } => {
            Some(ResolvedTier::Native { model, effort, fork_turns, agent_type, fallback })
        }
        NormalizedLeaf::None => None,
    }
}

#[cfg(test)]
mod tier_resolution_tests {
    use super::*;

    fn write_config(root: &Path, models: Value) {
        let path = root.join(".bee").join("config.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_string(&json!({ "models": models })).unwrap()).unwrap();
    }

    #[test]
    fn resolve_tier_ceiling_always_inherits() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(resolve_tier(dir.path(), "ceiling", "claude", ResolvePurpose::Cell), ResolvedTier::Inherit);
    }

    #[test]
    fn resolve_tier_defaults_apply_when_unconfigured() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            resolve_tier(dir.path(), "generation", "claude", ResolvePurpose::Cell),
            ResolvedTier::Model { model: "sonnet".to_string(), effort: None }
        );
        assert_eq!(resolve_tier(dir.path(), "generation", "codex", ResolvePurpose::Cell), ResolvedTier::Budget);
    }

    #[test]
    fn resolve_tier_review_falls_back_to_generation() {
        let dir = tempfile::tempdir().unwrap();
        write_config(dir.path(), json!({ "claude": { "generation": "sonnet", "review": null } }));
        assert_eq!(
            resolve_tier(dir.path(), "review", "claude", ResolvePurpose::Cell),
            ResolvedTier::Model { model: "sonnet".to_string(), effort: None }
        );
    }

    #[test]
    fn resolve_tier_cli_refuses_for_cell_and_allows_for_gather() {
        let dir = tempfile::tempdir().unwrap();
        write_config(dir.path(), json!({ "codex": { "review": { "kind": "cli", "command": "codex exec -" } } }));
        let cell = resolve_tier(dir.path(), "review", "codex", ResolvePurpose::Cell);
        assert!(matches!(cell, ResolvedTier::Refused { .. }), "expected refused, got {cell:?}");
        assert_eq!(
            resolve_tier(dir.path(), "review", "codex", ResolvePurpose::Gather),
            ResolvedTier::Cli { command: "codex exec -".to_string() }
        );
    }

    #[test]
    fn resolve_advisor_absent_by_default_present_when_configured() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(resolve_advisor(dir.path(), "claude"), None);
        write_config(dir.path(), json!({ "claude": { "advisor": "fable" } }));
        assert_eq!(
            resolve_advisor(dir.path(), "claude"),
            Some(ResolvedTier::Model { model: "fable".to_string(), effort: None })
        );
    }

    #[test]
    fn resolve_advisor_never_coerces_unknown_slot_to_generation() {
        // resolve_advisor must never fall through to resolve_tier's
        // slot-coercion behavior — proven by configuring ONLY generation and
        // confirming advisor still reads as None, not the generation model.
        let dir = tempfile::tempdir().unwrap();
        write_config(dir.path(), json!({ "claude": { "generation": "sonnet" } }));
        assert_eq!(resolve_advisor(dir.path(), "claude"), None);
    }

    #[test]
    fn model_for_tier_degrades_non_model_types_to_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(model_for_tier(dir.path(), "ceiling", "claude"), None);
        assert_eq!(model_for_tier(dir.path(), "generation", "codex"), None);
        assert_eq!(model_for_tier(dir.path(), "generation", "claude"), Some("sonnet".to_string()));
    }
}

// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have. The tier-resolution unit tests above are the exception
// (`tier_resolution_tests`, inline): they exercise this module's own new
// logic in isolation, independent of the modelguard_conformance node-oracle
// diff suite (crates/queen-bee/tests/modelguard_conformance.rs) that proves
// the end-to-end hook decision.
