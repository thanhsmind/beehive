// session_preamble — native port of packages/bee/lib/inject.mjs's
// `buildSessionPreamble` (plus the three shared renderers it exports:
// onboardingLine, bypassBannerLines, handoffBlockLines, firstOpenGate).
//
// CUTOVER MODULE. Until the Node runtime was deleted this whole surface was
// `Outcome::Delegate` in hooks/session_init.rs — the preamble pulls
// essentially the entire vendored lib closure, and contract C2 demanded the
// bytes match Node's. C2 retired with Node, and a delegation with nowhere to
// delegate is a crash, so the renderer is native here.
//
// The one deliberate wording divergence, taken FOR the cutover: the closing
// line named the Node entry point (`node .bee/bin/bee.mjs status --json`).
// It now names the binary (`.bee/bin/bee status --json`), which is the only
// spelling that still exists. Same for the knowledge-context command
// (inject.mjs:260). The emitted preamble carries NO `.mjs` spelling anywhere;
// `no_mjs_spelling_survives_anywhere_in_the_preamble` pins that.
//
// ─── Ported inject.mjs surface ─────────────────────────────────────────────
//
//   buildSessionPreamble           -> build_session_preamble  (pub)
//   onboardingLine                 -> onboarding_line         (pub)
//   bypassBannerLines              -> bypass_banner_lines     (pub)
//   handoffBlockLines              -> handoff_block_lines     (pub)
//   firstOpenGate / visibleGates   -> first_open_gate / visible_gates (pub/priv)
//   gatesLine                      -> gates_line
//   knowledgeContextLines          -> knowledge_context_lines
//   knowledgeContextBudgetForMode  -> knowledge_context_budget_for_mode
//   projectMapLines / specProjectMapLines / bundleProjectMapLines
//   criticalPatternsDigest / legacy… / bundle…
//
// ─── Re-derived Rust (the "may not edit that file" rule) ───────────────────
//
// Every lib helper the preamble consumes is ALREADY ported in this tree, but
// each lives behind a private fn inside a module this cell may not touch
// (verbs/status_full.rs, verbs/knowledge.rs, verbs/decisions.rs,
// hooks/chain_nudge.rs, ...). Following the precedent verbs/drivers.rs set
// with its `kctx` lift, the helpers below are lifts of those ports — not
// second implementations — each carrying a provenance line naming BOTH the
// .mjs source and the Rust port it was lifted from. Only two things are
// reused directly, because they are already `pub` on a module this cell may
// edit: `crate::state::{bypass_level, ship_visibility}` and
// `crate::fsutil::{read_json, warn_corrupt_json}`.
//
// ─── Fail-open discipline (inject.mjs's whole posture) ─────────────────────
//
// "Orientation is never a place to fail a session." Every `try {} catch {}`
// in inject.mjs is preserved as a Rust fallback, and Node's
// `readJson(file, fallback)` warn-and-fall-back becomes
// `warn_corrupt_json(&file)` + the same fallback. This module is TOTAL: it
// has no delegate arm, no error type, and no panicking path.
//
// ─── Documented divergences (beyond the two command spellings) ─────────────
//
//   * A bundle whose filenames are not valid UTF-8, or sort differently under
//     UTF-16 than UTF-8, delegated in verbs/knowledge.rs. Here they are read
//     lossily and sorted by UTF-8 order — a preamble may not delegate.
//   * A frontmatter scalar carrying a lone-surrogate escape delegated there
//     too; here it reads as an unparseable concept (empty data), which is the
//     same direction collectConcepts already takes for an unreadable file.
//   * `pipelineRecord.route.flags.join(',')` throws in Node when `flags` is
//     not an array (only `state route --set`, which validates, writes that
//     field). Totality wins: a non-array renders `flags=undefined []`.
//   * resolveContext's WorktreeLinkInvalidError would propagate out of
//     resolvePipeline in Node and crash the hook. Here it falls back to the
//     given root — the same direction resolvePipeline already takes for every
//     other unresolvable binding.

use crate::fsutil::{read_json, warn_corrupt_json, ReadJson};
use crate::jsjson;
use crate::state::{bypass_level, ship_visibility};
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

type JMap = Map<String, Value>;

// ─── constants (state.mjs / cells.mjs / knowledge.mjs / backlog.mjs) ───────

/// state.mjs BEE_VERSION.
const BEE_VERSION: &str = "1.20.3";
/// state.mjs GATE_NAMES.
const GATE_NAMES: [&str; 4] = ["context", "shape", "execution", "review"];
/// state.mjs COMMAND_KEYS.
const COMMAND_KEYS: [&str; 4] = ["setup", "start", "test", "verify"];
/// state.mjs normalizeCommands' companion slots (normalized alongside, never
/// listed by the preamble — COMMAND_KEYS is what it iterates).
const WORKTREE_COMPANION_COMMAND_KEYS: [&str; 3] = [
    "worktree_companion_start",
    "worktree_companion_end",
    "worktree_companion_mount",
];
/// state.mjs NO_TEST_SENTINEL.
const NO_TEST_SENTINEL: &str = "none";
/// cells.mjs CEILING_MAX_SHARE / SCARCITY_MIN_TIERED.
const CEILING_MAX_SHARE: f64 = 0.4;
const SCARCITY_MIN_TIERED: i64 = 3;
/// cells.mjs MODEL_TIERS.
const MODEL_TIERS: [&str; 3] = ["extraction", "generation", "ceiling"];
/// inject.mjs NO_WORK_PHASES.
const NO_WORK_PHASES: [&str; 2] = ["idle", "compounding-complete"];
/// inject.mjs CRITICAL_PATTERNS_HEADING.
const CRITICAL_PATTERNS_HEADING: &str = "## Critical patterns";
/// inject.mjs PROJECT_MAP_FILES.
const PROJECT_MAP_FILES: [(&str, &str); 2] =
    [("system-overview.md", "System overview"), ("reading-map.md", "Reading map")];
/// knowledge.mjs KNOWLEDGE_CONTEXT_LANE_BUDGETS / _DEFAULT_BUDGET.
const KNOWLEDGE_CONTEXT_LANE_BUDGETS: [(&str, i64); 4] =
    [("tiny", 8000), ("small", 12000), ("standard", 20000), ("high-risk", 30000)];
const KNOWLEDGE_CONTEXT_DEFAULT_BUDGET: i64 = 20000;
/// backlog.mjs PBI_STATUSES / BACKLOG_STATUSES.
const PBI_STATUSES: [&str; 5] = ["proposed", "in-flight", "parked", "done", "declined"];
const BACKLOG_STATUSES: [&str; 3] = ["proposed", "in-flight", "done"];

// ─── JS value helpers (lift: verbs/status_full.rs:183-255) ─────────────────

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

/// JS strict equality (===) over JSON-representable primitives; `None` models
/// `undefined`. Objects/arrays compare by reference in JS, so two separately
/// parsed values never compare equal here either.
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
fn tpl_or(o: Option<&Value>, fallback: &str) -> String {
    match o {
        None | Some(Value::Null) => fallback.to_string(),
        Some(v) => jsjson::js_to_string(v),
    }
}

/// JS Array.prototype.join over Values (null/undefined render empty).
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

/// JS `\s` (the set String.prototype.trim strips).
fn js_is_space(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

/// JS Math.round: floor(x + 0.5).
fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// Number -> template-literal text.
fn num_str(n: f64) -> String {
    jsjson::js_f64_to_string(n)
}

// ─── fs primitives (lift: verbs/status_full.rs:531-560) ────────────────────

/// `readJson(file, null)`, fail-open. A present-but-unparseable file warns
/// through the shared native helper and yields the fallback — never a bail:
/// inject.mjs's whole posture is that orientation does not fail a session.
fn read_json_open(file: &Path) -> Option<Value> {
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
fn read_json_object(file: &Path) -> Option<JMap> {
    match read_json_open(file) {
        Some(Value::Object(m)) => Some(m),
        _ => None,
    }
}

fn read_text_opt(file: &Path) -> Option<String> {
    std::fs::read(file).ok().map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// fsutil.mjs readJsonl: split /\r?\n/, trim, JSON.parse per line, silent skip.
fn read_jsonl(file: &Path) -> Vec<Value> {
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
fn date_parse_val(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::String(s)) => js_date_parse(s),
        _ => f64::NAN,
    }
}

// ─── localeCompare('en'[, {numeric:true}]) (lift: status_full.rs:449-528) ──

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

// ─── config (state.mjs readConfig / mergeConfigOverlay / normalizeCommands) ─
//
// provenance: lib/state.mjs readConfig (l. 1947) + mergeConfigOverlay
// (l. 1919); Rust lift of crate::state::{merge_config_overlay,
// read_config_raw} (state.rs:109-156) with the ONE change the preamble
// needs: read_config_raw bails on a corrupt config file, and a preamble may
// not bail — the corrupt file warns and reads as absent instead.

fn merge_config_overlay(base: &Value, overlay: &Value) -> Value {
    match overlay {
        Value::Array(items) => Value::Array(items.clone()),
        Value::Object(over) => {
            let base_obj = match base {
                Value::Object(m) => m.clone(),
                _ => JMap::new(),
            };
            let mut out = base_obj.clone();
            for (key, value) in over {
                let merged = match (base_obj.get(key), value) {
                    (Some(b @ Value::Object(_)), Value::Object(_)) => merge_config_overlay(b, value),
                    _ => match value {
                        Value::Array(items) => Value::Array(items.clone()),
                        other => other.clone(),
                    },
                };
                out.insert(key.clone(), merged);
            }
            Value::Object(out)
        }
        _ => base.clone(),
    }
}

/// The merged tracked+overlay config object, advisor key stripped. Fail-open.
fn read_config_raw_open(root: &Path) -> JMap {
    let tracked = read_json_object(&root.join(".bee").join("config.json")).unwrap_or_default();
    let overlay = read_json_object(&root.join(".bee").join("config.local.json"));
    let mut merged = match overlay {
        Some(over) => match merge_config_overlay(&Value::Object(tracked), &Value::Object(over)) {
            Value::Object(m) => m,
            _ => JMap::new(),
        },
        None => tracked,
    };
    merged.shift_remove("advisor");
    merged
}

/// provenance: state.mjs normalizeCommands; Rust lift of
/// verbs/status_full.rs:800-830.
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

/// provenance: state.mjs resolveProductRoot (l. 1065); Rust lift of
/// verbs/status_full.rs:1381-1420 with its buffered warns printed straight to
/// stderr (this module has no emit-time buffer — nothing here can bail, so a
/// warning can never leak alongside partial output).
fn resolve_product_root(root: &Path) -> PathBuf {
    let config = read_config_raw_open(root);
    match config.get("product_root") {
        None | Some(Value::Null) => root.to_path_buf(),
        Some(Value::String(s)) if s.is_empty() => root.to_path_buf(),
        Some(Value::String(s)) => {
            let resolved = if Path::new(s).is_absolute() {
                PathBuf::from(s)
            } else {
                root.join(s)
            };
            let is_dir = std::fs::metadata(&resolved).map(|m| m.is_dir()).unwrap_or(false);
            if !is_dir {
                eprintln!(
                    "bee: config product_root \"{s}\" -> \"{}\" is not an existing directory; product-doc reads (docs/backlog.md, docs/specs/) will find nothing until you fix .bee/config.json product_root. (GitHub #14)",
                    resolved.display()
                );
            }
            resolved
        }
        Some(other) => {
            let ty = match other {
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                _ => "object",
            };
            eprintln!(
                "bee: .bee/config.json product_root must be a string path (got {ty}); ignoring it and using the bee root."
            );
            root.to_path_buf()
        }
    }
}

/// state.mjs bypassBanner — one canonical loud line per active level.
fn bypass_banner(level: &str) -> &'static str {
    match level {
        "total" => "⚡⚡⚡ GATE BYPASS: TOTAL AUTOPILOT — ZERO STOPS. Every gate (any lane, high-risk/hard-gate included), secret-file reads, and review P1 findings auto-proceed; NO human checkpoint remains. Turn off: bee-hive bypass off",
        "full" => "⚡⚡ GATE BYPASS: FULL AUTOPILOT — ALL Gates 1-3 auto-approved including high-risk/hard-gate work; only secret-file reads and a review P1 finding still stop for the human. Turn off: bee-hive bypass off",
        "normal" => "⚡ GATE BYPASS: NORMAL — Gates 1-3 auto-approved for tiny/small/standard work only; high-risk/hard-gate, secret reads, and Gate 4 UAT still stop. Turn off: bee-hive bypass off",
        _ => "",
    }
}

// ─── state layer (state.mjs readState / readOnboarding / readHandoff) ──────
//
// provenance: lib/state.mjs defaultState/readState (l. 1097-1126); Rust lift
// of hooks/chain_nudge.rs:246-288, with the corrupt-file and JS-exotic
// approved_gates arms turned fail-open (warn + defaults) instead of Delegate.

fn default_gates() -> JMap {
    let mut m = JMap::new();
    for g in GATE_NAMES {
        m.insert(g.into(), Value::Bool(false));
    }
    m
}

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

/// Merge `{...defaults, ...overlay}` for a gates-shaped field: falsy overlays
/// (and any non-object) leave the defaults, JS-spread exotica included — the
/// preamble only ever READS gate booleans, so an index-keyed spread of a
/// string could not change a rendered gate either way.
fn merge_gates(overlay: Option<&Value>) -> JMap {
    match overlay {
        Some(Value::Object(o)) => {
            let mut g = default_gates();
            for (k, v) in o {
                g.insert(k.clone(), v.clone());
            }
            g
        }
        _ => default_gates(),
    }
}

/// state.mjs readState — fail-open merge over defaultState() with the D13
/// legacy 'validating' -> 'planning' coercion.
fn read_state(root: &Path) -> JMap {
    let file_state = read_json_object(&root.join(".bee").join("state.json"));
    let mut merged = default_state();
    let Some(state) = file_state else { return merged };
    for (k, v) in &state {
        merged.insert(k.clone(), v.clone()); // existing keys keep position (JS spread)
    }
    merged.insert("approved_gates".into(), Value::Object(merge_gates(state.get("approved_gates"))));
    if merged.get("phase") == Some(&json!("validating")) {
        merged.insert("phase".into(), json!("planning"));
    }
    merged
}

/// state.mjs readOnboarding — `readJson(.bee/onboarding.json, null)`.
fn read_onboarding(root: &Path) -> Option<Value> {
    read_json_open(&root.join(".bee").join("onboarding.json")).filter(truthy_ref)
}

fn truthy_ref(v: &Value) -> bool {
    truthy(v)
}

/// state.mjs readHandoff (l. 1217) — the fail-open DISPLAY read, with `kind`
/// normalized for objects (missing/unknown reads as 'pause', the safe
/// surface-and-wait side). `None` models every falsy result Node's
/// `readJson(file, null)` could hand back, which is exactly what
/// `handoffBlockLines`' `if (!handoff)` guard tests.
///
/// Exported because lib/compaction.mjs's capsule reads the same record — one
/// truth for the handoff block, never two copies of it.
pub fn read_handoff(root: &Path) -> Option<Value> {
    let parsed = read_json_open(&root.join(".bee").join("HANDOFF.json"))?;
    if !truthy(&parsed) {
        return None;
    }
    match parsed {
        Value::Object(m) => {
            let kind = if str_eq(m.get("kind"), "planned-next") { "planned-next" } else { "pause" };
            let mut out = m;
            out.insert("kind".into(), json!(kind)); // JS {...handoff, kind}: key keeps its position
            Some(Value::Object(out))
        }
        other => Some(other),
    }
}

// ─── lanes (state.mjs listLanes / readLane / lanePath) ─────────────────────
//
// provenance: lib/state.mjs l. 1698-1829; Rust lift of
// hooks/chain_nudge.rs:427-505 (read_lane_record, warn_corrupt_lane,
// path_relative) and verbs/status_full.rs:1612-1636 (list_lanes).

fn lanes_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("lanes")
}

/// state.mjs requireLaneFeature — a throw reads as "no lane" at every
/// fail-open call site.
fn require_lane_feature(value: Option<&Value>) -> Option<String> {
    let Some(Value::String(s)) = value else { return None };
    let feature = js_trim(s);
    if feature.is_empty() {
        return None;
    }
    if feature.contains('/') || feature.contains('\\') || feature.contains("..") {
        return None;
    }
    Some(feature.to_string())
}

fn path_relative(root: &Path, file: &Path) -> String {
    match file.strip_prefix(root) {
        Ok(rel) => rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(std::path::MAIN_SEPARATOR_STR),
        Err(_) => file.display().to_string(),
    }
}

fn warn_corrupt_lane(root: &Path, file: &Path) {
    let rel = path_relative(root, file);
    eprintln!(
        "readLane: skipping corrupt lane record \"{rel}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
    );
}

/// state.mjs readLane — fail-open display read; a corrupt or mismatched
/// record warns (byte-identical line) and reads as absent.
fn read_lane(root: &Path, feature: Option<&Value>) -> Option<JMap> {
    let name = require_lane_feature(feature)?;
    let file = lanes_dir(root).join(format!("{name}.json"));
    if !file.exists() {
        return None;
    }
    let parsed = read_json_open(&file);
    let Some(Value::Object(parsed)) = parsed else {
        warn_corrupt_lane(root, &file);
        return None;
    };
    if parsed.get("feature") != Some(&Value::String(name.clone())) {
        warn_corrupt_lane(root, &file);
        return None;
    }
    let mut merged = JMap::new();
    merged.insert("schema_version".into(), json!("1.0"));
    merged.insert("feature".into(), Value::String(name));
    merged.insert("mode".into(), Value::Null);
    merged.insert("phase".into(), json!("idle"));
    merged.insert("approved_gates".into(), Value::Object(default_gates()));
    merged.insert("summary".into(), json!(""));
    merged.insert("next_action".into(), json!(""));
    merged.insert("created_at".into(), Value::Null);
    for (k, v) in &parsed {
        merged.insert(k.clone(), v.clone());
    }
    merged.insert("approved_gates".into(), Value::Object(merge_gates(parsed.get("approved_gates"))));
    if merged.get("phase") == Some(&json!("validating")) {
        merged.insert("phase".into(), json!("planning"));
    }
    Some(merged)
}

/// state.mjs listLanes — fail-open enumeration in directory order.
fn list_lanes(root: &Path) -> Vec<JMap> {
    let Ok(entries) = std::fs::read_dir(lanes_dir(root)) else { return Vec::new() };
    let names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let mut lanes = Vec::new();
    for entry in names {
        let Some(stem) = entry.strip_suffix(".json") else { continue };
        if let Some(record) = read_lane(root, Some(&json!(stem))) {
            lanes.push(record);
        }
    }
    lanes
}

// ─── sessions + roots (claims.mjs readSession, state.mjs controlRootFor) ───
//
// provenance: Rust lift of hooks/chain_nudge.rs:290-440, with every Delegate
// and Crash arm collapsed into the fail-open direction (read as "no session"
// / "the root you were given").

fn well_formed_id(id: &str) -> bool {
    !id.contains('/') && !id.contains('\\') && !id.contains("..")
}

/// claims.mjs readSession (strict=false) — fail-open display read.
fn read_session(control_root: &Path, session_id: &str) -> Option<JMap> {
    let id = js_trim(session_id);
    if id.is_empty() || !well_formed_id(id) {
        return None;
    }
    let file = control_root.join(".bee").join("sessions").join(format!("{id}.json"));
    let session = read_json_object(&file)?;
    if session.get("id") != Some(&Value::String(id.to_string())) {
        return None;
    }
    Some(session)
}

fn js_absolute(p: &Path) -> PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf())
}

/// state.mjs readGitdirFile.
fn read_gitdir_file(file: &Path, base: &Path) -> Option<PathBuf> {
    let raw = std::fs::read_to_string(file).ok()?;
    let mut raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if let Some(rest) = raw.strip_prefix("gitdir:") {
        raw = rest.trim();
    }
    let sep_fixed = if cfg!(windows) { raw.to_string() } else { raw.replace('\\', "/") };
    Some(js_absolute(&base.join(sep_fixed)))
}

enum RootsCore {
    None,
    Ordinary(PathBuf),
    LinkedValid(PathBuf),
}

/// state.mjs resolveRootsCore — an invalid link reads as "no linked main",
/// never the WorktreeLinkInvalidError throw (this renderer is total).
fn resolve_roots_core(start: &Path) -> RootsCore {
    let mut nearest = js_absolute(start);
    loop {
        if nearest.join(".bee").join("onboarding.json").exists() && !nearest.join(".git").exists() {
            return RootsCore::Ordinary(nearest);
        }
        match nearest.parent() {
            Some(p) => nearest = p.to_path_buf(),
            None => break,
        }
    }
    let mut dir = js_absolute(start);
    let located = loop {
        if dir.join(".git").exists() {
            break Some(dir.clone());
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => break None,
        }
    };
    let Some(work_root) = located else {
        let mut dir = js_absolute(start);
        loop {
            if dir.join(".bee").join("onboarding.json").exists() {
                return RootsCore::Ordinary(dir);
            }
            match dir.parent() {
                Some(p) => dir = p.to_path_buf(),
                None => return RootsCore::None,
            }
        }
    };
    let marker = work_root.join(".git");
    let Ok(stat) = std::fs::metadata(&marker) else { return RootsCore::Ordinary(work_root) };
    if !stat.is_file() {
        return RootsCore::Ordinary(work_root);
    }
    let Some(gitdir) = read_gitdir_file(&marker, &work_root) else {
        return RootsCore::Ordinary(work_root);
    };
    let worktrees_root = gitdir.parent().map(Path::to_path_buf).unwrap_or_default();
    let common_git_dir = worktrees_root.parent().map(Path::to_path_buf).unwrap_or_default();
    if !(common_git_dir.file_name().is_some_and(|n| n == ".git")
        && worktrees_root.file_name().is_some_and(|n| n == "worktrees"))
    {
        return RootsCore::Ordinary(work_root);
    }
    let id = gitdir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if id.is_empty() || id == "." || id == ".." {
        return RootsCore::Ordinary(work_root);
    }
    let reverse = read_gitdir_file(&gitdir.join("gitdir"), &gitdir);
    if reverse.as_deref() != Some(js_absolute(&marker).as_path()) {
        return RootsCore::Ordinary(work_root);
    }
    match common_git_dir.parent().map(Path::to_path_buf) {
        Some(main_root) => RootsCore::LinkedValid(main_root),
        None => RootsCore::Ordinary(work_root),
    }
}

/// state.mjs controlRootFor(root).
fn control_root_for(root: &Path) -> PathBuf {
    match resolve_roots_core(root) {
        RootsCore::None => root.to_path_buf(),
        RootsCore::Ordinary(work_root) => work_root,
        RootsCore::LinkedValid(main_root) => main_root,
    }
}

/// state.mjs resolvePipeline's return, narrowed to what the preamble reads.
struct Pipeline {
    ok: bool,
    source: &'static str,
    feature: Option<String>,
    record: JMap,
}

/// state.mjs resolvePipeline (l. 1854): session record -> bound lane ->
/// default state.json. A binding that names an invalid/missing/corrupt lane
/// is a typed refusal (`ok:false`), never a silent fallback — the caller
/// then renders the DEFAULT record, exactly as inject.mjs does.
fn resolve_pipeline(root: &Path, session_id: Option<&str>) -> Pipeline {
    let defaults = || Pipeline {
        ok: true,
        source: "default",
        feature: None,
        record: read_state(root),
    };
    let refusal = || Pipeline {
        ok: false,
        source: "default",
        feature: None,
        record: JMap::new(),
    };
    let Some(sid) = session_id.filter(|s| !js_trim(s).is_empty()) else { return defaults() };
    let control_root = control_root_for(root);
    let Some(session) = read_session(&control_root, sid) else { return defaults() };
    let bound = match session.get("lane") {
        Some(Value::String(s)) => js_trim(s).to_string(),
        _ => String::new(),
    };
    if bound.is_empty() {
        return defaults();
    }
    if !well_formed_id(&bound) {
        return refusal(); // LANE_INVALID
    }
    let file = lanes_dir(&control_root).join(format!("{bound}.json"));
    if !file.exists() {
        return refusal(); // LANE_MISSING
    }
    match read_lane(&control_root, Some(&json!(bound.clone()))) {
        Some(record) => Pipeline { ok: true, source: "lane", feature: Some(bound), record },
        None => refusal(), // LANE_CORRUPT
    }
}

// ─── cells (cells.mjs listCells / scribingDebt / globalScribingDebt / …) ───
//
// provenance: lib/cells.mjs l. 692-738 (listCells), 2019-2058 (scribingDebt),
// 2098-2187 (scribingRunStampMs/bestScribingStampMs/globalScribingDebt),
// 2280-2307 (tierMix/ceilingScarcityWarning); Rust lift of
// verbs/status_full.rs:1837-2000 and hooks/chain_nudge.rs:645-830.

/// cells.mjs listCells over the ACTIVE dir only (no caller here passes
/// includeArchived), sorted by numeric-aware id compare.
fn list_cells(root: &Path, feature: Option<&Value>, status: Option<&str>) -> Vec<JMap> {
    let dir = root.join(".bee").join("cells");
    let Ok(rd) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut cells: Vec<JMap> = Vec::new();
    for entry in rd.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue; // the `archive` child (or any dir) is never a cell
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        let Some(cell) = read_json_object(&entry.path()) else { continue };
        // JS `if (feature && cell.feature !== feature) continue`.
        if let Some(f) = feature.filter(|f| truthy(f)) {
            if !strict_eq(cell.get("feature"), Some(f)) {
                continue;
            }
        }
        if let Some(s) = status {
            if !str_eq(cell.get("status"), s) {
                continue;
            }
        }
        cells.push(cell);
    }
    cells.sort_by(|a, b| {
        locale_cmp(&tpl(a.get("id")), &tpl(b.get("id")), true)
    });
    cells
}

/// cells.mjs scribingRunStampMs: Date.parse(run.at || run.date).
fn scribing_run_stamp_ms(run: Option<&Value>) -> Option<f64> {
    let run = run?;
    if !truthy(run) {
        return None;
    }
    let at = vget(run, "at").filter(|v| truthy(v));
    let candidate = match at {
        Some(v) => Some(v),
        None => vget(run, "date"),
    };
    let parsed = date_parse_val(candidate);
    parsed.is_finite().then_some(parsed)
}

/// cells.mjs bestScribingStampMs — ledger max, then the feature's own lane
/// stamp, then the default record's stamp (only when it names this feature).
fn best_scribing_stamp_ms(
    root: &Path,
    feature: &Value,
    ledger: &[Value],
    state: &JMap,
) -> Option<f64> {
    let mut best: Option<f64> = None;
    let mut consider = |ms: Option<f64>| {
        if let Some(v) = ms {
            if best.is_none() || v > best.unwrap() {
                best = Some(v);
            }
        }
    };
    for entry in ledger {
        if !truthy(entry) || !strict_eq(vget(entry, "feature"), Some(feature)) {
            continue;
        }
        let parsed = date_parse_val(vget(entry, "ts"));
        consider(parsed.is_finite().then_some(parsed));
    }
    if let Some(lane) = read_lane(root, Some(feature)) {
        consider(scribing_run_stamp_ms(lane.get("last_scribing_run")));
    }
    if let Some(run) = state.get("last_scribing_run") {
        if truthy(run) && strict_eq(vget(run, "feature"), Some(feature)) {
            consider(scribing_run_stamp_ms(Some(run)));
        }
    }
    best
}

fn read_scribing_ledger(root: &Path) -> Vec<Value> {
    read_jsonl(&root.join(".bee").join("logs").join("scribing-runs.jsonl"))
}

/// cells.mjs scribingDebt(root) — no opts on the preamble path.
fn scribing_debt(root: &Path) -> (usize, Vec<Value>) {
    let state = read_state(root);
    let feature = state.get("feature").cloned().unwrap_or(Value::Null);
    if !truthy(&feature) {
        return (0, Vec::new());
    }
    let ledger = read_scribing_ledger(root);
    let threshold = best_scribing_stamp_ms(root, &feature, &ledger, &state).unwrap_or(0.0);
    let mut ids = Vec::new();
    for cell in list_cells(root, Some(&feature), Some("capped")) {
        let trace = match cell.get("trace") {
            Some(v) if truthy(v) => v.clone(),
            _ => json!({}),
        };
        if vget(&trace, "behavior_change") != Some(&Value::Bool(true)) {
            continue;
        }
        let capped_at = date_parse_val(vget(&trace, "capped_at"));
        if capped_at.is_finite() && capped_at > threshold {
            ids.push(cell.get("id").cloned().unwrap_or(Value::Null));
        }
    }
    (ids.len(), ids)
}

/// cells.mjs globalScribingDebt — the orphan sweep across every feature.
fn global_scribing_debt(root: &Path) -> (usize, Vec<(String, Vec<Value>)>) {
    let cells: Vec<JMap> = list_cells(root, None, Some("capped"))
        .into_iter()
        .filter(|cell| {
            let trace = match cell.get("trace") {
                Some(v) if truthy(v) => v.clone(),
                _ => json!({}),
            };
            vget(&trace, "behavior_change") == Some(&Value::Bool(true))
        })
        .collect();
    if cells.is_empty() {
        return (0, Vec::new());
    }
    let state = read_state(root);
    let ledger = read_scribing_ledger(root);
    let mut stamp_cache: HashMap<String, Option<f64>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut by_feature: HashMap<String, Vec<Value>> = HashMap::new();
    for cell in &cells {
        let Some(feature_v) = cell.get("feature").filter(|f| truthy(f)) else { continue };
        let key = jsjson::js_to_string(feature_v);
        let trace = match cell.get("trace") {
            Some(v) if truthy(v) => v.clone(),
            _ => json!({}),
        };
        let capped_at = date_parse_val(vget(&trace, "capped_at"));
        let stamp = match stamp_cache.get(&key) {
            Some(s) => *s,
            None => {
                let s = best_scribing_stamp_ms(root, feature_v, &ledger, &state);
                stamp_cache.insert(key.clone(), s);
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
        if !by_feature.contains_key(&key) {
            order.push(key.clone());
            by_feature.insert(key.clone(), Vec::new());
        }
        by_feature
            .get_mut(&key)
            .unwrap()
            .push(cell.get("id").cloned().unwrap_or(Value::Null));
    }
    // .sort((a, b) => a.feature.localeCompare(b.feature, 'en')) — non-numeric.
    order.sort_by(|a, b| locale_cmp(a, b, false));
    let mut features = Vec::new();
    let mut count = 0usize;
    for feature in order {
        let ids = by_feature.remove(&feature).unwrap_or_default();
        count += ids.len();
        features.push((feature, ids));
    }
    (count, features)
}

/// cells.mjs ceilingScarcityWarning -> (pct, ceiling, tiered).
fn ceiling_scarcity_warning(root: &Path) -> Option<(f64, i64, i64)> {
    let state = read_state(root);
    let feature = state.get("feature").cloned().unwrap_or(Value::Null);
    let filter = if truthy(&feature) { Some(feature) } else { None };
    let cells = list_cells(root, filter.as_ref(), None);
    let (mut extraction, mut generation, mut ceiling) = (0i64, 0i64, 0i64);
    for cell in &cells {
        match cell.get("tier").and_then(|t| t.as_str()) {
            Some(t) if MODEL_TIERS.contains(&t) => match t {
                "extraction" => extraction += 1,
                "generation" => generation += 1,
                _ => ceiling += 1,
            },
            _ => {}
        }
    }
    let tiered = extraction + generation + ceiling;
    if tiered < SCARCITY_MIN_TIERED {
        return None;
    }
    let share = if tiered > 0 { ceiling as f64 / tiered as f64 } else { 0.0 };
    if share <= CEILING_MAX_SHARE {
        return None;
    }
    Some((js_round(share * 100.0), ceiling, tiered))
}

// ─── capture queue (capture.mjs pendingCaptureStubs / captureQueue) ────────
//
// provenance: lib/capture.mjs l. 85-103; Rust lift of
// verbs/status_full.rs:2463-2490 (only the count is read here).

fn capture_queue_count(root: &Path) -> usize {
    let events = read_jsonl(&root.join(".bee").join("capture-queue.jsonl"));
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
    stubs
        .into_iter()
        .filter(|s| !flushed.iter().any(|f| strict_eq(Some(f), vget(s, "id"))))
        .count()
}

// ─── decisions (decisions.mjs activeDecisions / datamark) ──────────────────
//
// provenance: lib/decisions.mjs l. 810-838 (default branch — the preamble
// never passes `all`) and l. 1047-1054 (datamark); Rust lift of
// verbs/status_full.rs:2280-2440.

fn decisions_path(root: &Path) -> PathBuf {
    root.join(".bee").join("decisions.jsonl")
}

/// decisions.mjs buildTagOverlay — latest tag event wins (date, then file
/// order). A mixed finite/NaN date set delegated in the verb port; here the
/// NaN rows simply keep file order, which is what V8's sort does for them in
/// practice and is the fail-open direction for an orientation block.
fn build_tag_overlay(events: &[Value]) -> Vec<(Value, (Option<Value>, Option<Value>))> {
    let mut tag_events: Vec<(usize, &Value, f64)> = Vec::new();
    for (idx, e) in events.iter().enumerate() {
        if truthy(e)
            && str_eq(vget(e, "type"), "tag")
            && matches!(vget(e, "target"), Some(Value::String(_)))
        {
            tag_events.push((idx, e, date_parse_val(vget(e, "date"))));
        }
    }
    tag_events.sort_by(|a, b| {
        let (x, y) = (a.2, b.2);
        let ord = match (x.is_finite(), y.is_finite()) {
            (true, true) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
            _ => Ordering::Equal,
        };
        ord.then(a.0.cmp(&b.0))
    });
    let mut overlay: Vec<(Value, (Option<Value>, Option<Value>))> = Vec::new();
    for (_, e, _) in tag_events {
        let target = vget(e, "target").cloned().unwrap_or(Value::Null);
        let patch = (
            match vget(e, "tags") {
                Some(Value::Array(a)) => Some(Value::Array(a.clone())),
                _ => None,
            },
            match vget(e, "scope") {
                Some(Value::String(s)) if !s.is_empty() => Some(Value::String(s.clone())),
                _ => None,
            },
        );
        if let Some(slot) = overlay.iter_mut().find(|(k, _)| strict_eq(Some(k), Some(&target))) {
            slot.1 = patch;
        } else {
            overlay.push((target, patch));
        }
    }
    overlay
}

fn apply_tag_overlay(
    event: &Value,
    overlay: &[(Value, (Option<Value>, Option<Value>))],
) -> Value {
    let Some(id) = vget(event, "id") else { return event.clone() };
    let Some((_, (tags, scope))) = overlay.iter().find(|(k, _)| strict_eq(Some(k), Some(id))) else {
        return event.clone();
    };
    let Value::Object(m) = event else { return event.clone() };
    let mut next = m.clone();
    if let Some(tags) = tags {
        next.insert("tags".into(), tags.clone());
    }
    if let Some(scope) = scope {
        next.insert("scope".into(), scope.clone());
    }
    Value::Object(next)
}

/// decisions.mjs activeDecisions(root, { recent }) — default branch only.
fn active_decisions(root: &Path, recent: Option<usize>) -> Vec<Value> {
    let events = read_jsonl(&decisions_path(root));
    let overlay = build_tag_overlay(&events);
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
    let in_set = |set: &[Value], id: Option<&Value>| set.iter().any(|v| strict_eq(Some(v), id));
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
    let s = match text {
        None | Some(Value::Null) => String::new(),
        Some(v) => jsjson::js_to_string(v),
    };
    // .replace(/```+/g, '')
    let chars: Vec<char> = s.chars().collect();
    let mut no_ticks = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '`' {
            let mut j = i;
            while j < chars.len() && chars[j] == '`' {
                j += 1;
            }
            if j - i < 3 {
                for k in i..j {
                    no_ticks.push(chars[k]);
                }
            }
            i = j;
            continue;
        }
        no_ticks.push(chars[i]);
        i += 1;
    }
    let no_tags = strip_role_tags(&no_ticks);
    let cleaned: String = no_tags
        .chars()
        .filter(|c| {
            let cp = *c as u32;
            !(cp <= 0x08 || cp == 0x0B || cp == 0x0C || (0x0E..=0x1F).contains(&cp) || cp == 0x7F)
        })
        .collect();
    format!("«{}»", js_trim(&cleaned))
}

/// `/<\/?\s*(?:system|assistant|user|developer|tool)\b[^>]*>/gi`
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
            while j < chars.len() && js_is_space(chars[j]) {
                j += 1;
            }
            for role in ROLES {
                let rl: Vec<char> = role.chars().collect();
                if j + rl.len() <= chars.len()
                    && chars[j..j + rl.len()]
                        .iter()
                        .zip(rl.iter())
                        .all(|(a, b)| a.to_ascii_lowercase() == *b)
                {
                    let after = j + rl.len();
                    // \b — the role name must not run straight into a word char.
                    let boundary = after >= chars.len()
                        || !(chars[after].is_alphanumeric() || chars[after] == '_');
                    if boundary {
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
        out.push(chars[i]);
        i += 1;
    }
    out
}

// ─── backlog counts (backlog.mjs readBacklogCounts) ────────────────────────
//
// provenance: lib/backlog.mjs foldPbis/foldedBacklogCounts/
// legacyBacklogCounts; Rust lift of verbs/status_full.rs:2512-2632.

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

fn read_backlog_counts(root: &Path) -> Option<JMap> {
    let text = read_text_opt(&root.join(".bee").join("backlog.jsonl"));
    let mut has_events = false;
    let mut items: HashMap<String, String> = HashMap::new();
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
            match vget(&row, "event").and_then(|v| v.as_str()).unwrap_or("") {
                "add" => {
                    if items.contains_key(&id) {
                        continue;
                    }
                    let status = match vget(&row, "status").and_then(|v| v.as_str()) {
                        Some(s) if PBI_STATUSES.contains(&s) => s.to_string(),
                        _ => "proposed".to_string(),
                    };
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
        return Some(counts);
    }
    // legacyBacklogCounts over <productRoot>/docs/backlog.md.
    let file = resolve_product_root(root).join("docs").join("backlog.md");
    let text = read_text_opt(&file)?;
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
    Some(counts)
}

// ─── knowledge bundle (knowledge.mjs bundleDir/collectConcepts/bundleMode) ─
//
// provenance: lib/knowledge.mjs l. 144-146, 499-655, 754-816; Rust lift of
// verbs/knowledge.rs:220-700 (key_re_ok, is_reserved_basename,
// parse_frontmatter and its scalar/flow-list helpers, list_bundle_markdown).
// The two delegate arms that port carries (non-sortable filenames,
// lone-surrogate escapes) are collapsed to the fail-open direction here.

fn bundle_dir(root: &Path) -> PathBuf {
    resolve_product_root(root).join("docs").join("knowledge")
}

fn key_re_ok(key: &str) -> bool {
    // /^[A-Za-z_][A-Za-z0-9_.-]*$/
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

fn is_reserved_basename(base: &str) -> bool {
    base == "index.md" || base == "log.md"
}

fn parse_scalar_token(raw: &str) -> Option<Value> {
    if raw == "true" {
        return Some(Value::Bool(true));
    }
    if raw == "false" {
        return Some(Value::Bool(false));
    }
    if raw.starts_with('"') {
        return match serde_json::from_str::<Value>(raw) {
            Ok(Value::String(s)) => Some(Value::String(s)),
            _ => None,
        };
    }
    if raw.starts_with('\'') {
        return None;
    }
    if matches!(
        raw.chars().next(),
        Some('&' | '*' | '!' | '|' | '>' | '%' | '@' | '`' | '{' | '}')
    ) {
        return None;
    }
    Some(Value::String(raw.to_string()))
}

fn parse_flow_list(raw: &str) -> Option<Value> {
    if !raw.ends_with(']') {
        return None;
    }
    let inner = js_trim(&raw[1..raw.len() - 1]);
    if inner.is_empty() {
        return Some(Value::Array(Vec::new()));
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
        return None;
    }
    segments.push(current);
    let mut value = Vec::new();
    for segment in &segments {
        let token = js_trim(segment);
        if token.is_empty() {
            return None;
        }
        value.push(parse_scalar_token(token)?);
    }
    Some(Value::Array(value))
}

fn parse_key_value_line(line: &str, target: &mut JMap) -> Option<()> {
    let sep = line.find(": ")?;
    let key = &line[..sep];
    if !key_re_ok(key) || target.contains_key(key) {
        return None;
    }
    let raw = &line[sep + 2..];
    if raw.is_empty() {
        return None;
    }
    let parsed = if raw.starts_with('[') { parse_flow_list(raw)? } else { parse_scalar_token(raw)? };
    target.insert(key.to_string(), parsed);
    Some(())
}

/// knowledge.mjs parseFrontmatter, narrowed to what the preamble needs:
/// `Some(data)` when frontmatter is present AND accepted, `None` otherwise
/// (absent, unclosed, or outside the emitted subset). Both callers below
/// treat every `None` the same way collectConcepts treats an unreadable
/// file — an empty-data row, never a failure.
fn parse_frontmatter(text: &str) -> Option<JMap> {
    let open_len = if text.starts_with("---\r\n") {
        5
    } else if text.starts_with("---\n") {
        4
    } else {
        return None;
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
    block_end?;
    let inner_raw = &text[open_len..inner_end];
    let inner_lines: Vec<&str> = if inner_raw.is_empty() {
        Vec::new()
    } else {
        let mut v: Vec<&str> = inner_raw.split('\n').collect();
        v.pop();
        v
    };
    let mut data: JMap = JMap::new();
    let mut in_bee_map = false;
    for raw_line in inner_lines {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.is_empty() || line.contains('\t') {
            return None;
        }
        if let Some(inner) = line.strip_prefix("  ") {
            if !in_bee_map || inner.starts_with(' ') {
                return None;
            }
            let bee = data.get_mut("bee").and_then(Value::as_object_mut)?;
            parse_key_value_line(inner, bee)?;
            continue;
        }
        if line.starts_with(' ') {
            return None;
        }
        in_bee_map = false;
        // /^([^:\s]+):$/ — a map header line.
        let header_key = line
            .strip_suffix(':')
            .filter(|key| !key.is_empty() && key.chars().all(|c| c != ':' && !js_is_space(c)));
        if let Some(key) = header_key {
            if !key_re_ok(key) || key != "bee" || data.contains_key("bee") {
                return None;
            }
            data.insert("bee".to_string(), Value::Object(JMap::new()));
            in_bee_map = true;
            continue;
        }
        parse_key_value_line(line, &mut data)?;
    }
    Some(data)
}

/// lstat-level symlink test matching Node's dirent.isSymbolicLink().
fn is_symlinkish(path: &Path) -> bool {
    let Ok(md) = std::fs::symlink_metadata(path) else { return false };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        (md.file_attributes() & 0x400) != 0 // FILE_ATTRIBUTE_REPARSE_POINT
    }
    #[cfg(not(windows))]
    {
        md.file_type().is_symlink()
    }
}

/// knowledge.mjs listBundleMarkdown — never leaves docs/knowledge/ (D23).
fn list_bundle_markdown(dir: &Path) -> Vec<String> {
    fn walk(abs: &Path, rel: &str, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(abs) else { return };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let child_abs = entry.path();
            if is_symlinkish(&child_abs) {
                continue; // a symlink could escape the bundle — never follow (D23)
            }
            let Ok(ft) = entry.file_type() else { continue };
            let child_rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            if ft.is_dir() {
                walk(&child_abs, &child_rel, out);
            } else if ft.is_file() && name.ends_with(".md") {
                out.push(child_rel);
            }
        }
    }
    let mut out = Vec::new();
    if dir.exists() {
        walk(dir, "", &mut out);
    }
    out.sort();
    out
}

fn read_file_lossy(path: &Path) -> Option<String> {
    std::fs::read(path).ok().map(|b| String::from_utf8_lossy(&b).into_owned())
}

struct Concept {
    path: String,
    data: JMap,
}

/// knowledge.mjs collectConcepts — the ONE inventory path (D12).
fn collect_concepts(root: &Path) -> Vec<Concept> {
    let dir = bundle_dir(root);
    let mut concepts = Vec::new();
    for rel in list_bundle_markdown(&dir) {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if is_reserved_basename(base) {
            continue;
        }
        let data = read_file_lossy(&join_rel(&dir, &rel))
            .and_then(|text| parse_frontmatter(&text))
            .unwrap_or_default();
        concepts.push(Concept { path: rel, data });
    }
    concepts
}

fn join_rel(dir: &Path, rel: &str) -> PathBuf {
    let mut p = dir.to_path_buf();
    for seg in rel.split('/') {
        p.push(seg);
    }
    p
}

/// knowledge.mjs bundleMode (G8) — the ONE "does this repo have a bundle?"
/// predicate. Never throws; a missing root, an unreadable tree, or a FILE
/// where the bundle directory should be all read as `false`.
fn bundle_mode(root: &Path) -> bool {
    let dir = bundle_dir(root);
    match std::fs::metadata(&dir) {
        Ok(m) if m.is_dir() => {}
        _ => return false,
    }
    for rel in list_bundle_markdown(&dir) {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if is_reserved_basename(base) {
            continue;
        }
        let Some(text) = read_file_lossy(&join_rel(&dir, &rel)) else { continue };
        if let Some(data) = parse_frontmatter(&text) {
            if matches!(data.get("type"), Some(Value::String(t)) if !t.is_empty()) {
                return true;
            }
        }
    }
    false
}

// ─── inject.mjs proper ─────────────────────────────────────────────────────

/// `state.adoptHandoff`'s typed result, as the SessionStart hook computes it
/// and hands it to the renderer. The renderer NEVER mutates: adoption is the
/// hook's job (inject.mjs's PURITY PIN, fsh-10 panel W2).
#[derive(Debug, Clone, Default)]
pub struct HandoffOutcome {
    pub ok: bool,
    pub code: Option<String>,
    pub reason: Option<String>,
    pub next_cell: Option<String>,
}

/// inject.mjs knowledgeContextBudgetForMode (i54-closeout D3): the ONE preset
/// table `--lane` resolves against; an unset/unrecognized mode falls back to
/// the bare default.
fn knowledge_context_budget_for_mode(mode: Option<&Value>) -> i64 {
    let Some(Value::String(m)) = mode else { return KNOWLEDGE_CONTEXT_DEFAULT_BUDGET };
    KNOWLEDGE_CONTEXT_LANE_BUDGETS
        .iter()
        .find(|(k, _)| k == m)
        .map(|(_, v)| *v)
        .unwrap_or(KNOWLEDGE_CONTEXT_DEFAULT_BUDGET)
}

fn is_no_work_phase(record: &JMap) -> bool {
    let phase = record.get("phase");
    NO_WORK_PHASES.iter().any(|p| str_eq(phase, p))
}

/// inject.mjs visibleGates — gate 4 is user-invoked, so it is pending only
/// inside a live review session, and a terminal record owes no gate at all.
fn visible_gates(record: &JMap) -> Vec<&'static str> {
    if is_no_work_phase(record) {
        return Vec::new();
    }
    if str_eq(record.get("phase"), "reviewing") {
        GATE_NAMES.to_vec()
    } else {
        GATE_NAMES.iter().copied().filter(|g| *g != "review").collect()
    }
}

/// The first gate this record still owes, or `None`. Shared with the compact
/// capsule (compaction-hardening D6 item 8) so a compacted session is told
/// about exactly the gates a live session would be told about.
pub fn first_open_gate(record: &JMap) -> Option<&'static str> {
    let gates = record.get("approved_gates");
    visible_gates(record)
        .into_iter()
        .find(|gate| !matches!(gates.and_then(|v| vget(v, gate)), Some(Value::Bool(true))))
}

fn gates_line(record: &JMap) -> String {
    let shown = visible_gates(record);
    if shown.is_empty() {
        return "none pending (no active work)".to_string();
    }
    let gates = record.get("approved_gates");
    shown
        .iter()
        .map(|gate| {
            let approved = matches!(gates.and_then(|v| vget(v, gate)), Some(Value::Bool(true)));
            format!("{gate}: {}", if approved { "approved" } else { "pending" })
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

// ─── the three shared renderers (compaction-hardening cz-5, D6/D26) ────────
//
// build_session_preamble below and build_compact_capsule (hooks/compaction.rs)
// are two callers of ONE truth for each of these three blocks, never two
// copies of it: D6 items 3, 4 and 5 require the capsule to carry these EXACT
// bytes, and a second hand-written copy is the classic way "verbatim" quietly
// stops being verbatim one edit later.
//
// BLANK-LINE OWNERSHIP IS THE CALLER'S — decided deliberately (cz-5 STEP 1).
// The preamble opens its HANDOFF block with a bare `lines.push("")`. That
// blank is SPACING BETWEEN SECTIONS, not part of the block: the capsule
// composes its own sections with its own joiner, and a helper that carried a
// leading blank would force the capsule to inherit the preamble's spacing
// assumptions (and would make the block's first byte depend on what happens
// to precede it). `handoff_block_lines` therefore returns the block's OWN
// lines only; each caller emits its own separator.

/// inject.mjs's onboarding line, all three arms (missing / drifted / ok).
pub fn onboarding_line(onboarding: Option<&Value>) -> String {
    let Some(onboarding) = onboarding.filter(|v| truthy(v)) else {
        return "- Onboarding: MISSING — run bee-hive onboarding before anything else.".to_string();
    };
    let version = vget(onboarding, "bee_version");
    if opt_truthy(version) && !str_eq(version, BEE_VERSION) {
        return format!(
            "- Onboarding: installed at bee {} but plugin is {BEE_VERSION} — re-run onboarding to refresh vendored helpers.",
            tpl(version)
        );
    }
    let shown = if opt_truthy(version) { tpl(version) } else { BEE_VERSION.to_string() };
    format!("- Onboarding: ok (bee {shown})")
}

/// The loud gate-bypass banner: `[]` when off, 1 line for normal, 2 for
/// full/total.
pub fn bypass_banner_lines(level: &str) -> Vec<String> {
    if level.is_empty() || level == "off" {
        return Vec::new();
    }
    let mut lines = vec![format!("- {}", bypass_banner(level))];
    if level == "full" || level == "total" {
        let tail = if level == "total" {
            "This includes secret-file reads and review P1 findings: nothing pauses for the human."
        } else {
            "Only reading a secret-shaped file and a review P1 finding still pause for the human."
        };
        lines.push(format!(
            "  The agent does NOT stop for these gates — it records the recommended choice, logs a one-line audit decision, and continues. {tail}"
        ));
    }
    lines
}

/// The wait-and-never-auto-resume HANDOFF block, WITHOUT its leading blank.
///
/// `outcome` is a real parameter, not a formality (D26): the SessionStart
/// hook sets `{ok:false, code:"WRONG_SOURCE"}` whenever a planned-next
/// handoff exists on a non-adopting source — `compact` included — and the
/// refusal REASON is the only thing telling the session why it is waiting
/// instead of starting. Dropping the parameter renders a block that looks
/// verbatim and is not.
pub fn handoff_block_lines(handoff: &Value, outcome: Option<&HandoffOutcome>) -> Vec<String> {
    if !truthy(handoff) {
        return Vec::new();
    }
    let mut lines =
        vec!["### HANDOFF present — present it and WAIT — never auto-resume".to_string()];
    lines.push(format!(
        "- Phase: {} | Feature: {} | Mode: {}",
        tpl_or(vget(handoff, "phase"), "unknown"),
        tpl_or(vget(handoff, "feature"), "unknown"),
        tpl_or(vget(handoff, "mode"), "unknown"),
    ));
    if let Some(Value::Array(cells)) = vget(handoff, "cells_in_flight") {
        if !cells.is_empty() {
            lines.push(format!("- Cells in flight: {}", js_join(cells, ", ")));
        }
    }
    if opt_truthy(vget(handoff, "next_action")) {
        lines.push(format!("- Saved next action: {}", tpl(vget(handoff, "next_action"))));
    }
    if str_eq(vget(handoff, "kind"), "planned-next") {
        if let Some(outcome) = outcome.filter(|o| !o.ok) {
            let reason = outcome
                .reason
                .clone()
                .or_else(|| outcome.code.clone())
                .unwrap_or_else(|| "unknown reason".to_string());
            lines.push(format!("- Adoption not applied: {reason}"));
        }
    }
    lines
}

/// inject.mjs knowledgeContextLines (okf-8, D38) — the startup bridge.
/// Silence beats a nag: no active feature emits nothing at all.
fn knowledge_context_lines(root: &Path, record: &JMap) -> Vec<String> {
    let feature = match record.get("feature") {
        Some(Value::String(s)) => js_trim(s).to_string(),
        _ => String::new(),
    };
    if feature.is_empty() || is_no_work_phase(record) {
        return Vec::new();
    }
    let has_work_item = collect_concepts(root).iter().any(|concept| {
        if !matches!(concept.data.get("type"), Some(Value::String(t)) if t == "bee.work-item") {
            return false;
        }
        let bee = match concept.data.get("bee") {
            Some(Value::Object(m)) => m.clone(),
            _ => JMap::new(),
        };
        strict_eq(bee.get("id"), Some(&json!(feature.clone())))
    });
    if !has_work_item {
        return vec![format!(
            "- No knowledge work item for \"{feature}\" — offer to author docs/knowledge/work/{feature}/work-item.md (template: docs/knowledge/areas/okf-profile/concept-model-and-authoring.md, Templates) so the next session starts from curated context."
        )];
    }
    let budget = knowledge_context_budget_for_mode(record.get("mode"));
    vec![
        "### Knowledge context — load it before code".to_string(),
        // CUTOVER wording divergence: inject.mjs:260 spelled this
        // `node .bee/bin/bee.mjs knowledge context …`.
        format!("- `.bee/bin/bee knowledge context --work {feature} --budget {budget}`"),
        "- Run it and read the manifest's files before touching code — that manifest is this feature's curated context, and it replaces scanning docs/history.".to_string(),
    ]
}

/// inject.mjs projectMapLines (D5/D10 + okf-integration-close-f4 D2).
fn project_map_lines(root: &Path, bundle: bool) -> Vec<String> {
    let mut lines = vec!["### Project map".to_string()];
    let body = if bundle { bundle_project_map_lines(root) } else { spec_project_map_lines(root) };
    lines.extend(body);
    if let Some(backlog) = read_backlog_counts(root) {
        lines.push(format!(
            "- PBI: {} done / {} in-flight / {} proposed",
            tpl(backlog.get("done")),
            tpl(backlog.get("inFlight")),
            tpl(backlog.get("proposed")),
        ));
    }
    lines
}

fn spec_project_map_lines(root: &Path) -> Vec<String> {
    let specs_dir = resolve_product_root(root).join("docs").join("specs");
    let present: Vec<(&str, &str)> = PROJECT_MAP_FILES
        .iter()
        .copied()
        .filter(|(file, _)| specs_dir.join(file).exists())
        .collect();
    if present.is_empty() {
        return vec![
            "- Project map missing (Q1/Q2 unanswerable from repo) — bee-capturing bootstrap available.".to_string(),
        ];
    }
    let mut lines: Vec<String> = present
        .iter()
        .map(|(file, label)| format!("- {label}: docs/specs/{file}"))
        .collect();
    let area_count = std::fs::read_dir(&specs_dir)
        .map(|rd| {
            rd.flatten()
                .filter(|entry| {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    entry.file_type().map(|t| t.is_file()).unwrap_or(false)
                        && name.ends_with(".md")
                        && !PROJECT_MAP_FILES.iter().any(|(file, _)| *file == name)
                })
                .count()
        })
        .unwrap_or(0);
    lines.push(format!(
        "- Specced areas: {area_count} (docs/specs/ — read the spec before the code)"
    ));
    lines
}

fn bundle_project_map_lines(root: &Path) -> Vec<String> {
    let mut lines = vec![
        "- Knowledge bundle: docs/knowledge/ (index: docs/knowledge/index.md) — read the bundle before the code".to_string(),
    ];
    let concepts = collect_concepts(root);
    let mut areas: Vec<String> = Vec::new();
    for concept in &concepts {
        if let Some(rest) = concept.path.strip_prefix("areas/") {
            if let Some(idx) = rest.find('/') {
                let slug = &rest[..idx];
                if !slug.is_empty() && !areas.iter().any(|a| a == slug) {
                    areas.push(slug.to_string());
                }
            }
        }
    }
    lines.push(format!(
        "- Bundle holds: {} area(s), {} concept(s) (docs/specs/ is the read-only compatibility surface)",
        areas.len(),
        concepts.len()
    ));
    lines
}

/// inject.mjs criticalPatternsDigest — routes on the ONE bundle predicate
/// (G12), same line cap in both branches.
fn critical_patterns_digest(root: &Path, max_lines: usize, bundle: bool) -> Option<Vec<String>> {
    if bundle {
        bundle_critical_patterns_digest(root, max_lines)
    } else {
        legacy_critical_patterns_digest(root, max_lines)
    }
}

fn legacy_critical_patterns_digest(root: &Path, max_lines: usize) -> Option<Vec<String>> {
    let file = root
        .join("docs")
        .join("history")
        .join("learnings")
        .join("critical-patterns.md");
    let text = read_file_lossy(&file)?;
    let lines: Vec<String> = text
        .split('\n')
        .map(|l| js_trim(l.strip_suffix('\r').unwrap_or(l)).to_string())
        .filter(|l| !l.is_empty() && !l.starts_with("<!--"))
        .collect();
    if lines.is_empty() {
        return None;
    }
    Some(lines.into_iter().take(max_lines).collect())
}

/// `.replace(/\]\((?!https?:|\/)/g, '](docs/knowledge/')` — index links are
/// bundle-relative, and the preamble is read from the repo root.
fn rewrite_bundle_links(row: &str) -> String {
    let b = row.as_bytes();
    let mut out = String::with_capacity(row.len());
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b']' && i + 1 < b.len() && b[i + 1] == b'(' {
            let rest = &row[i + 2..];
            let excluded = rest.starts_with('/')
                || rest.starts_with("http:")
                || rest.starts_with("https:");
            out.push_str(if excluded { "](" } else { "](docs/knowledge/" });
            i += 2;
            continue;
        }
        // Push one whole char (indices only advance on char boundaries above).
        let ch = row[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn bundle_critical_patterns_digest(root: &Path, max_lines: usize) -> Option<Vec<String>> {
    let text = read_file_lossy(&bundle_dir(root).join("index.md"))?;
    let all: Vec<String> = text
        .split('\n')
        .map(|l| js_trim(l.strip_suffix('\r').unwrap_or(l)).to_string())
        .collect();
    let start = all.iter().position(|l| l == CRITICAL_PATTERNS_HEADING)?;
    let mut rows: Vec<String> = Vec::new();
    for line in all.iter().skip(start + 1) {
        if line.starts_with("## ") {
            break;
        }
        if line.starts_with("- ") {
            rows.push(rewrite_bundle_links(line));
        }
    }
    if rows.is_empty() {
        return None;
    }
    let keep = std::cmp::max(1, max_lines.saturating_sub(1));
    let mut recent: Vec<String> = rows[rows.len().saturating_sub(keep)..].to_vec();
    recent.reverse();
    let mut out = vec![format!(
        "- {} critical pattern(s) in the bundle — the {} most recent below; full list: docs/knowledge/index.md (\"Critical patterns\").",
        rows.len(),
        recent.len()
    )];
    out.extend(recent);
    Some(out)
}

/// inject.mjs `buildSessionPreamble(root, { sessionId, handoffOutcome })`.
/// Pure: reads state, never writes. Fail-open everywhere — orientation is
/// never a place to fail a session.
pub fn build_session_preamble(
    root: &Path,
    session_id: Option<&str>,
    handoff_outcome: Option<&HandoffOutcome>,
) -> String {
    let state = read_state(root);
    let onboarding = read_onboarding(root);
    let handoff = read_handoff(root);
    let pipeline = resolve_pipeline(root, session_id);
    let pipeline_record = if pipeline.ok { pipeline.record.clone() } else { state.clone() };
    // okf-integration-close-f4 D1/D2/D3: the ONE predicate, resolved once and
    // handed to every section that branches on it (G12). Fail-safe direction
    // is the legacy branch — orientation never fails a session.
    let bundle = bundle_mode(root);
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!("## bee v{BEE_VERSION}"));
    lines.push(onboarding_line(onboarding.as_ref()));
    lines.push(format!(
        "- Phase: {} | Mode: {} | Feature: {}",
        tpl(pipeline_record.get("phase")),
        tpl_or(pipeline_record.get("mode"), "none"),
        tpl_or(pipeline_record.get("feature"), "none"),
    ));
    lines.push(format!("- Gates: {}", gates_line(&pipeline_record)));
    if pipeline.ok && pipeline.source == "lane" {
        let bound = pipeline.feature.clone().map(Value::String);
        let others: Vec<JMap> = list_lanes(root)
            .into_iter()
            .filter(|lane| {
                !strict_eq(lane.get("feature"), bound.as_ref())
                    && !str_eq(lane.get("phase"), "idle")
                    && !str_eq(lane.get("phase"), "compounding-complete")
            })
            .collect();
        if !others.is_empty() {
            let names: Vec<Value> =
                others.iter().map(|l| l.get("feature").cloned().unwrap_or(Value::Null)).collect();
            lines.push(format!(
                "- {} other active lane(s): {}",
                others.len(),
                js_join(&names, ", ")
            ));
        }
    }
    let config = read_config_raw_open(root);
    for line in bypass_banner_lines(bypass_level(&config)) {
        lines.push(line);
    }
    // spec #81 P1 (sv-1): zero preamble cost when off — only 'draft-pr' adds
    // a line, matching bypassBannerLines' own "omit entirely when nothing to
    // report" convention just above.
    if ship_visibility(&config) == "draft-pr" {
        lines.push(
            "- Ship visibility: draft-pr — first cap opens a draft PR, every cap pushes (routing-and-contracts \"Ship visibility\")".to_string(),
        );
    }
    // explicit-triage D2: zero preamble cost when absent — only a recorded
    // route (bee state route --set) adds a line.
    if let Some(route) = pipeline_record.get("route").filter(|r| truthy(r)) {
        let flags = vget(route, "flags");
        let (flag_count, flag_list) = match flags {
            Some(Value::Array(items)) => (items.len().to_string(), js_join(items, ",")),
            // Node would throw on `.join` here; totality wins (see the module
            // banner's divergence list).
            _ => ("undefined".to_string(), String::new()),
        };
        lines.push(format!(
            "- Route: class={} | lane={} | flags={flag_count} [{flag_list}] | files={}",
            tpl(vget(route, "class")),
            tpl(vget(route, "lane")),
            tpl(vget(route, "product_files")),
        ));
    }
    if handoff_outcome.map(|o| o.ok).unwrap_or(false) {
        // fsh-10 (D1): adoption succeeded — start-now, no confirmation needed.
        // adoptHandoff already cleared .bee/HANDOFF.json, so `handoff` above
        // is already None by the time this renders — handoff_outcome is the
        // only surviving record of what happened.
        let outcome = handoff_outcome.unwrap();
        let next_cell_id = outcome.next_cell.clone().unwrap_or_else(|| "unknown".to_string());
        let next_cell = read_json_open(
            &root.join(".bee").join("cells").join(format!("{next_cell_id}.json")),
        );
        let title = next_cell.as_ref().and_then(|c| vget(c, "title")).filter(|v| truthy(v));
        lines.push(String::new());
        lines.push(
            "### PLANNED-NEXT ADOPTED — starting now, no confirmation needed (D1)".to_string(),
        );
        lines.push(match title {
            Some(t) => format!("- Cell: {next_cell_id} — {}", tpl(Some(t))),
            None => format!("- Cell: {next_cell_id}"),
        });
        lines.push(format!(
            "- Lane: {}",
            tpl_or(next_cell.as_ref().and_then(|c| vget(c, "lane")), "unknown")
        ));
        if let Some(verify) = next_cell.as_ref().and_then(|c| vget(c, "verify")).filter(|v| truthy(v))
        {
            lines.push(format!("- Verify: `{}`", tpl(Some(verify))));
        }
    } else if let Some(handoff) = handoff.as_ref() {
        // The leading blank is the CALLER's separator, never the block's own
        // first byte (see the blank-line ownership note above).
        lines.push(String::new());
        lines.extend(handoff_block_lines(handoff, handoff_outcome));
    }

    let commands = normalize_commands(config.get("commands"));
    let recorded_keys: Vec<&str> = COMMAND_KEYS
        .iter()
        .copied()
        .filter(|key| opt_truthy(commands.get(*key)))
        .collect();
    if !recorded_keys.is_empty() {
        lines.push(String::new());
        lines.push("### Standard commands (host project)".to_string());
        for key in &recorded_keys {
            lines.push(format!("- {key}: `{}`", tpl(commands.get(*key))));
        }
        if str_eq(commands.get("verify"), NO_TEST_SENTINEL) {
            // no-test-repos D1 (decision 55b951e1): the sentinel REPLACES the
            // CI-status-gate paragraph outright with one loud line — never a
            // silent drop of the gate.
            lines.push(format!(
                "- Test gates disabled by repo declaration (commands.verify: {NO_TEST_SENTINEL}) — cells cap on diff-backed outcomes; re-enable by recording real commands."
            ));
        } else if opt_truthy(commands.get("verify")) {
            lines.push(
                "- CI status gate: before your first `cells claim` of this session, check CI instead of running anything locally — the latest full-verify run on the base branch plus any open verify-red issue; red is surfaced and becomes its own fix-first tiny cell — never build on red. No local full-suite run is ever owed: the dev loop runs registry-scoped tests only, and the full suite is CI-owned. The claim is the trigger, not arrival: a session that claims no cell owes no CI check.".to_string(),
            );
            if str_eq(commands.get("test"), NO_TEST_SENTINEL) {
                lines.push(format!(
                    "- Dev-loop test command disabled by repo declaration (commands.test: {NO_TEST_SENTINEL}) — the CI-owned verify above still governs; re-enable by recording a real `test` command."
                ));
            }
        }
    }

    // okf-8 (D38): the startup bridge sits ahead of the project map.
    let knowledge = knowledge_context_lines(root, &pipeline_record);
    if !knowledge.is_empty() {
        lines.push(String::new());
        lines.extend(knowledge);
    }

    lines.push(String::new());
    lines.extend(project_map_lines(root, bundle));

    // D11: capture-mode spine. okf-integration-close-f4 D3: the nudge names
    // the RESOLVED target rather than hardcoding docs/specs/.
    let (debt_count, debt_cells) = scribing_debt(root);
    if debt_count > 0 {
        lines.push(String::new());
        lines.push(format!(
            "### Scribing debt: {debt_count} behavior_change cell(s) uncaptured"
        ));
        lines.push(format!(
            "- {} capped since the last scribing run — capture pending (decision c8e25271): run bee-capturing when you choose; settled behavior belongs in {}.",
            js_join(&debt_cells, ", "),
            if bundle { "docs/knowledge/" } else { "docs/specs/" }
        ));
    }

    // scribing-integrity si-1: the orphan sweep, one loud line.
    let (orphan_count, orphan_features) = global_scribing_debt(root);
    if orphan_count > 0 {
        lines.push(String::new());
        lines.push(format!(
            "### Orphaned scribing debt: {orphan_count} cell(s) across {} feature(s)",
            orphan_features.len()
        ));
        lines.push(format!(
            "- {} — capped with no scribing sync ever recorded for their feature; run bee-capturing for each, then `bee state scribing-run --feature <feature> --areas \"<a,b>\" --next-action \"<n>\"` to stamp the repair.",
            orphan_features
                .iter()
                .map(|(feature, cells)| format!("{feature} ({})", js_join(cells, ", ")))
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }

    // Decision 0017: capture stubs queued mid-flow, awaiting their flush pass.
    let queue_count = capture_queue_count(root);
    if queue_count > 0 {
        lines.push(String::new());
        lines.push(format!("### Capture queue: {queue_count} stub(s) pending flush"));
        lines.push(
            "- Settlements were stubbed mid-flow (decision 0017) — offer the flush now before new work: bee-capturing drains the queue oldest-first and merges each stub into its area spec.".to_string(),
        );
    }

    // P7: keep the ceiling model scarce.
    if let Some((pct, ceiling, tiered)) = ceiling_scarcity_warning(root) {
        lines.push(String::new());
        lines.push(format!(
            "### Ceiling-model scarcity: {}% of tiered cells on ceiling",
            num_str(pct)
        ));
        lines.push(format!(
            "- {ceiling}/{tiered} cells tiered ceiling (> {}%) — the cost lever erodes when the strongest model touches most dispatches; re-tier routine cells to generation/extraction (decision 0012).",
            num_str(js_round(CEILING_MAX_SHARE * 100.0))
        ));
    }

    if let Some(digest) = critical_patterns_digest(root, 10, bundle) {
        lines.push(String::new());
        lines.push("### Critical patterns (digest)".to_string());
        lines.extend(digest);
    }

    let decisions = active_decisions(root, Some(3));
    if !decisions.is_empty() {
        lines.push(String::new());
        lines.push("### Recent decisions".to_string());
        for event in &decisions {
            lines.push(format!(
                "- {} ({})",
                datamark(vget(event, "decision")),
                tpl(vget(event, "date"))
            ));
        }
    }

    lines.push(String::new());
    // CUTOVER wording divergence: inject.mjs:583 spelled the command
    // `node .bee/bin/bee.mjs status --json`. Everything else is unchanged.
    lines.push("Everything above is already read — do not re-fetch it. Run `.bee/bin/bee status --json` (and `decisions active`) yourself when you are about to ROUTE WORK — claim, plan, change phase — or need detail this block does not carry (agent-run — never hand bee commands to the user). Route via bee-hive.".to_string());
    lines.join("\n")
}

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, content: &str) {
        let file = root.join(rel);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }

    /// The smallest repo the preamble can render against: an onboarding
    /// marker and an idle state record, nothing else.
    fn minimal_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(tmp.path(), ".bee/state.json", r#"{"phase":"idle"}"#);
        tmp
    }

    fn render(root: &Path) -> String {
        build_session_preamble(root, None, None)
    }

    // ── (1) the cutover contract ──────────────────────────────────────────

    #[test]
    fn a_minimal_repo_renders_and_closes_on_the_binary_spelling() {
        let tmp = minimal_repo();
        let text = render(tmp.path());
        assert!(text.starts_with(&format!("## bee v{BEE_VERSION}\n")), "{text}");
        assert!(
            text.ends_with("Everything above is already read — do not re-fetch it. Run `.bee/bin/bee status --json` (and `decisions active`) yourself when you are about to ROUTE WORK — claim, plan, change phase — or need detail this block does not carry (agent-run — never hand bee commands to the user). Route via bee-hive."),
            "closing line drifted:\n{text}"
        );
    }

    #[test]
    fn no_mjs_spelling_survives_anywhere_in_the_preamble() {
        // Every section that could carry a command is turned ON at once, so
        // the sweep covers the knowledge-context line too — the other .mjs
        // spelling inject.mjs carried.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(
            root,
            ".bee/state.json",
            r#"{"phase":"swarming","feature":"f1","mode":"standard","route":{"class":"c","lane":"standard","flags":["a"],"product_files":3}}"#,
        );
        write(
            root,
            ".bee/config.json",
            r#"{"gate_bypass":"total","ship_visibility":"draft-pr","commands":{"setup":"s","start":"r","test":"t","verify":"v"}}"#,
        );
        write(root, ".bee/HANDOFF.json", r#"{"kind":"pause","phase":"swarming"}"#);
        write(
            root,
            "docs/knowledge/areas/okf-profile/a.md",
            "---\ntype: bee.area\ntitle: A\n---\nbody\n",
        );
        write(
            root,
            "docs/knowledge/work/f1/work-item.md",
            "---\ntype: bee.work-item\nbee:\n  id: f1\n---\nbody\n",
        );
        write(
            root,
            "docs/knowledge/index.md",
            "## Critical patterns\n- [p1](areas/x/p1.md)\n\n## Other\n",
        );
        write(root, ".bee/decisions.jsonl", "{\"type\":\"decide\",\"id\":\"d1\",\"decision\":\"keep it\",\"date\":\"2026-01-01\"}\n");
        write(root, ".bee/capture-queue.jsonl", "{\"kind\":\"stub\",\"id\":\"s1\",\"at\":\"2026-01-01T00:00:00.000Z\"}\n");
        let text = render(root);
        assert!(!text.contains(".mjs"), "an .mjs spelling survived:\n{text}");
        assert!(
            text.contains("- `.bee/bin/bee knowledge context --work f1 --budget 20000`"),
            "knowledge-context command missing or misspelled:\n{text}"
        );
    }

    // ── (2) every optional section: present when it should be, gone when not ─

    #[test]
    fn the_bypass_banner_is_omitted_when_off_and_two_lines_at_full() {
        let tmp = minimal_repo();
        assert!(!render(tmp.path()).contains("GATE BYPASS"));
        write(tmp.path(), ".bee/config.json", r#"{"gate_bypass":"full"}"#);
        let text = render(tmp.path());
        assert!(text.contains("⚡⚡ GATE BYPASS: FULL AUTOPILOT"), "{text}");
        assert!(text.contains("Only reading a secret-shaped file"), "{text}");
        assert_eq!(bypass_banner_lines("off").len(), 0);
        assert_eq!(bypass_banner_lines("").len(), 0);
        assert_eq!(bypass_banner_lines("normal").len(), 1);
        assert_eq!(bypass_banner_lines("full").len(), 2);
        assert_eq!(bypass_banner_lines("total").len(), 2);
    }

    #[test]
    fn ship_visibility_costs_nothing_until_draft_pr() {
        let tmp = minimal_repo();
        assert!(!render(tmp.path()).contains("Ship visibility"));
        write(tmp.path(), ".bee/config.json", r#"{"ship_visibility":"draft-pr"}"#);
        assert!(render(tmp.path()).contains("- Ship visibility: draft-pr — first cap opens a draft PR"));
    }

    #[test]
    fn the_route_line_appears_only_for_a_recorded_route() {
        let tmp = minimal_repo();
        assert!(!render(tmp.path()).contains("- Route:"));
        write(
            tmp.path(),
            ".bee/state.json",
            r#"{"phase":"planning","route":{"class":"feature","lane":"small","flags":["x","y"],"product_files":4}}"#,
        );
        assert!(render(tmp.path())
            .contains("- Route: class=feature | lane=small | flags=2 [x,y] | files=4"));
    }

    #[test]
    fn the_standard_commands_block_is_omitted_with_no_recorded_commands() {
        let tmp = minimal_repo();
        assert!(!render(tmp.path()).contains("### Standard commands"));
        write(tmp.path(), ".bee/config.json", r#"{"commands":{"verify":"npm test"}}"#);
        let text = render(tmp.path());
        assert!(text.contains("### Standard commands (host project)"), "{text}");
        assert!(text.contains("- verify: `npm test`"), "{text}");
        assert!(text.contains("- CI status gate:"), "{text}");
        // The sentinel REPLACES the CI paragraph with one loud line.
        write(tmp.path(), ".bee/config.json", r#"{"commands":{"verify":"none"}}"#);
        let text = render(tmp.path());
        assert!(text.contains("- Test gates disabled by repo declaration (commands.verify: none)"));
        assert!(!text.contains("- CI status gate:"));
    }

    #[test]
    fn the_knowledge_context_bridge_is_silent_with_no_active_work() {
        let tmp = minimal_repo();
        write(tmp.path(), "docs/knowledge/areas/a/c.md", "---\ntype: bee.area\n---\nx\n");
        // idle: nothing at all, even with a bundle present.
        assert!(!render(tmp.path()).contains("### Knowledge context"));
        // An active feature with NO work item gets exactly one offer line.
        write(tmp.path(), ".bee/state.json", r#"{"phase":"swarming","feature":"f9"}"#);
        let text = render(tmp.path());
        assert!(text.contains("- No knowledge work item for \"f9\" —"), "{text}");
        assert!(!text.contains("### Knowledge context"), "{text}");
        // With a work item it becomes the three-line pointer, budget by mode.
        write(
            tmp.path(),
            "docs/knowledge/work/f9/work-item.md",
            "---\ntype: bee.work-item\nbee:\n  id: f9\n---\nx\n",
        );
        write(tmp.path(), ".bee/state.json", r#"{"phase":"swarming","feature":"f9","mode":"tiny"}"#);
        let text = render(tmp.path());
        assert!(
            text.contains("- `.bee/bin/bee knowledge context --work f9 --budget 8000`"),
            "{text}"
        );
    }

    #[test]
    fn scribing_debt_capture_queue_and_scarcity_each_omit_when_empty() {
        let tmp = minimal_repo();
        let text = render(tmp.path());
        for absent in [
            "### Scribing debt:",
            "### Orphaned scribing debt:",
            "### Capture queue:",
            "### Ceiling-model scarcity:",
            "### Critical patterns (digest)",
            "### Recent decisions",
        ] {
            assert!(!text.contains(absent), "{absent} leaked into an empty repo:\n{text}");
        }

        let root = tmp.path();
        write(root, ".bee/state.json", r#"{"phase":"swarming","feature":"f1"}"#);
        write(
            root,
            ".bee/cells/c1.json",
            r#"{"id":"c1","feature":"f1","status":"capped","tier":"ceiling","trace":{"behavior_change":true,"capped_at":"2026-01-02T00:00:00.000Z"}}"#,
        );
        write(
            root,
            ".bee/cells/c2.json",
            r#"{"id":"c2","feature":"f1","status":"capped","tier":"ceiling","trace":{"behavior_change":true,"capped_at":"2026-01-02T00:00:00.000Z"}}"#,
        );
        write(
            root,
            ".bee/cells/c3.json",
            r#"{"id":"c3","feature":"f1","status":"open","tier":"extraction"}"#,
        );
        write(root, ".bee/capture-queue.jsonl", "{\"kind\":\"stub\",\"id\":\"s1\",\"at\":\"2026-01-01T00:00:00.000Z\"}\n");
        write(
            root,
            ".bee/decisions.jsonl",
            "{\"type\":\"decide\",\"id\":\"d1\",\"decision\":\"a\",\"date\":\"2026-01-01\"}\n",
        );
        write(root, "docs/history/learnings/critical-patterns.md", "<!-- note -->\n- pattern one\n");
        let text = render(root);
        assert!(text.contains("### Scribing debt: 2 behavior_change cell(s) uncaptured"), "{text}");
        assert!(text.contains("- c1, c2 capped since the last scribing run"), "{text}");
        assert!(text.contains("settled behavior belongs in docs/specs/."), "{text}");
        assert!(
            text.contains("### Orphaned scribing debt: 2 cell(s) across 1 feature(s)"),
            "{text}"
        );
        assert!(text.contains("- f1 (c1, c2) — capped with no scribing sync"), "{text}");
        assert!(text.contains("### Capture queue: 1 stub(s) pending flush"), "{text}");
        assert!(text.contains("### Ceiling-model scarcity: 67% of tiered cells on ceiling"), "{text}");
        assert!(text.contains("- 2/3 cells tiered ceiling (> 40%)"), "{text}");
        assert!(text.contains("### Critical patterns (digest)\n- pattern one"), "{text}");
        assert!(text.contains("### Recent decisions\n- «a» (2026-01-01)"), "{text}");
    }

    #[test]
    fn the_project_map_switches_on_the_one_bundle_predicate() {
        let tmp = minimal_repo();
        // No maps at all -> the missing-map warning.
        assert!(render(tmp.path()).contains("- Project map missing (Q1/Q2 unanswerable from repo)"));
        write(tmp.path(), "docs/specs/system-overview.md", "x\n");
        write(tmp.path(), "docs/specs/area-a.md", "x\n");
        let text = render(tmp.path());
        assert!(text.contains("- System overview: docs/specs/system-overview.md"), "{text}");
        assert!(text.contains("- Specced areas: 1 (docs/specs/ — read the spec before the code)"), "{text}");
        // A real bundle flips both the map and the scribing target.
        write(tmp.path(), "docs/knowledge/areas/a/c.md", "---\ntype: bee.area\n---\nx\n");
        let text = render(tmp.path());
        assert!(text.contains("- Knowledge bundle: docs/knowledge/"), "{text}");
        assert!(text.contains("- Bundle holds: 1 area(s), 1 concept(s)"), "{text}");
        assert!(!text.contains("- Specced areas:"), "{text}");
        // A directory with no parsing concept is NOT a bundle (G8).
        let bare = minimal_repo();
        std::fs::create_dir_all(bare.path().join("docs/knowledge")).unwrap();
        write(bare.path(), "docs/knowledge/.gitkeep", "");
        assert!(!bundle_mode(bare.path()), "an empty directory is not a bundle");
    }

    #[test]
    fn the_bundle_digest_counts_reverses_and_rewrites_its_links() {
        let tmp = minimal_repo();
        write(tmp.path(), "docs/knowledge/areas/a/c.md", "---\ntype: bee.pattern\n---\nx\n");
        let mut index = String::from("# Index\n\n## Critical patterns\n");
        for n in 1..=12 {
            index.push_str(&format!("- [p{n}](areas/a/p{n}.md)\n"));
        }
        index.push_str("- [ext](https://example.com/x)\n\n## Other\n- ignored\n");
        write(tmp.path(), "docs/knowledge/index.md", &index);
        let text = render(tmp.path());
        assert!(
            text.contains("- 13 critical pattern(s) in the bundle — the 9 most recent below"),
            "{text}"
        );
        // Newest-first, and bundle-relative links rewritten; absolute/http untouched.
        assert!(text.contains("- [ext](https://example.com/x)"), "{text}");
        assert!(text.contains("- [p12](docs/knowledge/areas/a/p12.md)"), "{text}");
        assert!(!text.contains("- [p4]"), "only the 9 most recent rows ride the digest:\n{text}");
    }

    // ── (3) fail-open on a corrupt store ──────────────────────────────────

    #[test]
    fn a_corrupt_state_file_still_renders_a_preamble() {
        let tmp = minimal_repo();
        write(tmp.path(), ".bee/state.json", "{ this is not json");
        let text = render(tmp.path());
        // defaultState() shows through, and the preamble is whole.
        assert!(text.contains("- Phase: idle | Mode: none | Feature: none"), "{text}");
        assert!(text.contains("- Gates: none pending (no active work)"), "{text}");
        assert!(text.ends_with("Route via bee-hive."), "{text}");
    }

    #[test]
    fn a_corrupt_config_handoff_and_cell_still_render_a_preamble() {
        let tmp = minimal_repo();
        write(tmp.path(), ".bee/config.json", "{nope");
        write(tmp.path(), ".bee/HANDOFF.json", "[[[");
        write(tmp.path(), ".bee/cells/c1.json", "}{");
        write(tmp.path(), ".bee/decisions.jsonl", "not json at all\n");
        let text = render(tmp.path());
        assert!(text.contains("## bee v"), "{text}");
        assert!(!text.contains("### HANDOFF present"), "a corrupt handoff reads as absent:\n{text}");
        assert!(text.ends_with("Route via bee-hive."), "{text}");
    }

    // ── (4) the handoff block's three arms ────────────────────────────────

    #[test]
    fn a_pause_handoff_renders_the_wait_block_after_a_blank_separator() {
        let tmp = minimal_repo();
        write(
            tmp.path(),
            ".bee/HANDOFF.json",
            r#"{"kind":"pause","phase":"swarming","feature":"f1","mode":"small","cells_in_flight":["c1","c2"],"next_action":"resume c1"}"#,
        );
        let text = render(tmp.path());
        assert!(
            text.contains("\n\n### HANDOFF present — present it and WAIT — never auto-resume\n"),
            "the caller owns the blank separator:\n{text}"
        );
        assert!(text.contains("- Phase: swarming | Feature: f1 | Mode: small"), "{text}");
        assert!(text.contains("- Cells in flight: c1, c2"), "{text}");
        assert!(text.contains("- Saved next action: resume c1"), "{text}");
        assert!(!text.contains("- Adoption not applied:"), "{text}");
        // A kindless record normalizes to pause, byte-identically.
        write(tmp.path(), ".bee/HANDOFF.json", r#"{"phase":"swarming"}"#);
        assert!(render(tmp.path()).contains("### HANDOFF present"));
        // And the block itself carries no leading blank of its own.
        let handoff = read_handoff(tmp.path()).unwrap();
        assert_eq!(
            handoff_block_lines(&handoff, None)[0],
            "### HANDOFF present — present it and WAIT — never auto-resume"
        );
    }

    #[test]
    fn an_adopted_planned_next_replaces_the_wait_block_with_start_now() {
        let tmp = minimal_repo();
        // adoptHandoff cleared HANDOFF.json already — the outcome is the only record.
        write(
            tmp.path(),
            ".bee/cells/c7.json",
            r#"{"id":"c7","title":"Wire the thing","lane":"small","verify":"npm test -- c7"}"#,
        );
        let outcome = HandoffOutcome {
            ok: true,
            next_cell: Some("c7".to_string()),
            ..Default::default()
        };
        let text = build_session_preamble(tmp.path(), None, Some(&outcome));
        assert!(
            text.contains(
                "\n\n### PLANNED-NEXT ADOPTED — starting now, no confirmation needed (D1)\n"
            ),
            "{text}"
        );
        assert!(text.contains("- Cell: c7 — Wire the thing"), "{text}");
        assert!(text.contains("- Lane: small"), "{text}");
        assert!(text.contains("- Verify: `npm test -- c7`"), "{text}");
        assert!(!text.contains("### HANDOFF present"), "{text}");

        // An unknown cell degrades to the unknown arms, never a failure.
        let outcome = HandoffOutcome { ok: true, ..Default::default() };
        let text = build_session_preamble(tmp.path(), None, Some(&outcome));
        assert!(text.contains("- Cell: unknown"), "{text}");
        assert!(text.contains("- Lane: unknown"), "{text}");
        assert!(!text.contains("- Verify:"), "{text}");
    }

    #[test]
    fn a_refused_adoption_waits_and_names_the_reason() {
        let tmp = minimal_repo();
        write(
            tmp.path(),
            ".bee/HANDOFF.json",
            r#"{"kind":"planned-next","phase":"swarming","feature":"f1","mode":"small"}"#,
        );
        let outcome = HandoffOutcome {
            ok: false,
            code: Some("WRONG_SOURCE".to_string()),
            reason: Some("resumed sessions never adopt".to_string()),
            next_cell: None,
        };
        let text = build_session_preamble(tmp.path(), None, Some(&outcome));
        assert!(text.contains("### HANDOFF present — present it and WAIT"), "{text}");
        assert!(text.contains("- Adoption not applied: resumed sessions never adopt"), "{text}");
        assert!(!text.contains("PLANNED-NEXT ADOPTED"), "{text}");

        // reason ?? code ?? 'unknown reason'.
        let handoff = read_handoff(tmp.path()).unwrap();
        let code_only = HandoffOutcome {
            ok: false,
            code: Some("WRONG_SOURCE".to_string()),
            ..Default::default()
        };
        assert!(handoff_block_lines(&handoff, Some(&code_only))
            .iter()
            .any(|l| l == "- Adoption not applied: WRONG_SOURCE"));
        let bare = HandoffOutcome { ok: false, ..Default::default() };
        assert!(handoff_block_lines(&handoff, Some(&bare))
            .iter()
            .any(|l| l == "- Adoption not applied: unknown reason"));
        // A PAUSE handoff never carries a refusal line, outcome or not.
        write(tmp.path(), ".bee/HANDOFF.json", r#"{"kind":"pause"}"#);
        let handoff = read_handoff(tmp.path()).unwrap();
        assert!(!handoff_block_lines(&handoff, Some(&code_only))
            .iter()
            .any(|l| l.starts_with("- Adoption not applied")));
    }

    // ── the shared renderers, on their own ────────────────────────────────

    #[test]
    fn onboarding_line_covers_all_three_arms() {
        assert_eq!(
            onboarding_line(None),
            "- Onboarding: MISSING — run bee-hive onboarding before anything else."
        );
        assert_eq!(
            onboarding_line(Some(&json!({"bee_version": "0.9.0"}))),
            format!("- Onboarding: installed at bee 0.9.0 but plugin is {BEE_VERSION} — re-run onboarding to refresh vendored helpers.")
        );
        assert_eq!(
            onboarding_line(Some(&json!({"bee_version": BEE_VERSION}))),
            format!("- Onboarding: ok (bee {BEE_VERSION})")
        );
        // A record with no version at all reads as ok at the plugin version.
        assert_eq!(
            onboarding_line(Some(&json!({}))),
            format!("- Onboarding: ok (bee {BEE_VERSION})")
        );
    }

    #[test]
    fn first_open_gate_skips_review_outside_a_review_session_and_terminal_records() {
        let rec = |phase: &str, gates: Value| -> JMap {
            let mut m = JMap::new();
            m.insert("phase".into(), json!(phase));
            m.insert("approved_gates".into(), gates);
            m
        };
        assert_eq!(first_open_gate(&rec("idle", json!({}))), None);
        assert_eq!(first_open_gate(&rec("compounding-complete", json!({}))), None);
        assert_eq!(first_open_gate(&rec("planning", json!({}))), Some("context"));
        assert_eq!(
            first_open_gate(&rec("planning", json!({"context": true, "shape": true, "execution": true}))),
            None,
            "review is on-demand — never pending outside a review session"
        );
        assert_eq!(
            first_open_gate(&rec("reviewing", json!({"context": true, "shape": true, "execution": true}))),
            Some("review")
        );
        // gatesLine follows the same rule.
        assert_eq!(gates_line(&rec("idle", json!({}))), "none pending (no active work)");
        assert_eq!(
            gates_line(&rec("planning", json!({"context": true}))),
            "context: approved | shape: pending | execution: pending"
        );
    }

    #[test]
    fn a_lane_bound_session_reports_the_other_active_lanes() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(root, ".bee/sessions/s1.json", r#"{"id":"s1","lane":"f1"}"#);
        write(root, ".bee/lanes/f1.json", r#"{"feature":"f1","phase":"swarming","mode":"small"}"#);
        write(root, ".bee/lanes/f2.json", r#"{"feature":"f2","phase":"planning"}"#);
        write(root, ".bee/lanes/f3.json", r#"{"feature":"f3","phase":"idle"}"#);
        let text = build_session_preamble(root, Some("s1"), None);
        assert!(text.contains("- Phase: swarming | Mode: small | Feature: f1"), "{text}");
        assert!(text.contains("- 1 other active lane(s): f2"), "{text}");
        // An unresolvable binding falls back to the DEFAULT record, silently.
        write(root, ".bee/sessions/s2.json", r#"{"id":"s2","lane":"nope"}"#);
        let text = build_session_preamble(root, Some("s2"), None);
        assert!(text.contains("- Phase: idle | Mode: none | Feature: none"), "{text}");
        assert!(!text.contains("other active lane(s)"), "{text}");
    }
}
