// the lifted JS value helpers, fs primitives, Date.parse and localeCompare
//
// Split out of the single 3.1k-line hooks/session_preamble.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, warn_corrupt_json, ReadJson};
use crate::jsjson;
use crate::state::{bypass_level, ship_visibility};
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::version::BEE_VERSION;

// ─── JS value helpers (lift: verbs/status_full.rs:183-255) ─────────────────

pub(crate) fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

pub(crate) fn opt_truthy(o: Option<&Value>) -> bool {
    o.map(truthy).unwrap_or(false)
}

/// Property access: `undefined` is `None` (distinct from JSON null).
pub(crate) fn vget<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.as_object().and_then(|m| m.get(key))
}

/// JS strict equality (===) over JSON-representable primitives; `None` models
/// `undefined`. Objects/arrays compare by reference in JS, so two separately
/// parsed values never compare equal here either.
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

pub(crate) fn str_eq(v: Option<&Value>, s: &str) -> bool {
    matches!(v, Some(Value::String(x)) if x == s)
}

/// Template-literal coercion; `undefined` renders "undefined".
pub(crate) fn tpl(o: Option<&Value>) -> String {
    match o {
        None => "undefined".to_string(),
        Some(v) => jsjson::js_to_string(v),
    }
}

/// `value ?? fallback` — nullish only.
pub(crate) fn tpl_or(o: Option<&Value>, fallback: &str) -> String {
    match o {
        None | Some(Value::Null) => fallback.to_string(),
        Some(v) => jsjson::js_to_string(v),
    }
}

/// JS Array.prototype.join over Values (null/undefined render empty).
pub(crate) fn js_join(items: &[Value], sep: &str) -> String {
    items
        .iter()
        .map(|v| match v {
            Value::Null => String::new(),
            other => jsjson::js_to_string(other),
        })
        .collect::<Vec<_>>()
        .join(sep)
}

/// JS String.prototype.trim (Unicode whitespace + BOM).
pub(crate) fn js_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

/// JS `\s` (the set String.prototype.trim strips).
pub(crate) fn js_is_space(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

/// JS Math.round: floor(x + 0.5).
pub(crate) fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// Number -> template-literal text.
pub(crate) fn num_str(n: f64) -> String {
    jsjson::js_f64_to_string(n)
}

// ─── fs primitives (lift: verbs/status_full.rs:531-560) ────────────────────

/// `readJson(file, null)`, fail-open. A present-but-unparseable file warns
/// through the shared native helper and yields the fallback — never a bail:
/// inject.mjs's whole posture is that orientation does not fail a session.
pub(crate) fn read_json_open(file: &Path) -> Option<Value> {
    match read_json(file) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => {
            warn_corrupt_json(file);
            None
        }
        ReadJson::Parsed(v) => Some(v),
    }
}

/// `readJson(file, null)` narrowed to a plain object (the shape every caller
/// below actually wants); anything else reads as absent.
pub(crate) fn read_json_object(file: &Path) -> Option<JMap> {
    match read_json_open(file) {
        Some(Value::Object(m)) => Some(m),
        _ => None,
    }
}

pub(crate) fn read_text_opt(file: &Path) -> Option<String> {
    std::fs::read(file).ok().map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// fsutil.mjs readJsonl: split /\r?\n/, trim, JSON.parse per line, silent skip.
pub(crate) fn read_jsonl(file: &Path) -> Vec<Value> {
    let Some(text) = read_text_opt(file) else { return Vec::new() };
    let mut events = Vec::new();
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = js_trim(line);
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            events.push(v);
        }
    }
    events
}

// ─── Date.parse (lift: verbs/status_full.rs:278-440) ───────────────────────

/// Date.parse for the ECMA-262 Date Time String Format (the only shapes bee
/// writes). Date-only forms are UTC; date-time forms without an offset are
/// LOCAL time (ES spec); anything else is NaN.
pub(crate) fn js_date_parse(s: &str) -> f64 {
    fn digits(b: &[u8], i: usize, n: usize) -> Option<i64> {
        if i + n > b.len() {
            return None;
        }
        let mut v: i64 = 0;
        for k in 0..n {
            let c = b[i + k];
            if !c.is_ascii_digit() {
                return None;
            }
            v = v * 10 + (c - b'0') as i64;
        }
        Some(v)
    }
    let b = s.as_bytes();
    let mut i = 0;
    let year: i64;
    if !b.is_empty() && (b[0] == b'+' || b[0] == b'-') {
        let sign = if b[0] == b'-' { -1 } else { 1 };
        let Some(v) = digits(b, 1, 6) else { return f64::NAN };
        year = sign * v;
        i += 7;
    } else {
        let Some(v) = digits(b, 0, 4) else { return f64::NAN };
        year = v;
        i += 4;
    }
    let mut month: i64 = 1;
    let mut day: i64 = 1;
    if i < b.len() && b[i] == b'-' {
        let Some(v) = digits(b, i + 1, 2) else { return f64::NAN };
        month = v;
        i += 3;
        if i < b.len() && b[i] == b'-' {
            let Some(v) = digits(b, i + 1, 2) else { return f64::NAN };
            day = v;
            i += 3;
        }
    }
    let (mut hour, mut minute, mut second, mut millis) = (0i64, 0i64, 0i64, 0i64);
    let mut has_time = false;
    let mut offset_minutes: Option<i64> = None;
    if i < b.len() && b[i] == b'T' {
        has_time = true;
        let Some(v) = digits(b, i + 1, 2) else { return f64::NAN };
        hour = v;
        i += 3;
        if i >= b.len() || b[i] != b':' {
            return f64::NAN;
        }
        let Some(v) = digits(b, i + 1, 2) else { return f64::NAN };
        minute = v;
        i += 3;
        if i < b.len() && b[i] == b':' {
            let Some(v) = digits(b, i + 1, 2) else { return f64::NAN };
            second = v;
            i += 3;
            if i < b.len() && b[i] == b'.' {
                let start = i + 1;
                let mut end = start;
                while end < b.len() && b[end].is_ascii_digit() {
                    end += 1;
                }
                if end == start {
                    return f64::NAN;
                }
                let mut ms = 0i64;
                for k in 0..3 {
                    ms = ms * 10 + if start + k < end { (b[start + k] - b'0') as i64 } else { 0 };
                }
                millis = ms;
                i = end;
            }
        }
        if i < b.len() {
            match b[i] {
                b'Z' => {
                    offset_minutes = Some(0);
                    i += 1;
                }
                b'+' | b'-' => {
                    let sign = if b[i] == b'-' { -1 } else { 1 };
                    let Some(oh) = digits(b, i + 1, 2) else { return f64::NAN };
                    i += 3;
                    if i >= b.len() || b[i] != b':' {
                        return f64::NAN;
                    }
                    let Some(om) = digits(b, i + 1, 2) else { return f64::NAN };
                    i += 3;
                    if oh > 23 || om > 59 {
                        return f64::NAN;
                    }
                    offset_minutes = Some(sign * (oh * 60 + om));
                }
                _ => {}
            }
        }
    }
    if i != b.len() {
        return f64::NAN;
    }
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return f64::NAN;
    }
    if hour > 24
        || minute > 59
        || second > 59
        || (hour == 24 && (minute != 0 || second != 0 || millis != 0))
    {
        return f64::NAN;
    }
    let Some(date) = chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32) else {
        return f64::NAN;
    };
    let naive = date.and_hms_opt(0, 0, 0).unwrap()
        + chrono::Duration::hours(hour)
        + chrono::Duration::minutes(minute)
        + chrono::Duration::seconds(second)
        + chrono::Duration::milliseconds(millis);
    if has_time && offset_minutes.is_none() {
        use chrono::TimeZone;
        match chrono::Local.from_local_datetime(&naive).earliest() {
            Some(dt) => dt.timestamp_millis() as f64,
            None => f64::NAN,
        }
    } else {
        (naive.and_utc().timestamp_millis() - offset_minutes.unwrap_or(0) * 60_000) as f64
    }
}

/// Date.parse over a possibly-absent/non-string Value.
pub(crate) fn date_parse_val(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::String(s)) => js_date_parse(s),
        _ => f64::NAN,
    }
}

// ─── localeCompare('en'[, {numeric:true}]) (lift: status_full.rs:449-528) ──

pub(crate) fn char_class_key(c: char) -> (u8, u32) {
    if c.is_whitespace() {
        return (0, c as u32);
    }
    match c {
        '_' => (1, 0),
        '-' => (1, 1),
        ',' => (1, 2),
        ';' => (1, 3),
        ':' => (1, 4),
        '!' => (1, 5),
        '?' => (1, 6),
        '.' => (1, 7),
        _ if c.is_ascii_digit() => (2, c as u32 - '0' as u32),
        _ if c.is_alphabetic() => (3, c.to_lowercase().next().unwrap_or(c) as u32),
        _ => (1, 100 + c as u32),
    }
}

pub(crate) fn locale_cmp(a: &str, b: &str, numeric: bool) -> Ordering {
    let av: Vec<char> = a.chars().collect();
    let bv: Vec<char> = b.chars().collect();
    let (mut i, mut j) = (0usize, 0usize);
    while i < av.len() && j < bv.len() {
        let (ca, cb) = (av[i], bv[j]);
        if numeric && ca.is_ascii_digit() && cb.is_ascii_digit() {
            let si = i;
            while i < av.len() && av[i].is_ascii_digit() {
                i += 1;
            }
            let sj = j;
            while j < bv.len() && bv[j].is_ascii_digit() {
                j += 1;
            }
            let ra: String = av[si..i].iter().collect();
            let rb: String = bv[sj..j].iter().collect();
            let ta = ra.trim_start_matches('0');
            let tb = rb.trim_start_matches('0');
            let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
            if ord != Ordering::Equal {
                return ord;
            }
            continue;
        }
        let ord = char_class_key(ca).cmp(&char_class_key(cb));
        if ord != Ordering::Equal {
            return ord;
        }
        i += 1;
        j += 1;
    }
    let ord = (av.len() - i).cmp(&(bv.len() - j));
    if ord != Ordering::Equal {
        return ord;
    }
    let (mut i, mut j) = (0usize, 0usize);
    while i < av.len() && j < bv.len() {
        let (ca, cb) = (av[i], bv[j]);
        if numeric && ca.is_ascii_digit() && cb.is_ascii_digit() {
            while i < av.len() && av[i].is_ascii_digit() {
                i += 1;
            }
            while j < bv.len() && bv[j].is_ascii_digit() {
                j += 1;
            }
            continue;
        }
        if ca != cb && ca.is_alphabetic() && cb.is_alphabetic() {
            let (la, lb) = (ca.is_lowercase(), cb.is_lowercase());
            if la != lb {
                return if la { Ordering::Less } else { Ordering::Greater };
            }
        }
        i += 1;
        j += 1;
    }
    Ordering::Equal
}
