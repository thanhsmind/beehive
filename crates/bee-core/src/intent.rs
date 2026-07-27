//! intent — the INTENT ANCHOR store (rpl-2, CONTEXT.md D3/D7).
//!
//! A line-for-line port of `packages/bee/lib/intent.mjs`, which is FROZEN
//! (D1) and is therefore the oracle, never something to improve on. The
//! three disciplines that module's own header names are the three this port
//! has to keep observable:
//!
//! * **SMALL** — a fixed field set. [`Anchor`]'s declaration order IS the
//!   serialization order, and it is the mjs object-literal order verbatim
//!   (`intent.mjs:123-137,203-215`), because `serde_json` is built here with
//!   `preserve_order` and `JSON.stringify` emits insertion order.
//! * **VERBATIM** — `request` is the user's own bytes. Nothing here trims,
//!   re-wraps or truncates it, and [`advance_intent`]'s signature has no
//!   parameter that could carry a new request or acceptance into the record
//!   (the D1 invariant is structural in mjs; it stays structural here).
//! * **FAIL-OPEN** — every reader returns `None` rather than erroring. A
//!   missing or unusable anchor is silence.
//!
//! ## Storage contract (D3: zero migration, mjs and Rust interleave)
//!
//! ONE JSON FILE PER SANITIZED KEY at `.bee/intent/<key>.json` — never a
//! jsonl journal (`intent.mjs:52,69-70`). Writes go through
//! [`crate::fsutil::write_json_atomic`], which is the port of
//! `writeJsonAtomic`: read-modify-write with **NO LOCK**, because
//! `intent.mjs:216,238` takes none. Introducing one here would be a
//! divergence in concurrent behavior that no output diff could ever see.
//!
//! ## The `.slice(0, 120)` question, measured rather than assumed
//!
//! `sanitizeIntentKey` ends in `.slice(0, 120)` and JS `slice` counts UTF-16
//! CODE UNITS — not bytes, not scalar values. Rust's `&s[..120]` (bytes) and
//! `.chars().take(120)` (scalar values) each disagree with it on non-ASCII,
//! and the JS form can split a surrogate pair mid-character.
//!
//! What makes all three agree here is the step BEFORE the slice:
//! `.replace(/[^A-Za-z0-9._-]+/g, '-')` rewrites every run of characters
//! outside `[A-Za-z0-9._-]` to a single ASCII `-`, so the string reaching
//! `.slice` is **provably pure ASCII** and code units, bytes and scalar
//! values are the same count. That is a measured claim, not a read one: see
//! [`tests::sanitize_output_is_always_ascii`] here and, on the mjs side, the
//! `intent/key-*` `--cmd-check` scenarios, which drive an astral-plane key,
//! a key longer than 120 code units, and a key whose 120th code-unit
//! boundary falls inside a surrogate pair through BOTH runtimes.
//!
//! [`js_slice_code_units`] is still written on code units rather than on
//! chars, so the property above is what makes it safe rather than what it
//! silently assumes.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::fsutil::{js_trim, read_json, write_json_atomic};
use crate::state::read_state;

/// `intent.mjs:35` `INTENT_SCHEMA_VERSION`.
pub const INTENT_SCHEMA_VERSION: &str = "1.0";

/// `intent.mjs:43` `DEFAULT_INTENT_KEY` — the last-resort key for work with
/// neither an active feature nor a session id (D2).
pub const DEFAULT_INTENT_KEY: &str = "default";

/// `intent.mjs:49` `NO_WORK_PHASES`. The PHASE — never the `feature` string,
/// which outlives both of these — decides whether a feature is "active".
pub const NO_WORK_PHASES: &[&str] = &["idle", "compounding-complete"];

/// `intent.mjs:51` `intentDir`.
pub fn intent_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("intent")
}

/// `intent.mjs:69` `intentPath`.
pub fn intent_path(root: &Path, key: &str) -> PathBuf {
    intent_dir(root).join(format!("{}.json", sanitize_intent_key(key)))
}

/// The `[A-Za-z0-9._-]` character class of `intent.mjs:62`'s NEGATED regex.
fn is_key_safe(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-'
}

/// JS `String.prototype.slice(0, limit)` over UTF-16 CODE UNITS.
///
/// The one shape where this cannot be exact is a cut that lands between the
/// two code units of a surrogate pair: JS keeps the lone high surrogate,
/// which is not a Rust `char` at all. This function drops the pair whole
/// instead. That branch is UNREACHABLE from [`sanitize_intent_key`] — the
/// preceding `[^A-Za-z0-9._-]+` collapse leaves an ASCII-only string, where
/// every char is exactly one code unit — and that unreachability is proven
/// by [`tests::sanitize_output_is_always_ascii`] rather than asserted.
fn js_slice_code_units(s: &str, limit: usize) -> String {
    let mut out = String::with_capacity(s.len().min(limit));
    let mut units = 0usize;
    for ch in s.chars() {
        let width = ch.len_utf16();
        if units + width > limit {
            break;
        }
        out.push(ch);
        units += width;
    }
    out
}

/// `intent.mjs:58` `sanitizeIntentKey`. Keys become filenames, so they are
/// constrained here rather than trusted. NEVER fails: an unusable key
/// degrades to [`DEFAULT_INTENT_KEY`] instead of failing a write the user
/// cannot recover.
///
/// The four steps, in the mjs source's exact order — the ORDER is
/// load-bearing, not incidental. `/-+$/` runs BEFORE `.slice(0, 120)`, so a
/// key that is truncated to exactly 120 characters may legitimately END in
/// `-`: the trailing-dash strip already ran, against the untruncated string.
/// A port that reordered these two would silently emit a 119-character key
/// for that input.
pub fn sanitize_intent_key(key: &str) -> String {
    let raw = js_trim(key);
    if raw.is_empty() {
        return DEFAULT_INTENT_KEY.to_string();
    }

    // 1. /[^A-Za-z0-9._-]+/g -> '-'  (each RUN collapses to ONE dash)
    let mut collapsed = String::with_capacity(raw.len());
    let mut in_run = false;
    for ch in raw.chars() {
        if is_key_safe(ch) {
            collapsed.push(ch);
            in_run = false;
        } else if !in_run {
            collapsed.push('-');
            in_run = true;
        }
    }

    // 2. /^[-.]+/ -> ''
    let lead = collapsed
        .char_indices()
        .find(|(_, c)| *c != '-' && *c != '.')
        .map(|(i, _)| i)
        .unwrap_or(collapsed.len());
    let trimmed_lead = &collapsed[lead..];

    // 3. /-+$/ -> ''
    let tail = trimmed_lead
        .char_indices()
        .rev()
        .find(|(_, c)| *c != '-')
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    let trimmed_tail = &trimmed_lead[..tail];

    // 4. .slice(0, 120) — UTF-16 code units (see the module doc)
    let safe = js_slice_code_units(trimmed_tail, 120);
    if safe.is_empty() {
        DEFAULT_INTENT_KEY.to_string()
    } else {
        safe
    }
}

// ─── the stored record ─────────────────────────────────────────────────────

/// The normalized anchor. FIELD ORDER IS SERIALIZATION ORDER and mirrors
/// `normalizeAnchor` (`intent.mjs:123-137`) / the `writeIntent` literal
/// (`:203-215`) exactly.
///
/// `advanced_at` is the only conditional key: mjs spreads it in only when the
/// source record carries a string (`intent.mjs:136`), and appends it last in
/// `advanceIntent` (`:233-237`) — either way it lands at the end, which is
/// where `skip_serializing_if` puts it here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Anchor {
    pub schema_version: String,
    pub key: String,
    pub written_at: Option<String>,
    /// VERBATIM — never trimmed, never truncated, never re-wrapped.
    pub request: String,
    pub acceptance: String,
    pub next_action: Option<String>,
    pub feature: Option<String>,
    pub lane: Option<String>,
    pub cell: Option<String>,
    pub do_not_reverse: Vec<String>,
    pub stop_conditions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advanced_at: Option<String>,
}

/// `optionalString` (`intent.mjs:113`).
fn optional_string(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(s)) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
        _ => None,
    }
}

/// JS `String(value)` for the element coercion inside `normalizeList`.
/// Reachable only from a hand-mangled store (the CLI always supplies a
/// comma-joined string), but a port that silently dropped a non-string entry
/// would diverge on exactly that store.
fn js_string_of(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Number(n) => {
            // JS prints an integral double without a fractional part.
            if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 && f.is_finite() && f.abs() < 1e21 {
                    return format!("{}", f as i64);
                }
            }
            n.to_string()
        }
        // `Array.prototype.toString` == `join(',')`, with null/undefined
        // rendering as the empty string.
        Value::Array(items) => items
            .iter()
            .map(|v| if v.is_null() { String::new() } else { js_string_of(v) })
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".to_string(),
    }
}

/// `normalizeList` (`intent.mjs:102`).
fn normalize_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .map(|v| js_trim(&js_string_of(v)).to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        Some(Value::String(s)) if !js_trim(s).is_empty() => s
            .split(',')
            .map(|part| js_trim(part).to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// `normalizeAnchor` (`intent.mjs:120`). A record missing the one field that
/// matters is NOT an anchor: corrupt, half-written or hand-mangled files read
/// as absent (D5) rather than as a half-anchor that could hand a summarizer a
/// truncated objective.
pub fn normalize_anchor(raw: &Value, key: &str) -> Option<Anchor> {
    let obj = raw.as_object()?; // `!raw || typeof raw !== 'object' || Array.isArray(raw)`
    let request = match obj.get("request") {
        Some(Value::String(s)) if !js_trim(s).is_empty() => s.clone(),
        _ => return None,
    };
    Some(Anchor {
        schema_version: match obj.get("schema_version") {
            Some(Value::String(s)) => s.clone(),
            _ => INTENT_SCHEMA_VERSION.to_string(),
        },
        // mjs: `typeof raw.key === 'string' && raw.key` — truthy, so the
        // empty string falls back to the lookup key.
        key: match obj.get("key") {
            Some(Value::String(s)) if !s.is_empty() => s.clone(),
            _ => key.to_string(),
        },
        written_at: match obj.get("written_at") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        },
        request,
        acceptance: match obj.get("acceptance") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        },
        next_action: optional_string(obj.get("next_action")),
        feature: optional_string(obj.get("feature")),
        lane: optional_string(obj.get("lane")),
        cell: optional_string(obj.get("cell")),
        do_not_reverse: normalize_list(obj.get("do_not_reverse")),
        stop_conditions: normalize_list(obj.get("stop_conditions")),
        advanced_at: match obj.get("advanced_at") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        },
    })
}

// ─── key resolution ────────────────────────────────────────────────────────

/// The lookup options `intentKeyCandidates` takes (`intent.mjs:91`).
#[derive(Debug, Clone, Default)]
pub struct Lookup {
    pub session_id: Option<String>,
    pub feature: Option<String>,
    pub key: Option<String>,
}

/// `activeFeature` (`intent.mjs:74`). Fail-open: any read problem => `None`.
pub fn active_feature(root: &Path) -> Option<String> {
    let state = read_state(root);
    if NO_WORK_PHASES.contains(&state.phase.as_str()) {
        return None;
    }
    match state.feature {
        Some(f) if !js_trim(&f).is_empty() => Some(js_trim(&f).to_string()),
        _ => None,
    }
}

/// `intentKeyCandidates` (`intent.mjs:91`), in priority order: an explicit
/// feature, the active feature, the session id, then the shared default.
/// [`write_intent`] lands on `candidates[0]`; every reader walks the whole
/// list, so an anchor written by the CLI (no session id) is still found by a
/// hook (session id present), and one written under a feature is still found
/// after that feature closes.
pub fn intent_key_candidates(root: &Path, options: &Lookup) -> Vec<String> {
    if let Some(key) = &options.key {
        if !js_trim(key).is_empty() {
            return vec![sanitize_intent_key(key)];
        }
    }
    let mut candidates: Vec<String> = Vec::new();
    let explicit = options
        .feature
        .as_deref()
        .map(js_trim)
        .filter(|f| !f.is_empty())
        .map(str::to_string);
    let resolved = explicit.or_else(|| active_feature(root));
    if let Some(r) = resolved {
        candidates.push(sanitize_intent_key(&r));
    }
    if let Some(session) = options.session_id.as_deref() {
        if !js_trim(session).is_empty() {
            candidates.push(sanitize_intent_key(session));
        }
    }
    candidates.push(DEFAULT_INTENT_KEY.to_string());
    // `[...new Set(candidates)]` — first occurrence wins, order preserved.
    let mut seen: Vec<String> = Vec::with_capacity(candidates.len());
    for c in candidates {
        if !seen.contains(&c) {
            seen.push(c);
        }
    }
    seen
}

fn read_anchor_at(root: &Path, key: &str) -> Option<Anchor> {
    let raw: Value = read_json(&intent_path(root, key), Value::Null);
    normalize_anchor(&raw, key)
}

/// `readIntent` (`intent.mjs:146`). Returns `None` when nothing is stored,
/// when the file is unreadable, and when its content is not a usable anchor.
/// NEVER fails — this is called from hooks whose whole contract is fail-open.
pub fn read_intent(root: &Path, options: &Lookup) -> Option<Anchor> {
    for key in intent_key_candidates(root, options) {
        if let Some(anchor) = read_anchor_at(root, &key) {
            return Some(anchor);
        }
    }
    None
}

/// `locateIntentKey` (`intent.mjs:159`) — which key currently HOLDS an
/// anchor, `None` when none does.
pub fn locate_intent_key(root: &Path, options: &Lookup) -> Option<String> {
    for key in intent_key_candidates(root, options) {
        if read_anchor_at(root, &key).is_some() {
            return Some(key);
        }
    }
    None
}

// ─── writers ───────────────────────────────────────────────────────────────

/// The `fields` half of `writeIntent`'s signature (`intent.mjs:179`). Every
/// member is the flag's own string, already `String(...)`-coerced by the CLI
/// and NOT otherwise touched — `request` in particular reaches here verbatim.
#[derive(Debug, Clone, Default)]
pub struct IntentFields {
    pub request: Option<String>,
    pub acceptance: Option<String>,
    pub next_action: Option<String>,
    pub feature: Option<String>,
    pub lane: Option<String>,
    pub cell: Option<String>,
    pub do_not_reverse: Option<String>,
    pub stop_conditions: Option<String>,
}

fn opt_str_field(value: &Option<String>) -> Option<String> {
    optional_string(value.as_ref().map(|s| Value::String(s.clone())).as_ref())
}

fn list_field(value: &Option<String>) -> Vec<String> {
    normalize_list(value.as_ref().map(|s| Value::String(s.clone())).as_ref())
}

/// `writeIntent` (`intent.mjs:179`). `request` is stored EXACTLY as given.
///
/// Immutability (D1): request and acceptance are immutable once set. Writing
/// a DIFFERENT request/acceptance over a live anchor errors unless `force` is
/// passed — an objective is replaced deliberately (a new task), never drifted
/// into. Re-writing the SAME request is idempotent and always allowed, so a
/// re-run of "anchor this task" is never a failure.
///
/// The `Err` payload is the mjs `Error.message` verbatim: `bee.mjs:7200`
/// routes a thrown handler error straight into `emitError`, so this string is
/// user-visible output and is diffed byte-for-byte by the parity harness.
pub fn write_intent(
    root: &Path,
    fields: &IntentFields,
    options: &Lookup,
    force: bool,
    now_iso: &str,
) -> Result<Anchor, String> {
    let request = match fields.request.as_deref() {
        Some(r) if !js_trim(r).is_empty() => r.to_string(),
        _ => {
            return Err(
                "writeIntent: `request` is required and must be the user's VERBATIM words.".to_string()
            )
        }
    };
    let acceptance = match fields.acceptance.as_deref() {
        Some(a) if !js_trim(a).is_empty() => a.to_string(),
        _ => {
            return Err(
                "writeIntent: `acceptance` is required — an anchor with no \"done means\" cannot detect drift."
                    .to_string(),
            )
        }
    };

    let key = intent_key_candidates(root, options)
        .into_iter()
        .next()
        .unwrap_or_else(|| DEFAULT_INTENT_KEY.to_string());
    let existing = read_anchor_at(root, &key);
    if let Some(existing) = &existing {
        if !force {
            if existing.request != request {
                return Err(format!(
                    "writeIntent: an anchor already exists at \"{key}\" with a different request — request is immutable once set (D1). Advance it (`bee intent advance`), clear it (`bee intent clear`), or pass --force to replace the objective deliberately."
                ));
            }
            if existing.acceptance != acceptance {
                return Err(format!(
                    "writeIntent: an anchor already exists at \"{key}\" with different acceptance criteria — acceptance is immutable once set (D1). Clear it (`bee intent clear`) or pass --force to replace the objective deliberately."
                ));
            }
        }
    }

    let anchor = Anchor {
        schema_version: INTENT_SCHEMA_VERSION.to_string(),
        key: key.clone(),
        written_at: Some(now_iso.to_string()),
        request,
        acceptance,
        next_action: opt_str_field(&fields.next_action),
        feature: opt_str_field(&fields.feature).or_else(|| active_feature(root)),
        lane: opt_str_field(&fields.lane),
        cell: opt_str_field(&fields.cell),
        do_not_reverse: list_field(&fields.do_not_reverse),
        stop_conditions: list_field(&fields.stop_conditions),
        advanced_at: None,
    };
    // NO LOCK — `intent.mjs:216` takes none, and introducing one here would
    // be a concurrency divergence no output diff could ever see.
    write_json_atomic(&intent_path(root, &key), &anchor)
        .map_err(|e| format!("writeIntent: {} — {e}", intent_path(root, &key).display()))?;
    Ok(anchor)
}

/// `advanceIntent` (`intent.mjs:228`). A segment finishes; the through-line
/// (request + acceptance) is untouched and only `next_action` moves.
///
/// The signature makes the immutability STRUCTURAL, exactly as in mjs: there
/// is no parameter that could carry a new request or acceptance into the
/// stored record. Keep it that way.
pub fn advance_intent(
    root: &Path,
    next_action: Option<&str>,
    options: &Lookup,
    now_iso: &str,
) -> Option<Anchor> {
    let key = locate_intent_key(root, options)?;
    let anchor = read_anchor_at(root, &key)?;
    let advanced = Anchor {
        next_action: optional_string(next_action.map(|s| Value::String(s.to_string())).as_ref()),
        advanced_at: Some(now_iso.to_string()),
        ..anchor
    };
    // mjs ignores the write's own failure mode here too (it would throw and
    // surface through emitError); a failed write leaves the record untouched.
    if write_json_atomic(&intent_path(root, &key), &advanced).is_err() {
        return None;
    }
    Some(advanced)
}

/// `clearIntent`'s `{cleared, key}` return (`intent.mjs:243`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClearRecord {
    pub cleared: bool,
    pub key: String,
}

/// `clearIntent` (`intent.mjs:243`) — remove the anchor. Never fails.
pub fn clear_intent(root: &Path, options: &Lookup) -> ClearRecord {
    let key = locate_intent_key(root, options).unwrap_or_else(|| {
        intent_key_candidates(root, options)
            .into_iter()
            .next()
            .unwrap_or_else(|| DEFAULT_INTENT_KEY.to_string())
    });
    let file = intent_path(root, &key);
    if !file.exists() {
        return ClearRecord { cleared: false, key };
    }
    match fs::remove_file(&file) {
        Ok(()) => ClearRecord { cleared: true, key },
        Err(_) => ClearRecord { cleared: false, key },
    }
}

// ─── renderers (intent.mjs:255-324) ────────────────────────────────────────
//
// Both take an anchor OBJECT (never a key) so they stay pure and testable,
// and both return '' for a null anchor — the silence D5 requires. Neither
// ever reflows `request`: it is emitted on its own line(s), byte for byte,
// under a label.

pub const INTENT_PRECOMPACT_HEADER: &str =
    "=== BEE INTENT ANCHOR — VERBATIM · DO NOT SUMMARIZE · DO NOT PARAPHRASE ===";
pub const INTENT_PRECOMPACT_FOOTER: &str = "=== END BEE INTENT ANCHOR ===";
pub const INTENT_RESUME_HEADER: &str =
    "## INTENT ANCHOR — read this FIRST (the objective; bee workflow state follows below)";

/// `contextLines` (`intent.mjs:262`).
fn context_lines(anchor: &Anchor) -> Vec<String> {
    let mut lines = Vec::new();
    if !anchor.do_not_reverse.is_empty() {
        lines.push(format!("DO NOT REVERSE: {}", anchor.do_not_reverse.join(" | ")));
    }
    if !anchor.stop_conditions.is_empty() {
        lines.push(format!("STOP IF: {}", anchor.stop_conditions.join(" | ")));
    }
    let mut where_parts: Vec<String> = Vec::new();
    if let Some(f) = &anchor.feature {
        where_parts.push(format!("feature={f}"));
    }
    if let Some(l) = &anchor.lane {
        where_parts.push(format!("lane={l}"));
    }
    if let Some(c) = &anchor.cell {
        where_parts.push(format!("cell={c}"));
    }
    if !where_parts.is_empty() {
        lines.push(format!("CONTEXT: {}", where_parts.join(" ")));
    }
    lines
}

/// `precompactBlock` (`intent.mjs:292`) — D3, what PreCompact pushes into the
/// preserved context. The label top and bottom is the mechanism that makes
/// the block survive a summary; it is not decoration.
pub fn precompact_block(anchor: Option<&Anchor>) -> String {
    let Some(anchor) = anchor else { return String::new() };
    let mut lines = vec![
        INTENT_PRECOMPACT_HEADER.to_string(),
        "This block is the OBJECTIVE and outranks every phase/gate/workflow detail in this".to_string(),
        "context. Carry it through the compaction unchanged, word for word.".to_string(),
        "ORIGINAL REQUEST (verbatim):".to_string(),
        anchor.request.clone(),
        format!("DONE MEANS: {}", anchor.acceptance),
    ];
    if let Some(next) = &anchor.next_action {
        lines.push(format!("NEXT ACTION: {next}"));
    }
    lines.extend(context_lines(anchor));
    lines.push(INTENT_PRECOMPACT_FOOTER.to_string());
    lines.join("\n")
}

/// `resumeBlock` (`intent.mjs:313`) — D4, what a compact/resume session start
/// LEADS with. The ordering is the whole fix.
pub fn resume_block(anchor: Option<&Anchor>) -> String {
    let Some(anchor) = anchor else { return String::new() };
    let mut lines = vec![
        INTENT_RESUME_HEADER.to_string(),
        "ORIGINAL REQUEST (verbatim):".to_string(),
        anchor.request.clone(),
        format!("DONE MEANS: {}", anchor.acceptance),
    ];
    if let Some(next) = &anchor.next_action {
        lines.push(format!("NEXT ACTION: {next}"));
    }
    lines.extend(context_lines(anchor));
    lines.push("Everything below is workflow state — it serves the request above, it never replaces it.".to_string());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// THE load-bearing property: the string that reaches `.slice(0, 120)` is
    /// pure ASCII, which is what makes UTF-16 code units, bytes and scalar
    /// values the same count — and therefore what makes the named
    /// `.slice`-on-code-units trap unreachable here rather than merely
    /// unlikely. Every expected value below was MEASURED against the frozen
    /// `sanitizeIntentKey` oracle before it was written down.
    #[test]
    fn sanitize_output_is_always_ascii() {
        let smile = char::from_u32(0x1f600).unwrap(); // astral: 2 UTF-16 units
        let bee = char::from_u32(0x1f41d).unwrap();
        let cases: Vec<String> = vec![
            "../../etc/passwd".to_string(),
            "  ".to_string(),
            "feature/with spaces".to_string(),
            "x".repeat(400),
            "意図アンカー".to_string(),
            format!("{bee}{bee}{bee}-intent"),
            // the 120th code-unit boundary falls INSIDE the surrogate pair
            format!("{}{}{}", "a".repeat(119), smile, "b".repeat(10)),
            format!("{}{}{}", "a".repeat(60), smile.to_string().repeat(60), "z".repeat(60)),
        ];
        for case in &cases {
            let out = sanitize_intent_key(case);
            assert!(out.is_ascii(), "sanitize({case:?}) produced non-ASCII {out:?}");
            assert!(out.chars().count() <= 120, "sanitize({case:?}) exceeded 120: {out:?}");
            assert!(!out.is_empty(), "sanitize never returns empty");
        }
    }

    #[test]
    fn sanitize_matches_the_measured_mjs_oracle() {
        let smile = char::from_u32(0x1f600).unwrap();
        let bee = char::from_u32(0x1f41d).unwrap();
        // Left column drives sanitizeIntentKey; right column is what the
        // frozen mjs oracle printed for it.
        let table: Vec<(String, String)> = vec![
            ("../../etc/passwd".into(), "etc-passwd".into()),
            ("  ".into(), "default".into()),
            ("feature/with spaces".into(), "feature-with-spaces".into()),
            ("...--foo".into(), "foo".into()),
            ("foo---".into(), "foo".into()),
            ("a\\b\\..\\c".into(), "a-b-..-c".into()),
            (format!("{bee}{bee}{bee}"), "default".into()),
            ("意図アンカー".into(), "default".into()),
            (format!("{bee}{bee}{bee}-intent"), "intent".into()),
            ("x".repeat(400), "x".repeat(120)),
            // /-+$/ runs BEFORE .slice(0,120), so this one legitimately ends
            // in a dash at exactly 120 characters.
            (
                format!("{}{}{}", "a".repeat(119), smile, "b".repeat(10)),
                format!("{}-", "a".repeat(119)),
            ),
            (
                format!("{}{}{}", "a".repeat(60), smile.to_string().repeat(60), "z".repeat(60)),
                format!("{}-{}", "a".repeat(60), "z".repeat(59)),
            ),
        ];
        for (input, expected) in table {
            assert_eq!(sanitize_intent_key(&input), expected, "input {input:?}");
        }
    }

    #[test]
    fn normalize_anchor_reads_unusable_records_as_absent() {
        for raw in [
            json!({}),
            json!({ "request": 42 }),
            json!([]),
            json!({ "request": "   " }),
            json!("a string"),
            Value::Null,
        ] {
            assert!(normalize_anchor(&raw, "default").is_none(), "{raw} must read as absent");
        }
    }

    #[test]
    fn normalize_anchor_defaults_match_the_mjs_shape() {
        let a = normalize_anchor(&json!({ "request": "do the thing" }), "k").unwrap();
        assert_eq!(a.schema_version, "1.0");
        assert_eq!(a.key, "k");
        assert_eq!(a.written_at, None);
        assert_eq!(a.acceptance, ""); // empty STRING, not null
        assert_eq!(a.do_not_reverse, Vec::<String>::new());
        assert_eq!(a.advanced_at, None);
    }

    #[test]
    fn normalize_list_matches_both_mjs_input_shapes() {
        assert_eq!(normalize_list(Some(&json!(" a , b ,, c "))), vec!["a", "b", "c"]);
        assert_eq!(normalize_list(Some(&json!([" a ", "", 42, true]))), vec!["a", "42", "true"]);
        assert_eq!(normalize_list(Some(&json!(42))), Vec::<String>::new());
        assert_eq!(normalize_list(None), Vec::<String>::new());
    }

    #[test]
    fn serialized_key_order_is_the_mjs_literal_order() {
        let a = normalize_anchor(&json!({ "request": "r", "advanced_at": "t" }), "k").unwrap();
        let text = serde_json::to_string(&a).unwrap();
        let order: Vec<&str> = [
            "schema_version",
            "key",
            "written_at",
            "request",
            "acceptance",
            "next_action",
            "feature",
            "lane",
            "cell",
            "do_not_reverse",
            "stop_conditions",
            "advanced_at",
        ]
        .to_vec();
        let mut cursor = 0usize;
        for field in order {
            let needle = format!("\"{field}\":");
            let at = text[cursor..].find(&needle).map(|i| i + cursor);
            assert!(at.is_some(), "{field} missing or out of order in {text}");
            cursor = at.unwrap();
        }
        // absent advanced_at is OMITTED, never emitted as null
        let b = normalize_anchor(&json!({ "request": "r" }), "k").unwrap();
        assert!(!serde_json::to_string(&b).unwrap().contains("advanced_at"));
    }

    #[test]
    fn no_work_phases_are_exactly_idle_and_compounding_complete() {
        assert_eq!(NO_WORK_PHASES, ["idle", "compounding-complete"]);
    }
}
