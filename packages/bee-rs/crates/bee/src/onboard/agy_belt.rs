use crate::jsjson;
use serde_json::{json, Map, Value};
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeError(pub String);

impl fmt::Display for MergeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for MergeError {}

fn hook_command(neutral_answer: &str) -> String {
    let arm = super::hooks_wiring::main_checkout_arm("agy-belt", "..", "");
    format!(
        "for b in ../.bee/bin/bee ../.bee/bin/bee.exe; do [ -x \"$b\" ] && exec \"$b\" hook agy-belt; done; {arm}; printf '%s\\n' '{neutral_answer}'"
    )
}

fn handler(neutral_answer: &str) -> Value {
    json!({
        "type": "command",
        "command": hook_command(neutral_answer),
        "timeout": 30
    })
}

pub fn render_bee_key() -> Value {
    json!({
        "SessionStart": [handler("{}")],
        "PreInvocation": [handler("{}")],
        "PreToolUse": [{
            "matcher": "",
            "hooks": [handler("{\"decision\":\"ask\"}")]
        }],
        "PostToolUse": [{
            "matcher": "",
            "hooks": [handler("{}")]
        }],
        "Stop": [handler("{}")]
    })
}

pub fn merge_into(existing: Option<&str>) -> Result<String, MergeError> {
    let bee = render_bee_key();
    let mut obj = match existing {
        None => Map::new(),
        Some(raw) => {
            let parsed: Value = serde_json::from_str(raw)
                .map_err(|e| MergeError(format!(".agents/hooks.json is not valid JSON: {e}")))?;
            match parsed {
                Value::Object(map) => map,
                _ => return Err(MergeError(".agents/hooks.json must be a JSON object".to_string())),
            }
        }
    };
    obj.insert("bee".to_string(), bee);
    Ok(format!("{}\n", jsjson::stringify_pretty(&Value::Object(obj))))
}

pub fn needs_apply(hooks_path: &Path) -> bool {
    match std::fs::read_to_string(hooks_path) {
        Err(_) => true,
        Ok(raw) => match serde_json::from_str::<Value>(&raw) {
            Ok(Value::Object(obj)) => obj.get("bee") != Some(&render_bee_key()),
            _ => true,
        },
    }
}

pub fn merge_file(hooks_path: &Path) -> Result<Option<String>, MergeError> {
    match std::fs::read_to_string(hooks_path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => merge_into(None).map(Some),
        Err(e) => Err(MergeError(format!(".agents/hooks.json cannot be read: {e}"))),
        Ok(raw) => {
            let parsed: Value = serde_json::from_str(&raw)
                .map_err(|e| MergeError(format!(".agents/hooks.json is not valid JSON: {e}")))?;
            let Value::Object(ref obj) = parsed else {
                return Err(MergeError(".agents/hooks.json must be a JSON object".to_string()));
            };
            if obj.get("bee") == Some(&render_bee_key()) {
                return Ok(None);
            }
            merge_into(Some(&raw)).map(Some)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_into_fresh_writes_bee_key_only() {
        let out = merge_into(None).unwrap();
        assert!(out.ends_with('\n'));
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed.as_object().unwrap().len(), 1);
        assert_eq!(parsed["bee"], render_bee_key());
    }

    #[test]
    fn merge_into_keeps_owner_keys_unchanged() {
        let existing = r#"{
  "owner": {
    "PreToolUse": [{"matcher": "run_command", "hooks": [{"type": "command", "command": "echo hi", "timeout": 5}]}]
  },
  "extra": [1, 2, 3]
}"#;
        let out = merge_into(Some(existing)).unwrap();
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            parsed["owner"],
            json!({
                "PreToolUse": [{"matcher": "run_command", "hooks": [{"type": "command", "command": "echo hi", "timeout": 5}]}]
            })
        );
        assert_eq!(parsed["extra"], json!([1, 2, 3]));
        assert_eq!(parsed["bee"], render_bee_key());
    }

    #[test]
    fn merge_into_replaces_stale_bee_key() {
        let existing = r#"{
  "owner": {"SessionStart": []},
  "bee": {"SessionStart": [{"type": "command", "command": "old", "timeout": 1}]}
}"#;
        let out = merge_into(Some(existing)).unwrap();
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["owner"], json!({"SessionStart": []}));
        assert_eq!(parsed["bee"], render_bee_key());
    }

    #[test]
    fn merge_into_refuses_malformed_and_non_object() {
        let err_bad = merge_into(Some("{not json")).unwrap_err();
        assert!(err_bad.to_string().contains(".agents/hooks.json"), "{err_bad}");
        let err_arr = merge_into(Some("[1, 2]")).unwrap_err();
        assert!(err_arr.to_string().contains(".agents/hooks.json"), "{err_arr}");
    }

    #[test]
    fn merge_into_is_idempotent() {
        let first = merge_into(Some(r#"{"owner": {"a": 1}}"#)).unwrap();
        let second = merge_into(Some(&first)).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn checked_in_hooks_json_bee_key_matches_render() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../.agents/hooks.json");
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let parsed: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed["bee"], render_bee_key());
    }
}
