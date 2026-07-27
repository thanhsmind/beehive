//! hookconfig — the narrow slice of `.bee/bin/lib/state.mjs`'s
//! `readConfig`/`hookEnabled` that the queen-bee hook runtime needs
//! (rust-port-7). `.bee/bin/lib/state.mjs` is FROZEN (D1); this module
//! reproduces its `hookEnabled(root, name)` semantics exactly:
//! `config.hooks[name] !== false` — strict inequality against the literal
//! boolean `false`. Any other value (absent, `true`, a stray non-boolean)
//! means enabled. `state.mjs`'s own `DEFAULT_HOOKS` spread never changes
//! this outcome (every default value is `true`), so it is intentionally not
//! reproduced here — see the comment on [`hook_enabled`].
//!
//! Also mirrors the config.local.json overlay (`localConfigPath`,
//! `mergeConfigOverlay`, `LOCAL_ONLY_CONFIG_NAMESPACES` includes `hooks`) so
//! a machine-local hook toggle is honored exactly like the mjs source.

use std::fs;
use std::path::Path;

use serde_json::{Map, Value};

fn read_json_object(path: &Path) -> Option<Map<String, Value>> {
    let text = fs::read_to_string(path).ok()?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    match serde_json::from_str::<Value>(text).ok()? {
        Value::Object(m) => Some(m),
        _ => None,
    }
}

/// Port of `mergeConfigOverlay` (overlay wins on every conflict; arrays
/// replace wholesale; a non-object/array overlay value replaces the base
/// value; `base` is never mutated).
fn merge_overlay(base: &Value, overlay: &Value) -> Value {
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
                    (Some(bv @ Value::Object(_)), Value::Object(_)) => merge_overlay(bv, value),
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

/// Port of `hookEnabled(root, name)`. `DEFAULT_HOOKS` in the mjs source is
/// intentionally NOT reproduced here: every one of its entries is `true`,
/// and `readConfig` only ever spreads it BEFORE the tracked/overlay values,
/// so it can never introduce a `false` that wasn't already present in
/// `config.json`/`config.local.json`. Reproducing it would be a no-op that
/// only risks silently drifting from the frozen source if it ever changed —
/// the `!== false` check below is the entire observable contract.
pub fn hook_enabled(root: &Path, name: &str) -> bool {
    let tracked = read_json_object(&root.join(".bee").join("config.json"))
        .map(Value::Object)
        .unwrap_or_else(|| Value::Object(Map::new()));
    let overlay = read_json_object(&root.join(".bee").join("config.local.json")).map(Value::Object);
    let config = match overlay {
        Some(o) => merge_overlay(&tracked, &o),
        None => tracked,
    };
    let value = config.get("hooks").and_then(|h| h.get(name));
    !matches!(value, Some(Value::Bool(false)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write(root: &Path, rel: &str, value: &Value) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_string(value).unwrap()).unwrap();
    }

    #[test]
    fn hook_enabled_defaults_true_when_config_absent() {
        let dir = tempfile::tempdir().unwrap();
        assert!(hook_enabled(dir.path(), "tools-logger"));
    }

    #[test]
    fn hook_enabled_false_only_on_literal_false() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            ".bee/config.json",
            &json!({ "hooks": { "tools-logger": false, "codex-subagent-audit": 0 } }),
        );
        assert!(!hook_enabled(dir.path(), "tools-logger"));
        // strict `!== false`: a non-boolean falsy-looking value (0) still
        // counts as enabled, matching JS strict inequality.
        assert!(hook_enabled(dir.path(), "codex-subagent-audit"));
    }

    #[test]
    fn hook_enabled_local_overlay_wins_over_tracked() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), ".bee/config.json", &json!({ "hooks": { "tools-logger": true } }));
        write(dir.path(), ".bee/config.local.json", &json!({ "hooks": { "tools-logger": false } }));
        assert!(!hook_enabled(dir.path(), "tools-logger"));
    }
}
