//! validate_args — port of `.bee/bin/lib/validate-args.mjs` (rust-port-11,
//! CONTEXT.md D2/D7): given a command-registry entry's JSON-Schema
//! `parameters` and a parsed-args map, decide whether the call is
//! well-formed — before anything dispatches or executes. Never panics:
//! always returns a structured problem list (empty = ok).
//!
//! `validate-args.mjs` is FROZEN for the duration of the rust-port feature
//! (D1) — this module is conformance-checked through the write-guard hook's
//! CLI-shape corpus (`crates/queen-bee/tests/writeguard_bash.rs`, driving
//! the sha256-verified node oracle), never edited to "improve" on it.
//!
//! Shape note: the mjs `validate()` returns `{ok:true}` or
//! `{ok:false, error, problems}` where `error` is always the FIRST problem.
//! Here that collapses to `Vec<Problem>` — empty means ok, `problems[0]` is
//! the mjs `error` — because the only extra field (`error.command`) is never
//! read by the one caller this cell ports (bee-write-guard.mjs's
//! checkCliShape builds its message from `entry.name` directly).

use serde_json::Value;

/// One validation problem: `field` is `None` for the schema-level problem
/// (mjs `field: null`), `Some(flag)` otherwise.
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    pub field: Option<String>,
    pub reason: String,
}

/// Port of `isValidParameterSchema` — structural check that `parameters` is
/// JSON-Schema in the exact shape D3 requires: `{type:"object", properties,
/// required}`, every `required` name present in `properties`, every property
/// carrying a `type`.
pub fn is_valid_parameter_schema(schema: &Value) -> bool {
    let Value::Object(obj) = schema else { return false };
    if obj.get("type").and_then(Value::as_str) != Some("object") {
        return false;
    }
    let Some(Value::Object(properties)) = obj.get("properties") else { return false };
    let Some(Value::Array(required)) = obj.get("required") else { return false };
    for field in required {
        let Some(name) = field.as_str() else { return false };
        if !properties.contains_key(name) {
            return false;
        }
    }
    for prop_schema in properties.values() {
        if prop_schema.get("type").and_then(Value::as_str).is_none() {
            return false;
        }
    }
    true
}

/// mjs `isPresent`: present unless `undefined` (absent), `null`, or `''`.
fn is_present(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}

/// JS `Number(str)` finiteness for the CLI's string encoding of numbers:
/// trimmed decimal/exponent forms parse; `Infinity` spellings and garbage
/// read as non-finite; hex/octal/binary prefixes parse as integers (JS
/// Number accepts them).
fn js_number_is_finite(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false; // callers guard on non-empty; Number('') is 0 but never reaches here
    }
    for (prefix, radix) in [("0x", 16), ("0X", 16), ("0o", 8), ("0O", 8), ("0b", 2), ("0B", 2)] {
        if let Some(rest) = t.strip_prefix(prefix) {
            return !rest.is_empty() && i128::from_str_radix(rest, radix).is_ok();
        }
    }
    if matches!(t, "Infinity" | "+Infinity" | "-Infinity") {
        return false;
    }
    t.parse::<f64>().map(|v| v.is_finite()).unwrap_or(false)
}

/// Port of `typeMatches` — CLI flags arrive as strings (argv parsing never
/// produces real booleans or numbers), so a schema of type
/// "boolean"/"number" must still accept the CLI's own string encoding.
fn type_matches(json_type: &str, value: &Value) -> bool {
    match json_type {
        "string" => value.is_string(),
        "boolean" => {
            value.is_boolean() || matches!(value.as_str(), Some("true") | Some("false"))
        }
        "number" | "integer" => match value {
            Value::Number(n) => n.as_f64().map(f64::is_finite).unwrap_or(false),
            Value::String(s) => !s.trim().is_empty() && js_number_is_finite(s),
            _ => false,
        },
        // comma-separated CLI convention
        "array" => value.is_array() || value.is_string(),
        _ => true,
    }
}

/// JS `Array.prototype.join(', ')` over an enum annotation's members.
fn enum_join(members: &[Value]) -> String {
    members
        .iter()
        .map(|m| match m {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Port of `validate(commandEntry, parsedArgs)` over the entry's
/// `parameters` schema. Collects EVERY problem (ce-1, cli-ergonomics D1) —
/// required misses first, in `schema.required` order, then value problems in
/// `parsed_args` insertion order (the mjs object iteration order for
/// string-keyed flags). Empty result = the mjs `{ok:true}`.
pub fn validate(parameters: &Value, parsed_args: &[(String, Value)]) -> Vec<Problem> {
    if !is_valid_parameter_schema(parameters) {
        return vec![Problem {
            field: None,
            reason: "command has no valid JSON-Schema parameters".to_string(),
        }];
    }

    let properties = parameters.get("properties").and_then(Value::as_object).expect("checked above");
    let required: Vec<&str> = parameters
        .get("required")
        .and_then(Value::as_array)
        .expect("checked above")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    let mut problems: Vec<Problem> = Vec::new();

    for field in &required {
        let value = parsed_args.iter().find(|(k, _)| k == field).map(|(_, v)| v);
        if !is_present(value) {
            problems.push(Problem { field: Some((*field).to_string()), reason: "required, missing".to_string() });
        }
    }

    for (field, value) in parsed_args {
        let Some(prop_schema) = properties.get(field) else {
            // unknown-flag rejection is the dispatcher's own concern — never
            // duplicated here (packages-engine-move-3, C7/C8/C9).
            continue;
        };
        let json_type = prop_schema.get("type").and_then(Value::as_str).unwrap_or("");
        if !type_matches(json_type, value) {
            problems.push(Problem {
                field: Some(field.clone()),
                reason: format!("invalid type, expected {json_type}"),
            });
            continue; // a mistyped value can't also be enum-checked meaningfully
        }
        // Enum enforcement is scoped to REQUIRED fields only (DB3 guard) —
        // enforcing enum on ANY present field would silently re-open the
        // STDOUT leak DB3 closed. See validate-args.mjs's own comment.
        if required.contains(&field.as_str()) {
            if let Some(Value::Array(members)) = prop_schema.get("enum") {
                if !members.iter().any(|m| m == value) {
                    problems.push(Problem {
                        field: Some(field.clone()),
                        reason: format!("invalid value, expected one of {}", enum_join(members)),
                    });
                }
            }
        }
    }

    problems
}
