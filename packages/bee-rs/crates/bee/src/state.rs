// state — Rust port of the state-layer reads `status --brief` needs
// (state.mjs readState / readConfig / bypassLevel / shipVisibility).
//
// CUTOVER (2026-08-01). The strangler rule used to read: any input Node
// handles via warn-with-V8-message (corrupt JSON) or via JS spread exotica (a
// truthy non-object approved_gates) makes the native path bail before
// emitting anything, and the whole command re-runs under Node. Node is gone.
// Both classes are native now:
//
//   * corrupt JSON — `fsutil::warn_corrupt_json` says what Node's warning
//     said, in our words, and the read falls back to defaults exactly as
//     `readJson(file, null)` did.
//   * a truthy non-object `approved_gates` — the JS object spread is
//     reproduced directly (see `spread_gates`), so no input shape is left
//     without an answer.
//
// These reads are infallible now — the single-variant bail enum they used to
// thread was never constructed after the cutover above, so the 2026-08-04
// cleanup deleted it and the signatures return their values directly.

use crate::fsutil::{read_json, warn_corrupt_json, ReadJson};
use crate::jsjson;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

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

pub fn read_state_brief(root: &Path) -> BriefState {
    let state_file = state_path(root);
    let file_state = match read_json(&state_file) {
        ReadJson::Missing => None,
        // readJson(state.json, null) warned and returned null; the caller then
        // fell through to defaultState(). Same shape, our wording.
        ReadJson::Corrupt => {
            warn_corrupt_json(&state_file);
            None
        }
        ReadJson::Parsed(v) => match v {
            Value::Object(m) => Some(m),
            _ => None, // non-object parses fall back to defaultState(), silently
        },
    };

    let default_phase = json!("idle");
    let Some(state) = file_state else {
        return BriefState {
            phase: default_phase,
            feature: Value::Null,
            mode: Value::Null,
            gates: default_gates(),
            route: Value::Null,
        };
    };

    // { ...defaultState(), ...state } — file value wins whenever the KEY is
    // present, even when the value is null.
    let pick = |key: &str, default: Value| -> Value {
        state.get(key).cloned().unwrap_or(default)
    };

    // approved_gates: { ...defaults, ...(state.approved_gates || {}) }.
    // Falsy (null/false/0/"") -> defaults. A truthy non-object spreads an
    // exotic key set in JS, which `spread_gates` now reproduces instead of
    // bailing (cutover note at the top of this file).
    let gates = spread_gates(state.get("approved_gates"));

    // coerceLegacyPhase: 'validating' -> 'planning' (D13).
    let mut phase = pick("phase", default_phase);
    if phase == json!("validating") {
        phase = json!("planning");
    }

    BriefState {
        phase,
        feature: pick("feature", Value::Null),
        mode: pick("mode", Value::Null),
        gates,
        route: match state.get("route") {
            // state.route ?? null — null/undefined both land on null.
            None | Some(Value::Null) => Value::Null,
            Some(v) => v.clone(),
        },
    }
}

/// `{ ...defaultGates(), ...(value || {}) }` for EVERY JS value, not just the
/// object case. The spread copies own enumerable properties, so:
///   * falsy (absent / null / false / 0 / "") — `|| {}` makes it the empty
///     object: defaults, untouched.
///   * an object — its keys win over the defaults, in insertion order.
///   * an array — index keys "0".."n-1" carrying the elements.
///   * a non-empty string — index keys "0".."n-1" carrying single characters.
///   * any other truthy primitive (true, a non-zero number) — primitives have
///     no own enumerable properties, so the spread copies nothing: defaults.
///
/// This used to bail to Node for the last three cases, because the port
/// refused to guess at JS spread semantics while Node could still answer.
/// Divergence worth naming: a string is indexed here by Unicode scalar, where
/// JS indexes by UTF-16 code unit — they differ only for astral characters in
/// a value that is already nonsense (`approved_gates` holding a string).
fn spread_gates(value: Option<&Value>) -> Map<String, Value> {
    let mut merged = default_gates();
    match value {
        None | Some(Value::Null) | Some(Value::Bool(false)) => {}
        Some(Value::String(s)) if s.is_empty() => {}
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => {}
        Some(Value::Object(overlay)) => {
            for (k, v) in overlay {
                merged.insert(k.clone(), v.clone());
            }
        }
        Some(Value::Array(items)) => {
            for (i, v) in items.iter().enumerate() {
                merged.insert(i.to_string(), v.clone());
            }
        }
        Some(Value::String(s)) => {
            for (i, c) in s.chars().enumerate() {
                merged.insert(i.to_string(), Value::String(c.to_string()));
            }
        }
        // true, or a non-zero number: nothing to copy.
        Some(_) => {}
    }
    merged
}

// ── config (gate_bypass / ship_visibility slice of readConfig) ─────────────

/// mergeConfigOverlay: overlay wins; plain objects merge recursively; arrays
/// replace wholesale; scalars replace.
/// (pub(crate) since the cutover: hooks/compaction.rs's fail-open readConfig
/// twin needs the SAME merge, and a second copy of it would be a second
/// answer to "which value wins".)
pub(crate) fn merge_config_overlay(base: &Value, overlay: &Value) -> Value {
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
pub fn read_config_raw(root: &Path) -> Map<String, Value> {
    let read_obj = |file: PathBuf| -> Option<Map<String, Value>> {
        match read_json(&file) {
            ReadJson::Missing => None,
            // readConfig's `readJson(file, {})` warned and fell back; a
            // corrupt config therefore reads as "no config here", exactly
            // like an absent one, and the merge continues.
            ReadJson::Corrupt => {
                warn_corrupt_json(&file);
                None
            }
            ReadJson::Parsed(Value::Object(m)) => Some(m),
            ReadJson::Parsed(_) => None,
        }
    };
    let tracked = read_obj(root.join(".bee").join("config.json")).unwrap_or_default();
    let overlay = read_obj(root.join(".bee").join("config.local.json"));
    let mut merged = match overlay {
        Some(over) => match merge_config_overlay(&Value::Object(tracked), &Value::Object(over)) {
            Value::Object(m) => m,
            _ => unreachable!("object-over-object merge yields an object"),
        },
        None => tracked,
    };
    merged.shift_remove("advisor");
    merged
}

/// hookEnabled: `config.hooks[name] !== false` over merged tracked+overlay
/// config — enabled unless the file explicitly carries `false` (unknown
/// names default enabled; DEFAULT_HOOKS are all true, so the merge with
/// defaults never changes this predicate).
pub fn hook_enabled(root: &Path, name: &str) -> bool {
    let config = read_config_raw(root);
    !matches!(
        config.get("hooks").and_then(|h| h.get(name)),
        Some(Value::Bool(false))
    )
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
        let s = read_state_brief(tmp.path());
        assert_eq!(s.phase, json!("idle"));
        assert_eq!(s.feature, Value::Null);
        assert_eq!(jsjson::stringify(&Value::Object(s.gates)),
            r#"{"context":false,"shape":false,"execution":false,"review":false}"#);
    }

    #[test]
    fn file_keys_override_defaults_and_legacy_phase_coerces() {
        let tmp = tempfile::tempdir().unwrap();
        write_state(tmp.path(), r#"{"phase":"validating","feature":"f1","approved_gates":{"shape":true,"extra":1}}"#);
        let s = read_state_brief(tmp.path());
        assert_eq!(s.phase, json!("planning"));
        assert_eq!(s.feature, json!("f1"));
        // Default key order first, extras appended in file order.
        assert_eq!(jsjson::stringify(&Value::Object(s.gates)),
            r#"{"context":false,"shape":true,"execution":false,"review":false,"extra":1}"#);
    }

    /// CUTOVER: this used to assert a bail to Node. Node's readJson warned
    /// and returned null, and defaultState() took over — so that is what the
    /// native read does now, warning in our own words on stderr.
    #[test]
    fn corrupt_state_warns_and_falls_back_to_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        write_state(tmp.path(), "{broken");
        let s = read_state_brief(tmp.path());
        assert_eq!(s.phase, json!("idle"));
        assert_eq!(s.feature, Value::Null);
        assert_eq!(s.mode, Value::Null);
        assert_eq!(s.route, Value::Null);
        assert_eq!(
            jsjson::stringify(&Value::Object(s.gates)),
            r#"{"context":false,"shape":false,"execution":false,"review":false}"#
        );
    }

    /// A corrupt config reads as "no config here" — the same value an absent
    /// one produces — so every derived predicate keeps its default.
    #[test]
    fn corrupt_config_warns_and_reads_as_empty() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(tmp.path().join(".bee").join("config.json"), "{broken").unwrap();
        let cfg = read_config_raw(tmp.path());
        assert!(cfg.is_empty());
        assert_eq!(bypass_level(&cfg), "off");
        assert!(hook_enabled(tmp.path(), "session-init"), "unknown names default enabled");
    }

    /// A corrupt OVERLAY leaves the tracked config standing, exactly as
    /// `readJson(config.local.json, null)`'s fallback did.
    #[test]
    fn corrupt_overlay_leaves_the_tracked_config_standing() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(
            tmp.path().join(".bee").join("config.json"),
            r#"{"gate_bypass":"full"}"#,
        )
        .unwrap();
        std::fs::write(tmp.path().join(".bee").join("config.local.json"), "nope").unwrap();
        let cfg = read_config_raw(tmp.path());
        assert_eq!(bypass_level(&cfg), "full");
    }

    /// The JS object spread, for every shape `approved_gates` can hold. Each
    /// expectation is what `node -p 'JSON.stringify({...d, ...(v||{})})'`
    /// prints. These four truthy non-object cases used to bail to Node.
    #[test]
    fn approved_gates_spread_covers_every_js_shape() {
        let defaults = r#"{"context":false,"shape":false,"execution":false,"review":false}"#;
        let s = |v: Option<Value>| {
            jsjson::stringify(&Value::Object(spread_gates(v.as_ref())))
        };
        assert_eq!(s(None), defaults);
        assert_eq!(s(Some(json!(null))), defaults);
        assert_eq!(s(Some(json!(false))), defaults);
        assert_eq!(s(Some(json!(0))), defaults);
        assert_eq!(s(Some(json!(""))), defaults);
        // Primitives carry no own enumerable properties.
        assert_eq!(s(Some(json!(true))), defaults);
        assert_eq!(s(Some(json!(7))), defaults);
        // Arrays and strings spread to index keys.
        assert_eq!(
            s(Some(json!([true, false]))),
            r#"{"context":false,"shape":false,"execution":false,"review":false,"0":true,"1":false}"#
        );
        assert_eq!(
            s(Some(json!("ab"))),
            r#"{"context":false,"shape":false,"execution":false,"review":false,"0":"a","1":"b"}"#
        );
    }

    /// And through the real reader, so the whole record still renders.
    #[test]
    fn exotic_approved_gates_no_longer_bails() {
        let tmp = tempfile::tempdir().unwrap();
        write_state(tmp.path(), r#"{"phase":"executing","approved_gates":"ab"}"#);
        let s = read_state_brief(tmp.path());
        assert_eq!(s.phase, json!("executing"));
        assert_eq!(
            jsjson::stringify(&Value::Object(s.gates)),
            r#"{"context":false,"shape":false,"execution":false,"review":false,"0":"a","1":"b"}"#
        );
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
