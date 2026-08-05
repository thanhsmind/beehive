// state — the state-layer reads `status --brief` needs (read_state /
// read_config / bypass_level / ship_visibility).
//
// Every read here is infallible and total:
//
//   * corrupt JSON — `fsutil::warn_corrupt_json` logs a warning and the read
//     falls back to defaults.
//   * a non-object `approved_gates` — `spread_gates` (D2,
//     docs/history/js-parity-cleanup/CONTEXT.md) merges only for the object
//     shape; every other shape falls back to defaults, so no input shape is
//     left without an answer.

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

    // approved_gates: an object merges its keys over the defaults; every
    // other shape (missing/null/bool/number/string/array) falls back to the
    // defaults (D2, docs/history/js-parity-cleanup/CONTEXT.md).
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

/// Rust-native gate merge (D2, docs/history/js-parity-cleanup/CONTEXT.md):
/// `approved_gates` merges over the defaults only when it is a JSON object —
/// its keys win, in insertion order. EVERY other shape (missing, null, bool,
/// number, string, array) yields the defaults untouched.
///
/// This replaces two diverging JS-spread emulations: one that char/index-keyed
/// strings and arrays, and one that bailed to a Node delegate that no longer
/// exists. Neither the Rust store nor a legitimate writer ever produces a
/// non-object `approved_gates`; a hand-corrupted state.json holding one used
/// to surface char/index-indexed gate keys and now reads as defaults instead
/// — a deliberate, logged behavior change, not a compatibility gap.
pub(crate) fn spread_gates(value: Option<&Value>) -> Map<String, Value> {
    match value {
        Some(Value::Object(overlay)) => {
            let mut merged = default_gates();
            for (k, v) in overlay {
                merged.insert(k.clone(), v.clone());
            }
            merged
        }
        _ => default_gates(),
    }
}

// ── config (gate_bypass / ship_visibility slice of readConfig) ─────────────

/// Overlay wins; plain objects merge recursively; arrays replace wholesale;
/// scalars replace.
/// (pub(crate): hooks/compaction.rs's fail-open config-read twin needs the
/// SAME merge, and a second copy of it would be a second answer to "which
/// value wins".)
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

/// docViewerPrefix (decision 4205835b, docs/history/doc-viewer-links/plan.md):
/// opt-in mdview URL prefix. `doc_viewer.base_url` + `doc_viewer.project`
/// join as `<base>/p/<project>` — mdview's own URL layout, so an agent that
/// has this prefix can turn a repo-relative doc path into a link the user
/// can click instead of a bare path.
///
/// Absent key => None, silent: that IS the default, not a mistake. A key
/// that LOOKS configured but is missing a field, holds an empty field after
/// trimming, or is the wrong shape => None PLUS one stderr line naming the
/// key (the same shape `ship_visibility` uses above for an unrecognized
/// value) — a half-set key that quietly does nothing is the trap worth
/// warning about.
pub fn doc_viewer_prefix(config: &Map<String, Value>) -> Option<String> {
    let obj = match config.get("doc_viewer") {
        None => return None,
        Some(Value::Object(m)) => m,
        Some(_) => return warn_half_set_doc_viewer(),
    };
    let (base_url, project) = match (obj.get("base_url"), obj.get("project")) {
        (Some(Value::String(b)), Some(Value::String(p))) => (b, p),
        _ => return warn_half_set_doc_viewer(),
    };
    // One trailing slash off base_url (a doubled slash is left to the
    // author to notice); every surrounding slash off project, since a
    // leading OR trailing one would double up against the joined `/p/`.
    let base = base_url.strip_suffix('/').unwrap_or(base_url);
    let project = project.trim_matches('/');
    if base.is_empty() || project.is_empty() {
        return warn_half_set_doc_viewer();
    }
    Some(format!("{base}/p/{project}"))
}

fn warn_half_set_doc_viewer() -> Option<String> {
    eprint!(
        "config: doc_viewer is set but incomplete in .bee/config.json — base_url and project must both be non-empty strings; doc links disabled.\n"
    );
    None
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

    /// Corrupt state.json warns on stderr and falls back to the default
    /// state.
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

    /// D2 (docs/history/js-parity-cleanup/CONTEXT.md): the Rust-native gate
    /// merge, for every shape `approved_gates` can hold. An object merges its
    /// keys over the defaults; every other shape — including the array/string
    /// cases that used to spread to index-keyed JS exotica — falls back to
    /// the defaults untouched.
    #[test]
    fn approved_gates_merge_is_object_or_defaults() {
        let defaults = r#"{"context":false,"shape":false,"execution":false,"review":false}"#;
        let s = |v: Option<Value>| jsjson::stringify(&Value::Object(spread_gates(v.as_ref())));
        assert_eq!(s(None), defaults);
        assert_eq!(s(Some(json!(null))), defaults);
        assert_eq!(s(Some(json!(false))), defaults);
        assert_eq!(s(Some(json!(0))), defaults);
        assert_eq!(s(Some(json!(""))), defaults);
        assert_eq!(s(Some(json!(true))), defaults);
        assert_eq!(s(Some(json!(7))), defaults);
        // Exotic shapes that used to spread to index keys now fall to defaults.
        assert_eq!(s(Some(json!([true, false]))), defaults);
        assert_eq!(s(Some(json!("ab"))), defaults);
        // An object still merges its keys over the defaults.
        assert_eq!(
            s(Some(json!({"context": true}))),
            r#"{"context":true,"shape":false,"execution":false,"review":false}"#
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
            r#"{"context":false,"shape":false,"execution":false,"review":false}"#
        );
    }

    #[test]
    fn overlay_wins_scalar_and_deep_merges_objects() {
        let base = serde_json::from_str::<Value>(r#"{"gate_bypass":"off","o":{"a":1,"b":2}}"#).unwrap();
        let over = serde_json::from_str::<Value>(r#"{"gate_bypass":"full","o":{"b":3}}"#).unwrap();
        let merged = merge_config_overlay(&base, &over);
        assert_eq!(jsjson::stringify(&merged), r#"{"gate_bypass":"full","o":{"a":1,"b":3}}"#);
    }

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

    fn doc_viewer_config(v: Value) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("doc_viewer".into(), v);
        m
    }

    #[test]
    fn doc_viewer_prefix_joins_base_url_and_project() {
        let cfg = doc_viewer_config(json!({"base_url": "http://10.255.255.254:7700", "project": "beedashboard"}));
        assert_eq!(
            doc_viewer_prefix(&cfg).as_deref(),
            Some("http://10.255.255.254:7700/p/beedashboard")
        );
    }

    #[test]
    fn doc_viewer_prefix_trims_one_trailing_slash_off_base_url() {
        let cfg = doc_viewer_config(json!({"base_url": "http://host:7700/", "project": "p"}));
        assert_eq!(doc_viewer_prefix(&cfg).as_deref(), Some("http://host:7700/p/p"));
    }

    #[test]
    fn doc_viewer_prefix_strips_surrounding_slashes_off_project() {
        let cfg = doc_viewer_config(json!({"base_url": "http://host:7700", "project": "/beedashboard/"}));
        assert_eq!(
            doc_viewer_prefix(&cfg).as_deref(),
            Some("http://host:7700/p/beedashboard")
        );
    }

    /// Absent key: None, and nothing recorded — that IS the default.
    #[test]
    fn doc_viewer_prefix_absent_is_silently_none() {
        assert_eq!(doc_viewer_prefix(&Map::new()), None);
    }

    /// Half-set (one field only): None, plus a warning — asserted on the
    /// VALUE half here, same convention as `ship_visibility`'s test above;
    /// the warning's bytes are pinned wherever the real stderr is asserted.
    #[test]
    fn doc_viewer_prefix_half_set_warns_and_returns_none() {
        let cfg = doc_viewer_config(json!({"base_url": "http://host:7700"}));
        assert_eq!(doc_viewer_prefix(&cfg), None, "project missing");
        let cfg = doc_viewer_config(json!({"project": "p"}));
        assert_eq!(doc_viewer_prefix(&cfg), None, "base_url missing");
        let cfg = doc_viewer_config(json!({"base_url": 7700, "project": "p"}));
        assert_eq!(doc_viewer_prefix(&cfg), None, "base_url not a string");
    }

    #[test]
    fn doc_viewer_prefix_empty_string_field_warns_and_returns_none() {
        let cfg = doc_viewer_config(json!({"base_url": "", "project": "p"}));
        assert_eq!(doc_viewer_prefix(&cfg), None, "empty base_url");
        let cfg = doc_viewer_config(json!({"base_url": "http://host:7700", "project": ""}));
        assert_eq!(doc_viewer_prefix(&cfg), None, "empty project");
        // A base_url that is nothing but a trailing slash trims to empty.
        let cfg = doc_viewer_config(json!({"base_url": "/", "project": "p"}));
        assert_eq!(doc_viewer_prefix(&cfg), None, "base_url is only a slash");
    }

    #[test]
    fn doc_viewer_prefix_non_object_warns_and_returns_none() {
        for bad in [json!("http://host:7700"), json!(true), json!([1, 2])] {
            let cfg = doc_viewer_config(bad.clone());
            assert_eq!(doc_viewer_prefix(&cfg), None, "non-object doc_viewer: {bad}");
        }
    }

    /// config.local.json overlays tracked config.json for doc_viewer, through
    /// the merged reader — the same overlay-wins contract every other config
    /// key gets.
    #[test]
    fn doc_viewer_prefix_local_overlay_wins_over_tracked_config() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(
            tmp.path().join(".bee").join("config.json"),
            r#"{"doc_viewer":{"base_url":"http://tracked:7700","project":"tracked-project"}}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join(".bee").join("config.local.json"),
            r#"{"doc_viewer":{"base_url":"http://local:7700","project":"local-project"}}"#,
        )
        .unwrap();
        let cfg = read_config_raw(tmp.path());
        assert_eq!(
            doc_viewer_prefix(&cfg).as_deref(),
            Some("http://local:7700/p/local-project")
        );
    }
}
