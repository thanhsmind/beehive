// state — Rust port of the state-layer reads `status --brief` needs
// (state.mjs readState / readConfig / bypassLevel / shipVisibility).
//
// Strangler rule: any input Node handles via warn-with-V8-message (corrupt
// JSON) or via JS spread exotica (a truthy non-object approved_gates) makes
// the native path bail with `Bail::NeedsNode` BEFORE emitting anything, and
// the whole command re-runs under Node.

use crate::fsutil::{read_json, ReadJson};
use crate::jsjson;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

pub enum Bail {
    NeedsNode,
}

pub fn state_path(root: &Path) -> PathBuf {
    root.join(".bee").join("state.json")
}

fn default_gates() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("context".into(), Value::Bool(false));
    m.insert("shape".into(), Value::Bool(false));
    m.insert("execution".into(), Value::Bool(false));
    m.insert("review".into(), Value::Bool(false));
    m
}

pub const GATE_NAMES: [&str; 4] = ["context", "shape", "execution", "review"];

/// The slice of readState() that buildStatusBrief consumes.
pub struct BriefState {
    pub phase: Value,
    pub feature: Value,
    pub mode: Value,
    pub gates: Map<String, Value>,
    pub route: Value,
}

pub fn read_state_brief(root: &Path) -> Result<BriefState, Bail> {
    let file_state = match read_json(&state_path(root)) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => return Err(Bail::NeedsNode), // Node warns with the V8 message
        ReadJson::Parsed(v) => match v {
            Value::Object(m) => Some(m),
            _ => None, // non-object parses fall back to defaultState(), silently
        },
    };

    let default_phase = json!("idle");
    let Some(state) = file_state else {
        return Ok(BriefState {
            phase: default_phase,
            feature: Value::Null,
            mode: Value::Null,
            gates: default_gates(),
            route: Value::Null,
        });
    };

    // { ...defaultState(), ...state } — file value wins whenever the KEY is
    // present, even when the value is null.
    let pick = |key: &str, default: Value| -> Value {
        state.get(key).cloned().unwrap_or(default)
    };

    // approved_gates: { ...defaults, ...(state.approved_gates || {}) }.
    // Falsy (null/false/0/"") -> defaults. Truthy non-object spreads exotic
    // key sets in JS (string chars, array indices) — delegate those.
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
        Some(_) => return Err(Bail::NeedsNode),
    };

    // coerceLegacyPhase: 'validating' -> 'planning' (D13).
    let mut phase = pick("phase", default_phase);
    if phase == json!("validating") {
        phase = json!("planning");
    }

    Ok(BriefState {
        phase,
        feature: pick("feature", Value::Null),
        mode: pick("mode", Value::Null),
        gates,
        route: match state.get("route") {
            // state.route ?? null — null/undefined both land on null.
            None | Some(Value::Null) => Value::Null,
            Some(v) => v.clone(),
        },
    })
}

// ── config (gate_bypass / ship_visibility slice of readConfig) ─────────────

/// mergeConfigOverlay: overlay wins; plain objects merge recursively; arrays
/// replace wholesale; scalars replace.
fn merge_config_overlay(base: &Value, overlay: &Value) -> Value {
    match overlay {
        Value::Array(items) => Value::Array(items.clone()),
        Value::Object(over) => {
            let base_obj = match base {
                Value::Object(m) => m.clone(),
                _ => Map::new(),
            };
            let mut out = base_obj.clone();
            for (key, value) in over {
                let merged = match (base_obj.get(key), value) {
                    (Some(b @ Value::Object(_)), Value::Object(_)) => merge_config_overlay(b, value),
                    _ => match value {
                        Value::Array(items) => Value::Array(items.clone()),
                        other => other.clone(),
                    },
                };
                out.insert(key.clone(), merged);
            }
            Value::Object(out)
        }
        _ => base.clone(),
    }
}

/// Merged tracked+overlay config as a raw object (advisor key stripped like
/// readConfig does; the normalize* steps don't touch the keys brief reads).
pub fn read_config_raw(root: &Path) -> Result<Map<String, Value>, Bail> {
    let read_obj = |file: PathBuf| -> Result<Option<Map<String, Value>>, Bail> {
        match read_json(&file) {
            ReadJson::Missing => Ok(None),
            ReadJson::Corrupt => Err(Bail::NeedsNode),
            ReadJson::Parsed(Value::Object(m)) => Ok(Some(m)),
            ReadJson::Parsed(_) => Ok(None),
        }
    };
    let tracked = read_obj(root.join(".bee").join("config.json"))?.unwrap_or_default();
    let overlay = read_obj(root.join(".bee").join("config.local.json"))?;
    let mut merged = match overlay {
        Some(over) => match merge_config_overlay(&Value::Object(tracked), &Value::Object(over)) {
            Value::Object(m) => m,
            _ => unreachable!("object-over-object merge yields an object"),
        },
        None => tracked,
    };
    merged.shift_remove("advisor");
    Ok(merged)
}

/// hookEnabled: `config.hooks[name] !== false` over merged tracked+overlay
/// config — enabled unless the file explicitly carries `false` (unknown
/// names default enabled; DEFAULT_HOOKS are all true, so the merge with
/// defaults never changes this predicate).
pub fn hook_enabled(root: &Path, name: &str) -> Result<bool, Bail> {
    let config = read_config_raw(root)?;
    Ok(!matches!(
        config.get("hooks").and_then(|h| h.get(name)),
        Some(Value::Bool(false))
    ))
}

pub fn bypass_level(config: &Map<String, Value>) -> &'static str {
    match config.get("gate_bypass") {
        Some(Value::String(s)) if s == "total" => "total",
        Some(Value::String(s)) if s == "full" => "full",
        Some(Value::Bool(true)) => "normal",
        Some(Value::String(s)) if s == "on" || s == "normal" => "normal",
        _ => "off",
    }
}

/// shipVisibility: 'off'/'draft-pr' pass; absent/null silently 'off'; any
/// other value warns to stderr (byte-identical line) and normalizes to 'off'.
pub fn ship_visibility(config: &Map<String, Value>) -> String {
    match config.get("ship_visibility") {
        None | Some(Value::Null) => "off".to_string(),
        Some(Value::String(s)) if s == "off" || s == "draft-pr" => s.clone(),
        Some(other) => {
            eprint!(
                "config: unrecognized ship_visibility \"{}\" in .bee/config.json — normalized to \"off\". Allowed: off, draft-pr.\n",
                jsjson::js_to_string(other)
            );
            "off".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_state(root: &Path, content: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("state.json"), content).unwrap();
    }

    #[test]
    fn missing_state_yields_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let s = read_state_brief(tmp.path()).ok().unwrap();
        assert_eq!(s.phase, json!("idle"));
        assert_eq!(s.feature, Value::Null);
        assert_eq!(jsjson::stringify(&Value::Object(s.gates)),
            r#"{"context":false,"shape":false,"execution":false,"review":false}"#);
    }

    #[test]
    fn file_keys_override_defaults_and_legacy_phase_coerces() {
        let tmp = tempfile::tempdir().unwrap();
        write_state(tmp.path(), r#"{"phase":"validating","feature":"f1","approved_gates":{"shape":true,"extra":1}}"#);
        let s = read_state_brief(tmp.path()).ok().unwrap();
        assert_eq!(s.phase, json!("planning"));
        assert_eq!(s.feature, json!("f1"));
        // Default key order first, extras appended in file order.
        assert_eq!(jsjson::stringify(&Value::Object(s.gates)),
            r#"{"context":false,"shape":true,"execution":false,"review":false,"extra":1}"#);
    }

    #[test]
    fn corrupt_state_bails_to_node() {
        let tmp = tempfile::tempdir().unwrap();
        write_state(tmp.path(), "{broken");
        assert!(read_state_brief(tmp.path()).is_err());
    }

    #[test]
    fn overlay_wins_scalar_and_deep_merges_objects() {
        let base = serde_json::from_str::<Value>(r#"{"gate_bypass":"off","o":{"a":1,"b":2}}"#).unwrap();
        let over = serde_json::from_str::<Value>(r#"{"gate_bypass":"full","o":{"b":3}}"#).unwrap();
        let merged = merge_config_overlay(&base, &over);
        assert_eq!(jsjson::stringify(&merged), r#"{"gate_bypass":"full","o":{"a":1,"b":3}}"#);
    }

    // R5 port of scripts/tests/test_ship_visibility.mjs — the normalizer had
    // no test at all; only its absent→"off" case was incidentally asserted
    // from the status renderer.
    #[test]
    fn ship_visibility_passes_the_two_known_values_and_normalizes_the_rest() {
        let cfg = |v: Value| -> Map<String, Value> {
            let mut m = Map::new();
            m.insert("ship_visibility".into(), v);
            m
        };
        assert_eq!(ship_visibility(&Map::new()), "off", "absent is off");
        assert_eq!(ship_visibility(&cfg(json!(null))), "off", "null is off");
        assert_eq!(ship_visibility(&cfg(json!("off"))), "off");
        assert_eq!(
            ship_visibility(&cfg(json!("draft-pr"))),
            "draft-pr",
            "draft-pr must survive — the whole point of the setting"
        );
        // Unrecognized values of every JS type normalize to off. (The warning
        // line's bytes are a pinned contract, but it goes to the process's
        // real stderr, so it is asserted at the status surface rather than
        // here; this case pins the VALUE half.)
        for bad in [json!("DRAFT-PR"), json!("on"), json!(true), json!(3), json!({"a":1})] {
            assert_eq!(ship_visibility(&cfg(bad.clone())), "off", "unrecognized: {bad}");
        }
    }

    #[test]
    fn bypass_levels_normalize() {
        let mk = |v: Value| {
            let mut m = Map::new();
            m.insert("gate_bypass".into(), v);
            m
        };
        assert_eq!(bypass_level(&mk(json!("total"))), "total");
        assert_eq!(bypass_level(&mk(json!("full"))), "full");
        assert_eq!(bypass_level(&mk(json!(true))), "normal");
        assert_eq!(bypass_level(&mk(json!("on"))), "normal");
        assert_eq!(bypass_level(&mk(json!(false))), "off");
        assert_eq!(bypass_level(&mk(json!("weird"))), "off");
        assert_eq!(bypass_level(&Map::new()), "off");
    }
}
