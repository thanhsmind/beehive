// the shared JS-semantics helpers other verb modules consume
//
// Split out of the single 3k-line verbs/reservations.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, resolve_store_root_worktree, Roots, RootsWt, StoreRoots};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ─── shared JS-semantics helpers (also consumed by verbs/decisions.rs) ─────

/// JS \s (and String.prototype.trim) whitespace class.
pub(crate) fn js_is_ws(c: char) -> bool {
    matches!(c,
        ' ' | '\t' | '\n' | '\r' | '\u{b}' | '\u{c}' | '\u{a0}' | '\u{1680}'
        | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}'
        | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

pub(crate) fn js_trim(s: &str) -> &str {
    s.trim_matches(js_is_ws)
}

/// JS truthiness of a JSON value (undefined handled by callers as None).
pub(crate) fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// JS property access: objects yield the property, everything else reads as
/// undefined (None). Arrays carry no named data properties in the shapes bee
/// stores, so None matches JS's undefined there too.
pub(crate) fn jget<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    match v {
        Value::Object(m) => m.get(key),
        _ => None,
    }
}

/// `${value}` template coercion.
pub(crate) fn js_disp(v: &Value) -> String {
    jsjson::js_to_string(v)
}

/// `${maybe}` where the key may be absent (undefined).
pub(crate) fn js_disp_opt(v: Option<&Value>) -> String {
    match v {
        Some(v) => js_disp(v),
        None => "undefined".to_string(),
    }
}

/// JSON.stringify(string) — used inside error-message interpolations.
pub(crate) fn js_quote(s: &str) -> String {
    jsjson::stringify(&Value::String(s.to_string()))
}

pub(crate) fn v_is_str(v: &Value, s: &str) -> bool {
    matches!(v, Value::String(x) if x == s)
}

/// Marker for JS-exotic input this port does not model — the probe delegates.
pub(crate) struct Exotic;

pub(crate) type Ex<T> = Result<T, Exotic>;

/// Date.parse subset: full ISO with offset (what toISOString writes) plus the
/// date-only "YYYY-MM-DD" form (UTC midnight in JS). Ok(None) = NaN. A
/// non-empty string outside this grammar might still parse under V8's legacy
/// fallback — Err(Exotic), delegate.
pub(crate) fn js_date_parse(s: &str) -> Ex<Option<f64>> {
    if js_trim(s).is_empty() {
        return Ok(None); // Date.parse('') / whitespace-only → NaN
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(Some(dt.timestamp_millis() as f64));
    }
    let b = s.as_bytes();
    let strict_date_only = b.len() == 10
        && b[0..4].iter().all(|c| c.is_ascii_digit())
        && b[4] == b'-'
        && b[5..7].iter().all(|c| c.is_ascii_digit())
        && b[7] == b'-'
        && b[8..10].iter().all(|c| c.is_ascii_digit());
    if strict_date_only {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            if let Some(n) = d.and_hms_opt(0, 0, 0) {
                return Ok(Some(n.and_utc().timestamp_millis() as f64));
            }
        }
        return Ok(None); // shape-valid but impossible date (2026-13-40) → NaN in JS too
    }
    Err(Exotic)
}

/// Date.parse of a JSON value. Absent/null coerce to NaN in JS
/// ("undefined"/"null" strings); other non-strings coerce via ToString,
/// which this port does not model → Exotic.
pub(crate) fn date_parse_val(v: Option<&Value>) -> Ex<Option<f64>> {
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => js_date_parse(s),
        Some(_) => Err(Exotic),
    }
}

pub(crate) fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

/// new Date(ms).toISOString() — beyond the JS Date range Node throws; Exotic.
pub(crate) fn iso_from_ms(ms: f64) -> Ex<String> {
    if !ms.is_finite() || ms.abs() > 8.64e15 {
        return Err(Exotic);
    }
    let dt = chrono::DateTime::from_timestamp_millis(ms as i64).ok_or(Exotic)?;
    Ok(dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

/// new Date().toISOString() (utcNow in the .mjs files).
pub(crate) fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// JS Math.round (half toward +infinity).
pub(crate) fn js_round(v: f64) -> f64 {
    (v + 0.5).floor()
}

/// Re-shape parsed JSON the way JSON.parse does in JS: every number becomes
/// an f64 (big u64/i64 round exactly like V8 does). EVERY finite number is
/// accepted — the `|n| >= 1e21` exponent-notation class is retired (see the
/// CUTOVER note inside).
pub(crate) fn js_numberify(v: &Value) -> Ex<Value> {
    match v {
        Value::Number(n) => {
            let f = n.as_f64().ok_or(Exotic)?;
            // CUTOVER: |n| >= 1e21 used to return Exotic here (jsjson printed
            // such a number differently than V8, so the verb delegated rather
            // than break C2). jsjson::js_f64_to_string now implements the
            // spec's exponential form, so there is nothing left to dodge and
            // no runtime left to dodge to. The remaining Err arms are
            // unreachable from JSON, which carries no NaN/Infinity.
            Ok(Value::Number(Number::from_f64(f).ok_or(Exotic)?))
        }
        Value::Array(items) => items
            .iter()
            .map(js_numberify)
            .collect::<Ex<Vec<_>>>()
            .map(Value::Array),
        Value::Object(m) => {
            let mut out = Map::new();
            for (k, x) in m {
                out.insert(k.clone(), js_numberify(x)?);
            }
            Ok(Value::Object(out))
        }
        other => Ok(other.clone()),
    }
}

/// crypto.randomUUID-shaped v4 id. Uniqueness (pid + counter + clock nanos
/// through sha256) is what the store needs; no RNG dependency, same
/// derivation style as lock.rs's fresh_token.
pub(crate) fn pseudo_uuid_v4() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut hasher = Sha256::new();
    hasher.update(b"bee-uuid");
    hasher.update(std::process::id().to_le_bytes());
    hasher.update(COUNTER.fetch_add(1, Ordering::Relaxed).to_le_bytes());
    hasher.update(nanos.to_le_bytes());
    let d = hasher.finalize();
    let mut b: [u8; 16] = d[..16].try_into().unwrap();
    b[6] = (b[6] & 0x0f) | 0x40; // version 4
    b[8] = (b[8] & 0x3f) | 0x80; // variant 10xx
    let hex: Vec<String> = b.iter().map(|x| format!("{x:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        hex[0..4].join(""),
        hex[4..6].join(""),
        hex[6..8].join(""),
        hex[8..10].join(""),
        hex[10..16].join("")
    )
}
