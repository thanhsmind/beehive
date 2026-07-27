//! Declared volatile allowlist (CONTEXT.md D7a, validation decision W7 +
//! advisor note 3): the ONLY normalization the parity differ is allowed to
//! apply before comparing two legs' output. Everything else is compared
//! byte-for-byte (well, char-for-char after UTF-8 decode) — this is
//! deliberately NOT a blanket timestamp/whitespace normalizer.
//!
//! # rpl-1: masking without a shape assertion is itself a red
//!
//! Every write verb in the ledger groups stamps wall-clock and randomness
//! with no injection seam — `crypto.randomUUID()` / `new Date()
//! .toISOString()` at `packages/bee/lib/decisions.mjs:320-322`,
//! `lib/reviews.mjs:46,348`, `lib/backlog.mjs:284` — so a raw byte-diff can
//! never match on a write verb, and some masking is unavoidable. The danger
//! is that a mask is a licence to differ: once `"date"` is scrubbed to
//! `<TS>`, a Rust port emitting `2026-07-26T00:00:00Z` (chrono's
//! second-precision RFC3339, a real and easy mistake) diffs clean against
//! mjs's `2026-07-26T00:00:00.000Z`. The mask would have hidden a genuine
//! byte-compatibility break.
//!
//! Two rules close that, and both are load-bearing for every later cell:
//!
//! 1. **KEY-GATED, deny-by-default.** A value is masked only when its JSON
//!    key is NAMED in [`VOLATILE_FIELDS`]. There is no pattern-based
//!    scrubber that eats any timestamp-shaped run wherever it appears. A new
//!    volatile field nobody declared is therefore compared literally and
//!    fails loudly, instead of being silently absorbed.
//! 2. **SHAPE-GATED, asserted on both sides.** A declared key's value is
//!    masked only when it MATCHES the declared [`Shape`]. A value that does
//!    not match is left exactly as it is — so if one leg emits the mjs shape
//!    and the other does not, the two normalized texts still differ and the
//!    diff fires. The shape is asserted on both sides precisely because
//!    neither side is ever masked "by position": each is masked on its own
//!    merits, and disagreement survives.
//!
//! Allowlist members:
//! - each leg's own temp-root absolute path, rewritten to the literal
//!   `<ROOT>` (root-path rewriting is part of the declared allowlist, not
//!   blanket normalization — CONTEXT.md D7a diff policy);
//! - the JSON keys named in [`VOLATILE_FIELDS`], each with its required
//!   shape;
//! - on STDERR only, the mjs runtime's own `[bee] <cmd> <n>ms` work-
//!   visibility line — see [`strip_runtime_stderr_artifacts`];
//! - on STDERR only, the PARSER TAIL (and only the tail) of the shared
//!   `readJson` parse-failure warning — see [`reconcile_parse_warnings`].
//!   This is the one artifact whose text genuinely cannot be identical on
//!   the two legs, because each runtime quotes its OWN JSON parser. The
//!   sentence's invariant prefix (`bee: could not parse JSON at <path> — `,
//!   including the path) and its invariant suffix (`. Using fallback; fix
//!   the file.`) are still compared byte-for-byte; only the parser text
//!   between them is replaced, and only when it matches that leg's parser
//!   dialect.

use crate::runner::Runtime;

const TS_PLACEHOLDER: &str = "<TS>";
const UUID_PLACEHOLDER: &str = "<UUID>";
const PID_PLACEHOLDER: &str = "<PID>";
const TOKEN_PLACEHOLDER: &str = "<TOKEN>";
pub const ROOT_PLACEHOLDER: &str = "<ROOT>";
pub const PARSE_ERROR_PLACEHOLDER: &str = "<PARSE_ERROR>";

/// How a declared key name is matched.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyMatch {
    Exact,
    Contains,
}

/// The required shape of a declared volatile value. A value that does not
/// match its declared shape is NEVER masked.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    /// Exactly what JS `new Date().toISOString()` emits:
    /// `YYYY-MM-DDTHH:MM:SS.mmmZ` — 24 characters, THREE fractional digits,
    /// literal trailing `Z`. `chrono`'s `to_rfc3339()` and
    /// `SecondsFormat::Secs`/`Micros`/`Nanos` variants all differ from this;
    /// pinning the exact form is the entire point.
    IsoMillisZ,
    /// Exactly what `crypto.randomUUID()` emits: lowercase
    /// `8-4-4-4-12` hex with the version nibble `4` and the variant nibble
    /// in `[89ab]`.
    UuidV4,
    /// A run of ASCII digits (a process id).
    Digits,
    /// Any non-empty string — used only for opaque secrets whose content is
    /// deliberately unconstrained (lock tokens).
    OpaqueString,
}

/// One declared volatile field.
pub struct VolatileField {
    pub key: &'static str,
    pub key_match: KeyMatch,
    pub shape: Shape,
    pub placeholder: &'static str,
}

/// THE ALLOWLIST. Deny-by-default: a JSON key that is not in this table is
/// never masked, no matter how volatile its value looks.
///
/// Each entry names the mjs producer it exists for, so a reader can check
/// the claim rather than trust it.
pub const VOLATILE_FIELDS: &[VolatileField] = &[
    // `decisions.mjs:322` / `reviews.mjs:46,350` — `new Date().toISOString()`
    // on every decision event and review candidate.
    VolatileField { key: "date", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    // `backlog.mjs:284` — the PBI event stamp.
    VolatileField { key: "ts", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    // `capture.mjs` queue rows.
    VolatileField { key: "at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    // `reviews.mjs` session/candidate stamps (`utcNow()` at :46).
    VolatileField { key: "created_at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    VolatileField { key: "updated_at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    VolatileField { key: "requested_at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    VolatileField { key: "checked_at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    VolatileField { key: "generated_at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    // cells trace / claims / reservations / sessions stamps.
    VolatileField { key: "capped_at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    VolatileField { key: "claimed_at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    VolatileField { key: "reserved_at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    VolatileField { key: "released_at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    VolatileField { key: "started_at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    VolatileField { key: "last_heartbeat", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    VolatileField { key: "written_at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    // `intent.mjs:236` — `advanceIntent`'s own `new Date().toISOString()`
    // stamp, the ONE key on the intent anchor that `written_at` does not
    // already cover (rpl-2). Declared with the same IsoMillisZ shape gate:
    // a Rust port emitting second- or micro-precision here still fails the
    // diff rather than being masked into agreement.
    VolatileField { key: "advanced_at", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    VolatileField { key: "timestamp", key_match: KeyMatch::Exact, shape: Shape::IsoMillisZ, placeholder: TS_PLACEHOLDER },
    // `decisions.mjs:320` / `reviews.mjs:348` — `crypto.randomUUID()`. The
    // shape gate is what makes this safe to declare at all: the vast
    // majority of `id` values in these stores are stable slugs
    // (`fixture-cell-00001`, `rpl-1`, `fixture-review-approved-stale`), and
    // those never match UUID-v4, so they are compared literally.
    VolatileField { key: "id", key_match: KeyMatch::Exact, shape: Shape::UuidV4, placeholder: UUID_PLACEHOLDER },
    // Lock bodies (D9) — `{pid, session, ts, token}`.
    VolatileField { key: "pid", key_match: KeyMatch::Exact, shape: Shape::Digits, placeholder: PID_PLACEHOLDER },
    VolatileField { key: "token", key_match: KeyMatch::Contains, shape: Shape::OpaqueString, placeholder: TOKEN_PLACEHOLDER },
];

/// Apply the full declared allowlist to `text` (stdout, or a store file's
/// content), in a fixed order: root-path rewrite first (most specific,
/// avoids the keyed pass re-matching inside a path fragment), then the
/// key-gated, shape-gated volatile fields.
pub fn normalize(text: &str, own_root: &str) -> String {
    let rewritten = rewrite_root(text, own_root);
    mask_volatile_fields(&rewritten)
}

/// Normalize a leg's STDERR.
///
/// Same allowlist as [`normalize`], plus the two runtime-specific artifacts
/// on this channel — see [`strip_runtime_stderr_artifacts`] and
/// [`reconcile_parse_warnings`]. Returns `Err` when either artifact violates
/// its declared shape; callers surface the error as a diff rather than
/// swallowing it.
pub fn normalize_stderr(text: &str, own_root: &str, runtime: Runtime) -> Result<String, String> {
    let stripped = strip_runtime_stderr_artifacts(text, runtime)?;
    let reconciled = reconcile_parse_warnings(&stripped, runtime)?;
    Ok(normalize(&reconciled, own_root))
}

/// The ONE runtime-specific stderr artifact, declared and shape-asserted.
///
/// `bee.mjs:7246` writes `[bee] <cmd> <n>ms\n` to STDERR on every direct
/// invocation (work-visibility D3, decision 4439bd7e — deliberately
/// stderr-only so stdout stays byte-identical for every verb). `queen-bee`
/// does not emit it. Diffing stderr at all therefore requires accounting for
/// this line, and doing that by "ignoring anything that looks like `[bee]`"
/// would be exactly the blanket-mask mistake this module exists to prevent.
///
/// So the contract is asserted instead of assumed, deny-by-default in both
/// directions:
///
/// - the **mjs** leg MUST end with exactly one well-shaped timing line, and
///   must carry no other `[bee] ` line anywhere — a differently shaped one
///   means the frozen contract moved and this normalization is no longer
///   valid, which is an error, not a pass;
/// - the **queen-bee** leg must carry NO `[bee] ` line at all — if the port
///   ever starts emitting one, that is a real divergence and must not be
///   quietly absorbed by a symmetric strip.
pub fn strip_runtime_stderr_artifacts(text: &str, runtime: Runtime) -> Result<String, String> {
    let marked: Vec<&str> = text.lines().filter(|l| l.starts_with("[bee] ")).collect();
    match runtime {
        Runtime::QueenBee => {
            if !marked.is_empty() {
                return Err(format!(
                    "queen-bee emitted a `[bee] …` work-visibility line on stderr ({:?}) — mjs owns that channel artifact (bee.mjs:7246); a symmetric strip here would hide a real divergence",
                    marked
                ));
            }
            Ok(text.to_string())
        }
        Runtime::Mjs => {
            if marked.len() != 1 {
                return Err(format!(
                    "mjs stderr carries {} `[bee] …` timing line(s), expected exactly 1 (bee.mjs:7246) — the work-visibility D3 contract changed, so this stderr normalization is no longer valid. stderr was: {text:?}",
                    marked.len()
                ));
            }
            let line = marked[0];
            if !is_timing_line(line) {
                return Err(format!(
                    "mjs stderr's `[bee] …` line {line:?} does not match the declared `[bee] <cmd> <n>ms` shape (bee.mjs:7246) — refusing to strip a line whose shape is not the one that was asserted"
                ));
            }
            // It is written last, after `main()` resolves. Anything after it
            // would mean something else appended to stderr afterwards.
            let mut kept: Vec<&str> = Vec::new();
            let mut seen = false;
            for l in text.lines() {
                if !seen && l == line {
                    seen = true;
                    continue;
                }
                kept.push(l);
            }
            if kept.is_empty() {
                return Ok(String::new());
            }
            Ok(format!("{}\n", kept.join("\n")))
        }
    }
}

/// `[bee] <cmd tokens> <digits>ms` — `cmd` is the resolved registry name
/// with `.` replaced by a space, or the literal `unknown` when resolution
/// failed (`bee.mjs:7220-7229`).
fn is_timing_line(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("[bee] ") else { return false };
    let Some(ms) = rest.strip_suffix("ms") else { return false };
    let Some((cmd, digits)) = ms.rsplit_once(' ') else { return false };
    if cmd.is_empty() || digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    cmd.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b' ')
}

// ─── the readJson parse-failure warning (rpl-11) ───────────────────────────
//
// The invariant head of the sentence, verbatim from the frozen oracle
// `packages/bee/lib/fsutil.mjs` `readJson`. MEASURED, not transcribed: a node
// child (v24.14.1) was run over a deliberately unparseable file and its
// stderr hexdumped, so the separator is known to be U+2014 EM DASH with one
// ASCII space on each side (`342 200 224`), and the sentence is known to end
// with a literal `.` and a single `\n` from `console.warn`:
//
//   bee: could not parse JSON at .bee/tmp/rpl-11/bad.json — Expected property
//   name or '}' in JSON at position 2 (line 1 column 3). Using fallback; fix
//   the file.
//
const PARSE_WARN_PREFIX: &str = "bee: could not parse JSON at ";
const PARSE_WARN_SEP: &str = " — ";
const PARSE_WARN_SUFFIX: &str = ". Using fallback; fix the file.";

/// The SECOND declared stderr artifact, and the only one whose text is
/// genuinely allowed to differ between the two legs.
///
/// `packages/bee/lib/fsutil.mjs:42` warns
/// `` `bee: could not parse JSON at ${file} — ${err.message}. Using fallback;
/// fix the file.` `` where `err.message` is **V8's** JSON parser text;
/// `crates/bee-core/src/fsutil.rs:82-88` emits the same sentence with
/// **serde_json's** text. Neither is wrong and neither can be changed (D1
/// freezes the mjs leg), so the two can never match byte-for-byte. Without
/// this reconciliation there is nowhere to account for that, and every ledger
/// group would have to omit unparseable-store reads from its scenarios
/// entirely — silently shipping zero coverage of the exposed surface
/// (`reviews.mjs:109,128`, `feedback.mjs:547,772`, `decisions.mjs:127`, plus
/// the `state.mjs`/`cells.mjs` reads nearly every command transits). JSONL
/// reads are unaffected: `readJsonl`/`read_jsonl` skip corrupt lines silently
/// on both sides.
///
/// The fix follows the `[bee] <cmd> <n>ms` precedent
/// ([`strip_runtime_stderr_artifacts`]) exactly — a DECLARED artifact,
/// shape-asserted in both directions, never a blanket ignore. The warning is
/// split into an **invariant prefix**, a **parser tail**, and an **invariant
/// suffix**:
///
/// - the prefix `bee: could not parse JSON at <path> — ` **including the
///   path** is preserved verbatim into the normalized text, so it is still
///   compared byte-for-byte between the legs. Two legs blaming two different
///   files stay a diff;
/// - the suffix `. Using fallback; fix the file.` is likewise preserved, and
///   a warning that does not carry it is an ERROR — the frozen sentence
///   moved, so this normalization no longer describes reality;
/// - only the tail between them is replaced with
///   [`PARSE_ERROR_PLACEHOLDER`], and only when it matches the JSON-parser
///   dialect **of the runtime that produced it** ([`is_parser_tail`]). An
///   empty tail, prose, or the *other* runtime's dialect is an error, not a
///   pass.
///
/// Two consequences that are the whole point:
///
/// 1. The warning is REPLACED, never removed. A leg that emits no warning
///    where the other does still differs, because the placeholder line is
///    present on one side and absent on the other.
/// 2. There is no blanket stderr strip. Everything that is not this exact
///    declared sentence is left untouched and compared literally, and
///    [`strip_runtime_stderr_artifacts`]'s refusal of asymmetric strips is
///    unchanged.
pub fn reconcile_parse_warnings(text: &str, runtime: Runtime) -> Result<String, String> {
    if !text.contains(PARSE_WARN_PREFIX) {
        return Ok(text.to_string());
    }
    let trailing_newline = text.ends_with('\n');
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let Some(body) = line.strip_prefix(PARSE_WARN_PREFIX) else {
            if line.contains(PARSE_WARN_PREFIX) {
                // The declared artifact occupies a WHOLE line (`console.warn`
                // / `eprintln!` both terminate it). Finding it mid-line means
                // something interleaved with it, and a mask applied to a line
                // whose framing is not the asserted one is exactly the
                // blanket-strip mistake this module exists to prevent.
                return Err(format!(
                    "stderr line {line:?} embeds the readJson parse-failure warning mid-line — the declared artifact is a whole line (fsutil.mjs:42 `console.warn`); refusing to reconcile a sentence whose framing is not the one that was asserted"
                ));
            }
            out.push(line.to_string());
            continue;
        };
        let Some(body) = body.strip_suffix(PARSE_WARN_SUFFIX) else {
            return Err(format!(
                "{} stderr's readJson parse-failure warning {line:?} does not match the declared shape `{PARSE_WARN_PREFIX}<path>{PARSE_WARN_SEP}<parser message>{PARSE_WARN_SUFFIX}` (fsutil.mjs:42 / bee-core fsutil.rs:82-88) — the frozen sentence moved, so this reconciliation is no longer valid",
                runtime.label()
            ));
        };
        // `rsplit_once`, not `split_once`: a PATH may legitimately contain an
        // em dash, a parser message never does, so the LAST separator is the
        // real one.
        let Some((path, tail)) = body.rsplit_once(PARSE_WARN_SEP) else {
            return Err(format!(
                "{} stderr's readJson parse-failure warning {line:?} carries no `{PARSE_WARN_SEP}` separator — the declared shape (fsutil.mjs:42) moved",
                runtime.label()
            ));
        };
        if path.is_empty() {
            return Err(format!(
                "{} stderr's readJson parse-failure warning {line:?} names no file — the path is part of the INVARIANT prefix and is compared byte-for-byte, so an empty one cannot be reconciled",
                runtime.label()
            ));
        }
        if !is_parser_tail(tail, runtime) {
            return Err(format!(
                "{} stderr's readJson parse-failure warning has parser tail {tail:?}, which is not a {} JSON-parser message — the tail is the ONLY part permitted to differ between the legs and it is never accepted unconditionally; an empty, prose, or wrong-dialect tail is a divergence, not a pass. Full line: {line:?}",
                runtime.label(),
                match runtime {
                    Runtime::Mjs => "V8",
                    Runtime::QueenBee => "serde_json",
                },
            ));
        }
        out.push(format!("{PARSE_WARN_PREFIX}{path}{PARSE_WARN_SEP}{PARSE_ERROR_PLACEHOLDER}{PARSE_WARN_SUFFIX}"));
    }
    let mut joined = out.join("\n");
    if trailing_newline && !joined.is_empty() {
        joined.push('\n');
    }
    Ok(joined)
}

/// Does `tail` look like a JSON parser message produced by `runtime`'s OWN
/// parser? Asserted per leg, never symmetrically: the mjs leg quoting
/// serde_json's dialect (or vice versa) means one runtime is reporting the
/// other's failure, which is a divergence worth failing on.
fn is_parser_tail(tail: &str, runtime: Runtime) -> bool {
    if tail.is_empty() {
        return false;
    }
    match runtime {
        Runtime::Mjs => is_v8_json_parse_message(tail),
        Runtime::QueenBee => is_serde_json_parse_message(tail),
    }
}

/// V8's `SyntaxError.message` for `JSON.parse`, in the three families a real
/// node child was MEASURED to produce over 15 deliberately corrupt files
/// (node v24.14.1):
///
/// 1. `<reason> at position <n> (line <n> column <n>)` — e.g. `Expected
///    property name or '}' in JSON at position 2 (line 1 column 3)`,
///    `Unterminated string in JSON at position 10 (line 1 column 11)`,
///    `Unexpected non-whitespace character after JSON at position 8 (line 1
///    column 9)`. The trailing `(line … column …)` clause is a node-20+
///    addition: node 18, which this repo's CI matrix still builds against
///    (`.github/workflows/ci.yml` `node-version: [18, 20, 22]`), emits the
///    same message WITHOUT it (`Unexpected token o in JSON at position 24`),
///    so the parenthetical is optional and the position clause is what is
///    actually asserted;
/// 2. `Unexpected end of JSON input` — the one message with no position at
///    all (empty file, truncated array);
/// 3. `Unexpected token '<c>', <snippet> is not valid JSON` — V8's
///    snippet-quoting form for a short top-level failure (node 20+ only).
///
/// Deliberately a CLOSED set. A message outside it means either the node
/// runtime's parser text changed or something other than a JSON parse error
/// reached this sentence; both are worth a loud diff, which is the safe
/// failure mode — a false refusal is visible, a false acceptance is not.
fn is_v8_json_parse_message(tail: &str) -> bool {
    // family 2
    if tail == "Unexpected end of JSON input" {
        return true;
    }
    // family 3
    if let Some(head) = tail.strip_suffix(" is not valid JSON") {
        return head.starts_with("Unexpected token '") && head.contains("', ");
    }
    // family 1, node 20+: `… at position <n> (line <n> column <n>)`
    if let Some(rest) = tail.strip_suffix(')') {
        let Some((head, paren)) = rest.rsplit_once(" (") else { return false };
        return is_line_column(paren) && has_position_clause(head);
    }
    // family 1, node 18: `… at position <n>`, no parenthetical
    has_position_clause(tail)
}

/// `serde_json::Error`'s `Display`: every syntax/EOF/data error it can raise
/// from `from_str` ends with ` at line <n> column <n>` (the io category is
/// unreachable here — `read_json` reads the file to a `String` first, so the
/// parse never sees an io error). The reason text in front of it is free-
/// form, which is exactly why the position clause is what gets asserted.
fn is_serde_json_parse_message(tail: &str) -> bool {
    let Some((reason, rest)) = tail.rsplit_once(" at line ") else { return false };
    if reason.is_empty() {
        return false;
    }
    is_line_column(&format!("line {rest}"))
}

/// `line <digits> column <digits>`, exactly.
fn is_line_column(s: &str) -> bool {
    let Some(rest) = s.strip_prefix("line ") else { return false };
    let Some((line, column)) = rest.split_once(" column ") else { return false };
    is_ascii_digits(line) && is_ascii_digits(column)
}

/// `<reason> at position <digits>`, with a non-empty reason.
fn has_position_clause(head: &str) -> bool {
    let Some((reason, position)) = head.rsplit_once(" at position ") else { return false };
    !reason.is_empty() && is_ascii_digits(position)
}

fn is_ascii_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// Replace every occurrence of `own_root` (a leg's own absolute temp-root
/// path) with the literal `<ROOT>`. Matches the root path itself and any
/// longer path that starts with it (e.g. `own_root` plus `/.bee/...`), by
/// virtue of plain substring replacement.
fn rewrite_root(text: &str, own_root: &str) -> String {
    if own_root.is_empty() {
        return text.to_string();
    }
    text.replace(own_root, ROOT_PLACEHOLDER)
}

fn matches_shape(shape: Shape, raw_value: &str) -> bool {
    // `raw_value` is the JSON token exactly as it appeared, quotes included
    // for strings.
    match shape {
        Shape::IsoMillisZ => unquote(raw_value).map(is_iso_millis_z).unwrap_or(false),
        Shape::UuidV4 => unquote(raw_value).map(is_uuid_v4).unwrap_or(false),
        Shape::Digits => {
            let bare = unquote(raw_value).unwrap_or(raw_value);
            !bare.is_empty() && bare.bytes().all(|b| b.is_ascii_digit())
        }
        Shape::OpaqueString => unquote(raw_value).map(|s| !s.is_empty()).unwrap_or(false),
    }
}

fn unquote(raw: &str) -> Option<&str> {
    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        Some(&raw[1..raw.len() - 1])
    } else {
        None
    }
}

/// `YYYY-MM-DDTHH:MM:SS.mmmZ`, exactly 24 chars — the ONE form JS
/// `Date.prototype.toISOString` emits for in-range years.
pub fn is_iso_millis_z(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 24 {
        return false;
    }
    let digits = [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18, 20, 21, 22];
    if !digits.iter().all(|&i| b[i].is_ascii_digit()) {
        return false;
    }
    b[4] == b'-' && b[7] == b'-' && b[10] == b'T' && b[13] == b':' && b[16] == b':' && b[19] == b'.' && b[23] == b'Z'
}

/// `crypto.randomUUID()`'s output: lowercase hex, version nibble `4`,
/// variant nibble in `[89ab]`.
pub fn is_uuid_v4(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 36 {
        return false;
    }
    for (i, &c) in b.iter().enumerate() {
        match i {
            8 | 13 | 18 | 23 => {
                if c != b'-' {
                    return false;
                }
            }
            _ => {
                if !(c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) {
                    return false;
                }
            }
        }
    }
    b[14] == b'4' && matches!(b[19], b'8' | b'9' | b'a' | b'b')
}

/// Scan JSON-shaped text for `"<key>":<value>` pairs whose key is declared
/// in [`VOLATILE_FIELDS`] AND whose value matches that entry's shape, and
/// replace the value with `"<placeholder>"`.
fn mask_volatile_fields(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            if let Some((key, key_end)) = read_json_key(text, i) {
                let field = VOLATILE_FIELDS.iter().find(|f| match f.key_match {
                    KeyMatch::Exact => key.eq_ignore_ascii_case(f.key),
                    KeyMatch::Contains => key.to_ascii_lowercase().contains(&f.key.to_ascii_lowercase()),
                });
                if let Some(field) = field {
                    let mut j = key_end;
                    while j < bytes.len() && (bytes[j] as char).is_whitespace() {
                        j += 1;
                    }
                    if bytes.get(j) == Some(&b':') {
                        j += 1;
                        while j < bytes.len() && (bytes[j] as char).is_whitespace() {
                            j += 1;
                        }
                        if let Some(value_end) = skip_json_value(text, j) {
                            // THE SHAPE GATE. A declared key whose value is
                            // not in the declared shape is left untouched,
                            // so a leg that emits the wrong shape still
                            // differs from the leg that emits the right one.
                            if matches_shape(field.shape, &text[j..value_end]) {
                                out.push_str(&text[i..key_end]);
                                out.push(':');
                                out.push('"');
                                out.push_str(field.placeholder);
                                out.push('"');
                                i = value_end;
                                continue;
                            }
                        }
                    }
                }
            }
        }
        let ch_len = next_char_len(text, i);
        out.push_str(&text[i..i + ch_len]);
        i += ch_len;
    }
    out
}

fn next_char_len(text: &str, i: usize) -> usize {
    text[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1)
}

/// Given `text[start] == '"'`, read a JSON string key. Returns
/// `(key_content, byte_index_just_after_closing_quote)`.
fn read_json_key(text: &str, start: usize) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    if bytes.get(start) != Some(&b'"') {
        return None;
    }
    let mut i = start + 1;
    let mut key = String::new();
    while i < bytes.len() {
        match bytes[i] {
            b'"' => return Some((key, i + 1)),
            b'\\' if i + 1 < bytes.len() => {
                key.push(bytes[i] as char);
                key.push(bytes[i + 1] as char);
                i += 2;
            }
            _ => {
                let ch_len = next_char_len(text, i);
                key.push_str(&text[i..i + ch_len]);
                i += ch_len;
            }
        }
    }
    None
}

/// Skip a single JSON value (string, number, true/false/null) starting at
/// `text[start]`. Deliberately does NOT descend into objects/arrays —
/// volatile values in this codebase's stores are always scalars, and
/// refusing to touch a nested structure is the safer failure mode (no match
/// found -> value left untouched -> a real diff there still shows).
fn skip_json_value(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    match bytes.get(start)? {
        b'"' => {
            let mut i = start + 1;
            while i < bytes.len() {
                match bytes[i] {
                    b'"' => return Some(i + 1),
                    b'\\' if i + 1 < bytes.len() => i += 2,
                    _ => i += next_char_len(text, i),
                }
            }
            None
        }
        b't' if text[start..].starts_with("true") => Some(start + 4),
        b'f' if text[start..].starts_with("false") => Some(start + 5),
        b'n' if text[start..].starts_with("null") => Some(start + 4),
        b'-' | b'0'..=b'9' => {
            let mut i = start;
            if bytes[i] == b'-' {
                i += 1;
            }
            while i < bytes.len()
                && (bytes[i].is_ascii_digit()
                    || bytes[i] == b'.'
                    || bytes[i] == b'e'
                    || bytes[i] == b'E'
                    || bytes[i] == b'+'
                    || bytes[i] == b'-')
            {
                i += 1;
            }
            Some(i)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISO: &str = "2026-07-26T05:23:13.467Z";
    const UUID: &str = "3f2a9c1e-6b4d-4f8a-9c2e-1a2b3c4d5e6f";

    #[test]
    fn rewrites_own_root_only() {
        let out = normalize("path is /tmp/leg-a/.bee/state.json", "/tmp/leg-a");
        assert_eq!(out, "path is <ROOT>/.bee/state.json");
    }

    #[test]
    fn does_not_rewrite_other_legs_root() {
        // Only THIS leg's own root is a declared allowlist member; the
        // other leg's differing path is a real diff signal, not noise.
        let out = normalize("path is /tmp/leg-b/.bee/state.json", "/tmp/leg-a");
        assert_eq!(out, "path is /tmp/leg-b/.bee/state.json");
    }

    #[test]
    fn masks_a_declared_key_holding_the_declared_shape() {
        assert_eq!(normalize(&format!("\"date\":\"{ISO}\""), ""), "\"date\":\"<TS>\"");
        assert_eq!(normalize(&format!("\"id\":\"{UUID}\""), ""), "\"id\":\"<UUID>\"");
    }

    // ── deny-by-default: an UNDECLARED key is never masked ────────────────

    #[test]
    fn an_undeclared_key_holding_a_volatile_value_is_never_masked() {
        // must_have: "the volatile-field list is deny-by-default: an unnamed
        // volatile field fails the diff rather than passing quietly, proven
        // by a test adding one." `emitted` is not in VOLATILE_FIELDS, so two
        // legs emitting two different timestamps under it still DIFFER.
        let a = normalize(&format!("{{\"emitted\":\"{ISO}\"}}"), "");
        let b = normalize("{\"emitted\":\"2026-07-26T05:23:99.999Z\"}", "");
        assert!(a.contains(ISO), "an undeclared key must keep its raw value: {a}");
        assert_ne!(a, b, "an undeclared volatile field must survive into the diff");
    }

    #[test]
    fn a_bare_timestamp_outside_any_key_is_never_masked() {
        // There is no pattern-based scrubber any more: prose that happens to
        // contain an ISO timestamp is compared literally.
        let out = normalize(&format!("handoff written {ISO} (>7 days)"), "");
        assert!(out.contains(ISO), "{out}");
    }

    // ── shape gate: the mask can never hide a shape change ────────────────

    #[test]
    fn a_declared_key_holding_the_wrong_timestamp_shape_is_left_alone() {
        // The exact chrono trap: RFC3339 with second precision. The mjs form
        // masks; this one does not, so the two still differ.
        let mjs = normalize(&format!("\"date\":\"{ISO}\""), "");
        let chrono_secs = normalize("\"date\":\"2026-07-26T05:23:13Z\"", "");
        let chrono_micros = normalize("\"date\":\"2026-07-26T05:23:13.467123Z\"", "");
        let chrono_offset = normalize("\"date\":\"2026-07-26T05:23:13.467+00:00\"", "");
        assert_eq!(mjs, "\"date\":\"<TS>\"");
        for wrong in [&chrono_secs, &chrono_micros, &chrono_offset] {
            assert_ne!(&mjs, wrong, "a wrong-shaped timestamp must not normalize into the mjs one");
        }
    }

    #[test]
    fn a_declared_key_holding_a_non_v4_uuid_is_left_alone() {
        let v4 = normalize(&format!("\"id\":\"{UUID}\""), "");
        // version nibble 1 instead of 4
        let v1 = normalize("\"id\":\"3f2a9c1e-6b4d-1f8a-9c2e-1a2b3c4d5e6f\"", "");
        // uppercase hex — `crypto.randomUUID()` never emits it
        let upper = normalize("\"id\":\"3F2A9C1E-6B4D-4F8A-9C2E-1A2B3C4D5E6F\"", "");
        assert_eq!(v4, "\"id\":\"<UUID>\"");
        assert_ne!(v4, v1);
        assert_ne!(v4, upper);
    }

    #[test]
    fn slug_ids_are_compared_literally_not_masked() {
        let out = normalize("{\"id\":\"fixture-cell-00001\"}", "");
        assert_eq!(out, "{\"id\":\"fixture-cell-00001\"}");
    }

    #[test]
    fn iso_millis_z_predicate_is_exact() {
        assert!(is_iso_millis_z("2026-07-26T00:00:00.000Z"));
        for bad in [
            "2026-07-26T00:00:00Z",
            "2026-07-26T00:00:00.00Z",
            "2026-07-26T00:00:00.0000Z",
            "2026-07-26T00:00:00.000+00:00",
            "2026-07-26 00:00:00.000Z",
            "2026-07-26T00:00:00.000",
        ] {
            assert!(!is_iso_millis_z(bad), "{bad} must not pass the mjs toISOString shape");
        }
    }

    #[test]
    fn pid_and_token_keep_their_previous_behavior() {
        let out = normalize("{\"pid\":12345,\"pidgeon\":12345}", "");
        assert!(out.contains("\"pid\":\"<PID>\""));
        assert!(out.contains("\"pidgeon\":12345"));
        let out = normalize("{\"token\":\"abc\",\"claim_token\":\"xyz\"}", "");
        assert!(out.contains("\"token\":\"<TOKEN>\""));
        assert!(out.contains("\"claim_token\":\"<TOKEN>\""));
    }

    #[test]
    fn leaves_unrelated_numbers_untouched() {
        let out = normalize("{\"cells_count\":250,\"decisions_bytes\":700000}", "");
        assert_eq!(out, "{\"cells_count\":250,\"decisions_bytes\":700000}");
    }

    // ── the mjs stderr timing line ────────────────────────────────────────

    #[test]
    fn strips_exactly_one_well_shaped_mjs_timing_line() {
        let out = strip_runtime_stderr_artifacts("cells show: unknown flag --x.\n[bee] cells show 3ms\n", Runtime::Mjs)
            .unwrap();
        assert_eq!(out, "cells show: unknown flag --x.\n");
        assert_eq!(strip_runtime_stderr_artifacts("[bee] unknown 2ms\n", Runtime::Mjs).unwrap(), "");
    }

    #[test]
    fn refuses_mjs_stderr_with_no_timing_line() {
        let err = strip_runtime_stderr_artifacts("boom\n", Runtime::Mjs).unwrap_err();
        assert!(err.contains("expected exactly 1"), "{err}");
    }

    #[test]
    fn refuses_a_misshaped_timing_line_instead_of_stripping_it() {
        let err = strip_runtime_stderr_artifacts("[bee] cells show FASTms\n", Runtime::Mjs).unwrap_err();
        assert!(err.contains("does not match the declared"), "{err}");
    }

    // ── the readJson parse-failure warning (rpl-11) ──────────────────────
    //
    // The mjs sentences below are VERBATIM output of the frozen oracle
    // (`packages/bee/lib/fsutil.mjs` `readJson`) executed over deliberately
    // unparseable files on node v24.14.1 — measured, never transcribed from
    // the source text. Only the leading path was swapped for this leg's
    // temp root.
    const MJS_WARN_TAIL: &str = "Expected property name or '}' in JSON at position 2 (line 1 column 3)";

    /// The queen-bee tail, in `serde_json`'s dialect. This crate has zero
    /// dependencies by design (`enrich.rs`), so the port's parser cannot be
    /// linked in here to generate one — which is exactly why the cmd-check
    /// scenario `seam/unparseable-whole-json-store` exists: it runs the REAL
    /// `queen-bee` binary over a corrupt store and puts its actual stderr
    /// through this predicate. If the serde dialect ever stops matching,
    /// that scenario fails; these unit tests pin the rule, the scenario pins
    /// the reality.
    /// MEASURED from the real `target/release/queen-bee status --json` over
    /// the same corrupt `.bee/cells/archive/summary.json` body the cmd-check
    /// scenario seeds. The mjs leg answered the SAME read with `Unexpected
    /// token 'o', ..."seable": not json at"... is not valid JSON` — two
    /// texts with nothing in common, which is the whole problem.
    const RUST_WARN_TAIL: &str = "expected ident at line 1 column 26";

    fn mjs_warn(root: &str, tail: &str) -> String {
        format!("bee: could not parse JSON at {root}/.bee/config.json — {tail}. Using fallback; fix the file.\n[bee] status 12ms\n")
    }

    fn rust_warn(root: &str, tail: &str) -> String {
        format!("bee: could not parse JSON at {root}/.bee/config.json — {tail}. Using fallback; fix the file.\n")
    }

    #[test]
    fn the_two_runtimes_parser_texts_reconcile_to_one_normalized_sentence() {
        // THE POINT OF THE CELL. V8's text and serde_json's text can never
        // match byte-for-byte, so without this reconciliation no scenario
        // could ever cover an unparseable whole-JSON store.
        let a = normalize_stderr(&mjs_warn("/tmp/leg-a", MJS_WARN_TAIL), "/tmp/leg-a", Runtime::Mjs).unwrap();
        let b = normalize_stderr(&rust_warn("/tmp/leg-b", RUST_WARN_TAIL), "/tmp/leg-b", Runtime::QueenBee)
            .unwrap();
        assert_eq!(a, b, "the two legs' parse warnings must reconcile");
        assert_eq!(
            a,
            "bee: could not parse JSON at <ROOT>/.bee/config.json — <PARSE_ERROR>. Using fallback; fix the file.\n",
            "the invariant prefix, the path and the invariant suffix must all survive normalization"
        );
    }

    #[test]
    fn a_leg_that_emits_no_warning_still_differs_from_one_that_does() {
        // No blanket strip: the warning is REPLACED, never removed, so a
        // port that silently swallowed the parse failure is a diff.
        let warned =
            normalize_stderr(&mjs_warn("/tmp/leg-a", MJS_WARN_TAIL), "/tmp/leg-a", Runtime::Mjs).unwrap();
        let silent = normalize_stderr("", "/tmp/leg-b", Runtime::QueenBee).unwrap();
        assert_ne!(warned, silent, "a leg that emitted no warning must not normalize into agreement");
        assert!(warned.contains(PARSE_ERROR_PLACEHOLDER), "{warned}");
    }

    #[test]
    fn a_differing_path_inside_the_invariant_prefix_is_still_a_diff() {
        // Only the tail is masked. The path is part of the asserted
        // invariant prefix and is compared byte-for-byte.
        let a = normalize_stderr(&mjs_warn("/tmp/leg-a", MJS_WARN_TAIL), "/tmp/leg-a", Runtime::Mjs).unwrap();
        let b = normalize_stderr(
            &format!(
                "bee: could not parse JSON at /tmp/leg-b/.bee/state.json — {RUST_WARN_TAIL}. Using fallback; fix the file.\n"
            ),
            "/tmp/leg-b",
            Runtime::QueenBee,
        )
        .unwrap();
        assert_ne!(a, b, "config.json vs state.json must survive as a diff");
    }

    #[test]
    fn an_empty_parser_tail_is_refused_on_either_leg() {
        for (rt, label) in [(Runtime::Mjs, "mjs"), (Runtime::QueenBee, "queen-bee")] {
            let text = if rt == Runtime::Mjs {
                "bee: could not parse JSON at /r/.bee/config.json — . Using fallback; fix the file.\n[bee] status 1ms\n".to_string()
            } else {
                "bee: could not parse JSON at /r/.bee/config.json — . Using fallback; fix the file.\n".to_string()
            };
            let err = normalize_stderr(&text, "/r", rt).unwrap_err();
            assert!(err.contains("parser tail"), "{label}: {err}");
        }
    }

    #[test]
    fn a_non_parser_shaped_tail_is_refused_on_either_leg() {
        // The tail is never accepted unconditionally: it must look like the
        // JSON parser message of the runtime that produced it.
        let cases: [(Runtime, &str, &str); 4] = [
            (Runtime::Mjs, "boom", "arbitrary prose"),
            // serde_json's own dialect, coming out of the mjs leg: wrong.
            (Runtime::Mjs, "key must be a string at line 1 column 3", "serde text on the mjs leg"),
            (Runtime::QueenBee, "boom", "arbitrary prose"),
            // V8's dialect, coming out of the rust leg: wrong.
            (
                Runtime::QueenBee,
                "Expected property name or '}' in JSON at position 2 (line 1 column 3)",
                "V8 text on the queen-bee leg",
            ),
        ];
        for (rt, tail, why) in cases {
            let text = if rt == Runtime::Mjs {
                format!("bee: could not parse JSON at /r/x.json — {tail}. Using fallback; fix the file.\n[bee] status 1ms\n")
            } else {
                format!("bee: could not parse JSON at /r/x.json — {tail}. Using fallback; fix the file.\n")
            };
            let err = normalize_stderr(&text, "/r", rt).unwrap_err();
            assert!(err.contains("parser tail"), "{why}: expected a refusal, got: {err}");
        }
    }

    #[test]
    fn a_warning_whose_invariant_suffix_moved_is_refused_not_masked() {
        let err = normalize_stderr(
            "bee: could not parse JSON at /r/x.json — Unexpected end of JSON input\n",
            "/r",
            Runtime::QueenBee,
        )
        .unwrap_err();
        assert!(err.contains("declared shape"), "{err}");
    }

    #[test]
    fn every_measured_v8_parse_message_is_accepted_on_the_mjs_leg() {
        // Every one of these is VERBATIM node v24.14.1 output through the
        // frozen `readJson`, measured over 15 deliberately corrupt files.
        for tail in [
            "Unexpected end of JSON input",
            "Expected property name or '}' in JSON at position 1 (line 1 column 2)",
            "Expected double-quoted property name in JSON at position 7 (line 1 column 8)",
            "Unterminated string in JSON at position 10 (line 1 column 11)",
            "Bad escaped character in JSON at position 8 (line 1 column 9)",
            "Bad control character in string literal in JSON at position 8 (line 1 column 9)",
            "Unexpected token 'h', \"hello\" is not valid JSON",
            "Unexpected token 'o', ...\"seable\": not json at\"... is not valid JSON",
            "Unexpected non-whitespace character after JSON at position 8 (line 1 column 9)",
            "No number after minus sign in JSON at position 7 (line 1 column 8)",
            "Bad Unicode escape in JSON at position 9 (line 1 column 10)",
            // node 18 (still in the CI matrix): same family, no parenthetical.
            "Unexpected token o in JSON at position 24",
            "Unexpected string in JSON at position 12",
        ] {
            let text = format!(
                "bee: could not parse JSON at /r/x.json — {tail}. Using fallback; fix the file.\n[bee] status 1ms\n"
            );
            let out = normalize_stderr(&text, "/r", Runtime::Mjs)
                .unwrap_or_else(|e| panic!("measured V8 message must be accepted: {tail:?} -> {e}"));
            assert_eq!(
                out,
                "bee: could not parse JSON at <ROOT>/x.json — <PARSE_ERROR>. Using fallback; fix the file.\n"
            );
        }
    }

    #[test]
    fn the_serde_json_dialect_is_accepted_on_the_queen_bee_leg() {
        // `serde_json::Error`'s `Display` always appends ` at line <n>
        // column <n>` for a syntax/EOF/data error, whatever the reason text
        // is (the io category is unreachable — `read_json` reads the file to
        // a String first). The first entry is MEASURED from the real
        // `queen-bee` binary; the live proof that the dialect still
        // describes reality is the `seam/unparseable-whole-json-store`
        // cmd-check scenario, which puts the port's ACTUAL stderr through
        // this predicate on every run.
        for tail in [
            RUST_WARN_TAIL,
            "key must be a string at line 1 column 3",
            "EOF while parsing a value at line 1 column 0",
            "expected value at line 1 column 1",
            "expected `,` or `}` at line 1 column 8",
            "control character (\\u0000-\\u001F) found while parsing a string at line 1 column 9",
            "trailing characters at line 1 column 9",
        ] {
            let text =
                format!("bee: could not parse JSON at /r/x.json — {tail}. Using fallback; fix the file.\n");
            let out = normalize_stderr(&text, "/r", Runtime::QueenBee)
                .unwrap_or_else(|e| panic!("serde_json message must be accepted: {tail:?} -> {e}"));
            assert_eq!(
                out,
                "bee: could not parse JSON at <ROOT>/x.json — <PARSE_ERROR>. Using fallback; fix the file.\n"
            );
        }
    }

    #[test]
    fn unrelated_stderr_lines_are_untouched_by_the_reconciliation() {
        let out = normalize_stderr(
            "cells show: unknown flag --x.\nbee: could not parse JSON at /r/x.json — Unexpected end of JSON input. Using fallback; fix the file.\n[bee] cells show 3ms\n",
            "/r",
            Runtime::Mjs,
        )
        .unwrap();
        assert_eq!(
            out,
            "cells show: unknown flag --x.\nbee: could not parse JSON at <ROOT>/x.json — <PARSE_ERROR>. Using fallback; fix the file.\n"
        );
    }

    #[test]
    fn refuses_a_queen_bee_leg_that_emits_the_line() {
        let err = strip_runtime_stderr_artifacts("[bee] cells show 1ms\n", Runtime::QueenBee).unwrap_err();
        assert!(err.contains("queen-bee emitted"), "{err}");
        // The clean case passes through untouched.
        assert_eq!(strip_runtime_stderr_artifacts("boom\n", Runtime::QueenBee).unwrap(), "boom\n");
    }
}
