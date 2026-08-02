// trace helpers and the failure-signature normalizer
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

// ─── trace helpers (lib/cells.mjs) ─────────────────────────────────────────

/// lib/cells.mjs defaultTrace() — key order is load-bearing (JSON bytes).
pub(crate) fn default_trace() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("worker".into(), Value::Null);
    m.insert("outcome".into(), Value::Null);
    m.insert("files_changed".into(), Value::Array(vec![]));
    m.insert("deviations".into(), Value::Array(vec![]));
    m.insert("friction".into(), Value::Null);
    m.insert("capped_at".into(), Value::Null);
    m.insert("behavior_change".into(), Value::Bool(false));
    m.insert("verification_evidence".into(), Value::Null);
    m.insert("verify_output".into(), Value::Null);
    m.insert("verify_passed".into(), Value::Null);
    m.insert("claim_session".into(), Value::Null);
    m
}

/// JS object-spread assignment: existing keys keep their position, new keys
/// append — serde_json's preserve_order Map::insert has exactly this shape.
pub(crate) fn spread_into(base: &mut Map<String, Value>, overlay: &Map<String, Value>) {
    for (k, v) in overlay {
        base.insert(k.clone(), v.clone());
    }
}

/// `{...defaultTrace(), ...(cell.trace || {})}` — the merge every mutator
/// opens with. Falsy trace -> defaults; plain object -> overlay; a truthy
/// number/bool spreads nothing ({...5} === {}); a non-empty string or array
/// spreads index keys (JS-exotic) -> Delegate.
pub(crate) fn merge_trace(trace: Option<&Value>) -> MR<Map<String, Value>> {
    let mut base = default_trace();
    match trace {
        None | Some(Value::Null) | Some(Value::Bool(false)) => {}
        Some(Value::String(s)) if s.is_empty() => {}
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => {}
        Some(Value::Object(m)) => spread_into(&mut base, m),
        Some(Value::Array(a)) if a.is_empty() => {}
        Some(Value::Number(_)) | Some(Value::Bool(true)) => {}
        Some(_) => return Err(Fail::Delegate), // string/array index spread — JS-exotic
    }
    Ok(base)
}

/// lib/cells.mjs releaseTrace: clear claim + legacy verify evidence, keep the
/// rest (assignment order pins where absent keys append).
pub(crate) fn release_trace(mut trace: Map<String, Value>) -> Map<String, Value> {
    for key in [
        "worker",
        "claimed_at",
        "claim_session",
        "verify_command",
        "verify_output",
        "verify_passed",
        "verified_at",
    ] {
        trace.insert(key.into(), Value::Null);
    }
    trace
}

/// lib/cells.mjs appendAttempt — one revision-ledger row, claim identity read
/// from the LIVE control-plane claim file.
pub(crate) fn append_attempt(
    root: &Path,
    id: &str,
    mut trace: Map<String, Value>,
    verdict: &str,
    failure_signature: Option<String>,
    note: Option<&str>,
) -> MR<Map<String, Value>> {
    let attempts: Vec<Value> = match trace.get("attempts") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let control = control_root(root)?;
    let claim = read_claim(&control, id)?;
    let claim_str = |key: &str| -> Value {
        match claim.as_ref().and_then(|c| c.get(key)) {
            Some(Value::String(s)) => Value::String(s.clone()),
            _ => Value::Null,
        }
    };
    let acquired_at = match claim.as_ref().and_then(|c| c.get("acquired_at")) {
        Some(Value::String(s)) => Value::String(s.clone()),
        _ => claim_str("claimed_at"), // legacy claims: fall back to claimed_at
    };
    let worker = match trace.get("worker") {
        Some(Value::String(s)) => Value::String(s.clone()),
        _ => Value::Null,
    };
    let mut entry = Map::new();
    entry.insert(
        "n".into(),
        Value::Number(Number::from_f64((attempts.len() + 1) as f64).unwrap()),
    );
    entry.insert("at".into(), Value::String(utc_now()));
    entry.insert("claim_session".into(), claim_str("session"));
    entry.insert("claimed_at".into(), claim_str("claimed_at"));
    entry.insert("acquired_at".into(), acquired_at);
    entry.insert("worker".into(), worker);
    entry.insert("verdict".into(), Value::String(verdict.to_string()));
    entry.insert(
        "failure_signature".into(),
        failure_signature.map(Value::String).unwrap_or(Value::Null),
    );
    entry.insert("note".into(), note.map(|n| Value::String(n.to_string())).unwrap_or(Value::Null));
    let mut next = attempts;
    next.push(Value::Object(entry));
    trace.insert("attempts".into(), Value::Array(next));
    Ok(trace)
}

/// lib/cells.mjs checkClaimOwnership (D4/msh-4).
pub(crate) struct Ownership {
    pub(crate) ok: bool,
    pub(crate) reason: String,
    pub(crate) holder: Option<Value>,
}

pub(crate) fn check_claim_ownership(root: &Path, id: &str, session_flag: Option<&str>) -> MR<Ownership> {
    let control = control_root(root)?;
    let claim = read_claim(&control, id)?;
    let now = rsv::now_ms();
    if !claim_active(claim.as_ref(), now)? {
        return Ok(Ownership { ok: true, reason: String::new(), holder: None });
    }
    let claim = claim.unwrap();
    let owner = match claim.get("session") {
        Some(v) if js_truthy(v) => v.clone(),
        _ => return Ok(Ownership { ok: true, reason: String::new(), holder: None }),
    };
    let caller = resolve_session_flag_env(session_flag);
    // `caller === owner` — strict; a non-string owner can never equal a
    // string caller, and a null caller (undefined session) never equals it.
    if let (Some(caller), Value::String(owner_s)) = (&caller, &owner) {
        if caller == owner_s {
            return Ok(Ownership { ok: true, reason: String::new(), holder: None });
        }
    }
    let owner_disp = jsjson::js_to_string(&owner);
    Ok(Ownership {
        ok: false,
        reason: format!(
            "cell \"{id}\" is claimed by session \"{owner_disp}\" ({}) — another session owns it. Pass --force-ownership to override (audited).",
            claim_expiry(Some(&claim))?
        ),
        holder: Some(owner),
    })
}

/// lib/cells.mjs appendOwnershipOverride (D4 Δ5).
pub(crate) fn append_ownership_override(
    mut trace: Map<String, Value>,
    verb: &str,
    session_flag: Option<&str>,
    ownership: &Ownership,
) -> Map<String, Value> {
    let overrides: Vec<Value> = match trace.get("ownership_overrides") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let mut entry = Map::new();
    entry.insert("verb".into(), Value::String(verb.to_string()));
    entry.insert(
        "forced_by".into(),
        resolve_session_flag_env(session_flag).map(Value::String).unwrap_or(Value::Null),
    );
    entry.insert(
        "owner_bypassed".into(),
        if ownership.ok { Value::Null } else { ownership.holder.clone().unwrap_or(Value::Null) },
    );
    entry.insert("at".into(), Value::String(utc_now()));
    let mut next = overrides;
    next.push(Value::Object(entry));
    trace.insert("ownership_overrides".into(), Value::Array(next));
    trace
}

/// lib/cells.mjs guardClaimOwnership.
pub(crate) fn guard_claim_ownership(
    root: &Path,
    id: &str,
    trace: Map<String, Value>,
    verb: &str,
    session_flag: Option<&str>,
    force: bool,
) -> MR<Map<String, Value>> {
    let ownership = check_claim_ownership(root, id, session_flag)?;
    if !force {
        if !ownership.ok {
            return Err(Fail::Thrown(format!("{verb}: {}", ownership.reason)));
        }
        return Ok(trace);
    }
    Ok(append_ownership_override(trace, verb, session_flag, &ownership))
}

// ─── failure-signature normalizer (lib/cells.mjs D1) ───────────────────────
// Hand-rolled ports of the four scrub regexes + the failing-line pick; the
// sha256[..12] value lands in cell files, so byte-exact parity with V8's
// regex semantics is load-bearing (pinned by Node-computed test vectors).

pub(crate) fn is_ascii_word(c: char) -> bool {
    is_word_char(c) // JS \w without the u flag: [A-Za-z0-9_]
}

/// /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?/g -> "<ts>"
pub(crate) fn scrub_iso_timestamps(chars: &[char]) -> Vec<char> {
    let d = |i: usize| i < chars.len() && chars[i].is_ascii_digit();
    let lit = |i: usize, c: char| i < chars.len() && chars[i] == c;
    let mut out = Vec::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        let m = (|| -> Option<usize> {
            let mut j = i;
            for _ in 0..4 {
                if !d(j) {
                    return None;
                }
                j += 1;
            }
            if !lit(j, '-') {
                return None;
            }
            j += 1;
            for _ in 0..2 {
                if !d(j) {
                    return None;
                }
                j += 1;
            }
            if !lit(j, '-') {
                return None;
            }
            j += 1;
            for _ in 0..2 {
                if !d(j) {
                    return None;
                }
                j += 1;
            }
            if !lit(j, 'T') {
                return None;
            }
            j += 1;
            for block in 0..3 {
                for _ in 0..2 {
                    if !d(j) {
                        return None;
                    }
                    j += 1;
                }
                if block < 2 {
                    if !lit(j, ':') {
                        return None;
                    }
                    j += 1;
                }
            }
            if lit(j, '.') && d(j + 1) {
                j += 1;
                while d(j) {
                    j += 1;
                }
            }
            if lit(j, 'Z') {
                j += 1;
            }
            Some(j)
        })();
        match m {
            Some(end) => {
                out.extend("<ts>".chars());
                i = end;
            }
            None => {
                out.push(chars[i]);
                i += 1;
            }
        }
    }
    out
}

/// /[A-Za-z]:\\[^\s"'<>]*/g -> "<path>"
pub(crate) fn scrub_win_paths(chars: &[char]) -> Vec<char> {
    let mut out = Vec::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i].is_ascii_alphabetic()
            && i + 2 < chars.len()
            && chars[i + 1] == ':'
            && chars[i + 2] == '\\'
        {
            let mut j = i + 3;
            while j < chars.len() {
                let c = chars[j];
                if rsv::js_is_ws(c) || c == '"' || c == '\'' || c == '<' || c == '>' {
                    break;
                }
                j += 1;
            }
            out.extend("<path>".chars());
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// /\/(?:[\w.-]+\/)+[\w.-]*/g -> "<path>" (two segments minimum).
pub(crate) fn scrub_unix_paths(chars: &[char]) -> Vec<char> {
    let seg = |c: char| is_ascii_word(c) || c == '.' || c == '-';
    let mut out = Vec::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '/' {
            let mut j = i + 1;
            let mut groups = 0usize;
            loop {
                let mut k = j;
                while k < chars.len() && seg(chars[k]) {
                    k += 1;
                }
                if k > j && k < chars.len() && chars[k] == '/' {
                    groups += 1;
                    j = k + 1;
                } else {
                    // trailing [\w.-]* from j
                    if groups > 0 {
                        j = k;
                    }
                    break;
                }
            }
            if groups > 0 {
                out.extend("<path>".chars());
                i = j;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// /\b[0-9a-fA-F]{6,}\b/g -> "<hex>" — equivalently: any maximal \w run made
/// solely of hex digits, length >= 6 (boundaries cannot fall inside a run).
pub(crate) fn scrub_hex_runs(chars: &[char]) -> Vec<char> {
    let mut out = Vec::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        if is_ascii_word(chars[i]) {
            let mut j = i;
            while j < chars.len() && is_ascii_word(chars[j]) {
                j += 1;
            }
            let run = &chars[i..j];
            if run.len() >= 6 && run.iter().all(|c| c.is_ascii_hexdigit()) {
                out.extend("<hex>".chars());
            } else {
                out.extend_from_slice(run);
            }
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

pub(crate) const FAILING_LINE_NEEDLES: [&str; 4] = ["fail", "error", "refus", "denied"];

pub(crate) fn is_failing_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    FAILING_LINE_NEEDLES.iter().any(|n| lower.contains(n))
}

/// lib/cells.mjs normalizeFailureSignature — sha256(chosen line)[..12].
pub(crate) fn normalize_failure_signature(output: &str) -> String {
    let chars: Vec<char> = output.chars().collect();
    let scrubbed: String = scrub_hex_runs(&scrub_unix_paths(&scrub_win_paths(&scrub_iso_timestamps(
        &chars,
    ))))
    .into_iter()
    .collect();
    let lines: Vec<&str> = scrubbed.split('\n').map(|l| l.trim_end_matches('\r')).collect();
    let trimmed: Vec<&str> = lines.iter().map(|l| js_trim(l)).collect();
    let chosen = trimmed
        .iter()
        .find(|l| !l.is_empty() && is_failing_line(l))
        .or_else(|| trimmed.iter().find(|l| !l.is_empty()))
        .copied()
        .unwrap_or("");
    let mut hasher = Sha256::new();
    hasher.update(chosen.as_bytes());
    let hex = format!("{:x}", hasher.finalize());
    hex[..12].to_string()
}
