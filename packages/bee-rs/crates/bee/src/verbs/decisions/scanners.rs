// the secret/injection scanners, datamark, and the tag taxonomy
//
// Split out of the single 3.5k-line verbs/decisions.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{append_jsonl, ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, AcquireOnce};
use crate::verbs::reservations::{
    date_parse_val, finish, jget, js_date_parse, js_disp, js_disp_opt, js_is_ws, js_number_flag,
    js_numberify, js_quote, js_trim, keys_known, now_iso, parse_flags,
    pseudo_uuid_v4, truthy, v_is_str, Err2, Ex, Exotic, FlagV, Flags, Out, Pre, R2,
};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ─── secret / injection pattern scanners (decisions.mjs constants) ─────────

pub(crate) fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub(crate) fn boundary_before(chars: &[char], i: usize) -> bool {
    i == 0 || !is_word(chars[i - 1])
}

pub(crate) fn starts_with_ci(chars: &[char], pos: usize, lit: &str) -> bool {
    let lit: Vec<char> = lit.chars().collect();
    if pos + lit.len() > chars.len() {
        return false;
    }
    lit.iter()
        .enumerate()
        .all(|(j, c)| chars[pos + j].eq_ignore_ascii_case(c))
}

pub(crate) fn starts_with_cs(chars: &[char], pos: usize, lit: &str) -> bool {
    let lit: Vec<char> = lit.chars().collect();
    if pos + lit.len() > chars.len() {
        return false;
    }
    lit.iter().enumerate().all(|(j, c)| chars[pos + j] == *c)
}

pub(crate) fn ws_run(chars: &[char], pos: usize) -> usize {
    let mut n = 0;
    while pos + n < chars.len() && js_is_ws(chars[pos + n]) {
        n += 1;
    }
    n
}

/// /-----BEGIN [A-Z ]*PRIVATE KEY-----/
pub(crate) fn m_private_key(chars: &[char]) -> bool {
    const HEAD: &str = "-----BEGIN ";
    const TAIL: &str = "PRIVATE KEY-----";
    for i in 0..chars.len() {
        if !starts_with_cs(chars, i, HEAD) {
            continue;
        }
        let start = i + HEAD.chars().count();
        let mut k = start;
        loop {
            if starts_with_cs(chars, k, TAIL) {
                return true;
            }
            if k < chars.len() && (chars[k].is_ascii_uppercase() || chars[k] == ' ') {
                k += 1;
            } else {
                break;
            }
        }
    }
    false
}

/// /\bAKIA[0-9A-Z]{16}\b/
pub(crate) fn m_akia(chars: &[char]) -> bool {
    for i in 0..chars.len() {
        if !starts_with_cs(chars, i, "AKIA") || !boundary_before(chars, i) {
            continue;
        }
        let body = i + 4;
        if body + 16 > chars.len() {
            continue;
        }
        if !(body..body + 16).all(|k| chars[k].is_ascii_digit() || chars[k].is_ascii_uppercase()) {
            continue;
        }
        let after = body + 16;
        if after == chars.len() || !is_word(chars[after]) {
            return true;
        }
    }
    false
}

/// /\bghp_[A-Za-z0-9]{20,}\b/ — the greedy run can only satisfy the trailing
/// \b at its maximal extent (every backtrack lands before a word char).
pub(crate) fn m_ghp(chars: &[char]) -> bool {
    for i in 0..chars.len() {
        if !starts_with_cs(chars, i, "ghp_") || !boundary_before(chars, i) {
            continue;
        }
        let start = i + 4;
        let mut k = start;
        while k < chars.len() && chars[k].is_ascii_alphanumeric() {
            k += 1;
        }
        if k - start < 20 {
            continue;
        }
        if k == chars.len() || !is_word(chars[k]) {
            return true;
        }
    }
    false
}

pub(crate) fn sk_class(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// /\bsk-[A-Za-z0-9_-]{20,}\b/ — backtracks over the class run to satisfy \b.
pub(crate) fn m_sk(chars: &[char]) -> bool {
    for i in 0..chars.len() {
        if !starts_with_cs(chars, i, "sk-") || !boundary_before(chars, i) {
            continue;
        }
        let start = i + 3;
        let mut m = 0;
        while start + m < chars.len() && sk_class(chars[start + m]) {
            m += 1;
        }
        let mut k = m;
        while k >= 20 {
            let last = chars[start + k - 1];
            let next = chars.get(start + k);
            let boundary = match next {
                None => is_word(last),
                Some(nc) => is_word(last) != is_word(*nc),
            };
            if boundary {
                return true;
            }
            k -= 1;
        }
    }
    false
}

/// /\beyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{10,}/ — the literal '.' is only
/// reachable at the end of the maximal class run.
pub(crate) fn m_jwt(chars: &[char]) -> bool {
    for i in 0..chars.len() {
        if !starts_with_cs(chars, i, "eyJ") || !boundary_before(chars, i) {
            continue;
        }
        let start = i + 3;
        let mut r1 = 0;
        while start + r1 < chars.len() && sk_class(chars[start + r1]) {
            r1 += 1;
        }
        if r1 < 20 {
            continue;
        }
        let dot = start + r1;
        if dot >= chars.len() || chars[dot] != '.' {
            continue;
        }
        let mut r2 = 0;
        while dot + 1 + r2 < chars.len() && sk_class(chars[dot + 1 + r2]) {
            r2 += 1;
        }
        if r2 >= 10 {
            return true;
        }
    }
    false
}

/// /\b(?:api[_-]?key|secret|token|password|passwd)\s*[:=]\s*['"]?[^\s'"]{6,}/i
pub(crate) fn m_kv_secret(chars: &[char]) -> bool {
    let keyword_end = |pos: usize| -> Vec<usize> {
        let mut ends = Vec::new();
        // api[_-]?key
        if starts_with_ci(chars, pos, "api") {
            let mut j = pos + 3;
            if j < chars.len() && (chars[j] == '_' || chars[j] == '-') {
                j += 1;
            }
            if starts_with_ci(chars, j, "key") {
                ends.push(j + 3);
            } else if starts_with_ci(chars, pos + 3, "key") {
                ends.push(pos + 6); // apikey (no separator consumed)
            }
        }
        for kw in ["secret", "token", "password", "passwd"] {
            if starts_with_ci(chars, pos, kw) {
                ends.push(pos + kw.chars().count());
            }
        }
        ends
    };
    for i in 0..chars.len() {
        if !boundary_before(chars, i) {
            continue;
        }
        for end in keyword_end(i) {
            let mut j = end + ws_run(chars, end);
            if j >= chars.len() || !(chars[j] == ':' || chars[j] == '=') {
                continue;
            }
            j += 1;
            j += ws_run(chars, j);
            if j < chars.len() && (chars[j] == '\'' || chars[j] == '"') {
                j += 1;
            }
            let mut run = 0;
            while j + run < chars.len() {
                let c = chars[j + run];
                if js_is_ws(c) || c == '\'' || c == '"' {
                    break;
                }
                run += 1;
            }
            if run >= 6 {
                return true;
            }
        }
    }
    false
}

/// /ignore\s+(?:all\s+)?(?:previous|prior|above|earlier)\s+(?:instructions|messages|context|prompts?)/i
/// and its disregard sibling (no trailing noun group).
pub(crate) fn m_ignore_like(chars: &[char], head: &str, needs_noun: bool) -> bool {
    let tail_from = |pos: usize| -> bool {
        for kw in ["previous", "prior", "above", "earlier"] {
            if starts_with_ci(chars, pos, kw) {
                let after = pos + kw.chars().count();
                if !needs_noun {
                    return true;
                }
                let w = ws_run(chars, after);
                if w == 0 {
                    continue;
                }
                let p = after + w;
                for noun in ["instructions", "messages", "context", "prompt"] {
                    // "prompt" covers "prompts?" for a boolean match.
                    if starts_with_ci(chars, p, noun) {
                        return true;
                    }
                }
            }
        }
        false
    };
    for i in 0..chars.len() {
        if !starts_with_ci(chars, i, head) {
            continue;
        }
        let after = i + head.chars().count();
        let w1 = ws_run(chars, after);
        if w1 == 0 {
            continue;
        }
        let k = after + w1;
        // With the optional (?:all\s+) first, then backtrack to without it.
        if starts_with_ci(chars, k, "all") {
            let a = k + 3;
            let w2 = ws_run(chars, a);
            if w2 > 0 && tail_from(a + w2) {
                return true;
            }
        }
        if tail_from(k) {
            return true;
        }
    }
    false
}

pub(crate) const ROLE_TAGS: [&str; 5] = ["system", "assistant", "user", "developer", "tool"];

/// One match of /<\/?\s*(?:system|assistant|user|developer|tool)\b[^>]*>/i at
/// position i — Some(end index of '>') when it matches.
pub(crate) fn role_tag_match(chars: &[char], i: usize) -> Option<usize> {
    if chars.get(i) != Some(&'<') {
        return None;
    }
    let mut j = i + 1;
    if chars.get(j) == Some(&'/') {
        j += 1;
    }
    j += ws_run(chars, j);
    let mut kw_end = None;
    for kw in ROLE_TAGS {
        if starts_with_ci(chars, j, kw) {
            kw_end = Some(j + kw.chars().count());
            break;
        }
    }
    let j2 = kw_end?;
    if let Some(c) = chars.get(j2) {
        if is_word(*c) {
            return None; // \b after the keyword
        }
    }
    let mut k = j2;
    while k < chars.len() {
        if chars[k] == '>' {
            return Some(k);
        }
        k += 1;
    }
    None
}

pub(crate) fn m_role_tag(chars: &[char]) -> bool {
    (0..chars.len()).any(|i| role_tag_match(chars, i).is_some())
}

/// /\[\s*(?:system|assistant|user|developer)\s*\]/i
pub(crate) fn m_role_bracket(chars: &[char]) -> bool {
    for i in 0..chars.len() {
        if chars[i] != '[' {
            continue;
        }
        let j = i + 1 + ws_run(chars, i + 1);
        for kw in ["system", "assistant", "user", "developer"] {
            if starts_with_ci(chars, j, kw) {
                let k = j + kw.chars().count();
                let k2 = k + ws_run(chars, k);
                if chars.get(k2) == Some(&']') {
                    return true;
                }
            }
        }
    }
    false
}

/// (matcher, JS regex literal as String(pattern) renders it)
pub(crate) type PatternRow = (fn(&[char]) -> bool, &'static str);

pub(crate) const SECRET_PATTERNS: [PatternRow; 6] = [
    (m_private_key, "/-----BEGIN [A-Z ]*PRIVATE KEY-----/"),
    (m_akia, "/\\bAKIA[0-9A-Z]{16}\\b/"),
    (m_ghp, "/\\bghp_[A-Za-z0-9]{20,}\\b/"),
    (m_sk, "/\\bsk-[A-Za-z0-9_-]{20,}\\b/"),
    (m_jwt, "/\\beyJ[A-Za-z0-9_-]{20,}\\.[A-Za-z0-9_-]{10,}/"),
    (m_kv_secret, "/\\b(?:api[_-]?key|secret|token|password|passwd)\\s*[:=]\\s*['\"]?[^\\s'\"]{6,}/i"),
];

pub(crate) fn i_ignore(chars: &[char]) -> bool {
    m_ignore_like(chars, "ignore", true)
}

pub(crate) fn i_disregard(chars: &[char]) -> bool {
    m_ignore_like(chars, "disregard", false)
}

pub(crate) const INJECTION_PATTERNS: [PatternRow; 4] = [
    (i_ignore, "/ignore\\s+(?:all\\s+)?(?:previous|prior|above|earlier)\\s+(?:instructions|messages|context|prompts?)/i"),
    (i_disregard, "/disregard\\s+(?:all\\s+)?(?:previous|prior|above|earlier)/i"),
    (m_role_tag, "/<\\/?\\s*(?:system|assistant|user|developer|tool)\\b[^>]*>/i"),
    (m_role_bracket, "/\\[\\s*(?:system|assistant|user|developer)\\s*\\]/i"),
];

/// provenance: decisions.mjs assertSafeContent — first matching pattern wins.
pub(crate) fn assert_safe_content(field: &str, value: Option<&str>) -> Result<(), String> {
    let Some(value) = value else { return Ok(()) };
    if value.is_empty() {
        return Ok(());
    }
    let chars: Vec<char> = value.chars().collect();
    for (matcher, display) in SECRET_PATTERNS {
        if matcher(&chars) {
            return Err(format!(
                "Decision rejected: field \"{field}\" matches a secret pattern ({display}). Never log credentials — describe the decision without the secret."
            ));
        }
    }
    for (matcher, display) in INJECTION_PATTERNS {
        if matcher(&chars) {
            return Err(format!(
                "Decision rejected: field \"{field}\" contains instruction-like content ({display}). Decision text must be data, not instructions."
            ));
        }
    }
    Ok(())
}

// ─── datamark (decisions.mjs) ──────────────────────────────────────────────

pub(crate) fn datamark(v: Option<&Value>) -> String {
    // String(text ?? '')
    let text = match v {
        None | Some(Value::Null) => String::new(),
        Some(other) => js_disp(other),
    };
    let chars: Vec<char> = text.chars().collect();
    // 1) /```+/g — runs of three-or-more backticks removed entirely.
    let mut pass1: Vec<char> = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '`' {
            let mut n = 0;
            while i + n < chars.len() && chars[i + n] == '`' {
                n += 1;
            }
            if n < 3 {
                for _ in 0..n {
                    pass1.push('`');
                }
            }
            i += n;
        } else {
            pass1.push(chars[i]);
            i += 1;
        }
    }
    // 2) role tags removed globally, left to right, non-overlapping.
    let mut pass2: Vec<char> = Vec::with_capacity(pass1.len());
    let mut i = 0;
    while i < pass1.len() {
        if let Some(end) = role_tag_match(&pass1, i) {
            i = end + 1;
        } else {
            pass2.push(pass1[i]);
            i += 1;
        }
    }
    // 3) control chars (tab/newline/CR survive).
    let pass3: String = pass2
        .into_iter()
        .filter(|c| {
            !matches!(*c,
                '\u{0}'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1f}' | '\u{7f}')
        })
        .collect();
    format!("«{}»", js_trim(&pass3))
}

/// provenance: bee.mjs formatDecision.
pub(crate) fn format_decision(event: &Value) -> String {
    let head = format!(
        "[{}] {} (id {}, {})",
        js_disp_opt(jget(event, "date")),
        datamark(jget(event, "decision")),
        js_disp_opt(jget(event, "id")),
        js_disp_opt(jget(event, "type")),
    );
    let why = format!("  why: {}", datamark(jget(event, "rationale")));
    let mut lines = vec![head, why];
    if let Some(alt) = jget(event, "alternatives") {
        if truthy(alt) {
            lines.push(format!("  alternatives: {}", datamark(Some(alt))));
        }
    }
    lines.join("\n")
}

// ─── tags / taxonomy (decisions.mjs) ───────────────────────────────────────

pub(crate) const TAG_PATTERN_DISPLAY: &str =
    "/^[a-z0-9][a-z0-9-]*(:[a-z0-9][a-z0-9-]*)?$/";

/// One slug segment: the pre-namespace rule, unchanged —
/// `[a-z0-9]` then any run of `[a-z0-9-]`.
fn tag_segment_test(s: &str) -> bool {
    let mut it = s.chars();
    match it.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    it.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A tag slug, optionally namespaced by AT MOST ONE interior colon
/// (`contract:<name>` — the derived contract-status label, slp-contract
/// D2). Both segments obey the plain slug rule, so a colon at either end,
/// an empty segment, or a second colon is refused. Every pre-namespace tag
/// still validates: no colon means one segment, tested exactly as before.
///
/// The single predicate behind `normalize_tags` and
/// `normalize_tag_event_tags_value` — both inherit this shape, and
/// `TAG_PATTERN_DISPLAY` (the text every refusal prints) describes it.
pub(crate) fn tag_pattern_test(s: &str) -> bool {
    let mut segments = s.split(':');
    let Some(first) = segments.next() else {
        return false;
    };
    if !tag_segment_test(first) {
        return false;
    }
    match segments.next() {
        None => true,
        Some(second) => tag_segment_test(second) && segments.next().is_none(),
    }
}

/// provenance: bee.mjs splitList.
pub(crate) fn split_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| js_trim(s).to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// provenance: decisions.mjs normalizeTags (logDecision flavor).
pub(crate) fn normalize_tags(tags: Option<Vec<String>>) -> Result<Option<Vec<String>>, String> {
    let Some(tags) = tags else { return Ok(None) };
    let cleaned: Vec<String> = tags.iter().map(|t| js_trim(t).to_string()).collect();
    for tag in &cleaned {
        if !tag_pattern_test(tag) {
            return Err(format!(
                "logDecision: tag {} is not a valid lowercase slug (must match {TAG_PATTERN_DISPLAY}).",
                js_quote(tag)
            ));
        }
    }
    Ok(if cleaned.is_empty() { None } else { Some(cleaned) })
}

// provenance: decisions.mjs normalizeTagEventTags (tag-event flavor:
// required, never empty). The `&[String]` form that lived here is gone: both
// callers now route through tagDecisionsBatch, so the RAW-value form
// (`normalize_tag_event_tags_value`, beside the batch) is the only one, and
// the flag path gets Node's own `tagDecision = batch([entry])[0]` shape.

pub(crate) fn taxonomy_file_exists(root: &Path) -> bool {
    taxonomy_path(root).exists()
}

pub(crate) struct Taxonomy {
    pub(crate) schema_version: Value,
    pub(crate) tags: Vec<Value>,
    pub(crate) candidates: Vec<String>,
}

/// provenance: decisions.mjs loadTaxonomy — `readJson(file, null)` fail-open.
/// A corrupt taxonomy WARNS once and reads as "no taxonomy", which is the
/// `null` fallback Node's `!raw` guard already turned into `null`. The
/// downstream effect is unchanged: with no taxonomy, classification is not
/// required and `decisions log` takes its warn-only branch.
pub(crate) fn load_taxonomy(root: &Path) -> Ex<Option<Taxonomy>> {
    let raw = match read_json(&taxonomy_path(root)) {
        ReadJson::Missing => return Ok(None),
        ReadJson::Corrupt => {
            crate::fsutil::warn_corrupt_json(&taxonomy_path(root));
            return Ok(None);
        }
        ReadJson::Parsed(v) => js_numberify(&v)?,
    };
    let Value::Object(ref m) = raw else {
        return Ok(None); // !raw || not object || Array → null
    };
    let tags = match m.get("tags") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let candidates = match m.get("candidates") {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|c| match c {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    };
    let schema_version = match m.get("schema_version") {
        None | Some(Value::Null) => json!(1.0), // raw.schema_version ?? 1
        Some(v) => v.clone(),
    };
    Ok(Some(Taxonomy { schema_version, tags, candidates }))
}

pub(crate) fn taxonomy_known_names(t: &Taxonomy) -> Vec<String> {
    let mut known: Vec<String> = Vec::new();
    for tag in &t.tags {
        // fresh.tags.map((t) => t && t.name) — only string names can ever
        // equal a validated slug, so non-strings are droppable here.
        if truthy(tag) {
            if let Some(Value::String(name)) = jget(tag, "name") {
                known.push(name.clone());
            }
        }
    }
    known.extend(t.candidates.iter().cloned());
    known
}

pub(crate) const UNTAGGED_REFUSED_MESSAGE: &str = "decisions: docs/decisions/taxonomy.json exists — this decision event needs at least one tag. Pass --tags (e.g. \"billing,recall\").";

/// provenance: decisions.mjs classifyDecisionTags +
/// appendTaxonomyCandidatesSync (the locked read-modify-write).
pub(crate) fn classify_decision_tags(root: &Path, tags: &[String], lock_retries: u32) -> R2<()> {
    let Some(taxonomy) = load_taxonomy(root)? else {
        return Ok(()); // bootstrap-safe: no taxonomy, never refuses
    };
    if tags.is_empty() {
        // DecisionsUntaggedRefusedError — same emitError channel as any
        // handler throw (Err2::Msg → ctx.fail).
        return Err(Err2::Msg(UNTAGGED_REFUSED_MESSAGE.into()));
    }
    let known = taxonomy_known_names(&taxonomy);
    let unknown: Vec<&String> = tags.iter().filter(|t| !known.contains(t)).collect();
    if unknown.is_empty() {
        return Ok(());
    }
    let guard = acquire_decisions_lock(root, lock_retries).map_err(Err2::Msg)?;
    let fresh = load_taxonomy(root)?;
    if let Some(fresh) = fresh {
        let fresh_known = taxonomy_known_names(&fresh);
        let mut next = fresh.candidates.clone();
        for tag in &unknown {
            if !fresh_known.contains(tag) && !next.contains(tag) {
                next.push((*tag).clone());
            }
        }
        if next.len() != fresh.candidates.len() {
            let mut out = Map::new();
            out.insert("schema_version".into(), fresh.schema_version.clone());
            out.insert("tags".into(), Value::Array(fresh.tags.clone()));
            out.insert(
                "candidates".into(),
                Value::Array(next.into_iter().map(Value::String).collect()),
            );
            write_json_atomic(&taxonomy_path(root), &Value::Object(out)).map_err(|_| Err2::Ex)?;
        }
    }
    drop(guard);
    Ok(())
}
