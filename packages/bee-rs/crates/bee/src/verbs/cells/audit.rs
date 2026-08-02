// decisions.mjs write-time safety patterns and the audit rows cells verbs log
//
// Split out of the single 9.4k-line verbs/cells.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots};
use crate::state as bstate;
use crate::verbs::reservations as rsv;
use crate::verbs::reservations::{Err2, FlagV, Out, R2};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── decisions.mjs write-time safety patterns (exact matchers) ─────────────
// The refusal message embeds the JS regex literal's own toString, so both
// the DETECTION and the PATTERN TEXT are pinned here.

pub(crate) fn ci_starts_with(chars: &[char], i: usize, needle: &str) -> bool {
    let n: Vec<char> = needle.chars().collect();
    if i + n.len() > chars.len() {
        return false;
    }
    n.iter().enumerate().all(|(k, c)| chars[i + k].to_ascii_lowercase() == *c)
}

pub(crate) fn cs_starts_with(chars: &[char], i: usize, needle: &str) -> bool {
    let n: Vec<char> = needle.chars().collect();
    if i + n.len() > chars.len() {
        return false;
    }
    n.iter().enumerate().all(|(k, c)| chars[i + k] == *c)
}

pub(crate) fn ws_run(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() && rsv::js_is_ws(chars[i]) {
        i += 1;
    }
    i
}

pub(crate) fn word_boundary_before(chars: &[char], i: usize) -> bool {
    i == 0 || !is_ascii_word(chars[i - 1])
}

pub(crate) fn word_at(chars: &[char], i: usize) -> bool {
    i < chars.len() && is_ascii_word(chars[i])
}

/// First SECRET_CONTENT_PATTERNS hit, as the JS regex literal string.
pub(crate) fn find_secret_pattern(text: &str) -> Option<&'static str> {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    // /-----BEGIN [A-Z ]*PRIVATE KEY-----/
    for i in 0..len {
        if cs_starts_with(&chars, i, "-----BEGIN ") {
            let p = i + 11;
            let mut r = p;
            while r < len && (chars[r] == ' ' || chars[r].is_ascii_uppercase()) {
                r += 1;
            }
            for s in p..=r {
                if cs_starts_with(&chars, s, "PRIVATE KEY-----") {
                    return Some("/-----BEGIN [A-Z ]*PRIVATE KEY-----/");
                }
            }
        }
    }
    // /\bAKIA[0-9A-Z]{16}\b/
    for i in 0..len {
        if word_boundary_before(&chars, i) && cs_starts_with(&chars, i, "AKIA") {
            let p = i + 4;
            if p + 16 <= len
                && chars[p..p + 16].iter().all(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
                && !word_at(&chars, p + 16)
            {
                return Some("/\\bAKIA[0-9A-Z]{16}\\b/");
            }
        }
    }
    // /\bghp_[A-Za-z0-9]{20,}\b/
    for i in 0..len {
        if word_boundary_before(&chars, i) && cs_starts_with(&chars, i, "ghp_") {
            let p = i + 4;
            let mut r = p;
            while r < len && chars[r].is_ascii_alphanumeric() {
                r += 1;
            }
            if r - p >= 20 && !word_at(&chars, r) {
                return Some("/\\bghp_[A-Za-z0-9]{20,}\\b/");
            }
        }
    }
    // /\bsk-[A-Za-z0-9_-]{20,}\b/
    let sk_class = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
    for i in 0..len {
        if word_boundary_before(&chars, i) && cs_starts_with(&chars, i, "sk-") {
            let p = i + 3;
            let mut r = p;
            while r < len && sk_class(chars[r]) {
                r += 1;
            }
            let run = r - p;
            if run >= 20 {
                for k in (20..=run).rev() {
                    let last_word = is_ascii_word(chars[p + k - 1]);
                    let next_word = word_at(&chars, p + k);
                    if last_word != next_word {
                        return Some("/\\bsk-[A-Za-z0-9_-]{20,}\\b/");
                    }
                }
            }
        }
    }
    // /\beyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{10,}/
    for i in 0..len {
        if word_boundary_before(&chars, i) && cs_starts_with(&chars, i, "eyJ") {
            let p = i + 3;
            let mut r = p;
            while r < len && sk_class(chars[r]) {
                r += 1;
            }
            if r - p >= 20 && r < len && chars[r] == '.' {
                let q = r + 1;
                let mut r2 = q;
                while r2 < len && sk_class(chars[r2]) {
                    r2 += 1;
                }
                if r2 - q >= 10 {
                    return Some("/\\beyJ[A-Za-z0-9_-]{20,}\\.[A-Za-z0-9_-]{10,}/");
                }
            }
        }
    }
    // /\b(?:api[_-]?key|secret|token|password|passwd)\s*[:=]\s*['"]?[^\s'"]{6,}/i
    const KEYWORDS: [&str; 7] = ["api_key", "api-key", "apikey", "secret", "token", "password", "passwd"];
    for i in 0..len {
        if !word_boundary_before(&chars, i) {
            continue;
        }
        for kw in KEYWORDS {
            if !ci_starts_with(&chars, i, kw) {
                continue;
            }
            let mut j = ws_run(&chars, i + kw.chars().count());
            if !(j < len && (chars[j] == ':' || chars[j] == '=')) {
                continue;
            }
            j = ws_run(&chars, j + 1);
            if j < len && (chars[j] == '\'' || chars[j] == '"') {
                j += 1;
            }
            let mut r = j;
            while r < len && !rsv::js_is_ws(chars[r]) && chars[r] != '\'' && chars[r] != '"' {
                r += 1;
            }
            if r - j >= 6 {
                return Some("/\\b(?:api[_-]?key|secret|token|password|passwd)\\s*[:=]\\s*['\"]?[^\\s'\"]{6,}/i");
            }
        }
    }
    None
}

/// First INJECTION_PATTERNS hit, as the JS regex literal string.
pub(crate) fn find_injection_pattern(text: &str) -> Option<&'static str> {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let qualifiers = ["previous", "prior", "above", "earlier"];
    let terminals = ["instructions", "messages", "context", "prompt"];
    let ws1 = |i: usize| -> Option<usize> {
        let j = ws_run(&chars, i);
        if j > i {
            Some(j)
        } else {
            None
        }
    };
    let match_alt = |i: usize, alts: &[&str]| -> Option<usize> {
        for alt in alts {
            if ci_starts_with(&chars, i, alt) {
                return Some(i + alt.chars().count());
            }
        }
        None
    };
    // /ignore\s+(?:all\s+)?(?:previous|prior|above|earlier)\s+(?:instructions|messages|context|prompts?)/i
    for i in 0..len {
        if !ci_starts_with(&chars, i, "ignore") {
            continue;
        }
        let Some(j) = ws1(i + 6) else { continue };
        let starts = if ci_starts_with(&chars, j, "all") {
            match ws1(j + 3) {
                Some(k) => vec![j, k],
                None => vec![j],
            }
        } else {
            vec![j]
        };
        for start in starts {
            let Some(q) = match_alt(start, &qualifiers) else { continue };
            let Some(w) = ws1(q) else { continue };
            if match_alt(w, &terminals).is_some() {
                return Some("/ignore\\s+(?:all\\s+)?(?:previous|prior|above|earlier)\\s+(?:instructions|messages|context|prompts?)/i");
            }
        }
    }
    // /disregard\s+(?:all\s+)?(?:previous|prior|above|earlier)/i
    for i in 0..len {
        if !ci_starts_with(&chars, i, "disregard") {
            continue;
        }
        let Some(j) = ws1(i + 9) else { continue };
        let starts = if ci_starts_with(&chars, j, "all") {
            match ws1(j + 3) {
                Some(k) => vec![j, k],
                None => vec![j],
            }
        } else {
            vec![j]
        };
        for start in starts {
            if match_alt(start, &qualifiers).is_some() {
                return Some("/disregard\\s+(?:all\\s+)?(?:previous|prior|above|earlier)/i");
            }
        }
    }
    // /<\/?\s*(?:system|assistant|user|developer|tool)\b[^>]*>/i
    let tags = ["system", "assistant", "user", "developer", "tool"];
    for i in 0..len {
        if chars[i] != '<' {
            continue;
        }
        let mut j = i + 1;
        if j < len && chars[j] == '/' {
            j += 1;
        }
        j = ws_run(&chars, j);
        let Some(k) = match_alt(j, &tags) else { continue };
        if word_at(&chars, k) {
            continue; // \b after the tag name
        }
        let mut m = k;
        while m < len && chars[m] != '>' {
            m += 1;
        }
        if m < len {
            return Some("/<\\/?\\s*(?:system|assistant|user|developer|tool)\\b[^>]*>/i");
        }
    }
    // /\[\s*(?:system|assistant|user|developer)\s*\]/i
    let btags = ["system", "assistant", "user", "developer"];
    for i in 0..len {
        if chars[i] != '[' {
            continue;
        }
        let j = ws_run(&chars, i + 1);
        let Some(k) = match_alt(j, &btags) else { continue };
        let m = ws_run(&chars, k);
        if m < len && chars[m] == ']' {
            return Some("/\\[\\s*(?:system|assistant|user|developer)\\s*\\]/i");
        }
    }
    None
}

/// decisions.mjs assertSafe over the logDecision field set.
pub(crate) fn assert_safe_decision_fields(fields: &[(&str, Option<&str>)]) -> MR<()> {
    for (field, value) in fields {
        let Some(value) = value else { continue }; // typeof !== 'string' skip
        if value.is_empty() {
            continue;
        }
        if let Some(pattern) = find_secret_pattern(value) {
            return Err(Fail::Thrown(format!(
                "Decision rejected: field \"{field}\" matches a secret pattern ({pattern}). Never log credentials — describe the decision without the secret."
            )));
        }
        if let Some(pattern) = find_injection_pattern(value) {
            return Err(Fail::Thrown(format!(
                "Decision rejected: field \"{field}\" contains instruction-like content ({pattern}). Decision text must be data, not instructions."
            )));
        }
    }
    Ok(())
}

// ─── decisions.mjs logDecision (the audit rows cells verbs write) ──────────

pub(crate) const DECISIONS_LOCK_NAME: &str = "decisions";

pub(crate) fn decisions_path(root: &Path) -> PathBuf {
    root.join(".bee").join("decisions.jsonl")
}

pub(crate) fn taxonomy_path(root: &Path) -> PathBuf {
    root.join("docs").join("decisions").join("taxonomy.json")
}

pub(crate) struct Taxonomy {
    pub(crate) schema_version: Value,
    pub(crate) tags: Vec<Value>,
    pub(crate) candidates: Vec<String>,
}

/// decisions.mjs loadTaxonomy — readJson-backed (corrupt -> warn + the same
/// absent-file fallback).
pub(crate) fn load_taxonomy(root: &Path) -> MR<Option<Taxonomy>> {
    match read_store_json(&taxonomy_path(root))? {
        None => Ok(None),
        Some(Value::Object(raw)) => {
            let tags = match raw.get("tags") {
                Some(Value::Array(a)) => a.clone(),
                _ => Vec::new(),
            };
            let candidates = match raw.get("candidates") {
                Some(Value::Array(a)) => a
                    .iter()
                    .filter_map(|c| match c {
                        Value::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            };
            let schema_version = match raw.get("schema_version") {
                Some(Value::Null) | None => json!(1.0),
                Some(v) => v.clone(),
            };
            Ok(Some(Taxonomy { schema_version, tags, candidates }))
        }
        Some(_) => Ok(None),
    }
}

/// decisions.mjs withDecisionsLockSync — bounded 15 x 20ms retry, typed
/// DecisionsLockBusyError on exhaustion.
pub(crate) fn with_decisions_lock<T>(root: &Path, f: impl FnOnce() -> MR<T>) -> MR<T> {
    let mut attempt = 0u32;
    loop {
        match lock::acquire_store_lock_once(root, DECISIONS_LOCK_NAME) {
            lock::AcquireOnce::Acquired(mut guard) => {
                let out = f();
                guard.release();
                return out;
            }
            lock::AcquireOnce::Busy { holder } => {
                attempt += 1;
                if attempt > GATE_RETRY_ATTEMPTS - 1 {
                    return Err(Fail::Thrown(format!(
                        "decisions store lock \"{DECISIONS_LOCK_NAME}\" busy: held by {}",
                        holder_who(&holder)
                    )));
                }
                std::thread::sleep(std::time::Duration::from_millis(GATE_RETRY_DELAY_MS));
            }
        }
    }
}

/// decisions.mjs classifyDecisionTags + appendTaxonomyCandidatesSync.
pub(crate) fn classify_decision_tags(root: &Path, tags: &[String]) -> MR<()> {
    let Some(taxonomy) = load_taxonomy(root)? else { return Ok(()) };
    if tags.is_empty() {
        return Err(Fail::Thrown(
            "decisions: docs/decisions/taxonomy.json exists — this decision event needs at least one tag. Pass --tags (e.g. \"billing,recall\").".into(),
        ));
    }
    let mut known: Vec<String> = taxonomy
        .tags
        .iter()
        .filter_map(|t| match t.get("name") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        })
        .collect();
    known.extend(taxonomy.candidates.iter().cloned());
    let unknown: Vec<String> = tags.iter().filter(|t| !known.contains(t)).cloned().collect();
    if unknown.is_empty() {
        return Ok(());
    }
    with_decisions_lock(root, || {
        let Some(fresh) = load_taxonomy(root)? else { return Ok(()) };
        let mut fresh_known: Vec<String> = fresh
            .tags
            .iter()
            .filter_map(|t| match t.get("name") {
                Some(Value::String(s)) => Some(s.clone()),
                _ => None,
            })
            .collect();
        fresh_known.extend(fresh.candidates.iter().cloned());
        let mut next = fresh.candidates.clone();
        for tag in &unknown {
            if !fresh_known.contains(tag) && !next.contains(tag) {
                next.push(tag.clone());
            }
        }
        if next.len() != fresh.candidates.len() {
            let mut body = Map::new();
            body.insert("schema_version".into(), fresh.schema_version.clone());
            body.insert("tags".into(), Value::Array(fresh.tags.clone()));
            body.insert(
                "candidates".into(),
                Value::Array(next.into_iter().map(Value::String).collect()),
            );
            write_json_atomic(&taxonomy_path(root), &Value::Object(body))
                .map_err(|e| Fail::Thrown(format!("{e}")))?;
        }
        Ok(())
    })
}

/// decisions.mjs logDecision — the exact event shape/append the cells verbs
/// produce (alternatives/confidence always null here; scope 'repo', source
/// 'user', matching every cells.mjs call site).
pub(crate) fn log_decision(root: &Path, decision: &str, rationale: &str, tags: &[&str]) -> MR<()> {
    if js_trim(decision).is_empty() {
        return Err(Fail::Thrown("logDecision: decision text is required.".into()));
    }
    if js_trim(rationale).is_empty() {
        return Err(Fail::Thrown("logDecision: rationale is required.".into()));
    }
    assert_safe_decision_fields(&[
        ("decision", Some(decision)),
        ("rationale", Some(rationale)),
        ("alternatives", None),
        ("scope", Some("repo")),
        ("source", Some("user")),
    ])?;
    let normalized: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
    classify_decision_tags(root, &normalized)?;
    let mut event = Map::new();
    event.insert("id".into(), Value::String(rsv::pseudo_uuid_v4()));
    event.insert("type".into(), Value::String("decide".into()));
    event.insert("date".into(), Value::String(utc_now()));
    event.insert("decision".into(), Value::String(js_trim(decision).to_string()));
    event.insert("rationale".into(), Value::String(js_trim(rationale).to_string()));
    event.insert("alternatives".into(), Value::Null);
    event.insert("scope".into(), Value::String("repo".into()));
    event.insert("source".into(), Value::String("user".into()));
    event.insert("confidence".into(), Value::Null);
    if !normalized.is_empty() {
        event.insert(
            "tags".into(),
            Value::Array(normalized.into_iter().map(Value::String).collect()),
        );
    }
    with_decisions_lock(root, || {
        crate::fsutil::append_jsonl(&decisions_path(root), &Value::Object(event))
            .map_err(|e| Fail::Thrown(format!("{e}")))
    })
}
