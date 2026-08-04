// JS value helpers, the Date.parse subset, and localeCompare
//
// Split out of the single 7k-line verbs/status_full.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, LinkedRoots, RootsWt};
use crate::state::{bypass_level, read_config_raw};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Value};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use crate::version::BEE_VERSION;

// ─── JS value helpers ──────────────────────────────────────────────────────

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

/// JS strict equality (===) over JSON-representable primitives; None models
/// `undefined`. Objects/arrays compare by reference in JS — two separately
/// parsed values are never reference-equal, so they compare false here.
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
pub(crate) fn nullish(o: Option<&Value>) -> bool {
    matches!(o, None | Some(Value::Null))
}

/// JS Array.prototype.join over Values (null/undefined -> empty string).
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

/// JS Math.round: floor(x + 0.5).
pub(crate) fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

// ─── dates (V8 Date.parse for the ISO shapes bee writes, toISOString) ──────

pub(crate) fn now_ms() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(f64::NAN)
}

/// new Date(ms).toISOString() — millisecond UTC ISO.
pub(crate) fn to_iso(ms: f64) -> String {
    use chrono::TimeZone;
    match chrono::Utc.timestamp_millis_opt(ms as i64) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        _ => "Invalid Date".to_string(),
    }
}

/// Date.parse for the ECMA-262 Date Time String Format (the only shapes bee
/// writes: toISOString output, plus date-only "YYYY-MM-DD" legacy stamps).
/// Date-only forms are UTC; date-time forms without an offset are LOCAL time
/// (ES spec); anything else parses NaN.
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
        let v = match digits(b, 1, 6) {
            Some(v) => v,
            None => return f64::NAN,
        };
        year = sign * v;
        i += 7;
    } else {
        year = match digits(b, 0, 4) {
            Some(v) => v,
            None => return f64::NAN,
        };
        i += 4;
    }
    let mut month: i64 = 1;
    let mut day: i64 = 1;
    if i < b.len() && b[i] == b'-' {
        month = match digits(b, i + 1, 2) {
            Some(v) => v,
            None => return f64::NAN,
        };
        i += 3;
        if i < b.len() && b[i] == b'-' {
            day = match digits(b, i + 1, 2) {
                Some(v) => v,
                None => return f64::NAN,
            };
            i += 3;
        }
    }
    let (mut hour, mut minute, mut second, mut millis) = (0i64, 0i64, 0i64, 0i64);
    let mut has_time = false;
    let mut offset_minutes: Option<i64> = None;
    if i < b.len() && b[i] == b'T' {
        has_time = true;
        hour = match digits(b, i + 1, 2) {
            Some(v) => v,
            None => return f64::NAN,
        };
        i += 3;
        if i >= b.len() || b[i] != b':' {
            return f64::NAN;
        }
        minute = match digits(b, i + 1, 2) {
            Some(v) => v,
            None => return f64::NAN,
        };
        i += 3;
        if i < b.len() && b[i] == b':' {
            second = match digits(b, i + 1, 2) {
                Some(v) => v,
                None => return f64::NAN,
            };
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
                    let oh = match digits(b, i + 1, 2) {
                        Some(v) => v,
                        None => return f64::NAN,
                    };
                    i += 3;
                    if i >= b.len() || b[i] != b':' {
                        return f64::NAN;
                    }
                    let om = match digits(b, i + 1, 2) {
                        Some(v) => v,
                        None => return f64::NAN,
                    };
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
    if hour > 24 || minute > 59 || second > 59 || (hour == 24 && (minute != 0 || second != 0 || millis != 0)) {
        return f64::NAN;
    }
    let date = match chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32) {
        Some(d) => d,
        None => return f64::NAN,
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

/// Date.parse over a possibly-absent/non-string Value (non-strings coerce to
/// unparseable strings in every bee case -> NaN).
pub(crate) fn date_parse_val(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::String(s)) => js_date_parse(s),
        _ => f64::NAN,
    }
}

// ─── localeCompare('en'[, {numeric:true}]) ─────────────────────────────────
// Two-pass ICU-ish comparator matching measured Node behavior on the id/
// feature alphabet ([A-Za-z0-9._-] plus ISO timestamps):
//   primary:  class order _ < - < . < (other punct) < digits < letters
//             (letters case-folded; numeric mode compares digit runs by
//              value, "01" == "1" — no length tiebreak, matching ICU)
//   tertiary: first case difference, lowercase before uppercase.

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
    // Tertiary (case) pass — only when primary-equal.
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
