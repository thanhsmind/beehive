// bee hook write-guard — Rust port of hooks/bee-write-guard.mjs (+ its
// tokenize-command.mjs helper), bee's most safety-critical hook. PreToolUse
// for Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion plus the Codex
// apply_patch path. Four checks in one guard, first hit wins — see the .mjs
// header for the (a)-(d) map; every branch below carries a provenance comment
// naming its .mjs source.
//
// GUARD PORTING CONTRACT (rust-port.md C2/C5): a decision (allow vs deny)
// must NEVER flip relative to the .mjs. Every branch whose Rust equivalence
// cannot be PROVEN returns Outcome::Delegate (the dispatcher re-runs the
// original Node wrapper with identical stdin). Failing toward Node is always
// safe; failing open differently than the .mjs never is.
//
// THE VENDORED-LIB BYTE GATE: the .mjs dynamically imports vendored lib
// modules from <storeRoot>/.bee/bin/lib/*.mjs. This port replicates the
// CURRENT packages/bee/lib implementations; if the vendored files differ in
// any byte (mid-upgrade host, tampered fixture, a test whose guards.mjs
// deliberately throws on import), the native semantics are unproven and the
// whole run delegates. The import closure of state.mjs + guards.mjs +
// validate-args.mjs + command-registry.mjs is embedded at compile time and
// byte-compared at runtime before any native decision is made.
//
// DELEGATED BRANCHES (each justified at its site):
//   - any readJson()-level corrupt JSON on the native path (Node warns to
//     stderr with the V8 parse message — unreplicable bytes);
//   - CLI-shape check (d) when a bee.mjs/bee_*.mjs-shaped token is present
//     and no denial was already computed (registry+validate-args semantics);
//   - node -e/--eval/-p inline-eval commands (internals-reach regex);
//   - companion-mount resolution when .bee/companion-session.json exists and
//     the target already failed containment;
//   - a declared guards.memory_root (non-empty string) when a target failed
//     containment;
//   - drive-relative (C:foo) / UNC (\\srv\...) target spellings on Windows;
//   - JS-throw equivalents inside the shared-nested-checkout primitive
//     (strict-mode session reads, non-ENOENT fs errors) — Node turns those
//     into a typed deny plus a V8-worded crash log line;
//   - timestamp strings chrono cannot parse where JS Date.parse might.
//
// Output is fully buffered: nothing is written before the native/delegate
// decision is final, so Delegate always re-runs Node with zero output emitted.

use crate::hooks::adapter::{append_hook_log, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson;
use crate::state::hook_enabled;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HOOK_NAME: &str = "write-guard";

// ─── strangler bail ────────────────────────────────────────────────────────

/// "Needs Node": the branch's Rust equivalence is unproven — delegate.
#[derive(Debug, Clone, Copy)]
struct Nd;
type R<T> = Result<T, Nd>;

// ─── embedded vendored-lib byte gate ───────────────────────────────────────
// The import closure of state.mjs + guards.mjs + validate-args.mjs +
// command-registry.mjs (computed from the `from './x.mjs'` graph). Each entry
// is byte-compared against <storeRoot>/.bee/bin/lib/<name> at runtime.

const EMBEDDED_LIB: &[(&str, &str)] = &[
    ("backlog.mjs", include_str!("../../../../../bee/lib/backlog.mjs")),
    ("capture.mjs", include_str!("../../../../../bee/lib/capture.mjs")),
    ("cells.mjs", include_str!("../../../../../bee/lib/cells.mjs")),
    ("claims.mjs", include_str!("../../../../../bee/lib/claims.mjs")),
    ("command-registry.mjs", include_str!("../../../../../bee/lib/command-registry.mjs")),
    ("compaction.mjs", include_str!("../../../../../bee/lib/compaction.mjs")),
    ("decisions.mjs", include_str!("../../../../../bee/lib/decisions.mjs")),
    ("dispatch-guard.mjs", include_str!("../../../../../bee/lib/dispatch-guard.mjs")),
    ("fsutil.mjs", include_str!("../../../../../bee/lib/fsutil.mjs")),
    ("guards.mjs", include_str!("../../../../../bee/lib/guards.mjs")),
    ("inject.mjs", include_str!("../../../../../bee/lib/inject.mjs")),
    ("intent.mjs", include_str!("../../../../../bee/lib/intent.mjs")),
    ("judge.mjs", include_str!("../../../../../bee/lib/judge.mjs")),
    ("knowledge.mjs", include_str!("../../../../../bee/lib/knowledge.mjs")),
    ("lease-store.mjs", include_str!("../../../../../bee/lib/lease-store.mjs")),
    ("lock.mjs", include_str!("../../../../../bee/lib/lock.mjs")),
    ("path-identity.mjs", include_str!("../../../../../bee/lib/path-identity.mjs")),
    ("reservations.mjs", include_str!("../../../../../bee/lib/reservations.mjs")),
    ("reviews.mjs", include_str!("../../../../../bee/lib/reviews.mjs")),
    ("schedule.mjs", include_str!("../../../../../bee/lib/schedule.mjs")),
    ("state.mjs", include_str!("../../../../../bee/lib/state.mjs")),
    ("validate-args.mjs", include_str!("../../../../../bee/lib/validate-args.mjs")),
    ("workflow-store.mjs", include_str!("../../../../../bee/lib/workflow-store.mjs")),
    ("workspace-store.mjs", include_str!("../../../../../bee/lib/workspace-store.mjs")),
    ("worktree-holds.mjs", include_str!("../../../../../bee/lib/worktree-holds.mjs")),
    ("worktree-store.mjs", include_str!("../../../../../bee/lib/worktree-store.mjs")),
];

fn lib_byte_gate(store_root: &Path) -> R<()> {
    let lib_dir = store_root.join(".bee").join("bin").join("lib");
    for (name, embedded) in EMBEDDED_LIB {
        match std::fs::read(lib_dir.join(name)) {
            Ok(bytes) if bytes == embedded.as_bytes() => {}
            _ => return Err(Nd),
        }
    }
    Ok(())
}

// ─── JS string / value helpers ─────────────────────────────────────────────

/// JS \s (and String.prototype.trim) whitespace class.
fn js_is_ws(c: char) -> bool {
    matches!(c,
        ' ' | '\t' | '\n' | '\r' | '\u{b}' | '\u{c}' | '\u{a0}' | '\u{1680}'
        | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}'
        | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

fn js_trim(s: &str) -> &str {
    s.trim_matches(js_is_ws)
}

fn js_trim_end(s: &str) -> &str {
    s.trim_end_matches(js_is_ws)
}

/// JS truthiness of a JSON value (undefined handled by callers as None).
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// `${value}` template coercion for present values.
fn js_disp(v: &Value) -> String {
    jsjson::js_to_string(v)
}

/// `${maybe}` where the key may be absent (undefined).
fn js_disp_opt(v: Option<&Value>) -> String {
    match v {
        Some(v) => js_disp(v),
        None => "undefined".to_string(),
    }
}

/// Non-empty trimmed string or None (the `typeof x === "string" && x.trim()`
/// idiom).
fn str_trim_nonempty(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
        _ => None,
    }
}

/// UTF-16 code-unit length (JS String.length).
fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

fn ascii_eq_ci(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

// ─── Node path port (win32 flavor on Windows, posix elsewhere) ─────────────
// Faithful subset of node:path used by the .mjs. Drive-relative ("C:foo") and
// UNC ("\\srv\...") spellings on Windows are Nd — Node's resolution for those
// consults per-drive cwd state this port does not model.

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

/// path.isAbsolute
fn np_is_absolute(p: &str) -> bool {
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
fn np_check_modelable(p: &str) -> R<()> {
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

/// path.resolve over the given args (last wins), with the process cwd as the
/// implicit final fallback — exactly Node's iteration order. All modelable
/// inputs only (drive-relative/UNC → Nd via np_check_modelable).
fn np_resolve(args: &[&str]) -> R<String> {
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

fn np_resolve2(base: &str, p: &str) -> R<String> {
    np_resolve(&[base, p])
}

fn np_resolve1(p: &str) -> R<String> {
    np_resolve(&[p])
}

/// path.relative — Node win32/posix algorithm (case-insensitive comparison on
/// Windows, original-case output).
fn np_relative(from: &str, to: &str) -> R<String> {
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
fn np_dirname(p: &str) -> String {
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

/// `path.resolve(base, ...segments)` where segments are plain basenames.
fn np_resolve_segments(base: &str, segments: &[String]) -> R<String> {
    if segments.is_empty() {
        return np_resolve1(base);
    }
    let joined = format!("{}{}{}", base, SEP, segments.join(&SEP.to_string()));
    np_resolve1(&joined)
}

// ─── fs helpers ────────────────────────────────────────────────────────────

/// fs.realpathSync.native in a catch-ALL try (adapter/bee-write-guard.mjs
/// flavor: any error → None).
fn realpath_any(p: &str) -> Option<String> {
    dunce::canonicalize(Path::new(p)).ok().map(|b| b.to_string_lossy().into_owned())
}

/// guards.mjs realpathOrNull flavor (F2): ENOENT → None, any other error is a
/// JS throw — Nd here.
fn realpath_f2(p: &str) -> R<Option<String>> {
    match dunce::canonicalize(Path::new(p)) {
        Ok(b) => Ok(Some(b.to_string_lossy().into_owned())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(Nd),
    }
}

fn io_err_is_enoent(e: &std::io::Error) -> bool {
    if e.kind() == std::io::ErrorKind::NotFound {
        return true;
    }
    if cfg!(windows) {
        matches!(e.raw_os_error(), Some(2) | Some(3))
    } else {
        false
    }
}

/// The lexical-target existence walk shared by canonicalRelPath and
/// resolveTargetRealpath: climb through ENOENT segments to the deepest
/// existing ancestor. Ok(None) = "walk failed like Node returning null";
/// Err(Nd) never (non-ENOENT lstat errors return null in the .mjs too).
fn walk_existing_ancestor(lexical: &str) -> Option<(String, Vec<String>)> {
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
fn date_parse_ms(v: Option<&Value>) -> R<Option<f64>> {
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
fn ms_to_iso(ms: f64) -> R<String> {
    if !ms.is_finite() || ms.abs() > 8.64e15 {
        return Err(Nd);
    }
    let dt = chrono::DateTime::from_timestamp_millis(ms as i64).ok_or(Nd)?;
    Ok(dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

/// JS Math.round (half toward +infinity).
fn js_round(v: f64) -> f64 {
    (v + 0.5).floor()
}

// ─── fsutil.mjs readJson (provenance: lib/fsutil.mjs readJson) ─────────────
// Corrupt JSON is Nd: Node warns to stderr with the V8 message.

fn read_json_g(file: &Path) -> R<Option<Value>> {
    match crate::fsutil::read_json(file) {
        crate::fsutil::ReadJson::Missing => Ok(None),
        crate::fsutil::ReadJson::Corrupt => Err(Nd),
        crate::fsutil::ReadJson::Parsed(v) => Ok(Some(v)),
    }
}

// ─── state.mjs ports ───────────────────────────────────────────────────────

/// provenance: state.mjs KNOWN_PHASES / isKnownPhase.
const KNOWN_PHASES: &[&str] = &[
    "idle", "exploring", "planning", "swarming", "reviewing", "scribing",
    "compounding", "grooming", "compounding-complete",
];

fn is_known_phase(phase: &Value) -> bool {
    matches!(phase, Value::String(s) if KNOWN_PHASES.contains(&s.as_str()))
}

/// provenance: state.mjs defaultState() — only the keys guard logic reads.
fn default_state() -> Map<String, Value> {
    let mut gates = Map::new();
    for g in ["context", "shape", "execution", "review"] {
        gates.insert(g.into(), Value::Bool(false));
    }
    let mut m = Map::new();
    m.insert("schema_version".into(), Value::String("1.0".into()));
    m.insert("phase".into(), Value::String("idle".into()));
    m.insert("feature".into(), Value::Null);
    m.insert("mode".into(), Value::Null);
    m.insert("approved_gates".into(), Value::Object(gates));
    m.insert("workers".into(), Value::Array(vec![]));
    m.insert("summary".into(), Value::String(String::new()));
    m.insert(
        "next_action".into(),
        Value::String("No active bee work — awaiting a user request.".into()),
    );
    m
}

/// provenance: state.mjs readState — fail-open merge over defaultState with
/// the D13 legacy-phase coercion. Corrupt file → Nd (readJson warn).
fn read_state(root: &Path) -> R<Map<String, Value>> {
    let file = root.join(".bee").join("state.json");
    let parsed = read_json_g(&file)?;
    let obj = match parsed {
        Some(Value::Object(m)) => m,
        _ => return Ok(default_state()),
    };
    let mut merged = default_state();
    for (k, v) in &obj {
        merged.insert(k.clone(), v.clone());
    }
    // approved_gates: { ...defaults, ...(state.approved_gates || {}) } — a
    // truthy non-object spreads only numeric/char keys, which the four gate
    // names never collide with, so defaults stand for them.
    let mut gates = Map::new();
    for g in ["context", "shape", "execution", "review"] {
        gates.insert(g.into(), Value::Bool(false));
    }
    if let Some(Value::Object(over)) = obj.get("approved_gates") {
        for (k, v) in over {
            gates.insert(k.clone(), v.clone());
        }
    }
    merged.insert("approved_gates".into(), Value::Object(gates));
    if merged.get("phase") == Some(&Value::String("validating".into())) {
        merged.insert("phase".into(), Value::String("planning".into()));
    }
    Ok(merged)
}

/// provenance: state.mjs readConfig (merged tracked+overlay, advisor
/// stripped). Only raw pass-through keys (guards.*, worktree_first,
/// product_root, hooks) are consumed by this hook; the normalize* steps in
/// the .mjs never touch those. Corrupt file → Nd.
fn read_config(root: &Path) -> R<Map<String, Value>> {
    crate::state::read_config_raw(root).map_err(|_| Nd)
}

/// provenance: state.mjs resolveProductRoot — consulted (via resolveContext)
/// only for its WARNING side effects; a configured product_root that would
/// warn (non-string, or missing directory) is Nd.
fn check_product_root_silent(root: &Path, config: &Map<String, Value>) -> R<()> {
    match config.get("product_root") {
        None | Some(Value::Null) => Ok(()),
        Some(Value::String(s)) if s.is_empty() => Ok(()),
        Some(Value::String(s)) => {
            let resolved = if np_is_absolute(s) {
                np_check_modelable(s)?;
                s.clone()
            } else {
                np_resolve2(&root.to_string_lossy(), s)?
            };
            let is_dir = std::fs::metadata(Path::new(&resolved)).map(|m| m.is_dir()).unwrap_or(false);
            if is_dir { Ok(()) } else { Err(Nd) } // Node warns here
        }
        Some(_) => Err(Nd), // non-string → Node warns
    }
}

/// provenance: worktree-store.mjs readGrants — swallow-all read of
/// <mainStoreRoot>/runtime/worktree-grants.json.
fn read_grants(main_bee_dir: &Path) -> Map<String, Value> {
    let file = main_bee_dir.join("runtime").join("worktree-grants.json");
    match std::fs::read_to_string(&file) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(m)) => m,
            _ => Map::new(),
        },
        Err(_) => Map::new(),
    }
}

/// provenance: state.mjs readGitdirFile.
fn read_gitdir_file(file: &Path, base: &str) -> R<Option<String>> {
    let raw = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };
    let mut raw = js_trim(&raw);
    if let Some(rest) = raw.strip_prefix("gitdir:") {
        raw = js_trim(rest);
    }
    if raw.is_empty() {
        return Ok(None);
    }
    let fixed: String = if cfg!(windows) {
        raw.to_string()
    } else {
        raw.replace('\\', "/")
    };
    Ok(Some(np_resolve2(base, &fixed)?))
}

/// The guard-relevant slice of state.mjs resolveContext(cwd).
#[derive(Clone, Default)]
struct JsCtx {
    control_root: Option<String>,
    workspace_root: Option<String>,
    workspace_id: Option<String>,
    worktree_id: Option<String>,
}

enum CtxOutcome {
    Ok(JsCtx),
    /// resolveRootsCore threw (WorktreeLinkInvalidError or a raw stat error).
    Threw,
}

/// provenance: state.mjs resolveRootsCore + resolveContext (workspace-id
/// slice). Returns Threw where Node would throw.
fn resolve_context(cwd: &str) -> R<CtxOutcome> {
    // Nearest onboarding-marker-without-git.
    let mut nearest = np_resolve1(cwd)?;
    loop {
        let n = Path::new(&nearest);
        if n.join(".bee").join("onboarding.json").exists() && !n.join(".git").exists() {
            return finish_ordinary(&nearest);
        }
        let parent = np_dirname(&nearest);
        if parent == nearest {
            break;
        }
        nearest = parent;
    }
    // locateGitRoot.
    let mut located: Option<(String, String)> = None;
    let mut dir = np_resolve1(cwd)?;
    loop {
        if Path::new(&dir).join(".git").exists() {
            let marker = Path::new(&dir).join(".git").to_string_lossy().into_owned();
            located = Some((dir.clone(), marker));
            break;
        }
        let parent = np_dirname(&dir);
        if parent == dir {
            break;
        }
        dir = parent;
    }
    let (work_root, marker) = match located {
        Some(pair) => pair,
        None => {
            // onboarding-marker-anywhere fallback.
            let mut d = np_resolve1(cwd)?;
            loop {
                if Path::new(&d).join(".bee").join("onboarding.json").exists() {
                    return finish_ordinary(&d);
                }
                let parent = np_dirname(&d);
                if parent == d {
                    break;
                }
                d = parent;
            }
            return Ok(CtxOutcome::Ok(JsCtx::default()));
        }
    };
    let marker_stat = match std::fs::metadata(Path::new(&marker)) {
        Ok(s) => s,
        Err(_) => return Ok(CtxOutcome::Threw), // statSync throw (broken symlink .git)
    };
    if !marker_stat.is_file() {
        return finish_ordinary(&work_root);
    }
    // Linked-worktree validation.
    let gitdir = match read_gitdir_file(Path::new(&marker), &work_root)? {
        Some(g) => g,
        None => return Ok(CtxOutcome::Threw), // WorktreeLinkInvalidError
    };
    let worktrees_root = np_resolve2(&gitdir, "..")?;
    let common_git_dir = np_resolve2(&worktrees_root, "..")?;
    if np_basename(&common_git_dir) != ".git" || np_basename(&worktrees_root) != "worktrees" {
        return Ok(CtxOutcome::Threw);
    }
    let id = np_basename(&gitdir);
    if id.is_empty() || id == "." || id == ".." {
        return Ok(CtxOutcome::Threw);
    }
    let reverse = read_gitdir_file(&Path::new(&gitdir).join("gitdir"), &gitdir)?;
    let marker_resolved = np_resolve1(&marker)?;
    match reverse {
        Some(r) if np_resolve1(&r)? == marker_resolved => {}
        _ => return Ok(CtxOutcome::Threw),
    }
    let main_root = np_dirname(&common_git_dir);
    let grants = read_grants(&Path::new(&main_root).join(".bee"));
    let granted = grants.get(&id) == Some(&Value::Bool(true));
    // resolveContext tail (linked branch).
    let config = read_config(Path::new(&work_root))?; // resolveProductRoot(workspaceRoot)
    check_product_root_silent(Path::new(&work_root), &config)?;
    Ok(CtxOutcome::Ok(JsCtx {
        control_root: Some(main_root),
        workspace_root: Some(work_root),
        workspace_id: Some(if granted { id.clone() } else { "main".into() }),
        worktree_id: Some(id),
    }))
}

fn finish_ordinary(root: &str) -> R<CtxOutcome> {
    // resolveContext tail for an ordinary checkout: gitCommonDir stat can
    // throw only for exotic .git states — statSync inside resolveContext is
    // guarded by existsSync first; a race is Nd-irrelevant here.
    let config = read_config(Path::new(root))?;
    check_product_root_silent(Path::new(root), &config)?;
    Ok(CtxOutcome::Ok(JsCtx {
        control_root: Some(root.to_string()),
        workspace_root: Some(root.to_string()),
        workspace_id: Some("main".into()),
        worktree_id: None,
    }))
}

/// provenance: state.mjs controlRootFor — resolveContext(root).controlRoot ??
/// root; a THROW here propagates in Node (no catch until the hook's outer
/// catch-all) → Nd.
fn control_root_for_state(root: &str) -> R<String> {
    match resolve_context(root)? {
        CtxOutcome::Ok(ctx) => Ok(ctx.control_root.unwrap_or_else(|| root.to_string())),
        CtxOutcome::Threw => Err(Nd),
    }
}

// ─── claims.mjs ports ──────────────────────────────────────────────────────

fn sessions_dir(root: &str) -> PathBuf {
    Path::new(root).join(".bee").join("sessions")
}

fn plain_id_ok(id: &str) -> bool {
    let t = js_trim(id);
    !t.is_empty() && !t.contains('/') && !t.contains('\\') && !t.contains("..")
}

/// provenance: claims.mjs readSession (strict=false). Corrupt → Nd (readJson
/// warn); malformed id / missing / shape mismatch → None.
fn read_session(root: &str, session_id: &str) -> R<Option<Map<String, Value>>> {
    if !plain_id_ok(session_id) {
        return Ok(None);
    }
    let file = sessions_dir(root).join(format!("{}.json", js_trim(session_id)));
    let parsed = read_json_g(&file)?;
    match parsed {
        Some(Value::Object(m)) => {
            if m.get("id") == Some(&Value::String(js_trim(session_id).to_string())) {
                Ok(Some(m))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

/// provenance: claims.mjs readSession (strict=true): a parse error or a
/// non-ENOENT read error THROWS in Node (F1) — Nd here (Node's typed
/// detection-error deny carries a V8-worded crash log we cannot replicate).
fn read_session_strict(root: &str, session_id: &str) -> R<Option<Map<String, Value>>> {
    if !plain_id_ok(session_id) {
        return Ok(None);
    }
    let file = sessions_dir(root).join(format!("{}.json", js_trim(session_id)));
    let text = match std::fs::read(&file) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) if io_err_is_enoent(&e) => return Ok(None),
        Err(_) => return Err(Nd),
    };
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let parsed: Value = serde_json::from_str(text).map_err(|_| Nd)?;
    match parsed {
        Value::Object(m) if m.get("id") == Some(&Value::String(js_trim(session_id).to_string())) => {
            Ok(Some(m))
        }
        _ => Ok(None),
    }
}

/// provenance: claims.mjs listSessionRecords.
fn list_session_records(root: &str, strict: bool) -> R<Vec<Map<String, Value>>> {
    let entries = match std::fs::read_dir(sessions_dir(root)) {
        Ok(e) => e,
        Err(e) => {
            if strict && !io_err_is_enoent(&e) {
                return Err(Nd); // F1 throw in Node
            }
            return Ok(Vec::new());
        }
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".json") {
            names.push(name);
        }
    }
    // fs.readdirSync returns sorted order on most platforms; Node does not
    // re-sort, but iteration order only affects which record is seen first —
    // all our consumers are order-independent predicates (some/filter).
    let mut out = Vec::new();
    for name in names {
        let stem = &name[..name.len() - ".json".len()];
        let rec = if strict {
            read_session_strict(root, stem)?
        } else {
            read_session(root, stem)?
        };
        if let Some(r) = rec {
            out.push(r);
        }
    }
    Ok(out)
}

const HEARTBEAT_STALE_SECONDS: f64 = 900.0;

fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

/// provenance: claims.mjs heartbeatStale.
fn heartbeat_stale(session: &Map<String, Value>, now: f64) -> R<bool> {
    let beat = date_parse_ms(session.get("last_heartbeat"))?;
    match beat {
        None => Ok(true),
        Some(ms) => Ok(ms + HEARTBEAT_STALE_SECONDS * 1000.0 <= now),
    }
}

/// provenance: claims.mjs isConcurrentMode.
fn is_concurrent_mode(root: &str, exclude: Option<&str>, strict: bool) -> R<bool> {
    let exclude = exclude.map(js_trim).unwrap_or("");
    let now = now_ms();
    for session in list_session_records(root, strict)? {
        let id_matches = session.get("id") == Some(&Value::String(exclude.to_string()));
        if !id_matches && !heartbeat_stale(&session, now)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// provenance: claims.mjs activeWorkers — reduced to the live session-id
/// view resolveLiveWorkerCount consumes (lane/cell fields are dead there),
/// but the claims-directory scan is still performed so a corrupt claim file
/// (which Node's readJson would WARN about) is caught → Nd.
fn active_worker_session_ids(control_root: &str, exclude: Option<&str>) -> R<Vec<String>> {
    let exclude = exclude.map(js_trim).unwrap_or("");
    let now = now_ms();
    let mut live: Vec<String> = Vec::new();
    for session in list_session_records(control_root, false)? {
        let id = match session.get("id") {
            Some(Value::String(s)) => s.clone(),
            _ => continue,
        };
        if id != exclude && !heartbeat_stale(&session, now)? {
            live.push(id);
        }
    }
    if live.is_empty() {
        return Ok(Vec::new());
    }
    // Claims scan (side-effect parity: corrupt claim JSON warns in Node).
    let claims_dir = Path::new(control_root).join(".bee").join("claims");
    if let Ok(entries) = std::fs::read_dir(&claims_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            let stem = &name[..name.len() - ".json".len()];
            if !plain_id_ok(stem) {
                continue; // requireId throw → caught/skipped in Node
            }
            read_json_g(&claims_dir.join(&name))?; // Corrupt → Nd
        }
    }
    Ok(live)
}

// ─── reservations.mjs + lease-store.mjs read ports ─────────────────────────

const SESSIONLESS_SESSION_ID: &str = "\u{0}bee-reservation-sessionless\u{0}";

/// provenance: reservations.mjs normalizePath (== lease-store
/// canonicalizePath).
fn res_normalize_path(v: &str) -> String {
    let mut s = v.replace('\\', "/");
    // strip ONE leading "./" run: /^\.\/+/
    if s.starts_with("./") {
        let rest = s[1..].trim_start_matches('/');
        s = rest.to_string();
    }
    // strip trailing slashes: /\/+$/
    while s.ends_with('/') {
        s.pop();
    }
    s
}

fn res_normalize_value(v: Option<&Value>) -> String {
    // String(value || '') — falsy → ''.
    match v {
        Some(val) if truthy(val) => res_normalize_path(&js_disp(val)),
        _ => String::new(),
    }
}

/// provenance: reservations.mjs pathsOverlap.
fn paths_overlap(a: &str, b: &str) -> bool {
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
        return true;
    }
    lb.starts_with(&format!("{}/", rb)) || rb.starts_with(&format!("{}/", lb))
}

/// provenance: reservations.mjs findMainRoot/controlRootFor — the
/// self-contained, never-throwing main-root walk.
fn control_root_for_res(root: &str) -> String {
    (|| -> Option<String> {
        // locateGitRootForRoot
        let mut dir = np_resolve1(root).ok()?;
        let (work_root, marker) = loop {
            let m = Path::new(&dir).join(".git");
            if m.exists() {
                break (dir.clone(), m);
            }
            let parent = np_dirname(&dir);
            if parent == dir {
                return None;
            }
            dir = parent;
        };
        let is_file = std::fs::metadata(&marker).ok()?.is_file();
        if !is_file {
            return Some(work_root);
        }
        let read_ptr = |file: &Path, base: &str| -> Option<String> {
            let raw = std::fs::read_to_string(file).ok()?;
            let mut raw = js_trim(&raw);
            if let Some(rest) = raw.strip_prefix("gitdir:") {
                raw = js_trim(rest);
            }
            if raw.is_empty() {
                return None;
            }
            let fixed = if cfg!(windows) { raw.to_string() } else { raw.replace('\\', "/") };
            np_resolve2(base, &fixed).ok()
        };
        let gitdir = read_ptr(&marker, &work_root)?;
        let worktrees_root = np_resolve2(&gitdir, "..").ok()?;
        let common_git_dir = np_resolve2(&worktrees_root, "..").ok()?;
        if np_basename(&common_git_dir) != ".git" || np_basename(&worktrees_root) != "worktrees" {
            return None;
        }
        let id = np_basename(&gitdir);
        if id.is_empty() || id == "." || id == ".." {
            return None;
        }
        let reverse = read_ptr(&Path::new(&gitdir).join("gitdir"), &gitdir)?;
        let marker_s = marker.to_string_lossy().into_owned();
        if np_resolve1(&reverse).ok()? != np_resolve1(&marker_s).ok()? {
            return None;
        }
        Some(np_dirname(&common_git_dir))
    })()
    .unwrap_or_else(|| root.to_string())
}

/// provenance: lease-store.mjs listAllLeaseFiles + readLeaseSafe (silent
/// skip on corrupt — no warn, so no Nd) filtered to path-type leases
/// (reservations.mjs listPathLeaseRecords).
fn list_path_lease_records(root: &str) -> Vec<Map<String, Value>> {
    let control = control_root_for_res(root);
    let leases_root = Path::new(&control).join(".bee").join("runtime").join("leases");
    let mut out = Vec::new();
    for dir in [leases_root.join("cells"), leases_root.join("paths")] {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
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
                if let Ok(Value::Object(m)) = serde_json::from_str::<Value>(&text) {
                    let is_path = matches!(m.get("resource"), Some(Value::String(s)) if s.starts_with("path:"));
                    if is_path {
                        out.push(m);
                    }
                }
            }
        }
    }
    out
}

/// The reservation-shape view of a path lease. provenance: reservations.mjs
/// leaseToReservation / leaseAgent / leaseTtlSeconds.
struct Resv {
    agent: Option<Value>,       // workspace_id minus "agent:" prefix (any JSON type)
    cell: Option<Value>,        // workflow_id
    path: String,               // resource minus "path:"
    ttl_seconds: Option<f64>,   // None = NaN
    reserved_at: Option<Value>, // acquired_at
    session: Option<Value>,     // present only when non-sentinel truthy
    kind: Value,                // kind || 'lease'
}

fn lease_to_reservation(rec: &Map<String, Value>) -> R<Resv> {
    let resource = match rec.get("resource") {
        Some(Value::String(s)) => s.clone(),
        _ => unreachable!("filtered to path leases"),
    };
    let ttl = match rec.get("expires_at") {
        None | Some(Value::Null) => Some(0.0),
        Some(exp) => {
            let e = date_parse_ms(Some(exp))?;
            let a = date_parse_ms(rec.get("acquired_at"))?;
            match (e, a) {
                (Some(e), Some(a)) => Some(js_round((e - a) / 1000.0).max(0.0)),
                _ => None, // NaN through Math.max/round
            }
        }
    };
    let agent = rec.get("workspace_id").map(|w| match w {
        Value::String(s) if s.starts_with("agent:") => Value::String(s["agent:".len()..].to_string()),
        other => other.clone(),
    });
    let session = match rec.get("session_id") {
        Some(v) if truthy(v) && v != &Value::String(SESSIONLESS_SESSION_ID.to_string()) => {
            Some(v.clone())
        }
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

/// provenance: reservations.mjs isLeaseRecordExpired.
fn lease_record_expired(rec: &Map<String, Value>, now: f64) -> R<bool> {
    match rec.get("expires_at") {
        None | Some(Value::Null) => Ok(false),
        Some(v) => match date_parse_ms(Some(v))? {
            None => Ok(false),
            Some(ms) => Ok(ms <= now),
        },
    }
}

/// provenance: reservations.mjs listReservations({activeOnly:true}).
fn list_active_reservations(root: &str) -> R<Vec<Resv>> {
    let now = now_ms();
    let mut out = Vec::new();
    for rec in list_path_lease_records(root) {
        if !lease_record_expired(&rec, now)? {
            out.push(lease_to_reservation(&rec)?);
        }
    }
    Ok(out)
}

/// provenance: reservations.mjs findConflicts.
fn find_conflicts(root: &str, agent: &str, paths: &[String]) -> R<Vec<Resv>> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for resv in list_active_reservations(root)? {
        let same_agent = matches!(&resv.agent, Some(Value::String(s)) if s == agent);
        if !same_agent && paths.iter().any(|p| paths_overlap(&resv.path, p)) {
            out.push(resv);
        }
    }
    Ok(out)
}

/// provenance: reservations.mjs findSessionConflicts.
fn find_session_conflicts(root: &str, session_id: &str, paths: &[String]) -> R<Vec<Resv>> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let acting = js_trim(session_id);
    let mut out = Vec::new();
    for resv in list_active_reservations(root)? {
        let sess_ok = match &resv.session {
            Some(Value::String(s)) if !js_trim(s).is_empty() && s != acting => true,
            _ => false,
        };
        if sess_ok && paths.iter().any(|p| paths_overlap(&resv.path, p)) {
            out.push(resv);
        }
    }
    Ok(out)
}

/// provenance: reservations.mjs isHardConflict.
fn is_hard_conflict(resv: &Resv, target: &str) -> bool {
    !(resv.kind == Value::String("intent".into())
        && res_normalize_path(&resv.path) != res_normalize_path(target))
}

/// provenance: guards.mjs reservationStoreCorrupt.
fn reservation_store_corrupt(root: &str) -> bool {
    let file = Path::new(root).join(".bee").join("reservations.json");
    if !file.exists() {
        return false;
    }
    match std::fs::read_to_string(&file) {
        Ok(text) => serde_json::from_str::<Value>(&text).is_err(),
        Err(_) => true, // readFileSync throw is caught → corrupt
    }
}

// ─── worktree-holds.mjs ports ──────────────────────────────────────────────

fn holds_ledger_path(main_root: &str) -> PathBuf {
    Path::new(main_root).join(".bee").join("runtime").join("cross-worktree-holds.json")
}

/// provenance: worktree-holds.mjs holdsStoreCorrupt.
fn holds_store_corrupt(main_root: &str) -> bool {
    let file = holds_ledger_path(main_root);
    if !file.exists() {
        return false;
    }
    match std::fs::read_to_string(&file) {
        Ok(text) => serde_json::from_str::<Value>(&text).is_err(),
        Err(_) => true,
    }
}

/// provenance: worktree-holds.mjs findForeignHolds (+ isActive/isExpired).
fn find_foreign_holds(main_root: &str, holder: &str, paths: &[String]) -> R<Vec<Map<String, Value>>> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let store = read_json_g(&holds_ledger_path(main_root))?;
    let holds = match store {
        Some(Value::Object(m)) => match m.get("holds") {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    let acting = js_trim(holder);
    let now = now_ms();
    let mut out = Vec::new();
    for hold in holds {
        let hold = match hold {
            Value::Object(m) => m,
            _ => continue, // property reads on a non-object entry — a non-
                           // object array element would throw in Node only on
                           // null property access; entry.released_at on a
                           // string is undefined (== null → active)… model:
        };
        // released_at == null (JS loose) → active half
        let released_null = matches!(hold.get("released_at"), None | Some(Value::Null));
        if !released_null {
            continue;
        }
        // isExpired
        let ttl = hold.get("ttl_seconds").and_then(Value::as_f64);
        let expired = match ttl {
            Some(t) if t > 0.0 => match date_parse_ms(hold.get("mirrored_at"))? {
                Some(m) => m + t * 1000.0 <= now,
                None => false,
            },
            _ => false,
        };
        if expired {
            continue;
        }
        let holder_matches = matches!(hold.get("holder"), Some(Value::String(s)) if s == acting);
        if holder_matches {
            continue;
        }
        let hold_path = res_normalize_value(hold.get("path"));
        let _ = hold_path; // pathsOverlap normalizes again; use raw coercion:
        let hp = match hold.get("path") {
            Some(v) => js_disp(v),
            None => String::new(),
        };
        if paths.iter().any(|p| paths_overlap(&hp, p)) {
            out.push(hold);
        }
    }
    Ok(out)
}

/// provenance: guards.mjs holdExpiry (reservation flavor).
fn hold_expiry(resv: &Resv) -> R<String> {
    let reserved = date_parse_ms(resv.reserved_at.as_ref())?;
    match (reserved, resv.ttl_seconds) {
        (Some(r), Some(t)) if t > 0.0 => Ok(format!("expires {}", ms_to_iso(r + t * 1000.0)?)),
        _ => Ok("no expiry".to_string()),
    }
}

/// provenance: guards.mjs foreignHoldExpiry.
fn foreign_hold_expiry(hold: &Map<String, Value>) -> R<String> {
    let mirrored = date_parse_ms(hold.get("mirrored_at"))?;
    let ttl = hold.get("ttl_seconds").and_then(Value::as_f64);
    match (mirrored, ttl) {
        (Some(m), Some(t)) if t > 0.0 => Ok(format!("expires {}", ms_to_iso(m + t * 1000.0)?)),
        _ => Ok("no expiry".to_string()),
    }
}

// ─── workspace-store.mjs ports ─────────────────────────────────────────────

enum WorkspaceRead {
    Missing,
    Corrupt,
    Ok(Map<String, Value>),
}

/// provenance: workspace-store.mjs readWorkspaceRecord (read-only slice; the
/// guard only consumes write_owner_session).
fn read_workspace(control_root: &str, id: &str) -> WorkspaceRead {
    if !plain_id_ok(id) {
        // requireWorkspaceId throws WORKSPACE_INVALID_ID — checkWorkspace-
        // Ownership's catch treats any non-MISSING error as corrupt.
        return WorkspaceRead::Corrupt;
    }
    let file = Path::new(control_root)
        .join(".bee")
        .join("runtime")
        .join("workspaces")
        .join(format!("{}.json", js_trim(id)));
    let text = match std::fs::read(&file) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) if io_err_is_enoent(&e) => return WorkspaceRead::Missing,
        Err(_) => return WorkspaceRead::Corrupt,
    };
    let parsed: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return WorkspaceRead::Corrupt,
    };
    let obj = match parsed {
        Value::Object(m) => m,
        _ => return WorkspaceRead::Corrupt,
    };
    if obj.get("id") != Some(&Value::String(js_trim(id).to_string())) {
        return WorkspaceRead::Corrupt;
    }
    let mut merged = Map::new();
    merged.insert("write_owner_session".into(), Value::Null);
    merged.insert("fence_epoch".into(), Value::Number(0.into()));
    merged.insert("attached_sessions".into(), Value::Array(vec![]));
    merged.insert("branch".into(), Value::Null);
    merged.insert("base_sha".into(), Value::Null);
    for (k, v) in obj {
        merged.insert(k, v);
    }
    WorkspaceRead::Ok(merged)
}

// ─── guards.mjs pure ports ─────────────────────────────────────────────────

/// provenance: guards.mjs normalizeRel.
fn normalize_rel(rel: &str) -> String {
    let s = rel.replace('\\', "/");
    if s.starts_with("./") {
        s[1..].trim_start_matches('/').to_string()
    } else {
        s
    }
}

const GATE_ALLOWED_PREFIXES: [&str; 4] = [".bee/", "docs/", "plans/", "AGENTS.md"];

/// provenance: guards.mjs underAllowedPrefix.
fn under_allowed_prefix(rel: &str) -> bool {
    let normalized = normalize_rel(rel);
    GATE_ALLOWED_PREFIXES.iter().any(|prefix| {
        if let Some(bare) = prefix.strip_suffix('/') {
            normalized == bare || normalized.starts_with(prefix)
        } else {
            normalized == *prefix
        }
    })
}

/// provenance: guards.mjs DIRECT_EDIT_DENY.
fn direct_edit_verb(normalized: &str) -> Option<&'static str> {
    match normalized {
        ".bee/state.json" => Some("bee.mjs state set --owner <selected pre-mutation phase>, or the dedicated state gate/worker/scribing-run verb"),
        ".bee/backlog.jsonl" => Some("bee.mjs backlog add"),
        "docs/backlog.md" => Some("bee.mjs backlog pbi add / bee.mjs backlog pbi status / bee.mjs backlog pbi amend to change data, or bee.mjs backlog render --write to regenerate the view"),
        ".bee/runtime/cross-worktree-holds.json" => Some("bee.mjs reservations reserve/release (holds are mirrored into the ledger automatically)"),
        ".bee/runtime/worktree-grants.json" => Some("bee.mjs worktree register / unregister"),
        ".bee/companion-session.json" => Some("bee worktree new --with-companion (started/ended automatically by the companion lifecycle)"),
        _ => None,
    }
}

/// provenance: guards.mjs HISTORY_CODE_EXTENSIONS / docsHistoryCodeDeny.
fn docs_history_code_deny(normalized: &str) -> Option<String> {
    if !normalized.starts_with("docs/history/") {
        return None;
    }
    let dot = normalized.rfind('.')?;
    let ext = normalized[dot..].to_lowercase();
    const EXTS: [&str; 20] = [
        ".sh", ".bash", ".zsh", ".fish", ".ps1", ".bat", ".cmd", ".mjs", ".cjs", ".js", ".jsx",
        ".ts", ".tsx", ".py", ".rb", ".go", ".rs", ".java", ".php", ".pl",
    ];
    const EXTS2: [&str; 2] = [".lua", ".r"];
    if EXTS.contains(&ext.as_str()) || EXTS2.contains(&ext.as_str()) {
        Some(ext)
    } else {
        None
    }
}

/// provenance: guards.mjs SCRATCH_* / scratchShapeDeny.
fn scratch_shape_deny(normalized: &str) -> Option<String> {
    let under_any = |prefixes: &[&str]| {
        prefixes.iter().any(|p| normalized == &p[..p.len() - 1] || normalized.starts_with(p))
    };
    if under_any(&[".bee/tmp/", ".bee/spikes/", ".bee/logs/", ".bee/workers/"]) {
        return None;
    }
    if normalized == ".bee/decisions.jsonl" {
        return None;
    }
    if under_any(&[
        "docs/", ".bee/cells/", ".claude-plugin/skills/", ".codex-plugin/skills/",
        ".claude/skills/", ".agents/skills/",
    ]) {
        return None;
    }
    let basename = &normalized[normalized.rfind('/').map(|i| i + 1).unwrap_or(0)..];
    let lower = basename.to_lowercase();
    // SCRATCH_DOTFILE_RE: ^\.[^/]*(?:debug|stress|scratch)[^/]*$ (i)
    if lower.starts_with('.')
        && (lower.contains("debug") || lower.contains("stress") || lower.contains("scratch"))
    {
        return Some("a dotfile named like a debug/stress/scratch script".to_string());
    }
    // SCRATCH_PREFIX_RE: ^(?:verdict|probe|digest)- (i)
    if lower.starts_with("verdict-") || lower.starts_with("probe-") || lower.starts_with("digest-") {
        return Some("a verdict-/probe-/digest- style scratch payload".to_string());
    }
    // SCRATCH_EXT_RE: \.(tmp|log|bak)$ (i), exempted in test/fixture dirs.
    let ext_hit = [".tmp", ".log", ".bak"].iter().any(|e| lower.ends_with(e));
    if ext_hit && !in_test_fixture_dir(normalized) {
        let dot = basename.rfind('.').unwrap();
        return Some(format!("a {} scratch file", &basename[dot..]));
    }
    None
}

/// provenance: guards.mjs TEST_FIXTURE_DIR_RE — (^|/)(test|tests|__tests__|
/// fixtures|__fixtures__|testdata|examples)(/|$) case-insensitive.
fn in_test_fixture_dir(normalized: &str) -> bool {
    const NAMES: [&str; 7] = ["test", "tests", "__tests__", "fixtures", "__fixtures__", "testdata", "examples"];
    let lower = normalized.to_lowercase();
    let segments: Vec<&str> = lower.split('/').collect();
    segments.iter().any(|s| NAMES.contains(s))
}

/// provenance: guards.mjs SECRET_PATTERNS + checkRead privacy half.
fn is_secret_path(normalized: &str) -> bool {
    let lower = normalized.to_lowercase();
    let base = &lower[lower.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0)..];
    // /(^|[\\/])\.env(\.[A-Za-z0-9._-]+)?$/i
    if base == ".env" {
        return true;
    }
    if let Some(rest) = base.strip_prefix(".env.") {
        if !rest.is_empty()
            && rest.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        {
            return true;
        }
    }
    if lower.ends_with(".pem") || lower.ends_with(".key") || lower.ends_with(".p12") {
        return true;
    }
    if base.starts_with("id_rsa") || base.starts_with("credentials") {
        return true;
    }
    // /(^|[\\/])secrets\.[^\\/]+$/i
    if let Some(rest) = base.strip_prefix("secrets.") {
        if !rest.is_empty() {
            return true;
        }
    }
    false
}

const SCOUT_DIRS: [&str; 8] = [
    "node_modules/", "dist/", "build/", ".git/objects", "vendor/", "coverage/", ".next/",
    "__pycache__/",
];

enum ReadVerdict {
    Allow,
    Deny { reason: String, marker: Option<String> },
}

/// provenance: guards.mjs checkRead.
fn check_read(rel: &str) -> ReadVerdict {
    let normalized = normalize_rel(rel);
    if is_secret_path(&normalized) {
        let question = format!(
            "\"{}\" looks like a secret/credential file. Ask the user before reading it.",
            normalized
        );
        let mut obj = Map::new();
        obj.insert("file".into(), Value::String(normalized.clone()));
        obj.insert("question".into(), Value::String(question.clone()));
        let marker = format!("@@BEE_PRIVACY@@{}@@END@@", jsjson::stringify(&Value::Object(obj)));
        return ReadVerdict::Deny {
            reason: format!("bee privacy guard: {}", question),
            marker: Some(marker),
        };
    }
    if let Some(hit) = SCOUT_DIRS
        .iter()
        .find(|dir| normalized.starts_with(*dir) || normalized.contains(&format!("/{}", dir)))
    {
        return ReadVerdict::Deny {
            reason: format!(
                "bee scout guard: \"{}\" is inside \"{}\" — generated/vendored content. Read the source or lockfile instead.",
                normalized, hit
            ),
            marker: None,
        };
    }
    ReadVerdict::Allow
}

// ─── tokenizer (provenance: hooks/tokenize-command.mjs == guards.mjs
// tokenize — byte-identical algorithm, hand-synced there, single port here) ──

fn tokenize(command: &str) -> Vec<String> {
    let chars: Vec<char> = command.chars().collect();
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut has_current = false;
    let mut i = 0usize;
    macro_rules! flush {
        () => {
            if has_current {
                tokens.push(std::mem::take(&mut current));
                #[allow(unused_assignments)]
                {
                    has_current = false;
                }
            }
        };
    }
    while i < chars.len() {
        let ch = chars[i];
        if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
            flush!();
            i += 1;
            continue;
        }
        if ch == '\\' && i + 1 < chars.len() {
            current.push(chars[i + 1]);
            has_current = true;
            i += 2;
            continue;
        }
        if ch == '"' || ch == '\'' {
            let close = chars[i + 1..].iter().position(|&c| c == ch).map(|p| p + i + 1);
            let end = close.unwrap_or(chars.len());
            for &c in &chars[i + 1..end] {
                current.push(c);
            }
            has_current = true;
            i = end + 1;
            continue;
        }
        if (ch == '&' && chars.get(i + 1) == Some(&'&')) || (ch == '|' && chars.get(i + 1) == Some(&'|')) {
            flush!();
            tokens.push(format!("{}{}", ch, ch));
            i += 2;
            continue;
        }
        if ch == ';' || ch == '&' || ch == '|' {
            flush!();
            tokens.push(ch.to_string());
            i += 1;
            continue;
        }
        current.push(ch);
        has_current = true;
        i += 1;
    }
    flush!();
    tokens
}

fn is_separator(t: &str) -> bool {
    matches!(t, "&&" | "||" | ";" | "|" | "&")
}

// ─── bash target extraction (provenance: guards.mjs extractBashTargets) ────

struct BashTargets {
    paths: Vec<String>,
    broad_write: bool,
}

fn is_flag(t: &str) -> bool {
    t.starts_with('-')
}

/// provenance: guards.mjs isBroad + BROAD_TARGETS.
fn is_broad(target: &str) -> bool {
    const BROAD: [&str; 7] = [".", "..", "/", "~", "*", "./*", "/*"];
    let normalized = normalize_rel(target);
    BROAD.contains(&target)
        || BROAD.contains(&normalized.as_str())
        || normalized.ends_with("/*")
        || normalized.ends_with("/.")
        || normalized == "*"
}

/// provenance: guards.mjs hasGitShortFlag.
fn has_git_short_flag(tokens: &[String], letter: char) -> bool {
    tokens.iter().any(|t| {
        t.len() >= 2
            && t.starts_with('-')
            && !t[1..].is_empty()
            && t[1..].chars().all(|c| c.is_ascii_alphabetic())
            && t[1..].contains(letter)
    })
}

fn extract_bash_targets(command: &str) -> BashTargets {
    let tokens = tokenize(command);
    let mut paths: Vec<String> = Vec::new();
    let mut broad_write = false;
    let add_target = |target: &str, broad: &mut bool, paths: &mut Vec<String>| {
        if target.is_empty() || target == "/dev/null" || target == "NUL" {
            return;
        }
        if is_broad(target) {
            *broad = true;
        }
        paths.push(target.to_string());
    };
    let mut i = 0usize;
    while i < tokens.len() {
        let token = &tokens[i];
        // Redirection: /^\d?>{1,2}(.*)$/ — fd-duplication (&-target) excluded.
        if let Some(inline) = match_redirect(token) {
            if !inline.is_empty() {
                if !inline.starts_with('&') {
                    add_target(&inline, &mut broad_write, &mut paths);
                }
            } else if let Some(next) = tokens.get(i + 1) {
                if !is_separator(next) && !next.starts_with('&') {
                    add_target(next, &mut broad_write, &mut paths);
                    i += 1;
                }
            }
            i += 1;
            continue;
        }
        if is_separator(token) {
            i += 1;
            continue;
        }
        let cmd = token.replace('\\', "/");
        let cmd = cmd.rsplit('/').next().unwrap_or("");
        if cmd == "git" && matches!(tokens.get(i + 1).map(String::as_str), Some("add") | Some("mv") | Some("rm")) {
            let git_verb = tokens[i + 1].clone();
            let mut end = i + 2;
            while end < tokens.len() && !is_separator(&tokens[end]) {
                end += 1;
            }
            let segment: Vec<String> = tokens[i + 2..end].to_vec();
            if git_verb == "add"
                && (segment.iter().any(|t| t == "--all")
                    || segment.iter().any(|t| t == "--update")
                    || has_git_short_flag(&segment, 'A')
                    || has_git_short_flag(&segment, 'u'))
            {
                broad_write = true;
            }
            for t in &segment {
                if !is_flag(t) {
                    // D8: staging a CLI-owned file with `git add` is not a
                    // direct-edit target.
                    let cli_owned_stage_only =
                        git_verb == "add" && direct_edit_verb(&normalize_rel(t)).is_some();
                    if !cli_owned_stage_only {
                        add_target(t, &mut broad_write, &mut paths);
                    }
                }
            }
            i = end;
            continue;
        }
        if cmd == "git" && tokens.get(i + 1).map(String::as_str) == Some("commit") {
            let mut end = i + 2;
            while end < tokens.len() && !is_separator(&tokens[end]) {
                end += 1;
            }
            let segment: Vec<String> = tokens[i + 2..end].to_vec();
            if segment.iter().any(|t| t == "--all") || has_git_short_flag(&segment, 'a') {
                broad_write = true;
            }
            i = end;
            continue;
        }
        if cmd == "sed" {
            let mut in_place = false;
            let mut last = i;
            let mut args: Vec<String> = Vec::new();
            let mut j = i + 1;
            while j < tokens.len() && !is_separator(&tokens[j]) {
                if tokens[j].starts_with("-i") {
                    in_place = true;
                } else if !is_flag(&tokens[j]) {
                    args.push(tokens[j].clone());
                }
                last = j;
                j += 1;
            }
            if in_place {
                for file in args.iter().skip(1) {
                    add_target(file, &mut broad_write, &mut paths);
                }
            }
            i = last + 1;
            continue;
        }
        if matches!(cmd, "rm" | "mv" | "cp" | "mkdir" | "touch" | "tee") {
            let mut saw_any = false;
            let mut last = i;
            let mut j = i + 1;
            while j < tokens.len() && !is_separator(&tokens[j]) {
                if !is_flag(&tokens[j]) {
                    add_target(&tokens[j], &mut broad_write, &mut paths);
                    saw_any = true;
                }
                last = j;
                j += 1;
            }
            if cmd == "rm" && !saw_any {
                broad_write = true;
            }
            i = last + 1;
            continue;
        }
        i += 1;
    }
    BashTargets { paths, broad_write }
}

/// /^\d?>{1,2}(.*)$/ → Some(captured tail) when the token is a redirect.
fn match_redirect(token: &str) -> Option<String> {
    let chars: Vec<char> = token.chars().collect();
    let mut idx = 0usize;
    if idx < chars.len() && chars[idx].is_ascii_digit() {
        // optional single digit — but only if a '>' follows it or starts the token
        if chars.get(idx + 1) == Some(&'>') {
            idx += 1;
        } else {
            return None;
        }
    }
    if chars.get(idx) != Some(&'>') {
        return None;
    }
    idx += 1;
    if chars.get(idx) == Some(&'>') {
        idx += 1;
    }
    Some(chars[idx..].iter().collect())
}

// ─── exclusive-path globs (provenance: guards.mjs DEFAULT_EXCLUSIVE_PATHS +
// globToRegExp + isExclusivePath) ──────────────────────────────────────────

const DEFAULT_EXCLUSIVE_PATHS: [&str; 15] = [
    "**/migrations/**",
    "package-lock.json",
    "**/package-lock.json",
    "yarn.lock",
    "**/yarn.lock",
    "pnpm-lock.yaml",
    "**/pnpm-lock.yaml",
    "Cargo.lock",
    "**/Cargo.lock",
    "composer.lock",
    "**/composer.lock",
    "Gemfile.lock",
    "**/Gemfile.lock",
    "docs/history/codex-harness-hardening/release-manifest.json",
    ".bee/onboarding.json",
];

#[derive(Clone)]
enum GlobTok {
    Lit(char),
    Star,          // '*'  -> [^/]*
    AnyAll,        // '**' (no trailing slash) -> .*
    AnyDirsPrefix, // '**/' -> (?:.*/)?
}

fn glob_tokens(glob: &str) -> Vec<GlobTok> {
    let normalized = {
        let s = glob.replace('\\', "/");
        if s.starts_with("./") {
            s[1..].trim_start_matches('/').to_string()
        } else {
            s
        }
    };
    let chars: Vec<char> = normalized.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '*' && chars.get(i + 1) == Some(&'*') {
            if chars.get(i + 2) == Some(&'/') {
                out.push(GlobTok::AnyDirsPrefix);
                i += 3;
            } else {
                out.push(GlobTok::AnyAll);
                i += 2;
            }
            continue;
        }
        if chars[i] == '*' {
            out.push(GlobTok::Star);
            i += 1;
            continue;
        }
        out.push(GlobTok::Lit(chars[i]));
        i += 1;
    }
    out
}

fn glob_match(toks: &[GlobTok], input: &[char]) -> bool {
    match toks.first() {
        None => input.is_empty(),
        Some(GlobTok::Lit(c)) => {
            input.first() == Some(c) && glob_match(&toks[1..], &input[1..])
        }
        Some(GlobTok::Star) => {
            // [^/]* — try consuming 0..n non-slash chars
            let mut n = 0usize;
            loop {
                if glob_match(&toks[1..], &input[n..]) {
                    return true;
                }
                if n >= input.len() || input[n] == '/' {
                    return false;
                }
                n += 1;
            }
        }
        Some(GlobTok::AnyAll) => {
            // .* — any run (regex '.' with no /s flag excludes newline; paths
            // with newlines are unmatchable there — mirror by excluding '\n')
            let mut n = 0usize;
            loop {
                if glob_match(&toks[1..], &input[n..]) {
                    return true;
                }
                if n >= input.len() || input[n] == '\n' {
                    return false;
                }
                n += 1;
            }
        }
        Some(GlobTok::AnyDirsPrefix) => {
            // (?:.*/)? — empty, or any run ending at a '/'
            if glob_match(&toks[1..], input) {
                return true;
            }
            for (idx, &c) in input.iter().enumerate() {
                if c == '/' && glob_match(&toks[1..], &input[idx + 1..]) {
                    return true;
                }
                if c == '\n' {
                    break;
                }
            }
            false
        }
    }
}

/// provenance: guards.mjs isExclusivePath — defaults EXTENDED by
/// config.guards.exclusive_paths.
fn is_exclusive_path(root: &Path, normalized: &str) -> R<bool> {
    let config = read_config(root)?;
    let mut globs: Vec<String> = DEFAULT_EXCLUSIVE_PATHS.iter().map(|s| s.to_string()).collect();
    if let Some(Value::Object(g)) = config.get("guards") {
        if let Some(Value::Array(extra)) = g.get("exclusive_paths") {
            for e in extra {
                if let Value::String(s) = e {
                    if !js_trim(s).is_empty() {
                        globs.push(s.clone());
                    }
                }
            }
        }
    }
    let input: Vec<char> = normalized.chars().collect();
    Ok(globs.iter().any(|g| glob_match(&glob_tokens(g), &input)))
}

// ─── git classification (provenance: guards.mjs ige-2 / gc-2 section) ─────

fn git_global_flag_takes_value(t: &str) -> bool {
    matches!(t, "-C" | "-c" | "--git-dir" | "--work-tree" | "--namespace")
}

struct GitInvocation {
    subcommand: Option<String>,
    rest: Vec<String>,
}

/// provenance: guards.mjs findGitInvocation.
fn find_git_invocation(tokens: &[String]) -> Option<GitInvocation> {
    let mut i = 0usize;
    while i < tokens.len() {
        if is_separator(&tokens[i]) {
            i += 1;
            continue;
        }
        let cmd = tokens[i].replace('\\', "/");
        let cmd = cmd.rsplit('/').next().unwrap_or("");
        if cmd != "git" {
            i += 1;
            continue;
        }
        let mut end = i + 1;
        while end < tokens.len() && !is_separator(&tokens[end]) {
            end += 1;
        }
        let invocation: Vec<String> = tokens[i + 1..end].to_vec();
        let mut subcommand: Option<String> = None;
        let mut sub_idx: Option<usize> = None;
        let mut j = 0usize;
        while j < invocation.len() {
            let t = &invocation[j];
            if git_global_flag_takes_value(t) {
                j += 2;
                continue;
            }
            if t.starts_with('-') {
                j += 1;
                continue;
            }
            subcommand = Some(t.clone());
            sub_idx = Some(j);
            break;
        }
        return match (subcommand, sub_idx) {
            (Some(s), Some(idx)) => Some(GitInvocation {
                subcommand: Some(s),
                rest: invocation[idx + 1..].to_vec(),
            }),
            _ => Some(GitInvocation { subcommand: None, rest: Vec::new() }),
        };
    }
    None
}

/// provenance: guards.mjs runGitCapture.
fn run_git_capture(cwd: &str, args: &[&str]) -> Option<Vec<String>> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None; // execFileSync throws on non-zero
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    Some(
        text.split(['\n'])
            .map(|l| js_trim(l.trim_end_matches('\r')).to_string())
            .filter(|l| !l.is_empty())
            .collect(),
    )
}

const GIT_BROAD_PATHSPECS: [&str; 4] = [".", ":", ":/", "./"];

/// provenance: guards.mjs extractExplicitPathspecs.
fn extract_explicit_pathspecs(rest: &[String]) -> Vec<String> {
    match rest.iter().position(|t| t == "--") {
        None => rest.iter().filter(|t| !t.starts_with('-')).cloned().collect(),
        Some(idx) => rest[idx + 1..].to_vec(),
    }
}

/// provenance: guards.mjs resolveGitMutationPaths.
fn resolve_git_mutation_paths(cwd: &str, subcommand: &str, rest: &[String]) -> Option<Vec<String>> {
    let broad = |p: &String| GIT_BROAD_PATHSPECS.contains(&p.as_str()) || p.contains('*');
    if subcommand == "commit" {
        let dash = rest.iter().position(|t| t == "--");
        let explicit: Vec<String> = match dash {
            None => Vec::new(),
            Some(idx) => rest[idx + 1..].to_vec(),
        };
        let pre: Vec<String> = match dash {
            None => rest.to_vec(),
            Some(idx) => rest[..idx].to_vec(),
        };
        let is_all = has_git_short_flag(&pre, 'a') || pre.iter().any(|t| t == "--all");
        let staged = run_git_capture(cwd, &["diff", "--cached", "--name-only"])?;
        if !explicit.is_empty() {
            if explicit.iter().any(broad) {
                return None;
            }
            return Some(explicit);
        }
        if !is_all {
            return Some(staged);
        }
        let unstaged = run_git_capture(cwd, &["diff", "--name-only"])?;
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for p in staged.into_iter().chain(unstaged.into_iter()) {
            if seen.insert(p.clone()) {
                out.push(p);
            }
        }
        return Some(out);
    }
    let pathspecs = extract_explicit_pathspecs(rest);
    if pathspecs.is_empty() {
        return None;
    }
    if pathspecs.iter().any(broad) {
        return None;
    }
    Some(pathspecs)
}

struct TreeVerbClass {
    verb: String,
    why: &'static str,
}

/// provenance: guards.mjs classifyConcurrentTreeVerb.
fn classify_concurrent_tree_verb(subcommand: Option<&str>, rest: &[String]) -> Option<TreeVerbClass> {
    let sub = subcommand?;
    if sub == "add" {
        if has_git_short_flag(rest, 'N') || rest.iter().any(|t| t == "--intent-to-add") {
            return None;
        }
        return Some(TreeVerbClass {
            verb: "add".into(),
            why: "it stages content into the SHARED index, so the next sibling worker to commit sweeps your files into their commit — the exact attribution loss that happened twice in one wave.",
        });
    }
    if sub == "commit" {
        let dash = rest.iter().position(|t| t == "--");
        let pre: Vec<String> = match dash {
            None => rest.to_vec(),
            Some(idx) => rest[..idx].to_vec(),
        };
        if has_git_short_flag(&pre, 'a') || pre.iter().any(|t| t == "--all") {
            return Some(TreeVerbClass {
                verb: "commit -a".into(),
                why: "`-a`/`--all` commits every tracked modification in the checkout, including a sibling worker's in-progress edits.",
            });
        }
        if let Some(idx) = dash {
            let pathspecs: Vec<String> = rest[idx + 1..].to_vec();
            if !pathspecs.is_empty()
                && !pathspecs
                    .iter()
                    .any(|p| GIT_BROAD_PATHSPECS.contains(&p.as_str()) || p.contains('*'))
            {
                return None;
            }
        }
        return Some(TreeVerbClass {
            verb: "commit".into(),
            why: "with no explicit `-- <paths>` pathspec it commits whatever sits in the SHARED index, which may be a sibling worker's staged work.",
        });
    }
    if sub == "stash" {
        let first_word = rest.iter().find(|t| !t.starts_with('-'));
        if let Some(w) = first_word {
            if matches!(w.as_str(), "list" | "show") {
                return None;
            }
        }
        return Some(TreeVerbClass {
            verb: "stash".into(),
            why: "it sweeps every uncommitted change in the checkout out of the tree, including edits a sibling worker is still writing.",
        });
    }
    if sub == "apply" {
        if rest.iter().any(|t| matches!(t.as_str(), "--check" | "--stat" | "--summary" | "--numstat")) {
            return None;
        }
        return Some(TreeVerbClass {
            verb: "apply".into(),
            why: "it rewrites tree content wholesale, and reservations cannot protect a tree.",
        });
    }
    if matches!(sub, "reset" | "clean" | "checkout" | "restore" | "revert" | "rebase" | "cherry-pick" | "merge") {
        return Some(TreeVerbClass {
            verb: sub.to_string(),
            why: "it rewrites the working tree or index as a whole, which no file reservation can protect — reservations govern FILES, and the working tree is not a file.",
        });
    }
    None
}

/// provenance: guards.mjs concurrentTreeRefusal.
fn concurrent_tree_refusal(verb: &str, why: &str, worker_clause: &str) -> String {
    format!(
        "bee concurrent-worker git guard: `git {verb}` is refused because {worker_clause}. {why} \
FIX: inspection is always allowed — git status / git diff / git log. To land your own work, make ONE path-scoped \
commit through your OWN temp index instead of the shared one: \
GIT_INDEX_FILE=<tmp> git read-tree HEAD, then GIT_INDEX_FILE=<tmp> git update-index --add <your paths>, \
GIT_INDEX_FILE=<tmp> git write-tree, git commit-tree <tree> -p HEAD -m \"<msg>\", git update-ref HEAD <commit>. \
For a path git does not track yet, `git add -N <path>` first (intent-to-add stages no content). \
A genuinely path-scoped `git commit -- <your paths>` is allowed too. Never reset / stash / checkout / clean / \
restore / revert across the shared tree while a sibling worker holds work in it — a file reservation cannot protect a tree."
    )
}

/// provenance: guards.mjs sessionWorkspaceId.
fn session_workspace_id(control_root: &str, session_id: &Value) -> R<String> {
    let sid = match session_id {
        Value::String(s) => s.clone(),
        _ => return Ok("main".to_string()), // requireId throw → readSession null → 'main'
    };
    let session = read_session(control_root, &sid)?;
    Ok(match session.and_then(|s| s.get("workspace_id").cloned()) {
        Some(Value::String(w)) if !js_trim(&w).is_empty() => w,
        _ => "main".to_string(),
    })
}

enum WorkerCount {
    Resolved(usize),
    Unresolved(&'static str),
}

/// provenance: guards.mjs resolveLiveWorkerCount.
fn resolve_live_worker_count(root: &str, control_root: &str, ctx: &JsCtx) -> R<WorkerCount> {
    let own_workspace = ctx.workspace_id.clone().unwrap_or_else(|| "main".to_string());
    if reservation_store_corrupt(root) {
        return Ok(WorkerCount::Unresolved("the reservation store is present but unparseable"));
    }
    let reservations = list_active_reservations(root)?;
    let mut worker_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut sessions_with_agents: std::collections::HashSet<String> = std::collections::HashSet::new();
    for resv in &reservations {
        let agent = match &resv.agent {
            Some(Value::String(s)) => js_trim(s).to_string(),
            _ => String::new(),
        };
        if agent.is_empty() {
            continue;
        }
        let session = match &resv.session {
            Some(Value::String(s)) => js_trim(s).to_string(),
            _ => String::new(),
        };
        // sameWorkspace: unattributed counts; else compare stamped ids.
        if !session.is_empty()
            && session_workspace_id(control_root, &Value::String(session.clone()))? != own_workspace
        {
            continue;
        }
        worker_keys.insert(format!("{}::agent:{}", session, agent));
        if !session.is_empty() {
            sessions_with_agents.insert(session);
        }
    }
    let workers = active_worker_session_ids(control_root, None)?;
    for sid in workers {
        let sid_t = js_trim(&sid).to_string();
        if sid_t.is_empty() || sessions_with_agents.contains(&sid_t) {
            continue;
        }
        if session_workspace_id(control_root, &Value::String(sid_t.clone()))? != own_workspace {
            continue;
        }
        worker_keys.insert(format!("{}::session", sid_t));
    }
    Ok(WorkerCount::Resolved(worker_keys.len()))
}

/// provenance: guards.mjs intakeFixLine / intakeRefusal.
fn intake_fix_line() -> String {
    format!(
        "FIX: commit or write bookkeeping directly — {} are exempt from this gate — \
or route the request through bee-hive first (classify the mode; tiny fixes stay tiny — one cell, a 2-minute \
reality check, Gate 3, go), then execute. Last resort, repo-level opt-out: \
bee config set --key guards.idle_gate --value false (re-enable with: bee config unset --key guards.idle_gate).",
        GATE_ALLOWED_PREFIXES.join(", ")
    )
}

fn intake_refusal(phase: &Value, blocked: &str, extra: &str) -> String {
    format!(
        "bee intake gate: no bee work is active (phase: {}) — {} is blocked. {}{}",
        js_disp(phase),
        blocked,
        extra,
        intake_fix_line()
    )
}

fn is_terminal_phase(phase: &Value) -> bool {
    matches!(phase, Value::String(s) if s == "idle" || s == "compounding-complete")
}

fn is_gated_phase(phase: &Value) -> bool {
    matches!(phase, Value::String(s) if s == "exploring" || s == "planning")
}

// ─── write-record resolution (provenance: guards.mjs resolveWriteRecord /
// resolveWriteTopology; state.mjs resolvePipeline/readLane) ────────────────

struct Topo {
    ctx: JsCtx,
    control_root: String,
}

/// provenance: guards.mjs resolveWriteTopology.
fn resolve_write_topology(root: &str, control_root_override: Option<&str>) -> R<Topo> {
    let ctx = match resolve_context(root)? {
        CtxOutcome::Ok(c) => c,
        CtxOutcome::Threw => JsCtx::default(),
    };
    let over = control_root_override
        .map(js_trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let control_root = over
        .or_else(|| ctx.control_root.clone())
        .unwrap_or_else(|| root.to_string());
    Ok(Topo { ctx, control_root })
}

enum RecordResolution {
    Ok { record: Map<String, Value>, source: &'static str },
    Fail { reason: String },
}

/// provenance: guards.mjs resolveWriteRecord + state.mjs resolvePipeline.
fn resolve_write_record(
    control_root: &str,
    state: &Map<String, Value>,
    session_id: Option<&str>,
    emit: &mut Emit,
) -> R<RecordResolution> {
    let sid = match session_id.map(js_trim).filter(|s| !s.is_empty()) {
        Some(s) => s,
        None => {
            return Ok(RecordResolution::Ok { record: state.clone(), source: "default" });
        }
    };
    // resolvePipeline(controlRoot, { sessionId }).
    let control2 = control_root_for_state(control_root)?;
    let defaults = |_: &mut Emit| -> R<RecordResolution> {
        Ok(RecordResolution::Ok {
            record: read_state(Path::new(control_root))?,
            source: "default",
        })
    };
    let session = match read_session(&control2, sid)? {
        Some(s) => s,
        None => return defaults(emit),
    };
    let bound = match session.get("lane") {
        Some(Value::String(s)) if !js_trim(s).is_empty() => js_trim(s).to_string(),
        _ => return defaults(emit),
    };
    let session_id_disp = js_disp_opt(session.get("id"));
    // lanePath validity (state.mjs requireLaneFeature).
    if bound.contains('/') || bound.contains('\\') || bound.contains("..") {
        return Ok(RecordResolution::Fail {
            reason: format!(
                "bee lane guard: session \"{}\" is bound to lane \"{}\", which is not a valid lane name (lane feature must be a plain id (no path separators).) — never guessed back to the default pipeline. FIX: rebind or unbind the session (claims.mjs bindSessionLane/unbindSessionLane).",
                session_id_disp, bound
            ),
        });
    }
    let file = Path::new(&control2).join(".bee").join("lanes").join(format!("{}.json", bound));
    let file_s = file.to_string_lossy().into_owned();
    let rel_file = np_relative(&control2, &file_s)?;
    if !file.exists() {
        return Ok(RecordResolution::Fail {
            reason: format!(
                "bee lane guard: session \"{}\" is bound to lane \"{}\" but {} does not exist — resolution never guesses back to the default pipeline. FIX: start the lane (startFeature with lane mode) or unbind the session.",
                session_id_disp, bound, rel_file
            ),
        });
    }
    // readLane(control2, bound).
    let parsed = read_json_g(&file)?; // Corrupt JSON → Nd (readJson warn)
    let lane_record = parsed.and_then(|v| match v {
        Value::Object(m) if m.get("feature") == Some(&Value::String(bound.clone())) => Some(m),
        _ => None,
    });
    let record = match lane_record {
        None => {
            // readLane's corrupt-shape console.warn line, then LANE_CORRUPT.
            emit.stderr.push_str(&format!(
                "readLane: skipping corrupt lane record \"{}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {}\").\n",
                rel_file, rel_file
            ));
            return Ok(RecordResolution::Fail {
                reason: format!(
                    "bee lane guard: session \"{}\" is bound to lane \"{}\" but its record is corrupt — display never guesses and mutations must refuse. FIX: inspect/restore {}, then retry.",
                    session_id_disp, bound, rel_file
                ),
            });
        }
        Some(m) => m,
    };
    // laneRecordFrom merge over defaultLaneRecord.
    let mut merged = Map::new();
    merged.insert("schema_version".into(), Value::String("1.0".into()));
    merged.insert("feature".into(), Value::String(bound.clone()));
    merged.insert("mode".into(), Value::Null);
    merged.insert("phase".into(), Value::String("idle".into()));
    let mut gates = Map::new();
    for g in ["context", "shape", "execution", "review"] {
        gates.insert(g.into(), Value::Bool(false));
    }
    merged.insert("approved_gates".into(), Value::Object(gates.clone()));
    merged.insert("summary".into(), Value::String(String::new()));
    merged.insert("next_action".into(), Value::String(String::new()));
    merged.insert("created_at".into(), Value::Null);
    for (k, v) in &record {
        merged.insert(k.clone(), v.clone());
    }
    let mut merged_gates = gates;
    if let Some(Value::Object(over)) = record.get("approved_gates") {
        for (k, v) in over {
            merged_gates.insert(k.clone(), v.clone());
        }
    }
    merged.insert("approved_gates".into(), Value::Object(merged_gates));
    if merged.get("phase") == Some(&Value::String("validating".into())) {
        merged.insert("phase".into(), Value::String("planning".into()));
    }
    Ok(RecordResolution::Ok { record: merged, source: "lane" })
}

/// provenance: guards.mjs resolveHoldTopology.
fn resolve_hold_topology<'a>(ctx: &'a JsCtx, control_root: &'a str) -> Option<(String, String)> {
    ctx.workspace_root.as_ref()?;
    match &ctx.worktree_id {
        None => Some((control_root.to_string(), "main".to_string())),
        Some(wt) => match &ctx.workspace_id {
            Some(ws) if ws == wt => Some((control_root.to_string(), ws.clone())),
            _ => None,
        },
    }
}

/// provenance: guards.mjs resolveWritePolicyMode.
fn resolve_write_policy_mode(config: &Map<String, Value>) -> &'static str {
    let configured = match config.get("guards") {
        Some(Value::Object(g)) => match g.get("write_policy") {
            Some(Value::String(s)) => js_trim(s).to_string(),
            _ => String::new(),
        },
        _ => String::new(),
    };
    match configured.as_str() {
        "observe" => "observe",
        "shared-disjoint" => "shared-disjoint",
        _ => "isolated",
    }
}

enum Ownership {
    Open,
    Corrupt,
    Blocked { owner: String },
}

/// provenance: guards.mjs checkWorkspaceOwnership.
fn check_workspace_ownership(control_root: &str, ctx: &JsCtx, session_id: &str) -> R<Ownership> {
    let workspace_id = ctx.workspace_id.clone().unwrap_or_else(|| "main".to_string());
    let workspace = match read_workspace(control_root, &workspace_id) {
        WorkspaceRead::Missing => return Ok(Ownership::Open),
        WorkspaceRead::Corrupt => return Ok(Ownership::Corrupt),
        WorkspaceRead::Ok(w) => w,
    };
    let owner = match workspace.get("write_owner_session") {
        Some(v) if truthy(v) => v.clone(),
        _ => return Ok(Ownership::Open),
    };
    if owner == Value::String(session_id.to_string()) {
        return Ok(Ownership::Open);
    }
    let owner_str = match &owner {
        Value::String(s) => s.clone(),
        _ => {
            // readSession(controlRoot, non-string) → sessionPath requireId
            // throws → caught → null → not live → never blocks.
            return Ok(Ownership::Open);
        }
    };
    let owner_session = read_session(control_root, &owner_str)?;
    let live = match owner_session {
        Some(s) => !heartbeat_stale(&s, now_ms())?,
        None => false,
    };
    if !live {
        return Ok(Ownership::Open);
    }
    Ok(Ownership::Blocked { owner: owner_str })
}

// ─── checkWrite (provenance: guards.mjs checkWrite, step for step) ─────────

enum WV {
    Allow,
    AllowWarn(String),
    Deny(String),
}

#[allow(clippy::too_many_arguments)]
fn check_write(
    root: &str,
    state: &Map<String, Value>,
    rel_path: &str,
    agent_name: Option<&str>,
    session_id: Option<&str>,
    control_root_override: Option<&str>,
    emit: &mut Emit,
) -> R<WV> {
    let normalized = normalize_rel(rel_path);

    if let Some(verb) = direct_edit_verb(&normalized) {
        return Ok(WV::Deny(format!(
            "bee direct-edit guard: \"{}\" is CLI-owned — direct edits are blocked in every phase. \
Hand-edited state files reintroduce schema drift (the exact class the CLI validates away). \
FIX: use {} instead of editing this file directly.",
            normalized, verb
        )));
    }

    if let Some(ext) = docs_history_code_deny(&normalized) {
        return Ok(WV::Deny(format!(
            "bee docs-history guard: \"{}\" writes a \"{}\" code file into docs/history/, which is \
the tech-agnostic KNOWLEDGE layer (.md only — CONTEXT.md, plan.md, reports, walkthrough). Code never lives there. \
FIX: put a persistent verify/helper script in the project's own scripts (committed with the product) and point \
the cell's verify command at it; put a disposable proof in .bee/spikes/<feature>/. Never docs/history.",
            normalized, ext
        )));
    }

    if let Some(kind) = scratch_shape_deny(&normalized) {
        return Ok(WV::Deny(format!(
            "bee scratch-shape guard: \"{}\" looks like {} landing in a tracked directory. \
Every ephemeral file bee writes for its own working purposes belongs in .bee/tmp/<feature-or-session>/ \
(feasibility code in .bee/spikes/<feature>/), never a tracked path (docs/specs/doctrine-layer.md). \
FIX: write it to .bee/tmp/ instead (or .bee/spikes/ for a feasibility proof), and let `bee tmp sweep` clear it later.",
            normalized, kind
        )));
    }

    let topo = resolve_write_topology(root, control_root_override)?;
    let ctx = &topo.ctx;
    let control_root = topo.control_root.clone();

    let (record, source) = match resolve_write_record(&control_root, state, session_id, emit)? {
        RecordResolution::Fail { reason } => return Ok(WV::Deny(reason)),
        RecordResolution::Ok { record, source } => (record, source),
    };

    // Cross-session hold deny (fsh-7 D3) — sessionId-gated.
    if let Some(sid) = session_id.map(js_trim).filter(|s| !s.is_empty()) {
        if reservation_store_corrupt(root) {
            let res_rel = np_relative(
                root,
                &Path::new(root).join(".bee").join("reservations.json").to_string_lossy(),
            )?;
            return Ok(WV::Deny(format!(
                "bee hold guard: the reservation store ({}) is present but \
unreadable/corrupt — failing closed for a session-aware write rather than silently treating it as empty. \
FIX: inspect/restore the reservation store, then retry.",
                res_rel
            )));
        }
        let hold_conflicts = find_session_conflicts(root, sid, &[normalized.clone()])?;
        if !hold_conflicts.is_empty() {
            let acting_workspace = ctx.workspace_id.clone().unwrap_or_else(|| "main".to_string());
            let mut same_workspace: Vec<&Resv> = Vec::new();
            for holder in &hold_conflicts {
                let holder_session = holder.session.clone().unwrap_or(Value::Null);
                if session_workspace_id(&control_root, &holder_session)? == acting_workspace {
                    same_workspace.push(holder);
                }
            }
            if let Some(holder) = same_workspace.first() {
                return Ok(WV::Deny(format!(
                    "bee cross-session hold: \"{}\" is held by session \"{}\" (agent {}, cell {}), {}. \
Wait for the hold to expire or coordinate with that session — a cross-session hold is a hard block (D3).",
                    normalized,
                    js_disp_opt(holder.session.as_ref()),
                    js_disp_opt(holder.agent.as_ref()),
                    js_disp_opt(holder.cell.as_ref()),
                    hold_expiry(holder)?
                )));
            }
        }
    }

    // Cross-worktree foreign-hold consultation (xwh-4 / msn-14 D4).
    if let Some((main_root, holder_id)) = resolve_hold_topology(ctx, &control_root) {
        if holds_store_corrupt(&main_root) {
            return Ok(WV::Deny(
                "bee cross-worktree hold guard: the shared holds ledger (.bee/runtime/cross-worktree-holds.json \
in the main checkout) is present but unreadable/corrupt — failing closed rather than silently \
treating it as empty. FIX: inspect/restore the ledger in the main checkout, then retry."
                    .to_string(),
            ));
        }
        let foreign = find_foreign_holds(&main_root, &holder_id, &[normalized.clone()])?;
        if let Some(hold) = foreign.first() {
            let feature_disp = match hold.get("feature") {
                Some(v) if truthy(v) => js_disp(v),
                _ => "unknown".to_string(),
            };
            let cell_clause = match hold.get("cell") {
                Some(v) if truthy(v) => format!(", cell {}", js_disp(v)),
                _ => String::new(),
            };
            if is_exclusive_path(Path::new(root), &normalized)? {
                return Ok(WV::Deny(format!(
                    "bee cross-worktree hold: \"{}\" is held by checkout \"{}\" (feature {}{}), {}. \
Wait for the hold to expire or coordinate with that checkout — a cross-worktree hold is a hard block.",
                    normalized,
                    js_disp_opt(hold.get("holder")),
                    feature_disp,
                    cell_clause,
                    foreign_hold_expiry(hold)?
                )));
            }
            return Ok(WV::AllowWarn(format!(
                "bee cross-worktree hold: \"{}\" is also held by checkout \"{}\" (feature {}{}), {} — \
advisory only (different workspace, not an exclusive resource). \
Coordinate with that checkout if possible; otherwise \"bee worktree merge\" will surface any real conflict \
between the two checkouts at merge time.",
                normalized,
                js_disp_opt(hold.get("holder")),
                feature_disp,
                cell_clause,
                foreign_hold_expiry(hold)?
            )));
        }
    }

    // phase = record?.phase || 'idle'
    let phase = match record.get("phase") {
        Some(v) if truthy(v) => v.clone(),
        _ => Value::String("idle".into()),
    };

    // Workspace-ownership deny (msn-21, class (c)).
    if let Some(sid) = session_id.map(js_trim).filter(|s| !s.is_empty()) {
        if source == "default" && phase != Value::String("swarming".into()) {
            let config = read_config(Path::new(&control_root))?;
            if resolve_write_policy_mode(&config) == "isolated" {
                match check_workspace_ownership(&control_root, ctx, sid)? {
                    Ownership::Open => {}
                    Ownership::Corrupt => {
                        let ws_id = ctx.workspace_id.clone().unwrap_or_else(|| "main".to_string());
                        let ws_file = Path::new(&control_root)
                            .join(".bee")
                            .join("runtime")
                            .join("workspaces")
                            .join(format!("{}.json", ws_id));
                        let ws_rel = np_relative(&control_root, &ws_file.to_string_lossy())?;
                        return Ok(WV::Deny(format!(
                            "bee workspace-ownership guard: the workspace record for \"{}\" ({}) is present but \
unreadable/corrupt — failing closed for a session-aware write rather than silently treating it as \
unowned. FIX: inspect/restore the workspace record, then retry.",
                            ws_id, ws_rel
                        )));
                    }
                    Ownership::Blocked { owner } => {
                        let ws_id = ctx.workspace_id.clone().unwrap_or_else(|| "main".to_string());
                        return Ok(WV::Deny(format!(
                            "bee write-policy: workspace \"{}\" is write-owned by session \"{}\" \
— a second write-capable session defaults to isolation, never a shared write into the same checkout. \
FIX: coordinate with that session, wait for its heartbeat to go stale, or start your own feature with \
`bee.mjs state start-feature --isolate` (or set guards.auto_isolate to true in .bee/config.json) to work \
in a fresh worktree instead.",
                            ws_id, owner
                        )));
                    }
                }
            }
        }
    }

    if is_terminal_phase(&phase) {
        let config = read_config(Path::new(&control_root))?;
        let idle_gate_on = !matches!(
            config.get("guards"),
            Some(g) if truthy(g) && g.get("idle_gate") == Some(&Value::Bool(false))
        );
        if idle_gate_on && !under_allowed_prefix(&normalized) {
            return Ok(WV::Deny(intake_refusal(
                &phase,
                &format!("writing \"{}\"", normalized),
                "",
            )));
        }
        return Ok(WV::Allow);
    }

    if is_gated_phase(&phase) {
        let execution_approved = record
            .get("approved_gates")
            .and_then(|g| g.get("execution"))
            == Some(&Value::Bool(true));
        if !execution_approved && !under_allowed_prefix(&normalized) {
            return Ok(WV::Deny(format!(
                "bee gate: phase is \"{}\" and gate \"execution\" is not approved — \
writing \"{}\" is blocked. Allowed now: {}. \
Get execution approval (bee-hive) before touching source files.",
                js_disp(&phase),
                normalized,
                GATE_ALLOWED_PREFIXES.join(", ")
            )));
        }
        return Ok(WV::Allow);
    }

    if phase == Value::String("swarming".into()) {
        let env_agent = std::env::var("BEE_AGENT_NAME").ok().filter(|s| !s.is_empty());
        let agent = agent_name
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or(env_agent);
        if let Some(agent) = agent {
            let conflicts = find_conflicts(root, &agent, &[normalized.clone()])?;
            if !conflicts.is_empty() {
                let hard: Vec<&Resv> =
                    conflicts.iter().filter(|c| is_hard_conflict(c, &normalized)).collect();
                if !hard.is_empty() {
                    let held = hard
                        .iter()
                        .map(|c| {
                            format!(
                                "{} holds \"{}\" (cell {})",
                                js_disp_opt(c.agent.as_ref()),
                                c.path,
                                js_disp_opt(c.cell.as_ref())
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; ");
                    return Ok(WV::Deny(format!(
                        "bee reservation conflict: \"{}\" is reserved by another agent — {}. \
Reserve the path first or return [BLOCKED] to the orchestrator.",
                        normalized, held
                    )));
                }
                let warned = conflicts
                    .iter()
                    .map(|c| {
                        format!(
                            "{}'s declared intent \"{}\" (cell {}) covers \"{}\"",
                            js_disp_opt(c.agent.as_ref()),
                            c.path,
                            js_disp_opt(c.cell.as_ref()),
                            normalized
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                return Ok(WV::AllowWarn(format!(
                    "bee reservation intent: {} — advisory only (kind: intent), not a hard block.",
                    warned
                )));
            }
        }
        return Ok(WV::Allow);
    }

    if !is_known_phase(&phase) {
        return Ok(WV::Deny(format!(
            "bee phase guard: phase \"{}\" is not a recognized phase — writing \"{}\" is refused rather \
than silently allowed through an unhandled state. FIX: restore a valid phase (bee state set), or if this \
is a legitimate new phase, add explicit dispatch for it in checkWrite.",
            js_disp(&phase),
            normalized
        )));
    }

    Ok(WV::Allow)
}

// ─── checkGitBashCommand (provenance: guards.mjs, full port) ───────────────

#[allow(clippy::too_many_arguments)]
fn check_git_bash_command(
    root: &str,
    state: &Map<String, Value>,
    command: &str,
    cwd: &str,
    session_id: Option<&str>,
    control_root_override: Option<&str>,
    emit: &mut Emit,
) -> R<Option<WV>> {
    let topo = resolve_write_topology(root, control_root_override)?;
    let record = match resolve_write_record(&topo.control_root, state, session_id, emit)? {
        RecordResolution::Fail { reason } => return Ok(Some(WV::Deny(reason))),
        RecordResolution::Ok { record, .. } => record,
    };
    let phase = match record.get("phase") {
        Some(v) if truthy(v) => v.clone(),
        _ => Value::String("idle".into()),
    };

    let tokens = tokenize(command);
    let invocation = match find_git_invocation(&tokens) {
        None => return Ok(None),
        Some(inv) => inv,
    };
    let subcommand = invocation.subcommand.clone();
    let rest = invocation.rest;

    // gc-2: concurrent-worker whole-tree denial — phase-independent.
    if let Some(classified) = classify_concurrent_tree_verb(subcommand.as_deref(), &rest) {
        match resolve_live_worker_count(root, &topo.control_root, &topo.ctx)? {
            WorkerCount::Unresolved(reason) => {
                return Ok(Some(WV::Deny(concurrent_tree_refusal(
                    &classified.verb,
                    classified.why,
                    &format!(
                        "the live-worker count could not be resolved ({}), which is treated as more than one worker",
                        reason
                    ),
                ))));
            }
            WorkerCount::Resolved(count) => {
                if count > 1 {
                    return Ok(Some(WV::Deny(concurrent_tree_refusal(
                        &classified.verb,
                        classified.why,
                        &format!("{} workers are live in this checkout", count),
                    ))));
                }
            }
        }
    }

    if !is_terminal_phase(&phase) {
        return Ok(None);
    }

    let config = read_config(Path::new(&topo.control_root))?;
    let idle_gate_on = !matches!(
        config.get("guards"),
        Some(g) if truthy(g) && g.get("idle_gate") == Some(&Value::Bool(false))
    );
    if !idle_gate_on {
        return Ok(None);
    }

    const READONLY: [&str; 12] = [
        "status", "log", "diff", "show", "rev-parse", "ls-files", "check-ignore", "merge-base",
        "rev-list", "describe", "blame", "cat-file",
    ];
    if let Some(sub) = &subcommand {
        if READONLY.contains(&sub.as_str()) {
            return Ok(None); // { allow: true } — no denial
        }
        let flag_gated: Option<&[&str]> = match sub.as_str() {
            "branch" | "tag" => Some(&["--list"]),
            "remote" => Some(&["-v", "--verbose"]),
            _ => None,
        };
        if let Some(flags) = flag_gated {
            if rest.iter().any(|t| flags.contains(&t.as_str())) {
                return Ok(None);
            }
        }
    }

    if subcommand.as_deref() == Some("push") {
        return Ok(Some(WV::Deny(intake_refusal(
            &phase,
            "`git push`",
            "git push is outward-facing and is never exempted from this gate, regardless of what it would push. ",
        ))));
    }

    const MUTATING: [&str; 15] = [
        "commit", "add", "rm", "mv", "checkout", "restore", "tag", "merge", "reset", "stash",
        "clean", "apply", "cherry-pick", "revert", "rebase",
    ];
    const PATH_RESOLVABLE: [&str; 6] = ["commit", "add", "rm", "mv", "checkout", "restore"];
    if let Some(sub) = &subcommand {
        if MUTATING.contains(&sub.as_str()) {
            let resolved = if PATH_RESOLVABLE.contains(&sub.as_str()) {
                resolve_git_mutation_paths(cwd, sub, &rest)
            } else {
                None
            };
            let paths = match resolved {
                None => {
                    return Ok(Some(WV::Deny(intake_refusal(
                        &phase,
                        &format!(
                            "running `git {}` (its changed paths could not be proved bookkeeping-only)",
                            sub
                        ),
                        "",
                    ))));
                }
                Some(p) => p,
            };
            let offending = paths
                .iter()
                .map(|p| normalize_rel(p))
                .find(|p| !under_allowed_prefix(p));
            if let Some(off) = offending {
                return Ok(Some(WV::Deny(intake_refusal(
                    &phase,
                    &format!("running `git {}` — it would change \"{}\"", sub, off),
                    "",
                ))));
            }
            return Ok(None); // { allow: true, kind: 'git-bookkeeping' }
        }
    }

    let named = match &subcommand {
        Some(s) => s.clone(),
        None => js_trim(command).to_string(),
    };
    Ok(Some(WV::Deny(intake_refusal(
        &phase,
        &format!("running `git {}`", named),
        "This git subcommand is not recognized as read-only or as a modeled bookkeeping-eligible mutation, so it is refused rather than assumed safe. ",
    ))))
}

// ─── hook-local helpers (provenance: bee-write-guard.mjs top half) ─────────

const GENERIC_CONTAINMENT_MESSAGE: &str =
    "bee write guard denied this target: it could not be canonically contained inside the physical worktree. \
FIX: use a plain in-worktree path without traversal, outside absolute paths, or symlink escapes.";
const GENERIC_BASH_CONTAINMENT_MESSAGE: &str =
    "bee write guard denied Bash: one or more extracted targets could not be canonically contained inside the physical worktree. \
FIX: use plain in-worktree paths without traversal, outside absolute paths, or symlink escapes.";

/// provenance: bee-write-guard.mjs HOME_PREFIXED_TARGET_RE /
/// isHomePrefixedTarget (gmr-1).
fn is_home_prefixed(raw: &str) -> bool {
    let tail_after = |prefix: &str| -> Option<char> {
        raw.strip_prefix(prefix).and_then(|rest| rest.chars().next())
    };
    if let Some(rest) = raw.strip_prefix('~') {
        // ~[A-Za-z0-9._+-]* then a separator
        let mut chars = rest.chars();
        let mut c = chars.next();
        while let Some(ch) = c {
            if ch == '/' || ch == '\\' {
                return true;
            }
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '+' | '-') {
                c = chars.next();
                continue;
            }
            return false;
        }
        return false;
    }
    if matches!(tail_after("$HOME"), Some('/') | Some('\\')) {
        return true;
    }
    if matches!(tail_after("${HOME}"), Some('/') | Some('\\')) {
        return true;
    }
    false
}

/// provenance: bee-write-guard.mjs normalizeToolPath — replace(/\\(?!\s)/g,
/// path.sep): identity on Windows; on POSIX a backslash not followed by JS
/// whitespace becomes '/'.
fn normalize_tool_path(raw: &str) -> String {
    if cfg!(windows) {
        return raw.to_string();
    }
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c == '\\' {
            match chars.get(i + 1) {
                Some(&n) if js_is_ws(n) => out.push('\\'),
                _ => out.push('/'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// provenance: bee-write-guard.mjs lexicalRelPath.
fn lexical_rel_path(root: &str, cwd: &str, raw: Option<&Value>) -> R<Option<String>> {
    let raw_s = match raw {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => return Ok(None),
    };
    np_check_modelable(&raw_s)?;
    let base = if !cwd.is_empty() { cwd } else { root };
    let abs = if np_is_absolute(&raw_s) {
        np_resolve1(&raw_s)?
    } else {
        np_resolve2(base, &raw_s)?
    };
    let rel = np_relative(root, &abs)?;
    if rel.is_empty() || rel == "." || rel.starts_with("..") || np_is_absolute(&rel) {
        return Ok(None);
    }
    Ok(Some(rel.split(SEP).collect::<Vec<_>>().join("/")))
}

/// provenance: bee-write-guard.mjs canonicalRelPath.
fn canonical_rel_path(root: &str, cwd: &str, raw: Option<&Value>) -> R<Option<String>> {
    let raw_s = match raw {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => return Ok(None),
    };
    if is_home_prefixed(&raw_s) {
        return Ok(None);
    }
    let root_real = match realpath_any(root) {
        Some(r) => r,
        None => return Ok(None),
    };
    let normalized = normalize_tool_path(&raw_s);
    #[cfg(not(windows))]
    {
        // Foreign Windows spellings on a POSIX host cannot be safely mapped.
        let b = raw_s.as_bytes();
        let win_drive = b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/');
        if win_drive || raw_s.starts_with("\\\\") {
            return Ok(None);
        }
    }
    if !np_is_absolute(&normalized)
        && normalized.split(SEP).any(|s| s == "..")
    {
        return Ok(None);
    }
    np_check_modelable(&normalized)?;
    let cwd_base = if np_is_absolute(cwd) { cwd } else { root_real.as_str() };
    let lexical = if np_is_absolute(&normalized) {
        np_resolve1(&normalized)?
    } else {
        np_resolve2(cwd_base, &normalized)?
    };
    let (ancestor, unresolved) = match walk_existing_ancestor(&lexical) {
        Some(x) => x,
        None => return Ok(None),
    };
    let ancestor_real = match realpath_any(&ancestor) {
        Some(r) => r,
        None => return Ok(None),
    };
    let canonical = np_resolve_segments(&ancestor_real, &unresolved)?;
    let rel = np_relative(&root_real, &canonical)?;
    if rel.is_empty()
        || rel == "."
        || rel == ".."
        || rel.starts_with(&format!("..{}", SEP))
        || np_is_absolute(&rel)
    {
        return Ok(None);
    }
    Ok(Some(rel.split(SEP).collect::<Vec<_>>().join("/")))
}

/// provenance: bee-write-guard.mjs resolveTargetRealpath (catch-all flavor).
fn resolve_target_realpath(cwd: &str, root: &str, raw: &Value) -> R<Option<String>> {
    let raw_s = match raw {
        Value::String(s) if !s.is_empty() => s.clone(),
        _ => return Ok(None),
    };
    if is_home_prefixed(&raw_s) {
        return Ok(None);
    }
    let normalized = normalize_tool_path(&raw_s);
    #[cfg(not(windows))]
    {
        let b = raw_s.as_bytes();
        let win_drive = b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/');
        if win_drive || raw_s.starts_with("\\\\") {
            return Ok(None);
        }
    }
    np_check_modelable(&normalized)?;
    let cwd_base = if np_is_absolute(cwd) { cwd } else { root };
    let lexical = if np_is_absolute(&normalized) {
        np_resolve1(&normalized)?
    } else {
        np_resolve2(cwd_base, &normalized)?
    };
    let (ancestor, unresolved) = match walk_existing_ancestor(&lexical) {
        Some(x) => x,
        None => return Ok(None),
    };
    let ancestor_real = match realpath_any(&ancestor) {
        Some(r) => r,
        None => return Ok(None),
    };
    Ok(Some(np_resolve_segments(&ancestor_real, &unresolved)?))
}

/// provenance: bee-write-guard.mjs lexicalAbsTarget (wcg-2 D1b).
fn lexical_abs_target(root: &str, cwd: &str, raw: &str) -> R<String> {
    let normalized = normalize_tool_path(raw);
    np_check_modelable(&normalized)?;
    let cwd_base = if np_is_absolute(cwd) { cwd } else { root };
    if np_is_absolute(&normalized) {
        np_resolve1(&normalized)
    } else {
        np_resolve2(cwd_base, &normalized)
    }
}

/// provenance: bee-write-guard.mjs isUnderRoot.
fn is_under_root(parent_real: &str, child_real: &str) -> R<bool> {
    if parent_real.is_empty() || child_real.is_empty() {
        return Ok(false);
    }
    let rel = np_relative(parent_real, child_real)?;
    Ok(rel.is_empty()
        || (rel != ".." && !rel.starts_with(&format!("..{}", SEP)) && !np_is_absolute(&rel)))
}

/// provenance: bee-write-guard.mjs readGitdirPointer (catch-all flavor).
fn read_gitdir_pointer(file: &Path, base: &str) -> R<Option<String>> {
    let raw = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };
    let mut raw = js_trim(&raw);
    if let Some(rest) = raw.strip_prefix("gitdir:") {
        raw = js_trim(rest);
    }
    if raw.is_empty() {
        return Ok(None);
    }
    let fixed = if cfg!(windows) { raw.to_string() } else { raw.replace('\\', "/") };
    Ok(Some(np_resolve2(base, &fixed)?))
}

/// provenance: bee-write-guard.mjs deriveCurrentWorktree.
fn derive_current_worktree(root: &str) -> R<Option<(String, String)>> {
    let marker = Path::new(root).join(".git");
    let stat = match std::fs::metadata(&marker) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };
    if !stat.is_file() {
        return Ok(None);
    }
    let gitdir = match read_gitdir_pointer(&marker, root)? {
        Some(g) => g,
        None => return Ok(None),
    };
    let worktrees_root = np_resolve2(&gitdir, "..")?;
    let common_git_dir = np_resolve2(&worktrees_root, "..")?;
    if np_basename(&worktrees_root) != "worktrees" || np_basename(&common_git_dir) != ".git" {
        return Ok(None);
    }
    let main_root = match realpath_any(&np_dirname(&common_git_dir)) {
        Some(m) => m,
        None => return Ok(None),
    };
    Ok(Some((main_root, np_basename(&gitdir))))
}

/// provenance: bee-write-guard.mjs resolveGrantedWorktreeRoot.
fn resolve_granted_worktree_root(main_root: &str, id: &str) -> R<Option<String>> {
    let gwd = Path::new(main_root).join(".git").join("worktrees").join(id);
    match std::fs::metadata(&gwd) {
        Ok(s) if s.is_dir() => {}
        _ => return Ok(None),
    }
    let gwd_s = gwd.to_string_lossy().into_owned();
    let forward = match read_gitdir_pointer(&gwd.join("gitdir"), &gwd_s)? {
        Some(f) => f,
        None => return Ok(None),
    };
    let worktree_root = np_dirname(&forward);
    let reverse = match read_gitdir_pointer(&Path::new(&worktree_root).join(".git"), &worktree_root)? {
        Some(r) => r,
        None => return Ok(None),
    };
    if np_resolve1(&reverse)? != np_resolve1(&gwd_s)? {
        return Ok(None);
    }
    Ok(realpath_any(&worktree_root))
}

/// provenance: bee-write-guard.mjs readGrantedWorktreeIds.
fn read_granted_worktree_ids(main_root: &str) -> Vec<String> {
    let file = Path::new(main_root)
        .join(".bee")
        .join("runtime")
        .join("worktree-grants.json");
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(m)) => m
            .iter()
            .filter(|(_, v)| **v == Value::Bool(true))
            .map(|(k, _)| k.clone())
            .collect(),
        _ => Vec::new(),
    }
}

/// provenance: bee-write-guard.mjs describeCrossWorktreeTarget (message-only
/// enrichment; every failure keeps the generic containment message).
fn describe_cross_worktree_target(root: &str, cwd: &str, raw: &Value) -> R<Option<String>> {
    let target_real = match resolve_target_realpath(cwd, root, raw)? {
        Some(t) => t,
        None => return Ok(None),
    };
    let current = derive_current_worktree(root)?;
    let main_root = match &current {
        Some((m, _)) => m.clone(),
        None => match realpath_any(root) {
            Some(m) => m,
            None => return Ok(None),
        },
    };
    if current.is_some() && is_under_root(&main_root, &target_real)? {
        return Ok(Some(
            "bee write guard denied this target: it could not be canonically contained inside the physical worktree — \
this path belongs to the main checkout, not this worktree. FIX: run this from a session rooted there."
                .to_string(),
        ));
    }
    for id in read_granted_worktree_ids(&main_root) {
        if let Some((_, cur_id)) = &current {
            if &id == cur_id {
                continue;
            }
        }
        if let Some(worktree_root) = resolve_granted_worktree_root(&main_root, &id)? {
            if is_under_root(&worktree_root, &target_real)? {
                return Ok(Some(format!(
                    "bee write guard denied this target: it could not be canonically contained inside the physical worktree — \
it resolves inside worktree \"{id}\". FIX: open a session with cwd={worktree_root} to work there, or merge it \
back from main via `bee worktree merge --id {id}`."
                )));
            }
        }
    }
    Ok(None)
}

// ─── worktree-first refusal (provenance: bee-write-guard.mjs §worktree-first,
// docs/specs/worktree-first.md §2) ─────────────────────────────────────────

/// provenance: bee-write-guard.mjs readWorktreeRecordedFeature (plain
/// try/catch parses — no readJson warn, so corrupt files just fall through).
fn read_worktree_recorded_feature(worktree_root: &str) -> Option<String> {
    let identity_file = Path::new(worktree_root)
        .join(".bee")
        .join("runtime")
        .join("worktree-identity.json");
    if let Ok(text) = std::fs::read_to_string(&identity_file) {
        if let Ok(Value::Object(m)) = serde_json::from_str::<Value>(&text) {
            if let Some(Value::String(f)) = m.get("feature") {
                if !f.is_empty() {
                    return Some(f.clone());
                }
            }
        }
    }
    let state_file = Path::new(worktree_root).join(".bee").join("state.json");
    if let Ok(text) = std::fs::read_to_string(&state_file) {
        if let Ok(Value::Object(m)) = serde_json::from_str::<Value>(&text) {
            if let Some(Value::String(f)) = m.get("feature") {
                if !f.is_empty() {
                    return Some(f.clone());
                }
            }
        }
    }
    None
}

/// provenance: bee-write-guard.mjs findFeatureWorktreeGrant.
fn find_feature_worktree_grant(main_root: &str, feature: &str) -> R<Option<(String, String)>> {
    for id in read_granted_worktree_ids(main_root) {
        if let Some(worktree_root) = resolve_granted_worktree_root(main_root, &id)? {
            if read_worktree_recorded_feature(&worktree_root).as_deref() == Some(feature) {
                return Ok(Some((id, worktree_root)));
            }
        }
    }
    Ok(None)
}

/// provenance: bee-write-guard.mjs worktreeFirstExemptRel.
fn worktree_first_exempt_rel(rel: &str) -> bool {
    if rel.is_empty() {
        return true;
    }
    if rel == "**" {
        return true;
    }
    if rel.ends_with(".md") {
        return true;
    }
    GATE_ALLOWED_PREFIXES.iter().any(|prefix| {
        if let Some(bare) = prefix.strip_suffix('/') {
            rel == bare || rel.starts_with(prefix)
        } else {
            rel == *prefix
        }
    })
}

/// provenance: bee-write-guard.mjs checkWorktreeFirstDenial.
fn check_worktree_first(
    worktree_resolution: &str,
    root: &str,
    store_root: &Path,
    state: &Map<String, Value>,
    rel_paths: &[String],
) -> R<Option<String>> {
    if worktree_resolution != "ordinary" {
        return Ok(None);
    }
    let feature = match state.get("feature") {
        Some(Value::String(f)) if !f.is_empty() => f.clone(),
        _ => return Ok(None),
    };
    let lane = match state.get("route") {
        Some(Value::Object(route)) => match route.get("lane") {
            Some(Value::String(l)) if !l.is_empty() => l.clone(),
            _ => return Ok(None),
        },
        _ => return Ok(None),
    };
    if lane == "docs" {
        return Ok(None);
    }
    let config = read_config(store_root)?;
    if config.get("worktree_first") == Some(&Value::String("off".into())) {
        return Ok(None);
    }
    let offender = match rel_paths.iter().find(|rel| !worktree_first_exempt_rel(rel)) {
        Some(o) => o.clone(),
        None => return Ok(None),
    };
    let main_root = match realpath_any(root) {
        Some(m) => m,
        None => return Ok(None),
    };
    let (grant_id, grant_root) = match find_feature_worktree_grant(&main_root, &feature)? {
        Some(g) => g,
        None => return Ok(None),
    };
    Ok(Some(format!(
        "bee worktree-first guard: \"{offender}\" is a feature source write in the MAIN checkout, but the active \
feature \"{feature}\" (lane \"{lane}\") holds granted worktree \"{grant_id}\" — code-touching feature work \
lives in its worktree from the start; main stays clean for integration, docs-lane, and release work \
(docs/specs/worktree-first.md). FIX: open your session at {grant_root} and make this edit there, \
then land it from main with `bee worktree merge --id {grant_id}`. Deliberate override: set \
worktree_first: \"off\" in .bee/config.json to disable this refusal (a recorded, visible choice)."
    )))
}

// ─── large-read guard (provenance: bee-write-guard.mjs router-cost rc-1) ───

fn resolve_max_read_lines(config: &Map<String, Value>) -> f64 {
    match config.get("guards") {
        Some(g) if truthy(g) => match g.get("max_read_lines") {
            Some(Value::Number(n)) => {
                let f = n.as_f64().unwrap_or(f64::NAN);
                if f.is_finite() && f > 0.0 { f } else { 800.0 }
            }
            _ => 800.0,
        },
        _ => 800.0,
    }
}

fn check_read_size_denial(abs: &Path, label: &str, threshold: f64) -> Option<String> {
    let stat = std::fs::metadata(abs).ok()?;
    if !stat.is_file() {
        return None;
    }
    if stat.len() > 25 * 1024 * 1024 {
        return None;
    }
    let buffer = std::fs::read(abs).ok()?;
    if buffer.iter().take(8000).any(|&b| b == 0) {
        return None;
    }
    let mut count = buffer.iter().filter(|&&b| b == 10).count();
    if !buffer.is_empty() && *buffer.last().unwrap() != 10 {
        count += 1;
    }
    if (count as f64) < threshold {
        return None;
    }
    Some(format!(
        "bee read-size guard: \"{label}\" is {count} lines (threshold: {}) and this Read \
has neither `offset` nor `limit` — reading it unbounded would load the whole file into context. \
FIX: pass `limit` (and optionally `offset`) to read a slice, or dispatch a `bee-extract` worker to read the whole file.",
        jsjson::js_f64_to_string(threshold)
    ))
}

// ─── companion mount / memory root delegation gates ────────────────────────

/// provenance: bee-write-guard.mjs resolveCompanionMountedRelPath — consulted
/// only for a target that already failed containment. A present marker means
/// live symlink verification the port does not replicate → Nd; an absent
/// marker is the .mjs's own catch → null.
fn companion_mount_rel(root: &str) -> R<Option<String>> {
    let marker = Path::new(root).join(".bee").join("companion-session.json");
    if marker.exists() {
        return Err(Nd);
    }
    Ok(None)
}

/// provenance: bee-write-guard.mjs isMemoryRootHit / isDeclaredMemoryRootTarget
/// (gmr-3) — a declared guards.memory_root (non-empty string) engages marker
/// verification the port does not replicate → Nd; anything else is false.
fn memory_root_hit(store_root: &Path) -> R<bool> {
    let config = read_config(store_root)?;
    match config.get("guards") {
        Some(Value::Object(g)) => match g.get("memory_root") {
            Some(Value::String(s)) if !js_trim(s).is_empty() => Err(Nd),
            _ => Ok(false),
        },
        _ => Ok(false),
    }
}

// ─── shared nested/companion checkout detection (provenance: guards.mjs
// wcg-1 isSharedNestedCheckoutTarget + helpers; F2 error posture: ENOENT is
// silent, any other fs error is a JS throw → Nd, which delegates to Node's
// typed fail-closed detection-error refusal) ───────────────────────────────

fn has_git_node_f2(dir: &str) -> R<bool> {
    match std::fs::metadata(Path::new(dir).join(".git")) {
        Ok(_) => Ok(true),
        Err(e) if io_err_is_enoent(&e) => Ok(false),
        Err(_) => Err(Nd),
    }
}

fn resolve_existing_realpath_f2(abs: &str) -> R<Option<String>> {
    let mut cursor = abs.to_string();
    let mut unresolved: Vec<String> = Vec::new();
    loop {
        match realpath_f2(&cursor)? {
            Some(real) => {
                return Ok(Some(if unresolved.is_empty() {
                    real
                } else {
                    np_resolve_segments(&real, &unresolved)?
                }));
            }
            None => {
                let parent = np_dirname(&cursor);
                if parent == cursor {
                    return Ok(None);
                }
                unresolved.insert(0, np_basename(&cursor));
                cursor = parent;
            }
        }
    }
}

fn resolve_verified_companion_mount_real(root: &str) -> R<Option<String>> {
    let marker_file = Path::new(root).join(".bee").join("companion-session.json");
    let raw = match std::fs::read(&marker_file) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) if io_err_is_enoent(&e) => return Ok(None),
        Err(_) => return Err(Nd), // F2: propagates in Node
    };
    let marker: Value = serde_json::from_str(&raw).map_err(|_| Nd)?; // F2: corrupt marker throws
    let obj = match &marker {
        Value::Object(m) => m,
        _ => return Ok(None),
    };
    let declared = match obj.get("worktreePath") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => return Ok(None),
    };
    let mount = match obj.get("mountPath") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => return Ok(None),
    };
    let declared_real = match realpath_f2(&declared)? {
        Some(d) => d,
        None => return Ok(None),
    };
    let live = Path::new(root).join(&mount).to_string_lossy().into_owned();
    let live_mount_real = match realpath_f2(&live)? {
        Some(l) => l,
        None => return Ok(None),
    };
    if declared_real != live_mount_real {
        return Ok(None);
    }
    Ok(Some(live_mount_real))
}

fn target_inside_verified_companion_mount(root: &str, abs_target: &str) -> R<bool> {
    let live = match resolve_verified_companion_mount_real(root)? {
        Some(l) => l,
        None => return Ok(false),
    };
    let target_real = match resolve_existing_realpath_f2(abs_target)? {
        Some(t) => t,
        None => return Ok(false),
    };
    is_under_root(&live, &target_real)
}

fn find_nested_checkout_dir(root_real: &str, abs_target: &str) -> R<Option<String>> {
    let mut cursor = abs_target.to_string();
    loop {
        let parent = np_dirname(&cursor);
        if parent == cursor {
            return Ok(None);
        }
        cursor = parent;
        let cursor_real = match realpath_f2(&cursor)? {
            Some(c) => c,
            None => continue, // does not exist yet — keep climbing
        };
        if cursor_real == root_real {
            return Ok(None);
        }
        if !is_under_root(root_real, &cursor_real)? {
            return Ok(None);
        }
        if has_git_node_f2(&cursor_real)? {
            return Ok(Some(cursor_real));
        }
    }
}

fn is_registered_submodule(root_real: &str, nested_real: &str) -> R<bool> {
    let content = match std::fs::read(Path::new(root_real).join(".gitmodules")) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) if io_err_is_enoent(&e) => return Ok(false),
        Err(_) => return Err(Nd),
    };
    for line in content.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        // /^\s*path\s*=\s*(.+?)\s*$/
        let after_ws = line.trim_start_matches(js_is_ws);
        let Some(after_path) = after_ws.strip_prefix("path") else { continue };
        let after_ws2 = after_path.trim_start_matches(js_is_ws);
        let Some(rest) = after_ws2.strip_prefix('=') else { continue };
        if rest.is_empty() {
            continue; // (.+?) needs at least one char
        }
        let cap = {
            let t = js_trim(rest);
            if t.is_empty() {
                // all-whitespace remainder: the lazy capture holds one ws char
                rest.chars().last().map(|c| c.to_string()).unwrap_or_default()
            } else {
                t.to_string()
            }
        };
        let entry = np_resolve2(root_real, &cap)?;
        if let Some(entry_real) = realpath_f2(&entry)? {
            if entry_real == nested_real {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// provenance: guards.mjs isSharedNestedCheckoutTarget (wcg-1/wcg-2,
/// Port-D4 controlRoot).
fn is_shared_nested_checkout_target(
    root: &str,
    abs_target: &str,
    exclude_session: Option<&str>,
    control_root: Option<&str>,
) -> R<bool> {
    let concurrency_root = control_root.filter(|s| !s.is_empty()).unwrap_or(root);
    if !is_concurrent_mode(concurrency_root, exclude_session, true)? {
        return Ok(false);
    }
    let root_real = match realpath_f2(root)? {
        Some(r) => r,
        None => return Ok(false),
    };
    if target_inside_verified_companion_mount(root, abs_target)? {
        return Ok(true);
    }
    if let Some(nested) = find_nested_checkout_dir(&root_real, abs_target)? {
        if !is_registered_submodule(&root_real, &nested)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// provenance: bee-write-guard.mjs sharedNestedCheckoutRefusal (wcg-2 D3/D4).
fn shared_nested_checkout_refusal(rel: &str) -> String {
    format!(
        "bee shared-checkout guard: \"{rel}\" is inside a nested checkout that another \
live session can also reach, and no verified companion mount covers it. \
Writing here can silently overwrite the other session's work — the exact \
failure this guard exists to prevent. \
FIX: open a FRESH companion worktree — run `bee worktree new --with-companion` \
to create a new worktree that mounts this shared checkout under a verified \
marker, then do this work there. The current worktree cannot be converted \
into a companion mount; you must create a new one."
    )
}

// ─── apply_patch extraction (provenance: bee-write-guard.mjs codex-parity) ─

fn apply_patch_text(tool_input: &Map<String, Value>) -> Option<String> {
    for key in ["input", "patch", "command"] {
        if let Some(Value::String(s)) = tool_input.get(key) {
            if s.contains("*** Begin Patch") {
                return Some(s.clone());
            }
        }
    }
    None
}

/// provenance: bee-write-guard.mjs PATCH_TARGET_RE + extractApplyPatchTargets.
fn extract_apply_patch_targets(patch_text: &str) -> Vec<String> {
    let mut targets = Vec::new();
    for line in patch_text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let Some(after_stars) = line.strip_prefix("***") else { continue };
        let after_ws = after_stars.trim_start_matches(js_is_ws);
        if after_ws.len() == after_stars.len() {
            continue; // \s+ requires at least one whitespace char
        }
        let mut matched = None;
        for verb in ["Add File", "Update File", "Delete File", "Move to"] {
            if let Some(rest) = after_ws.strip_prefix(verb) {
                if let Some(rest) = rest.strip_prefix(':') {
                    matched = Some(rest);
                    break;
                }
            }
        }
        let Some(rest) = matched else { continue };
        if rest.is_empty() {
            continue; // `\s*(.+?)\s*$` needs at least one char after ':'
        }
        targets.push(js_trim(rest).to_string());
    }
    targets
}

// ─── agent-name inference (provenance: bee-write-guard.mjs inferAgentName) ─

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// /\bBEE_AGENT_NAME=(["']?)([^"'\s]+)\1/ hand-rolled.
fn agent_name_from_command(command: &str) -> Option<String> {
    const NEEDLE: &str = "BEE_AGENT_NAME=";
    let chars: Vec<char> = command.chars().collect();
    let n: Vec<char> = NEEDLE.chars().collect();
    let mut i = 0usize;
    while i + n.len() <= chars.len() {
        if chars[i..i + n.len()] != n[..] {
            i += 1;
            continue;
        }
        // \b before 'B'
        if i > 0 && is_word_char(chars[i - 1]) {
            i += 1;
            continue;
        }
        let mut j = i + n.len();
        let quote = match chars.get(j) {
            Some(&q @ ('"' | '\'')) => {
                j += 1;
                Some(q)
            }
            _ => None,
        };
        let start = j;
        while j < chars.len() {
            let c = chars[j];
            if c == '"' || c == '\'' || js_is_ws(c) {
                break;
            }
            j += 1;
        }
        if j > start {
            let ok = match quote {
                Some(q) => chars.get(j) == Some(&q), // backreference
                None => true,                        // empty \1 always matches
            };
            if ok {
                return Some(chars[start..j].iter().collect());
            }
        }
        i += 1;
    }
    None
}

fn infer_agent_name(payload: &Map<String, Value>, tool_input: &Map<String, Value>) -> Option<String> {
    for key in ["agent_name", "agentName", "agent_nickname", "subagent_type"] {
        if let Some(s) = str_trim_nonempty(payload.get(key)) {
            return Some(s);
        }
    }
    let command = match tool_input.get("command") {
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    };
    if let Some(m) = agent_name_from_command(&command) {
        return Some(m);
    }
    match std::env::var("BEE_AGENT_NAME") {
        Ok(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

// ─── AskUserQuestion (provenance: guards.mjs checkAskUserQuestion +
// bee-write-guard.mjs ask-guard-autofix emission) ──────────────────────────

enum AskResult {
    Allow,
    Deny(String),
    Fixed { fixed: Value, notes: Vec<String> },
}

fn check_ask_user_question(tool_input: &Map<String, Value>) -> R<AskResult> {
    let questions = match tool_input.get("questions") {
        Some(Value::Array(a)) => a.clone(),
        _ => return Ok(AskResult::Allow),
    };
    let n = questions.len();
    if n < 1 || n > 4 {
        return Ok(AskResult::Deny(format!(
            "bee AskUserQuestion guard: {n} question(s) — the tool takes 1–4 per call. Split into separate calls."
        )));
    }
    let mut fixes: Vec<(usize, String)> = Vec::new();
    for (i, q) in questions.iter().enumerate() {
        let q = match q {
            Value::Object(m) => m,
            _ => continue,
        };
        let where_ = if n > 1 { format!(" (question {})", i + 1) } else { String::new() };
        if let Some(Value::String(h)) = q.get("header") {
            if utf16_len(h) > 12 {
                fixes.push((i, h.clone()));
            }
        }
        if let Some(Value::Array(options)) = q.get("options") {
            let on = options.len();
            if on < 2 || on > 4 {
                return Ok(AskResult::Deny(format!(
                    "bee AskUserQuestion guard: {on} option(s){where_} — each question needs 2–4 options (an \"Other\" free-text choice is added automatically). Fold overflow into a follow-up question."
                )));
            }
            for (j, o) in options.iter().enumerate() {
                let o = match o {
                    Value::Object(m) => m,
                    _ => continue,
                };
                let label = match o.get("label") {
                    Some(Value::String(l)) if !js_trim(l).is_empty() => l.clone(),
                    _ => {
                        return Ok(AskResult::Deny(format!(
                            "bee AskUserQuestion guard: option {}{} is missing a non-empty \"label\". Every option needs a label and a description.",
                            j + 1,
                            where_
                        )));
                    }
                };
                match o.get("description") {
                    Some(Value::String(d)) if !js_trim(d).is_empty() => {}
                    _ => {
                        return Ok(AskResult::Deny(format!(
                            "bee AskUserQuestion guard: option \"{label}\"{where_} is missing a non-empty \"description\". Every option needs a label and a description."
                        )));
                    }
                }
            }
        }
    }
    if fixes.is_empty() {
        return Ok(AskResult::Allow);
    }
    let mut fixed = Value::Object(tool_input.clone());
    let mut notes = Vec::new();
    for (idx, old) in fixes {
        if !old.is_ascii() {
            // UTF-16 slice(0,11)+trimEnd parity for non-ASCII headers is
            // unproven (surrogate-pair splits) — delegate.
            return Err(Nd);
        }
        let truncated = js_trim_end(&old[..11]);
        let new_header = format!("{truncated}…");
        fixed["questions"][idx]["header"] = Value::String(new_header.clone());
        notes.push(format!(
            "header \"{}\" ({} chars) → \"{}\"",
            old,
            utf16_len(&old),
            new_header
        ));
    }
    Ok(AskResult::Fixed { fixed, notes })
}

// ─── CLI-shape / internals-reach delegation detectors ──────────────────────

/// True when a token's basename matches LEGACY_HELPER_RE (^bee_[a-z]+\.mjs$/i)
/// or DISPATCHER_RE (^bee\.mjs$/i) — checkCliShape would then resolve against
/// the command registry, which is not ported → the caller delegates.
fn has_bee_cli_token(command: &str) -> bool {
    for token in tokenize(command) {
        let base = token.replace('\\', "/");
        let base = base.rsplit('/').next().unwrap_or("");
        let lower = base.to_lowercase();
        if lower == "bee.mjs" {
            return true;
        }
        if let Some(mid) = lower.strip_prefix("bee_").and_then(|r| r.strip_suffix(".mjs")) {
            if !mid.is_empty() && mid.chars().all(|c| c.is_ascii_alphabetic()) {
                return true;
            }
        }
    }
    false
}

/// True when a shell segment invokes node/nodejs with an inline-eval script
/// (-e/--eval/-p or --eval=…) — checkBinLibImportBashCommand's regex scan is
/// not ported → the caller delegates.
fn has_node_inline_eval(command: &str) -> bool {
    let tokens = tokenize(command);
    let mut i = 0usize;
    while i < tokens.len() {
        if is_separator(&tokens[i]) {
            i += 1;
            continue;
        }
        let mut end = i;
        while end < tokens.len() && !is_separator(&tokens[end]) {
            end += 1;
        }
        let segment = &tokens[i..end];
        i = end;
        let cmd = segment
            .first()
            .map(|t| t.replace('\\', "/"))
            .unwrap_or_default();
        let cmd = cmd.rsplit('/').next().unwrap_or("");
        if cmd != "node" && cmd != "nodejs" {
            continue;
        }
        for (k, token) in segment.iter().enumerate() {
            if matches!(token.as_str(), "-e" | "--eval" | "-p") {
                if segment.get(k + 1).is_some() {
                    return true;
                }
            }
            if token.starts_with("--eval=") {
                return true;
            }
        }
    }
    false
}

// ─── buffered emission ─────────────────────────────────────────────────────

#[derive(Default)]
struct Emit {
    stdout: String,
    stderr: String,
    code: u8,
    /// Deferred coverage-gap log lines: (root, gap, detail, ts).
    gaps: Vec<(PathBuf, &'static str, String, String)>,
}

fn push_gap(emit: &mut Emit, root: &Path, gap: &'static str, detail: String) {
    emit.gaps.push((root.to_path_buf(), gap, detail, now_iso()));
}

const DETAIL_MAX: usize = 300;

fn flush(emit: Emit, source: Option<&str>) -> Outcome {
    for (root, gap, detail, ts) in &emit.gaps {
        // Same field order as adapter.mjs logCoverageGap: ts, hook, event,
        // gap, detail, source?.
        let truncated: String = if detail.chars().count() <= DETAIL_MAX {
            detail.clone()
        } else {
            format!("{}...", detail.chars().take(DETAIL_MAX).collect::<String>())
        };
        let mut entry = Map::new();
        entry.insert("ts".into(), Value::String(ts.clone()));
        entry.insert("hook".into(), Value::String(HOOK_NAME.to_string()));
        entry.insert("event".into(), Value::String("coverage-gap".to_string()));
        entry.insert("gap".into(), Value::String(gap.to_string()));
        entry.insert("detail".into(), Value::String(truncated));
        if let Some(s) = source {
            entry.insert("source".into(), Value::String(s.to_string()));
        }
        append_hook_log(root, &Value::Object(entry));
    }
    use std::io::Write;
    if !emit.stdout.is_empty() {
        let _ = std::io::stdout().write_all(emit.stdout.as_bytes());
    }
    if !emit.stderr.is_empty() {
        let _ = std::io::stderr().write_all(emit.stderr.as_bytes());
    }
    Outcome::Done(ExitCode::from(emit.code))
}

// ─── main orchestration (provenance: bee-write-guard.mjs main()) ───────────

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    let argv = argv.to_vec();
    let stdin = stdin.to_string();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let ctx = read_hook_context(HOOK_NAME, &argv, &stdin);
        run_native(&ctx).map(|emit| (emit, ctx.source))
    })) {
        Ok(Ok((emit, source))) => flush(emit, source),
        Ok(Err(Nd)) => Outcome::Delegate,
        Err(_) => Outcome::Delegate, // a native panic is never a verdict
    }
}

fn first_truthy<'a>(map: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    for key in keys {
        if let Some(v) = map.get(*key) {
            if truthy(v) {
                return Some(v);
            }
        }
    }
    None
}

fn run_native(ctx: &HookContext) -> R<Emit> {
    let mut emit = Emit::default();
    let Some(root_pb) = ctx.root.clone() else {
        return Ok(emit);
    };
    let root = root_pb.to_string_lossy().into_owned();
    np_check_modelable(&root)?;
    let payload = &ctx.payload;

    // toolName = payload.tool_name || payload.toolName || ""
    let tool_name: String = match first_truthy(payload, &["tool_name", "toolName"]) {
        Some(Value::String(s)) => s.clone(),
        Some(_) => "\u{0}non-string-tool\u{0}".to_string(), // truthy non-string never matches a tool
        None => String::new(),
    };
    let is_write_tool = matches!(tool_name.as_str(), "Edit" | "Write" | "MultiEdit");
    let is_apply = matches!(tool_name.as_str(), "apply_patch" | "ApplyPatch");
    let is_read_tool = matches!(tool_name.as_str(), "Read" | "Glob" | "Grep");
    let write_capable = is_write_tool || tool_name == "Bash" || is_apply;

    if write_capable && ctx.worktree_resolution == "linked-invalid" {
        emit.stderr.push_str(
            "bee worktree guard denied this write: WORKTREE_LINK_INVALID — linked worktree metadata could not be validated. \
FIX: repair or recreate the Git worktree before retrying; no worktree-local .bee store is trusted.",
        );
        emit.code = 2;
        return Ok(emit);
    }

    let store_root_pb = ctx.store_root.clone().unwrap_or_else(|| root_pb.clone());
    let store_root = store_root_pb.to_string_lossy().into_owned();
    np_check_modelable(&store_root)?;
    if !store_root_pb.join(".bee").join("bin").join("lib").join("state.mjs").is_file() {
        return Ok(emit);
    }

    // Native semantics are proven only against byte-identical vendored lib.
    lib_byte_gate(&store_root_pb)?;

    if !hook_enabled(&store_root_pb, HOOK_NAME).map_err(|_| Nd)? {
        return Ok(emit);
    }

    let tool_input: Map<String, Value> = match payload.get("tool_input") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    };
    let cwd = ctx.cwd.to_string_lossy().into_owned();

    let mut denial: Option<String> = None;
    let mut fixed_ask: Option<(Value, Vec<String>)> = None;
    let mut reservation_warnings: Vec<String> = Vec::new();
    let control_root_s: Option<String> =
        ctx.control_root.as_ref().map(|p| p.to_string_lossy().into_owned());

    if tool_name == "AskUserQuestion" {
        match check_ask_user_question(&tool_input)? {
            AskResult::Allow => {}
            AskResult::Deny(reason) => denial = Some(reason),
            AskResult::Fixed { fixed, notes } => fixed_ask = Some((fixed, notes)),
        }
    } else if is_read_tool {
        let raw = first_truthy(&tool_input, &["file_path", "path"]);
        if let Some(rel) = lexical_rel_path(&root, &cwd, raw)? {
            match check_read(&rel) {
                ReadVerdict::Deny { reason, marker } => {
                    let mut parts = vec![reason];
                    if let Some(m) = marker {
                        parts.push(m);
                    }
                    denial = Some(parts.join("\n"));
                }
                ReadVerdict::Allow => {
                    if tool_name == "Read"
                        && !tool_input.contains_key("offset")
                        && !tool_input.contains_key("limit")
                    {
                        let config = read_config(&store_root_pb)?;
                        let threshold = resolve_max_read_lines(&config);
                        let abs = Path::new(&root).join(rel.replace('/', &SEP.to_string()));
                        if let Some(reason) = check_read_size_denial(&abs, &rel, threshold) {
                            denial = Some(reason);
                        }
                    }
                }
            }
        }
    } else if write_capable {
        let state = read_state(&store_root_pb)?;
        let agent_name = infer_agent_name(payload, &tool_input);
        let session_id = str_trim_nonempty(payload.get("session_id"));
        let mut rel_paths: Vec<String> = Vec::new();
        let mut shared_candidates: Vec<(String, String)> = Vec::new(); // (rel, abs)

        if is_apply {
            match apply_patch_text(&tool_input) {
                None => {
                    push_gap(
                        &mut emit,
                        &root_pb,
                        "applypatch-unparsed",
                        "apply_patch intercepted but no canonical patch envelope found in tool_input".to_string(),
                    );
                }
                Some(patch_text) => {
                    let targets = extract_apply_patch_targets(&patch_text);
                    for t in &targets {
                        if let Some(r) =
                            canonical_rel_path(&root, &cwd, Some(&Value::String(t.clone())))?
                        {
                            rel_paths.push(r);
                        }
                    }
                    if targets.is_empty() || rel_paths.len() < targets.len() {
                        let detail = if targets.is_empty() {
                            "apply_patch intercepted but no Add/Update/Delete/Move/\"Move to\" target line could be parsed from the patch body".to_string()
                        } else {
                            format!(
                                "apply_patch intercepted but {} of {} target(s) could not be proved inside the repo",
                                targets.len() - rel_paths.len(),
                                targets.len()
                            )
                        };
                        push_gap(&mut emit, &root_pb, "applypatch-unparsed", detail);
                        denial = Some(
                            "bee apply_patch guard: this patch's target set could not be fully proved inside the repo — \
denying rather than risking an unchecked write. \
FIX: use canonical \"*** Add File:\", \"*** Update File:\", \"*** Delete File:\", and \"*** Move to:\" \
lines naming plain in-repo relative paths (no path traversal, no unresolvable escapes), then resubmit."
                                .to_string(),
                        );
                    }
                }
            }
        } else if tool_name == "Bash" {
            let command = match tool_input.get("command") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            if !command.is_empty() {
                let targets = extract_bash_targets(&command);
                struct Cand {
                    raw: String,
                    canonical: Option<String>,
                    rel: Option<String>,
                }
                let mut cands: Vec<Cand> = Vec::new();
                for p in &targets.paths {
                    let canonical =
                        canonical_rel_path(&root, &cwd, Some(&Value::String(p.clone())))?;
                    let rel = match &canonical {
                        Some(c) => Some(c.clone()),
                        None => companion_mount_rel(&root)?, // marker present → Nd
                    };
                    if rel.is_none() {
                        // gmr-3: memory-root consult only for wall-failed
                        // targets; a declared root delegates.
                        let _ = memory_root_hit(&store_root_pb)?;
                    }
                    cands.push(Cand { raw: p.clone(), canonical, rel });
                }
                rel_paths = cands.iter().filter_map(|c| c.rel.clone()).collect();
                for c in &cands {
                    if let Some(canonical) = &c.canonical {
                        shared_candidates
                            .push((canonical.clone(), lexical_abs_target(&root, &cwd, &c.raw)?));
                    }
                }
                // memory-root hits are always 0 natively (a declared root
                // delegates above), so the count equation reduces to:
                if rel_paths.len() != targets.paths.len() {
                    let first_failing = cands.iter().find(|c| c.rel.is_none());
                    let enriched = match first_failing {
                        Some(c) => describe_cross_worktree_target(
                            &root,
                            &cwd,
                            &Value::String(c.raw.clone()),
                        )?,
                        None => None,
                    };
                    denial =
                        Some(enriched.unwrap_or_else(|| GENERIC_BASH_CONTAINMENT_MESSAGE.to_string()));
                } else if rel_paths.is_empty() && targets.broad_write {
                    rel_paths = vec!["**".to_string()];
                }
            }
        } else {
            // Edit / Write / MultiEdit: toolInput.file_path || ""
            let raw_v = match tool_input.get("file_path") {
                Some(v) if truthy(v) => v.clone(),
                _ => Value::String(String::new()),
            };
            let canonical = canonical_rel_path(&root, &cwd, Some(&raw_v))?;
            let rel = match &canonical {
                Some(c) => Some(c.clone()),
                None => companion_mount_rel(&root)?,
            };
            if let Some(rel) = rel {
                rel_paths = vec![rel];
                if let Some(canonical) = canonical {
                    if let Value::String(raw_s) = &raw_v {
                        shared_candidates
                            .push((canonical, lexical_abs_target(&root, &cwd, raw_s)?));
                    }
                }
            } else if memory_root_hit(&store_root_pb)? {
                // gmr-3 D6: pre-approved short-circuit (unreachable natively —
                // a declared memory root delegates; kept for shape parity).
            } else {
                let enriched = describe_cross_worktree_target(&root, &cwd, &raw_v)?;
                denial = Some(enriched.unwrap_or_else(|| GENERIC_CONTAINMENT_MESSAGE.to_string()));
            }
        }

        // wcg-2: shared nested-checkout refusal, BEFORE checkWrite.
        let mut shared_denied = false;
        if denial.is_none() {
            for (rel, abs) in &shared_candidates {
                if is_shared_nested_checkout_target(
                    &root,
                    abs,
                    session_id.as_deref(),
                    control_root_s.as_deref(),
                )? {
                    denial = Some(shared_nested_checkout_refusal(rel));
                    shared_denied = true;
                    break;
                }
            }
        }

        if !shared_denied {
            for rel in &rel_paths {
                match check_write(
                    &store_root,
                    &state,
                    rel,
                    agent_name.as_deref(),
                    session_id.as_deref(),
                    control_root_s.as_deref(),
                    &mut emit,
                )? {
                    WV::Deny(reason) => {
                        denial = Some(reason);
                        break;
                    }
                    WV::AllowWarn(warning) => reservation_warnings.push(warning),
                    WV::Allow => {}
                }
            }
        }

        if denial.is_none() && !rel_paths.is_empty() {
            if let Some(reason) = check_worktree_first(
                ctx.worktree_resolution,
                &root,
                &store_root_pb,
                &state,
                &rel_paths,
            )? {
                denial = Some(reason);
            }
        }

        if denial.is_none() && tool_name == "Bash" {
            let command = match tool_input.get("command") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            if !command.is_empty() {
                if let Some(WV::Deny(reason)) = check_git_bash_command(
                    &store_root,
                    &state,
                    &command,
                    &cwd,
                    session_id.as_deref(),
                    control_root_s.as_deref(),
                    &mut emit,
                )? {
                    denial = Some(reason);
                }
            }
        }

        if denial.is_none() && tool_name == "Bash" {
            let command = match tool_input.get("command") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            if !js_trim(&command).is_empty() && has_node_inline_eval(&command) {
                return Err(Nd); // internals-reach guard: regex scan not ported
            }
        }
    }

    // Check (d) — CLI-shape validation. The vendored validate-args.mjs /
    // command-registry.mjs import side effects are proven silent by the byte
    // gate; checkCliShape is null unless a bee-cli-shaped token resolves, and
    // it can only ASSIGN a denial when none exists yet.
    if tool_name == "Bash" {
        let command = match tool_input.get("command") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        if !command.is_empty() && denial.is_none() && has_bee_cli_token(&command) {
            return Err(Nd); // registry/validate semantics not ported
        }
    }

    if let Some((fixed, notes)) = fixed_ask {
        let notes_joined = notes.join("; ");
        let mut hso = Map::new();
        hso.insert("hookEventName".into(), Value::String("PreToolUse".into()));
        hso.insert("permissionDecision".into(), Value::String("allow".into()));
        hso.insert("permissionDecisionReason".into(), Value::String(notes_joined.clone()));
        hso.insert("updatedInput".into(), fixed);
        hso.insert(
            "additionalContext".into(),
            Value::String(format!("bee AskUserQuestion guard auto-fixed: {notes_joined}")),
        );
        let mut output = Map::new();
        output.insert("hookSpecificOutput".into(), Value::Object(hso));
        output.insert(
            "systemMessage".into(),
            Value::String(format!("bee AskUserQuestion guard: {notes_joined}")),
        );
        emit.stdout.push_str(&jsjson::stringify(&Value::Object(output)));
        emit.code = 0;
        return Ok(emit);
    }

    if denial.is_none() && !reservation_warnings.is_empty() {
        let joined = reservation_warnings.join("\n");
        let mut hso = Map::new();
        hso.insert("hookEventName".into(), Value::String("PreToolUse".into()));
        hso.insert("permissionDecision".into(), Value::String("allow".into()));
        hso.insert("permissionDecisionReason".into(), Value::String(joined.clone()));
        let mut output = Map::new();
        output.insert("hookSpecificOutput".into(), Value::Object(hso));
        output.insert("systemMessage".into(), Value::String(joined));
        emit.stdout.push_str(&jsjson::stringify(&Value::Object(output)));
        emit.code = 0;
        return Ok(emit);
    }

    if let Some(reason) = denial {
        emit.stderr.push_str(&reason);
        emit.code = 2;
        return Ok(emit);
    }
    Ok(emit)
}

// ─── tests ─────────────────────────────────────────────────────────────────
// Mirrors packages/bee/hooks/test_write_guard.mjs's decision table with
// tempfile fixtures (fixture lib vendored from the EMBEDDED set, exactly the
// bytes the gate demands — the same copyLib discipline the Node tests use).

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct Fx {
        _dir: tempfile::TempDir,
        root: PathBuf,
    }

    fn copy_lib(root: &Path) {
        let lib = root.join(".bee").join("bin").join("lib");
        std::fs::create_dir_all(&lib).unwrap();
        for (name, content) in EMBEDDED_LIB {
            std::fs::write(lib.join(name), content).unwrap();
        }
    }

    fn write_state(root: &Path, state: &Value) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(
            root.join(".bee").join("state.json"),
            format!("{}\n", serde_json::to_string_pretty(state).unwrap()),
        )
        .unwrap();
    }

    fn swarming_state(execution: bool) -> Value {
        json!({
            "phase": "swarming",
            "mode": "standard",
            "feature": "demo",
            "approved_gates": { "context": true, "shape": true, "execution": execution, "review": false }
        })
    }

    fn build_fixture(phase: &str, execution_approved: bool) -> Fx {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&root);
        let mut st = swarming_state(execution_approved);
        st["phase"] = json!(phase);
        write_state(&root, &st);
        Fx { _dir: dir, root }
    }

    fn run_payload(payload: Value, cwd: &Path) -> R<Emit> {
        let mut body = match payload {
            Value::Object(m) => m,
            _ => panic!("payload must be an object"),
        };
        body.insert("cwd".into(), Value::String(cwd.to_string_lossy().into_owned()));
        let stdin = jsjson::stringify(&Value::Object(body));
        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        run_native(&ctx)
    }

    fn expect_done(payload: Value, cwd: &Path) -> Emit {
        match run_payload(payload, cwd) {
            Ok(e) => e,
            Err(_) => panic!("expected a native verdict, got Delegate"),
        }
    }

    fn expect_delegate(payload: Value, cwd: &Path) {
        assert!(run_payload(payload, cwd).is_err(), "expected Delegate");
    }

    fn seed_lease(root: &Path, path: &str, agent: &str, cell: &str, session: Option<&str>, kind: &str) {
        let dir = root.join(".bee").join("runtime").join("leases").join("paths");
        std::fs::create_dir_all(&dir).unwrap();
        let now = now_ms();
        let acquired = ms_to_iso(now).unwrap();
        let expires = ms_to_iso(now + 3600.0 * 1000.0).unwrap();
        let record = json!({
            "resource": format!("path:{}", res_normalize_path(path)),
            "mode": "write",
            "workflow_id": cell,
            "session_id": session.unwrap_or(SESSIONLESS_SESSION_ID),
            "workspace_id": format!("agent:{}", agent),
            "epoch": 0,
            "acquired_at": acquired,
            "expires_at": expires,
            "kind": kind
        });
        let n = std::fs::read_dir(&dir).map(|d| d.count()).unwrap_or(0);
        std::fs::write(
            dir.join(format!("lease-{n}.json")),
            format!("{}\n", serde_json::to_string_pretty(&record).unwrap()),
        )
        .unwrap();
    }

    fn add_live_session(root: &Path, id: &str) {
        let dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        let now = ms_to_iso(now_ms()).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "id": id, "started_at": now, "last_heartbeat": now
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    fn edit(path: &str) -> Value {
        json!({ "tool_name": "Edit", "tool_input": { "file_path": path } })
    }
    fn bash(cmd: &str) -> Value {
        json!({ "tool_name": "Bash", "tool_input": { "command": cmd } })
    }
    fn patch(input: &str) -> Value {
        json!({ "tool_name": "apply_patch", "tool_input": { "input": input } })
    }

    // ── row1/2/3/3b/3c/3d/4/6: direct-edit deny table ──────────────────────

    #[test]
    fn direct_edit_state_json_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(edit(".bee/state.json"), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee.mjs state"));
        assert!(e.stderr.contains("FIX"));
        assert!(e.stderr.contains("direct-edit"));
    }

    #[test]
    fn direct_edit_backlog_write_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            json!({"tool_name":"Write","tool_input":{"file_path":".bee/backlog.jsonl","content":"{}\n"}}),
            &fx.root,
        );
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee.mjs backlog add"));
    }

    #[test]
    fn bash_redirect_into_backlog_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("cat notes.txt >> .bee/backlog.jsonl"), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee.mjs backlog add"));
    }

    #[test]
    fn sed_in_place_on_state_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("sed -i \"s/idle/swarming/\" .bee/state.json"), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee.mjs state"));
    }

    #[test]
    fn docs_backlog_md_denied_with_owning_verbs() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            json!({"tool_name":"Write","tool_input":{"file_path":"docs/backlog.md","content":"x\n"}}),
            &fx.root,
        );
        assert_eq!(e.code, 2);
        for needle in [
            "bee.mjs backlog pbi add",
            "bee.mjs backlog pbi status",
            "bee.mjs backlog pbi amend",
            "bee.mjs backlog render --write",
            "direct-edit",
        ] {
            assert!(e.stderr.contains(needle), "missing {needle}: {}", e.stderr);
        }
    }

    #[test]
    fn rest_of_docs_unaffected() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(edit("docs/history/demo/CONTEXT.md"), &fx.root);
        assert_eq!(e.code, 0, "stderr: {}", e.stderr);
    }

    #[test]
    fn bee_cells_json_passes() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(edit(".bee/cells/demo-1.json"), &fx.root);
        assert_eq!(e.code, 0);
    }

    #[test]
    fn idle_still_denies_direct_edit_and_allows_other_bee_paths() {
        let fx = build_fixture("idle", true);
        let e = expect_done(edit(".bee/state.json"), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee.mjs state"));
        let e2 = expect_done(edit(".bee/cells/demo-1.json"), &fx.root);
        assert_eq!(e2.code, 0);
    }

    // ── CLI-shaped bash commands delegate (check (d) not ported) ───────────

    #[test]
    fn bee_cli_shapes_delegate() {
        let fx = build_fixture("swarming", true);
        expect_delegate(bash("node .bee/bin/bee_state.mjs set --phase swarming"), &fx.root);
        expect_delegate(bash("node .bee/bin/bee_cells.mjs cap --outcome \"done\""), &fx.root);
        expect_delegate(bash("node .bee/bin/bee.mjs cells cap --outcome \"done\""), &fx.root);
    }

    #[test]
    fn tampered_vendored_lib_delegates() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("bin").join("lib").join("guards.mjs"),
            "throw new Error('boom');\n",
        )
        .unwrap();
        // row7's fixture shape: the byte gate refuses to prove equivalence.
        expect_delegate(edit(".bee/state.json"), &fx.root);
    }

    #[test]
    fn node_inline_eval_delegates() {
        let fx = build_fixture("swarming", true);
        expect_delegate(
            bash("node -e \"import('./.bee/bin/lib/cells.mjs').then(() => {})\""),
            &fx.root,
        );
        // A file-based node run is native and allowed.
        let e = expect_done(bash("node scripts/test_guards.mjs"), &fx.root);
        assert_eq!(e.code, 0);
    }

    // ── AskUserQuestion (ask-guard-autofix D1/D2) ──────────────────────────

    #[test]
    fn ask_long_header_is_auto_fixed() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            json!({"tool_name":"AskUserQuestion","tool_input":{"questions":[
                {"question":"q","header":"Worktree switch","options":[
                    {"label":"A","description":"x"},{"label":"B","description":"y"}]}]}}),
            &fx.root,
        );
        assert_eq!(e.code, 0);
        let parsed: Value = serde_json::from_str(&e.stdout).unwrap();
        assert_eq!(parsed["hookSpecificOutput"]["permissionDecision"], json!("allow"));
        assert_eq!(
            parsed["hookSpecificOutput"]["updatedInput"]["questions"][0]["header"],
            json!("Worktree sw…")
        );
    }

    #[test]
    fn ask_mixed_fixable_and_unfixable_denies() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            json!({"tool_name":"AskUserQuestion","tool_input":{"questions":[
                {"question":"q","header":"Worktree switch","options":[
                    {"label":"only-one","description":"x"}]}]}}),
            &fx.root,
        );
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("option"));
    }

    #[test]
    fn ask_valid_allowed() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            json!({"tool_name":"AskUserQuestion","tool_input":{"questions":[
                {"question":"q","header":"Approach","options":[
                    {"label":"A","description":"x"},{"label":"B","description":"y"}]}]}}),
            &fx.root,
        );
        assert_eq!(e.code, 0);
        assert!(e.stdout.is_empty());
    }

    // ── apply_patch matrix (rows 8-29) ─────────────────────────────────────

    #[test]
    fn apply_patch_add_safe_passes() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            patch("*** Begin Patch\n*** Add File: src/new-file.txt\n+hello world\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn apply_patch_update_state_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            patch("*** Begin Patch\n*** Update File: .bee/state.json\n@@\n-old\n+new\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee.mjs state"));
    }

    #[test]
    fn apply_patch_delete_backlog_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            patch("*** Begin Patch\n*** Delete File: .bee/backlog.jsonl\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee.mjs backlog add"));
    }

    #[test]
    fn apply_patch_move_safe_passes_and_denied_destination_denies() {
        let fx = build_fixture("swarming", true);
        let ok = expect_done(
            patch("*** Begin Patch\n*** Update File: src/old-name.txt\n*** Move to: src/new-name.txt\n@@\n-old\n+new\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(ok.code, 0);
        let deny = expect_done(
            patch("*** Begin Patch\n*** Update File: src/old-name.txt\n*** Move to: .bee/state.json\n@@\n-old\n+new\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(deny.code, 2);
        assert!(deny.stderr.contains("bee.mjs state"));
    }

    #[test]
    fn apply_patch_multi_target_one_denied_denies_whole() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            patch("*** Begin Patch\n*** Add File: src/a.txt\n+content\n*** Update File: src/b.txt\n@@\n-x\n+y\n*** Delete File: .bee/state.json\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee.mjs state"));
        let ok = expect_done(
            patch("*** Begin Patch\n*** Add File: src/a.txt\n+content\n*** Update File: src/b.txt\n@@\n-x\n+y\n*** Delete File: src/c.txt\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(ok.code, 0);
    }

    #[test]
    fn apply_patch_unicode_reserved_path_denied_with_holder() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "café/résumé.md", "otto", "other-cell", None, "lease");
        let e = expect_done(
            json!({"tool_name":"apply_patch","tool_input":{"input":"*** Begin Patch\n*** Add File: café/résumé.md\n+hello\n*** End Patch"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("café/résumé.md"));
        assert!(e.stderr.contains("otto"));
    }

    #[test]
    fn apply_patch_spaced_path_reserved_denied() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "my folder/file name.txt", "otto", "other-cell", None, "lease");
        let e = expect_done(
            json!({"tool_name":"apply_patch","tool_input":{"input":"*** Begin Patch\n*** Update File: my folder/file name.txt\n@@\n-a\n+b\n*** End Patch"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("my folder/file name.txt"));
    }

    #[test]
    fn apply_patch_escaped_space_path_resolves_and_denies() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "my\\ folder/escaped.txt", "otto", "other-cell", None, "lease");
        let e = expect_done(
            json!({"tool_name":"apply_patch","tool_input":{"input":"*** Begin Patch\n*** Add File: my\\ folder/escaped.txt\n+hi\n*** End Patch"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("otto"));
    }

    #[test]
    fn apply_patch_unprovable_shapes_deny() {
        let fx = build_fixture("swarming", true);
        for body in [
            "*** Begin Patch\n*** Add File\n+content\n*** End Patch",
            "*** Begin Patch\n*** Rename File: src/a.txt -> src/b.txt\n*** End Patch",
            "*** Begin Patch\n*** Add File:    \n+content\n*** End Patch",
            "*** Begin Patch\n*** Add File: ../../outside-repo.txt\n+x\n*** End Patch",
            // mixed provable+unprovable, both orders (rows 27/28/29)
            "*** Begin Patch\n*** Add File: src/safe-first.txt\n+hello\n*** Update File:    \n@@\n-old\n+new\n*** End Patch",
            "*** Begin Patch\n*** Update File:    \n@@\n-old\n+new\n*** Add File: src/safe-second.txt\n+hello\n*** End Patch",
            "*** Begin Patch\n*** Update File: src/valid.txt\n@@\n-old\n+new\n*** Update File: src/other.txt\n*** Move to: ../../outside-repo.txt\n@@\n-a\n+b\n*** End Patch",
        ] {
            let e = expect_done(patch(body), &fx.root);
            assert_eq!(e.code, 2, "should deny: {body}");
            assert!(e.stderr.contains("FIX"), "{body}");
            assert!(!e.gaps.is_empty(), "coverage gap logged: {body}");
        }
    }

    #[test]
    fn apply_patch_absolute_outside_denies() {
        let fx = build_fixture("swarming", true);
        let outside = tempfile::tempdir().unwrap();
        let target = outside.path().join("outside.txt");
        let body = format!(
            "*** Begin Patch\n*** Add File: {}\n+x\n*** End Patch",
            target.to_string_lossy()
        );
        let e = expect_done(patch(&body), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("FIX"));
    }

    #[test]
    fn apply_patch_no_envelope_fails_open_with_gap() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(patch("not a patch at all"), &fx.root);
        assert_eq!(e.code, 0);
        assert_eq!(e.gaps.len(), 1);
        assert_eq!(e.gaps[0].1, "applypatch-unparsed");
        assert!(e.gaps[0].2.contains("no canonical patch envelope"));
    }

    #[test]
    fn apply_patch_gate_policy_denies_source_allows_docs() {
        let fx = build_fixture("planning", false);
        let deny = expect_done(
            patch("*** Begin Patch\n*** Add File: src/feature.txt\n+new code\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(deny.code, 2);
        assert!(deny.stderr.contains("bee gate"));
        let ok = expect_done(
            patch("*** Begin Patch\n*** Add File: docs/notes.md\n+notes\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(ok.code, 0);
    }

    #[test]
    fn apply_patch_self_reservation_passes() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "src/mine.txt", "mel", "other-cell", None, "lease");
        let e = expect_done(
            json!({"tool_name":"apply_patch","tool_input":{"input":"*** Begin Patch\n*** Update File: src/mine.txt\n@@\n-a\n+b\n*** End Patch"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ── reservations via Bash + intent advisory ────────────────────────────

    #[test]
    fn bash_write_to_reserved_file_denied() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "src/held.txt", "otto", "cell-1", None, "lease");
        let e = expect_done(
            json!({"tool_name":"Bash","tool_input":{"command":"printf x > \"src/held.txt\""},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee reservation conflict"));
        assert!(e.stderr.contains("otto holds \"src/held.txt\" (cell cell-1)"));
    }

    #[test]
    fn intent_reservation_allows_with_warning() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "src/*", "otto", "plan-cell", None, "intent");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/app.js"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
        let parsed: Value = serde_json::from_str(&e.stdout).unwrap();
        assert_eq!(parsed["hookSpecificOutput"]["permissionDecision"], json!("allow"));
        let msg = parsed["systemMessage"].as_str().unwrap();
        assert!(msg.contains("bee reservation intent"));
        assert!(msg.contains("advisory only (kind: intent)"));
    }

    #[test]
    fn cross_session_hold_denies_and_corrupt_store_fails_closed() {
        let fx = build_fixture("swarming", true);
        add_live_session(&fx.root, "other");
        seed_lease(&fx.root, "src/held.txt", "otto", "cell-9", Some("other"), "lease");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/held.txt"},"session_id":"mine"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee cross-session hold"));
        assert!(e.stderr.contains("\"other\""));

        // corrupt projection store fails closed for a session-aware write
        std::fs::write(fx.root.join(".bee").join("reservations.json"), "{ not json").unwrap();
        let e2 = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/anything.txt"},"session_id":"mine"}),
            &fx.root,
        );
        assert_eq!(e2.code, 2);
        assert!(e2.stderr.contains("bee hold guard"));
        assert!(e2.stderr.contains("unreadable/corrupt"));
    }

    // ── linked-worktree matrix (rows 30-34) ────────────────────────────────

    struct Linked {
        _main_dir: tempfile::TempDir,
        _work_dir: tempfile::TempDir,
        main_root: PathBuf,
        work_root: PathBuf,
    }

    fn build_linked(valid: bool) -> Linked {
        let main_dir = tempfile::tempdir().unwrap();
        let work_dir = tempfile::tempdir().unwrap();
        let main_root = dunce::canonicalize(main_dir.path()).unwrap();
        let work_root = dunce::canonicalize(work_dir.path()).unwrap();
        let gitdir = main_root.join(".git").join("worktrees").join("fixture");
        std::fs::create_dir_all(&gitdir).unwrap();
        std::fs::write(
            work_root.join(".git"),
            format!("gitdir: {}\n", gitdir.to_string_lossy()),
        )
        .unwrap();
        if valid {
            std::fs::write(
                gitdir.join("gitdir"),
                format!("{}\n", work_root.join(".git").to_string_lossy()),
            )
            .unwrap();
        }
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        std::fs::write(main_root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&main_root);
        write_state(
            &main_root,
            &json!({
                "phase": "swarming", "mode": "high-risk", "feature": "worktree-isolation",
                "approved_gates": { "context": true, "shape": true, "execution": true, "review": false }
            }),
        );
        std::fs::create_dir_all(work_root.join("src")).unwrap();
        Linked { _main_dir: main_dir, _work_dir: work_dir, main_root, work_root }
    }

    #[test]
    fn linked_worktree_reads_foreign_reservation_from_main_store() {
        let lx = build_linked(true);
        seed_lease(&lx.main_root, "src/held.txt", "otto", "other", None, "lease");
        let deny = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/held.txt"},"agent_name":"mel"}),
            &lx.work_root,
        );
        assert_eq!(deny.code, 2, "{}", deny.stderr);
        assert!(deny.stderr.contains("otto"));
        let allow = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/held.txt"},"agent_name":"otto"}),
            &lx.work_root,
        );
        assert_eq!(allow.code, 0, "{}", allow.stderr);
    }

    #[test]
    fn linked_invalid_denies_before_mutation() {
        let lx = build_linked(false);
        for payload in [
            edit("src/new.txt"),
            bash("printf x > \"src/new.txt\""),
            patch("*** Begin Patch\n*** Add File: src/new.txt\n+x\n*** End Patch"),
        ] {
            let e = expect_done(payload, &lx.work_root);
            assert_eq!(e.code, 2);
            assert!(e.stderr.contains("WORKTREE_LINK_INVALID"));
        }
    }

    #[test]
    fn escape_rows_deny_and_contained_backslashes_pass() {
        let lx = build_linked(true);
        let traversal = expect_done(edit("../outside.txt"), &lx.work_root);
        assert_eq!(traversal.code, 2);
        assert_eq!(traversal.stderr, GENERIC_CONTAINMENT_MESSAGE);
        let win_traversal = expect_done(edit("..\\outside-win.txt"), &lx.work_root);
        assert_eq!(win_traversal.code, 2);
        let absolute_main = expect_done(
            edit(&lx.main_root.join("src").join("main-only.txt").to_string_lossy()),
            &lx.work_root,
        );
        assert_eq!(absolute_main.code, 2);
        // Contained Windows separators normalize into the reservation namespace.
        let contained = expect_done(edit("src\\nested\\new.txt"), &lx.work_root);
        assert_eq!(contained.code, 0, "{}", contained.stderr);
    }

    #[cfg(windows)]
    #[test]
    fn home_spellings_get_identical_deny() {
        let lx = build_linked(true);
        let home = std::env::var("USERPROFILE").unwrap();
        let absolute = format!("{}\\.claude\\bee-gmr1-probe.md", home);
        let spellings = [
            absolute.clone(),
            "~/.claude/bee-gmr1-probe.md".to_string(),
            "$HOME/.claude/bee-gmr1-probe.md".to_string(),
            "${HOME}/.claude/bee-gmr1-probe.md".to_string(),
        ];
        let baseline = expect_done(edit(&spellings[0]), &lx.work_root);
        assert_eq!(baseline.code, 2);
        for s in &spellings[1..] {
            let e = expect_done(edit(s), &lx.work_root);
            assert_eq!(e.code, 2, "{s}");
            assert_eq!(e.stderr, baseline.stderr, "{s}");
        }
    }

    #[test]
    fn bare_tilde_is_not_containment_denied() {
        let lx = build_linked(true);
        let e = expect_done(bash("rm -rf ~"), &lx.work_root);
        assert!(!e.stderr.contains("could not be canonically contained"), "{}", e.stderr);
    }

    // ── worktree-first (docs/specs/worktree-first.md §2) ───────────────────

    struct Wtf {
        _root_dir: tempfile::TempDir,
        _wt_dir: tempfile::TempDir,
        root: PathBuf,
        wt_root: PathBuf,
        id: String,
    }

    fn build_worktree_first(lane: &str, config_off: bool) -> Wtf {
        let fx_dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(fx_dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&root);
        let route_state = json!({
            "phase": "swarming", "mode": "standard", "feature": "demo",
            "route": { "class": "feature", "lane": lane, "flags": [], "product_files": 2, "rationale": null, "updated_at": ms_to_iso(now_ms()).unwrap() },
            "approved_gates": { "context": true, "shape": true, "execution": true, "review": false }
        });
        write_state(&root, &route_state);
        if config_off {
            std::fs::write(
                root.join(".bee").join("config.json"),
                "{\"worktree_first\":\"off\"}\n",
            )
            .unwrap();
        }
        let id = "wtf-demo-wt".to_string();
        let wt_dir = tempfile::tempdir().unwrap();
        let wt_root = dunce::canonicalize(wt_dir.path()).unwrap();
        let git_worktree_dir = root.join(".git").join("worktrees").join(&id);
        std::fs::create_dir_all(&git_worktree_dir).unwrap();
        std::fs::write(
            git_worktree_dir.join("gitdir"),
            format!("{}\n", wt_root.join(".git").to_string_lossy()),
        )
        .unwrap();
        std::fs::write(
            wt_root.join(".git"),
            format!("gitdir: {}\n", git_worktree_dir.to_string_lossy()),
        )
        .unwrap();
        std::fs::create_dir_all(root.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            root.join(".bee").join("runtime").join("worktree-grants.json"),
            format!("{}\n", json!({ &id: true })),
        )
        .unwrap();
        std::fs::create_dir_all(wt_root.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            wt_root.join(".bee").join("runtime").join("worktree-identity.json"),
            format!("{}\n", json!({ "feature": "demo", "created_at": ms_to_iso(now_ms()).unwrap() })),
        )
        .unwrap();
        std::fs::write(wt_root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&wt_root);
        write_state(&wt_root, &route_state);
        Wtf { _root_dir: fx_dir, _wt_dir: wt_dir, root, wt_root, id }
    }

    #[test]
    fn worktree_first_denies_main_source_write() {
        let wtf = build_worktree_first("standard", false);
        let e = expect_done(edit("src/app.js"), &wtf.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("worktree-first"));
        assert!(e.stderr.contains(&*wtf.wt_root.file_name().unwrap().to_string_lossy()));
        assert!(e.stderr.contains(&format!("bee worktree merge --id {}", wtf.id)));
        assert!(e.stderr.contains("worktree_first: \"off\""));
        assert!(e.stderr.contains("\"demo\"") && e.stderr.contains("\"standard\""));
        // Bash-extracted target too.
        let eb = expect_done(bash("printf x > src/app.js"), &wtf.root);
        assert_eq!(eb.code, 2);
    }

    #[test]
    fn worktree_first_exemptions_hold() {
        let wtf = build_worktree_first("standard", false);
        assert_eq!(expect_done(edit("docs/notes/plan.md"), &wtf.root).code, 0);
        assert_eq!(expect_done(edit("README.md"), &wtf.root).code, 0);
        // Inside the granted worktree the guard never fires.
        let inside = expect_done(edit("src/app.js"), &wtf.wt_root);
        assert_eq!(inside.code, 0, "{}", inside.stderr);
        // docs-lane route is exempt.
        let docs_lane = build_worktree_first("docs", false);
        assert_eq!(expect_done(edit("src/app.js"), &docs_lane.root).code, 0);
        // recorded off-switch disables the refusal.
        let off = build_worktree_first("standard", true);
        assert_eq!(expect_done(edit("src/app.js"), &off.root).code, 0);
        // corrupt grants registry fails OPEN.
        let corrupt = build_worktree_first("standard", false);
        std::fs::write(
            corrupt.root.join(".bee").join("runtime").join("worktree-grants.json"),
            "{ not json",
        )
        .unwrap();
        assert_eq!(expect_done(edit("src/app.js"), &corrupt.root).code, 0);
    }

    // ── scratch-shape guard (rows 35-45) ───────────────────────────────────

    #[test]
    fn scratch_shape_matrix() {
        let fx = build_fixture("swarming", true);
        let deny = |p: &str| {
            let e = expect_done(
                json!({"tool_name":"Write","tool_input":{"file_path":p,"content":"x\n"}}),
                &fx.root,
            );
            assert_eq!(e.code, 2, "expected deny for {p}: {}", e.stderr);
            assert!(e.stderr.contains(".bee/tmp/"), "{p}");
        };
        let allow = |p: &str| {
            let e = expect_done(
                json!({"tool_name":"Write","tool_input":{"file_path":p,"content":"x\n"}}),
                &fx.root,
            );
            assert_eq!(e.code, 0, "expected allow for {p}: {}", e.stderr);
        };
        deny(".bee/bin/.foo_stress_debug.sh");
        allow(".bee/tmp/th6/.foo_stress_debug.sh");
        allow("docs/history/tree-hygiene/reports/verdict-th6.md");
        allow(".bee/cells/probe-th-6.json");
        allow(".claude-plugin/skills/bee-swarming/probe-render.json");
        allow("test/fixtures/sample.log");
        deny("results.log");
        deny(".rel9999_stress_debug.sh");
        deny("scripts/scratch-notes.tmp");
        deny("scripts/probe-foo.mjs");
        // decisions ledger append stays allowed
        let e = expect_done(bash("printf \"x\" >> .bee/decisions.jsonl"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ── intake-gate git exemption (rows 47-56) ─────────────────────────────

    fn run_git(cwd: &Path, args: &[&str]) {
        let st = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(st.success(), "git {:?}", args);
    }

    fn build_git_fixture(phase: &str) -> Fx {
        let fx = build_fixture(phase, false);
        run_git(&fx.root, &["init", "-q"]);
        run_git(&fx.root, &["config", "user.email", "ige2@example.com"]);
        run_git(&fx.root, &["config", "user.name", "ige2 fixture"]);
        fx
    }

    fn stage_file(root: &Path, rel: &str) {
        let abs = root.join(rel.replace('/', &SEP.to_string()));
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(&abs, "x\n").unwrap();
        run_git(root, &["add", rel]);
    }

    #[test]
    fn git_readonly_allowed_at_terminal_phase() {
        let fx = build_git_fixture("idle");
        assert_eq!(expect_done(bash("git status"), &fx.root).code, 0);
        assert_eq!(expect_done(bash("git log --oneline -5"), &fx.root).code, 0);
    }

    #[test]
    fn git_commit_bookkeeping_exemption_and_source_refusal() {
        let bk = build_git_fixture("idle");
        stage_file(&bk.root, ".bee/cells/demo-1.json");
        stage_file(&bk.root, "docs/notes.md");
        let ok = expect_done(bash("git commit -m \"bookkeeping only\""), &bk.root);
        assert_eq!(ok.code, 0, "{}", ok.stderr);

        let src = build_git_fixture("idle");
        stage_file(&src.root, ".bee/cells/demo-1.json");
        stage_file(&src.root, "src/feature.js");
        let deny = expect_done(bash("git commit -m \"mixed change\""), &src.root);
        assert_eq!(deny.code, 2, "{}", deny.stderr);
        assert!(deny.stderr.contains("intake gate"));
        assert!(deny.stderr.contains("src/feature.js"));
        // D3: the bookkeeping route is named before guards.idle_gate.
        let bk_idx = deny.stderr.find("commit or write bookkeeping").unwrap();
        let gate_idx = deny.stderr.find("guards.idle_gate").unwrap();
        assert!(bk_idx < gate_idx);
    }

    #[test]
    fn git_push_and_unknown_subcommands_refused() {
        let fx = build_git_fixture("idle");
        let push = expect_done(bash("git push origin main"), &fx.root);
        assert_eq!(push.code, 2);
        assert!(push.stderr.contains("never exempted"));
        let unk = expect_done(bash("git bisect start"), &fx.root);
        assert_eq!(unk.code, 2);
    }

    #[test]
    fn git_add_pathspec_exemption() {
        let fx = build_git_fixture("idle");
        std::fs::create_dir_all(fx.root.join("src")).unwrap();
        std::fs::write(fx.root.join("src").join("new.js"), "x\n").unwrap();
        let deny = expect_done(bash("git add src/new.js"), &fx.root);
        assert_eq!(deny.code, 2, "{}", deny.stderr);
        std::fs::create_dir_all(fx.root.join("docs")).unwrap();
        std::fs::write(fx.root.join("docs").join("new.md"), "# x\n").unwrap();
        let ok = expect_done(bash("git add docs/new.md"), &fx.root);
        assert_eq!(ok.code, 0, "{}", ok.stderr);
    }

    #[test]
    fn git_commit_outside_terminal_phase_unaffected() {
        let fx = build_git_fixture("swarming");
        write_state(&fx.root, &swarming_state(true));
        stage_file(&fx.root, "src/feature.js");
        let e = expect_done(bash("git commit -m \"normal work\""), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ── large-read guard (rows 57-64) ──────────────────────────────────────

    #[test]
    fn read_size_guard_matrix() {
        let fx = build_fixture("swarming", true);
        let big: String = (0..900).map(|i| format!("line {i}\n")).collect();
        std::fs::write(fx.root.join("big.md"), &big).unwrap();
        std::fs::write(fx.root.join("small.md"), "line 1\nline 2\nline 3\n").unwrap();
        std::fs::create_dir_all(fx.root.join("a-directory")).unwrap();

        let read = |ti: Value| json!({ "tool_name": "Read", "tool_input": ti });
        let deny = expect_done(read(json!({"file_path":"big.md"})), &fx.root);
        assert_eq!(deny.code, 2);
        assert!(deny.stderr.contains("big.md"));
        assert!(deny.stderr.contains("900"));
        assert!(deny.stderr.contains("800"));
        assert!(deny.stderr.contains("limit"));
        assert!(deny.stderr.contains("bee-extract"));

        assert_eq!(expect_done(read(json!({"file_path":"big.md","limit":50})), &fx.root).code, 0);
        assert_eq!(expect_done(read(json!({"file_path":"big.md","offset":100})), &fx.root).code, 0);
        assert_eq!(expect_done(read(json!({"file_path":"small.md"})), &fx.root).code, 0);
        assert_eq!(expect_done(read(json!({"file_path":"a-directory"})), &fx.root).code, 0);
        assert_eq!(expect_done(read(json!({"file_path":"does-not-exist.md"})), &fx.root).code, 0);

        // hooks.write-guard=false disables the whole guard.
        let disabled = build_fixture("swarming", true);
        std::fs::write(disabled.root.join("big.md"), &big).unwrap();
        std::fs::write(
            disabled.root.join(".bee").join("config.json"),
            format!("{}\n", serde_json::to_string_pretty(&json!({"hooks":{"write-guard":false}})).unwrap()),
        )
        .unwrap();
        assert_eq!(expect_done(read(json!({"file_path":"big.md"})), &disabled.root).code, 0);

        // custom threshold trips on a 3-line file.
        let custom = build_fixture("swarming", true);
        std::fs::write(custom.root.join("small.md"), "line 1\nline 2\nline 3\n").unwrap();
        std::fs::write(
            custom.root.join(".bee").join("config.json"),
            format!("{}\n", serde_json::to_string_pretty(&json!({"guards":{"max_read_lines":2}})).unwrap()),
        )
        .unwrap();
        let e = expect_done(read(json!({"file_path":"small.md"})), &custom.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("(threshold: 2)"));
    }

    #[test]
    fn secret_and_scout_reads_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(json!({"tool_name":"Read","tool_input":{"file_path":".env"}}), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee privacy guard"));
        assert!(e.stderr.contains("@@BEE_PRIVACY@@"));
        assert!(e.stderr.contains("@@END@@"));
        let s = expect_done(
            json!({"tool_name":"Read","tool_input":{"file_path":"node_modules/x/index.js"}}),
            &fx.root,
        );
        assert_eq!(s.code, 2);
        assert!(s.stderr.contains("bee scout guard"));
    }

    // ── shared nested-checkout guard (wcg rows 71/72/78/80) ────────────────

    #[test]
    fn nested_checkout_concurrent_denies_solo_allows() {
        let fx = build_fixture("swarming", true);
        let nested = fx.root.join("repo");
        std::fs::create_dir_all(nested.join(".git")).unwrap();
        std::fs::write(nested.join("foo.js"), "// nested plain\n").unwrap();

        // Solo (no live session): allowed.
        let solo = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"repo/foo.js"},"session_id":"me"}),
            &fx.root,
        );
        assert_eq!(solo.code, 0, "{}", solo.stderr);

        // Another live session: denied with the paved-road refusal.
        add_live_session(&fx.root, "other-live");
        let deny = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"repo/foo.js"},"session_id":"me"}),
            &fx.root,
        );
        assert_eq!(deny.code, 2, "{}", deny.stderr);
        assert!(deny.stderr.contains("bee shared-checkout guard"));
        assert!(deny.stderr.contains("bee worktree new --with-companion"));

        // Bash branch is wired too.
        let bash_deny = expect_done(
            json!({"tool_name":"Bash","tool_input":{"command":"cp new.js repo/foo.js"},"session_id":"me"}),
            &fx.root,
        );
        assert_eq!(bash_deny.code, 2);
    }

    #[test]
    fn own_live_session_is_excluded() {
        let fx = build_fixture("swarming", true);
        let nested = fx.root.join("repo");
        std::fs::create_dir_all(nested.join(".git")).unwrap();
        std::fs::write(nested.join("foo.js"), "// nested plain\n").unwrap();
        add_live_session(&fx.root, "me");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"repo/foo.js"},"session_id":"me"}),
            &fx.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn companion_marker_present_delegates_on_containment_failure() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("companion-session.json"),
            "{\"sessionId\":\"s1\",\"worktreePath\":\"/x\",\"mountPath\":\"repo\"}\n",
        )
        .unwrap();
        expect_delegate(edit("../outside.txt"), &fx.root);
        // A contained target never consults the marker — stays native.
        let e = expect_done(edit("src/inside.txt"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn declared_memory_root_delegates_on_containment_failure() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("config.json"),
            "{\"guards\":{\"memory_root\":\"~/.claude/projects/x/memory\"}}\n",
        )
        .unwrap();
        expect_delegate(edit("../outside.txt"), &fx.root);
        let e = expect_done(edit("src/inside.txt"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ── unknown phase / gate phases ────────────────────────────────────────

    #[test]
    fn unknown_phase_fails_closed() {
        let fx = build_fixture("bogus-phase", true);
        let e = expect_done(edit("src/app.js"), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee phase guard"));
        assert!(e.stderr.contains("bogus-phase"));
    }

    #[test]
    fn gated_phase_denies_source_until_execution_approved() {
        let fx = build_fixture("planning", false);
        let deny = expect_done(edit("src/app.js"), &fx.root);
        assert_eq!(deny.code, 2);
        assert!(deny.stderr.contains("bee gate"));
        assert!(deny.stderr.contains("execution"));
        let docs = expect_done(edit("docs/plan.md"), &fx.root);
        assert_eq!(docs.code, 0);
        let approved = build_fixture("planning", true);
        assert_eq!(expect_done(edit("src/app.js"), &approved.root).code, 0);
    }

    #[test]
    fn idle_intake_gate_denies_source_and_respects_opt_out() {
        let fx = build_fixture("idle", false);
        let deny = expect_done(edit("src/app.js"), &fx.root);
        assert_eq!(deny.code, 2);
        assert!(deny.stderr.contains("bee intake gate"));
        assert!(deny.stderr.contains("phase: idle"));
        std::fs::write(
            fx.root.join(".bee").join("config.json"),
            "{\"guards\":{\"idle_gate\":false}}\n",
        )
        .unwrap();
        assert_eq!(expect_done(edit("src/app.js"), &fx.root).code, 0);
    }

    // ── misc plumbing rows ─────────────────────────────────────────────────

    #[test]
    fn missing_lib_presence_gate_exits_zero() {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        write_state(&root, &swarming_state(true));
        let e = expect_done(edit(".bee/state.json"), &root);
        assert_eq!(e.code, 0); // no vendored lib: fail-open like the .mjs
    }

    #[test]
    fn disabled_hook_exits_zero() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("config.json"),
            "{\"hooks\":{\"write-guard\":false}}\n",
        )
        .unwrap();
        let e = expect_done(edit(".bee/state.json"), &fx.root);
        assert_eq!(e.code, 0);
    }

    #[test]
    fn corrupt_state_json_delegates() {
        let fx = build_fixture("swarming", true);
        std::fs::write(fx.root.join(".bee").join("state.json"), "{broken").unwrap();
        // Node warns to stderr with the V8 message — unreplicable → Delegate.
        expect_delegate(edit("src/app.js"), &fx.root);
    }

    #[test]
    fn plain_bash_is_native_and_allowed() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("echo hi"), &fx.root);
        assert_eq!(e.code, 0);
        assert!(e.stdout.is_empty() && e.stderr.is_empty());
    }

    #[test]
    fn bee_agent_name_env_prefix_is_parsed_from_command() {
        assert_eq!(agent_name_from_command("BEE_AGENT_NAME=mel printf x > f"), Some("mel".into()));
        assert_eq!(agent_name_from_command("BEE_AGENT_NAME=\"mel\" x"), Some("mel".into()));
        assert_eq!(agent_name_from_command("BEE_AGENT_NAME='mel' x"), Some("mel".into()));
        assert_eq!(agent_name_from_command("XBEE_AGENT_NAME=mel"), None);
        assert_eq!(agent_name_from_command("BEE_AGENT_NAME=\"mel x"), None);
    }

    // ── tokenizer decision-table cases ─────────────────────────────────────

    #[test]
    fn tokenizer_matches_mjs_semantics() {
        assert_eq!(tokenize("a b"), vec!["a", "b"]);
        // separators split even glued to text
        assert_eq!(tokenize("x 2>/dev/null; y"), vec!["x", "2>/dev/null", ";", "y"]);
        assert_eq!(tokenize("a&&b"), vec!["a", "&&", "b"]);
        // adjacent quoted/unquoted segments merge (bash word-splitting)
        assert_eq!(tokenize("'.bee/state'\".json\""), vec![".bee/state.json"]);
        // backslash escapes the next char literally
        assert_eq!(tokenize("a\\;b.txt"), vec!["a;b.txt"]);
        // unterminated quote runs to end
        assert_eq!(tokenize("\"a b"), vec!["a b"]);
    }

    #[test]
    fn quote_concat_cannot_evade_direct_edit_deny() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("printf x > '.bee/state'\".json\""), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee.mjs state"));
    }

    // ── Node path-port vectors (generated from node:path win32) ────────────

    #[cfg(windows)]
    #[test]
    fn node_win32_path_vectors() {
        assert_eq!(np_relative("D:\\a\\b", "D:\\a\\b\\c.txt").unwrap(), "c.txt");
        assert_eq!(np_relative("D:\\a\\b", "D:\\a\\x").unwrap(), "..\\x");
        assert_eq!(np_relative("D:\\a", "C:\\b").unwrap(), "C:\\b");
        assert_eq!(np_relative("D:\\a\\b", "D:\\a").unwrap(), "..");
        assert_eq!(np_resolve2("D:\\a", "..\\..\\x").unwrap(), "D:\\x");
        assert_eq!(np_resolve2("D:\\a\\b", "src/new file.txt").unwrap(), "D:\\a\\b\\src\\new file.txt");
        assert_eq!(np_resolve2("D:\\a", "\\foo").unwrap(), "D:\\foo");
        assert_eq!(np_relative("d:\\A\\B", "D:\\a\\b\\C.txt").unwrap(), "C.txt");
        assert_eq!(np_relative("D:\\", "D:\\x").unwrap(), "x");
        assert_eq!(np_dirname("D:\\"), "D:\\");
        assert_eq!(np_resolve2("D:\\a", "café/résumé.md").unwrap(), "D:\\a\\café\\résumé.md");
        assert_eq!(np_relative("D:\\a\\b", "D:\\a\\bc").unwrap(), "..\\bc");
        assert_eq!(np_relative("D:\\a\\b", "D:\\a\\b").unwrap(), "");
        assert_eq!(np_relative("D:\\a\\b\\c", "D:\\a").unwrap(), "..\\..");
        assert_eq!(np_resolve2("D:\\a", "..").unwrap(), "D:\\");
        assert_eq!(np_resolve2("D:\\a", ".").unwrap(), "D:\\a");
        assert_eq!(np_basename("D:\\a\\b.txt"), "b.txt");
        assert_eq!(np_dirname("D:\\a\\b.txt"), "D:\\a");
        assert_eq!(np_resolve2("D:\\a", "my\\ folder/escaped.txt").unwrap(), "D:\\a\\my\\ folder\\escaped.txt");
        assert_eq!(np_relative("D:\\a", "D:\\a\\..b\\x").unwrap(), "..b\\x");
        assert_eq!(np_resolve2("D:\\a\\b", "..\\outside.txt").unwrap(), "D:\\a\\outside.txt");
        assert_eq!(np_resolve2("D:\\x", "").unwrap(), "D:\\x");
        assert_eq!(np_relative("D:\\a\\b\\", "D:\\a\\b\\c").unwrap(), "c");
        assert_eq!(np_relative("D:\\a\\b", "D:\\A\\B\\c").unwrap(), "c");
        assert_eq!(np_relative("D:\\x", "E:\\y").unwrap(), "E:\\y");
        assert!(np_resolve2("D:\\a", "C:foo").is_err()); // drive-relative → Nd
        assert!(np_resolve1("\\\\srv\\share\\x").is_err()); // UNC → Nd
    }

    #[test]
    fn paths_overlap_vectors() {
        assert!(paths_overlap("src/api", "src/api/router.ts"));
        assert!(paths_overlap("src/api/*", "src/api/router.ts"));
        assert!(paths_overlap("a", "a"));
        assert!(!paths_overlap("src/a", "src/b"));
        assert!(paths_overlap("*", "anything"));
        assert!(paths_overlap("my/ folder/escaped.txt", "my\\ folder/escaped.txt"));
    }

    #[test]
    fn glob_matcher_vectors() {
        let m = |g: &str, p: &str| glob_match(&glob_tokens(g), &p.chars().collect::<Vec<_>>());
        assert!(m("**/migrations/**", "db/migrations/001.sql"));
        assert!(m("**/migrations/**", "migrations/001.sql"));
        assert!(!m("**/migrations/**", "migrations"));
        assert!(m("package-lock.json", "package-lock.json"));
        assert!(m("**/package-lock.json", "pkg/a/package-lock.json"));
        assert!(!m("package-lock.json", "pkg/package-lock.json"));
        assert!(m("**/generated/**", "src/generated/client.ts"));
    }

    #[test]
    fn extract_bash_targets_vectors() {
        let t = extract_bash_targets("cat notes.txt >> .bee/backlog.jsonl");
        assert_eq!(t.paths, vec![".bee/backlog.jsonl"]);
        let t = extract_bash_targets("printf x 2>&1");
        assert!(t.paths.is_empty());
        let t = extract_bash_targets("rm -rf");
        assert!(t.paths.is_empty());
        assert!(t.broad_write);
        let t = extract_bash_targets("git add --all");
        assert!(t.broad_write);
        let t = extract_bash_targets("git add .bee/state.json");
        assert!(t.paths.is_empty()); // D8: staging a CLI-owned file is not a direct edit
        let t = extract_bash_targets("git mv a.txt b.txt");
        assert_eq!(t.paths, vec!["a.txt", "b.txt"]);
        let t = extract_bash_targets("sed -i \"s/a/b/\" f.txt");
        assert_eq!(t.paths, vec!["f.txt"]);
        let t = extract_bash_targets("node x.mjs > out.log && echo done");
        assert_eq!(t.paths, vec!["out.log"]);
    }

    // ── D9 glued/spaced separator matrix ──────────────────────────────────
    // R5 port of packages/bee/tests/test_guards_tokenizer.mjs. The existing
    // `tokenizer_matches_mjs_semantics` asserts only two of the five
    // separator forms; the whole point of that suite is that EVERY form in
    // the SEPARATORS set splits identically whether glued or spaced — a
    // glued `&` that failed to split used to garble the adjacent path and
    // leak command-verb tokens into the target list.

    #[test]
    fn d9_every_separator_form_splits_glued_and_spaced_alike() {
        for sep in [";", "&&", "&", "|", "||"] {
            let glued = extract_bash_targets(&format!("git add a.txt{sep}git add b.txt"));
            assert_eq!(
                glued.paths,
                vec!["a.txt", "b.txt"],
                "glued {sep:?} must not glue onto the adjacent token"
            );
            let spaced = extract_bash_targets(&format!("git add a.txt {sep} git add b.txt"));
            assert_eq!(spaced.paths, vec!["a.txt", "b.txt"], "spaced {sep:?}");
        }
    }

    #[test]
    fn d9_separator_lookalikes_are_not_boundaries() {
        // fd duplication is not a file write
        assert!(extract_bash_targets("echo hi 2>&1").paths.is_empty());
        assert!(extract_bash_targets("echo hi 1>&2").paths.is_empty());
        // a separator character inside quotes is data
        assert_eq!(extract_bash_targets("rm 'a&b.txt'").paths, vec!["a&b.txt"]);
        // a backslash-escaped separator stays part of the filename
        assert_eq!(extract_bash_targets("rm a\\;b.txt").paths, vec!["a;b.txt"]);
    }

    #[test]
    fn d8_staging_a_cli_owned_file_is_not_a_direct_edit_target() {
        // Chained form — the case the mixed command used to break.
        assert!(extract_bash_targets("git add .bee/backlog.jsonl && git commit -m \"stage\"")
            .paths
            .is_empty());
        assert!(extract_bash_targets("git add .bee/backlog.jsonl").paths.is_empty());
        // Control: an actual content mutation of the same file IS a target,
        // so the two assertions above are the D8 exemption firing rather
        // than the extractor going blind on `.bee/` paths.
        assert_eq!(
            extract_bash_targets("sed -i s/a/b/ .bee/backlog.jsonl").paths,
            vec![".bee/backlog.jsonl"]
        );
    }

    #[test]
    fn d12_companion_marker_is_direct_edit_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(edit(".bee/companion-session.json"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee worktree new --with-companion"), "{}", e.stderr);
    }

    // ── isSharedNestedCheckoutTarget primitive, rows 72–77 ─────────────────
    // R5 port of test_write_guard.mjs rows 72–77, which call the primitive
    // DIRECTLY (the .mjs imports isSharedNestedCheckoutTarget from the
    // vendored guards.mjs). Rows 71/78/80/81 already run through the hook in
    // `nested_checkout_concurrent_denies_solo_allows`; these five are the
    // exclusion arms that the wired rows never reach.

    /// Probe (never a platform guess): attempt a real directory symlink in a
    /// scratch dir. win32 denies this without Developer Mode / elevation.
    fn symlink_capable() -> bool {
        use std::sync::OnceLock;
        static CAP: OnceLock<bool> = OnceLock::new();
        *CAP.get_or_init(|| {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("t");
            std::fs::create_dir(&target).unwrap();
            symlink_dir(&target, &dir.path().join("l")).is_ok()
        })
    }

    fn symlink_dir(target: &Path, link: &Path) -> std::io::Result<()> {
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(target, link)
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link)
        }
    }

    /// A bare root with `.bee/` — the primitive needs no state.json, only the
    /// sessions dir it reads through `is_concurrent_mode`.
    fn wcg_root() -> Fx {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        Fx { _dir: dir, root }
    }

    fn flagged(root: &Path, target: &Path) -> bool {
        is_shared_nested_checkout_target(
            &root.to_string_lossy(),
            &target.to_string_lossy(),
            None,
            None,
        )
        .expect("primitive must decide natively")
    }

    #[test]
    fn row72_71_plain_nested_checkout_flags_only_when_concurrent() {
        let fx = wcg_root();
        let nested = fx.root.join("repo");
        std::fs::create_dir_all(nested.join(".git")).unwrap();
        std::fs::write(nested.join("foo.js"), "// nested plain\n").unwrap();
        let target = nested.join("foo.js");

        // row72: solo — the D6 backward-compat no-op.
        assert!(!flagged(&fx.root, &target), "row72: solo must not flag");
        // row71: another live session — STR65's unguarded incident shape.
        add_live_session(&fx.root, "other-live");
        assert!(flagged(&fx.root, &target), "row71: concurrent must flag");
    }

    #[test]
    fn row73_registered_submodule_is_never_flagged() {
        // The exclusion keys off `.gitmodules` registration, not the `.git`
        // shape, so the fixture only needs the registration the primitive
        // actually reads (Node's row73 spends a real `git submodule add` to
        // produce exactly these two artifacts).
        let fx = wcg_root();
        let nested = fx.root.join("repo");
        std::fs::create_dir_all(nested.join(".git")).unwrap();
        std::fs::write(nested.join("foo.js"), "// submodule file\n").unwrap();
        std::fs::write(
            fx.root.join(".gitmodules"),
            "[submodule \"repo\"]\n\tpath = repo\n\turl = ../remote.git\n",
        )
        .unwrap();
        add_live_session(&fx.root, "other-live");
        assert!(
            !flagged(&fx.root, &nested.join("foo.js")),
            "row73: a registered submodule is excluded even when concurrent"
        );

        // Control: the SAME tree without the registration IS flagged — proves
        // the assertion above is the exclusion firing, not a vacuous pass.
        std::fs::remove_file(fx.root.join(".gitmodules")).unwrap();
        assert!(flagged(&fx.root, &nested.join("foo.js")));

        // A .gitmodules registering some OTHER path does not excuse this one.
        std::fs::write(
            fx.root.join(".gitmodules"),
            "[submodule \"vendor\"]\n\tpath = vendor/lib\n",
        )
        .unwrap();
        assert!(flagged(&fx.root, &nested.join("foo.js")));
    }

    #[test]
    fn rows74_77_verified_companion_mount_exclusions() {
        const CAP: &str =
            "symlink creation denied — needs Developer Mode or an elevated shell";
        if !symlink_capable() {
            for row in [
                "row75: verified companion mount, solo, is NOT flagged",
                "row74: verified companion mount + concurrent session IS flagged",
                "row76: a marker whose worktreePath mismatches the live symlink is NOT flagged",
                "row77: a symlink mount with NO marker is NOT flagged by the primitive",
            ] {
                eprintln!("SKIP (env-limited: {CAP}) — {row}");
            }
            return;
        }

        let mount_dir = tempfile::tempdir().unwrap();
        let mount_target = dunce::canonicalize(mount_dir.path()).unwrap();
        std::fs::create_dir_all(mount_target.join(".git")).unwrap();
        std::fs::write(mount_target.join("foo.js"), "// companion file\n").unwrap();

        let write_marker = |root: &Path, worktree: &Path| {
            std::fs::write(
                root.join(".bee").join("companion-session.json"),
                format!(
                    "{}\n",
                    serde_json::to_string_pretty(&json!({
                        "sessionId": "s1",
                        "worktreePath": worktree.to_string_lossy(),
                        "mountPath": "repo"
                    }))
                    .unwrap()
                ),
            )
            .unwrap();
        };

        // rows 75 / 74 — verified marker, solo then concurrent.
        let verified = wcg_root();
        symlink_dir(&mount_target, &verified.root.join("repo")).unwrap();
        write_marker(&verified.root, &mount_target);
        let target = verified.root.join("repo").join("foo.js");
        assert!(!flagged(&verified.root, &target), "row75: solo is a no-op");
        add_live_session(&verified.root, "other-live");
        assert!(
            flagged(&verified.root, &target),
            "row74: a verified mount reachable by another live session IS flagged"
        );

        // row76 — the marker's declared worktreePath does not resolve to the
        // live symlink, so verification fails and the primitive stays quiet.
        let other_dir = tempfile::tempdir().unwrap();
        let other_real = dunce::canonicalize(other_dir.path()).unwrap();
        let mismatch = wcg_root();
        symlink_dir(&mount_target, &mismatch.root.join("repo")).unwrap();
        write_marker(&mismatch.root, &other_real);
        add_live_session(&mismatch.root, "other-live");
        assert!(
            !flagged(&mismatch.root, &mismatch.root.join("repo").join("foo.js")),
            "row76: verification failure is not a flag"
        );

        // row77 — a symlink mount with no marker at all: containment's job,
        // not this primitive's.
        let no_marker = wcg_root();
        symlink_dir(&mount_target, &no_marker.root.join("repo")).unwrap();
        add_live_session(&no_marker.root, "other-live");
        assert!(
            !flagged(&no_marker.root, &no_marker.root.join("repo").join("foo.js")),
            "row77: an unmarked symlink mount is not flagged by the primitive"
        );
    }
}
