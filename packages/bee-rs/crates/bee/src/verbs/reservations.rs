// bee reservations — native port of the `reservations` verb group.
//
// Ported argv shapes (everything else returns None BEFORE any output and the
// whole command re-runs under Node):
//   reservations list  [--active-only] [--json]
//   reservations reserve --agent A --cell C --path P [--ttl N] [--session S]
//                        [--kind intent|lease] [--json]
//   reservations release --agent A [--cell C] [--json]
//   reservations sweep [--json]
//
// Provenance: bee.mjs handleReservationsReserve/Release/List/Sweep +
// reservePathAtomic/releaseReservationsForAgent/resolveMainRoot/
// resolveHoldTopology/holdForeignExpiry, lib/reservations.mjs (the msn-16
// lease-store shim: listReservations/findConflicts/reserve/release/
// sweepExpired/pathsOverlap/isHardConflict/leaseToReservation/findMainRoot/
// controlRootFor), lib/lease-store.mjs (acquireLeases' O_EXCL create /
// releaseLease's plain rm / resolveResourceFile), lib/worktree-holds.mjs
// (readStore/insertHold/findForeignHolds/releaseHolds/sweepExpiredHolds/
// withHoldsLock) and lib/claims.mjs (resolveSessionId/listSessionRecords/
// heartbeatStale/isConcurrentMode).
//
// Locking: the mutating verbs contend on the SAME cross-process lock files
// Node uses — crate::lock::acquire_store_lock with the exact Node lock name
// "cross-worktree-holds" (worktree-holds.mjs CROSS_WORKTREE_HOLDS_LOCK), so
// the two runtimes serialize against each other mid-campaign. The Node
// verbs are async solely because withStoreLock is async — the sync Rust
// lock covers the same semantics (contract C1).
//
// Root topology: WORKTREE-NATIVE (see roots.rs's header for the per-verb
// flip list). The four verbs here resolve through
// crate::roots::resolve_store_root_worktree and carry both of bee.mjs's
// topology helpers for real:
//
//   * resolveMainRoot(root) — where the shared cross-worktree holds LEDGER
//     lives. `<mainRoot>/.bee/runtime/cross-worktree-holds.json`, always the
//     MAIN checkout's store, never a worktree's own (the same asymmetry the
//     grant registry relies on). `list` and `sweep` use it unconditionally.
//   * resolveHoldTopology(root) — `{mainRoot, holder}` for the two
//     hold-worthy topologies and `null` for everything else:
//       ordinary checkout        -> {workRoot, "main"}
//       GRANTED linked worktree  -> {mainRoot, its git-verified id}
//       UNGRANTED linked worktree-> null: `root` already IS the shared main
//                                  store, so mirroring under a synthetic
//                                  identity would just duplicate a
//                                  reservation the store already carries.
//                                  reserve/release skip the ENTIRE
//                                  cross-worktree section — no lock taken,
//                                  no foreign-hold check, no mirror row.
//
// A BROKEN link still delegates (resolveRoots throws in Node before dispatch).
// controlRootFor (reservations.mjs's own cycle-free findMainRoot walk, which
// cannot import state.mjs) is a THIRD, separate resolver, ported in full
// including its linked branch: it is where the LEASE files live, and it
// answers mainRoot for a granted worktree from the git link alone.
//
// CUTOVER (2026-08-01): corrupt JSON no longer delegates. The cross-worktree
// holds ledger and the session records both FAIL OPEN exactly as
// `readJson(file, fallback)` did — one `bee: could not parse JSON at …` line
// (crate::fsutil::warn_corrupt_json) and then the same fallback: an empty
// `{ holds: [] }` ledger, or that one session record skipped. The
// `|n| >= 1e21` class is retired too: jsjson::js_f64_to_string implements the
// full ECMA Number::toString, so js_numberify accepts every finite number.
//
// Delegation rules beyond the argv shape (all pre-checked BEFORE any store
// write): `null` ledger entries (a JS
// property access crash), date strings outside the ISO subset this port
// models (Date.parse's V8 fallback grammar), and unmodelable Windows path
// spellings. One accepted
// residual: a prechecked file turning corrupt DURING the run (external,
// non-locking writer) can still surface as a late None — the Node re-run
// then operates on the partially-mutated store, the same outcome as two
// sequential CLI invocations. A second documented deviation: Node's
// insertHold re-reads the holds ledger inside the atomic reserve section;
// every production ledger writer serializes on this same lock, so the store
// value read at the section's own findForeignHolds step is reused instead.

use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, resolve_store_root_worktree, Roots, RootsWt, StoreRoots};
use crate::verbs::{emit_no_root_error, record_timing};
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

/// JS strict equality (===) over parsed JSON primitives. Objects/arrays are
/// reference-compared in JS; two independently parsed values are never the
/// same reference, so `false` is faithful for them here.
pub(crate) fn js_strict_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::String(x), Value::String(y)) => x == y,
        _ => false,
    }
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

// ─── argv flag parsing (bee.mjs parseFlags, mirrored exactly) ──────────────

/// bee.mjs FLAG_ALONE_BOOLEANS — the closed set of flags that are boolean
/// when they appear alone; every other flag consumes the next token.
pub(crate) const FLAG_ALONE_BOOLEANS: &[&str] = &[
    "json", "stdin", "active-only", "dry-run", "write", "as-lane", "no-lane",
    "waive-scribing-debt", "waive-compounding", "html", "string", "cleanup",
    "force-ownership", "local", "all", "untagged", "check", "with-companion",
    "lanes-full", "strict", "queue-submit", "show", "isolate", "set", "brief",
    "all-but-active", "merge", "claim",
];

#[derive(Clone, PartialEq)]
pub(crate) enum FlagV {
    /// Flag-alone boolean — JS `true`.
    Present,
    /// String value (from `--f v` or `--f=v`).
    S(String),
}

pub(crate) struct Flags(pub Vec<(String, FlagV)>);

impl Flags {
    pub(crate) fn get(&self, name: &str) -> Option<&FlagV> {
        self.0.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }
    fn insert(&mut self, name: &str, value: FlagV) {
        // JS `flags[name] = value`: first-occurrence position, last value.
        if let Some(slot) = self.0.iter_mut().find(|(k, _)| k == name) {
            slot.1 = value;
        } else {
            self.0.push((name.to_string(), value));
        }
    }
    /// Non-empty string value (requireFlag's accepted shape). Anything else —
    /// absent, boolean-true, empty — is Node's own refusal path → caller
    /// delegates.
    pub(crate) fn req_str(&self, name: &str) -> Option<&str> {
        match self.get(name) {
            Some(FlagV::S(s)) if !s.is_empty() => Some(s),
            _ => None,
        }
    }
    /// `flags.x ? String(flags.x) : <absent>` for value flags.
    pub(crate) fn truthy_str(&self, name: &str) -> Option<&str> {
        match self.get(name) {
            Some(FlagV::S(s)) if !s.is_empty() => Some(s),
            _ => None,
        }
    }
}

/// bee.mjs parseFlags, byte-faithful: value flags consume the NEXT token
/// verbatim (even one that starts with "--"); any `--json` occurrence as a
/// FLAG sets json and is not stored. Returns None for shapes Node answers
/// itself (non-flag token, missing value) — the probe delegates.
pub(crate) fn parse_flags(tokens: &[&str]) -> Option<(Flags, bool)> {
    let mut flags = Flags(Vec::new());
    let mut json = false;
    let mut i = 0usize;
    while i < tokens.len() {
        let tok = tokens[i];
        if !tok.starts_with("--") {
            return None; // "unexpected argument" — Node's error path
        }
        let (name, value): (&str, FlagV) = match tok.find('=') {
            Some(eq) => (&tok[2..eq], FlagV::S(tok[eq + 1..].to_string())),
            None => {
                let name = &tok[2..];
                if FLAG_ALONE_BOOLEANS.contains(&name) {
                    (name, FlagV::Present)
                } else {
                    match tokens.get(i + 1) {
                        None => return None, // "flag --x requires a value"
                        Some(v) => {
                            i += 1;
                            (name, FlagV::S((*v).to_string()))
                        }
                    }
                }
            }
        };
        i += 1;
        if name == "json" {
            json = true;
            continue;
        }
        flags.insert(name, value);
    }
    Some((flags, json))
}

/// Routing gate for a verb's flag set: every parsed key must be a known flag
/// (or "help", which the dispatcher always allows) — an unknown flag is
/// Node's own emitError path, so the probe delegates.
pub(crate) fn keys_known(flags: &Flags, known: &[&str]) -> bool {
    flags
        .0
        .iter()
        .all(|(k, _)| k == "help" || known.contains(&k.as_str()))
}

/// A registry type:"number" flag through validate() + Number.parseInt(v,10).
/// Ok(Some(n)) — parsed integer part; Ok(None) — validate passes but parseInt
/// yields NaN (e.g. ".5"); Err — outside the plain-decimal grammar this port
/// models (hex, Infinity, overflow, empty) → delegate, Node owns the answer.
pub(crate) fn js_number_flag(raw: &str) -> Ex<Option<f64>> {
    let t = js_trim(raw);
    if t.is_empty() {
        return Err(Exotic); // validate(): invalid type for '' — Node's message
    }
    // grammar: [+-]? ( digits [ '.' digits? ] | '.' digits ) ( [eE][+-]?digits )?
    let bytes = t.as_bytes();
    let mut i = 0usize;
    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    let int_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let int_len = i - int_start;
    let mut frac_len = 0usize;
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let fs = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        frac_len = i - fs;
    }
    if int_len == 0 && frac_len == 0 {
        return Err(Exotic);
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
            i += 1;
        }
        let es = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == es {
            return Err(Exotic);
        }
    }
    if i != bytes.len() {
        return Err(Exotic);
    }
    let num: f64 = t.parse().map_err(|_| Exotic)?;
    if !num.is_finite() {
        return Err(Exotic); // Number(v) not finite → validate() refuses in Node
    }
    // parseInt(t, 10): optional sign + leading digit run.
    if int_len == 0 {
        return Ok(None); // ".5" → NaN
    }
    let int_str = &t[..int_start + int_len];
    let v: f64 = int_str.parse().map_err(|_| Exotic)?;
    Ok(Some(v))
}

// ─── Node path port (subset; provenance: hooks/write_guard.rs) ─────────────

#[cfg(windows)]
const SEP: char = '\\';
#[cfg(not(windows))]
const SEP: char = '/';

fn is_sep(c: char) -> bool {
    if cfg!(windows) { c == '\\' || c == '/' } else { c == '/' }
}

fn has_drive(p: &str) -> bool {
    let b = p.as_bytes();
    b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
}

fn np_check_modelable(p: &str) -> Ex<()> {
    if cfg!(windows) {
        if has_drive(p) && !p.chars().nth(2).map(is_sep).unwrap_or(false) {
            return Err(Exotic); // drive-relative "C:foo"
        }
        let mut ch = p.chars();
        if let (Some(a), Some(b)) = (ch.next(), ch.next()) {
            if is_sep(a) && is_sep(b) {
                return Err(Exotic); // UNC-ish
            }
        }
    }
    Ok(())
}

fn np_normalize_tail(path: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for seg in path.split(is_sep) {
        match seg {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            s => out.push(s),
        }
    }
    out.join(&SEP.to_string())
}

/// path.resolve over args (last wins), with the process cwd as the implicit
/// final fallback.
fn np_resolve(args: &[&str]) -> Ex<String> {
    let cwd_buf = std::env::current_dir()
        .map_err(|_| Exotic)?
        .to_string_lossy()
        .into_owned();
    let mut list: Vec<&str> = Vec::with_capacity(args.len() + 1);
    list.push(&cwd_buf);
    list.extend_from_slice(args);

    let mut device = String::new();
    let mut tail = String::new();
    let mut absolute = false;
    for p in list.iter().rev() {
        let p: &str = p;
        if p.is_empty() {
            continue;
        }
        np_check_modelable(p)?;
        let (dev, root_end, is_abs): (String, usize, bool) = if cfg!(windows) {
            if has_drive(p) {
                (p[..2].to_string(), 3usize, true)
            } else if p.chars().next().map(is_sep).unwrap_or(false) {
                (String::new(), 1usize, true)
            } else {
                (String::new(), 0usize, false)
            }
        } else if p.starts_with('/') {
            (String::new(), 1usize, true)
        } else {
            (String::new(), 0usize, false)
        };
        if !dev.is_empty() && !device.is_empty() && !dev.eq_ignore_ascii_case(&device) {
            continue;
        }
        if device.is_empty() {
            device = dev;
        }
        if !absolute {
            let part = &p[root_end.min(p.len())..];
            tail = if tail.is_empty() {
                part.to_string()
            } else if part.is_empty() {
                tail
            } else {
                format!("{}{}{}", part, SEP, tail)
            };
            absolute = is_abs;
        }
        if absolute && (!cfg!(windows) || !device.is_empty()) {
            break;
        }
    }
    if !absolute {
        return Err(Exotic);
    }
    if cfg!(windows) && device.is_empty() {
        return Err(Exotic);
    }
    let norm = np_normalize_tail(&tail);
    Ok(format!("{}{}{}", device, SEP, norm))
}

fn np_resolve1(p: &str) -> Ex<String> {
    np_resolve(&[p])
}

fn np_resolve2(base: &str, p: &str) -> Ex<String> {
    np_resolve(&[base, p])
}

fn np_dirname(p: &str) -> String {
    let chars: Vec<char> = p.chars().collect();
    let root_len = if cfg!(windows) {
        if has_drive(p) {
            3
        } else if !chars.is_empty() && is_sep(chars[0]) {
            1
        } else {
            0
        }
    } else if !chars.is_empty() && chars[0] == '/' {
        1
    } else {
        0
    };
    let mut end = chars.len();
    while end > root_len && is_sep(chars[end - 1]) {
        end -= 1;
    }
    let mut idx = None;
    let mut i = end;
    while i > root_len {
        i -= 1;
        if is_sep(chars[i]) {
            idx = Some(i);
            break;
        }
    }
    match idx {
        Some(i) => {
            let mut cut = i;
            while cut > root_len && is_sep(chars[cut - 1]) {
                cut -= 1;
            }
            chars[..cut.max(root_len)].iter().collect()
        }
        None => chars[..root_len.min(chars.len())].iter().collect(),
    }
}

fn np_basename(p: &str) -> String {
    let trimmed = p.trim_end_matches(is_sep);
    match trimmed.rfind(is_sep) {
        Some(i) => trimmed[i + 1..].to_string(),
        None => {
            if cfg!(windows) && has_drive(trimmed) && trimmed.len() <= 2 {
                String::new()
            } else {
                trimmed.to_string()
            }
        }
    }
}

// ─── reservations.mjs / lease-store.mjs ports ──────────────────────────────

pub(crate) const SESSIONLESS_SESSION_ID: &str = "\u{0}bee-reservation-sessionless\u{0}";
const LIST_ALL_HOLDS_SENTINEL: &str = "\u{0}bee-reservations-list-all\u{0}";
const RESERVATION_KINDS: [&str; 2] = ["intent", "lease"];
const DEFAULT_TTL_SECONDS: f64 = 3600.0;
const CROSS_WORKTREE_HOLDS_LOCK: &str = "cross-worktree-holds";

/// provenance: reservations.mjs normalizePath == lease-store.mjs
/// canonicalizePath (kept in sync by hand there; one copy here).
pub(crate) fn res_normalize_path(v: &str) -> String {
    let mut s = v.replace('\\', "/");
    if s.starts_with("./") {
        // /^\.\/+/ — one dot, then one-or-more slashes
        let rest = s[1..].trim_start_matches('/');
        s = rest.to_string();
    }
    while s.ends_with('/') {
        s.pop();
    }
    s
}

/// String(value || '') for a possibly-absent JSON value, then normalizePath.
fn res_normalize_value(v: Option<&Value>) -> String {
    match v {
        Some(val) if truthy(val) => res_normalize_path(&js_disp(val)),
        _ => String::new(),
    }
}

/// provenance: reservations.mjs pathsOverlap.
pub(crate) fn paths_overlap(a: &str, b: &str) -> bool {
    let left = res_normalize_path(a);
    let right = res_normalize_path(b);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    if left == right {
        return true;
    }
    let strip = |s: &str| -> String {
        if s.ends_with('*') {
            let mut t = s.trim_end_matches('*').to_string();
            while t.ends_with('/') {
                t.pop();
            }
            t
        } else {
            s.to_string()
        }
    };
    let lb = strip(&left);
    let rb = strip(&right);
    if lb == rb {
        return true;
    }
    if lb.is_empty() || rb.is_empty() {
        return true; // bare "*" covers everything
    }
    lb.starts_with(&format!("{}/", rb)) || rb.starts_with(&format!("{}/", lb))
}

fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// provenance: reservations.mjs findMainRoot/controlRootFor — the
/// self-contained, never-throwing git-common main-root walk (fails open to
/// `root` itself on every malformed shape).
pub(crate) fn control_root_for(root: &str) -> Ex<String> {
    let walked = (|| -> Ex<Option<String>> {
        // locateGitRootForRoot
        let mut dir = np_resolve1(root)?;
        let (work_root, marker) = loop {
            let m = Path::new(&dir).join(".git");
            if m.exists() {
                break (dir.clone(), m);
            }
            let parent = np_dirname(&dir);
            if parent == dir {
                return Ok(None);
            }
            dir = parent;
        };
        let is_file = match std::fs::metadata(&marker) {
            Ok(m) => m.is_file(),
            Err(_) => return Ok(None),
        };
        if !is_file {
            return Ok(Some(work_root)); // ordinary checkout: mainRoot === workRoot
        }
        let read_ptr = |file: &Path, base: &str| -> Ex<Option<String>> {
            let raw = match std::fs::read_to_string(file) {
                Ok(r) => r,
                Err(_) => return Ok(None),
            };
            let mut raw = js_trim(&raw);
            if raw.is_empty() {
                return Ok(None);
            }
            if let Some(rest) = raw.strip_prefix("gitdir:") {
                raw = js_trim(rest);
            }
            // raw.replace(/\\/g, path.sep) — a no-op on Windows.
            let fixed = if cfg!(windows) { raw.to_string() } else { raw.replace('\\', "/") };
            np_resolve2(base, &fixed).map(Some)
        };
        let Some(gitdir) = read_ptr(&marker, &work_root)? else {
            return Ok(None); // malformed — fail open
        };
        let worktrees_root = np_resolve2(&gitdir, "..")?;
        let common_git_dir = np_resolve2(&worktrees_root, "..")?;
        if np_basename(&common_git_dir) != ".git" || np_basename(&worktrees_root) != "worktrees" {
            return Ok(None);
        }
        let id = np_basename(&gitdir);
        if id.is_empty() || id == "." || id == ".." {
            return Ok(None);
        }
        let marker_s = marker.to_string_lossy().into_owned();
        let Some(reverse) = read_ptr(&Path::new(&gitdir).join("gitdir"), &gitdir)? else {
            return Ok(None);
        };
        if np_resolve1(&reverse)? != np_resolve1(&marker_s)? {
            return Ok(None);
        }
        Ok(Some(np_dirname(&common_git_dir)))
    })()?;
    Ok(walked.unwrap_or_else(|| root.to_string()))
}

/// provenance: lease-store.mjs listAllLeaseFiles + readLeaseSafe (silent skip
/// on corrupt — Node never warns there) filtered to path-type leases
/// (reservations.mjs listPathLeaseRecords), numbers reshaped like JSON.parse.
fn list_path_lease_records(root: &str) -> Ex<Vec<Map<String, Value>>> {
    let control = control_root_for(root)?;
    let leases_root = Path::new(&control).join(".bee").join("runtime").join("leases");
    let mut out = Vec::new();
    for dir in [leases_root.join("cells"), leases_root.join("paths")] {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue, // directory not created yet
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
            if !is_file {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(dir.join(&name)) {
                if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                    if let Value::Object(m) = js_numberify(&parsed)? {
                        let is_path = matches!(m.get("resource"), Some(Value::String(s)) if s.starts_with("path:"));
                        if is_path {
                            out.push(m);
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}

/// On-disk file for a path resource key (lease-store.mjs resolveResourceFile,
/// path branch): the resource is re-canonicalized from its own key, never
/// trusted from the source filename.
fn path_lease_file(control_root: &str, raw_path_id: &str) -> PathBuf {
    let canonical = res_normalize_path(raw_path_id);
    let resource_key = format!("path:{canonical}");
    Path::new(control_root)
        .join(".bee")
        .join("runtime")
        .join("leases")
        .join("paths")
        .join(format!("{}.json", sha256_hex(&resource_key)))
}

/// The reservation-shape view of a path lease (reservations.mjs
/// leaseToReservation / leaseAgent / leaseTtlSeconds). Option = absent key
/// (dropped by JSON.stringify, shows as "undefined" in text templates).
pub(crate) struct Resv {
    pub(crate) agent: Option<Value>,
    pub(crate) cell: Option<Value>,
    pub(crate) path: String,
    ttl_seconds: Option<f64>, // None = NaN → JSON null
    reserved_at: Option<Value>,
    pub(crate) session: Option<Value>,
    pub(crate) kind: Value,
}

fn lease_to_reservation(rec: &Map<String, Value>) -> Ex<Resv> {
    let resource = match rec.get("resource") {
        Some(Value::String(s)) => s.clone(),
        _ => return Err(Exotic), // callers filter to path leases; defensive
    };
    let ttl = match rec.get("expires_at") {
        None | Some(Value::Null) => Some(0.0), // never-expires sentinel
        Some(exp) => {
            let e = date_parse_val(Some(exp))?;
            let a = date_parse_val(rec.get("acquired_at"))?;
            match (e, a) {
                (Some(e), Some(a)) => Some(js_round((e - a) / 1000.0).max(0.0)),
                _ => None, // NaN flows through Math.round/max
            }
        }
    };
    let agent = rec.get("workspace_id").map(|w| match w {
        Value::String(s) if s.starts_with("agent:") => {
            Value::String(s["agent:".len()..].to_string())
        }
        other => other.clone(),
    });
    let session = match rec.get("session_id") {
        Some(v) if truthy(v) && !v_is_str(v, SESSIONLESS_SESSION_ID) => Some(v.clone()),
        _ => None,
    };
    let kind = match rec.get("kind") {
        Some(v) if truthy(v) => v.clone(),
        _ => Value::String("lease".into()),
    };
    Ok(Resv {
        agent,
        cell: rec.get("workflow_id").cloned(),
        path: resource["path:".len()..].to_string(),
        ttl_seconds: ttl,
        reserved_at: rec.get("acquired_at").cloned(),
        session,
        kind,
    })
}

/// Serialization order pinned by leaseToReservation's object literal: agent,
/// cell, path, ttl_seconds, reserved_at, released_at, [session], kind —
/// undefined-valued keys drop out of JSON.stringify.
fn resv_to_value(r: &Resv) -> Value {
    let mut m = Map::new();
    if let Some(a) = &r.agent {
        m.insert("agent".into(), a.clone());
    }
    if let Some(c) = &r.cell {
        m.insert("cell".into(), c.clone());
    }
    m.insert("path".into(), Value::String(r.path.clone()));
    m.insert(
        "ttl_seconds".into(),
        r.ttl_seconds
            .and_then(Number::from_f64)
            .map(Value::Number)
            .unwrap_or(Value::Null), // JSON.stringify(NaN) → null
    );
    if let Some(ra) = &r.reserved_at {
        m.insert("reserved_at".into(), ra.clone());
    }
    m.insert("released_at".into(), Value::Null);
    if let Some(s) = &r.session {
        m.insert("session".into(), s.clone());
    }
    m.insert("kind".into(), r.kind.clone());
    Value::Object(m)
}

/// provenance: reservations.mjs isLeaseRecordExpired (raw record, never the
/// translated shape).
fn lease_record_expired(rec: &Map<String, Value>, now: f64) -> Ex<bool> {
    match rec.get("expires_at") {
        None | Some(Value::Null) => Ok(false),
        Some(v) => match date_parse_val(Some(v))? {
            None => Ok(false),
            Some(ms) => Ok(ms <= now),
        },
    }
}

/// provenance: reservations.mjs listReservations.
///
/// pub(crate) since R6: verbs/state_group.rs's `state rebuild-projections`
/// needs it for rebuildReservationsProjection, and verbs/cells.rs's
/// `cells claim-next` needs it for findSessionConflicts.
pub(crate) fn list_reservations(root: &str, active_only: bool, now: f64) -> Ex<Vec<Resv>> {
    let mut out = Vec::new();
    for rec in list_path_lease_records(root)? {
        if active_only && lease_record_expired(&rec, now)? {
            continue;
        }
        out.push(lease_to_reservation(&rec)?);
    }
    Ok(out)
}

/// provenance: reservations.mjs reservationsPath.
pub(crate) fn reservations_path(root: &Path) -> PathBuf {
    root.join(".bee").join("reservations.json")
}

/// provenance: reservations.mjs rebuildReservationsProjection (msn-16/msn-18b).
/// Returns the row count (`{ authoritative: true, count }`'s count — the
/// authoritative flag is an unconditional `true` at the call site, because this
/// projection is never gated on workflow records).
///
/// The READ is control-rooted inside listReservations → listPathLeaseRecords;
/// the WRITE deliberately stays on the caller's own workspace root (msn-18b:
/// `.bee/reservations.json` is a legacy single-checkout DISPLAY projection).
///
/// A row whose `reserved_at` is not a string would make Node's comparator
/// inconsistent (`undefined < undefined` is false, so it answers 1 both ways)
/// and V8's TimSort order becomes implementation detail — delegate instead.
pub(crate) fn rebuild_reservations_projection(root: &Path) -> Ex<usize> {
    let root_s = root.to_str().ok_or(Exotic)?;
    let rows = list_reservations(root_s, true, now_ms())?;
    // `a.reserved_at !== b.reserved_at ? (a.reserved_at < b.reserved_at ? -1 : 1)
    //  : a.path !== b.path ? (a.path < b.path ? -1 : 1) : 0` — JS string
    // relational comparison is UTF-16 code-unit lexicographic.
    let mut keyed: Vec<(Vec<u16>, Vec<u16>, Resv)> = Vec::with_capacity(rows.len());
    for r in rows {
        let Some(Value::String(ra)) = r.reserved_at.clone() else {
            return Err(Exotic);
        };
        let path_key: Vec<u16> = r.path.encode_utf16().collect();
        keyed.push((ra.encode_utf16().collect(), path_key, r));
    }
    keyed.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    let rows: Vec<Value> = keyed.iter().map(|(_, _, r)| resv_to_value(r)).collect();
    let count = rows.len();
    write_json_atomic(
        &reservations_path(root),
        &json!({ "reservations": Value::Array(rows) }),
    )
    .map_err(|_| Exotic)?;
    Ok(count)
}

/// provenance: reservations.mjs findConflicts (agent-keyed, activeOnly).
fn find_conflicts(root: &str, agent: &str, path: &str, now: f64) -> Ex<Vec<Resv>> {
    let mut out = Vec::new();
    for resv in list_reservations(root, true, now)? {
        let same_agent = matches!(&resv.agent, Some(Value::String(s)) if s == agent);
        if !same_agent && paths_overlap(&resv.path, path) {
            out.push(resv);
        }
    }
    Ok(out)
}

/// provenance: reservations.mjs isHardConflict.
fn is_hard_conflict(resv: &Resv, target_path: &str) -> bool {
    let is_intent = v_is_str(&resv.kind, "intent");
    !(is_intent && res_normalize_path(&resv.path) != res_normalize_path(target_path))
}

// ─── worktree-holds.mjs ports ──────────────────────────────────────────────

fn holds_ledger_path(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime").join("cross-worktree-holds.json")
}

/// provenance: worktree-holds.mjs readStore — a missing, corrupt or
/// shape-less file is an empty ledger. A corrupt file WARNS once and then
/// takes `readJson(..., null)`'s fallback, which Node's `!store` guard turned
/// into `{ holds: [] }` — identical result, one line of explanation.
/// `null` entries (a JS property-access crash downstream) stay Exotic.
fn read_holds_store(root: &Path) -> Ex<Value> {
    let store = match read_json(&holds_ledger_path(root)) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => {
            crate::fsutil::warn_corrupt_json(&holds_ledger_path(root));
            None
        }
        ReadJson::Parsed(v) => Some(js_numberify(&v)?),
    };
    let ok_shape = store
        .as_ref()
        .map(|s| matches!(jget(s, "holds"), Some(Value::Array(_))))
        .unwrap_or(false);
    if !ok_shape {
        return Ok(json!({ "holds": [] }));
    }
    let store = store.unwrap();
    if let Some(Value::Array(holds)) = jget(&store, "holds") {
        if holds.iter().any(|h| h.is_null()) {
            return Err(Exotic); // `hold.released_at` on null throws in Node
        }
    }
    Ok(store)
}

fn holds_of(store: &Value) -> &Vec<Value> {
    match jget(store, "holds") {
        Some(Value::Array(a)) => a,
        _ => unreachable!("read_holds_store guarantees the shape"),
    }
}

/// provenance: worktree-holds.mjs isExpired.
fn hold_expired(hold: &Value, now: f64) -> Ex<bool> {
    let ttl = match jget(hold, "ttl_seconds") {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        _ => return Ok(false), // Number.isFinite(non-number) → false
    };
    if !ttl.is_finite() || ttl <= 0.0 {
        return Ok(false);
    }
    match date_parse_val(jget(hold, "mirrored_at"))? {
        None => Ok(false),
        Some(m) => Ok(m + ttl * 1000.0 <= now),
    }
}

/// provenance: worktree-holds.mjs isActive.
fn hold_active(hold: &Value, now: f64) -> Ex<bool> {
    let released = jget(hold, "released_at");
    let unreleased = matches!(released, None | Some(Value::Null)); // == null
    Ok(unreleased && !hold_expired(hold, now)?)
}

/// provenance: worktree-holds.mjs findForeignHolds (read-only, unlocked).
fn find_foreign_holds<'a>(
    store: &'a Value,
    acting: &str,
    request_path: &str,
    now: f64,
) -> Ex<Vec<&'a Value>> {
    let mut out = Vec::new();
    for hold in holds_of(store) {
        if !hold_active(hold, now)? {
            continue;
        }
        // hold.holder !== acting (strict) — non-strings can never equal it.
        let same = matches!(jget(hold, "holder"), Some(Value::String(s)) if s == acting);
        if same {
            continue;
        }
        let hold_path = res_normalize_value(jget(hold, "path"));
        if paths_overlap(&hold_path, request_path) {
            out.push(hold);
        }
    }
    Ok(out)
}

/// provenance: bee.mjs holdForeignExpiry.
fn hold_foreign_expiry(hold: &Value) -> Ex<String> {
    let mirrored = date_parse_val(jget(hold, "mirrored_at"))?;
    let ttl = match jget(hold, "ttl_seconds") {
        Some(Value::Number(n)) => n.as_f64(),
        _ => None,
    };
    match (mirrored, ttl) {
        (Some(m), Some(t)) if t > 0.0 => Ok(format!("expires {}", iso_from_ms(m + t * 1000.0)?)),
        _ => Ok("no expiry".to_string()),
    }
}

// ─── claims.mjs ports (sessions slice) ─────────────────────────────────────

const HEARTBEAT_STALE_SECONDS: f64 = 900.0;

/// provenance: claims.mjs listSessionRecords (fail-open flavor). A corrupt
/// record WARNS once and is skipped — `readJson(file, null)`'s fallback fails
/// readSession's `!session` guard, so the scan continues over the rest.
fn list_session_records(control_root: &str) -> Ex<Vec<Map<String, Value>>> {
    let dir = Path::new(control_root).join(".bee").join("sessions");
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        let stem = &name[..name.len() - ".json".len()];
        // requireId inside sessionPath: trimmed; separators/'..' throw → the
        // catch in readSession reads them as "no session".
        let id = js_trim(stem);
        if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains("..") {
            continue;
        }
        let file = dir.join(format!("{id}.json"));
        match read_json(&file) {
            ReadJson::Missing => continue,
            // readJson's `null` fallback fails readSession's `!session`
            // guard, so the record is skipped and the scan continues.
            ReadJson::Corrupt => {
                crate::fsutil::warn_corrupt_json(&file);
                continue;
            }
            ReadJson::Parsed(v) => {
                if let Value::Object(m) = js_numberify(&v)? {
                    if matches!(m.get("id"), Some(Value::String(s)) if s == id) {
                        out.push(m);
                    }
                }
            }
        }
    }
    Ok(out)
}

/// provenance: claims.mjs heartbeatStale.
fn heartbeat_stale(session: &Map<String, Value>, now: f64) -> Ex<bool> {
    match date_parse_val(session.get("last_heartbeat"))? {
        None => Ok(true),
        Some(ms) => Ok(ms + HEARTBEAT_STALE_SECONDS * 1000.0 <= now),
    }
}

fn env_nonempty(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(v) if !js_trim(&v).is_empty() => Some(js_trim(&v).to_string()),
        _ => None,
    }
}

/// provenance: claims.mjs resolveSessionId (flag → BEE_SESSION_ID →
/// CLAUDE_CODE_SESSION_ID → single-live-session adoption → null).
///
/// pub(crate) since the `worktree new` port: bee.mjs's handleWorktreeNew
/// resolves the acting session the same way, against controlRootFor(mainRoot).
pub(crate) fn resolve_session_id(flag: Option<&str>, control_root: &str) -> Ex<Option<String>> {
    if let Some(f) = flag {
        if !js_trim(f).is_empty() {
            return Ok(Some(js_trim(f).to_string()));
        }
    }
    if let Some(v) = env_nonempty("BEE_SESSION_ID") {
        return Ok(Some(v));
    }
    if let Some(v) = env_nonempty("CLAUDE_CODE_SESSION_ID") {
        return Ok(Some(v));
    }
    let records = list_session_records(control_root)?;
    let mut fresh: Vec<&Map<String, Value>> = Vec::new();
    for r in &records {
        if !heartbeat_stale(r, now_ms())? {
            fresh.push(r);
        }
    }
    if fresh.len() == 1 {
        if let Some(Value::String(id)) = fresh[0].get("id") {
            return Ok(Some(id.clone()));
        }
    }
    Ok(None)
}

/// provenance: claims.mjs isConcurrentMode (no exclusion — the sessionless
/// reserve caller has no id of its own).
fn is_concurrent_mode(control_root: &str) -> Ex<bool> {
    is_concurrent_mode_excluding(control_root, None)
}

/// provenance: claims.mjs isConcurrentMode with `excludeSessionId` — the
/// acting session's OWN heartbeat is never "another" live session.
///
/// `strict` has no separate arm here. CUTOVER: `list_session_records` used to
/// answer `Exotic` for an unreadable record, which made this port
/// strict-equivalent by construction. It now warns and SKIPS that record,
/// matching Node's own non-strict readSession — so an unreadable record reads
/// as "not a live session" and is loudly reported, rather than silently
/// disappearing. (guards.mjs's strict mode, which throws instead, is a
/// separate caller and is not reached from here.)
///
/// pub(crate) since the `worktree new` port (bee.mjs's wcg-3 guard).
pub(crate) fn is_concurrent_mode_excluding(
    control_root: &str,
    exclude_session_id: Option<&str>,
) -> Ex<bool> {
    let exclude = exclude_session_id.map(js_trim).unwrap_or("");
    let now = now_ms();
    for session in list_session_records(control_root)? {
        let is_excluded = matches!(session.get("id"), Some(Value::String(s)) if s == exclude);
        if !is_excluded && !heartbeat_stale(&session, now)? {
            return Ok(true);
        }
    }
    Ok(false)
}

// ─── emission plumbing (mirrors status_brief.rs / bee.mjs emit/emitError) ──
// pub(crate): verbs/decisions.rs shares this exact emit/fail/timing shape.

pub(crate) struct Ctx {
    pub(crate) root: PathBuf,
    cmd: &'static str,
    use_json: bool,
    t0: Instant,
    drift_changed: bool,
    drift_hint: &'static str,
}

pub(crate) enum Pre {
    Go(Ctx),
    Emitted(ExitCode),
}

pub(crate) fn prelude(cmd: &'static str, use_json: bool, t0: Instant) -> Option<Pre> {
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::NeedsNode => return None,
        Roots::None => return Some(Pre::Emitted(emit_no_root_error(&cwd, cmd, use_json, t0))),
    };
    let drift = check_manifest_drift(&root).ok()?;
    Some(Pre::Go(Ctx {
        root,
        cmd,
        use_json,
        t0,
        drift_changed: drift.manifest_changed,
        drift_hint: drift.hint,
    }))
}

/// The WORKTREE-NATIVE prelude, used ONLY by this module's four verbs.
///
/// Deliberately separate from `prelude` above: verbs/decisions.rs,
/// verbs/cells.rs and verbs/drivers.rs share that one, and none of them has
/// had its worktree-sensitive branches ported — widening the shared door
/// would flip them silently. They keep `resolve_store_root`'s
/// `LinkedValid => NeedsNode` arm; only the reservations verbs opt in here.
pub(crate) enum PreWt {
    Go(Ctx, StoreRoots),
    Emitted(ExitCode),
}

fn prelude_worktree(cmd: &'static str, use_json: bool, t0: Instant) -> Option<PreWt> {
    let cwd = std::env::current_dir().ok()?;
    let roots = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(r) => r,
        RootsWt::NeedsNode => return None,
        RootsWt::None => return Some(PreWt::Emitted(emit_no_root_error(&cwd, cmd, use_json, t0))),
    };
    let drift = check_manifest_drift(&roots.root).ok()?;
    Some(PreWt::Go(
        Ctx {
            root: roots.root.clone(),
            cmd,
            use_json,
            t0,
            drift_changed: drift.manifest_changed,
            drift_hint: drift.hint,
        },
        roots,
    ))
}

impl Ctx {
    /// bee.mjs emit(): drift line (stderr) + result (stdout) + timing.
    fn emit(&self, result: &Value, text: &str, exit_code: u8) -> ExitCode {
        if self.drift_changed {
            eprintln!("manifest_changed: true — {}", self.drift_hint);
        }
        if self.use_json {
            println!("{}", jsjson::stringify_pretty(result));
        } else {
            println!("{text}");
        }
        record_timing(&self.root, self.cmd, self.t0, exit_code == 0);
        ExitCode::from(exit_code)
    }

    /// bee.mjs emitError(): no drift line, {"error"} or stderr, exit 1.
    fn fail(&self, message: &str) -> ExitCode {
        if self.use_json {
            println!("{}", jsjson::stringify(&json!({ "error": message })));
        } else {
            eprintln!("{message}");
        }
        record_timing(&self.root, self.cmd, self.t0, false);
        ExitCode::FAILURE
    }
}

/// A handler outcome: an emitted payload or a thrown-Error message.
pub(crate) enum Out {
    Emit(Value, String, u8),
    Thrown(String),
}
pub(crate) type R2<T> = Result<T, Err2>;
#[derive(Debug)]
pub(crate) enum Err2 {
    Ex,
    Msg(String),
}
impl From<Exotic> for Err2 {
    fn from(_: Exotic) -> Self {
        Err2::Ex
    }
}

// ─── routing ───────────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "reservations" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let toks: Vec<&str> = args[2..]
        .iter()
        .map(|a| a.to_str())
        .collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None; // Node renders command-scoped help
    }
    let (flags, use_json) = parse_flags(&toks)?;
    match verb {
        "list" => run_list(flags, use_json, t0),
        "reserve" => run_reserve(flags, use_json, t0),
        "release" => run_release(flags, use_json, t0),
        "sweep" => run_sweep(flags, use_json, t0),
        _ => None,
    }
}

pub(crate) fn finish(ctx: &Ctx, out: R2<Out>) -> Option<ExitCode> {
    match out {
        Ok(Out::Emit(result, text, code)) => Some(ctx.emit(&result, &text, code)),
        Ok(Out::Thrown(msg)) => Some(ctx.fail(&msg)),
        Err(Err2::Msg(msg)) => Some(ctx.fail(&msg)),
        Err(Err2::Ex) => None,
    }
}

// ─── reservations list ─────────────────────────────────────────────────────

fn run_list(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["active-only"]) {
        return None;
    }
    // validate(): a boolean-typed flag given as =value must be true/false.
    match flags.get("active-only") {
        None | Some(FlagV::Present) => {}
        Some(FlagV::S(s)) if s == "true" || s == "false" => {}
        Some(FlagV::S(_)) => return None,
    }
    let active_only = matches!(flags.get("active-only"), Some(FlagV::Present));

    let (ctx, roots) = match prelude_worktree("reservations list", use_json, t0)? {
        PreWt::Go(c, r) => (c, r),
        PreWt::Emitted(code) => return Some(code),
    };
    // resolveMainRoot(root): the shared ledger always lives in MAIN's store.
    let ledger_root = roots.main_root();
    let root_s = ctx.root.to_str()?.to_string();
    let out = (|| -> R2<Out> {
        let reservations = list_reservations(&root_s, active_only, now_ms())?;
        let store = read_holds_store(&ledger_root)?;
        let cross: Vec<Value> = find_foreign_holds(&store, LIST_ALL_HOLDS_SENTINEL, "*", now_ms())?
            .into_iter()
            .cloned()
            .collect();
        let mut cross_lines = Vec::new();
        for h in &cross {
            let cell = match jget(h, "cell") {
                Some(v) if truthy(v) => js_disp(v),
                _ => "unknown".to_string(),
            };
            cross_lines.push(format!(
                "{} | cell {} | {} | mirrored {} | {}",
                js_disp_opt(jget(h, "holder")),
                cell,
                js_disp_opt(jget(h, "path")),
                js_disp_opt(jget(h, "mirrored_at")),
                hold_foreign_expiry(h)?
            ));
        }

        let mut lines: Vec<String> = Vec::new();
        if reservations.is_empty() {
            lines.push("No reservations.".to_string());
        } else {
            lines.push(
                reservations
                    .iter()
                    .map(|r| {
                        // released_at is null by construction (msn-16 shim) —
                        // the ternary's released branch is unreachable.
                        format!(
                            "{} | cell {} | {} | reserved {} | active/expired by TTL",
                            js_disp_opt(r.agent.as_ref()),
                            js_disp_opt(r.cell.as_ref()),
                            r.path,
                            js_disp_opt(r.reserved_at.as_ref()),
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
        if !cross.is_empty() {
            lines.push("cross_worktree:".to_string());
            lines.extend(cross_lines);
        }
        let result = json!({
            "reservations": reservations.iter().map(resv_to_value).collect::<Vec<_>>(),
            "cross_worktree": cross,
        });
        Ok(Out::Emit(result, lines.join("\n"), 0))
    })();
    finish(&ctx, out)
}

// ─── reservations reserve ──────────────────────────────────────────────────

struct ReserveParams {
    agent: String,
    cell: String,
    path: String,
    ttl: Option<f64>,
    session: Option<String>,
    kind: Option<String>,
}

fn run_reserve(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["agent", "cell", "path", "ttl", "session", "kind"]) {
        return None;
    }
    let agent = flags.req_str("agent")?.to_string();
    let cell = flags.req_str("cell")?.to_string();
    let path = flags.req_str("path")?.to_string();
    let ttl_flag = match flags.get("ttl") {
        None => None,
        Some(FlagV::S(s)) => Some(s.clone()),
        Some(FlagV::Present) => return None,
    };
    let session = flags.truthy_str("session").map(str::to_string);
    let kind = flags.truthy_str("kind").map(str::to_string);

    let (ctx, roots) = match prelude_worktree("reservations reserve", use_json, t0)? {
        PreWt::Go(c, r) => (c, r),
        PreWt::Emitted(code) => return Some(code),
    };
    let topology = roots.hold_topology();
    // handleReservationsReserve's own --ttl gate runs before everything else.
    let ttl: Option<f64> = match &ttl_flag {
        None => None,
        Some(raw) => match js_number_flag(raw) {
            Err(_) => return None, // Node's validate() owns the message
            Ok(parsed) => match parsed {
                Some(v) if v.is_finite() && v > 0.0 => Some(v),
                _ => return Some(ctx.fail("--ttl must be a positive integer (seconds).")),
            },
        },
    };
    let root_s = ctx.root.to_str()?.to_string();
    let params = ReserveParams { agent, cell, path, ttl, session, kind };

    let topo = topology.as_ref().map(|(m, h)| Topo { main_root: m, holder: h });

    // Pre-checks (pure reads) — any Exotic delegates before the lock, so the
    // Node re-run owns the whole command including its own lock telemetry.
    if reserve_prechecks(topo, &root_s, &params).is_err() {
        return None;
    }

    let out = reserve_exec(topo, &root_s, &params, lock::MAX_ATTEMPTS);
    finish(&ctx, out)
}

/// bee.mjs resolveHoldTopology's `{mainRoot, holder}`, borrowed for one
/// command. `None` (an UNGRANTED linked worktree) means the cross-worktree
/// section is skipped entirely — no lock, no ledger read, no mirror row.
#[derive(Clone, Copy)]
struct Topo<'a> {
    main_root: &'a Path,
    holder: &'a str,
}

fn reserve_prechecks(topo: Option<Topo>, root_s: &str, p: &ReserveParams) -> Ex<()> {
    let now = now_ms();
    if let Some(t) = topo {
        if res_normalize_path(&p.path).is_empty() {
            // insertHold would throw AFTER the lease write — a state Node must
            // own. Without a topology insertHold never runs, so an empty
            // normalized path is reserve()'s own plain "path is required."
            return Err(Exotic);
        }
        let store = read_holds_store(t.main_root)?;
        for hold in holds_of(&store) {
            hold_active(hold, now)?; // date-modelability probe
            hold_foreign_expiry(hold)?;
        }
    }
    let control_root = control_root_for(root_s)?;
    let flag_or_env_session = p
        .session
        .as_deref()
        .map(|s| !js_trim(s).is_empty())
        .unwrap_or(false)
        || env_nonempty("BEE_SESSION_ID").is_some()
        || env_nonempty("CLAUDE_CODE_SESSION_ID").is_some();
    if !flag_or_env_session {
        for r in list_session_records(&control_root)? {
            heartbeat_stale(&r, now)?;
        }
    }
    for rec in list_path_lease_records(root_s)? {
        lease_record_expired(&rec, now)?;
        lease_to_reservation(&rec)?;
    }
    // computeExpiresAt / toISOString range for the record about to be built.
    let ttl_eff = p.ttl.unwrap_or(DEFAULT_TTL_SECONDS);
    iso_from_ms(now + ttl_eff * 1000.0 + 60_000.0)?;
    Ok(())
}

/// The whole reservePathAtomic section — separated from run_reserve so tests
/// can drive it against a fixture root (max_attempts lets contention tests
/// refuse instantly like a hook would).
///
/// bee.mjs reservePathAtomic: WITH a topology the foreign-hold check, the
/// local reserve and the mirror-insert are ONE atomic section under
/// withHoldsLock(topology.mainRoot) (hardening-1-7-10 D3). WITHOUT one — an
/// ungranted linked worktree — Node runs the bare `doReserve()` and takes no
/// shared lock at all, so this port must not take one either: a LOCK_BUSY
/// refusal Node could never emit would be a C2 break of its own.
fn reserve_exec(
    topo: Option<Topo>,
    root_s: &str,
    p: &ReserveParams,
    max_attempts: u32,
) -> R2<Out> {
    let Some(t) = topo else {
        return reserve_locked(None, root_s, p);
    };
    let guard = match lock::acquire_store_lock(t.main_root, CROSS_WORKTREE_HOLDS_LOCK, max_attempts)
    {
        Ok(g) => g,
        Err(busy) => return Err(Err2::Msg(busy.message())),
    };
    let out = reserve_locked(Some(t), root_s, p);
    drop(guard); // Node releases in withStoreLock's finally, before emit
    out
}

fn reserve_locked(topo: Option<Topo>, root_s: &str, p: &ReserveParams) -> R2<Out> {
    // findForeignHolds runs its own fresh readStore inside the section.
    let mut store = match topo {
        Some(t) => Some(read_holds_store(t.main_root)?),
        None => None,
    };
    if let (Some(t), Some(store)) = (topo, store.as_ref()) {
        let foreign = find_foreign_holds(store, t.holder, &p.path, now_ms())?;
        if let Some(hold) = foreign.first() {
            let hold = (*hold).clone();
            let mut m = Map::new();
            m.insert("ok".into(), Value::Bool(false));
            m.insert("code".into(), Value::String("FOREIGN_HOLD".into()));
            if let Some(v) = jget(&hold, "holder") {
                m.insert("holder".into(), v.clone());
            }
            if let Some(v) = jget(&hold, "feature") {
                m.insert("feature".into(), v.clone());
            }
            if let Some(v) = jget(&hold, "cell") {
                m.insert("cell".into(), v.clone());
            }
            if let Some(v) = jget(&hold, "path") {
                m.insert("path".into(), v.clone());
            }
            let expiry = hold_foreign_expiry(&hold)?;
            m.insert("expires".into(), Value::String(expiry.clone()));
            let or_unknown = |k: &str| match jget(&hold, k) {
                Some(v) if truthy(v) => js_disp(v),
                _ => "unknown".to_string(),
            };
            let text = format!(
                "bee cross-worktree hold: \"{}\" is held by checkout \"{}\" (feature {}, cell {}), {}. Wait for the hold to expire or coordinate with that checkout — a cross-worktree hold is a hard block.",
                js_disp_opt(jget(&hold, "path")),
                js_disp_opt(jget(&hold, "holder")),
                or_unknown("feature"),
                or_unknown("cell"),
                expiry
            );
            return Ok(Out::Emit(Value::Object(m), text, 1));
        }
    }

    // ── reserve() (lib/reservations.mjs) ───────────────────────────────────
    if js_trim(&p.agent).is_empty() {
        return Ok(Out::Thrown("reserve: agent is required.".into()));
    }
    if js_trim(&p.cell).is_empty() {
        return Ok(Out::Thrown("reserve: cell id is required.".into()));
    }
    if js_trim(&p.path).is_empty() {
        return Ok(Out::Thrown("reserve: path is required.".into()));
    }
    let kind = p.kind.clone().unwrap_or_else(|| "lease".to_string());
    if !RESERVATION_KINDS.contains(&kind.as_str()) {
        return Ok(Out::Thrown(format!(
            "reserve: kind must be one of intent/lease (got {}).",
            js_quote(&kind)
        )));
    }
    let control_root = control_root_for(root_s)?;
    let resolved_session = resolve_session_id(p.session.as_deref(), &control_root)?;
    if resolved_session.is_none() && is_concurrent_mode(&control_root)? {
        let reason = format!(
            "reserve: cannot reserve \"{}\" without identifying the acting session while another session is active — pass --session-id or set BEE_SESSION_ID (CLAUDE_CODE_SESSION_ID is also honored).",
            js_trim(&p.path)
        );
        let result = json!({
            "ok": false,
            "code": "SESSION_REQUIRED",
            "reason": reason,
            "conflicts": [],
        });
        let text = "Reservation CONFLICT — return [BLOCKED] to the orchestrator:".to_string();
        return Ok(Out::Emit(result, text, 1));
    }

    let trimmed_agent = js_trim(&p.agent).to_string();
    let trimmed_cell = js_trim(&p.cell).to_string();
    let now = now_ms(); // reserve()'s own `now = Date.now()` default

    let overlap_conflicts: Vec<Resv> = find_conflicts(&control_root, &trimmed_agent, &p.path, now)?
        .into_iter()
        .filter(|c| is_hard_conflict(c, &p.path))
        .collect();
    if !overlap_conflicts.is_empty() {
        return Ok(conflict_out(&overlap_conflicts));
    }

    // acquireLeases (lease-store.mjs): O_EXCL create of the one path lease.
    let canonical = res_normalize_path(&p.path);
    let resource_key = format!("path:{canonical}");
    let ttl_eff = p.ttl.unwrap_or(DEFAULT_TTL_SECONDS);
    let acquired_at = iso_from_ms(now)?;
    let expires_at: Option<String> = if ttl_eff.is_finite() && ttl_eff > 0.0 {
        Some(iso_from_ms(now + ttl_eff * 1000.0)?)
    } else {
        None // computeExpiresAt: non-positive ttl never expires
    };
    let session_for_lease = resolved_session
        .clone()
        .unwrap_or_else(|| SESSIONLESS_SESSION_ID.to_string());
    let mut record = Map::new();
    record.insert("resource".into(), Value::String(resource_key.clone()));
    record.insert("mode".into(), Value::String("write".into()));
    record.insert("workflow_id".into(), Value::String(trimmed_cell.clone()));
    record.insert("session_id".into(), Value::String(session_for_lease));
    record.insert(
        "workspace_id".into(),
        Value::String(format!("agent:{trimmed_agent}")),
    );
    record.insert("epoch".into(), Value::Number(Number::from_f64(0.0).unwrap()));
    record.insert("acquired_at".into(), Value::String(acquired_at));
    record.insert(
        "expires_at".into(),
        expires_at.clone().map(Value::String).unwrap_or(Value::Null),
    );
    record.insert("kind".into(), Value::String(kind));

    let lease_file = path_lease_file(&control_root, &p.path);
    if let Some(dir) = lease_file.parent() {
        std::fs::create_dir_all(dir).map_err(|_| Err2::Ex)?;
    }
    let body = format!("{}\n", jsjson::stringify_pretty(&Value::Object(record.clone())));
    let create = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lease_file);
    match create {
        Ok(mut f) => {
            use std::io::Write;
            f.write_all(body.as_bytes()).map_err(|_| Err2::Ex)?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Lost an exact-path race — report like a pre-check conflict.
            let holder_resv: Option<Resv> = match std::fs::read_to_string(&lease_file) {
                Ok(text) => match serde_json::from_str::<Value>(&text) {
                    Ok(v) => {
                        let v = js_numberify(&v)?;
                        match v {
                            Value::Object(m)
                                if matches!(m.get("resource"), Some(Value::String(s)) if s.starts_with("path:")) =>
                            {
                                Some(lease_to_reservation(&m)?)
                            }
                            _ => None,
                        }
                    }
                    Err(_) => None, // readLeaseSafe: unparseable → null holder
                },
                Err(_) => None,
            };
            let conflicts: Vec<Resv> = holder_resv.into_iter().collect();
            return Ok(conflict_out(&conflicts));
        }
        Err(_) => return Err(Err2::Ex),
    }

    let reservation = lease_to_reservation(&record)?;

    // insertHold (worktree-holds.mjs), called from inside the held section —
    // the ledger read from this same section is reused (see module header).
    // Skipped wholesale without a topology, exactly like Node.
    if let (Some(t), Some(store)) = (topo, store.as_mut()) {
        let ttl_secs = reservation.ttl_seconds.unwrap_or(f64::NAN);
        let hold_ttl = if ttl_secs.is_finite() && ttl_secs > 0.0 {
            ttl_secs.floor()
        } else {
            DEFAULT_TTL_SECONDS
        };
        let mut hold = Map::new();
        hold.insert(
            "path".into(),
            Value::String(res_normalize_path(&reservation.path)),
        );
        // topology.holder: "main" from an ordinary checkout, the git-verified
        // worktree id from a granted one.
        hold.insert("holder".into(), Value::String(t.holder.to_string()));
        hold.insert("feature".into(), Value::Null);
        hold.insert(
            "session".into(),
            match &reservation.session {
                Some(Value::String(s)) if !js_trim(s).is_empty() => {
                    Value::String(js_trim(s).to_string())
                }
                _ => Value::Null,
            },
        );
        hold.insert("cell".into(), Value::String(trimmed_cell.clone()));
        hold.insert(
            "ttl_seconds".into(),
            Value::Number(Number::from_f64(hold_ttl).ok_or(Err2::Ex)?),
        );
        hold.insert("mirrored_at".into(), Value::String(now_iso()));
        hold.insert("released_at".into(), Value::Null);
        if let Some(Value::Array(holds)) = store.get_mut("holds") {
            holds.push(Value::Object(hold));
        }
        write_json_atomic(&holds_ledger_path(t.main_root), store).map_err(|_| Err2::Ex)?;
    }

    let resv_value = resv_to_value(&reservation);
    let text = format!(
        "Reserved \"{}\" for {} (cell {}, ttl {}s).",
        reservation.path,
        js_disp_opt(reservation.agent.as_ref()),
        js_disp_opt(reservation.cell.as_ref()),
        reservation
            .ttl_seconds
            .map(jsjson::js_f64_to_string)
            .unwrap_or_else(|| "NaN".to_string()),
    );
    let mut result = Map::new();
    result.insert("ok".into(), Value::Bool(true));
    result.insert("reservation".into(), resv_value);
    Ok(Out::Emit(Value::Object(result), text, 0))
}

/// reservePathAtomic's TYPED verdict — bee.mjs returns `{refusal}` (the
/// foreign hold) or `{reserveResult}` (reserve()'s own ok/conflict answer)
/// and leaves presentation to each caller: `reservations reserve` renders the
/// FOREIGN_HOLD / CONFLICT text, `dispatch prepare --claim` renders its own
/// `- checkout "…" holds "…"` lines and then unwinds.
///
/// Derived from the SAME `Out` `run_reserve` emits, never from a second copy
/// of the section — the values inside are the identical JS values Node's
/// `section.refusal` / `section.reserveResult` carry.
pub(crate) enum ReserveOutcome {
    /// `{refusal}` — the cross-worktree hold, with `holder`/`feature`/`cell`/
    /// `path` copied verbatim off the ledger row.
    ForeignHold(Map<String, Value>),
    /// `reserveResult.conflicts` — each row has `agent`/`path`/`cell`.
    Conflicts(Vec<Value>),
    /// `reserveResult.reservation` — note `.path` is the NORMALIZED path the
    /// lease record carries, not the raw `files[]` entry.
    Reserved(Value),
    /// reserve()'s own argument refusals (thrown at the CLI boundary).
    Thrown(String),
}

/// bee.mjs's `reservePathAtomic(root, {agent, cell, path})` — the ONE reserve
/// door `reservations reserve` and `dispatch prepare --claim` share, so the
/// foreign-hold check, the local reserve and the mirror-insert cannot diverge
/// between them.
///
/// pub(crate) since the `dispatch prepare --claim` port. `ttl`/`session`/
/// `kind` are structurally absent because the dispatch call site passes none:
/// reserve() then defaults to DEFAULT_TTL_SECONDS, `resolveSessionId` from the
/// environment only, and kind `'lease'` (hard conflicts).
pub(crate) fn reserve_path_atomic(
    topo: Option<(&Path, &str)>,
    root_s: &str,
    agent: &str,
    cell: &str,
    path: &str,
) -> R2<ReserveOutcome> {
    let params = ReserveParams {
        agent: agent.to_string(),
        cell: cell.to_string(),
        path: path.to_string(),
        ttl: None,
        session: None,
        kind: None,
    };
    let t = topo.map(|(m, h)| Topo { main_root: m, holder: h });
    // Every delegate-trigger front-loaded, before the cross-worktree lock.
    reserve_prechecks(t, root_s, &params)?;
    Ok(match reserve_exec(t, root_s, &params, lock::MAX_ATTEMPTS)? {
        Out::Thrown(m) => ReserveOutcome::Thrown(m),
        Out::Emit(Value::Object(m), _, _) => {
            if matches!(m.get("code"), Some(Value::String(c)) if c == "FOREIGN_HOLD") {
                ReserveOutcome::ForeignHold(m)
            } else if m.get("ok") == Some(&Value::Bool(true)) {
                ReserveOutcome::Reserved(m.get("reservation").cloned().unwrap_or(Value::Null))
            } else {
                ReserveOutcome::Conflicts(match m.get("conflicts") {
                    Some(Value::Array(a)) => a.clone(),
                    _ => Vec::new(),
                })
            }
        }
        Out::Emit(..) => return Err(Err2::Ex), // unreachable: always an object
    })
}

fn conflict_out(conflicts: &[Resv]) -> Out {
    let mut lines =
        vec!["Reservation CONFLICT — return [BLOCKED] to the orchestrator:".to_string()];
    for c in conflicts {
        lines.push(format!(
            "- {} holds \"{}\" (cell {})",
            js_disp_opt(c.agent.as_ref()),
            c.path,
            js_disp_opt(c.cell.as_ref())
        ));
    }
    let result = json!({
        "ok": false,
        "conflicts": conflicts.iter().map(resv_to_value).collect::<Vec<_>>(),
    });
    Out::Emit(result, lines.join("\n"), 1)
}

// ─── reservations release ──────────────────────────────────────────────────

fn run_release(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["agent", "cell"]) {
        return None;
    }
    let agent = flags.req_str("agent")?.to_string();
    let cell = flags.truthy_str("cell").map(str::to_string);

    let (ctx, roots) = match prelude_worktree("reservations release", use_json, t0)? {
        PreWt::Go(c, r) => (c, r),
        PreWt::Emitted(code) => return Some(code),
    };
    let topology = roots.hold_topology();
    let topo = topology.as_ref().map(|(m, h)| Topo { main_root: m, holder: h });
    let root_s = ctx.root.to_str()?.to_string();

    // Pre-checks: every store this verb will read or mutate.
    let precheck = (|| -> Ex<()> {
        let now = now_ms();
        for rec in list_path_lease_records(&root_s)? {
            lease_record_expired(&rec, now)?;
            lease_to_reservation(&rec)?;
        }
        // Without a topology the ledger is never opened (releaseHolds is
        // skipped), so it must not be probed either.
        if let Some(t) = topo {
            read_holds_store(t.main_root)?;
        }
        Ok(())
    })();
    if precheck.is_err() {
        return None;
    }

    let out = release_exec(topo, &root_s, &agent, cell.as_deref(), lock::MAX_ATTEMPTS);
    finish(&ctx, out)
}

/// releaseReservationsForAgent — pub(crate) since the `dispatch prepare
/// --claim` port, whose conflict unwind releases exactly the reservations the
/// same call had just taken (`agent` = the worker, `cell` = the claimed cell).
pub(crate) fn release_reservations_for_agent(
    topo: Option<(&Path, &str)>,
    root_s: &str,
    agent: &str,
    cell: Option<&str>,
) -> R2<Out> {
    let t = topo.map(|(m, h)| Topo { main_root: m, holder: h });
    release_exec(t, root_s, agent, cell, lock::MAX_ATTEMPTS)
}

fn release_exec(
    topo: Option<Topo>,
    root_s: &str,
    agent: &str,
    cell: Option<&str>,
    max_attempts: u32,
) -> R2<Out> {
    // releaseReservationsForAgent: matched rows FIRST (before release marks
    // them), to derive the ledger's {cell, session} scoping pairs.
    let matched: Vec<Resv> = list_reservations(root_s, true, now_ms())?
        .into_iter()
        .filter(|r| {
            let agent_match = matches!(&r.agent, Some(Value::String(s)) if s == agent);
            let cell_match = match cell {
                None => true,
                Some(c) => {
                    matches!(&r.cell, Some(v) if js_strict_eq(v, &Value::String(c.to_string())))
                }
            };
            agent_match && cell_match
        })
        .collect();
    let mut pairs: Vec<(Value, Option<Value>)> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for r in &matched {
        let Some(cell_v) = r.cell.as_ref().filter(|c| truthy(c)) else {
            continue;
        };
        let session_v = r.session.as_ref().filter(|s| truthy(s)).cloned();
        let key = format!(
            "{}::{}",
            js_disp(cell_v),
            session_v.as_ref().map(js_disp).unwrap_or_default()
        );
        if !seen.contains(&key) {
            seen.push(key);
            pairs.push((cell_v.clone(), session_v));
        }
    }

    // release() (lib/reservations.mjs).
    if js_trim(agent).is_empty() {
        return Ok(Out::Thrown("release: agent is required.".into()));
    }
    let control_root = control_root_for(root_s)?;
    let trimmed_agent = js_trim(agent);
    let mut released: u64 = 0;
    for rec in list_path_lease_records(root_s)? {
        let lease_agent = match rec.get("workspace_id") {
            Some(Value::String(s)) if s.starts_with("agent:") => {
                Value::String(s["agent:".len()..].to_string())
            }
            Some(other) => other.clone(),
            None => continue, // undefined !== an agent string — never a match
        };
        if !v_is_str(&lease_agent, trimmed_agent) {
            continue;
        }
        if let Some(c) = cell {
            let matches_cell =
                matches!(rec.get("workflow_id"), Some(v) if js_strict_eq(v, &Value::String(c.to_string())));
            if !matches_cell {
                continue;
            }
        }
        let resource = match rec.get("resource") {
            Some(Value::String(s)) => s.clone(),
            _ => continue,
        };
        let file = path_lease_file(&control_root, &resource["path:".len()..]);
        match std::fs::remove_file(&file) {
            Ok(()) => released += 1,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {} // released: false
            Err(_) => return Err(Err2::Ex),
        }
    }

    // xwh-2/gfb-1: clear THIS checkout's mirrored ledger entries, one locked
    // releaseHolds per {cell, session} pair — the same topology gate the
    // reserve side uses, so `if (topology)` skipping it entirely inside an
    // ungranted worktree leaves holds_released at 0 and takes no lock.
    let mut holds_released: u64 = 0;
    for (cell_v, session_v) in &pairs {
        let Some(t) = topo else { break };
        let guard =
            match lock::acquire_store_lock(t.main_root, CROSS_WORKTREE_HOLDS_LOCK, max_attempts) {
                Ok(g) => g,
                Err(busy) => return Err(Err2::Msg(busy.message())),
            };
        let mut store = read_holds_store(t.main_root)?;
        let released_at = now_iso();
        let mut count: u64 = 0;
        if let Some(Value::Array(holds)) = store.get_mut("holds") {
            for hold in holds.iter_mut() {
                let unreleased = matches!(jget(hold, "released_at"), None | Some(Value::Null));
                if !unreleased {
                    continue;
                }
                if !matches!(jget(hold, "holder"), Some(Value::String(s)) if s == t.holder) {
                    continue;
                }
                if let Some(s) = session_v {
                    let sess_match = matches!(jget(hold, "session"), Some(v) if js_strict_eq(v, s));
                    if !sess_match {
                        continue;
                    }
                }
                let cell_match = matches!(jget(hold, "cell"), Some(v) if js_strict_eq(v, cell_v));
                if !cell_match {
                    continue;
                }
                if let Value::Object(m) = hold {
                    m.insert("released_at".into(), Value::String(released_at.clone()));
                }
                count += 1;
            }
        }
        if count > 0 {
            write_json_atomic(&holds_ledger_path(t.main_root), &store).map_err(|_| Err2::Ex)?;
        }
        holds_released += count;
        drop(guard);
    }

    let result = json!({
        "released": released as f64,
        "holds_released": holds_released as f64,
    });
    let text = format!(
        "Released {released} reservation(s){}.",
        if holds_released > 0 {
            format!(" and {holds_released} cross-worktree hold(s)")
        } else {
            String::new()
        }
    );
    Ok(Out::Emit(result, text, 0))
}

// ─── reservations sweep ────────────────────────────────────────────────────

fn run_sweep(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let (ctx, roots) = match prelude_worktree("reservations sweep", use_json, t0)? {
        PreWt::Go(c, r) => (c, r),
        PreWt::Emitted(code) => return Some(code),
    };
    // sweep uses resolveMainRoot, NOT resolveHoldTopology: sweepExpiredHolds
    // resolves its own empty/missing ledger, so bee.mjs calls it
    // unconditionally — even from an ungranted worktree, which prunes MAIN's
    // ledger exactly as running it from main would.
    let ledger_root = roots.main_root();
    let root_s = ctx.root.to_str()?.to_string();

    let precheck = (|| -> Ex<()> {
        let now = now_ms();
        for rec in list_path_lease_records(&root_s)? {
            lease_record_expired(&rec, now)?;
        }
        let store = read_holds_store(&ledger_root)?;
        for hold in holds_of(&store) {
            hold_expired(hold, now)?;
        }
        Ok(())
    })();
    if precheck.is_err() {
        return None;
    }

    let out = sweep_exec(&ledger_root, &root_s, lock::MAX_ATTEMPTS);
    finish(&ctx, out)
}

/// `root` here is resolveMainRoot(root) — where the shared holds ledger and
/// its lock live, which is NOT the store root inside a linked worktree.
fn sweep_exec(root: &Path, root_s: &str, max_attempts: u32) -> R2<Out> {
    // sweepExpired (lib/reservations.mjs): per-record, lock-free.
    let control_root = control_root_for(root_s)?;
    let now = now_ms();
    let mut released: u64 = 0;
    for rec in list_path_lease_records(root_s)? {
        if !lease_record_expired(&rec, now)? {
            continue;
        }
        let resource = match rec.get("resource") {
            Some(Value::String(s)) => s.clone(),
            _ => continue,
        };
        let file = path_lease_file(&control_root, &resource["path:".len()..]);
        match std::fs::remove_file(&file) {
            Ok(()) => released += 1,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(Err2::Ex),
        }
    }

    // sweepExpiredHolds (worktree-holds.mjs): whole-ledger, locked.
    let guard = match lock::acquire_store_lock(root, CROSS_WORKTREE_HOLDS_LOCK, max_attempts) {
        Ok(g) => g,
        Err(busy) => return Err(Err2::Msg(busy.message())),
    };
    let mut store = read_holds_store(root)?;
    let now2 = now_ms();
    let released_at = now_iso();
    let mut holds_released: u64 = 0;
    if let Some(Value::Array(holds)) = store.get_mut("holds") {
        for hold in holds.iter_mut() {
            let unreleased = matches!(jget(hold, "released_at"), None | Some(Value::Null));
            if !unreleased {
                continue;
            }
            if !hold_expired(hold, now2)? {
                continue;
            }
            if let Value::Object(m) = hold {
                m.insert("released_at".into(), Value::String(released_at.clone()));
            }
            holds_released += 1;
        }
    }
    if holds_released > 0 {
        write_json_atomic(&holds_ledger_path(root), &store).map_err(|_| Err2::Ex)?;
    }
    drop(guard);

    let result = json!({
        "released": released as f64,
        "holds_released": holds_released as f64,
    });
    let text = format!(
        "Swept {released} expired reservation(s) and {holds_released} expired cross-worktree hold(s)."
    );
    Ok(Out::Emit(result, text, 0))
}

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_root() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(tmp.path().join(".bee").join("onboarding.json"), "{}\n").unwrap();
        tmp
    }

    fn params(agent: &str, cell: &str, path: &str) -> ReserveParams {
        ReserveParams {
            agent: agent.into(),
            cell: cell.into(),
            path: path.into(),
            ttl: None,
            session: None,
            kind: None,
        }
    }

    fn root_str(p: &Path) -> String {
        p.to_str().unwrap().to_string()
    }

    /// resolveHoldTopology's answer for an ORDINARY checkout — what every
    /// pre-existing fixture below has always exercised.
    fn main_topo(p: &Path) -> Option<Topo<'_>> {
        Some(Topo { main_root: p, holder: "main" })
    }

    /// Session-resolution tests assume no ambient session identity —
    /// edition-2024 env mutation is unsafe under threaded tests, so skip
    /// instead of scrubbing when the harness itself exports one.
    fn ambient_session() -> bool {
        env_nonempty("BEE_SESSION_ID").is_some() || env_nonempty("CLAUDE_CODE_SESSION_ID").is_some()
    }

    #[test]
    fn paths_overlap_vectors() {
        assert!(paths_overlap("src/api", "src/api"));
        assert!(paths_overlap("src/api", "src/api/router.ts"));
        assert!(paths_overlap("src/api/*", "src/api/router.ts"));
        assert!(paths_overlap("*", "anything"));
        assert!(!paths_overlap("", "src"));
        assert!(!paths_overlap("src/api", "src/apix"));
        assert!(paths_overlap("./src\\api/", "src/api"));
    }

    #[test]
    fn normalize_path_vectors() {
        assert_eq!(res_normalize_path("./a/b/"), "a/b");
        assert_eq!(res_normalize_path(".//a"), "a");
        assert_eq!(res_normalize_path("a\\b"), "a/b");
        assert_eq!(res_normalize_path("./"), "");
    }

    #[test]
    fn number_flag_grammar() {
        assert!(matches!(js_number_flag("60"), Ok(Some(v)) if v == 60.0));
        assert!(matches!(js_number_flag(" 5.5 "), Ok(Some(v)) if v == 5.0));
        assert!(matches!(js_number_flag("-3"), Ok(Some(v)) if v == -3.0));
        assert!(matches!(js_number_flag(".5"), Ok(None))); // parseInt NaN
        assert!(js_number_flag("0x10").is_err()); // outside modeled grammar
        assert!(js_number_flag("abc").is_err());
        assert!(js_number_flag("").is_err());
    }

    #[test]
    fn date_parse_subset() {
        assert!(matches!(
            js_date_parse("2020-01-02T03:04:05.678Z"),
            Ok(Some(v)) if v == 1577934245678.0
        ));
        let day = js_date_parse("2020-01-02").unwrap_or(None).unwrap();
        let full = js_date_parse("2020-01-02T00:00:00.000Z").unwrap_or(None).unwrap();
        assert_eq!(day, full); // date-only is UTC midnight in JS
        assert!(matches!(js_date_parse(""), Ok(None)));
        assert!(js_date_parse("July 4 2026").is_err()); // V8 legacy grammar — delegate
    }

    #[test]
    fn reserve_writes_node_shaped_lease_and_mirrored_hold() {
        if ambient_session() {
            return;
        }
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        let out = reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "cell-1", "src/x.ts"), 1);
        let Ok(Out::Emit(result, text, code)) = out else {
            panic!("expected an emit outcome");
        };
        assert_eq!(code, 0);
        assert!(text.starts_with("Reserved \"src/x.ts\" for worker-a (cell cell-1, ttl 3600s)."));
        assert_eq!(result["ok"], Value::Bool(true));
        assert_eq!(result["reservation"]["path"], "src/x.ts");
        assert_eq!(result["reservation"]["kind"], "lease");
        // Lease file exists at the sha256 of the resource key, Node-shaped.
        let file = path_lease_file(&root_s, "src/x.ts");
        let content = std::fs::read_to_string(&file).unwrap();
        assert!(content.ends_with("}\n"));
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["resource"], "path:src/x.ts");
        assert_eq!(parsed["mode"], "write");
        assert_eq!(parsed["workspace_id"], "agent:worker-a");
        assert_eq!(parsed["session_id"], SESSIONLESS_SESSION_ID);
        // Mirrored hold landed in the ledger.
        let ledger: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(tmp.path())).unwrap())
                .unwrap();
        assert_eq!(ledger["holds"][0]["holder"], "main");
        assert_eq!(ledger["holds"][0]["path"], "src/x.ts");
        assert_eq!(ledger["holds"][0]["released_at"], Value::Null);
    }

    #[test]
    fn reserve_exact_path_race_reports_conflict() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "cell-1", "src/x.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        let Ok(Out::Emit(result, text, code)) =
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-b", "cell-2", "src/x.ts"), 1)
        else {
            panic!("expected emit");
        };
        assert_eq!(code, 1);
        assert_eq!(result["ok"], Value::Bool(false));
        assert_eq!(result["conflicts"][0]["agent"], "worker-a");
        assert!(text.contains("Reservation CONFLICT"));
        assert!(text.contains("- worker-a holds \"src/x.ts\" (cell cell-1)"));
    }

    #[test]
    fn intent_kind_overlap_is_advisory_but_exact_path_still_hard() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        let mut intent = params("planner", "cell-p", "src/api/*");
        intent.kind = Some("intent".into());
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &intent, 1),
            Ok(Out::Emit(_, _, 0))
        ));
        // Broad overlap against the intent row: allowed.
        assert!(matches!(
            reserve_exec(
                main_topo(tmp.path()),
                &root_s,
                &params("worker-a", "cell-1", "src/api/router.ts"),
                1
            ),
            Ok(Out::Emit(_, _, 0))
        ));
        // Exact same resource: hard regardless of kind.
        let Ok(Out::Emit(result, _, 1)) =
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-b", "cell-2", "src/api/*"), 1)
        else {
            panic!("expected exact-path hard conflict");
        };
        assert_eq!(result["ok"], Value::Bool(false));
    }

    #[test]
    fn reserve_contends_on_the_shared_node_lock_name() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        let _held = lock::acquire_store_lock(tmp.path(), CROSS_WORKTREE_HOLDS_LOCK, 1)
            .ok()
            .unwrap();
        match reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "c", "src/x.ts"), 1) {
            Err(Err2::Msg(msg)) => {
                assert!(msg.contains("lock \"cross-worktree-holds\" busy: held by"), "{msg}");
            }
            _ => panic!("reserve under a held lock must refuse with the LockBusy message"),
        }
        // No lease was written while the lock was held.
        assert!(!path_lease_file(&root_s, "src/x.ts").exists());
    }

    #[test]
    fn invalid_kind_message_matches_node() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        let mut p = params("a", "c", "src/x.ts");
        p.kind = Some("true".into());
        match reserve_exec(main_topo(tmp.path()), &root_s, &p, 1) {
            Ok(Out::Thrown(msg)) => {
                assert_eq!(msg, "reserve: kind must be one of intent/lease (got \"true\").")
            }
            _ => panic!("expected thrown message"),
        }
    }

    #[test]
    fn release_deletes_lease_and_marks_mirrored_hold() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "cell-1", "src/x.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        let Ok(Out::Emit(result, text, 0)) = release_exec(main_topo(tmp.path()), &root_s, "worker-a", None, 1)
        else {
            panic!("expected release emit");
        };
        assert_eq!(result["released"], serde_json::json!(1.0));
        assert_eq!(result["holds_released"], serde_json::json!(1.0));
        assert_eq!(text, "Released 1 reservation(s) and 1 cross-worktree hold(s).");
        assert!(!path_lease_file(&root_s, "src/x.ts").exists());
        let ledger: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(tmp.path())).unwrap())
                .unwrap();
        assert!(ledger["holds"][0]["released_at"].is_string());
    }

    #[test]
    fn release_scoped_to_other_cell_releases_nothing() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "cell-1", "src/x.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        let Ok(Out::Emit(result, text, 0)) =
            release_exec(main_topo(tmp.path()), &root_s, "worker-a", Some("other-cell"), 1)
        else {
            panic!("expected release emit");
        };
        assert_eq!(result["released"], serde_json::json!(0.0));
        assert_eq!(text, "Released 0 reservation(s).");
        assert!(path_lease_file(&root_s, "src/x.ts").exists());
    }

    #[test]
    fn sweep_releases_expired_lease_and_hold() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        // A lease already past its expiry + a matching expired ledger row.
        let file = path_lease_file(&root_s, "src/old.ts");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(
            &file,
            "{\n  \"resource\": \"path:src/old.ts\",\n  \"mode\": \"write\",\n  \"workflow_id\": \"c\",\n  \"session_id\": \"s\",\n  \"workspace_id\": \"agent:a\",\n  \"epoch\": 0,\n  \"acquired_at\": \"2020-01-01T00:00:00.000Z\",\n  \"expires_at\": \"2020-01-01T01:00:00.000Z\",\n  \"kind\": \"lease\"\n}\n",
        )
        .unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee").join("runtime")).unwrap();
        std::fs::write(
            holds_ledger_path(tmp.path()),
            "{\n  \"holds\": [\n    {\n      \"path\": \"src/old.ts\",\n      \"holder\": \"main\",\n      \"feature\": null,\n      \"session\": null,\n      \"cell\": \"c\",\n      \"ttl_seconds\": 60,\n      \"mirrored_at\": \"2020-01-01T00:00:00.000Z\",\n      \"released_at\": null\n    }\n  ]\n}\n",
        )
        .unwrap();
        let Ok(Out::Emit(result, text, 0)) = sweep_exec(tmp.path(), &root_s, 1) else {
            panic!("expected sweep emit");
        };
        assert_eq!(result["released"], serde_json::json!(1.0));
        assert_eq!(result["holds_released"], serde_json::json!(1.0));
        assert_eq!(
            text,
            "Swept 1 expired reservation(s) and 1 expired cross-worktree hold(s)."
        );
        assert!(!file.exists());
    }

    #[test]
    /// CUTOVER (was `corrupt_ledger_or_null_hold_delegates`): a CORRUPT
    /// ledger no longer delegates — it warns and reads as the empty ledger,
    /// `readJson(file, null)`'s own fallback through Node's `!store` guard.
    /// A `null` HOLD still delegates: that one is a JS property-access crash,
    /// not a V8-message matter, and is out of the cutover's scope.
    fn corrupt_ledger_reads_empty_and_a_null_hold_still_delegates() {
        let tmp = fixture_root();
        std::fs::create_dir_all(tmp.path().join(".bee").join("runtime")).unwrap();
        std::fs::write(holds_ledger_path(tmp.path()), "{broken").unwrap();
        assert_eq!(read_holds_store(tmp.path()).ok().unwrap(), json!({"holds": []}));
        std::fs::write(holds_ledger_path(tmp.path()), "{\"holds\": [null]}").unwrap();
        assert!(read_holds_store(tmp.path()).is_err());
        // Shape-less parses read as an empty ledger, like Node's readStore —
        // which is exactly what the corrupt file now reads as, too.
        std::fs::write(holds_ledger_path(tmp.path()), "[1,2]").unwrap();
        let store = read_holds_store(tmp.path()).ok().unwrap();
        assert_eq!(store, json!({"holds": []}));
    }

    #[test]
    fn session_required_when_others_live_and_caller_unidentified() {
        if ambient_session() {
            return;
        }
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        let sessions = tmp.path().join(".bee").join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let hb = now_iso();
        // TWO live sessions: adoption (exactly-one rule) cannot fire, and
        // concurrent mode is on → typed SESSION_REQUIRED refusal.
        for id in ["other", "second"] {
            std::fs::write(
                sessions.join(format!("{id}.json")),
                format!(
                    "{{\n  \"id\": \"{id}\",\n  \"started_at\": \"{hb}\",\n  \"last_heartbeat\": \"{hb}\"\n}}\n"
                ),
            )
            .unwrap();
        }
        let Ok(Out::Emit(result, _, 1)) =
            reserve_exec(main_topo(tmp.path()), &root_s, &params("a", "c", "src/x.ts"), 1)
        else {
            panic!("expected SESSION_REQUIRED refusal");
        };
        assert_eq!(result["code"], "SESSION_REQUIRED");
        assert_eq!(result["conflicts"], json!([]));
    }

    #[test]
    fn single_live_session_is_adopted_and_stamped() {
        if ambient_session() {
            return;
        }
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        let sessions = tmp.path().join(".bee").join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let hb = now_iso();
        std::fs::write(
            sessions.join("solo.json"),
            format!(
                "{{\n  \"id\": \"solo\",\n  \"started_at\": \"{hb}\",\n  \"last_heartbeat\": \"{hb}\"\n}}\n"
            ),
        )
        .unwrap();
        let Ok(Out::Emit(result, _, 0)) =
            reserve_exec(main_topo(tmp.path()), &root_s, &params("a", "c", "src/x.ts"), 1)
        else {
            panic!("expected adopted-session reserve");
        };
        assert_eq!(result["reservation"]["session"], "solo");
    }

    #[test]
    fn parse_flags_mirrors_node() {
        let (flags, json) = parse_flags(&["--agent", "a", "--json", "--cell=c1"]).unwrap();
        assert!(json);
        assert_eq!(flags.req_str("agent"), Some("a"));
        assert_eq!(flags.req_str("cell"), Some("c1"));
        // Value flags consume the next token even when it looks like a flag.
        let (flags, json) = parse_flags(&["--agent", "--json"]).unwrap();
        assert!(!json);
        assert_eq!(flags.req_str("agent"), Some("--json"));
        // Trailing value-less flag → Node's own error → delegate.
        assert!(parse_flags(&["--agent"]).is_none());
        // Non-flag token → delegate.
        assert!(parse_flags(&["stray"]).is_none());
    }

    #[test]
    fn uuid_shape() {
        let a = pseudo_uuid_v4();
        let b = pseudo_uuid_v4();
        assert_ne!(a, b);
        assert_eq!(a.len(), 36);
        assert_eq!(&a[14..15], "4");
        assert!(matches!(&a[19..20], "8" | "9" | "a" | "b"));
    }

    // ── hold topology over REAL `git worktree add` fixtures ────────────────
    //
    // Pinned against Node on the same fixture shape first (twin-fixture
    // byte-diff of reserve/list/release/sweep from each checkout with
    // BEE_JS_ENTRY sabotaged, plus a diff of the resulting `.bee` trees).

    fn git(cwd: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git must be on PATH for the worktree fixtures");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// main + `wt-granted` (registered) + `wt-ungranted` (not).
    fn worktree_fixture(tmp: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let main = tmp.join("main");
        std::fs::create_dir_all(main.join(".bee")).unwrap();
        std::fs::write(main.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        std::fs::write(main.join("f.txt"), "x").unwrap();
        git(&main, &["init", "-q", "-b", "main", "."]);
        git(&main, &["config", "user.email", "a@b.c"]);
        git(&main, &["config", "user.name", "t"]);
        git(&main, &["add", "-A"]);
        git(&main, &["commit", "-qm", "init"]);
        let granted = tmp.join("wt-granted");
        let ungranted = tmp.join("wt-ungranted");
        git(&main, &["worktree", "add", "-q", granted.to_str().unwrap(), "-b", "wt/g"]);
        git(&main, &["worktree", "add", "-q", ungranted.to_str().unwrap(), "-b", "wt/u"]);
        std::fs::create_dir_all(main.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            main.join(".bee").join("runtime").join("worktree-grants.json"),
            "{\"wt-granted\": true}\n",
        )
        .unwrap();
        std::fs::create_dir_all(granted.join(".bee")).unwrap();
        std::fs::write(granted.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        (main, granted, ungranted)
    }

    fn roots_at(cwd: &Path) -> StoreRoots {
        match resolve_store_root_worktree(cwd) {
            RootsWt::Go(r) => r,
            _ => panic!("expected a resolvable root at {}", cwd.display()),
        }
    }

    fn nrm(p: &Path) -> String {
        p.to_string_lossy().replace('/', "\\")
    }

    /// bee.mjs resolveMainRoot + resolveHoldTopology, all three topologies.
    #[test]
    fn hold_topology_matches_node_for_every_checkout_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, ungranted) = worktree_fixture(tmp.path());

        // ORDINARY: {mainRoot: workRoot, holder: 'main'}.
        let r = roots_at(&main);
        assert_eq!(nrm(&r.main_root()), nrm(&main));
        let (m, h) = r.hold_topology().expect("ordinary always has a topology");
        assert_eq!(nrm(&m), nrm(&main));
        assert_eq!(h, "main");

        // GRANTED worktree: ledger at MAIN, holder = the git-verified id.
        let r = roots_at(&granted);
        assert_eq!(nrm(&r.root), nrm(&granted)); // its OWN store
        assert_eq!(nrm(&r.main_root()), nrm(&main));
        let (m, h) = r.hold_topology().expect("a granted worktree holds");
        assert_eq!(nrm(&m), nrm(&main));
        assert_eq!(h, "wt-granted");

        // UNGRANTED worktree: root already IS main's store, and the whole
        // cross-worktree wiring is SKIPPED (topology === null).
        let r = roots_at(&ungranted);
        assert_eq!(nrm(&r.root), nrm(&main));
        assert_eq!(nrm(&r.main_root()), nrm(&main)); // sweep still prunes main
        assert!(r.hold_topology().is_none());
    }

    /// A reserve from inside a GRANTED worktree mirrors into MAIN's ledger
    /// under the worktree's own id — and the reciprocal reserve from main is
    /// then refused with FOREIGN_HOLD.
    #[test]
    fn granted_worktree_mirrors_under_its_id_and_blocks_main() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, _ungranted) = worktree_fixture(tmp.path());
        let g = roots_at(&granted);
        let (gm, gh) = g.hold_topology().unwrap();
        let g_topo = Some(Topo { main_root: &gm, holder: &gh });
        let g_root_s = root_str(&g.root);

        assert!(matches!(
            reserve_exec(g_topo, &g_root_s, &params("wt-agent", "w1", "src/shared.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        // The mirror row lives in MAIN's ledger, holder = the worktree id.
        let ledger: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(&main)).unwrap())
                .unwrap();
        assert_eq!(ledger["holds"].as_array().unwrap().len(), 1);
        assert_eq!(ledger["holds"][0]["holder"], "wt-granted");
        assert_eq!(ledger["holds"][0]["path"], "src/shared.ts");
        // The worktree's OWN store never gets a ledger.
        assert!(!holds_ledger_path(&granted).exists());

        // MAIN now hits the foreign hold on the same path.
        let m = roots_at(&main);
        let (mm, mh) = m.hold_topology().unwrap();
        let m_topo = Some(Topo { main_root: &mm, holder: &mh });
        let m_root_s = root_str(&m.root);
        let Ok(Out::Emit(result, text, 1)) =
            reserve_exec(m_topo, &m_root_s, &params("main-agent", "c1", "src/shared.ts"), 1)
        else {
            panic!("expected a FOREIGN_HOLD refusal from main");
        };
        assert_eq!(result["code"], "FOREIGN_HOLD");
        assert_eq!(result["holder"], "wt-granted");
        assert!(text.starts_with("bee cross-worktree hold: \"src/shared.ts\" is held by checkout \"wt-granted\""));

        // Releasing from the worktree clears exactly its own mirrored rows.
        let Ok(Out::Emit(result, _, 0)) = release_exec(g_topo, &g_root_s, "wt-agent", None, 1)
        else {
            panic!("expected a clean release");
        };
        assert_eq!(result["holds_released"], 1.0);
    }

    /// An UNGRANTED worktree has NO topology: reserve takes no cross-worktree
    /// lock, writes no mirror row, and release reports zero holds — while the
    /// LEASE itself still lands in main's shared store (controlRootFor).
    #[test]
    fn ungranted_worktree_skips_the_cross_worktree_section_entirely() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, _granted, ungranted) = worktree_fixture(tmp.path());
        let r = roots_at(&ungranted);
        assert!(r.hold_topology().is_none());
        let root_s = root_str(&r.root);

        assert!(matches!(
            reserve_exec(None, &root_s, &params("u-agent", "c1", "src/only.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        // No ledger written anywhere — not in main, not in the worktree.
        assert!(!holds_ledger_path(&main).exists());
        assert!(!holds_ledger_path(&ungranted).exists());
        // But the lease DID land, in main's control root (shared store).
        let Ok(rows) = list_reservations(&root_s, true, now_ms()) else {
            panic!("listable");
        };
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].path, "src/only.ts");
        let Ok(ctrl) = control_root_for(&root_s) else { panic!("control root") };
        assert_eq!(nrm(Path::new(&ctrl)), nrm(&main));

        let Ok(Out::Emit(result, text, 0)) = release_exec(None, &root_s, "u-agent", None, 1) else {
            panic!("expected a clean release");
        };
        assert_eq!(result["released"], 1.0);
        assert_eq!(result["holds_released"], 0.0);
        assert_eq!(text, "Released 1 reservation(s).");
    }

    /// sweep uses resolveMainRoot, NOT the topology: even from an ungranted
    /// worktree it prunes MAIN's ledger, exactly as running it from main does.
    #[test]
    fn sweep_prunes_mains_ledger_from_an_ungranted_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, _granted, ungranted) = worktree_fixture(tmp.path());
        std::fs::create_dir_all(holds_ledger_path(&main).parent().unwrap()).unwrap();
        std::fs::write(
            holds_ledger_path(&main),
            r#"{"holds":[{"path":"src/old.ts","holder":"someone-else","feature":null,"session":null,"cell":"c","ttl_seconds":60,"mirrored_at":"2020-01-01T00:00:00.000Z","released_at":null}]}"#,
        )
        .unwrap();
        let r = roots_at(&ungranted);
        let Ok(Out::Emit(result, text, 0)) =
            sweep_exec(&r.main_root(), &root_str(&r.root), 1)
        else {
            panic!("expected a clean sweep");
        };
        assert_eq!(result["holds_released"], 1.0);
        assert_eq!(
            text,
            "Swept 0 expired reservation(s) and 1 expired cross-worktree hold(s)."
        );
        let ledger: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(&main)).unwrap())
                .unwrap();
        assert!(ledger["holds"][0]["released_at"].is_string());
    }

    // ── R5 test migration from packages/bee/tests/test_lease_store.mjs ─────
    //
    // WHAT IS TESTED HERE, AND WHERE THE REST LIVES — the lease store THIS
    // file ports is the SINGLE-resource path-lease half that reservations.mjs
    // drives (acquire one lease under O_EXCL, release by agent/cell, sweep by
    // expiry), so the rows below are the single-resource echoes.
    //
    // The rest of lease-store.mjs — the multi-resource batch (hash-sorted
    // acquire order, partial rollback, typed LEASE_HELD /
    // LEASE_INVALID_REQUEST), renewLease / renewLeasesBySession with the
    // LEASE_MISSING refusal, and LEASE_FENCE_STALE on BOTH renew and release
    // including the "file is never removed on a fenced refusal" property — now
    // lives in `src/lease_store.rs` with its own tests. `reserve_locked` here
    // is deliberately NOT rewired through it: this arm is byte-diffed against
    // Node through four live verbs, and rewiring it would be a behavior risk
    // with no behavior gain (see that module's header).

    /// Reads the whole path-lease directory as a set of file names, so a test
    /// can assert nothing was added or left behind.
    fn lease_files(root_s: &str) -> Vec<String> {
        let dir = Path::new(root_s)
            .join(".bee")
            .join("runtime")
            .join("leases")
            .join("paths");
        let mut names: Vec<String> = match std::fs::read_dir(&dir) {
            Ok(rd) => rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect(),
            Err(_) => Vec::new(),
        };
        names.sort();
        names
    }

    fn write_lease_file(root_s: &str, path: &str, expires_at: Value) {
        let file = path_lease_file(root_s, path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        let record = json!({
            "resource": format!("path:{}", res_normalize_path(path)),
            "mode": "write",
            "workflow_id": "c",
            "session_id": "s",
            "workspace_id": "agent:a",
            "epoch": 0,
            "acquired_at": "2020-01-01T00:00:00.000Z",
            "expires_at": expires_at,
            "kind": "lease",
        });
        std::fs::write(&file, format!("{}\n", jsjson::stringify_pretty(&record))).unwrap();
    }

    fn write_holds_ledger(root: &Path, holds: Value) {
        std::fs::create_dir_all(holds_ledger_path(root).parent().unwrap()).unwrap();
        std::fs::write(holds_ledger_path(root), jsjson::stringify(&json!({"holds": holds}))).unwrap();
    }

    /// Oracle: "sweepExpiredLeases deletes only expired leases; never-expiring
    /// (ttl<=0) leases are never swept (TTL semantics)".
    ///
    /// computeExpiresAt stores `expires_at: null` for a non-positive ttl
    /// (reserve_locked's own non-positive-ttl branch), and both the sweep and
    /// the active-only listing must read that as "never expires".
    #[test]
    fn never_expiring_leases_and_holds_survive_a_sweep_that_takes_the_expired_ones() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        write_lease_file(&root_s, "src/expired.ts", json!("2020-01-01T01:00:00.000Z"));
        write_lease_file(&root_s, "src/fresh.ts", json!("2999-01-01T00:00:00.000Z"));
        write_lease_file(&root_s, "src/forever.ts", Value::Null);
        assert_eq!(lease_files(&root_s).len(), 3);

        // ttl_seconds 0 and -5 are both "never expires"; 60s from 2020 is long
        // expired — the control that the sweep is doing anything at all.
        write_holds_ledger(
            tmp.path(),
            json!([
                {"path": "src/expired.ts", "holder": "main", "feature": null, "session": null,
                 "cell": "c", "ttl_seconds": 60, "mirrored_at": "2020-01-01T00:00:00.000Z", "released_at": null},
                {"path": "src/forever.ts", "holder": "main", "feature": null, "session": null,
                 "cell": "c", "ttl_seconds": 0, "mirrored_at": "2020-01-01T00:00:00.000Z", "released_at": null},
                {"path": "src/forever-neg.ts", "holder": "main", "feature": null, "session": null,
                 "cell": "c", "ttl_seconds": -5, "mirrored_at": "2020-01-01T00:00:00.000Z", "released_at": null},
            ]),
        );

        let Ok(Out::Emit(result, text, 0)) = sweep_exec(tmp.path(), &root_s, 1) else {
            panic!("expected sweep emit");
        };
        assert_eq!(result["released"], json!(1.0), "exactly the expired lease");
        assert_eq!(result["holds_released"], json!(1.0), "exactly the expired hold");
        assert_eq!(
            text,
            "Swept 1 expired reservation(s) and 1 expired cross-worktree hold(s)."
        );
        assert!(!path_lease_file(&root_s, "src/expired.ts").exists());
        assert!(path_lease_file(&root_s, "src/fresh.ts").exists(), "an unexpired lease survives");
        assert!(
            path_lease_file(&root_s, "src/forever.ts").exists(),
            "a never-expiring (expires_at null) lease survives"
        );
        let ledger: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(tmp.path())).unwrap()).unwrap();
        assert!(ledger["holds"][0]["released_at"].is_string(), "the expired hold was marked");
        assert_eq!(ledger["holds"][1]["released_at"], Value::Null, "ttl 0 never expires");
        assert_eq!(ledger["holds"][2]["released_at"], Value::Null, "a negative ttl never expires");

        // The active-only listing agrees, and reports the never-expiring lease
        // with reservations.mjs's ttl_seconds sentinel of 0.
        let mut rows = list_reservations(&root_s, true, now_ms()).ok().unwrap();
        rows.sort_by(|a, b| a.path.cmp(&b.path));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].path, "src/forever.ts");
        assert_eq!(rows[0].ttl_seconds, Some(0.0));
        assert_eq!(rows[1].path, "src/fresh.ts");

        // A second sweep is a no-op: nothing left is expired.
        let Ok(Out::Emit(result, _, 0)) = sweep_exec(tmp.path(), &root_s, 1) else {
            panic!("expected sweep emit");
        };
        assert_eq!(result["released"], json!(0.0));
        assert_eq!(result["holds_released"], json!(0.0));
    }

    /// The single-resource echo of the oracle's "partial acquire rolls back
    /// fully … zero residue": a reserve that loses its resource must leave the
    /// lease directory and the mirrored ledger exactly as it found them.
    #[test]
    fn a_lost_reserve_leaves_zero_residue_in_the_lease_dir_and_the_ledger() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "cell-1", "src/x.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        let files_before = lease_files(&root_s);
        // Semantic snapshot: the ledger must be unchanged as a RECORD SET, not
        // merely as bytes a rewrite happened to reproduce.
        let ledger_before: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(tmp.path())).unwrap()).unwrap();
        assert_eq!(files_before.len(), 1);
        assert_eq!(ledger_before["holds"].as_array().unwrap().len(), 1);

        // Overlap conflict on the exact same resource.
        let Ok(Out::Emit(result, _, 1)) =
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-b", "cell-2", "src/x.ts"), 1)
        else {
            panic!("expected the conflict refusal");
        };
        assert_eq!(result["ok"], Value::Bool(false));
        // Overlap conflict on a path that only OVERLAPS an existing lease.
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-b", "cell-2", "src/x.ts/inner"), 1),
            Ok(Out::Emit(_, _, 1))
        ));
        assert_eq!(lease_files(&root_s), files_before, "no lease file survived a refusal");
        let ledger_after: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(tmp.path())).unwrap()).unwrap();
        assert_eq!(ledger_after, ledger_before, "no hold row survived a refusal");

        // Control: a non-overlapping resource DOES add exactly one of each, so
        // the zero-residue assertions above are not vacuous.
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-b", "cell-2", "src/y.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        assert_eq!(lease_files(&root_s).len(), 2);
        let ledger_final: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(tmp.path())).unwrap()).unwrap();
        assert_eq!(ledger_final["holds"].as_array().unwrap().len(), 2);
    }

    /// The single-resource echo of the oracle's LEASE_INVALID_REQUEST row
    /// ("missing fields, bad type … refuses before any file is created") and
    /// of "kind defaults to lease when omitted; an explicit intent is stamped
    /// verbatim; an invalid kind refuses before any file is created".
    #[test]
    fn a_malformed_reserve_request_is_refused_before_any_lease_file_is_created() {
        let tmp = fixture_root();
        let root_s = root_str(tmp.path());

        // Refusal wording is a pinned contract on this verb (bee.mjs serves
        // reserve()'s throws byte-identical) — hence the exact comparisons.
        let cases: [(ReserveParams, &str); 4] = [
            (params("  ", "cell-1", "src/x.ts"), "reserve: agent is required."),
            (params("worker-a", "", "src/x.ts"), "reserve: cell id is required."),
            (params("worker-a", "cell-1", "   "), "reserve: path is required."),
            (
                {
                    let mut p = params("worker-a", "cell-1", "src/x.ts");
                    p.kind = Some("exclusive".into());
                    p
                },
                "reserve: kind must be one of intent/lease (got \"exclusive\").",
            ),
        ];
        for (p, expected) in cases {
            match reserve_exec(main_topo(tmp.path()), &root_s, &p, 1) {
                Ok(Out::Thrown(msg)) => assert_eq!(msg, expected),
                other => panic!(
                    "expected a thrown refusal for {expected:?}, got {}",
                    match other {
                        Ok(Out::Emit(v, _, _)) => jsjson::stringify(&v),
                        Ok(Out::Thrown(m)) => m,
                        Err(_) => "an error".to_string(),
                    }
                ),
            }
            assert!(lease_files(&root_s).is_empty(), "a refused request created a lease file");
        }
        assert!(
            !holds_ledger_path(tmp.path()).exists(),
            "a refused request never writes the mirrored ledger"
        );

        // Control: the same request with every field valid succeeds, and the
        // stored record carries the DEFAULTED kind.
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &params("worker-a", "cell-1", "src/x.ts"), 1),
            Ok(Out::Emit(_, _, 0))
        ));
        let stored: Value = serde_json::from_str(
            &std::fs::read_to_string(path_lease_file(&root_s, "src/x.ts")).unwrap(),
        )
        .unwrap();
        assert_eq!(stored["kind"], json!("lease"), "an omitted kind defaults to lease");
        // …and an explicit kind is stamped verbatim.
        let mut intent = params("planner", "cell-p", "src/api/*");
        intent.kind = Some("intent".into());
        assert!(matches!(
            reserve_exec(main_topo(tmp.path()), &root_s, &intent, 1),
            Ok(Out::Emit(_, _, 0))
        ));
        let stored: Value = serde_json::from_str(
            &std::fs::read_to_string(path_lease_file(&root_s, "src/api/*")).unwrap(),
        )
        .unwrap();
        assert_eq!(stored["kind"], json!("intent"));
    }

    // ── CUTOVER: corrupt-JSON reads that used to delegate ─────────────────

    /// worktree-holds.mjs readStore: a corrupt ledger reads as the EMPTY
    /// ledger, which is `readJson(file, null)`'s own fallback through Node's
    /// `!store` guard — same value, one warning of explanation.
    #[test]
    fn a_corrupt_holds_ledger_reads_as_an_empty_ledger() {
        let tmp = fixture_root();
        let ledger = holds_ledger_path(tmp.path());
        std::fs::create_dir_all(ledger.parent().unwrap()).unwrap();
        std::fs::write(&ledger, "{broken").unwrap();
        assert_eq!(read_holds_store(tmp.path()).ok().unwrap(), json!({"holds": []}));
        // A missing file answers the same thing, which is the point.
        std::fs::remove_file(&ledger).unwrap();
        assert_eq!(read_holds_store(tmp.path()).ok().unwrap(), json!({"holds": []}));
    }

    /// claims.mjs listSessionRecords: a corrupt session record is skipped and
    /// the scan continues — the readable siblings still list.
    #[test]
    fn a_corrupt_session_record_is_skipped_not_delegated() {
        let tmp = fixture_root();
        let dir = tmp.path().join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("good.json"), r#"{"id":"good"}"#).unwrap();
        std::fs::write(dir.join("bad.json"), "{broken").unwrap();
        let records = list_session_records(tmp.path().to_str().unwrap()).ok().unwrap();
        assert_eq!(records.len(), 1, "the corrupt record is skipped");
        assert_eq!(records[0].get("id"), Some(&json!("good")));
    }

    /// The `|n| >= 1e21` delegate class is retired: js_numberify accepts every
    /// finite number now that jsjson prints the full ECMA Number::toString.
    #[test]
    fn large_numbers_no_longer_delegate() {
        let v: Value = serde_json::from_str("{\"n\":1e21}").unwrap();
        assert!(js_numberify(&v).is_ok());
    }
}
