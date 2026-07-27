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
//!   visibility line — see [`strip_runtime_stderr_artifacts`].

use crate::runner::Runtime;

const TS_PLACEHOLDER: &str = "<TS>";
const UUID_PLACEHOLDER: &str = "<UUID>";
const PID_PLACEHOLDER: &str = "<PID>";
const TOKEN_PLACEHOLDER: &str = "<TOKEN>";
pub const ROOT_PLACEHOLDER: &str = "<ROOT>";

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
/// Same allowlist as [`normalize`], plus the one runtime-specific artifact
/// on this channel — see [`strip_runtime_stderr_artifacts`]. Returns `Err`
/// when that artifact violates its declared shape; callers surface the error
/// as a diff rather than swallowing it.
pub fn normalize_stderr(text: &str, own_root: &str, runtime: Runtime) -> Result<String, String> {
    let stripped = strip_runtime_stderr_artifacts(text, runtime)?;
    Ok(normalize(&stripped, own_root))
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

    #[test]
    fn refuses_a_queen_bee_leg_that_emits_the_line() {
        let err = strip_runtime_stderr_artifacts("[bee] cells show 1ms\n", Runtime::QueenBee).unwrap_err();
        assert!(err.contains("queen-bee emitted"), "{err}");
        // The clean case passes through untouched.
        assert_eq!(strip_runtime_stderr_artifacts("boom\n", Runtime::QueenBee).unwrap(), "boom\n");
    }
}
