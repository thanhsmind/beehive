// status_full — native port of FULL `bee status` (default and --lanes-full,
// --json and text) and `bee orient` (--json and text).
//
// Provenance: bee.mjs handleStatus/buildStatus/renderStatusText (~874-1206),
// handleOrient/buildOrient/renderOrientText (~1229-1373), plus exactly the
// lib functions those consume:
//   lib/state.mjs        readState/readConfig/readOnboarding/readHandoff/
//                        bypassLevel/bypassBanner/shipVisibility/
//                        hasStaleAdvisorKey/validateModelsConfig/
//                        validateAgentFilesDrift/listLanes/readLane/
//                        resolveContext/controlRootFor/resolveProductRoot
//   lib/cells.mjs        listCells/readCell/readyCells/archivedTotals/
//                        scribingDebt/globalScribingDebt/bestScribingStampMs/
//                        tierMix/ceilingScarcityWarning
//   lib/claims.mjs       listSessionRecords/readSession/resolveSessionId/
//                        heartbeatStale/activeWorkers/readClaim/isClaimActive
//   lib/reservations.mjs listReservations (over lib/lease-store.mjs listLeases)
//   lib/decisions.mjs    activeDecisions/datamark (+ tag overlay)
//   lib/backlog.mjs      readBacklogCounts (fold + legacy table)
//   lib/reviews.mjs      listReviews/listCandidates/deriveCandidateStatus
//   lib/recovery.mjs     detectCrashCandidates/scanTranscriptRoots/
//                        readTranscriptTail/hasCleanEndTrio/lastDurableSettlement
//   lib/perf.mjs         claudeProjectsRoot/encodeProjectDir/resolveTranscript
//   lib/capture.mjs      captureQueue/pendingCaptureStubs
//   lib/worktree-store.mjs readGrants/findGrantedWorktreeForFeature
//   lib/source-identity.mjs classifySource
//   lib/fsutil.mjs       hashFile (sha256 of the lossy-utf8 STRING content)
//
// Strangler rules honored here:
//   - try_native accepts ONLY the six argv shapes below; --brief is handled
//     upstream by status_brief; anything else -> None before any output.
//   - Corrupt JSON anywhere on the snapshot path (any site where Node's
//     readJson would print its V8-message warning) -> Ex::Bail -> None
//     BEFORE any output (the manifest drift-cache write excepted).
//   - JS-exotic input (truthy non-object approved_gates spread, non-string
//     git args, ...) -> Ex::Bail as well: the Node re-run owns the edge.
//   - JS throw sites that Node CATCHES locally (buildReviewBlock /
//     buildRecoveryBlock / orientWorktreeContext try/catch) are modeled as
//     Ex::Thrown and caught at the same spots; a Thrown that would escape to
//     main()'s emitError instead bails to Node, which reproduces the error.
//   - Handler-time stderr warnings are BUFFERED in order and printed only at
//     emit (before the drift line), so a bail can never leak partial output.

use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, LinkedRoots, RootsWt};
use crate::state::{bypass_level, read_config_raw, Bail};
use crate::verbs::{emit_no_root_error, record_timing};
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

type JMap = Map<String, Value>;

// ─── constants (state.mjs / bee.mjs) ───────────────────────────────────────

/// state.mjs BEE_VERSION — must track the Node constant; the diff harness
/// catches drift (status text embeds it).
const BEE_VERSION: &str = "1.20.3";

const GATE_NAMES: [&str; 4] = ["context", "shape", "execution", "review"];
const PHASES: [&str; 8] = [
    "idle", "exploring", "planning", "swarming", "reviewing", "scribing", "compounding", "grooming",
];
const KNOWN_PHASES: [&str; 9] = [
    "idle", "exploring", "planning", "swarming", "reviewing", "scribing", "compounding", "grooming",
    "compounding-complete",
];
const COMMAND_KEYS: [&str; 4] = ["setup", "start", "test", "verify"];
const WORKTREE_COMPANION_COMMAND_KEYS: [&str; 3] = [
    "worktree_companion_start", "worktree_companion_end", "worktree_companion_mount",
];
const MODEL_TIERS: [&str; 3] = ["extraction", "generation", "ceiling"];
const EFFORT_LEVELS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];
const RUNTIMES: [&str; 2] = ["claude", "codex"];
const MODEL_NORMALIZE_SLOTS: [&str; 4] = ["extraction", "generation", "review", "advisor"];
const MODEL_VALIDATE_SLOTS: [&str; 4] = ["extraction", "generation", "review", "advisor"];
const ADVICE_CLASS_SLOTS: [&str; 2] = ["advisor", "review"];
const UNSAFE_CLI_FLAGS: [&str; 6] = [
    "--yolo",
    "--dangerously-skip-permissions",
    "--dangerously-bypass-approvals-and-sandbox",
    "--full-auto",
    "-s danger-full-access",
    "--sandbox danger-full-access",
];
const ADVICE_CLASS_WRITABLE_TOKENS: [&str; 4] = [
    "-s workspace-write",
    "--sandbox workspace-write",
    "--sandbox=workspace-write",
    "danger-full-access",
];
const STALE_ADVISOR_KEY_WARNING: &str = "advisor mode was removed in 0.1.23; the top-level advisor key in .bee/config.json is ignored — delete it. (This does not affect the models.<runtime>.advisor slot, which is separate and still valid.)";

// bee.mjs ~425-432
const STALE_HANDOFF_MS: f64 = 7.0 * 24.0 * 60.0 * 60.0 * 1000.0;
const POST_EXECUTION_REVIEW_PHASES: [&str; 3] = ["scribing", "compounding", "compounding-complete"];

// bee.mjs ~819-821
const CONTENTION_TAIL_MAX_BYTES: u64 = 65536;
const CONTENTION_RECENT_BUSY_LIMIT: usize = 5;
const CONTENTION_TOP_LOCKS_LIMIT: usize = 5;

// cells.mjs ~2280-2281
const CEILING_MAX_SHARE: f64 = 0.4;
const SCARCITY_MIN_TIERED: i64 = 3;

// claims.mjs
const DEFAULT_HEARTBEAT_STALE_SECONDS: f64 = 900.0;

// recovery.mjs
const DEFAULT_TAIL_MAX_BYTES: u64 = 262144;
const TERMINAL_LANE_PHASES: [&str; 2] = ["idle", "compounding-complete"];

// backlog.mjs
const BACKLOG_STATUSES: [&str; 3] = ["proposed", "in-flight", "done"];
const PBI_STATUSES: [&str; 5] = ["proposed", "in-flight", "parked", "done", "declined"];

// bee.mjs ~1229-1235
const ORIENT_PHASE_SKILL: [(&str, &str); 5] = [
    ("exploring", "bee-shaping"),
    ("planning", "bee-planning"),
    ("swarming", "bee-swarming"),
    ("scribing", "bee-capturing"),
    ("compounding", "bee-capturing"),
];

// ─── error plumbing ────────────────────────────────────────────────────────

/// Bail = delegate to Node before any output. Thrown = a JS exception Node
/// CATCHES locally (review/recovery/orient-worktree fail-open wrappers); one
/// escaping to the top level also bails (the Node re-run reproduces it).
#[derive(Debug)]
enum Ex {
    Bail,
    Thrown,
}
impl From<Bail> for Ex {
    fn from(_: Bail) -> Self {
        Ex::Bail
    }
}
type R<T> = Result<T, Ex>;

struct Ctx {
    root: PathBuf,
    cwd: PathBuf,
    /// `resolveRoots(process.cwd()).linked` — `None` for an ORDINARY
    /// checkout, which is every main-checkout run and every unit fixture, so
    /// the pre-flip behavior is reached by exactly the same code path.
    ///
    /// bee.mjs re-runs `resolveRoots(process.cwd())` inside each of
    /// ungrantedWorktreeNotice / grantedWorktreeContext /
    /// orientWorktreeContext because `root` alone cannot tell an ordinary
    /// checkout apart from an ungranted worktree quietly sharing main's
    /// store (its own comment at ungrantedWorktreeNotice, GH #30). Resolving
    /// it once here and threading it is equivalent — the walk is a pure read
    /// and nothing in a status/orient run mutates `.git` or the registry.
    linked: Option<LinkedRoots>,
    /// Buffered stderr lines (console.warn / process.stderr.write) in Node's
    /// emission order; printed at emit time, before the drift line.
    stderr: Vec<String>,
}

impl Ctx {
    fn warn(&mut self, line: String) {
        self.stderr.push(line);
    }

    /// The linked classification, only when the current checkout is a GRANTED
    /// worktree (bee.mjs grantedWorktreeContext's own test).
    fn granted_worktree(&self) -> Option<&LinkedRoots> {
        self.linked.as_ref().filter(|l| l.granted())
    }
}

// ─── JS value helpers ──────────────────────────────────────────────────────

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

fn opt_truthy(o: Option<&Value>) -> bool {
    o.map(truthy).unwrap_or(false)
}

/// Property access: `undefined` is `None` (distinct from JSON null).
fn vget<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.as_object().and_then(|m| m.get(key))
}

/// JS strict equality (===) over JSON-representable primitives; None models
/// `undefined`. Objects/arrays compare by reference in JS — two separately
/// parsed values are never reference-equal, so they compare false here.
fn strict_eq(a: Option<&Value>, b: Option<&Value>) -> bool {
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

fn str_eq(v: Option<&Value>, s: &str) -> bool {
    matches!(v, Some(Value::String(x)) if x == s)
}

/// Template-literal coercion; `undefined` renders "undefined".
fn tpl(o: Option<&Value>) -> String {
    match o {
        None => "undefined".to_string(),
        Some(v) => jsjson::js_to_string(v),
    }
}

/// `value ?? fallback` — nullish only.
fn nullish(o: Option<&Value>) -> bool {
    matches!(o, None | Some(Value::Null))
}

/// JS Array.prototype.join over Values (null/undefined -> empty string).
fn js_join(items: &[Value], sep: &str) -> String {
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
fn js_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

/// JS Math.round: floor(x + 0.5).
fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

// ─── dates (V8 Date.parse for the ISO shapes bee writes, toISOString) ──────

fn now_ms() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(f64::NAN)
}

/// new Date(ms).toISOString() — millisecond UTC ISO.
fn to_iso(ms: f64) -> String {
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
fn js_date_parse(s: &str) -> f64 {
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
fn date_parse_val(v: Option<&Value>) -> f64 {
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

fn char_class_key(c: char) -> (u8, u32) {
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

fn locale_cmp(a: &str, b: &str, numeric: bool) -> Ordering {
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

// ─── fs primitives (fsutil.mjs / recovery.mjs) ─────────────────────────────

/// readJson(file, fallback): Missing -> Ok(None); Corrupt (would print the
/// V8-message warning in Node) -> Ex::Bail; Parsed -> Ok(Some(v)).
fn rj(file: &Path) -> R<Option<Value>> {
    match crate::fsutil::read_json(file) {
        crate::fsutil::ReadJson::Missing => Ok(None),
        crate::fsutil::ReadJson::Corrupt => Err(Ex::Bail),
        crate::fsutil::ReadJson::Parsed(v) => Ok(Some(v)),
    }
}

fn read_text_opt(file: &Path) -> Option<String> {
    std::fs::read(file).ok().map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// readJsonl: split /\r?\n/, trim, JSON.parse per line, silent skip.
fn read_jsonl(file: &Path) -> Vec<Value> {
    match read_text_opt(file) {
        Some(text) => parse_jsonl_text(&text),
        None => Vec::new(),
    }
}

fn parse_jsonl_text(text: &str) -> Vec<Value> {
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

/// fsutil.mjs hashFile: sha256 of the file's UTF-8 STRING content — read as
/// lossy utf8 (Node fs.readFileSync 'utf8'), hash those string bytes. No BOM
/// strip, matching Node.
fn hash_file(file: &Path) -> Option<String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(file).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    Some(format!("{:x}", hasher.finalize()))
}

/// recovery.mjs readTranscriptTail — bounded tail window, drop the first
/// (truncated) line when the window starts mid-file, silent per-line parse.
fn read_transcript_tail(file: &Path, max_bytes: u64) -> R<Vec<Value>> {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(meta) = std::fs::metadata(file) else {
        return Ok(Vec::new()); // statSync throw -> [] (catch at top of fn)
    };
    let size = meta.len();
    if size == 0 {
        return Ok(Vec::new());
    }
    let start = size.saturating_sub(max_bytes);
    // Node openSync/readSync failures THROW (caught by the caller's own
    // fail-open wrapper where one exists).
    let mut f = std::fs::File::open(file).map_err(|_| Ex::Thrown)?;
    f.seek(SeekFrom::Start(start)).map_err(|_| Ex::Thrown)?;
    let mut buf = Vec::with_capacity((size - start) as usize);
    f.take(size - start).read_to_end(&mut buf).map_err(|_| Ex::Thrown)?;
    let mut text = String::from_utf8_lossy(&buf).into_owned();
    if start > 0 {
        match text.find('\n') {
            Some(nl) => text = text[nl + 1..].to_string(),
            None => text = String::new(),
        }
    }
    Ok(parse_jsonl_text(&text))
}

/// Node path.join/resolve-style lexical normalization of an absolute path
/// (separator unification + '.'/'..' collapse). Only used where Node's
/// path output shape is observable (worktree resolution, projects root).
fn normalize_abs_lexical(p: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    let unified: String = p.replace(['/', '\\'], &sep.to_string());
    let mut prefix = String::new();
    let mut rest = unified.as_str();
    if cfg!(windows) {
        let bytes = rest.as_bytes();
        if bytes.len() >= 2 && bytes[1] == b':' {
            prefix = rest[..2].to_string();
            rest = &rest[2..];
        }
    }
    let absolute = rest.starts_with(sep);
    let mut parts: Vec<&str> = Vec::new();
    for seg in rest.split(sep) {
        match seg {
            "" | "." => {}
            ".." => {
                if !parts.is_empty() {
                    parts.pop();
                }
            }
            s => parts.push(s),
        }
    }
    let mut out = prefix;
    if absolute {
        out.push(sep);
    }
    out.push_str(&parts.join(&sep.to_string()));
    out
}

/// Node path.resolve(base, p) for gitdir pointers: absolute p normalizes
/// alone; relative joins onto base.
fn path_resolve(base: &Path, p: &str) -> String {
    let is_abs = {
        let b = p.as_bytes();
        p.starts_with('/') || p.starts_with('\\') || (b.len() >= 2 && b[1] == b':')
    };
    if is_abs {
        normalize_abs_lexical(p)
    } else {
        normalize_abs_lexical(&format!("{}{}{}", base.display(), std::path::MAIN_SEPARATOR, p))
    }
}

fn path_dirname(p: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    match p.rfind(sep) {
        Some(idx) if idx > 0 => p[..idx].to_string(),
        Some(_) => p[..1].to_string(),
        None => p.to_string(),
    }
}

fn path_basename(p: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    match p.rfind(sep) {
        Some(idx) => p[idx + 1..].to_string(),
        None => p.to_string(),
    }
}

fn home_dir() -> String {
    if cfg!(windows) {
        std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_default()
    } else {
        std::env::var("HOME").unwrap_or_default()
    }
}

/// perf.mjs claudeProjectsRoot — CLAUDE_CONFIG_DIR override (JS || falsy),
/// else <home>/.claude; 'projects' joined with Node path.join shape.
fn claude_projects_root() -> String {
    let base = match std::env::var("CLAUDE_CONFIG_DIR") {
        Ok(v) if !v.is_empty() => v,
        _ => format!("{}{}.claude", home_dir(), std::path::MAIN_SEPARATOR),
    };
    normalize_abs_lexical(&format!(
        "{}{}projects",
        base.trim_end_matches(['/', '\\']),
        std::path::MAIN_SEPARATOR
    ))
}

/// perf.mjs encodeProjectDir: replace [\\/.] with '-'.
fn encode_project_dir(project_path: &str) -> String {
    project_path
        .chars()
        .map(|c| if c == '\\' || c == '/' || c == '.' { '-' } else { c })
        .collect()
}

// ─── state layer (state.mjs) ───────────────────────────────────────────────

fn default_gates() -> JMap {
    let mut m = JMap::new();
    for g in GATE_NAMES {
        m.insert(g.into(), Value::Bool(false));
    }
    m
}

/// state.mjs defaultState().
fn default_state() -> JMap {
    let mut m = JMap::new();
    m.insert("schema_version".into(), json!("1.0"));
    m.insert("phase".into(), json!("idle"));
    m.insert("feature".into(), Value::Null);
    m.insert("mode".into(), Value::Null);
    m.insert("approved_gates".into(), Value::Object(default_gates()));
    m.insert("workers".into(), json!([]));
    m.insert("summary".into(), json!(""));
    m.insert("next_action".into(), json!("No active bee work — awaiting a user request."));
    m
}

/// state.mjs readState — fail-open merge over defaultState(), with the D13
/// legacy 'validating' -> 'planning' coercion. Truthy non-object
/// approved_gates spreads JS-exotically -> bail.
fn read_state_full(ctx: &Ctx) -> R<JMap> {
    let parsed = rj(&ctx.root.join(".bee").join("state.json"))?;
    let file_state = match parsed {
        Some(Value::Object(m)) => Some(m),
        _ => None,
    };
    let mut merged = default_state();
    let Some(state) = file_state else {
        return Ok(merged);
    };
    for (k, v) in &state {
        merged.insert(k.clone(), v.clone()); // existing keys keep position (JS spread)
    }
    let gates = match state.get("approved_gates") {
        None | Some(Value::Null) | Some(Value::Bool(false)) => default_gates(),
        Some(Value::String(s)) if s.is_empty() => default_gates(),
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => default_gates(),
        Some(Value::Object(overlay)) => {
            let mut g = default_gates();
            for (k, v) in overlay {
                g.insert(k.clone(), v.clone());
            }
            g
        }
        Some(_) => return Err(Ex::Bail), // truthy non-object spread — JS-exotic
    };
    merged.insert("approved_gates".into(), Value::Object(gates));
    if str_eq(merged.get("phase"), "validating") {
        merged.insert("phase".into(), json!("planning"));
    }
    Ok(merged)
}

/// state.mjs readOnboarding.
fn read_onboarding(ctx: &Ctx) -> R<Option<Value>> {
    rj(&ctx.root.join(".bee").join("onboarding.json"))
}

/// state.mjs readHandoff — fail-open; non-object parses return verbatim; an
/// object gets `kind` normalized (missing/unknown -> 'pause') at its original
/// key position (JS `{...handoff, kind}` semantics).
fn read_handoff(ctx: &Ctx) -> R<Option<Value>> {
    let parsed = rj(&ctx.root.join(".bee").join("HANDOFF.json"))?;
    let Some(v) = parsed else { return Ok(Some(Value::Null)) }; // readJson fallback null
    match v {
        Value::Object(m) => {
            let kind = if str_eq(m.get("kind"), "planned-next") { "planned-next" } else { "pause" };
            let mut out = m;
            out.insert("kind".into(), json!(kind));
            Ok(Some(Value::Object(out)))
        }
        other => Ok(Some(other)),
    }
}

// ─── config layer (state.mjs readConfig + normalizers) ─────────────────────

struct Config {
    /// Merged tracked+overlay raw object, advisor key stripped (the value
    /// readConfig spreads as `...rest`; gate_bypass/ship_visibility/
    /// product_root/recovery read straight off this).
    raw: JMap,
    commands: JMap,
    models: JMap,
}

/// state.mjs normalizeCommands.
fn normalize_commands(raw: Option<&Value>) -> JMap {
    let mut commands = JMap::new();
    let Some(Value::Object(obj)) = raw else { return commands };
    for key in COMMAND_KEYS.iter().chain(WORKTREE_COMPANION_COMMAND_KEYS.iter()) {
        match obj.get(*key) {
            Some(Value::String(s)) if !js_trim(s).is_empty() => {
                commands.insert((*key).into(), json!(js_trim(s)));
            }
            Some(Value::Array(items)) if *key == "test" => {
                let list: Vec<Value> = items
                    .iter()
                    .filter_map(|c| c.as_str())
                    .filter(|c| !js_trim(c).is_empty())
                    .map(|c| json!(js_trim(c)))
                    .collect();
                if !list.is_empty() {
                    commands.insert((*key).into(), Value::Array(list));
                }
            }
            _ => {}
        }
    }
    commands
}

/// state.mjs normalizeTierValue — returns None for "undefined" (invalid
/// shape: the seeded default stays).
fn normalize_tier_value(value: Option<&Value>) -> Option<Value> {
    match value {
        Some(Value::String(s)) if !js_trim(s).is_empty() => Some(json!(js_trim(s))),
        Some(Value::Null) => Some(Value::Null),
        Some(Value::Object(o)) => {
            if str_eq(o.get("kind"), "cli") {
                if let Some(Value::String(cmd)) = o.get("command") {
                    if !js_trim(cmd).is_empty() {
                        let mut out = JMap::new();
                        out.insert("kind".into(), json!("cli"));
                        out.insert("command".into(), json!(js_trim(cmd)));
                        return Some(Value::Object(out));
                    }
                }
            }
            if str_eq(o.get("kind"), "native") {
                if let Some(Value::String(model)) = o.get("model") {
                    if !js_trim(model).is_empty() {
                        let mut out = JMap::new();
                        out.insert("kind".into(), json!("native"));
                        out.insert("model".into(), json!(js_trim(model)));
                        if let Some(Value::String(e)) = o.get("effort") {
                            if EFFORT_LEVELS.contains(&js_trim(e)) {
                                out.insert("effort".into(), json!(js_trim(e)));
                            }
                        }
                        if let Some(Value::String(ft)) = o.get("fork_turns") {
                            if js_trim(ft) == "none" {
                                out.insert("fork_turns".into(), json!("none"));
                            }
                        }
                        if let Some(Value::String(at)) = o.get("agent_type") {
                            if !js_trim(at).is_empty() {
                                out.insert("agent_type".into(), json!(js_trim(at)));
                            }
                        }
                        return Some(Value::Object(out));
                    }
                }
            }
            // Explicit-fallback composite: primary must be a valid native leaf.
            if let Some(primary @ Value::Object(p)) = o.get("primary") {
                let primary_ok = str_eq(p.get("kind"), "native")
                    && matches!(p.get("model"), Some(Value::String(m)) if !js_trim(m).is_empty());
                if primary_ok {
                    let mut out = JMap::new();
                    out.insert("primary".into(), normalize_tier_value(Some(primary)).unwrap_or(Value::Null));
                    if str_eq(o.get("fallback_policy"), "explicit-only") {
                        out.insert("fallback_policy".into(), json!("explicit-only"));
                        if let Some(Value::Object(fb)) = o.get("fallback") {
                            if str_eq(fb.get("kind"), "cli") {
                                if let Some(Value::String(cmd)) = fb.get("command") {
                                    if !js_trim(cmd).is_empty() {
                                        let mut f = JMap::new();
                                        f.insert("kind".into(), json!("cli"));
                                        f.insert("command".into(), json!(js_trim(cmd)));
                                        out.insert("fallback".into(), Value::Object(f));
                                    }
                                }
                            }
                        }
                    }
                    return Some(Value::Object(out));
                }
            }
            if o.get("kind").is_none() {
                if let Some(Value::String(model)) = o.get("model") {
                    if !js_trim(model).is_empty() {
                        let mut out = JMap::new();
                        out.insert("model".into(), json!(js_trim(model)));
                        if let Some(Value::String(e)) = o.get("effort") {
                            if EFFORT_LEVELS.contains(&js_trim(e)) {
                                out.insert("effort".into(), json!(js_trim(e)));
                            }
                        }
                        return Some(Value::Object(out));
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// state.mjs DEFAULT_MODELS + normalizeModels.
fn normalize_models(raw: Option<&Value>) -> JMap {
    let mut claude = JMap::new();
    claude.insert("extraction".into(), json!("haiku"));
    claude.insert("generation".into(), json!("sonnet"));
    claude.insert("review".into(), json!("opus"));
    let mut codex = JMap::new();
    codex.insert("extraction".into(), Value::Null);
    codex.insert("generation".into(), Value::Null);
    codex.insert("review".into(), Value::Null);
    let mut out = JMap::new();
    out.insert("claude".into(), Value::Object(claude));
    out.insert("codex".into(), Value::Object(codex));
    if let Some(Value::Object(m)) = raw {
        for rt in RUNTIMES {
            let Some(Value::Object(src)) = m.get(rt) else { continue };
            for slot in MODEL_NORMALIZE_SLOTS {
                if let Some(v) = normalize_tier_value(src.get(slot)) {
                    if let Some(Value::Object(target)) = out.get_mut(rt) {
                        target.insert(slot.into(), v);
                    }
                }
            }
        }
    }
    out
}

/// state.mjs normalizeDogfoodRepos — the WARNING side only (the normalized
/// list itself is never read by status). Every readConfig call re-emits.
fn dogfood_warnings(ctx: &mut Ctx, raw: &JMap) {
    let Some(Value::Array(items)) = raw.get("dogfood_repos") else { return };
    if std::env::var("BEE_HOOK_CONTEXT").map(|v| !v.is_empty()).unwrap_or(false) {
        // Warning suppressed under a hook context; entries still skipped.
        return;
    }
    for item in items {
        let raw_path: Option<&str> = match item {
            Value::String(s) => Some(s.as_str()),
            Value::Object(o) => match o.get("path") {
                Some(Value::String(p)) => Some(p.as_str()),
                _ => None,
            },
            _ => None,
        };
        let Some(raw_path) = raw_path else { continue };
        if js_trim(raw_path).is_empty() {
            continue;
        }
        let trimmed = js_trim(raw_path);
        let resolved = if Path::new(trimmed).is_absolute() {
            PathBuf::from(trimmed)
        } else {
            ctx.cwd.join(trimmed)
        };
        if let Err(err) = dunce::canonicalize(&resolved) {
            let code = match err.kind() {
                std::io::ErrorKind::NotFound => "ENOENT".to_string(),
                std::io::ErrorKind::PermissionDenied => "EACCES".to_string(),
                _ => format!("{err}"),
            };
            ctx.warn(format!(
                "dogfood_repos: skipping \"{raw_path}\" — {code} (dead or unreadable repo; the bee session continues)"
            ));
        }
    }
}

/// state.mjs readConfig — merged tracked+overlay (via crate::state::
/// read_config_raw, which bails on corrupt), advisor stripped, plus the
/// normalized commands/models this port consumes. Emits dogfood warnings on
/// EVERY call, mirroring Node's per-call normalization.
fn read_config(ctx: &mut Ctx) -> R<Config> {
    let raw = read_config_raw(&ctx.root)?;
    let commands = normalize_commands(raw.get("commands"));
    let models = normalize_models(raw.get("models"));
    dogfood_warnings(ctx, &raw);
    Ok(Config { raw, commands, models })
}

/// state.mjs shipVisibility — warn+normalize on unrecognized values.
fn ship_visibility(ctx: &mut Ctx) -> R<String> {
    let config = read_config(ctx)?;
    match config.raw.get("ship_visibility") {
        None | Some(Value::Null) => Ok("off".into()),
        Some(Value::String(s)) if s == "off" || s == "draft-pr" => Ok(s.clone()),
        Some(other) => {
            ctx.warn(format!(
                "config: unrecognized ship_visibility \"{}\" in .bee/config.json — normalized to \"off\". Allowed: off, draft-pr.",
                jsjson::js_to_string(other)
            ));
            Ok("off".into())
        }
    }
}

/// state.mjs bypassLevel(root).
fn bypass_level_root(ctx: &mut Ctx) -> R<&'static str> {
    let config = read_config(ctx)?;
    Ok(bypass_level(&config.raw))
}

/// state.mjs bypassBanner.
fn bypass_banner(level: &str) -> &'static str {
    match level {
        "total" => "⚡⚡⚡ GATE BYPASS: TOTAL AUTOPILOT — ZERO STOPS. Every gate (any lane, high-risk/hard-gate included), secret-file reads, and review P1 findings auto-proceed; NO human checkpoint remains. Turn off: bee-hive bypass off",
        "full" => "⚡⚡ GATE BYPASS: FULL AUTOPILOT — ALL Gates 1-3 auto-approved including high-risk/hard-gate work; only secret-file reads and a review P1 finding still stop for the human. Turn off: bee-hive bypass off",
        "normal" => "⚡ GATE BYPASS: NORMAL — Gates 1-3 auto-approved for tiny/small/standard work only; high-risk/hard-gate, secret reads, and Gate 4 UAT still stop. Turn off: bee-hive bypass off",
        _ => "",
    }
}

/// state.mjs hasStaleAdvisorKey — reads the TRACKED config.json raw.
fn has_stale_advisor_key(ctx: &Ctx) -> R<bool> {
    let raw = rj(&ctx.root.join(".bee").join("config.json"))?;
    Ok(matches!(raw, Some(Value::Object(m)) if m.contains_key("advisor")))
}

/// bee.mjs readRawConfigForValidation — None = no config file at all
/// (undefined); Some(v) = whatever was parsed (fallback null on corrupt would
/// warn in Node -> bail).
fn read_raw_config_for_validation(ctx: &Ctx) -> R<Option<Value>> {
    let file = ctx.root.join(".bee").join("config.json");
    if !file.exists() {
        return Ok(None);
    }
    Ok(Some(rj(&file)?.unwrap_or(Value::Null)))
}

struct Problem {
    code: &'static str,
    runtime: Option<&'static str>,
    slot: Option<&'static str>,
    message: String,
    /// Only for validateAgentFilesDrift rows.
    agent: Option<&'static str>,
}

/// state.mjs validateModelsConfig — never throws; returns problem rows.
fn validate_models_config(config: Option<&Value>) -> Vec<Problem> {
    let mut problems = Vec::new();
    let Some(config) = config else { return problems };
    let obj = match config {
        Value::Object(m) => m,
        _ => {
            problems.push(Problem {
                code: "config-malformed",
                runtime: None,
                slot: None,
                message: ".bee/config.json content is null or not an object — models config cannot be validated; defaults apply.".into(),
                agent: None,
            });
            return problems;
        }
    };
    let Some(models) = obj.get("models") else { return problems };
    let models = match models {
        Value::Object(m) => m,
        _ => {
            problems.push(Problem {
                code: "config-malformed",
                runtime: None,
                slot: None,
                message: "`models` in .bee/config.json is present but not an object — ignored; defaults apply.".into(),
                agent: None,
            });
            return problems;
        }
    };
    for rt in RUNTIMES {
        let Some(src) = models.get(rt) else { continue };
        let src = match src {
            Value::Object(m) => m,
            _ => {
                problems.push(Problem {
                    code: "runtime-malformed",
                    runtime: Some(rt),
                    slot: None,
                    message: format!("models.{rt} is present but not an object — ignored; defaults apply."),
                    agent: None,
                });
                continue;
            }
        };
        for slot in MODEL_VALIDATE_SLOTS {
            let value = match src.get(slot) {
                None | Some(Value::Null) => continue,
                Some(Value::String(_)) => continue,
                Some(v) => v,
            };
            let vobj = match value {
                Value::Object(m) => m,
                _ => {
                    problems.push(Problem {
                        code: "slot-value-malformed",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} is not a string, object, or null — ignored; defaults apply."),
                        agent: None,
                    });
                    continue;
                }
            };
            let is_composite = vobj.contains_key("primary")
                || vobj.contains_key("fallback")
                || vobj.contains_key("fallback_policy");
            if is_composite {
                let primary = vobj.get("primary");
                let primary_ok = matches!(primary, Some(Value::Object(p))
                    if str_eq(p.get("kind"), "native")
                        && matches!(p.get("model"), Some(Value::String(m)) if !js_trim(m).is_empty()));
                if !primary_ok {
                    problems.push(Problem {
                        code: "composite-primary-malformed",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} is a composite (primary/fallback) but its primary is not a valid native override {{kind:\"native\", model}} — ignored; today this silently reverts to the seeded default (D2)."),
                        agent: None,
                    });
                    continue;
                }
                if let Some(Value::Object(p)) = primary {
                    if let Some(ft) = p.get("fork_turns") {
                        if !str_eq(Some(ft), "none") {
                            problems.push(Problem {
                                code: "native-fork-turns-unknown",
                                runtime: Some(rt),
                                slot: Some(slot),
                                message: format!(
                                    "models.{rt}.{slot} composite primary has fork_turns:{} — only \"none\" is valid; a full-history fork rejects model overrides (E2/D2).",
                                    jsjson::stringify(ft)
                                ),
                                agent: None,
                            });
                        }
                    }
                }
                if !str_eq(vobj.get("fallback_policy"), "explicit-only") {
                    problems.push(Problem {
                        code: "composite-fallback-policy-missing",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} is a composite but has no fallback_policy:\"explicit-only\" — its cli fallback is silently dropped and no fallback is ever taken; silent native->cli fallback is forbidden (D1). Set fallback_policy:\"explicit-only\" to opt in."),
                        agent: None,
                    });
                    continue;
                }
                let fb_ok = matches!(vobj.get("fallback"), Some(Value::Object(f))
                    if str_eq(f.get("kind"), "cli")
                        && matches!(f.get("command"), Some(Value::String(c)) if !js_trim(c).is_empty()));
                if !fb_ok {
                    problems.push(Problem {
                        code: "composite-fallback-malformed",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} composite declares fallback_policy:\"explicit-only\" but its fallback is not a valid cli executor {{kind:\"cli\", command}} — the fallback is silently dropped; fix or remove it (D2)."),
                        agent: None,
                    });
                }
                continue;
            }
            if str_eq(vobj.get("kind"), "native") {
                let model_ok = matches!(vobj.get("model"), Some(Value::String(m)) if !js_trim(m).is_empty());
                if !model_ok {
                    problems.push(Problem {
                        code: "native-model-missing",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} is a native override (kind:\"native\") but has no non-empty model — the exact catalog model id is required; today this silently reverts to the seeded default (D2)."),
                        agent: None,
                    });
                    continue;
                }
                if let Some(ft) = vobj.get("fork_turns") {
                    if !str_eq(Some(ft), "none") {
                        problems.push(Problem {
                            code: "native-fork-turns-unknown",
                            runtime: Some(rt),
                            slot: Some(slot),
                            message: format!(
                                "models.{rt}.{slot} native override has fork_turns:{} — only \"none\" is valid; a full-history fork rejects model overrides (E2/D2).",
                                jsjson::stringify(ft)
                            ),
                            agent: None,
                        });
                    }
                }
                continue;
            }
            let looks_like_cli = vobj.contains_key("kind") || vobj.contains_key("command");
            if looks_like_cli {
                let kind_ok = str_eq(vobj.get("kind"), "cli");
                let command = vobj.get("command").and_then(|v| v.as_str());
                let command_ok = command.map(|c| !js_trim(c).is_empty()).unwrap_or(false);
                if !kind_ok || !command_ok {
                    problems.push(Problem {
                        code: "cli-malformed",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} looks like a cli executor but is missing kind:\"cli\" or a non-empty command — today this silently reverts to the seeded default; fix or remove it (W-e)."),
                        agent: None,
                    });
                    continue;
                }
                let transport_ok = matches!(vobj.get("promptVia"), Some(Value::String(p)) if !js_trim(p).is_empty());
                if !transport_ok {
                    problems.push(Problem {
                        code: "cli-prompt-transport-missing",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} is a cli executor with no declared prompt transport — set promptVia (e.g. \"stdin\") so the prompt reliably reaches it; never inferred from the command string (B2)."),
                        agent: None,
                    });
                }
                let command = command.unwrap_or("");
                for flag in UNSAFE_CLI_FLAGS {
                    if command.contains(flag) {
                        problems.push(Problem {
                            code: "cli-unsafe-flag",
                            runtime: Some(rt),
                            slot: Some(slot),
                            message: format!("models.{rt}.{slot} command contains \"{flag}\" — a known auto-approve/sandbox-bypass flag; remove it (B6/B7). This is a blocklist of KNOWN-BAD flags, not a positive read-only guarantee."),
                            agent: None,
                        });
                    }
                }
                if ADVICE_CLASS_SLOTS.contains(&slot) {
                    for token in ADVICE_CLASS_WRITABLE_TOKENS {
                        if command.contains(token) {
                            problems.push(Problem {
                                code: "cli-advice-slot-writable",
                                runtime: Some(rt),
                                slot: Some(slot),
                                message: format!("models.{rt}.{slot} is an advice-class cli slot (advisor/review must run read-only, AO8) and its command contains \"{token}\" — a known write-granting sandbox token; remove it. This is a blocklist of KNOWN write-granting tokens, not a positive read-only guarantee."),
                                agent: None,
                            });
                        }
                    }
                }
                continue;
            }
            let model_ok = matches!(vobj.get("model"), Some(Value::String(m)) if !js_trim(m).is_empty());
            if !model_ok {
                problems.push(Problem {
                    code: "model-shape-malformed",
                    runtime: Some(rt),
                    slot: Some(slot),
                    message: format!("models.{rt}.{slot} is an object but neither a valid cli executor nor a valid {{model}} shape — ignored; today this silently reverts to the seeded default."),
                    agent: None,
                });
            }
        }
    }
    problems
}

/// state.mjs readAgentFileModel — regex-free port of the frontmatter probe.
/// Returns (found, model): model None = frontmatter unparseable (or no
/// model line), Some = the trimmed value.
fn read_agent_file_model(file: &Path) -> (bool, Option<String>) {
    let Some(raw) = read_text_opt(file) else {
        return (false, None);
    };
    // /^---\r?\n([\s\S]*?)\r?\n---/ anchored at content start, lazy body.
    let body_start = if let Some(rest) = raw.strip_prefix("---\r\n") {
        Some((5usize, rest))
    } else {
        raw.strip_prefix("---\n").map(|rest| (4usize, rest))
    };
    let Some((offset, rest)) = body_start else {
        return (true, None);
    };
    // First "\n---" after the opening (an optional \r before it is consumed).
    let mut close: Option<usize> = None;
    let bytes = rest.as_bytes();
    let mut k = 0;
    while k + 4 <= bytes.len() {
        if &bytes[k..k + 4] == b"\n---" {
            close = Some(k);
            break;
        }
        k += 1;
    }
    let Some(close) = close else {
        return (true, None);
    };
    let _ = offset;
    let mut frontmatter = &rest[..close];
    if frontmatter.ends_with('\r') {
        frontmatter = &frontmatter[..frontmatter.len() - 1];
    }
    // /^model:\s*(.+)$/m over the frontmatter.
    for line in frontmatter.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if let Some(after) = line.strip_prefix("model:") {
            if !after.is_empty() {
                return (true, Some(js_trim(after).to_string()));
            }
        }
    }
    (true, None)
}

/// state.mjs validateAgentFilesDrift.
fn validate_agent_files_drift(ctx: &Ctx, raw_config: Option<&Value>) -> Vec<Problem> {
    const AGENT_FILE_TIER: [(&str, &str); 3] = [
        ("bee-gather", "generation"),
        ("bee-extract", "extraction"),
        ("bee-review", "review"),
    ];
    let mut problems = Vec::new();
    let raw_models = raw_config.and_then(|c| match c {
        Value::Object(m) => m.get("models"),
        _ => None,
    });
    let models = normalize_models(raw_models);
    for (agent_name, slot) in AGENT_FILE_TIER {
        let file = ctx.root.join(".claude").join("agents").join(format!("{agent_name}.md"));
        let (found, file_model) = read_agent_file_model(&file);
        if !found {
            continue;
        }
        let Some(file_model) = file_model else {
            problems.push(Problem {
                code: "agent-file-malformed",
                runtime: None,
                slot: Some(slot),
                message: format!(".claude/agents/{agent_name}.md has no readable \"model:\" frontmatter line — cannot check drift; re-run onboarding to re-render it."),
                agent: Some(agent_name),
            });
            continue;
        };
        let claude = models.get("claude").and_then(|v| v.as_object());
        let mut value = claude.and_then(|c| c.get(slot));
        if nullish(value) && slot == "review" {
            value = claude.and_then(|c| c.get("generation"));
        }
        let expected: Option<String> = match value {
            Some(Value::String(s)) => Some(s.clone()),
            Some(Value::Object(o)) => o.get("model").and_then(|m| m.as_str()).map(str::to_string),
            _ => None,
        };
        match expected {
            None => problems.push(Problem {
                code: "agent-file-drift",
                runtime: None,
                slot: Some(slot),
                message: format!(".claude/agents/{agent_name}.md declares model: \"{file_model}\" but the {slot} slot is now cli-shaped or unconfigured (no model name) — re-run onboarding to remove the stale file."),
                agent: Some(agent_name),
            }),
            Some(expected) if expected != file_model => problems.push(Problem {
                code: "agent-file-drift",
                runtime: None,
                slot: Some(slot),
                message: format!(".claude/agents/{agent_name}.md declares model: \"{file_model}\" but the configured {slot} model is \"{expected}\" — re-run onboarding to re-render it."),
                agent: Some(agent_name),
            }),
            _ => {}
        }
    }
    problems
}

/// state.mjs resolveProductRoot — root unless config `product_root` points
/// elsewhere; warnings replicated (each call re-reads config).
fn resolve_product_root(ctx: &mut Ctx) -> R<PathBuf> {
    let config = read_config(ctx)?;
    let configured = config.raw.get("product_root").cloned();
    match configured {
        None | Some(Value::Null) => Ok(ctx.root.clone()),
        Some(Value::String(s)) if s.is_empty() => Ok(ctx.root.clone()),
        Some(Value::String(s)) => {
            let resolved = if Path::new(&s).is_absolute() {
                PathBuf::from(&s)
            } else {
                PathBuf::from(normalize_abs_lexical(&format!(
                    "{}{}{}",
                    ctx.root.display(),
                    std::path::MAIN_SEPARATOR,
                    s
                )))
            };
            let is_dir = std::fs::metadata(&resolved).map(|m| m.is_dir()).unwrap_or(false);
            if !is_dir {
                ctx.warn(format!(
                    "bee: config product_root \"{s}\" -> \"{}\" is not an existing directory; product-doc reads (docs/backlog.md, docs/specs/) will find nothing until you fix .bee/config.json product_root. (GitHub #14)",
                    resolved.display()
                ));
            }
            Ok(resolved)
        }
        Some(other) => {
            let ty = match other {
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::Array(_) | Value::Object(_) => "object",
                _ => "object",
            };
            ctx.warn(format!(
                "bee: .bee/config.json product_root must be a string path (got {ty}); ignoring it and using the bee root."
            ));
            Ok(ctx.root.clone())
        }
    }
}

/// worktree-store.mjs readGrants — silent {} on any failure.
fn read_grants(main_store_root: &Path) -> JMap {
    let file = main_store_root.join("runtime").join("worktree-grants.json");
    match read_text_opt(&file).and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
        Some(Value::Object(m)) => m,
        _ => JMap::new(),
    }
}

/// state.mjs controlRootFor(root) -> resolveContext(root).controlRoot ?? root.
///
/// NOTE the argument: resolveContext is handed `root` (main()'s already-
/// resolved storeRoot), NOT cwd. That distinction is what makes this a
/// two-line function rather than a second walk:
///   * ordinary checkout        -> resolveRootsCore(root) is ordinary,
///                                 controlRoot === workspaceRoot === root.
///   * UNGRANTED linked worktree-> `root` already fell back to mainRoot, and
///                                 resolveRootsCore(mainRoot) is ordinary,
///                                 so controlRoot === root again.
///   * GRANTED linked worktree  -> `root` IS the worktree checkout, so
///                                 resolveRootsCore(root) is linked-valid and
///                                 controlRoot === its mainRoot.
/// So the only case that moves is the granted one, and `ctx.linked` (resolved
/// from the same cwd `root` was) already carries its mainRoot.
///
/// The resolveContext side effects Node performs anyway are replicated for
/// warning parity: readGrants over the MAIN store's `.bee` (silent), and
/// resolveProductRoot(workspaceRoot) — workspaceRoot is `root` in all three
/// cases above, so `resolve_product_root(ctx)` is exact — with its warnings.
fn control_root_for(ctx: &mut Ctx) -> R<PathBuf> {
    let control = match ctx.granted_worktree() {
        Some(l) => l.main_root.clone(),
        None => ctx.root.clone(),
    };
    let _ = read_grants(&control.join(".bee"));
    let _ = resolve_product_root(ctx)?;
    Ok(control)
}

/// reservations.mjs's cycle-safe controlRootFor replica: a pure git walk-up
/// (findMainRoot), NO config read — that module cannot import state.mjs's
/// controlRootFor without a cycle, so it carries its own. Ordinary git root
/// -> that root; a `.git` FILE -> the bidirectionally-validated mainRoot;
/// anything malformed or no git at all -> `root` (findMainRoot's null).
///
/// This is NOT the same walk as `control_root_for` above even though the two
/// agree on every shape bee actually produces: this one starts at `root` and
/// consults no grant registry, so it answers mainRoot for a granted worktree
/// via the git link alone.
fn reservations_control_root(ctx: &Ctx) -> PathBuf {
    let walk = || -> Option<PathBuf> {
        // locateGitRoot(root)
        let mut dir: Option<&Path> = Some(&ctx.root);
        let (work_root, marker) = loop {
            let d = dir?;
            let marker = d.join(".git");
            if marker.exists() {
                break (d.to_path_buf(), marker);
            }
            dir = d.parent();
        };
        if !std::fs::metadata(&marker).ok()?.is_file() {
            return Some(work_root); // ordinary checkout: mainRoot === workRoot
        }
        let read_ptr = |file: &Path, base: &Path| -> Option<String> {
            let raw = read_text_opt(file)?;
            let raw = js_trim(&raw);
            if raw.is_empty() {
                return None;
            }
            let raw = match raw.strip_prefix("gitdir:") {
                Some(rest) => js_trim(rest),
                None => raw,
            };
            let fixed = raw.replace('\\', &std::path::MAIN_SEPARATOR.to_string());
            Some(path_resolve(base, &fixed))
        };
        let gitdir = read_ptr(&marker, &work_root)?;
        let worktrees_root = path_resolve(Path::new(&gitdir), "..");
        let common_git_dir = path_resolve(Path::new(&worktrees_root), "..");
        if path_basename(&common_git_dir) != ".git" || path_basename(&worktrees_root) != "worktrees"
        {
            return None;
        }
        let id = path_basename(&gitdir);
        if id.is_empty() || id == "." || id == ".." {
            return None;
        }
        let reverse = read_ptr(&Path::new(&gitdir).join("gitdir"), Path::new(&gitdir))?;
        if path_resolve(Path::new(&reverse), ".") != path_resolve(&marker, ".") {
            return None;
        }
        Some(PathBuf::from(path_dirname(&common_git_dir)))
    };
    walk().unwrap_or_else(|| ctx.root.clone())
}

// ─── lanes (state.mjs) ─────────────────────────────────────────────────────

fn lanes_dir(ctx: &Ctx) -> PathBuf {
    ctx.root.join(".bee").join("lanes")
}

/// state.mjs requireLaneFeature — Err = the JS throw (bad name).
fn require_lane_feature(value: &str) -> Result<String, ()> {
    let feature = js_trim(value);
    if feature.is_empty() {
        return Err(());
    }
    if feature.contains('\\') || feature.contains('/') || feature.contains("..") {
        return Err(());
    }
    Ok(feature.to_string())
}

fn default_lane_record(feature: &str) -> JMap {
    let mut m = JMap::new();
    m.insert("schema_version".into(), json!("1.0"));
    m.insert("feature".into(), json!(feature));
    m.insert("mode".into(), Value::Null);
    m.insert("phase".into(), json!("idle"));
    m.insert("approved_gates".into(), Value::Object(default_gates()));
    m.insert("summary".into(), json!(""));
    m.insert("next_action".into(), json!(""));
    m.insert("created_at".into(), Value::Null);
    m
}

/// state.mjs laneRecordFrom — None when not an object naming THIS feature.
/// Truthy non-object approved_gates -> bail (JS-exotic spread).
fn lane_record_from(feature: &str, parsed: Option<&Value>) -> R<Option<JMap>> {
    let Some(Value::Object(obj)) = parsed else { return Ok(None) };
    if !str_eq(obj.get("feature"), feature) {
        return Ok(None);
    }
    let mut merged = default_lane_record(feature);
    for (k, v) in obj {
        merged.insert(k.clone(), v.clone());
    }
    let gates = match obj.get("approved_gates") {
        None | Some(Value::Null) | Some(Value::Bool(false)) => default_gates(),
        Some(Value::String(s)) if s.is_empty() => default_gates(),
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => default_gates(),
        Some(Value::Object(overlay)) => {
            let mut g = default_gates();
            for (k, v) in overlay {
                g.insert(k.clone(), v.clone());
            }
            g
        }
        Some(_) => return Err(Ex::Bail),
    };
    merged.insert("approved_gates".into(), Value::Object(gates));
    if str_eq(merged.get("phase"), "validating") {
        merged.insert("phase".into(), json!("planning"));
    }
    Ok(Some(merged))
}

/// state.mjs readLane — fail-open display read; a present-but-corrupt record
/// warns in Node (both the readJson V8 warning AND readLane's own line) ->
/// bail. A record that parses but mismatches feature warns readLane's line
/// only — deterministic text, but it always accompanies a mismatch that Node
/// still renders as null; replicated verbatim.
fn read_lane(ctx: &mut Ctx, feature: &str) -> R<Option<JMap>> {
    let Ok(id) = require_lane_feature(feature) else {
        return Ok(None);
    };
    let file = lanes_dir(ctx).join(format!("{id}.json"));
    if !file.exists() {
        return Ok(None);
    }
    let parsed = rj(&file)?; // corrupt -> bail (Node prints the V8 warning first)
    let trimmed = js_trim(feature).to_string();
    let record = lane_record_from(&trimmed, parsed.as_ref())?;
    if record.is_none() {
        // Node: console.warn with path.relative(root, file) — POSIX-ish only
        // when file sits under root (always true here).
        let rel = format!(".bee{sep}lanes{sep}{id}.json", sep = std::path::MAIN_SEPARATOR);
        ctx.warn(format!(
            "readLane: skipping corrupt lane record \"{rel}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
        ));
        return Ok(None);
    }
    Ok(record)
}

/// state.mjs listLanes — fail-open enumeration in directory order.
fn list_lanes(ctx: &mut Ctx) -> R<Vec<JMap>> {
    let Ok(entries) = std::fs::read_dir(lanes_dir(ctx)) else {
        return Ok(Vec::new());
    };
    // Node readdirSync returns the OS enumeration order; Rust read_dir uses
    // the same OS API, so the order is preserved rather than re-sorted.
    let names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let mut lanes = Vec::new();
    for entry in names {
        let Some(stem) = entry.strip_suffix(".json") else { continue };
        if let Some(record) = read_lane(ctx, stem)? {
            lanes.push(record);
        }
    }
    Ok(lanes)
}

// ─── cells (cells.mjs) ─────────────────────────────────────────────────────

fn cells_dir(ctx: &Ctx) -> PathBuf {
    ctx.root.join(".bee").join("cells")
}

const ARCHIVE_DIR_NAME: &str = "archive";

/// cells.mjs ID_PATTERN /^[A-Za-z0-9][A-Za-z0-9._-]*$/ over String(id).
fn id_pattern_ok(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

/// cells.mjs listCells (includeArchived always false on the status path).
/// feature/status filters use JS strict !==; sort by id, numeric 'en'.
fn list_cells(ctx: &Ctx, feature: Option<&Value>, status: Option<&str>) -> R<Vec<Value>> {
    let dir = cells_dir(ctx);
    let mut cells: Vec<Value> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue; // the `archive` child (or any dir) is never a cell
            }
            if !name.ends_with(".json") {
                continue;
            }
            let Some(cell) = rj(&entry.path())? else { continue };
            if !matches!(cell, Value::Object(_) | Value::Array(_)) {
                continue; // `typeof cell !== 'object'` (null already skipped)
            }
            if let Some(f) = feature {
                if truthy(f) && !strict_eq(vget(&cell, "feature"), Some(f)) {
                    continue;
                }
            }
            if let Some(s) = status {
                if !str_eq(vget(&cell, "status"), s) {
                    continue;
                }
            }
            cells.push(cell);
        }
    }
    cells.sort_by(|a, b| locale_cmp(&tpl(vget(a, "id")), &tpl(vget(b, "id")), true));
    Ok(cells)
}

/// cells.mjs readCell — active file first, then the archive fallback.
fn read_cell(ctx: &Ctx, id: &Value) -> R<Option<Value>> {
    let id_str = jsjson::js_to_string(id);
    if !truthy(id) || !id_pattern_ok(&id_str) {
        return Ok(None);
    }
    let active = rj(&cells_dir(ctx).join(format!("{id_str}.json")))?;
    if active.is_some() {
        return Ok(active);
    }
    let archive_root = cells_dir(ctx).join(ARCHIVE_DIR_NAME);
    let Ok(entries) = std::fs::read_dir(&archive_root) else {
        return Ok(None);
    };
    for entry in entries.filter_map(|e| e.ok()) {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let candidate = entry.path().join(format!("{id_str}.json"));
        if let Some(v) = rj(&candidate)? {
            return Ok(Some(v));
        }
    }
    Ok(None)
}

/// cells.mjs archivedTotals over the archive summary ledger.
fn archived_totals(ctx: &Ctx) -> R<JMap> {
    let file = cells_dir(ctx).join(ARCHIVE_DIR_NAME).join("summary.json");
    let summary = match rj(&file)? {
        Some(Value::Object(m)) => m,
        _ => JMap::new(),
    };
    let (mut capped, mut dropped) = (0f64, 0f64);
    for entry in summary.values() {
        let Value::Object(e) = entry else { continue };
        if let Some(n) = e.get("capped").and_then(|v| v.as_f64()) {
            if n.is_finite() {
                capped += n;
            }
        }
        if let Some(n) = e.get("dropped").and_then(|v| v.as_f64()) {
            if n.is_finite() {
                dropped += n;
            }
        }
    }
    let mut out = JMap::new();
    out.insert("capped".into(), json_num(capped));
    out.insert("dropped".into(), json_num(dropped));
    out.insert("total".into(), json_num(capped + dropped));
    Ok(out)
}

/// JS number -> Value (whole f64 collapses to integer like JSON.stringify).
fn json_num(n: f64) -> Value {
    if n.is_finite() && n.fract() == 0.0 && n.abs() < 9.007_199_254_740_992e15 {
        json!(n as i64)
    } else if n.is_finite() {
        json!(n)
    } else {
        Value::Null // JSON.stringify(NaN/Infinity) -> null
    }
}

/// cells.mjs readyCells: open cells whose deps are all capped.
fn ready_cells(ctx: &Ctx, feature: Option<&Value>) -> R<Vec<Value>> {
    let open = list_cells(ctx, feature, Some("open"))?;
    let mut ready = Vec::new();
    for cell in open {
        let mut all_capped = true;
        if let Some(Value::Array(deps)) = vget(&cell, "deps") {
            for dep in deps {
                let dep_cell = read_cell(ctx, dep)?;
                let capped = dep_cell
                    .as_ref()
                    .map(|c| str_eq(vget(c, "status"), "capped"))
                    .unwrap_or(false);
                if !capped {
                    all_capped = false;
                }
            }
        }
        if all_capped {
            ready.push(cell);
        }
    }
    Ok(ready)
}

/// cells.mjs readScribingLedger.
fn read_scribing_ledger(ctx: &Ctx) -> Vec<Value> {
    read_jsonl(&ctx.root.join(".bee").join("logs").join("scribing-runs.jsonl"))
}

/// cells.mjs scribingRunStampMs.
fn scribing_run_stamp_ms(run: Option<&Value>) -> Option<f64> {
    let run = run?;
    if !truthy(run) {
        return None;
    }
    // Date.parse(run.at || run.date) — JS ||.
    let at = vget(run, "at").filter(|v| truthy(v));
    let chosen = at.or_else(|| vget(run, "date"));
    let parsed = date_parse_val(chosen);
    if parsed.is_finite() {
        Some(parsed)
    } else {
        None
    }
}

/// cells.mjs bestScribingStampMs — ledger max, then the feature's lane stamp,
/// then the default record's stamp when it names this feature.
fn best_scribing_stamp_ms(
    ctx: &mut Ctx,
    feature: &Value,
    ledger: &[Value],
    state: &JMap,
) -> R<Option<f64>> {
    let mut best: Option<f64> = None;
    for entry in ledger {
        if !truthy(entry) || !strict_eq(vget(entry, "feature"), Some(feature)) {
            continue;
        }
        let parsed = date_parse_val(vget(entry, "ts"));
        if parsed.is_finite() && best.map(|b| parsed > b).unwrap_or(true) {
            best = Some(parsed);
        }
    }
    let feature_str = jsjson::js_to_string(feature);
    let lane = read_lane(ctx, &feature_str)?;
    if let Some(lane) = lane {
        if let Some(stamp) = scribing_run_stamp_ms(lane.get("last_scribing_run")) {
            if best.map(|b| stamp > b).unwrap_or(true) {
                best = Some(stamp);
            }
        }
    }
    if let Some(lsr) = state.get("last_scribing_run") {
        if truthy(lsr) && strict_eq(vget(lsr, "feature"), Some(feature)) {
            if let Some(stamp) = scribing_run_stamp_ms(Some(lsr)) {
                if best.map(|b| stamp > b).unwrap_or(true) {
                    best = Some(stamp);
                }
            }
        }
    }
    Ok(best)
}

/// cells.mjs scribingDebt(root) — no opts on the status path.
fn scribing_debt(ctx: &mut Ctx) -> R<JMap> {
    let state = read_state_full(ctx)?;
    let feature = state.get("feature").cloned().unwrap_or(Value::Null);
    let mut out = JMap::new();
    if !truthy(&feature) {
        out.insert("count".into(), json!(0));
        out.insert("cells".into(), json!([]));
        return Ok(out);
    }
    let ledger = read_scribing_ledger(ctx);
    let threshold = best_scribing_stamp_ms(ctx, &feature, &ledger, &state)?.unwrap_or(0.0);
    let capped = list_cells(ctx, Some(&feature), Some("capped"))?;
    let mut ids = Vec::new();
    for cell in capped {
        let trace = vget(&cell, "trace").cloned().unwrap_or(json!({}));
        if !matches!(vget(&trace, "behavior_change"), Some(Value::Bool(true))) {
            continue;
        }
        let capped_at = date_parse_val(vget(&trace, "capped_at"));
        if capped_at.is_finite() && capped_at > threshold {
            ids.push(vget(&cell, "id").cloned().unwrap_or(Value::Null));
        }
    }
    out.insert("count".into(), json!(ids.len()));
    out.insert("cells".into(), Value::Array(ids));
    Ok(out)
}

/// cells.mjs globalScribingDebt — the orphan sweep across every feature.
fn global_scribing_debt(ctx: &mut Ctx) -> R<JMap> {
    let capped = list_cells(ctx, None, Some("capped"))?;
    let cells: Vec<Value> = capped
        .into_iter()
        .filter(|cell| {
            let trace = vget(cell, "trace");
            matches!(trace.and_then(|t| vget(t, "behavior_change")), Some(Value::Bool(true)))
        })
        .collect();
    let mut out = JMap::new();
    if cells.is_empty() {
        out.insert("count".into(), json!(0));
        out.insert("features".into(), json!([]));
        return Ok(out);
    }
    let state = read_state_full(ctx)?;
    let ledger = read_scribing_ledger(ctx);
    let mut stamp_cache: HashMap<String, Option<f64>> = HashMap::new();
    // Insertion-ordered feature -> ids map (JS Map).
    let mut order: Vec<String> = Vec::new();
    let mut by_feature: HashMap<String, Vec<Value>> = HashMap::new();
    for cell in &cells {
        let feature_v = vget(cell, "feature");
        if !opt_truthy(feature_v) {
            continue;
        }
        let feature_v = feature_v.unwrap().clone();
        let feature_key = jsjson::js_to_string(&feature_v);
        let trace = vget(cell, "trace").cloned().unwrap_or(json!({}));
        let capped_at = date_parse_val(vget(&trace, "capped_at"));
        let stamp = match stamp_cache.get(&feature_key) {
            Some(s) => *s,
            None => {
                let s = best_scribing_stamp_ms(ctx, &feature_v, &ledger, &state)?;
                stamp_cache.insert(feature_key.clone(), s);
                s
            }
        };
        let orphaned = match stamp {
            None => true,
            Some(s) => capped_at.is_finite() && capped_at > s,
        };
        if !orphaned {
            continue;
        }
        if !by_feature.contains_key(&feature_key) {
            order.push(feature_key.clone());
            by_feature.insert(feature_key.clone(), Vec::new());
        }
        by_feature
            .get_mut(&feature_key)
            .unwrap()
            .push(vget(cell, "id").cloned().unwrap_or(Value::Null));
    }
    // sort((a,b) => a.feature.localeCompare(b.feature, 'en')) — non-numeric.
    order.sort_by(|a, b| locale_cmp(a, b, false));
    let mut features = Vec::new();
    let mut count = 0usize;
    for feature in order {
        let ids = by_feature.remove(&feature).unwrap_or_default();
        count += ids.len();
        let mut row = JMap::new();
        row.insert("feature".into(), json!(feature));
        row.insert("cells".into(), Value::Array(ids));
        features.push(Value::Object(row));
    }
    out.insert("count".into(), json!(count));
    out.insert("features".into(), Value::Array(features));
    Ok(out)
}

struct TierMix {
    counts: JMap,
    tiered: i64,
    ceiling: i64,
    ceiling_share: f64,
}

/// cells.mjs tierMix.
fn tier_mix(ctx: &Ctx, feature: Option<&Value>) -> R<TierMix> {
    // tierMix passes {} (no filter) when feature is null.
    let filter = feature.filter(|f| truthy(f));
    let cells = list_cells(ctx, filter, None)?;
    let (mut extraction, mut generation, mut ceiling, mut untiered) = (0i64, 0i64, 0i64, 0i64);
    for cell in &cells {
        match vget(cell, "tier").and_then(|t| t.as_str()) {
            Some(t) if MODEL_TIERS.contains(&t) => match t {
                "extraction" => extraction += 1,
                "generation" => generation += 1,
                _ => ceiling += 1,
            },
            _ => untiered += 1,
        }
    }
    let tiered = extraction + generation + ceiling;
    let ceiling_share = if tiered > 0 { ceiling as f64 / tiered as f64 } else { 0.0 };
    let mut counts = JMap::new();
    counts.insert("extraction".into(), json!(extraction));
    counts.insert("generation".into(), json!(generation));
    counts.insert("ceiling".into(), json!(ceiling));
    counts.insert("untiered".into(), json!(untiered));
    Ok(TierMix { counts, tiered, ceiling, ceiling_share })
}

/// cells.mjs ceilingScarcityWarning.
fn ceiling_scarcity_warning(ctx: &mut Ctx) -> R<Option<JMap>> {
    let state = read_state_full(ctx)?;
    let feature = state.get("feature").cloned().unwrap_or(Value::Null);
    let feature_arg = if truthy(&feature) { Some(feature) } else { None };
    let mix = tier_mix(ctx, feature_arg.as_ref())?;
    if mix.tiered < SCARCITY_MIN_TIERED {
        return Ok(None);
    }
    if mix.ceiling_share <= CEILING_MAX_SHARE {
        return Ok(None);
    }
    let mut out = JMap::new();
    out.insert("pct".into(), json_num(js_round(mix.ceiling_share * 100.0)));
    out.insert("ceiling".into(), json!(mix.ceiling));
    out.insert("tiered".into(), json!(mix.tiered));
    Ok(Some(out))
}

// ─── claims / sessions (claims.mjs) ────────────────────────────────────────

fn sessions_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("sessions")
}

fn claims_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("claims")
}

/// claims.mjs requireId — Err = the JS throw.
fn require_id(value: &str) -> Result<String, ()> {
    let id = js_trim(value);
    if id.is_empty() || id.contains('\\') || id.contains('/') || id.contains("..") {
        return Err(());
    }
    Ok(id.to_string())
}

/// claims.mjs readSession (strict=false).
fn read_session(root: &Path, session_id: &str) -> R<Option<JMap>> {
    let Ok(id) = require_id(session_id) else {
        return Ok(None);
    };
    let file = sessions_dir(root).join(format!("{id}.json"));
    let Some(session) = rj(&file)? else { return Ok(None) };
    let Value::Object(m) = session else { return Ok(None) };
    if !str_eq(m.get("id"), js_trim(session_id)) {
        return Ok(None);
    }
    Ok(Some(m))
}

/// claims.mjs listSessionRecords (strict=false), directory order.
fn list_session_records(root: &Path) -> R<Vec<JMap>> {
    let Ok(entries) = std::fs::read_dir(sessions_dir(root)) else {
        return Ok(Vec::new());
    };
    let mut sessions = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".json") else { continue };
        if let Some(record) = read_session(root, stem)? {
            sessions.push(record);
        }
    }
    Ok(sessions)
}

/// claims.mjs heartbeatStale.
fn heartbeat_stale(session: &JMap, now_ms_v: f64) -> bool {
    let beat = date_parse_val(session.get("last_heartbeat"));
    if !beat.is_finite() {
        return true;
    }
    beat + DEFAULT_HEARTBEAT_STALE_SECONDS * 1000.0 <= now_ms_v
}

/// claims.mjs resolveSessionId — flag(unused here)/env/env-legacy, then the
/// D5 single-live-session adoption when `root` is supplied.
fn resolve_session_id(root: Option<&Path>) -> R<Option<String>> {
    for var in ["BEE_SESSION_ID", "CLAUDE_CODE_SESSION_ID"] {
        if let Ok(v) = std::env::var(var) {
            if !js_trim(&v).is_empty() {
                return Ok(Some(js_trim(&v).to_string()));
            }
        }
    }
    if let Some(root) = root {
        let now = now_ms();
        let fresh: Vec<JMap> = list_session_records(root)?
            .into_iter()
            .filter(|s| !heartbeat_stale(s, now))
            .collect();
        if fresh.len() == 1 {
            return Ok(fresh[0].get("id").and_then(|v| v.as_str()).map(str::to_string));
        }
    }
    Ok(None)
}

/// claims.mjs readClaim.
fn read_claim(root: &Path, cell_id: &str) -> R<Option<JMap>> {
    let Ok(id) = require_id(cell_id) else {
        return Err(Ex::Thrown); // claimPath's requireId throw
    };
    let Some(claim) = rj(&claims_dir(root).join(format!("{id}.json")))? else {
        return Ok(None);
    };
    match claim {
        Value::Object(m) => Ok(Some(m)),
        _ => Ok(None), // `typeof claim !== 'object'` / null
    }
}

/// claims.mjs isClaimExpired/isClaimActive.
fn is_claim_active(claim: &JMap, now_ms_v: f64) -> bool {
    let ttl = claim.get("ttl_seconds").and_then(|v| v.as_f64());
    let Some(ttl) = ttl else { return true };
    if !ttl.is_finite() || ttl <= 0.0 {
        return true;
    }
    let claimed = date_parse_val(claim.get("claimed_at"));
    if !claimed.is_finite() {
        return true;
    }
    claimed + ttl * 1000.0 > now_ms_v
}

/// claims.mjs activeWorkers — live-heartbeat sessions joined with their
/// first active claim, one row per session.
fn active_workers(root: &Path, exclude_session_id: Option<&str>) -> R<Vec<JMap>> {
    let exclude = exclude_session_id.map(js_trim).unwrap_or("");
    let now = now_ms();
    let live: Vec<JMap> = list_session_records(root)?
        .into_iter()
        .filter(|s| !str_eq(s.get("id"), exclude) && !heartbeat_stale(s, now))
        .collect();
    if live.is_empty() {
        return Ok(Vec::new());
    }
    let mut claim_cell_by_session: HashMap<String, Value> = HashMap::new();
    if let Ok(entries) = std::fs::read_dir(claims_dir(root)) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(stem) = name.strip_suffix(".json") else { continue };
            let claim = match read_claim(root, stem) {
                Ok(c) => c,
                Err(Ex::Thrown) => continue, // "not a plain cell id" filename
                Err(e) => return Err(e),
            };
            let Some(claim) = claim else { continue };
            let session = claim.get("session");
            if !opt_truthy(session) || !is_claim_active(&claim, now) {
                continue;
            }
            // JS Map keyed by claim.session — only string keys are ever
            // retrievable against a session id.
            if let Some(Value::String(s)) = session {
                claim_cell_by_session
                    .entry(s.clone())
                    .or_insert_with(|| claim.get("cell").cloned().unwrap_or(Value::Null));
            }
        }
    }
    let mut rows = Vec::new();
    for session in live {
        let mut row = JMap::new();
        // { session_id: session.id } — undefined would be dropped by
        // JSON.stringify; readSession guarantees a string id.
        row.insert("session_id".into(), session.get("id").cloned().unwrap_or(Value::Null));
        let lane = match session.get("lane") {
            Some(Value::String(s)) if !s.is_empty() => json!(s),
            _ => Value::Null,
        };
        row.insert("lane".into(), lane);
        let sid = session.get("id").and_then(|v| v.as_str()).unwrap_or("");
        row.insert(
            "cell".into(),
            claim_cell_by_session.get(sid).cloned().unwrap_or(Value::Null),
        );
        match session.get("last_heartbeat") {
            Some(v) => {
                row.insert("last_heartbeat".into(), v.clone());
            }
            None => {} // undefined -> key dropped by JSON.stringify
        }
        rows.push(row);
    }
    Ok(rows)
}

// ─── reservations over the lease store (reservations.mjs / lease-store.mjs) ─

/// lease-store.mjs listAllLeaseFiles + listLeases (silent per-file skip),
/// re-rooted through reservations.mjs's cycle-safe controlRootFor.
fn list_path_lease_records(ctx: &Ctx) -> Vec<Value> {
    let control = reservations_control_root(ctx);
    let leases_root = control.join(".bee").join("runtime").join("leases");
    let mut records = Vec::new();
    for sub in ["cells", "paths"] {
        let dir = leases_root.join(sub);
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.filter_map(|e| e.ok()) {
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            // readLeaseSafe: silent null on unreadable/corrupt.
            let Some(text) = read_text_opt(&entry.path()) else { continue };
            let Ok(record) = serde_json::from_str::<Value>(&text) else { continue };
            records.push(record);
        }
    }
    // isPathLease filter.
    records
        .into_iter()
        .filter(|r| {
            truthy(r)
                && matches!(vget(r, "resource"), Some(Value::String(s)) if s.starts_with("path:"))
        })
        .collect()
}

/// reservations.mjs isLeaseRecordExpired.
fn is_lease_record_expired(record: &Value, now_ms_v: f64) -> bool {
    let expires = vget(record, "expires_at");
    if nullish(expires) {
        return false;
    }
    let ms = date_parse_val(expires);
    if !ms.is_finite() {
        return false;
    }
    ms <= now_ms_v
}

const SESSIONLESS_SESSION_ID: &str = "\u{0}bee-reservation-sessionless\u{0}";

/// reservations.mjs leaseToReservation — keys inserted only when the JS value
/// is defined (JSON.stringify drops undefined-valued keys).
fn lease_to_reservation(record: &Value) -> JMap {
    let mut out = JMap::new();
    // agent: leaseAgent(record)
    let workspace_id = vget(record, "workspace_id");
    let agent: Option<Value> = match workspace_id {
        Some(Value::String(s)) if s.starts_with("agent:") => Some(json!(&s["agent:".len()..])),
        Some(v) => Some(v.clone()),
        None => None,
    };
    if let Some(a) = agent {
        out.insert("agent".into(), a);
    }
    if let Some(cell) = vget(record, "workflow_id") {
        out.insert("cell".into(), cell.clone());
    }
    if let Some(Value::String(resource)) = vget(record, "resource") {
        out.insert("path".into(), json!(&resource["path:".len()..]));
    }
    // ttl: Math.max(0, Math.round((parse(expires)-parse(acquired))/1000)) or 0.
    let ttl = if nullish(vget(record, "expires_at")) {
        json!(0)
    } else {
        let diff = (date_parse_val(vget(record, "expires_at"))
            - date_parse_val(vget(record, "acquired_at")))
            / 1000.0;
        let rounded = js_round(diff);
        if rounded.is_nan() {
            Value::Null // Math.max(0, NaN) = NaN -> JSON null
        } else {
            json_num(rounded.max(0.0))
        }
    };
    out.insert("ttl_seconds".into(), ttl);
    if let Some(v) = vget(record, "acquired_at") {
        out.insert("reserved_at".into(), v.clone());
    }
    out.insert("released_at".into(), Value::Null);
    if let Some(session) = vget(record, "session_id") {
        if truthy(session) && !str_eq(Some(session), SESSIONLESS_SESSION_ID) {
            out.insert("session".into(), session.clone());
        }
    }
    let kind = vget(record, "kind").filter(|v| truthy(v)).cloned().unwrap_or(json!("lease"));
    out.insert("kind".into(), kind);
    out
}

/// reservations.mjs listReservations.
fn list_reservations(ctx: &Ctx, active_only: bool) -> Vec<JMap> {
    let now = now_ms();
    list_path_lease_records(ctx)
        .into_iter()
        .filter(|r| !active_only || !is_lease_record_expired(r, now))
        .map(|r| lease_to_reservation(&r))
        .collect()
}

// ─── decisions (decisions.mjs) ─────────────────────────────────────────────

fn decisions_path(ctx: &Ctx) -> PathBuf {
    ctx.root.join(".bee").join("decisions.jsonl")
}

/// decisions.mjs buildTagOverlay — last tag event per target wins after a
/// (date, index) stable sort.
fn build_tag_overlay(ctx: &Ctx) -> HashMap<String, (Option<Vec<Value>>, Option<String>)> {
    let events = read_jsonl(&decisions_path(ctx));
    let mut tag_events: Vec<(usize, &Value)> = events
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            truthy(e)
                && str_eq(vget(e, "type"), "tag")
                && matches!(vget(e, "target"), Some(Value::String(_)))
        })
        .collect();
    tag_events.sort_by(|(ai, a), (bi, b)| {
        let ams = date_parse_val(vget(a, "date"));
        let bms = date_parse_val(vget(b, "date"));
        if ams.is_finite() && bms.is_finite() && ams != bms {
            return ams.partial_cmp(&bms).unwrap_or(Ordering::Equal);
        }
        ai.cmp(bi)
    });
    let mut overlay = HashMap::new();
    for (_, event) in tag_events {
        let target = vget(event, "target").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let tags = match vget(event, "tags") {
            Some(Value::Array(a)) => Some(a.clone()),
            _ => None,
        };
        let scope = match vget(event, "scope") {
            Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        };
        overlay.insert(target, (tags, scope));
    }
    overlay
}

fn apply_tag_overlay(
    event: &Value,
    overlay: &HashMap<String, (Option<Vec<Value>>, Option<String>)>,
) -> Value {
    let id = vget(event, "id").and_then(|v| v.as_str());
    let Some(id) = id else { return event.clone() };
    let Some((tags, scope)) = overlay.get(id) else {
        return event.clone();
    };
    let mut next = match event {
        Value::Object(m) => m.clone(),
        _ => return event.clone(),
    };
    if let Some(tags) = tags {
        next.insert("tags".into(), Value::Array(tags.clone()));
    }
    if let Some(scope) = scope {
        next.insert("scope".into(), json!(scope));
    }
    Value::Object(next)
}

/// decisions.mjs activeDecisions (default branch — `all` is never true on
/// the status/orient path): decide/supersede events not superseded/redacted,
/// newest first (reverse file order), overlay applied, optional recent cap.
fn active_decisions(ctx: &Ctx, recent: Option<usize>) -> Vec<Value> {
    let overlay = build_tag_overlay(ctx);
    let events = read_jsonl(&decisions_path(ctx));
    let mut superseded: Vec<Value> = Vec::new();
    let mut redacted: Vec<Value> = Vec::new();
    for event in &events {
        if str_eq(vget(event, "type"), "supersede") && opt_truthy(vget(event, "supersedes")) {
            superseded.push(vget(event, "supersedes").unwrap().clone());
        }
        if str_eq(vget(event, "type"), "redact") && opt_truthy(vget(event, "redacts")) {
            redacted.push(vget(event, "redacts").unwrap().clone());
        }
    }
    let in_set = |set: &[Value], id: Option<&Value>| -> bool {
        // JS Set.has uses SameValueZero; ids are strings in practice.
        set.iter().any(|v| strict_eq(Some(v), id))
    };
    let mut active: Vec<Value> = events
        .iter()
        .filter(|event| {
            let ty = vget(event, "type");
            (str_eq(ty, "decide") || str_eq(ty, "supersede"))
                && !in_set(&superseded, vget(event, "id"))
                && !in_set(&redacted, vget(event, "id"))
        })
        .cloned()
        .collect();
    active.reverse();
    let mut out: Vec<Value> = active.iter().map(|e| apply_tag_overlay(e, &overlay)).collect();
    if let Some(n) = recent {
        out.truncate(n);
    }
    out
}

/// decisions.mjs datamark — neutralize resurfaced text.
fn datamark(text: Option<&Value>) -> String {
    // String(text ?? '')
    let s = match text {
        None | Some(Value::Null) => String::new(),
        Some(v) => jsjson::js_to_string(v),
    };
    // .replace(/```+/g, '') — runs of >= 3 backticks removed.
    let mut no_ticks = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '`' {
            let mut j = i;
            while j < chars.len() && chars[j] == '`' {
                j += 1;
            }
            if j - i >= 3 {
                i = j;
                continue;
            }
            for k in i..j {
                no_ticks.push(chars[k]);
            }
            i = j;
            continue;
        }
        no_ticks.push(chars[i]);
        i += 1;
    }
    // .replace(/<\/?\s*(?:system|assistant|user|developer|tool)\b[^>]*>/gi, '')
    let no_tags = strip_role_tags(&no_ticks);
    // control-char strip (keeps \t \n \r), then trim.
    let cleaned: String = no_tags
        .chars()
        .filter(|c| {
            let cp = *c as u32;
            !(cp <= 0x08 || cp == 0x0B || cp == 0x0C || (0x0E..=0x1F).contains(&cp) || cp == 0x7F)
        })
        .collect();
    format!("«{}»", js_trim(&cleaned))
}

fn strip_role_tags(s: &str) -> String {
    const ROLES: [&str; 5] = ["system", "assistant", "user", "developer", "tool"];
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    'outer: while i < chars.len() {
        if chars[i] == '<' {
            let mut j = i + 1;
            if j < chars.len() && chars[j] == '/' {
                j += 1;
            }
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            for role in ROLES {
                let rl = role.chars().count();
                if j + rl <= chars.len() {
                    let seg: String = chars[j..j + rl].iter().collect::<String>().to_lowercase();
                    if seg == role {
                        let after = j + rl;
                        // \b: next char must not be a word char.
                        let boundary = after >= chars.len()
                            || !(chars[after].is_ascii_alphanumeric() || chars[after] == '_');
                        if boundary {
                            // [^>]*>
                            let mut k = after;
                            while k < chars.len() && chars[k] != '>' {
                                k += 1;
                            }
                            if k < chars.len() {
                                i = k + 1;
                                continue 'outer;
                            }
                        }
                    }
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

// ─── capture queue (capture.mjs) ───────────────────────────────────────────

/// capture.mjs pendingCaptureStubs + captureQueue -> {count, ids}.
fn capture_queue_summary(ctx: &Ctx) -> JMap {
    let events = read_jsonl(&ctx.root.join(".bee").join("capture-queue.jsonl"));
    let mut flushed: Vec<Value> = Vec::new();
    let mut stubs: Vec<&Value> = Vec::new();
    for event in &events {
        if !matches!(event, Value::Object(_)) {
            continue;
        }
        if str_eq(vget(event, "kind"), "flush") && opt_truthy(vget(event, "id")) {
            flushed.push(vget(event, "id").unwrap().clone());
        } else if str_eq(vget(event, "kind"), "stub") && opt_truthy(vget(event, "id")) {
            stubs.push(event);
        }
    }
    let mut pending: Vec<&Value> = stubs
        .into_iter()
        .filter(|s| !flushed.iter().any(|f| strict_eq(Some(f), vget(s, "id"))))
        .collect();
    pending.sort_by(|a, b| locale_cmp(&tpl(vget(a, "at")), &tpl(vget(b, "at")), false));
    let mut out = JMap::new();
    out.insert("count".into(), json!(pending.len()));
    out.insert(
        "ids".into(),
        Value::Array(pending.iter().map(|s| vget(s, "id").cloned().unwrap_or(Value::Null)).collect()),
    );
    out
}

// ─── backlog counts (backlog.mjs) ──────────────────────────────────────────

/// backlog.mjs tokenKey: 'in-flight' -> 'inFlight'.
fn token_key(token: &str) -> String {
    let mut out = String::new();
    let mut upper_next = false;
    for c in token.chars() {
        if c == '-' {
            upper_next = true;
        } else if upper_next {
            out.extend(c.to_uppercase());
            upper_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// backlog.mjs foldPbis + foldedBacklogCounts + legacyBacklogCounts ->
/// readBacklogCounts. Returns None only when neither store parses.
fn read_backlog_counts(ctx: &mut Ctx) -> R<Option<JMap>> {
    // foldPbis over .bee/backlog.jsonl.
    let text = read_text_opt(&ctx.root.join(".bee").join("backlog.jsonl"));
    let mut has_events = false;
    let mut order: Vec<String> = Vec::new();
    let mut items: HashMap<String, String> = HashMap::new(); // id -> status
    if let Some(text) = text {
        for line in text.split('\n') {
            let line = line.strip_suffix('\r').unwrap_or(line);
            let trimmed = js_trim(line);
            if trimmed.is_empty() {
                continue;
            }
            let Ok(row) = serde_json::from_str::<Value>(trimmed) else { continue };
            if !matches!(row, Value::Object(_)) || !str_eq(vget(&row, "kind"), "pbi") {
                continue;
            }
            has_events = true;
            let id = match vget(&row, "id") {
                Some(Value::String(s)) if !s.is_empty() => s.clone(),
                _ => continue,
            };
            let event = vget(&row, "event").and_then(|v| v.as_str()).unwrap_or("");
            match event {
                "add" => {
                    if items.contains_key(&id) {
                        continue;
                    }
                    let status = match vget(&row, "status").and_then(|v| v.as_str()) {
                        Some(s) if PBI_STATUSES.contains(&s) => s.to_string(),
                        _ => "proposed".to_string(),
                    };
                    order.push(id.clone());
                    items.insert(id, status);
                }
                "status" => {
                    if let Some(item) = items.get_mut(&id) {
                        if let Some(s) = vget(&row, "status").and_then(|v| v.as_str()) {
                            if PBI_STATUSES.contains(&s) {
                                *item = s.to_string();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if has_events {
        let mut counts = JMap::new();
        for status in PBI_STATUSES {
            counts.insert(token_key(status), json!(0));
        }
        let mut total = 0i64;
        for status in items.values() {
            if PBI_STATUSES.contains(&status.as_str()) {
                let key = token_key(status);
                let n = counts.get(&key).and_then(|v| v.as_i64()).unwrap_or(0) + 1;
                counts.insert(key, json!(n));
                total += 1;
            }
        }
        counts.insert("total".into(), json!(total));
        return Ok(Some(counts));
    }
    // legacyBacklogCounts over <productRoot>/docs/backlog.md.
    let product_root = resolve_product_root(ctx)?;
    let file = product_root.join("docs").join("backlog.md");
    let Some(text) = read_text_opt(&file) else {
        return Ok(None);
    };
    let mut counts = JMap::new();
    for status in BACKLOG_STATUSES {
        counts.insert(token_key(status), json!(0));
    }
    let normalize_status = |cell: &str| -> String {
        cell.chars()
            .filter(|c| !matches!(c, '*' | '`' | '_'))
            .collect::<String>()
            .trim()
            .to_lowercase()
    };
    let split_row = |line: &str| -> Vec<String> {
        let mut cells: Vec<String> = line.split('|').map(|c| js_trim(c).to_string()).collect();
        if cells.first().map(|c| c.is_empty()).unwrap_or(false) {
            cells.remove(0);
        }
        if cells.last().map(|c| c.is_empty()).unwrap_or(false) {
            cells.pop();
        }
        cells
    };
    let mut status_index: Option<usize> = None;
    for raw_line in text.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if !line.contains('|') {
            continue;
        }
        let cells = split_row(line);
        match status_index {
            None => {
                if let Some(idx) = cells.iter().position(|c| normalize_status(c) == "status") {
                    status_index = Some(idx);
                }
            }
            Some(idx) => {
                if cells.len() <= idx {
                    continue;
                }
                let token = normalize_status(&cells[idx]);
                if BACKLOG_STATUSES.contains(&token.as_str()) {
                    let key = token_key(&token);
                    let n = counts.get(&key).and_then(|v| v.as_i64()).unwrap_or(0) + 1;
                    counts.insert(key, json!(n));
                }
            }
        }
    }
    let total: i64 = BACKLOG_STATUSES
        .iter()
        .map(|s| counts.get(&token_key(s)).and_then(|v| v.as_i64()).unwrap_or(0))
        .sum();
    counts.insert("total".into(), json!(total));
    Ok(Some(counts))
}

// ─── reviews (reviews.mjs) ─────────────────────────────────────────────────

/// reviews.mjs listReviews — fail-open per file; a corrupt session file would
/// print the readJson V8 warning in Node -> bail. A file that parses to a
/// non-object prints only the deterministic skip warning; replicated.
fn list_reviews(ctx: &mut Ctx) -> R<Vec<Value>> {
    let dir = ctx.root.join(".bee").join("reviews");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut sessions = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        let session = rj(&entry.path())?;
        match session {
            Some(v @ Value::Object(_)) => sessions.push(v),
            _ => {
                ctx.warn(format!(
                    "reviews: skipping corrupt session file {name} (list stays fail-open)"
                ));
            }
        }
    }
    sessions.sort_by(|a, b| locale_cmp(&tpl(vget(a, "id")), &tpl(vget(b, "id")), true));
    Ok(sessions)
}

/// reviews.mjs listCandidates.
fn list_candidates(ctx: &Ctx) -> Vec<Value> {
    read_jsonl(&ctx.root.join(".bee").join("review-candidates.jsonl"))
}

/// reviews.mjs sessionCoversCandidate. Property access on a null/undefined
/// candidate is the caller's throw (handled there).
fn session_covers_candidate(session: &Value, candidate: &Value) -> bool {
    let Some(Value::Array(included)) = vget(session, "included") else {
        return false;
    };
    let cand_feature = vget(candidate, "feature");
    let feature_match = included.iter().any(|e| {
        truthy(e) && str_eq(vget(e, "type"), "feature") && strict_eq(vget(e, "id"), cand_feature)
    });
    if feature_match {
        return true;
    }
    let cells: Vec<&Value> = match vget(candidate, "cells") {
        Some(Value::Array(c)) => c.iter().filter(|v| truthy(v)).collect(),
        _ => Vec::new(),
    };
    if cells.is_empty() {
        return false;
    }
    let included_cell_ids: Vec<Option<&Value>> = included
        .iter()
        .filter(|e| truthy(e) && str_eq(vget(e, "type"), "cell"))
        .map(|e| vget(e, "id"))
        .collect();
    cells
        .iter()
        .all(|id| included_cell_ids.iter().any(|iid| strict_eq(*iid, Some(id))))
}

/// reviews.mjs isSessionOpen.
fn is_session_open(session: &Value) -> bool {
    let decision = vget(session, "decision");
    !opt_truthy(decision) || !str_eq(decision.and_then(|d| vget(d, "status")), "approved")
}

enum GitAnswer {
    Covered(Option<bool>, bool), // (covered, unresolved)
    Since(Option<f64>, bool),    // (count, unresolved)
}

fn run_git(root: &Path, args: &[&str]) -> Option<(i32, String)> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    Some((
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    ))
}

/// reviews.mjs headCoveredBy — non-string git args would throw in Node's
/// spawnSync (caught by buildReviewBlock) -> Ex::Thrown.
fn head_covered_by(
    root: &Path,
    head: Option<&Value>,
    ref_: Option<&Value>,
    memo: &mut HashMap<String, GitAnswer>,
) -> R<(Option<bool>, bool)> {
    if strict_eq(head, ref_) {
        return Ok((Some(true), false));
    }
    let key = format!("covered {} {}", tpl(head), tpl(ref_));
    if let Some(GitAnswer::Covered(c, u)) = memo.get(&key) {
        return Ok((*c, *u));
    }
    let (Some(Value::String(h)), Some(Value::String(r))) = (head, ref_) else {
        return Err(Ex::Thrown); // spawnSync TypeError on non-string args
    };
    let result = run_git(root, &["merge-base", "--is-ancestor", h, r]);
    let value = match result {
        Some((0, _)) => (Some(true), false),
        Some((1, _)) => (Some(false), false),
        _ => (None, true),
    };
    memo.insert(key, GitAnswer::Covered(value.0, value.1));
    Ok(value)
}

/// reviews.mjs commitsSince — `${ref}..HEAD` template-coerces.
fn commits_since(
    root: &Path,
    ref_: Option<&Value>,
    memo: &mut HashMap<String, GitAnswer>,
) -> (Option<f64>, bool) {
    let key = format!("since {}", tpl(ref_));
    if let Some(GitAnswer::Since(c, u)) = memo.get(&key) {
        return (*c, *u);
    }
    let range = format!("{}..HEAD", tpl(ref_));
    let result = run_git(root, &["rev-list", &range, "--count"]);
    let value = match result {
        Some((0, stdout)) => {
            // parseInt(trim, 10): leading integer prefix.
            let t = js_trim(&stdout);
            let digits: String = t
                .chars()
                .take_while(|c| c.is_ascii_digit() || (*c == '-' && t.starts_with('-')))
                .collect();
            match digits.parse::<f64>() {
                Ok(n) if n.is_finite() => (Some(n), false),
                _ => (None, true),
            }
        }
        _ => (None, true),
    };
    memo.insert(key, GitAnswer::Since(value.0, value.1));
    value
}

/// reviews.mjs deriveCandidateStatus. Returns (status, session_id, note).
fn derive_candidate_status(
    root: &Path,
    candidate: &Value,
    sessions: &[Value],
    memo: &mut HashMap<String, GitAnswer>,
) -> R<(String, Option<Value>, Option<String>)> {
    // Node would throw a TypeError on a null/undefined candidate at property
    // access; the caller (buildReviewBlock) catches — model as Thrown.
    if matches!(candidate, Value::Null) {
        return Err(Ex::Thrown);
    }
    let covering: Vec<&Value> = sessions
        .iter()
        .filter(|s| session_covers_candidate(s, candidate))
        .collect();
    let open: Vec<&&Value> = covering.iter().filter(|s| is_session_open(s)).collect();
    if !open.is_empty() {
        let session = open[open.len() - 1];
        return Ok(("in review".into(), vget(session, "id").cloned(), None));
    }
    let approved: Vec<&&Value> = covering.iter().filter(|s| !is_session_open(s)).collect();
    let mut unresolved_session: Option<&Value> = None;
    for session in &approved {
        let (covered, unresolved) =
            head_covered_by(root, vget(candidate, "head"), vget(session, "head"), memo)?;
        if unresolved {
            if unresolved_session.is_none() {
                unresolved_session = Some(session);
            }
            continue;
        }
        if covered != Some(true) {
            continue;
        }
        let (count, since_unresolved) = commits_since(root, vget(session, "head"), memo);
        if since_unresolved {
            return Ok((
                "review stale".into(),
                vget(session, "id").cloned(),
                Some("range unresolvable".into()),
            ));
        }
        if count.unwrap_or(0.0) > 0.0 {
            return Ok(("review stale".into(), vget(session, "id").cloned(), None));
        }
        return Ok(("reviewed".into(), vget(session, "id").cloned(), None));
    }
    if let Some(session) = unresolved_session {
        return Ok((
            "review stale".into(),
            vget(session, "id").cloned(),
            Some("range unresolvable".into()),
        ));
    }
    Ok(("unreviewed".into(), None, None))
}

/// bee.mjs buildReviewBlock — fail-open: a Thrown anywhere degrades; Bail
/// propagates (Node would have warned, we delegate).
fn build_review_block(ctx: &mut Ctx) -> R<JMap> {
    let empty = || -> JMap {
        let mut counts = JMap::new();
        counts.insert("total".into(), json!(0));
        counts.insert("unreviewed".into(), json!(0));
        counts.insert("in_review".into(), json!(0));
        counts.insert("reviewed".into(), json!(0));
        counts.insert("stale".into(), json!(0));
        let mut m = JMap::new();
        m.insert("candidates".into(), Value::Object(counts));
        m.insert("open_sessions".into(), json!([]));
        m.insert("high_risk_unreviewed".into(), json!(0));
        m
    };
    let attempt = |ctx: &mut Ctx| -> R<JMap> {
        let candidates = list_candidates(ctx);
        let sessions = list_reviews(ctx)?;
        let (mut unreviewed, mut in_review, mut reviewed, mut stale) = (0i64, 0i64, 0i64, 0i64);
        let mut high_risk_unreviewed = 0i64;
        let mut memo: HashMap<String, GitAnswer> = HashMap::new();
        for candidate in &candidates {
            let (status, _sid, _note) =
                derive_candidate_status(&ctx.root, candidate, &sessions, &mut memo)?;
            match status.as_str() {
                "unreviewed" => unreviewed += 1,
                "in review" => in_review += 1,
                "reviewed" => reviewed += 1,
                "review stale" => stale += 1,
                _ => {}
            }
            if truthy(candidate)
                && str_eq(vget(candidate, "mode"), "high-risk")
                && (status == "unreviewed" || status == "review stale")
            {
                high_risk_unreviewed += 1;
            }
        }
        let open_sessions: Vec<Value> = sessions
            .iter()
            .filter(|s| is_session_open(s))
            .map(|s| vget(s, "id").cloned().unwrap_or(Value::Null))
            .collect();
        let mut counts = JMap::new();
        counts.insert("total".into(), json!(candidates.len()));
        counts.insert("unreviewed".into(), json!(unreviewed));
        counts.insert("in_review".into(), json!(in_review));
        counts.insert("reviewed".into(), json!(reviewed));
        counts.insert("stale".into(), json!(stale));
        let mut m = JMap::new();
        m.insert("candidates".into(), Value::Object(counts));
        m.insert("open_sessions".into(), Value::Array(open_sessions));
        m.insert("high_risk_unreviewed".into(), json!(high_risk_unreviewed));
        Ok(m)
    };
    match attempt(ctx) {
        Ok(m) => Ok(m),
        Err(Ex::Thrown) => {
            let mut m = empty();
            m.insert("degraded".into(), json!(true));
            Ok(m)
        }
        Err(e) => Err(e),
    }
}

// ─── recovery (recovery.mjs) ───────────────────────────────────────────────

/// perf.mjs resolveTranscript.
fn resolve_transcript(
    projects_root: Option<&str>,
    project_path: Option<&str>,
    session_id: Option<&str>,
    transcript_path: Option<&str>,
) -> Option<String> {
    if let Some(tp) = transcript_path {
        let t = js_trim(tp);
        if !t.is_empty() && Path::new(t).exists() {
            return Some(t.to_string());
        }
    }
    let (Some(projects_root), Some(project_path)) = (projects_root, project_path) else {
        return None;
    };
    let dir = PathBuf::from(normalize_abs_lexical(&format!(
        "{}{}{}",
        projects_root,
        std::path::MAIN_SEPARATOR,
        encode_project_dir(project_path)
    )));
    if let Some(sid) = session_id {
        let file = dir.join(format!("{sid}.jsonl"));
        if file.exists() {
            return Some(file.to_string_lossy().into_owned());
        }
        return None;
    }
    None // the newest-mtime branch is never reached from status/orient
}

/// recovery.mjs hasCleanEndTrio.
fn has_clean_end_trio(events: &[Value]) -> bool {
    if events.is_empty() {
        return false;
    }
    let is_conversational =
        |e: &Value| str_eq(vget(e, "type"), "user") || str_eq(vget(e, "type"), "assistant");
    let mut stop_idx: Option<usize> = None;
    for i in (0..events.len()).rev() {
        let e = &events[i];
        if truthy(e)
            && str_eq(vget(e, "type"), "system")
            && str_eq(vget(e, "subtype"), "stop_hook_summary")
        {
            stop_idx = Some(i);
            break;
        }
        if truthy(e) && is_conversational(e) {
            return false;
        }
    }
    let Some(stop_idx) = stop_idx else { return false };
    let mut turn_idx: Option<usize> = None;
    for (i, e) in events.iter().enumerate().skip(stop_idx + 1) {
        if truthy(e)
            && str_eq(vget(e, "type"), "system")
            && str_eq(vget(e, "subtype"), "turn_duration")
        {
            turn_idx = Some(i);
            break;
        }
        if truthy(e) && is_conversational(e) {
            return false;
        }
    }
    let Some(turn_idx) = turn_idx else { return false };
    let mut last_prompt_idx: Option<usize> = None;
    for (i, e) in events.iter().enumerate().skip(turn_idx + 1) {
        if truthy(e) && str_eq(vget(e, "type"), "last-prompt") {
            last_prompt_idx = Some(i);
            break;
        }
        if truthy(e) && is_conversational(e) {
            return false;
        }
    }
    let Some(last_prompt_idx) = last_prompt_idx else { return false };
    for e in events.iter().skip(last_prompt_idx + 1) {
        if truthy(e) && is_conversational(e) {
            return false;
        }
    }
    true
}

/// recovery.mjs eventTimestampMs.
fn event_timestamp_ms(event: &Value) -> f64 {
    if !matches!(event, Value::Object(_)) {
        return f64::NAN;
    }
    if let Some(Value::String(ts)) = vget(event, "timestamp") {
        return js_date_parse(ts);
    }
    if let Some(Value::String(at)) = vget(event, "at") {
        return js_date_parse(at);
    }
    f64::NAN
}

/// recovery.mjs toMs.
fn to_ms(v: Option<&Value>) -> f64 {
    match v {
        None | Some(Value::Null) => f64::NAN,
        Some(Value::String(s)) => js_date_parse(s),
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        Some(Value::Bool(b)) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Some(_) => f64::NAN,
    }
}

struct TranscriptRoot {
    runtime: String,
    path: String,
    scanned: bool,
    reason: Option<String>,
}

/// recovery.mjs scanTranscriptRoots — the Claude default root plus every
/// configured recovery.transcript_roots entry, each probed fresh.
fn scan_transcript_roots(ctx: &mut Ctx, projects_root: &str) -> R<Vec<TranscriptRoot>> {
    let config = read_config(ctx)?;
    let configured_raw = config
        .raw
        .get("recovery")
        .filter(|v| truthy(v))
        .and_then(|r| vget(r, "transcript_roots"))
        .cloned();
    let mut entries: Vec<(String, String, bool)> =
        vec![("claude".into(), projects_root.to_string(), false)];
    if let Some(Value::Array(items)) = configured_raw {
        for entry in items {
            let Value::Object(o) = &entry else { continue };
            let runtime = o.get("runtime").and_then(|v| v.as_str()).map(js_trim).unwrap_or("");
            let root_path = o.get("path").and_then(|v| v.as_str()).map(js_trim).unwrap_or("");
            if runtime.is_empty() || root_path.is_empty() {
                continue;
            }
            entries.push((runtime.to_string(), root_path.to_string(), true));
        }
    }
    let mut out = Vec::new();
    for (runtime, root_path, is_configured) in entries {
        let mut scanned = false;
        let mut reason: Option<String> = None;
        match std::fs::metadata(&root_path) {
            Ok(meta) => {
                scanned = meta.is_dir();
                if !scanned {
                    reason = Some("not-a-directory".into());
                }
            }
            Err(err) => {
                reason = Some(match err.kind() {
                    std::io::ErrorKind::NotFound => "ENOENT".into(),
                    std::io::ErrorKind::PermissionDenied => "EACCES".into(),
                    std::io::ErrorKind::NotADirectory => "ENOTDIR".into(),
                    _ => "unreadable".into(),
                });
            }
        }
        if !scanned && is_configured {
            ctx.warn(format!(
                "recovery: configured transcript root \"{root_path}\" (runtime \"{runtime}\") is {} — skipping (config: recovery.transcript_roots)",
                reason.as_deref().unwrap_or("unreadable")
            ));
        }
        out.push(TranscriptRoot { runtime, path: root_path, scanned, reason });
    }
    Ok(out)
}

/// recovery.mjs lastDurableSettlement with cp-1 injected shared inputs.
fn last_durable_settlement(
    lane: Option<&Value>,
    decisions: &[Value],
    capture_events: &[Value],
    cells: &[Value],
) -> Option<f64> {
    let mut max_ms: Option<f64> = None;
    let mut bump = |ms: f64| {
        if ms.is_finite() && max_ms.map(|m| ms > m).unwrap_or(true) {
            max_ms = Some(ms);
        }
    };
    for event in decisions {
        bump(date_parse_val(if truthy(event) { vget(event, "date") } else { None }));
    }
    let lane_truthy = lane.map(truthy).unwrap_or(false);
    for event in capture_events {
        if !truthy(event) || !str_eq(vget(event, "kind"), "stub") {
            continue;
        }
        if lane_truthy && !strict_eq(vget(event, "lane"), lane) {
            continue;
        }
        bump(date_parse_val(vget(event, "at")));
    }
    for cell in cells {
        if lane_truthy && !strict_eq(vget(cell, "feature"), lane) {
            continue;
        }
        let capped_at = vget(cell, "trace").and_then(|t| vget(t, "capped_at"));
        if opt_truthy(capped_at) {
            bump(date_parse_val(capped_at));
        }
    }
    max_ms
}

/// recovery.mjs sessionHasActiveClaim (control-root claims).
fn session_has_active_claim(control_root: &Path, session_id: &Value, now: f64) -> R<bool> {
    let Ok(entries) = std::fs::read_dir(claims_dir(control_root)) else {
        return Ok(false);
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".json") else { continue };
        let claim = match read_claim(control_root, stem) {
            Ok(c) => c,
            // readClaim's requireId throw propagates in Node (no local catch
            // here) — buildRecoveryBlock's own catch absorbs it.
            Err(e) => return Err(e),
        };
        let Some(claim) = claim else { continue };
        if strict_eq(claim.get("session"), Some(session_id)) && is_claim_active(&claim, now) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// recovery.mjs detectCrashCandidates.
fn detect_crash_candidates(ctx: &mut Ctx, projects_root: &str) -> R<Vec<Value>> {
    // resolveSessionId({flag: null}) — env chain only, no root adoption.
    let resolved_current = {
        let mut found: Option<String> = None;
        for var in ["BEE_SESSION_ID", "CLAUDE_CODE_SESSION_ID"] {
            if let Ok(v) = std::env::var(var) {
                if !js_trim(&v).is_empty() {
                    found = Some(js_trim(&v).to_string());
                    break;
                }
            }
        }
        found
    };
    let control_root = control_root_for(ctx)?;
    let sessions = list_session_records(&control_root)?;
    if sessions.is_empty() {
        return Ok(Vec::new());
    }
    let roots = scan_transcript_roots(ctx, projects_root)?;
    let now = now_ms();
    let project_path = ctx.root.to_string_lossy().into_owned();

    let mut shared: Option<(Vec<Value>, Vec<Value>, Vec<Value>)> = None;
    let mut candidates = Vec::new();
    for session in &sessions {
        if !session.contains_key("id") || !opt_truthy(session.get("id")) {
            continue;
        }
        if let Some(current) = &resolved_current {
            if str_eq(session.get("id"), current) {
                continue;
            }
        }
        if !heartbeat_stale(session, now) {
            continue;
        }
        let mut transcript: Option<String> = None;
        let mut transcript_runtime: Option<String> = None;
        let stored_path = session
            .get("transcript_path")
            .and_then(|v| v.as_str())
            .map(js_trim)
            .filter(|s| !s.is_empty());
        if let Some(stored) = stored_path {
            if let Some(found) = resolve_transcript(None, None, None, Some(stored)) {
                let matched = roots.iter().find(|r| {
                    if !r.scanned {
                        return false;
                    }
                    let sep = std::path::MAIN_SEPARATOR;
                    let prefix = if r.path.ends_with(sep) {
                        r.path.clone()
                    } else {
                        format!("{}{}", r.path, sep)
                    };
                    found.starts_with(&prefix)
                });
                transcript_runtime = matched.map(|r| r.runtime.clone());
                transcript = Some(found);
            }
        }
        if transcript.is_none() {
            let sid = session.get("id").and_then(|v| v.as_str()).unwrap_or("");
            for r in &roots {
                if !r.scanned {
                    continue;
                }
                if let Some(found) =
                    resolve_transcript(Some(&r.path), Some(&project_path), Some(sid), None)
                {
                    transcript = Some(found);
                    transcript_runtime = Some(r.runtime.clone());
                    break;
                }
            }
        }
        let Some(transcript) = transcript else { continue };
        let tail = read_transcript_tail(Path::new(&transcript), DEFAULT_TAIL_MAX_BYTES)?;
        if has_clean_end_trio(&tail) {
            continue;
        }
        // lane = session.lane || null
        let lane: Value = match session.get("lane") {
            Some(v) if truthy(v) => v.clone(),
            _ => Value::Null,
        };
        if shared.is_none() {
            shared = Some((
                active_decisions(ctx, None),
                read_jsonl(&ctx.root.join(".bee").join("capture-queue.jsonl")),
                list_cells(ctx, None, None)?,
            ));
        }
        let (decisions, capture_events, cells) = shared.as_ref().unwrap();
        let since_ms_opt = last_durable_settlement(Some(&lane), decisions, capture_events, cells);
        let since_ms = match since_ms_opt {
            Some(ms) => ms,
            None => to_ms(session.get("started_at")),
        };

        let mut work_signal: Option<&'static str> = None;
        if truthy(&lane) {
            let lane_str = jsjson::js_to_string(&lane);
            let lane_record = read_lane(ctx, &lane_str)?;
            if let Some(record) = lane_record {
                let phase = record.get("phase").and_then(|v| v.as_str()).unwrap_or("");
                if !TERMINAL_LANE_PHASES.contains(&phase) {
                    work_signal = Some("lane");
                }
            }
        }
        if work_signal.is_none() {
            let sid = session.get("id").cloned().unwrap_or(Value::Null);
            if session_has_active_claim(&control_root, &sid, now)? {
                work_signal = Some("claimed_cells");
            }
        }
        if work_signal.is_none() {
            let mut last_activity: Option<f64> = None;
            for event in &tail {
                let t = event_timestamp_ms(event);
                if t.is_finite() && last_activity.map(|l| t > l).unwrap_or(true) {
                    last_activity = Some(t);
                }
            }
            if let Some(last) = last_activity {
                if since_ms.is_finite() && last > since_ms {
                    work_signal = Some("transcript_activity");
                }
            }
        }
        let Some(work_signal) = work_signal else { continue };

        let mut row = JMap::new();
        row.insert("session_id".into(), session.get("id").cloned().unwrap_or(Value::Null));
        row.insert("lane".into(), lane);
        row.insert("transcript".into(), json!(transcript));
        row.insert(
            "runtime".into(),
            transcript_runtime.map(|r| json!(r)).unwrap_or(Value::Null),
        );
        // started_at/last_heartbeat: `session.x || null`.
        let started = match session.get("started_at") {
            Some(v) if truthy(v) => v.clone(),
            _ => Value::Null,
        };
        row.insert("started_at".into(), started);
        let heartbeat = match session.get("last_heartbeat") {
            Some(v) if truthy(v) => v.clone(),
            _ => Value::Null,
        };
        row.insert("last_heartbeat".into(), heartbeat);
        row.insert("work_signal".into(), json!(work_signal));
        row.insert(
            "since".into(),
            if since_ms.is_finite() { json!(to_iso(since_ms)) } else { Value::Null },
        );
        candidates.push(Value::Object(row));
    }
    Ok(candidates)
}

/// bee.mjs buildRecoveryBlock — Thrown degrades, Bail propagates.
fn build_recovery_block(ctx: &mut Ctx) -> R<JMap> {
    let attempt = |ctx: &mut Ctx| -> R<JMap> {
        let projects_root = claude_projects_root();
        let candidates = detect_crash_candidates(ctx, &projects_root)?;
        let roots = scan_transcript_roots(ctx, &projects_root)?;
        let mut m = JMap::new();
        m.insert("candidates".into(), Value::Array(candidates));
        m.insert(
            "roots".into(),
            Value::Array(
                roots
                    .into_iter()
                    .map(|r| {
                        let mut o = JMap::new();
                        o.insert("runtime".into(), json!(r.runtime));
                        o.insert("path".into(), json!(r.path));
                        o.insert("scanned".into(), json!(r.scanned));
                        o.insert(
                            "reason".into(),
                            r.reason.map(|x| json!(x)).unwrap_or(Value::Null),
                        );
                        Value::Object(o)
                    })
                    .collect(),
            ),
        );
        Ok(m)
    };
    match attempt(ctx) {
        Ok(m) => Ok(m),
        Err(Ex::Thrown) => {
            let mut m = JMap::new();
            m.insert("candidates".into(), json!([]));
            m.insert("degraded".into(), json!(true));
            Ok(m)
        }
        Err(e) => Err(e),
    }
}

// ─── runtime drift + source identity (bee.mjs / source-identity.mjs) ───────

/// bee.mjs computeRuntimeDrift — live vendored-file hashes vs the onboarding
/// ledger's managed map; fail-open to the version-only signal.
fn compute_runtime_drift(ctx: &Ctx, onboarding_raw: &Value) -> (bool, Vec<String>) {
    let version_drift = truthy(onboarding_raw) && {
        let v = vget(onboarding_raw, "bee_version");
        opt_truthy(v) && !str_eq(v, BEE_VERSION)
    };
    let managed = vget(onboarding_raw, "managed");
    let Some(managed @ (Value::Object(_) | Value::Array(_))) = managed else {
        return (version_drift, Vec::new());
    };
    let mut detail: Vec<String> = Vec::new();
    let mut check_group = |recorded: Option<&Value>, rel_dir: &str| {
        let Some(Value::Object(recorded)) = recorded else { return };
        for (name, recorded_hash) in recorded {
            let abs = if rel_dir.is_empty() {
                ctx.root.join(".bee").join("bin").join(name)
            } else {
                ctx.root.join(".bee").join("bin").join(rel_dir).join(name)
            };
            let rel_posix = if rel_dir.is_empty() {
                format!(".bee/bin/{name}")
            } else {
                format!(".bee/bin/{rel_dir}/{name}")
            };
            match hash_file(&abs) {
                None => detail.push(format!("{rel_posix} (missing)")),
                Some(live) => {
                    if !str_eq(Some(recorded_hash), &live) {
                        detail.push(rel_posix);
                    }
                }
            }
        }
    };
    check_group(vget(managed, "lib"), "lib");
    check_group(vget(managed, "helpers"), "");
    check_group(vget(managed, "prompts"), "prompts");
    if let Some(Value::Object(lib)) = vget(managed, "lib") {
        if let Ok(entries) = std::fs::read_dir(ctx.root.join(".bee").join("bin").join("lib")) {
            for entry in entries.filter_map(|e| e.ok()) {
                let f = entry.file_name().to_string_lossy().into_owned();
                if f.ends_with(".mjs") && !lib.contains_key(&f) {
                    detail.push(format!(".bee/bin/lib/{f} (extra)"));
                }
            }
        }
    }
    (version_drift || !detail.is_empty(), detail)
}

/// bee.mjs findRepoHive — canonical-first candidate order.
fn find_repo_hive(ctx: &Ctx) -> Option<PathBuf> {
    for segs in [vec!["skills"], vec![".claude", "skills"], vec![".agents", "skills"]] {
        let mut p = ctx.root.clone();
        for s in &segs {
            p = p.join(s);
        }
        let hive = p.join("bee-hive");
        if hive.exists() {
            return Some(hive);
        }
    }
    None
}

/// source-identity.mjs classifySource — only the (kind, root) pair status
/// consumes.
fn classify_source(hive_dir: &Path, home: &str) -> (String, Option<String>) {
    let source_root = hive_dir.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let plugin_root = source_root.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let plugin_root_str = plugin_root.to_string_lossy().into_owned();
    if source_root.join(".bee-render.json").exists() {
        return ("rendered_projection".into(), Some(plugin_root_str));
    }
    if !home.is_empty() {
        let global_root = PathBuf::from(normalize_abs_lexical(&format!(
            "{}{sep}.claude{sep}skills",
            home,
            sep = std::path::MAIN_SEPARATOR
        )));
        let rp = dunce::canonicalize(&source_root).ok();
        let rp_global = dunce::canonicalize(&global_root).ok();
        if let (Some(a), Some(b)) = (rp, rp_global) {
            if a == b {
                return ("legacy_global".into(), Some(plugin_root_str));
            }
        }
    }
    let projection_parent = path_basename(&plugin_root_str);
    if projection_parent == ".agents" || projection_parent == ".claude" {
        return ("project_projection".into(), Some(plugin_root_str));
    }
    let plugin_manifest = plugin_root.join(".claude-plugin").join("plugin.json");
    if plugin_manifest.exists() {
        // Node: JSON.parse(readFileSync(...,'utf8')) — NO BOM strip here, so a
        // BOM'd manifest parses as unknown, matching that exact behavior.
        let parse_ok = read_text_opt(&plugin_manifest)
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .is_some();
        if !parse_ok {
            return ("unknown".into(), Some(plugin_root_str));
        }
        if plugin_root.join(".git").exists() {
            return ("source_checkout".into(), Some(plugin_root_str));
        }
        return ("plugin_package".into(), Some(plugin_root_str));
    }
    ("unknown".into(), Some(plugin_root_str))
}

// ─── contention summary (bee.mjs buildContentionSummary) ───────────────────

fn build_contention_summary(ctx: &Ctx) -> R<Option<JMap>> {
    let file = ctx.root.join(".bee").join("logs").join("contention.jsonl");
    let events = read_transcript_tail(&file, CONTENTION_TAIL_MAX_BYTES)?;
    if events.is_empty() {
        return Ok(None);
    }
    let busy: Vec<&Value> = events
        .iter()
        .filter(|e| {
            truthy(e)
                && matches!(e, Value::Object(_) | Value::Array(_))
                && str_eq(vget(e, "result"), "busy")
        })
        .collect();
    if busy.is_empty() {
        return Ok(None);
    }
    // byLock: Map insertion order = first-seen.
    let mut lock_order: Vec<String> = Vec::new();
    let mut lock_counts: HashMap<String, i64> = HashMap::new();
    for e in &busy {
        let name = match vget(e, "lock_name") {
            Some(Value::String(s)) if !s.is_empty() => s.clone(),
            _ => "unknown".to_string(),
        };
        if !lock_counts.contains_key(&name) {
            lock_order.push(name.clone());
        }
        *lock_counts.entry(name).or_insert(0) += 1;
    }
    let mut ranked: Vec<(String, i64)> = lock_order
        .iter()
        .map(|n| (n.clone(), lock_counts[n]))
        .collect();
    // sort((a,b) => b[1]-a[1]) — stable, insertion order on ties.
    ranked.sort_by(|a, b| b.1.cmp(&a.1));
    let top_locks: Vec<Value> = ranked
        .into_iter()
        .take(CONTENTION_TOP_LOCKS_LIMIT)
        .map(|(lock_name, busy_count)| {
            let mut o = JMap::new();
            o.insert("lock_name".into(), json!(lock_name));
            o.insert("busy_count".into(), json!(busy_count));
            Value::Object(o)
        })
        .collect();
    let mut worst_wait_ms: Option<f64> = None;
    let mut worst_wait_lock: Value = Value::Null;
    for e in &events {
        if !truthy(e) || !matches!(e, Value::Object(_) | Value::Array(_)) {
            continue;
        }
        let wait = vget(e, "lock_wait_ms")
            .and_then(|v| if v.is_number() { v.as_f64() } else { None })
            .filter(|f| f.is_finite());
        if let Some(w) = wait {
            if worst_wait_ms.map(|m| w > m).unwrap_or(true) {
                worst_wait_ms = Some(w);
                worst_wait_lock = match vget(e, "lock_name") {
                    Some(Value::String(s)) if !s.is_empty() => json!(s),
                    _ => Value::Null,
                };
            }
        }
    }
    let recent_busy: Vec<Value> = busy
        .iter()
        .rev()
        .take(CONTENTION_RECENT_BUSY_LIMIT)
        .map(|e| {
            let mut o = JMap::new();
            o.insert(
                "ts".into(),
                match vget(e, "ts") {
                    Some(Value::String(s)) => json!(s),
                    _ => Value::Null,
                },
            );
            o.insert(
                "lock_name".into(),
                match vget(e, "lock_name") {
                    Some(Value::String(s)) => json!(s),
                    _ => Value::Null,
                },
            );
            // holder/caller: `e.x ?? null`.
            o.insert(
                "holder_session".into(),
                match vget(e, "holder_session") {
                    None | Some(Value::Null) => Value::Null,
                    Some(v) => v.clone(),
                },
            );
            o.insert(
                "caller_session".into(),
                match vget(e, "caller_session") {
                    None | Some(Value::Null) => Value::Null,
                    Some(v) => v.clone(),
                },
            );
            o.insert(
                "lock_wait_ms".into(),
                match vget(e, "lock_wait_ms") {
                    Some(v @ Value::Number(_)) => v.clone(),
                    _ => Value::Null,
                },
            );
            Value::Object(o)
        })
        .collect();
    let mut m = JMap::new();
    m.insert("busy_count".into(), json!(busy.len()));
    m.insert("recent_busy".into(), Value::Array(recent_busy));
    m.insert("top_locks".into(), Value::Array(top_locks));
    m.insert(
        "worst_wait_ms".into(),
        worst_wait_ms.map(json_num).unwrap_or(Value::Null),
    );
    m.insert("worst_wait_lock".into(), worst_wait_lock);
    Ok(Some(m))
}

// ─── worktree lookups (worktree-store.mjs / bee.mjs) ───────────────────────

/// worktree-store.mjs resolveWorktreeById — bidirectional gitdir validation.
fn resolve_worktree_by_id(main_root: &Path, id: &str) -> Option<String> {
    let git_worktree_dir = main_root.join(".git").join("worktrees").join(id);
    let meta = std::fs::metadata(&git_worktree_dir).ok()?;
    if !meta.is_dir() {
        return None;
    }
    let forward_raw = read_text_opt(&git_worktree_dir.join("gitdir"))?;
    let forward_raw = js_trim(&forward_raw);
    if forward_raw.is_empty() {
        return None;
    }
    let resolved_git_file = path_resolve(&git_worktree_dir, forward_raw);
    let worktree_root = path_dirname(&resolved_git_file);
    let reverse_raw = read_text_opt(Path::new(&worktree_root).join(".git").as_path())?;
    let reverse_raw = js_trim(&reverse_raw);
    // /^gitdir:\s*(.+)$/ over the single-line trimmed content.
    let rest = reverse_raw.strip_prefix("gitdir:")?;
    // /^gitdir:\s*(.+)$/ — \s* consumes any whitespace, then (.+)$ demands a
    // single newline-free remainder.
    let target = rest.trim_start();
    if target.is_empty() || target.contains('\n') {
        return None;
    }
    let reverse_resolved = path_resolve(Path::new(&worktree_root), js_trim(target));
    if !crate::path_identity::canonical_paths_equal(
        Path::new(&reverse_resolved),
        &git_worktree_dir,
    ) {
        return None;
    }
    Some(worktree_root)
}

/// worktree-store.mjs readWorktreeCreationFeature / readWorktreeStateFeature.
fn read_worktree_feature(worktree_root: &str) -> Option<String> {
    let read_feature = |file: PathBuf| -> Option<String> {
        let raw = read_text_opt(&file)?;
        let parsed = serde_json::from_str::<Value>(&raw).ok()?;
        match vget(&parsed, "feature") {
            Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        }
    };
    let base = Path::new(worktree_root).join(".bee");
    read_feature(base.join("runtime").join("worktree-identity.json"))
        .or_else(|| read_feature(base.join("state.json")))
}

/// worktree-store.mjs findGrantedWorktreeForFeature — never throws.
fn find_granted_worktree_for_feature(main_root: &Path, feature: &str) -> Option<(String, String)> {
    if feature.is_empty() {
        return None;
    }
    let grants = read_grants(&main_root.join(".bee"));
    for (id, granted) in &grants {
        if !matches!(granted, Value::Bool(true)) {
            continue;
        }
        let Some(worktree_root) = resolve_worktree_by_id(main_root, id) else {
            continue;
        };
        let identity = read_worktree_feature(&worktree_root);
        if identity.as_deref() == Some(feature) {
            return Some((id.clone(), worktree_root));
        }
    }
    None
}

/// bee.mjs ungrantedWorktreeNotice (GH #30) — messaging only, and the ONE
/// status field whose presence depends on the checkout's grant state.
///
/// Node re-resolves `process.cwd()` here rather than reading `root`, because
/// an UNGRANTED linked worktree's `root` already fell back to the main store
/// (the P40 default) and is therefore indistinguishable from an ordinary
/// checkout. `ctx.linked` is that same resolution. Emitted only when the
/// checkout is linked-valid AND storeRoot === mainRoot; an ordinary checkout
/// and a GRANTED worktree both omit the key entirely, byte-identical to the
/// pre-flip output. `null` from the throw arm is unreachable here: a
/// WorktreeLinkInvalidError already delegated the whole command.
const UNGRANTED_WORKTREE_NOTICE: &str = "⚠ This linked worktree is UNGRANTED — it SHARES the main checkout's store (same feature/phase/claims; no isolation). To work an isolated feature: run \"bee worktree new --feature <slug>\" from the main checkout. To grant isolation to THIS existing worktree instead: run \"bee worktree register --feature <slug>\" from inside it.";

fn ungranted_worktree_notice(ctx: &Ctx) -> Option<String> {
    let linked = ctx.linked.as_ref()?;
    if !linked.ungranted() {
        return None;
    }
    Some(UNGRANTED_WORKTREE_NOTICE.to_string())
}

/// bee.mjs readWorktreeBranch — best-effort branch for a granted worktree's
/// orient block. The linked worktree's HEAD lives at
/// `<mainRoot>/.git/worktrees/<id>/HEAD`; null on any failure or detached
/// HEAD. The Node regex is `/^ref:\s*refs\/heads\/(.+)$/` over the TRIMMED
/// file: `.` never matches a line terminator and there is no `m` flag, so the
/// captured branch must be non-empty and run to the very end of the string.
fn read_worktree_branch(main_root: &Path, id: &str) -> Option<String> {
    let head = read_text_opt(
        &main_root
            .join(".git")
            .join("worktrees")
            .join(id)
            .join("HEAD"),
    )?;
    let head = js_trim(&head);
    let rest = head.strip_prefix("ref:")?;
    // \s* — JS whitespace, the same class js_trim strips.
    let rest = rest.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    let branch = rest.strip_prefix("refs/heads/")?;
    if branch.is_empty() || branch.contains(['\n', '\r', '\u{2028}', '\u{2029}']) {
        return None;
    }
    Some(branch.to_string())
}

/// bee.mjs isCodeTouchingLane.
fn is_code_touching_lane(lane: Option<&Value>, other_live_session: bool) -> bool {
    if !opt_truthy(lane) || str_eq(lane, "docs") {
        return false;
    }
    if str_eq(lane, "tiny") && !other_live_session {
        return false;
    }
    true
}

/// bee.mjs otherLiveWorkPresent — D9a live-session signal; fails quiet.
fn other_live_work_present(ctx: &mut Ctx) -> R<bool> {
    let attempt = |ctx: &mut Ctx| -> R<bool> {
        let ctrl_root = control_root_for(ctx)?;
        let self_id = resolve_session_id(Some(&ctrl_root))?;
        let others = active_workers(&ctrl_root, self_id.as_deref())?;
        if others.is_empty() {
            return Ok(false);
        }
        for worker in &others {
            let record: Option<JMap> = match worker.get("lane") {
                Some(v) if truthy(v) => read_lane(ctx, &jsjson::js_to_string(v))?,
                _ => Some(read_state_full(ctx)?),
            };
            let phase = record.as_ref().and_then(|r| r.get("phase"));
            if opt_truthy(phase)
                && !str_eq(phase, "idle")
                && !str_eq(phase, "compounding-complete")
            {
                return Ok(true);
            }
        }
        Ok(false)
    };
    match attempt(ctx) {
        Ok(v) => Ok(v),
        Err(Ex::Thrown) => Ok(false), // Node's own catch -> false
        Err(e) => Err(e),
    }
}

// ─── lane rows / summary (bee.mjs buildLaneRows / buildLaneSummary) ────────

/// bee.mjs buildLaneRows — every lane record plus its bound session ids.
/// NOTE: the sessions come through bee.mjs's LOCAL listSessionRecords wrapper
/// (bee.mjs:3986), which re-roots through controlRootFor (msn-18c) — one
/// extra readConfig (and its warnings) per call, unlike claims.mjs's own
/// bare-root listSessionRecords.
fn build_lane_rows(ctx: &mut Ctx) -> R<Vec<JMap>> {
    let lanes = list_lanes(ctx)?;
    let ctrl_root = control_root_for(ctx)?;
    let sessions = list_session_records(&ctrl_root)?;
    let mut bound_by: HashMap<String, Vec<Value>> = HashMap::new();
    for session in &sessions {
        if let Some(Value::String(lane)) = session.get("lane") {
            if !lane.is_empty() {
                bound_by
                    .entry(lane.clone())
                    .or_default()
                    .push(session.get("id").cloned().unwrap_or(Value::Null));
            }
        }
    }
    Ok(lanes
        .into_iter()
        .map(|mut lane| {
            let key = lane.get("feature").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let bound = bound_by.get(&key).cloned().unwrap_or_default();
            lane.insert("bound_sessions".into(), Value::Array(bound));
            lane
        })
        .collect())
}

/// bee.mjs buildLaneSummary (lpsp-2): active lane in full + counts/ids.
fn build_lane_summary(ctx: &mut Ctx) -> R<JMap> {
    let lanes = build_lane_rows(ctx)?;
    let mut out = JMap::new();
    if lanes.is_empty() {
        out.insert("active".into(), Value::Null);
        out.insert("counts".into(), json!({}));
        out.insert("ids".into(), json!([]));
        return Ok(out);
    }
    let ctrl_root = control_root_for(ctx)?;
    let session_id = resolve_session_id(Some(&ctrl_root))?;
    let mut active: Option<JMap> = None;
    if let Some(session_id) = session_id {
        if let Some(session) = read_session(&ctrl_root, &session_id)? {
            if let Some(Value::String(lane)) = session.get("lane") {
                if !lane.is_empty() {
                    active = lanes
                        .iter()
                        .find(|l| str_eq(l.get("feature"), lane))
                        .cloned();
                }
            }
        }
    }
    let active_feature = active
        .as_ref()
        .and_then(|a| a.get("feature"))
        .cloned();
    let rest: Vec<&JMap> = lanes
        .iter()
        .filter(|l| match (&active, &active_feature) {
            (Some(_), Some(f)) => !strict_eq(l.get("feature"), Some(f)),
            _ => true,
        })
        .collect();
    let mut counts = JMap::new();
    for l in &rest {
        let key = tpl(l.get("phase"));
        let n = counts.get(&key).and_then(|v| v.as_i64()).unwrap_or(0) + 1;
        counts.insert(key, json!(n));
    }
    out.insert(
        "active".into(),
        active.map(Value::Object).unwrap_or(Value::Null),
    );
    out.insert("counts".into(), Value::Object(counts));
    out.insert(
        "ids".into(),
        Value::Array(rest.iter().map(|l| l.get("feature").cloned().unwrap_or(Value::Null)).collect()),
    );
    Ok(out)
}

// ─── buildStatus (bee.mjs ~874-1047) ───────────────────────────────────────

fn build_status(ctx: &mut Ctx, lanes_full: bool) -> R<JMap> {
    let state = read_state_full(ctx)?;
    let onboarding_raw = read_onboarding(ctx)?.unwrap_or(Value::Null);
    let handoff = read_handoff(ctx)?.unwrap_or(Value::Null);
    let cells = list_cells(ctx, None, None)?;
    let (mut open, mut claimed, mut capped, mut blocked) = (0i64, 0i64, 0i64, 0i64);
    for cell in &cells {
        match vget(cell, "status").and_then(|v| v.as_str()) {
            Some("open") => open += 1,
            Some("claimed") => claimed += 1,
            Some("capped") => capped += 1,
            Some("blocked") => blocked += 1,
            _ => {}
        }
    }
    let archived = archived_totals(ctx)?;
    let all_reservations = list_reservations(ctx, false);
    let active_res = list_reservations(ctx, true);
    // expiredUnreleased: `active.includes(r)` compares OBJECT REFERENCES from
    // two separate listReservations calls — always false in Node, faithfully
    // replicated: every released_at==null row counts (released_at is always
    // null by construction).
    let expired_unreleased = all_reservations
        .iter()
        .filter(|r| nullish(r.get("released_at")))
        .count();

    let config1 = read_config(ctx)?; // readConfig #1: `commands`
    let commands = config1.commands.clone();
    let backlog = read_backlog_counts(ctx)?;

    let mut staleness: Vec<Value> = Vec::new();
    if commands.is_empty() {
        staleness.push(json!(
            "No standard commands recorded — capture the host project's setup/start/test/verify into .bee/config.json `commands` so sessions can run the CI status gate."
        ));
    }
    if truthy(&onboarding_raw) {
        let v = vget(&onboarding_raw, "bee_version");
        if opt_truthy(v) && !str_eq(v, BEE_VERSION) {
            staleness.push(json!(format!(
                "Onboarding installed bee {} but plugin is {BEE_VERSION} — re-run onboarding.",
                tpl(v)
            )));
        }
    }
    if truthy(&handoff) && opt_truthy(vget(&handoff, "written_at")) {
        let age = now_ms() - date_parse_val(vget(&handoff, "written_at"));
        if age.is_finite() && age > STALE_HANDOFF_MS {
            staleness.push(json!(format!(
                "HANDOFF.json is older than 7 days (written {}).",
                tpl(vget(&handoff, "written_at"))
            )));
        }
    }
    if expired_unreleased > 0 {
        staleness.push(json!(format!(
            "{expired_unreleased} reservation(s) expired but never released — run bee_reservations.mjs sweep."
        )));
    }
    if has_stale_advisor_key(ctx)? {
        staleness.push(json!(STALE_ADVISOR_KEY_WARNING));
    }
    let raw_for_validation = read_raw_config_for_validation(ctx)?;
    for problem in validate_models_config(raw_for_validation.as_ref()) {
        // `${problem.runtime ? ` models.${runtime}.${slot}:` : ''}` — slot is
        // explicitly null on runtime-level rows, templating as "null".
        let runtime_part = match (problem.runtime, problem.slot) {
            (Some(rt), slot) if !rt.is_empty() => {
                format!(" models.{rt}.{}:", slot.unwrap_or("null"))
            }
            _ => String::new(),
        };
        staleness.push(json!(format!(
            "config validate [{}]{} {}",
            problem.code, runtime_part, problem.message
        )));
    }
    let raw_for_validation2 = read_raw_config_for_validation(ctx)?;
    for problem in validate_agent_files_drift(ctx, raw_for_validation2.as_ref()) {
        staleness.push(json!(format!(
            "config validate [{}] {} ({}): {}",
            problem.code,
            problem.agent.unwrap_or("undefined"),
            problem.slot.unwrap_or("undefined"),
            problem.message
        )));
    }
    let phase_known = state
        .get("phase")
        .and_then(|v| v.as_str())
        .map(|p| KNOWN_PHASES.contains(&p))
        .unwrap_or(false);
    if !phase_known {
        staleness.push(json!(format!(
            "Unknown phase \"{}\" — not in the enum ({}; terminal alias: compounding-complete). Set state.phase to a valid value (idle at feature close); invented phases break machine-checkable handoffs (decision 0004).",
            tpl(state.get("phase")),
            PHASES.join(", ")
        )));
    }
    let review = build_review_block(ctx)?;
    let recovery = build_recovery_block(ctx)?;

    let execution_approved = state
        .get("approved_gates")
        .map(|g| matches!(vget(g, "execution"), Some(Value::Bool(true))))
        .unwrap_or(false);
    let feature_or_null = state
        .get("feature")
        .filter(|f| truthy(f))
        .cloned();
    let ready = ready_cells(ctx, feature_or_null.as_ref())?;
    let unreviewed_count = review
        .get("candidates")
        .and_then(|c| vget(c, "unreviewed"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let recommended: Value = if !truthy(&onboarding_raw) {
        json!("Onboarding missing — run bee-hive onboarding.")
    } else if truthy(&handoff) {
        json!("HANDOFF present — present it to the user and WAIT. Never auto-resume.")
    } else if str_eq(state.get("phase"), "swarming") && !execution_approved {
        json!("NOT ready to swarm: gate \"execution\" is not approved.")
    } else if execution_approved && !ready.is_empty() {
        let ids: Vec<Value> = ready.iter().map(|c| vget(c, "id").cloned().unwrap_or(Value::Null)).collect();
        json!(format!(
            "{} ready cell(s): {} — orchestrator assigns them.",
            ready.len(),
            js_join(&ids, ", ")
        ))
    } else if state
        .get("phase")
        .and_then(|v| v.as_str())
        .map(|p| POST_EXECUTION_REVIEW_PHASES.contains(&p))
        .unwrap_or(false)
        && unreviewed_count > 0.0
    {
        json!(format!(
            "{} review candidate(s) awaiting: full review is user-invoked only, never dispatched automatically.",
            tpl(review.get("candidates").and_then(|c| vget(c, "unreviewed")))
        ))
    } else {
        match state.get("next_action") {
            Some(v) if truthy(v) => v.clone(),
            _ => json!("Invoke bee-hive."),
        }
    };

    let (drift_flag, drift_detail) = compute_runtime_drift(ctx, &onboarding_raw);
    let repo_hive = find_repo_hive(ctx);
    let (source_kind, source_root) = match &repo_hive {
        Some(hive) => classify_source(hive, &home_dir()),
        None => ("unknown".into(), None),
    };
    let worktree_notice = ungranted_worktree_notice(ctx);
    let contention = build_contention_summary(ctx)?;

    // Return-object literal: property VALUES evaluate top to bottom, so the
    // remaining readConfig-bearing calls happen in this exact order.
    let mut status = JMap::new();
    {
        let mut onboarding = JMap::new();
        onboarding.insert("installed".into(), json!(truthy(&onboarding_raw)));
        onboarding.insert(
            "bee_version".into(),
            match vget(&onboarding_raw, "bee_version") {
                None | Some(Value::Null) => Value::Null,
                Some(v) => v.clone(),
            },
        );
        onboarding.insert("plugin_version".into(), json!(BEE_VERSION));
        onboarding.insert("drift".into(), json!(drift_flag));
        if !drift_detail.is_empty() {
            onboarding.insert(
                "drift_detail".into(),
                Value::Array(drift_detail.iter().map(|d| json!(d)).collect()),
            );
        }
        status.insert("onboarding".into(), Value::Object(onboarding));
    }
    {
        let mut source = JMap::new();
        source.insert("kind".into(), json!(source_kind));
        source.insert(
            "root".into(),
            source_root.map(|r| json!(r)).unwrap_or(Value::Null),
        );
        status.insert("source".into(), Value::Object(source));
    }
    status.insert("phase".into(), state.get("phase").cloned().unwrap_or(Value::Null));
    status.insert("mode".into(), state.get("mode").cloned().unwrap_or(Value::Null));
    status.insert("feature".into(), state.get("feature").cloned().unwrap_or(Value::Null));
    status.insert(
        "gates".into(),
        state.get("approved_gates").cloned().unwrap_or(Value::Null),
    );
    let level1 = bypass_level_root(ctx)?; // readConfig
    status.insert("gate_bypass".into(), json!(level1 != "off"));
    let level2 = bypass_level_root(ctx)?; // readConfig (Node calls it twice)
    status.insert("gate_bypass_level".into(), json!(level2));
    let ship = ship_visibility(ctx)?; // readConfig
    status.insert("ship_visibility".into(), json!(ship));
    status.insert(
        "route".into(),
        match state.get("route") {
            None | Some(Value::Null) => Value::Null,
            Some(v) => v.clone(),
        },
    );
    let config_models = read_config(ctx)?; // readConfig: `models`
    status.insert("models".into(), Value::Object(config_models.models.clone()));
    {
        let mix = tier_mix(ctx, feature_or_null.as_ref())?;
        let mut tm = JMap::new();
        tm.insert("counts".into(), Value::Object(mix.counts));
        tm.insert("tiered".into(), json!(mix.tiered));
        tm.insert(
            "ceilingShare".into(),
            if mix.ceiling_share.fract() == 0.0 {
                json!(mix.ceiling_share as i64)
            } else {
                json!(mix.ceiling_share)
            },
        );
        status.insert("tier_mix".into(), Value::Object(tm));
    }
    status.insert(
        "ceiling_scarcity".into(),
        ceiling_scarcity_warning(ctx)?.map(Value::Object).unwrap_or(Value::Null),
    );
    status.insert("handoff".into(), handoff.clone());
    {
        let mut c = JMap::new();
        c.insert("open".into(), json!(open));
        c.insert("claimed".into(), json!(claimed));
        c.insert("capped".into(), json!(capped));
        c.insert("blocked".into(), json!(blocked));
        c.insert("archived".into(), Value::Object(archived));
        status.insert("cells".into(), Value::Object(c));
    }
    if lanes_full {
        let rows = build_lane_rows(ctx)?;
        status.insert(
            "lanes".into(),
            Value::Array(rows.into_iter().map(Value::Object).collect()),
        );
    } else {
        status.insert("lanes".into(), Value::Object(build_lane_summary(ctx)?));
    }
    status.insert("review".into(), Value::Object(review));
    status.insert("recovery".into(), Value::Object(recovery));
    {
        let mut sd = scribing_debt(ctx)?;
        sd.insert("orphaned".into(), Value::Object(global_scribing_debt(ctx)?));
        status.insert("scribing_debt".into(), Value::Object(sd));
    }
    status.insert("capture_queue".into(), Value::Object(capture_queue_summary(ctx)));
    status.insert(
        "pbi".into(),
        match &backlog {
            Some(counts) => {
                let mut p = JMap::new();
                p.insert("proposed".into(), counts.get("proposed").cloned().unwrap_or(Value::Null));
                p.insert("in_flight".into(), counts.get("inFlight").cloned().unwrap_or(Value::Null));
                p.insert("done".into(), counts.get("done").cloned().unwrap_or(Value::Null));
                Value::Object(p)
            }
            None => Value::Null,
        },
    );
    status.insert("commands".into(), Value::Object(commands));
    status.insert(
        "active_reservations".into(),
        Value::Array(active_res.into_iter().map(Value::Object).collect()),
    );
    {
        let ctrl_root = control_root_for(ctx)?; // readConfig inside resolveContext
        let workers = active_workers(&ctrl_root, None)?;
        status.insert(
            "workers".into(),
            Value::Array(workers.into_iter().map(Value::Object).collect()),
        );
    }
    status.insert(
        "critical_patterns_present".into(),
        json!(ctx
            .root
            .join("docs")
            .join("history")
            .join("learnings")
            .join("critical-patterns.md")
            .exists()),
    );
    {
        let recent = active_decisions(ctx, Some(3));
        let rows: Vec<Value> = recent
            .iter()
            .map(|event| {
                let mut o = JMap::new();
                match vget(event, "id") {
                    Some(v) => {
                        o.insert("id".into(), v.clone());
                    }
                    None => {} // undefined -> dropped by JSON.stringify
                }
                match vget(event, "date") {
                    Some(v) => {
                        o.insert("date".into(), v.clone());
                    }
                    None => {}
                }
                o.insert("decision".into(), json!(datamark(vget(event, "decision"))));
                Value::Object(o)
            })
            .collect();
        status.insert("recent_decisions".into(), Value::Array(rows));
    }
    status.insert("staleness_warnings".into(), Value::Array(staleness));
    status.insert("recommended_next".into(), recommended);
    if let Some(notice) = worktree_notice {
        status.insert("worktree_notice".into(), json!(notice));
    }
    if let Some(contention) = contention {
        status.insert("contention".into(), Value::Object(contention));
    }
    Ok(status)
}

// ─── renderStatusText (bee.mjs ~1081-1206) ─────────────────────────────────

/// bee.mjs formatSlot.
fn format_slot(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "null".to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(v) => {
            if str_eq(vget(v, "kind"), "cli") {
                let command = tpl(vget(v, "command"));
                let first = if command.starts_with(|c: char| c.is_whitespace()) {
                    ""
                } else {
                    command.split_whitespace().next().unwrap_or("")
                };
                return format!("cli({first})");
            }
            if opt_truthy(vget(v, "model")) {
                let model = tpl(vget(v, "model"));
                if opt_truthy(vget(v, "effort")) {
                    return format!("{model}@{}", tpl(vget(v, "effort")));
                }
                return model;
            }
            "null".to_string()
        }
    }
}

/// bee.mjs formatLaneRow.
fn format_lane_row(l: &Value) -> String {
    let gates = GATE_NAMES
        .iter()
        .map(|g| {
            let approved = opt_truthy(vget(l, "approved_gates").and_then(|ag| vget(ag, g)));
            format!("{g}={}", if approved { "approved" } else { "pending" })
        })
        .collect::<Vec<_>>()
        .join(" ");
    let bound = match vget(l, "bound_sessions") {
        Some(Value::Array(items)) if !items.is_empty() => {
            format!(" sessions={}", js_join(items, ","))
        }
        _ => String::new(),
    };
    format!("{} [{}] {gates}{bound}", tpl(vget(l, "feature")), tpl(vget(l, "phase")))
}

/// bee.mjs formatLaneSummaryLine — None = no line at all.
fn format_lane_summary_line(summary: &Value) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if opt_truthy(vget(summary, "active")) {
        parts.push(format!("active: {}", format_lane_row(vget(summary, "active").unwrap())));
    }
    if let Some(Value::Array(ids)) = vget(summary, "ids") {
        if !ids.is_empty() {
            let counts_str = match vget(summary, "counts") {
                Some(Value::Object(counts)) => counts
                    .iter()
                    .map(|(phase, n)| format!("{phase}={}", jsjson::js_to_string(n)))
                    .collect::<Vec<_>>()
                    .join(" "),
                _ => String::new(),
            };
            parts.push(format!(
                "{} other lane(s) [{counts_str}] (ids: {})",
                ids.len(),
                js_join(ids, ", ")
            ));
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(format!("Lanes: {}", parts.join(" | ")))
    }
}

fn render_status_text(status: &JMap) -> String {
    let s = |k: &str| status.get(k);
    let mut lines: Vec<String> = Vec::new();
    if opt_truthy(s("worktree_notice")) {
        lines.push(tpl(s("worktree_notice")));
    }
    lines.push(format!("bee status (plugin v{BEE_VERSION})"));
    {
        let onboarding = s("onboarding").cloned().unwrap_or(Value::Null);
        let installed = opt_truthy(vget(&onboarding, "installed"));
        let base = if installed {
            format!("installed (bee {})", tpl(vget(&onboarding, "bee_version")))
        } else {
            "MISSING".to_string()
        };
        let drift = if opt_truthy(vget(&onboarding, "drift")) {
            let detail = if opt_truthy(vget(&onboarding, "drift_detail")) {
                let n = vget(&onboarding, "drift_detail")
                    .and_then(|d| d.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                format!(": {n} file(s)")
            } else {
                String::new()
            };
            format!(" [drift{detail}]")
        } else {
            String::new()
        };
        lines.push(format!("Onboarding: {base}{drift}"));
    }
    lines.push(format!(
        "Phase: {} | Mode: {} | Feature: {}",
        tpl(s("phase")),
        if nullish(s("mode")) { "none".to_string() } else { tpl(s("mode")) },
        if nullish(s("feature")) { "none".to_string() } else { tpl(s("feature")) },
    ));
    lines.push(format!(
        "Gates: {}",
        GATE_NAMES
            .iter()
            .map(|g| {
                let approved = opt_truthy(s("gates").and_then(|gs| vget(gs, g)));
                format!("{g}={}", if approved { "approved" } else { "pending" })
            })
            .collect::<Vec<_>>()
            .join(" ")
    ));
    let level = s("gate_bypass_level");
    if opt_truthy(level) && !str_eq(level, "off") {
        lines.push(bypass_banner(&tpl(level)).to_string());
    }
    lines.push(format!(
        "Handoff: {}",
        if opt_truthy(s("handoff")) { "PRESENT — surface it and WAIT" } else { "none" }
    ));
    {
        let cells = s("cells").cloned().unwrap_or(Value::Null);
        let archived = vget(&cells, "archived").cloned().unwrap_or(Value::Null);
        let capped = vget(&cells, "capped").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
        let arch_capped = vget(&archived, "capped").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
        lines.push(format!(
            "Cells: open={} claimed={} capped={} blocked={} archived={} (total capped={})",
            tpl(vget(&cells, "open")),
            tpl(vget(&cells, "claimed")),
            tpl(vget(&cells, "capped")),
            tpl(vget(&cells, "blocked")),
            tpl(vget(&archived, "total")),
            jsjson::js_f64_to_string(capped + arch_capped),
        ));
    }
    match s("lanes") {
        Some(Value::Array(rows)) => {
            if !rows.is_empty() {
                lines.push(format!(
                    "Lanes: {}",
                    rows.iter().map(format_lane_row).collect::<Vec<_>>().join(" | ")
                ));
            }
        }
        Some(summary) => {
            if let Some(line) = format_lane_summary_line(summary) {
                lines.push(line);
            }
        }
        None => {}
    }
    let unreviewed = s("review")
        .and_then(|r| vget(r, "candidates"))
        .and_then(|c| vget(c, "unreviewed"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let phase_post_exec = s("phase")
        .and_then(|v| v.as_str())
        .map(|p| POST_EXECUTION_REVIEW_PHASES.contains(&p))
        .unwrap_or(false);
    if phase_post_exec && unreviewed > 0.0 {
        lines.push(format!(
            "Completed and verified; independent review not requested; {} candidate(s) awaiting review.",
            tpl(s("review").and_then(|r| vget(r, "candidates")).and_then(|c| vget(c, "unreviewed")))
        ));
    }
    {
        let sd = s("scribing_debt");
        if opt_truthy(sd) && vget(sd.unwrap(), "count").and_then(|v| v.as_f64()).unwrap_or(0.0) > 0.0 {
            let sd = sd.unwrap();
            let cells = match vget(sd, "cells") {
                Some(Value::Array(a)) => js_join(a, ", "),
                _ => String::new(),
            };
            lines.push(format!(
                "Capture pending: {} behavior_change cell(s) uncaptured ({cells}) — run bee-capturing when you choose (decision c8e25271; batching features is fine)",
                tpl(vget(sd, "count"))
            ));
        }
    }
    {
        let cq = s("capture_queue");
        if opt_truthy(cq) && vget(cq.unwrap(), "count").and_then(|v| v.as_f64()).unwrap_or(0.0) > 0.0 {
            lines.push(format!(
                "Capture queue pending: {} stub(s) awaiting flush — run bee-capturing when you choose (decision c8e25271), and before compact/clear",
                tpl(vget(cq.unwrap(), "count"))
            ));
        }
    }
    if opt_truthy(s("pbi")) {
        let pbi = s("pbi").unwrap();
        lines.push(format!(
            "PBI: {} done / {} in-flight / {} proposed",
            tpl(vget(pbi, "done")),
            tpl(vget(pbi, "in_flight")),
            tpl(vget(pbi, "proposed"))
        ));
    }
    {
        let commands = s("commands");
        let parts: Vec<String> = COMMAND_KEYS
            .iter()
            .filter(|key| opt_truthy(commands.and_then(|c| vget(c, key))))
            .map(|key| format!("{key}={}", tpl(commands.and_then(|c| vget(c, key)))))
            .collect();
        let joined = parts.join(" | ");
        lines.push(format!(
            "Standard commands: {}",
            if joined.is_empty() { "none recorded" } else { joined.as_str() }
        ));
    }
    lines.push(format!(
        "Active reservations: {}",
        s("active_reservations").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0)
    ));
    lines.push(format!(
        "Active workers: {}",
        s("workers").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0)
    ));
    if opt_truthy(s("contention")) {
        let c = s("contention").unwrap();
        let top = match vget(c, "top_locks") {
            Some(Value::Array(locks)) => locks
                .iter()
                .map(|l| format!("{}×{}", tpl(vget(l, "lock_name")), tpl(vget(l, "busy_count"))))
                .collect::<Vec<_>>()
                .join(", "),
            _ => String::new(),
        };
        let worst_lock = if opt_truthy(vget(c, "worst_wait_lock")) {
            format!(" on \"{}\"", tpl(vget(c, "worst_wait_lock")))
        } else {
            String::new()
        };
        lines.push(format!(
            "Contention: {} LOCK_BUSY event(s) recently (top: {top}); worst wait {}ms{worst_lock}",
            tpl(vget(c, "busy_count")),
            tpl(vget(c, "worst_wait_ms"))
        ));
    }
    lines.push(format!(
        "Critical patterns file: {}",
        if opt_truthy(s("critical_patterns_present")) { "present" } else { "absent" }
    ));
    if opt_truthy(s("models")) {
        let claude = s("models").and_then(|m| vget(m, "claude"));
        lines.push(format!(
            "Models (claude): generation={} extraction={} review={} · ceiling = the session model (keep it scarce; decisions 0012/0015/0021)",
            format_slot(claude.and_then(|c| vget(c, "generation"))),
            format_slot(claude.and_then(|c| vget(c, "extraction"))),
            format_slot(claude.and_then(|c| vget(c, "review"))),
        ));
    }
    {
        let tm = s("tier_mix");
        if opt_truthy(tm) && vget(tm.unwrap(), "tiered").and_then(|v| v.as_f64()).unwrap_or(0.0) > 0.0 {
            let tm = tm.unwrap();
            let counts = vget(tm, "counts").cloned().unwrap_or(Value::Null);
            let share = vget(tm, "ceilingShare").and_then(|v| v.as_f64()).unwrap_or(0.0);
            lines.push(format!(
                "Tier mix: extraction={} generation={} ceiling={} untiered={} (ceiling {}%)",
                tpl(vget(&counts, "extraction")),
                tpl(vget(&counts, "generation")),
                tpl(vget(&counts, "ceiling")),
                tpl(vget(&counts, "untiered")),
                jsjson::js_f64_to_string(js_round(share * 100.0))
            ));
        }
    }
    if opt_truthy(s("ceiling_scarcity")) {
        let cs = s("ceiling_scarcity").unwrap();
        lines.push(format!(
            "⚠ Ceiling scarcity: {}/{} tiered cells on ceiling ({}%) — re-tier routine cells (decision 0012)",
            tpl(vget(cs, "ceiling")),
            tpl(vget(cs, "tiered")),
            tpl(vget(cs, "pct"))
        ));
    }
    let high_risk = s("review")
        .and_then(|r| vget(r, "high_risk_unreviewed"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    if high_risk > 0.0 {
        lines.push(format!(
            "⚠ High-risk unreviewed: {} high-risk candidate(s) have not passed independent review — bee will not auto-dispatch reviewers; request review before merge/release.",
            tpl(s("review").and_then(|r| vget(r, "high_risk_unreviewed")))
        ));
    }
    if let Some(Value::Array(decisions)) = s("recent_decisions") {
        if !decisions.is_empty() {
            lines.push("Recent decisions:".to_string());
            for d in decisions {
                lines.push(format!("- {} ({})", tpl(vget(d, "decision")), tpl(vget(d, "date"))));
            }
        }
    }
    if let Some(Value::Array(warnings)) = s("staleness_warnings") {
        if !warnings.is_empty() {
            lines.push("Staleness warnings:".to_string());
            for w in warnings {
                lines.push(format!("- {}", jsjson::js_to_string(w)));
            }
        }
    }
    lines.push(format!("Recommended next: {}", tpl(s("recommended_next"))));
    lines.join("\n")
}

// ─── orient (bee.mjs ~1229-1373) ───────────────────────────────────────────

/// bee.mjs orientNextCommand.
fn orient_next_command(status: &JMap, ready_ids: &[Value]) -> Value {
    if opt_truthy(status.get("handoff")) {
        return json!("bee state handoff show --json");
    }
    if !ready_ids.is_empty() {
        return json!("bee cells ready --json");
    }
    Value::Null
}

/// bee.mjs orientDecisionLine — first line, UTF-16 160-cap with '...'.
fn orient_decision_line(decision: Option<&Value>) -> String {
    let s = tpl(decision);
    let first = s.split('\n').next().unwrap_or("");
    let line = js_trim(first);
    let units: Vec<u16> = line.encode_utf16().collect();
    if units.len() > 160 {
        // slice(0, 157) over UTF-16 units; bee decision text is BMP so the
        // lossy re-decode is exact in practice.
        let sliced = String::from_utf16_lossy(&units[..157]);
        format!("{sliced}...")
    } else {
        line.to_string()
    }
}

/// bee.mjs orientWorktreeContext — BOTH halves. Inside a GRANTED worktree the
/// packet carries the merge-back state; from the MAIN checkout with a
/// code-touching active feature that already has a granted worktree, it
/// carries "go there". Null everywhere else, so an ordinary orient with no
/// granted worktree is byte-unchanged. Never throws (Thrown -> None like
/// Node's catch).
fn orient_worktree_context(ctx: &mut Ctx, status: &JMap) -> R<Option<JMap>> {
    let attempt = |ctx: &mut Ctx| -> R<Option<JMap>> {
        // grantedWorktreeContext(): the current checkout when it is a GRANTED
        // linked worktree (its own storeRoot === its own worktreeRoot).
        if let Some(linked) = ctx.granted_worktree() {
            let id = linked.id.clone();
            let branch = read_worktree_branch(&linked.main_root, &id);
            let mut m = JMap::new();
            m.insert("location".into(), json!("worktree"));
            m.insert("id".into(), json!(id.clone()));
            m.insert(
                "feature".into(),
                match status.get("feature") {
                    None | Some(Value::Null) => Value::Null,
                    Some(v) => v.clone(),
                },
            );
            m.insert("branch".into(), branch.map(Value::String).unwrap_or(Value::Null));
            m.insert(
                "merge_command".into(),
                json!(format!("bee worktree merge --id {id}")),
            );
            return Ok(Some(m));
        }
        // `resolution.worktreeResolution !== 'ordinary'` — an UNGRANTED linked
        // worktree stops here (it is neither "go to the worktree" nor "you
        // are in one that owns the feature").
        if ctx.linked.is_some() {
            return Ok(None);
        }
        let feature = match status.get("feature") {
            None | Some(Value::Null) => Value::Null,
            Some(v) => v.clone(),
        };
        let lane = match status.get("route") {
            Some(route) if truthy(route) => vget(route, "lane").cloned(),
            _ => Some(Value::Null),
        };
        let lane_ref = lane.as_ref();
        if !truthy(&feature) {
            return Ok(None);
        }
        let other_live = if str_eq(lane_ref, "tiny") {
            other_live_work_present(ctx)?
        } else {
            true
        };
        if !is_code_touching_lane(lane_ref, other_live) {
            return Ok(None);
        }
        let feature_str = match &feature {
            Value::String(s) => s.clone(),
            other => jsjson::js_to_string(other),
        };
        let root = ctx.root.clone();
        let Some((id, worktree_root)) = find_granted_worktree_for_feature(&root, &feature_str)
        else {
            return Ok(None);
        };
        let mut m = JMap::new();
        m.insert("location".into(), json!("main"));
        m.insert("id".into(), json!(id));
        m.insert("feature".into(), feature);
        m.insert("path".into(), json!(worktree_root.clone()));
        m.insert("guidance".into(), json!(format!("open your session at {worktree_root}")));
        Ok(Some(m))
    };
    match attempt(ctx) {
        Ok(v) => Ok(v),
        Err(Ex::Thrown) => Ok(None),
        Err(e) => Err(e),
    }
}

/// bee.mjs buildOrient.
fn build_orient(ctx: &mut Ctx) -> R<JMap> {
    let status = build_status(ctx, false)?;
    let feature = match status.get("feature") {
        None | Some(Value::Null) => Value::Null,
        Some(v) => v.clone(),
    };
    let context_md: Value = if truthy(&feature) {
        // path.join(root, 'docs', 'history', feature) — a non-string feature
        // would throw in Node's path.join -> bail (Node re-run reproduces).
        let Value::String(feature_str) = &feature else {
            return Err(Ex::Bail);
        };
        if ctx
            .root
            .join("docs")
            .join("history")
            .join(feature_str)
            .join("CONTEXT.md")
            .exists()
        {
            json!(format!("docs/history/{feature_str}/CONTEXT.md"))
        } else {
            Value::Null
        }
    } else {
        Value::Null
    };
    let feature_arg = if truthy(&feature) { Some(feature.clone()) } else { None };
    let ready_ids: Vec<Value> = ready_cells(ctx, feature_arg.as_ref())?
        .iter()
        .take(5)
        .map(|c| vget(c, "id").cloned().unwrap_or(Value::Null))
        .collect();
    let mut blockers: Vec<Value> = Vec::new();
    if opt_truthy(status.get("handoff")) {
        blockers.push(json!("pending handoff — surface it to the user and wait"));
    }
    let sd = status.get("scribing_debt");
    if opt_truthy(sd) && vget(sd.unwrap(), "count").and_then(|v| v.as_f64()).unwrap_or(0.0) > 0.0 {
        blockers.push(json!(format!(
            "scribing debt: {} behavior_change cell(s) uncaptured",
            tpl(vget(sd.unwrap(), "count"))
        )));
    }
    if let Some(Value::Array(warnings)) = status.get("staleness_warnings") {
        for warning in warnings {
            if let Value::String(w) = warning {
                if w.contains("reservation(s) expired") {
                    blockers.push(warning.clone());
                }
            }
        }
    }
    let worktree = orient_worktree_context(ctx, &status)?;

    let mut packet = JMap::new();
    {
        let mut where_ = JMap::new();
        where_.insert("phase".into(), status.get("phase").cloned().unwrap_or(Value::Null));
        where_.insert("feature".into(), feature.clone());
        where_.insert(
            "mode".into(),
            match status.get("mode") {
                None | Some(Value::Null) => Value::Null,
                Some(v) => v.clone(),
            },
        );
        where_.insert("gates".into(), status.get("gates").cloned().unwrap_or(Value::Null));
        where_.insert(
            "gate_bypass_level".into(),
            status.get("gate_bypass_level").cloned().unwrap_or(Value::Null),
        );
        packet.insert("where".into(), Value::Object(where_));
    }
    {
        let mut decisions = JMap::new();
        decisions.insert("context_md".into(), context_md);
        decisions.insert("active_count".into(), json!(active_decisions(ctx, None).len()));
        let recent: Vec<Value> = match status.get("recent_decisions") {
            Some(Value::Array(rows)) => rows
                .iter()
                .map(|d| json!(orient_decision_line(vget(d, "decision"))))
                .collect(),
            _ => Vec::new(),
        };
        decisions.insert("recent".into(), Value::Array(recent));
        packet.insert("decisions".into(), Value::Object(decisions));
    }
    {
        let cells = status.get("cells").cloned().unwrap_or(Value::Null);
        let mut work = JMap::new();
        let mut counts = JMap::new();
        counts.insert("open".into(), vget(&cells, "open").cloned().unwrap_or(Value::Null));
        counts.insert("claimed".into(), vget(&cells, "claimed").cloned().unwrap_or(Value::Null));
        counts.insert("capped".into(), vget(&cells, "capped").cloned().unwrap_or(Value::Null));
        work.insert("cells".into(), Value::Object(counts));
        work.insert("ready".into(), Value::Array(ready_ids.clone()));
        work.insert("blockers".into(), Value::Array(blockers));
        packet.insert("work".into(), Value::Object(work));
    }
    if let Some(worktree) = &worktree {
        packet.insert("worktree".into(), Value::Object(worktree.clone()));
    }
    {
        let mut next = JMap::new();
        next.insert(
            "action".into(),
            status.get("recommended_next").cloned().unwrap_or(Value::Null),
        );
        let skill = status
            .get("phase")
            .and_then(|v| v.as_str())
            .and_then(|p| {
                ORIENT_PHASE_SKILL
                    .iter()
                    .find(|(phase, _)| *phase == p)
                    .map(|(_, s)| *s)
            })
            .unwrap_or("bee-hive");
        next.insert("skill".into(), json!(skill));
        let command = match &worktree {
            Some(w) if str_eq(w.get("location"), "main") => {
                w.get("guidance").cloned().unwrap_or(Value::Null)
            }
            _ => orient_next_command(&status, &ready_ids),
        };
        next.insert("command".into(), command);
        packet.insert("next".into(), Value::Object(next));
    }
    Ok(packet)
}

/// bee.mjs renderOrientText — at most six lines plus the conditional
/// blockers/worktree lines.
fn render_orient_text(packet: &JMap) -> String {
    let where_ = packet.get("where").cloned().unwrap_or(Value::Null);
    let gates = GATE_NAMES
        .iter()
        .map(|g| {
            if opt_truthy(vget(&where_, "gates").and_then(|gs| vget(gs, g))) {
                "true"
            } else {
                "false"
            }
        })
        .collect::<Vec<_>>()
        .join("/");
    let worktree = packet.get("worktree");
    let worktree_line: Option<String> = worktree.map(|w| {
        if str_eq(vget(w, "location"), "main") {
            format!(
                "worktree: feature \"{}\" lives in worktree {} — {}",
                tpl(vget(w, "feature")),
                tpl(vget(w, "id")),
                tpl(vget(w, "guidance"))
            )
        } else {
            let branch = if opt_truthy(vget(w, "branch")) {
                format!(" (branch {})", tpl(vget(w, "branch")))
            } else {
                String::new()
            };
            format!(
                "worktree: {}{branch} — merge back from main with {}",
                tpl(vget(w, "id")),
                tpl(vget(w, "merge_command"))
            )
        }
    });
    let decisions = packet.get("decisions").cloned().unwrap_or(Value::Null);
    let work = packet.get("work").cloned().unwrap_or(Value::Null);
    let next = packet.get("next").cloned().unwrap_or(Value::Null);
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "where: phase={} feature={} mode={} gates={gates} bypass={}",
        tpl(vget(&where_, "phase")),
        if nullish(vget(&where_, "feature")) { "none".to_string() } else { tpl(vget(&where_, "feature")) },
        if nullish(vget(&where_, "mode")) { "none".to_string() } else { tpl(vget(&where_, "mode")) },
        tpl(vget(&where_, "gate_bypass_level"))
    ));
    let context_part = if opt_truthy(vget(&decisions, "context_md")) {
        format!(" | context: {}", tpl(vget(&decisions, "context_md")))
    } else {
        String::new()
    };
    lines.push(format!(
        "decisions: {} active{context_part}",
        tpl(vget(&decisions, "active_count"))
    ));
    let cells = vget(&work, "cells").cloned().unwrap_or(Value::Null);
    let ready_part = match vget(&work, "ready") {
        Some(Value::Array(ready)) if !ready.is_empty() => {
            format!(" | ready: {}", js_join(ready, ", "))
        }
        _ => String::new(),
    };
    lines.push(format!(
        "work: open={} claimed={} capped={}{ready_part}",
        tpl(vget(&cells, "open")),
        tpl(vget(&cells, "claimed")),
        tpl(vget(&cells, "capped"))
    ));
    if let Some(Value::Array(blockers)) = vget(&work, "blockers") {
        if !blockers.is_empty() {
            lines.push(format!("blockers: {}", js_join(blockers, "; ")));
        }
    }
    if let Some(line) = worktree_line {
        lines.push(line);
    }
    lines.push(format!("skill: {}", tpl(vget(&next, "skill"))));
    lines.push(format!("next: {}", tpl(vget(&next, "action"))));
    lines.join("\n")
}

// ─── entry point ───────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum Verb {
    Status,
    Orient,
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    let strs: Vec<&str> = args.iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    // STRANGLER ROUTING RULE: exactly these six shapes; any --brief presence
    // was already served upstream by status_brief; everything else -> None.
    let (verb, lanes_full, use_json) = match strs.as_slice() {
        ["status"] => (Verb::Status, false, false),
        ["status", "--json"] => (Verb::Status, false, true),
        ["status", "--lanes-full"] => (Verb::Status, true, false),
        ["status", "--lanes-full", "--json"] => (Verb::Status, true, true),
        ["orient"] => (Verb::Orient, false, false),
        ["orient", "--json"] => (Verb::Orient, false, true),
        _ => return None,
    };
    run(verb, lanes_full, use_json, t0)
}

fn run(verb: Verb, lanes_full: bool, use_json: bool, t0: Instant) -> Option<ExitCode> {
    let cwd = std::env::current_dir().ok()?;
    let cmd = match verb {
        Verb::Status => "status",
        Verb::Orient => "orient",
    };
    // WORKTREE-NATIVE (see roots.rs's header): status/orient serve linked
    // worktrees themselves. A BROKEN link still delegates.
    let roots = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(r) => r,
        RootsWt::NeedsNode => return None,
        RootsWt::None => return Some(emit_no_root_error(&cwd, cmd, use_json, t0)),
    };
    let root = roots.root;
    // Drift check first (its cache write is the one permitted pre-bail side
    // effect — Node performs it before routing too).
    let drift = check_manifest_drift(&root).ok()?;
    let mut ctx = Ctx { root, cwd, linked: roots.linked, stderr: Vec::new() };
    let (payload, text) = match verb {
        Verb::Status => {
            let status = build_status(&mut ctx, lanes_full).ok()?;
            let text = render_status_text(&status);
            (Value::Object(status), text)
        }
        Verb::Orient => {
            let packet = build_orient(&mut ctx).ok()?;
            let text = render_orient_text(&packet);
            (Value::Object(packet), text)
        }
    };
    // Emission order (per stream): handler warnings, then the drift line on
    // stderr; the payload on stdout; the timing line last.
    for line in &ctx.stderr {
        eprintln!("{line}");
    }
    if drift.manifest_changed {
        eprintln!("manifest_changed: true — {}", drift.hint);
    }
    if use_json {
        println!("{}", jsjson::stringify_pretty(&payload));
    } else {
        println!("{text}");
    }
    record_timing(&ctx.root, cmd, t0, true);
    Some(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An ORDINARY-checkout context (`linked: None`) — the shape every
    /// pre-existing fixture below has always had.
    fn ctx_for(root: &Path) -> Ctx {
        Ctx {
            root: root.to_path_buf(),
            cwd: root.to_path_buf(),
            linked: None,
            stderr: Vec::new(),
        }
    }

    fn write(root: &Path, rel: &str, content: &str) {
        let file = root.join(rel);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }

    fn sha256_str(s: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(s.as_bytes());
        format!("{:x}", h.finalize())
    }

    #[test]
    fn runtime_drift_detects_content_missing_and_extra() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/bin/lib/a.mjs", "aaa");
        write(root, ".bee/bin/lib/extra.mjs", "zzz");
        write(root, ".bee/bin/helper.mjs", "hhh");
        let onboarding = json!({
            "bee_version": BEE_VERSION,
            "managed": {
                "lib": { "a.mjs": sha256_str("aaa"), "gone.mjs": "deadbeef" },
                "helpers": { "helper.mjs": "not-the-hash" }
            }
        });
        let ctx = ctx_for(root);
        let (drift, detail) = compute_runtime_drift(&ctx, &onboarding);
        assert!(drift);
        assert_eq!(
            detail,
            vec![
                ".bee/bin/lib/gone.mjs (missing)".to_string(),
                ".bee/bin/helper.mjs".to_string(),
                ".bee/bin/lib/extra.mjs (extra)".to_string(),
            ]
        );
        // Clean ledger: no drift.
        let clean = json!({
            "bee_version": BEE_VERSION,
            "managed": {
                "lib": { "a.mjs": sha256_str("aaa"), "extra.mjs": sha256_str("zzz") },
                "helpers": { "helper.mjs": sha256_str("hhh") }
            }
        });
        let (drift, detail) = compute_runtime_drift(&ctx, &clean);
        assert!(!drift);
        assert!(detail.is_empty());
        // Version-only drift with a legacy (no managed map) ledger.
        let legacy = json!({ "bee_version": "0.0.1" });
        let (drift, detail) = compute_runtime_drift(&ctx, &legacy);
        assert!(drift);
        assert!(detail.is_empty());
    }

    #[test]
    fn workers_derive_from_heartbeat_joined_claims() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let fresh = to_iso(now_ms());
        let stale = "2020-01-01T00:00:00.000Z";
        write(
            root,
            ".bee/sessions/live-1.json",
            &format!(r#"{{"id":"live-1","started_at":"{fresh}","last_heartbeat":"{fresh}","lane":"feat-x"}}"#),
        );
        write(
            root,
            ".bee/sessions/dead-1.json",
            &format!(r#"{{"id":"dead-1","started_at":"{stale}","last_heartbeat":"{stale}"}}"#),
        );
        write(
            root,
            ".bee/claims/cell-7.json",
            &format!(r#"{{"cell":"cell-7","session":"live-1","claimed_at":"{fresh}","ttl_seconds":3600}}"#),
        );
        // An expired claim for the same session must not win.
        write(
            root,
            ".bee/claims/cell-1.json",
            r#"{"cell":"cell-1","session":"live-1","claimed_at":"2020-01-01T00:00:00.000Z","ttl_seconds":1}"#,
        );
        let rows = active_workers(root, None).ok().unwrap();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.get("session_id"), Some(&json!("live-1")));
        assert_eq!(row.get("lane"), Some(&json!("feat-x")));
        assert_eq!(row.get("cell"), Some(&json!("cell-7")));
        // Excluding the live session leaves zero rows.
        assert!(active_workers(root, Some("live-1")).ok().unwrap().is_empty());
    }

    #[test]
    fn lanes_summary_vs_full() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            ".bee/lanes/alpha.json",
            r#"{"feature":"alpha","phase":"swarming","approved_gates":{"context":true}}"#,
        );
        write(
            root,
            ".bee/lanes/beta.json",
            r#"{"feature":"beta","phase":"idle"}"#,
        );
        let mut ctx = ctx_for(root);
        let rows = build_lane_rows(&mut ctx).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("feature"), Some(&json!("alpha")));
        assert_eq!(rows[0].get("bound_sessions"), Some(&json!([])));
        // Full row render.
        let row_text = format_lane_row(&Value::Object(rows[0].clone()));
        assert_eq!(
            row_text,
            "alpha [swarming] context=approved shape=pending execution=pending review=pending"
        );
        // Summary: no live session -> active null, counts + ids over all.
        let summary = build_lane_summary(&mut ctx).unwrap();
        assert_eq!(summary.get("active"), Some(&Value::Null));
        assert_eq!(summary.get("counts"), Some(&json!({"swarming": 1, "idle": 1})));
        assert_eq!(summary.get("ids"), Some(&json!(["alpha", "beta"])));
        let line = format_lane_summary_line(&Value::Object(summary)).unwrap();
        assert_eq!(line, "Lanes: 2 other lane(s) [swarming=1 idle=1] (ids: alpha, beta)");
    }

    #[test]
    fn staleness_warnings_fire_in_node_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // No commands recorded; stale advisor key; version mismatch; stale
        // handoff; unknown phase.
        write(root, ".bee/onboarding.json", r#"{"bee_version":"0.9.0"}"#);
        write(root, ".bee/config.json", r#"{"advisor":"x"}"#);
        write(
            root,
            ".bee/HANDOFF.json",
            r#"{"written_at":"2020-01-01T00:00:00.000Z","kind":"pause"}"#,
        );
        write(root, ".bee/state.json", r#"{"phase":"vibing"}"#);
        let mut ctx = ctx_for(root);
        let status = build_status(&mut ctx, false).unwrap();
        let warnings: Vec<String> = status
            .get("staleness_warnings")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .map(|w| w.as_str().unwrap().to_string())
            .collect();
        assert!(warnings[0].starts_with("No standard commands recorded"));
        assert_eq!(
            warnings[1],
            format!("Onboarding installed bee 0.9.0 but plugin is {BEE_VERSION} — re-run onboarding.")
        );
        assert_eq!(
            warnings[2],
            "HANDOFF.json is older than 7 days (written 2020-01-01T00:00:00.000Z)."
        );
        assert_eq!(warnings[3], STALE_ADVISOR_KEY_WARNING);
        assert!(warnings[4].starts_with("Unknown phase \"vibing\""));
        assert_eq!(warnings.len(), 5);
        // HANDOFF present wins the recommendation.
        assert_eq!(
            status.get("recommended_next"),
            Some(&json!("HANDOFF present — present it to the user and WAIT. Never auto-resume."))
        );
    }

    #[test]
    fn orient_recommended_next_selection_and_packet() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(root, ".bee/config.json", r#"{"commands":{"test":"npm t"}}"#);
        write(
            root,
            ".bee/state.json",
            r#"{"phase":"swarming","feature":"f1","mode":"standard","approved_gates":{"context":true,"shape":true,"execution":true}}"#,
        );
        write(
            root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f1","status":"open","lane":"standard","title":"t"}"#,
        );
        std::fs::create_dir_all(root.join("docs").join("history").join("f1")).unwrap();
        write(root, "docs/history/f1/CONTEXT.md", "# ctx");
        let mut ctx = ctx_for(root);
        let packet = build_orient(&mut ctx).unwrap();
        // exec approved + one ready cell -> ready recommendation + command.
        let next = packet.get("next").unwrap();
        assert_eq!(
            vget(next, "action"),
            Some(&json!("1 ready cell(s): c-1 — orchestrator assigns them."))
        );
        assert_eq!(vget(next, "skill"), Some(&json!("bee-swarming")));
        assert_eq!(vget(next, "command"), Some(&json!("bee cells ready --json")));
        let decisions = packet.get("decisions").unwrap();
        assert_eq!(vget(decisions, "context_md"), Some(&json!("docs/history/f1/CONTEXT.md")));
        assert_eq!(vget(decisions, "active_count"), Some(&json!(0)));
        let work = packet.get("work").unwrap();
        assert_eq!(vget(work, "ready"), Some(&json!(["c-1"])));
        assert_eq!(vget(work, "blockers"), Some(&json!([])));
        // Text renderer.
        let text = render_orient_text(&packet);
        let lines: Vec<&str> = text.split('\n').collect();
        assert_eq!(
            lines[0],
            "where: phase=swarming feature=f1 mode=standard gates=true/true/true/false bypass=off"
        );
        assert_eq!(lines[1], "decisions: 0 active | context: docs/history/f1/CONTEXT.md");
        assert_eq!(lines[2], "work: open=1 claimed=0 capped=0 | ready: c-1");
        assert_eq!(lines[3], "skill: bee-swarming");
        assert_eq!(lines[4], "next: 1 ready cell(s): c-1 — orchestrator assigns them.");
    }

    #[test]
    fn status_text_renderer_minimal_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(root, ".bee/config.json", r#"{"commands":{"test":"npm t"},"gate_bypass":true}"#);
        write(root, ".bee/state.json", r#"{"phase":"idle"}"#);
        let mut ctx = ctx_for(root);
        let status = build_status(&mut ctx, false).unwrap();
        let text = render_status_text(&status);
        let lines: Vec<&str> = text.split('\n').collect();
        assert_eq!(lines[0], format!("bee status (plugin v{BEE_VERSION})"));
        assert_eq!(lines[1], format!("Onboarding: installed (bee {BEE_VERSION})"));
        assert_eq!(lines[2], "Phase: idle | Mode: none | Feature: none");
        assert_eq!(
            lines[3],
            "Gates: context=pending shape=pending execution=pending review=pending"
        );
        assert_eq!(lines[4], bypass_banner("normal"));
        assert_eq!(lines[5], "Handoff: none");
        assert_eq!(
            lines[6],
            "Cells: open=0 claimed=0 capped=0 blocked=0 archived=0 (total capped=0)"
        );
        assert_eq!(lines[7], "Standard commands: test=npm t");
        assert_eq!(lines[8], "Active reservations: 0");
        assert_eq!(lines[9], "Active workers: 0");
        assert_eq!(lines[10], "Critical patterns file: absent");
        assert!(lines[11].starts_with("Models (claude): generation=sonnet extraction=haiku review=opus"));
        // Idle repo with no next_action override -> defaultState's line.
        assert_eq!(
            *lines.last().unwrap(),
            "Recommended next: No active bee work — awaiting a user request."
        );
        // JSON shape spot-checks.
        assert_eq!(status.get("gate_bypass"), Some(&json!(true)));
        assert_eq!(status.get("gate_bypass_level"), Some(&json!("normal")));
        assert_eq!(status.get("ship_visibility"), Some(&json!("off")));
        assert_eq!(status.get("pbi"), Some(&Value::Null));
    }

    #[test]
    fn locale_compare_matches_measured_node_behavior() {
        // Measured with Node localeCompare('en', {numeric:true}) / ('en').
        let cases_numeric = [
            ("1710-2", "1710-10", Ordering::Less),
            ("01", "1", Ordering::Equal),
            ("a-b", "ab", Ordering::Less),
            ("es-1", "ES-1", Ordering::Less),
            ("_", "-", Ordering::Less),
            ("-", ".", Ordering::Less),
            (".", "0", Ordering::Less),
            ("0", "a", Ordering::Less),
            ("a", "A", Ordering::Less),
            ("A", "b", Ordering::Less),
        ];
        for (a, b, expected) in cases_numeric {
            assert_eq!(locale_cmp(a, b, true), expected, "numeric {a} vs {b}");
        }
        assert_eq!(locale_cmp("1710-2", "1710-10", false), Ordering::Greater);
    }

    #[test]
    fn js_date_parse_iso_shapes() {
        assert_eq!(js_date_parse("2026-07-29T08:17:26.986Z"), 1785313046986.0);
        assert_eq!(js_date_parse("2026-07-29"), 1785283200000.0);
        assert!(js_date_parse("garbage").is_nan());
        assert!(js_date_parse("2026-02-31").is_nan());
        assert_eq!(to_iso(1785313046986.0), "2026-07-29T08:17:26.986Z");
    }

    #[test]
    fn datamark_neutralizes_text() {
        assert_eq!(datamark(Some(&json!("plain text"))), "«plain text»");
        assert_eq!(datamark(Some(&json!("a ``` b"))), "«a  b»");
        assert_eq!(datamark(Some(&json!("keep `` two"))), "«keep `` two»");
        assert_eq!(datamark(Some(&json!("x <system foo> y </user>"))), "«x  y»");
        assert_eq!(datamark(Some(&json!("no <systemic> tag"))), "«no <systemic> tag»");
        assert_eq!(datamark(None), "«»");
        assert_eq!(datamark(Some(&json!("  padded \u{0007} "))), "«padded»");
    }

    #[test]
    fn orient_decision_line_caps_at_160_utf16_units() {
        let long = "x".repeat(200);
        let capped = orient_decision_line(Some(&json!(long)));
        assert_eq!(capped.chars().count(), 160); // 157 + '...'
        assert!(capped.ends_with("..."));
        let short = orient_decision_line(Some(&json!("first line\nsecond")));
        assert_eq!(short, "first line");
    }

    #[test]
    fn lease_rows_render_as_reservations() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            ".bee/runtime/leases/paths/abc.json",
            r#"{"resource":"path:src/a.rs","mode":"write","workflow_id":"c-1","session_id":"s-1","workspace_id":"agent:worker-1","epoch":1,"acquired_at":"2026-07-29T00:00:00.000Z","expires_at":"2026-07-29T01:00:00.000Z","kind":"lease"}"#,
        );
        let ctx = ctx_for(root);
        let rows = list_reservations(&ctx, false);
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.get("agent"), Some(&json!("worker-1")));
        assert_eq!(row.get("cell"), Some(&json!("c-1")));
        assert_eq!(row.get("path"), Some(&json!("src/a.rs")));
        assert_eq!(row.get("ttl_seconds"), Some(&json!(3600)));
        assert_eq!(row.get("released_at"), Some(&Value::Null));
        assert_eq!(row.get("session"), Some(&json!("s-1")));
        // Expired by now -> filtered out of activeOnly, still listed raw.
        assert!(list_reservations(&ctx, true).is_empty());
    }

    // ── linked worktrees, over REAL `git worktree add` fixtures ────────────
    //
    // Every expectation below was pinned against Node on the SAME fixture
    // shape before it was written here (twin-fixture byte-diff of
    // `status --json` / `orient --json` from inside each checkout, with
    // BEE_JS_ENTRY sabotaged so bee.exe could not have delegated).

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

    /// A real main checkout with two real linked worktrees: `wt-granted`
    /// (registered in MAIN's grant registry, so it owns its own store) and
    /// `wt-ungranted` (unregistered, so it shares main's).
    fn worktree_fixture(tmp: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let main = tmp.join("main");
        std::fs::create_dir_all(&main).unwrap();
        write(&main, ".bee/onboarding.json", "{}");
        write(&main, "f.txt", "x");
        git(&main, &["init", "-q", "-b", "main", "."]);
        git(&main, &["config", "user.email", "a@b.c"]);
        git(&main, &["config", "user.name", "t"]);
        git(&main, &["add", "-A"]);
        git(&main, &["commit", "-qm", "init"]);
        let granted = tmp.join("wt-granted");
        let ungranted = tmp.join("wt-ungranted");
        git(&main, &["worktree", "add", "-q", granted.to_str().unwrap(), "-b", "wt/granted"]);
        git(&main, &["worktree", "add", "-q", ungranted.to_str().unwrap(), "-b", "wt/ungranted"]);
        write(&main, ".bee/runtime/worktree-grants.json", "{\"wt-granted\": true}\n");
        write(&granted, ".bee/onboarding.json", "{}");
        (main, granted, ungranted)
    }

    /// Build the Ctx `run()` would build standing in `cwd`.
    fn ctx_at(cwd: &Path) -> Ctx {
        match resolve_store_root_worktree(cwd) {
            RootsWt::Go(r) => Ctx {
                root: r.root,
                cwd: cwd.to_path_buf(),
                linked: r.linked,
                stderr: Vec::new(),
            },
            _ => panic!("expected a resolvable root at {}", cwd.display()),
        }
    }

    /// bee.mjs ungrantedWorktreeNotice: present ONLY inside an ungranted
    /// linked worktree. The main checkout and a granted worktree both omit
    /// the key entirely (GH #30) — this is the exact status shape whose loss
    /// blocked the routing flip.
    #[test]
    fn worktree_notice_fires_only_inside_an_ungranted_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, ungranted) = worktree_fixture(tmp.path());

        assert_eq!(ungranted_worktree_notice(&ctx_for(&main)), None);
        assert_eq!(ungranted_worktree_notice(&ctx_at(&main)), None);
        assert_eq!(ungranted_worktree_notice(&ctx_at(&granted)), None);
        let notice = ungranted_worktree_notice(&ctx_at(&ungranted)).expect("notice");
        assert_eq!(notice, UNGRANTED_WORKTREE_NOTICE);
        assert!(notice.starts_with("⚠ This linked worktree is UNGRANTED"));
        assert!(notice.ends_with("from inside it."));

        // And it lands in the payload under the right key, only there.
        let mut c = ctx_at(&ungranted);
        let status = build_status(&mut c, false).expect("status");
        assert_eq!(status.get("worktree_notice"), Some(&json!(notice)));
        let mut c = ctx_at(&granted);
        assert!(!build_status(&mut c, false).unwrap().contains_key("worktree_notice"));
        let mut c = ctx_at(&main);
        assert!(!build_status(&mut c, false).unwrap().contains_key("worktree_notice"));
    }

    /// state.mjs controlRootFor: sessions/claims/workers are CONTROL plane —
    /// from inside a granted worktree they must resolve onto MAIN's store,
    /// never the worktree's own. (An ungranted worktree's `root` already IS
    /// main, so it agrees trivially.)
    #[test]
    fn control_root_re_roots_onto_main_from_a_granted_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, ungranted) = worktree_fixture(tmp.path());
        let n = |p: &Path| normalize_abs_lexical(&p.to_string_lossy());

        assert_eq!(n(&control_root_for(&mut ctx_at(&main)).unwrap()), n(&main));
        assert_eq!(n(&control_root_for(&mut ctx_at(&granted)).unwrap()), n(&main));
        assert_eq!(n(&control_root_for(&mut ctx_at(&ungranted)).unwrap()), n(&main));
        // The store root itself is NOT re-rooted: it is the worktree's own
        // when granted, main's when not.
        assert_eq!(n(&ctx_at(&granted).root), n(&granted));
        assert_eq!(n(&ctx_at(&ungranted).root), n(&main));

        // A live session written into MAIN's store only is visible from the
        // granted worktree's status through that control root.
        let now = to_iso(now_ms());
        write(
            &main,
            ".bee/sessions/sess-live.json",
            &format!("{{\"id\":\"sess-live\",\"started_at\":\"{now}\",\"last_heartbeat\":\"{now}\"}}"),
        );
        let ctrl = control_root_for(&mut ctx_at(&granted)).unwrap();
        let workers = active_workers(&ctrl, None).unwrap();
        assert_eq!(workers.len(), 1);
        assert_eq!(workers[0].get("session_id"), Some(&json!("sess-live")));
        // Reading the worktree's own root instead would find nothing — this
        // is exactly the bug the `controlRoot == root` assumption caused.
        assert!(active_workers(&granted, None).unwrap().is_empty());
    }

    /// reservations.mjs's own cycle-safe control-root walk (LEASE files) also
    /// answers mainRoot inside a granted worktree — from the git link alone,
    /// with no grant registry involved.
    #[test]
    fn reservations_control_root_follows_the_git_link() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, ungranted) = worktree_fixture(tmp.path());
        let n = |p: &Path| normalize_abs_lexical(&p.to_string_lossy());
        assert_eq!(n(&reservations_control_root(&ctx_at(&main))), n(&main));
        assert_eq!(n(&reservations_control_root(&ctx_at(&granted))), n(&main));
        assert_eq!(n(&reservations_control_root(&ctx_at(&ungranted))), n(&main));
        // findMainRoot fails OPEN: a link it cannot validate answers `root`.
        let orphan = tmp.path().join("orphan");
        std::fs::create_dir_all(&orphan).unwrap();
        std::fs::write(orphan.join(".git"), "gitdir: nowhere").unwrap();
        let ctx = Ctx { root: orphan.clone(), cwd: orphan.clone(), linked: None, stderr: Vec::new() };
        assert_eq!(n(&reservations_control_root(&ctx)), n(&orphan));
    }

    /// bee.mjs readWorktreeBranch over a real `git worktree add` HEAD.
    #[test]
    fn worktree_branch_reads_the_linked_head() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, _granted, _ungranted) = worktree_fixture(tmp.path());
        assert_eq!(read_worktree_branch(&main, "wt-granted").as_deref(), Some("wt/granted"));
        assert_eq!(read_worktree_branch(&main, "no-such-id"), None);
        // Detached HEAD (a bare sha) is null, not the sha.
        std::fs::write(
            main.join(".git").join("worktrees").join("wt-granted").join("HEAD"),
            "0123456789abcdef0123456789abcdef01234567\n",
        )
        .unwrap();
        assert_eq!(read_worktree_branch(&main, "wt-granted"), None);
    }

    /// bee.mjs orientWorktreeContext, both halves, over the real fixture.
    #[test]
    fn orient_worktree_context_serves_both_halves() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, ungranted) = worktree_fixture(tmp.path());

        // Inside the GRANTED worktree: the merge-back packet. `feature` is
        // whatever status resolved, `branch` comes from the linked HEAD.
        let mut status = JMap::new();
        status.insert("feature".into(), json!("demo"));
        let block = orient_worktree_context(&mut ctx_at(&granted), &status)
            .unwrap()
            .expect("worktree block inside a granted worktree");
        assert_eq!(block.get("location"), Some(&json!("worktree")));
        assert_eq!(block.get("id"), Some(&json!("wt-granted")));
        assert_eq!(block.get("feature"), Some(&json!("demo")));
        assert_eq!(block.get("branch"), Some(&json!("wt/granted")));
        assert_eq!(
            block.get("merge_command"),
            Some(&json!("bee worktree merge --id wt-granted"))
        );
        // The text render takes the non-'main' branch.
        let mut packet = JMap::new();
        packet.insert("worktree".into(), Value::Object(block.clone()));
        packet.insert("where".into(), json!({"phase":"idle","feature":"demo","mode":null,"gates":{},"gate_bypass_level":"off"}));
        packet.insert("decisions".into(), json!({"context_md":null,"active_count":0,"recent":[]}));
        packet.insert("work".into(), json!({"cells":{"open":0,"claimed":0,"capped":0},"ready":[],"blockers":[]}));
        packet.insert("next".into(), json!({"action":"a","skill":"bee-hive","command":null}));
        assert!(render_orient_text(&packet).contains(
            "worktree: wt-granted (branch wt/granted) — merge back from main with bee worktree merge --id wt-granted"
        ));

        // Inside the UNGRANTED worktree: no block at all.
        assert!(orient_worktree_context(&mut ctx_at(&ungranted), &status)
            .unwrap()
            .is_none());

        // From MAIN with a code-touching lane whose feature lives in the
        // granted worktree: the "go there" block.
        write(&granted, ".bee/runtime/worktree-identity.json", "{\"feature\":\"demo\"}");
        let mut status_main = JMap::new();
        status_main.insert("feature".into(), json!("demo"));
        status_main.insert("route".into(), json!({"lane": "small"}));
        let block = orient_worktree_context(&mut ctx_at(&main), &status_main)
            .unwrap()
            .expect("main-side worktree block");
        assert_eq!(block.get("location"), Some(&json!("main")));
        assert_eq!(block.get("id"), Some(&json!("wt-granted")));
        assert!(tpl(block.get("guidance")).starts_with("open your session at "));
        // A docs lane is exempt -> no block, byte-unchanged orient.
        status_main.insert("route".into(), json!({"lane": "docs"}));
        assert!(orient_worktree_context(&mut ctx_at(&main), &status_main)
            .unwrap()
            .is_none());
    }

    /// The whole orient packet from inside a granted worktree carries the
    /// `worktree` key — the exact block whose loss was the measured C2 break
    /// that kept this routing flip parked.
    #[test]
    fn orient_packet_carries_the_worktree_block_inside_a_granted_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let (_main, granted, ungranted) = worktree_fixture(tmp.path());
        let packet = build_orient(&mut ctx_at(&granted)).expect("orient");
        let block = packet.get("worktree").expect("worktree block");
        assert_eq!(vget(block, "location"), Some(&json!("worktree")));
        assert_eq!(vget(block, "id"), Some(&json!("wt-granted")));
        // next.command stays orient's own (only the 'main' location overrides).
        assert!(!packet.contains_key("worktree_notice"));
        // The ungranted worktree's orient has no block.
        assert!(!build_orient(&mut ctx_at(&ungranted))
            .expect("orient")
            .contains_key("worktree"));
    }
}
