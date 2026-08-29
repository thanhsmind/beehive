// JS string/value helpers, the Node `path` port, fs and date helpers
//
// Split out of the single 5.9k-line hooks/write_guard.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::hooks::adapter::{append_hook_log, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson;
use crate::state::hook_enabled;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ─── the retired vendored-lib byte gate ────────────────────────────────────
//
// During the two-runtime window this guard byte-compared its embedded
// implementation (state + guards + validate-args + command-registry, 26
// files) against the host's on-disk vendored copy at runtime: any one-byte
// difference meant the native semantics were unproven, so the run delegated.
// That gate was correct for the whole two-runtime window and is WRONG the
// moment the window closes — there is no vendored copy to compare against,
// so every comparison would fail, every run would delegate, and the
// delegation would fail open. A guard that fails open on every single event
// is not a guard.
//
// Its activation probe went the same way, and that one was the sharper
// hazard: it asked "does the vendored library exist at this store root?" —
// "no" meant "bee is not installed here, decide nothing". Left alone, that
// question would have switched the write guard OFF in every repo on earth.
// The replacement asks the same question against the install shape that
// exists now: `.bee/onboarding.json`, the marker roots.rs already stops its
// walk on.

// ─── JS string / value helpers ─────────────────────────────────────────────

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

pub(crate) fn js_trim_end(s: &str) -> &str {
    s.trim_end_matches(js_is_ws)
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

/// `${value}` template coercion for present values.
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

/// Non-empty trimmed string or None (the `typeof x === "string" && x.trim()`
/// idiom).
pub(crate) fn str_trim_nonempty(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
        _ => None,
    }
}

pub(crate) fn ascii_eq_ci(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

// ─── Node path port (win32 flavor on Windows, posix elsewhere) ─────────────
// Faithful subset of node:path used by the .mjs. Drive-relative ("C:foo") and
// UNC ("\\srv\...") spellings on Windows are Nd — Node's resolution for those
// consults per-drive cwd state this port does not model.

#[cfg(windows)]
pub(crate) const SEP: char = '\\';

#[cfg(not(windows))]
pub(crate) const SEP: char = '/';

pub(crate) fn is_sep(c: char) -> bool {
    if cfg!(windows) { c == '\\' || c == '/' } else { c == '/' }
}

pub(crate) fn has_drive(p: &str) -> bool {
    let b = p.as_bytes();
    b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
}

/// path.isAbsolute
pub(crate) fn np_is_absolute(p: &str) -> bool {
    if cfg!(windows) {
        let mut ch = p.chars();
        match ch.next() {
            Some(c) if is_sep(c) => true,
            Some(_) if has_drive(p) => p.chars().nth(2).map(is_sep).unwrap_or(false),
            _ => false,
        }
    } else {
        p.starts_with('/')
    }
}

/// Windows spellings this port refuses to model (Nd): drive-relative and UNC.
pub(crate) fn np_check_modelable(p: &str) -> R<()> {
    if cfg!(windows) {
        if has_drive(p) && !p.chars().nth(2).map(is_sep).unwrap_or(false) {
            return Err(Nd); // drive-relative "C:foo"
        }
        let mut ch = p.chars();
        if let (Some(a), Some(b)) = (ch.next(), ch.next()) {
            if is_sep(a) && is_sep(b) {
                return Err(Nd); // UNC-ish
            }
        }
    }
    Ok(())
}

/// Node's normalizeString: collapse '.'/'..' segments (allow_above=false —
/// every use here resolves against an absolute base).
pub(crate) fn np_normalize_tail(path: &str) -> String {
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

/// path.resolve over the given args (last wins), with the process cwd as the
/// implicit final fallback — exactly Node's iteration order. All modelable
/// inputs only (drive-relative/UNC → Nd via np_check_modelable).
pub(crate) fn np_resolve(args: &[&str]) -> R<String> {
    let cwd_buf = std::env::current_dir().map_err(|_| Nd)?.to_string_lossy().into_owned();
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
        // (device, index-after-root, is-absolute) — modelable inputs only.
        let (dev, root_end, is_abs): (String, usize, bool) = if cfg!(windows) {
            if has_drive(p) {
                // modelable ⇒ p[2] is a separator ⇒ absolute drive path
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
        if !dev.is_empty() && !device.is_empty() && !ascii_eq_ci(&dev, &device) {
            continue; // a different device than the one already resolved
        }
        if device.is_empty() {
            device = dev;
        }
        if !absolute {
            // byte index is char-safe here: root_end counts ASCII prefix chars
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
        return Err(Nd); // nothing absolute in the chain — cwd was relative?
    }
    if cfg!(windows) && device.is_empty() {
        return Err(Nd); // rooted path with no drive anywhere — unmodelable
    }
    let norm = np_normalize_tail(&tail);
    Ok(format!("{}{}{}", device, SEP, norm))
}

pub(crate) fn np_resolve2(base: &str, p: &str) -> R<String> {
    np_resolve(&[base, p])
}

pub(crate) fn np_resolve1(p: &str) -> R<String> {
    np_resolve(&[p])
}

/// path.relative — Node win32/posix algorithm (case-insensitive comparison on
/// Windows, original-case output).
pub(crate) fn np_relative(from: &str, to: &str) -> R<String> {
    let from_orig = np_resolve1(from)?;
    let to_orig = np_resolve1(to)?;
    if from_orig == to_orig {
        return Ok(String::new());
    }
    let (from_l, to_l) = if cfg!(windows) {
        (from_orig.to_lowercase(), to_orig.to_lowercase())
    } else {
        (from_orig.clone(), to_orig.clone())
    };
    if from_l == to_l {
        return Ok(String::new());
    }
    let fb: Vec<char> = from_l.chars().collect();
    let tb: Vec<char> = to_l.chars().collect();
    let to_chars: Vec<char> = to_orig.chars().collect();
    let sep = SEP;

    let mut from_start = 0usize;
    while from_start < fb.len() && fb[from_start] == sep {
        from_start += 1;
    }
    let mut from_end = fb.len();
    while from_end > from_start + 1 && fb[from_end - 1] == sep {
        from_end -= 1;
    }
    let from_len = from_end - from_start;

    let mut to_start = 0usize;
    while to_start < tb.len() && tb[to_start] == sep {
        to_start += 1;
    }
    let mut to_end = tb.len();
    while to_end > to_start + 1 && tb[to_end - 1] == sep {
        to_end -= 1;
    }
    let to_len = to_end - to_start;

    let length = from_len.min(to_len);
    let mut last_common_sep: i64 = -1;
    let mut i = 0usize;
    while i < length {
        let fc = fb[from_start + i];
        if fc != tb[to_start + i] {
            break;
        } else if fc == sep {
            last_common_sep = i as i64;
        }
        i += 1;
    }
    if i != length {
        if last_common_sep == -1 {
            return Ok(to_orig);
        }
    } else {
        if to_len > length {
            if tb[to_start + length] == sep {
                return Ok(to_chars[to_start + length + 1..to_end].iter().collect());
            }
            if length == 2 {
                return Ok(to_chars[to_start + length..to_end].iter().collect());
            }
        }
        if from_len > length {
            if fb[from_start + length] == sep {
                last_common_sep = length as i64;
            } else if length == 2 {
                last_common_sep = 3;
            }
        }
        if last_common_sep == -1 {
            last_common_sep = 0;
        }
    }
    let mut out = String::new();
    let mut i = from_start as i64 + last_common_sep + 1;
    while i <= from_end as i64 {
        if i == from_end as i64 || fb[i as usize] == sep {
            out.push_str(if out.is_empty() { ".." } else { if cfg!(windows) { "\\.." } else { "/.." } });
        }
        i += 1;
    }
    let mut ts = to_start as i64 + last_common_sep;
    if !out.is_empty() {
        let tail: String = to_chars[ts as usize..to_end].iter().collect();
        return Ok(format!("{}{}", out, tail));
    }
    if (ts as usize) < to_chars.len() && to_chars[ts as usize] == sep {
        ts += 1;
    }
    Ok(to_chars[ts as usize..to_end].iter().collect())
}

/// path.dirname for resolved absolute inputs.
pub(crate) fn np_dirname(p: &str) -> String {
    let chars: Vec<char> = p.chars().collect();
    let root_len = if cfg!(windows) {
        if has_drive(p) { 3 } else if !chars.is_empty() && is_sep(chars[0]) { 1 } else { 0 }
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

/// path.basename for resolved absolute inputs.
pub(crate) fn np_basename(p: &str) -> String {
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

/// `path.resolve(base, ...segments)` where segments are plain basenames.
pub(crate) fn np_resolve_segments(base: &str, segments: &[String]) -> R<String> {
    if segments.is_empty() {
        return np_resolve1(base);
    }
    let joined = format!("{}{}{}", base, SEP, segments.join(&SEP.to_string()));
    np_resolve1(&joined)
}

// ─── fs helpers ────────────────────────────────────────────────────────────

/// fs.realpathSync.native in a catch-ALL try (adapter/bee-write-guard.mjs
/// flavor: any error → None).
pub(crate) fn realpath_any(p: &str) -> Option<String> {
    dunce::canonicalize(Path::new(p)).ok().map(|b| b.to_string_lossy().into_owned())
}

/// guards.mjs realpathOrNull flavor (F2): ENOENT → None, any other error is a
/// JS throw — Nd here.
pub(crate) fn realpath_f2(p: &str) -> R<Option<String>> {
    match dunce::canonicalize(Path::new(p)) {
        Ok(b) => Ok(Some(b.to_string_lossy().into_owned())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(Nd),
    }
}

pub(crate) fn io_err_is_enoent(e: &std::io::Error) -> bool {
    if e.kind() == std::io::ErrorKind::NotFound {
        return true;
    }
    if cfg!(windows) {
        // 2/3 are Windows' own file/path-not-found codes; 123
        // (ERROR_INVALID_NAME) and 161 (ERROR_BAD_PATHNAME) show up when a
        // segment like "*.log" is a literal glob token — POSIX stats that
        // as plain ENOENT, so Windows must count it not-found too, or the
        // ancestor walk stops instead of climbing.
        matches!(e.raw_os_error(), Some(2) | Some(3) | Some(123) | Some(161))
    } else {
        false
    }
}

/// The lexical-target existence walk shared by canonicalRelPath and
/// resolveTargetRealpath: climb through ENOENT segments to the deepest
/// existing ancestor. Ok(None) = "walk failed like Node returning null";
/// Err(Nd) never (non-ENOENT lstat errors return null in the .mjs too).
pub(crate) fn walk_existing_ancestor(lexical: &str) -> Option<(String, Vec<String>)> {
    let mut cursor = lexical.to_string();
    let mut unresolved: Vec<String> = Vec::new();
    loop {
        match std::fs::symlink_metadata(Path::new(&cursor)) {
            Ok(_) => return Some((cursor, unresolved)),
            Err(e) => {
                if !io_err_is_enoent(&e) {
                    return None;
                }
                let parent = np_dirname(&cursor);
                if parent == cursor {
                    return None;
                }
                unresolved.insert(0, np_basename(&cursor));
                cursor = parent;
            }
        }
    }
}

// ─── date helpers (JS Date.parse subset / toISOString) ─────────────────────

/// Date.parse of a JSON value: Ok(None) = NaN. A non-empty string chrono
/// cannot parse (where JS might) is Nd; numbers are Nd (JS parses them as
/// year strings).
pub(crate) fn date_parse_ms(v: Option<&Value>) -> R<Option<f64>> {
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => {
            if s.trim().is_empty() {
                return Ok(None);
            }
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return Ok(Some(dt.timestamp_millis() as f64));
            }
            // date-only form "YYYY-MM-DD" parses as UTC midnight in JS.
            if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                let dt = d.and_hms_opt(0, 0, 0).ok_or(Nd)?.and_utc();
                return Ok(Some(dt.timestamp_millis() as f64));
            }
            Err(Nd)
        }
        Some(_) => Err(Nd),
    }
}

/// new Date(ms).toISOString() — |ms| beyond the JS Date range is Nd (JS
/// throws there, reaching the hook's outer catch).
pub(crate) fn ms_to_iso(ms: f64) -> R<String> {
    if !ms.is_finite() || ms.abs() > 8.64e15 {
        return Err(Nd);
    }
    let dt = chrono::DateTime::from_timestamp_millis(ms as i64).ok_or(Nd)?;
    Ok(dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

/// JS Math.round (half toward +infinity).
pub(crate) fn js_round(v: f64) -> f64 {
    (v + 0.5).floor()
}

// ─── fsutil.mjs readJson (provenance: lib/fsutil.mjs readJson) ─────────────
// CUTOVER (2026-08-01): corrupt JSON used to be Nd, because Node's warn quoted
// V8's parse sentence. It is native now — warn once, then take readJson's
// `null` fallback, which every caller below already treats as "absent":
// readState falls back to defaultState(), readSession/readClaim read as no
// record, the worktree-holds ledger reads as zero holds, and a corrupt lane
// record still takes readLane's own second warn and the LANE_CORRUPT refusal
// (same reason text, same deny, same exit code as before).
//
// The warning is DEFERRED, not printed: this hook buffers every byte and
// flushes only on the Done path, so a run that later returns Nd (the worktree
// arms still do) must not have written to stderr already. read_json_g queues
// the line here; flush() writes the queue ahead of emit.stderr, which puts
// readJson's warn before the deny reason / readLane line it precedes in Node.

thread_local! {
    static CORRUPT_JSON_WARNINGS: std::cell::RefCell<Vec<String>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

pub(crate) fn take_corrupt_json_warnings() -> String {
    CORRUPT_JSON_WARNINGS.with(|q| q.borrow_mut().drain(..).collect::<Vec<_>>().join(""))
}

pub(crate) fn clear_corrupt_json_warnings() {
    CORRUPT_JSON_WARNINGS.with(|q| q.borrow_mut().clear());
}

/// Queue one of bee's OWN warning lines onto the same buffered channel
/// `read_json_g` uses — at most once per evaluation for an identical line.
///
/// sfg-5 added this door. A guard that reads store data it cannot understand
/// now takes the RESTRICTIVE answer (see `heartbeat_stale`), and a
/// restrictive answer can refuse a write for a reason that is nowhere in the
/// refusal — a silent lockout. The reader says what it saw and how to clear
/// it. Same queue, so the delegate contract still holds byte-for-byte:
/// nothing reaches stderr until the verdict is final, and a Delegate carries
/// zero output. The dedupe is what keeps one bad file to one line, however
/// many readers walk past it in one evaluation.
pub(crate) fn queue_guard_warning_once(line: String) {
    CORRUPT_JSON_WARNINGS.with(|q| {
        let mut q = q.borrow_mut();
        if !q.iter().any(|queued| *queued == line) {
            q.push(line);
        }
    });
}

pub(crate) fn read_json_g(file: &Path) -> R<Option<Value>> {
    match crate::fsutil::read_json(file) {
        crate::fsutil::ReadJson::Missing => Ok(None),
        crate::fsutil::ReadJson::Corrupt => {
            // Same sentence crate::fsutil::warn_corrupt_json prints, queued
            // instead of written (see the note above).
            CORRUPT_JSON_WARNINGS.with(|q| {
                q.borrow_mut().push(format!(
                    "bee: could not parse JSON at {} — {}. Using fallback; fix the file.\n",
                    file.display(),
                    corrupt_json_reason(file)
                ))
            });
            Ok(None)
        }
        crate::fsutil::ReadJson::Parsed(v) => Ok(Some(v)),
    }
}

/// Why the file would not parse, in bee's words — the buffered twin of
/// fsutil.rs's private `corrupt_json_reason` (copied because that helper is
/// private to crate::fsutil and this hook cannot print at read time).
pub(crate) fn corrupt_json_reason(file: &Path) -> String {
    let Ok(bytes) = std::fs::read(file) else {
        return "the file could not be read".to_string();
    };
    let text = String::from_utf8_lossy(&bytes);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    match serde_json::from_str::<Value>(text) {
        Ok(_) => "invalid JSON".to_string(), // raced: it parses now
        Err(e) if e.line() > 0 => {
            format!("invalid JSON at line {} column {}", e.line(), e.column())
        }
        Err(_) => "invalid JSON".to_string(),
    }
}
