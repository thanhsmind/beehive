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
use serde_json::{Map, Value};

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


// Tests live in crates/bee-core/tests/guard_support.rs (this cell's single
// integration target — cargo test -p bee-core --test guard_support) rather
// than here, so every reader's round-trip/logic proof sits in one place
// per must-have.
