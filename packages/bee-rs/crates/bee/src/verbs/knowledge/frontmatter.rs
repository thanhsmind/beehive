// the frontmatter emitter and its parser
//
// Split out of the single 4.4k-line verbs/knowledge.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── emitter (emitFrontmatter — the D12 subset's source of truth) ──────────

pub(crate) fn is_plain_safe(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    if value != js_trim(value) {
        return false;
    }
    // /[:#"'\\\[\]{},\t\r\n]/
    if value
        .chars()
        .any(|c| matches!(c, ':' | '#' | '"' | '\'' | '\\' | '[' | ']' | '{' | '}' | ',' | '\t' | '\r' | '\n'))
    {
        return false;
    }
    // /^[-?&*!|>%@`]/
    if matches!(
        value.chars().next(),
        Some('-' | '?' | '&' | '*' | '!' | '|' | '>' | '%' | '@' | '`')
    ) {
        return false;
    }
    !(value == "true" || value == "false" || value == "null")
}

/// emitScalar — Err(()) mirrors the JS throw (caught by the round-trip guard).
pub(crate) fn emit_scalar(value: &Value) -> Result<String, ()> {
    match value {
        Value::Bool(true) => Ok("true".to_string()),
        Value::Bool(false) => Ok("false".to_string()),
        Value::String(s) => Ok(if is_plain_safe(s) { s.clone() } else { js_quote_str(s) }),
        _ => Err(()),
    }
}

pub(crate) fn emit_value(value: &Value) -> Result<String, ()> {
    match value {
        Value::Array(items) => {
            let parts = items.iter().map(emit_scalar).collect::<Result<Vec<_>, ()>>()?;
            Ok(format!("[{}]", parts.join(", ")))
        }
        other => emit_scalar(other),
    }
}

pub(crate) fn emit_entries(lines: &mut Vec<String>, map: &Map<String, Value>, order: &[&str], indent: &str) -> Result<(), ()> {
    let known: Vec<&String> = order
        .iter()
        .filter_map(|k| map.keys().find(|key| key.as_str() == *k))
        .collect();
    let mut unknown: Vec<&String> = map
        .keys()
        .filter(|k| !order.contains(&k.as_str()) && k.as_str() != "bee")
        .collect();
    unknown.sort(); // JS default sort — keys are KEY_RE ASCII, byte order matches
    for key in known.into_iter().chain(unknown) {
        if !key_re_ok(key) {
            return Err(());
        }
        let value = &map[key.as_str()];
        if matches!(value, Value::Object(_)) {
            return Err(()); // nested map — only root-level "bee:" is legal
        }
        lines.push(format!("{indent}{key}: {}", emit_value(value)?));
    }
    Ok(())
}

/// emitFrontmatter(data) — canonical block incl. both --- lines, LF, trailing \n.
pub(crate) fn emit_frontmatter(data: &Map<String, Value>) -> Result<String, ()> {
    let mut lines = vec!["---".to_string()];
    emit_entries(&mut lines, data, &ROOT_KEY_ORDER, "")?;
    if let Some(bee) = data.get("bee") {
        let Value::Object(bee) = bee else { return Err(()) };
        lines.push("bee:".to_string());
        emit_entries(&mut lines, bee, &BEE_KEY_ORDER, "  ")?;
    }
    lines.push("---".to_string());
    Ok(format!("{}\n", lines.join("\n")))
}

// ─── parser (accepts exactly the emitted subset; loud typed failure) ───────

pub(crate) enum Fm {
    Absent,
    Parsed {
        data: Map<String, Value>,
        block: String,
        body: String,
    },
    Failed {
        code: &'static str,
        message: String,
        line: usize,
    },
}

pub(crate) fn fm_fail(code: &'static str, message: String, line: usize) -> Result<Value, Fm> {
    Err(Fm::Failed { code, message, line })
}

pub(crate) fn parse_scalar_token(raw: &str, line_no: usize) -> Result<Value, Fm> {
    if raw == "true" {
        return Ok(Value::Bool(true));
    }
    if raw == "false" {
        return Ok(Value::Bool(false));
    }
    if raw.starts_with('"') {
        match serde_json::from_str::<Value>(raw) {
            Ok(Value::String(s)) => return Ok(Value::String(s)),
            Ok(_) => {
                return fm_fail("bad_quoted_string", "quoted value did not decode to a string".to_string(), line_no)
            }
            // CUTOVER: a lone-surrogate escape (U+D800..U+DFFF) used to
            // return Fm::NeedsNode here — V8's JSON.parse accepted it where
            // serde never can, so the whole command delegated. There is no
            // second parser left, so it takes the SAME bad_quoted_string
            // finding every other undecodable quoted scalar takes.
            Err(_) => {
                return fm_fail(
                    "bad_quoted_string",
                    format!("quoted value {} is not one complete JSON string", js_quote_str(raw)),
                    line_no,
                );
            }
        }
    }
    if raw.starts_with('\'') {
        return fm_fail(
            "single_quoted_string",
            "single-quoted scalars are outside the emitted subset — use double quotes".to_string(),
            line_no,
        );
    }
    // /^[&*!|>%@`{}]/
    if matches!(raw.chars().next(), Some('&' | '*' | '!' | '|' | '>' | '%' | '@' | '`' | '{' | '}')) {
        return fm_fail(
            "unsupported_scalar",
            format!(
                "value starting with \"{}\" (anchor/alias/block/flow-map indicator) is outside the emitted subset",
                raw.chars().next().unwrap()
            ),
            line_no,
        );
    }
    Ok(Value::String(raw.to_string()))
}

pub(crate) fn parse_flow_list(raw: &str, line_no: usize) -> Result<Value, Fm> {
    if !raw.ends_with(']') {
        return fm_fail(
            "bad_flow_list",
            format!("flow list {} does not close with \"]\"", js_quote_str(raw)),
            line_no,
        );
    }
    let inner = js_trim(&raw[1..raw.len() - 1]);
    if inner.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escaped = false;
    for ch in inner.chars() {
        if in_quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quote = false;
            }
        } else if ch == '"' {
            current.push(ch);
            in_quote = true;
        } else if ch == ',' {
            segments.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    if in_quote {
        return fm_fail("bad_flow_list", "unterminated quoted item inside flow list".to_string(), line_no);
    }
    segments.push(current);
    let mut value = Vec::new();
    for segment in &segments {
        let token = js_trim(segment);
        if token.is_empty() {
            return fm_fail("bad_flow_list", "empty item inside flow list".to_string(), line_no);
        }
        value.push(parse_scalar_token(token, line_no)?);
    }
    Ok(Value::Array(value))
}

pub(crate) fn parse_key_value_line(line: &str, target: &mut Map<String, Value>, line_no: usize, prefix: &str) -> Result<(), Fm> {
    let Some(sep) = line.find(": ") else {
        return fm_fail(
            "unrecognized_line",
            format!(
                "line {} is not \"key: value\", a \"bee:\" map header, or a closing \"---\"",
                js_quote_str(line)
            ),
            line_no,
        )
        .map(|_| ());
    };
    let key = &line[..sep];
    if !key_re_ok(key) {
        return fm_fail(
            "bad_key",
            format!("{} is not a legal frontmatter key", js_quote_str(key)),
            line_no,
        )
        .map(|_| ());
    }
    if target.contains_key(key) {
        return fm_fail("duplicate_key", format!("duplicate key \"{prefix}{key}\""), line_no).map(|_| ());
    }
    let raw = &line[sep + 2..];
    if raw.is_empty() {
        return fm_fail("empty_value", format!("key \"{prefix}{key}\" has no value after \": \""), line_no)
            .map(|_| ());
    }
    let parsed = if raw.starts_with('[') {
        parse_flow_list(raw, line_no)?
    } else {
        parse_scalar_token(raw, line_no)?
    };
    target.insert(key.to_string(), parsed);
    Ok(())
}

/// parseFrontmatter(text) — see lib/knowledge.mjs for the full contract.
pub(crate) fn parse_frontmatter(text: &str) -> Fm {
    let open_len = if text.starts_with("---\r\n") {
        5
    } else if text.starts_with("---\n") {
        4
    } else {
        return Fm::Absent;
    };

    let mut cursor = open_len;
    let mut block_end: Option<usize> = None;
    let mut inner_end = 0usize;
    while cursor <= text.len() {
        let nl = text[cursor..].find('\n').map(|p| p + cursor);
        let line_end = nl.unwrap_or(text.len());
        let mut line = &text[cursor..line_end];
        if let Some(stripped) = line.strip_suffix('\r') {
            line = stripped;
        }
        if line == "---" {
            inner_end = cursor;
            block_end = Some(nl.map(|p| p + 1).unwrap_or(text.len()));
            break;
        }
        let Some(nl) = nl else { break };
        cursor = nl + 1;
    }
    let Some(block_end) = block_end else {
        return Fm::Failed {
            code: "unclosed_frontmatter",
            message: "frontmatter opened with \"---\" but never closed".to_string(),
            line: 1,
        };
    };

    let block = text[..block_end].to_string();
    let body = text[block_end..].to_string();
    let inner_raw = &text[open_len..inner_end];
    let inner_lines: Vec<&str> = if inner_raw.is_empty() {
        Vec::new()
    } else {
        let mut v: Vec<&str> = inner_raw.split('\n').collect();
        v.pop();
        v
    };

    let mut data: Map<String, Value> = Map::new();
    let mut in_bee_map = false;
    let mut line_no = 1usize;
    for raw_line in inner_lines {
        line_no += 1;
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.is_empty() {
            return Fm::Failed {
                code: "blank_line",
                message: "blank line inside frontmatter is outside the emitted subset".to_string(),
                line: line_no,
            };
        }
        if line.contains('\t') {
            return Fm::Failed {
                code: "tab_in_frontmatter",
                message: "tab character inside frontmatter is outside the emitted subset".to_string(),
                line: line_no,
            };
        }
        if let Some(inner) = line.strip_prefix("  ") {
            if !in_bee_map {
                return Fm::Failed {
                    code: "unexpected_indent",
                    message: "indented line outside the \"bee:\" map".to_string(),
                    line: line_no,
                };
            }
            if inner.starts_with(' ') {
                return Fm::Failed {
                    code: "bad_indent",
                    message: "bee: map entries are indented exactly two spaces".to_string(),
                    line: line_no,
                };
            }
            let bee = data
                .get_mut("bee")
                .and_then(Value::as_object_mut)
                .expect("bee map exists while in_bee_map");
            match parse_key_value_line(inner, bee, line_no, "bee.") {
                Ok(()) => continue,
                Err(f) => return f,
            }
        }
        if line.starts_with(' ') {
            return Fm::Failed {
                code: "bad_indent",
                message: "root-level lines must not be indented".to_string(),
                line: line_no,
            };
        }
        in_bee_map = false;
        // /^([^:\s]+):$/ — a map header line.
        let header_key = line.strip_suffix(':').filter(|key| {
            !key.is_empty() && key.chars().all(|c| c != ':' && !js_is_space(c))
        });
        if let Some(key) = header_key {
            if !key_re_ok(key) {
                return Fm::Failed {
                    code: "bad_key",
                    message: format!("{} is not a legal frontmatter key", js_quote_str(key)),
                    line: line_no,
                };
            }
            if key != "bee" {
                return Fm::Failed {
                    code: "unsupported_map",
                    message: format!(
                        "nested map \"{key}:\" is outside the emitted subset (the only nested map is \"bee:\")"
                    ),
                    line: line_no,
                };
            }
            if data.contains_key("bee") {
                return Fm::Failed {
                    code: "duplicate_key",
                    message: "duplicate key \"bee\"".to_string(),
                    line: line_no,
                };
            }
            data.insert("bee".to_string(), Value::Object(Map::new()));
            in_bee_map = true;
            continue;
        }
        if let Err(f) = parse_key_value_line(line, &mut data, line_no, "") {
            return f;
        }
    }

    Fm::Parsed { data, block, body }
}
