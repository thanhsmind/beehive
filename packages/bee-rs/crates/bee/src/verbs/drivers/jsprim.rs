// JS string primitives
//
// Split out of the single 4.9k-line verbs/drivers.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::roots::{resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::textutil::{char_len, truncate_chars_head};
use crate::verbs::reservations::{
    finish, js_is_ws, parse_flags, prelude, pseudo_uuid_v4, truthy, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{
    release_reservations_for_agent, reserve_path_atomic, Err2, ReserveOutcome,
};
use serde_json::{Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

// ═══ JS string primitives ══════════════════════════════════════════════════

/// provenance: JS String.prototype.trim — same set verbs/test_runner.rs's
/// js_trim uses (ECMA WhiteSpace ∪ LineTerminator, incl. U+FEFF/NBSP).
pub(crate) fn js_trim(s: &str) -> &str {
    s.trim_matches(js_is_ws)
}

/// JS template-literal coercion for a possibly-absent field.
pub(crate) fn tpl(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(v) => jsjson::js_to_string(v),
    }
}

/// JS `typeof v === 'string' && v` — the truthy-string idiom.
pub(crate) fn truthy_str(v: Option<&Value>) -> Option<&str> {
    match v {
        Some(Value::String(s)) if !s.is_empty() => Some(s),
        _ => None,
    }
}

/// JS `a === b` over JSON primitives (`None` models `undefined`).
pub(crate) fn strict_eq(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (None, None) => true,
        (None, _) | (_, None) => false,
        (Some(x), Some(y)) => match (x, y) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(p), Value::Bool(q)) => p == q,
            (Value::Number(p), Value::Number(q)) => p.as_f64() == q.as_f64(),
            (Value::String(p), Value::String(q)) => p == q,
            _ => false,
        },
    }
}

pub(crate) fn vget<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.as_object().and_then(|m| m.get(key))
}

/// provenance: dispatch-prepare.mjs oneLine(text, max = 140) —
/// `String(text ?? '').replace(/\s+/g, ' ').trim()`, ellipsised at `max`
/// CHARS with a 3-char "..." tail (decision D3: char-based, not the
/// historical UTF-16-unit count).
pub(crate) fn one_line(text: Option<&Value>, max: usize) -> String {
    let raw = match text {
        None | Some(Value::Null) => String::new(),
        Some(v) => jsjson::js_to_string(v),
    };
    // /\s+/g -> ' '
    let mut collapsed = String::with_capacity(raw.len());
    let mut in_ws = false;
    for c in raw.chars() {
        if js_is_ws(c) {
            in_ws = true;
        } else {
            if in_ws {
                collapsed.push(' ');
                in_ws = false;
            }
            collapsed.push(c);
        }
    }
    if in_ws {
        collapsed.push(' ');
    }
    let flat = js_trim(&collapsed).to_string();
    if char_len(&flat) > max {
        format!("{}...", truncate_chars_head(&flat, max - 3))
    } else {
        flat
    }
}
