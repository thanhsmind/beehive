// argv flag parsing and the Node path port
//
// Split out of the single 3k-line verbs/reservations.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, resolve_store_root_worktree, Roots, RootsWt, StoreRoots};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ─── argv flag parsing (bee.mjs parseFlags, mirrored exactly) ──────────────

/// bee.mjs FLAG_ALONE_BOOLEANS — the closed set of flags that are boolean
/// when they appear alone; every other flag consumes the next token.
pub(crate) const FLAG_ALONE_BOOLEANS: &[&str] = &[
    "json", "stdin", "active-only", "dry-run", "write", "as-lane", "no-lane",
    "waive-scribing-debt", "waive-compounding", "html", "string", "cleanup",
    "force-ownership", "local", "all", "untagged", "check", "with-companion",
    "lanes-full", "strict", "queue-submit", "show", "isolate", "set", "brief",
    "all-but-active", "merge", "claim",
];

#[derive(Clone, PartialEq)]
pub(crate) enum FlagV {
    /// Flag-alone boolean — JS `true`.
    Present,
    /// String value (from `--f v` or `--f=v`).
    S(String),
}

pub(crate) struct Flags(pub Vec<(String, FlagV)>);

impl Flags {
    pub(crate) fn get(&self, name: &str) -> Option<&FlagV> {
        self.0.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }
    pub(crate) fn insert(&mut self, name: &str, value: FlagV) {
        // JS `flags[name] = value`: first-occurrence position, last value.
        if let Some(slot) = self.0.iter_mut().find(|(k, _)| k == name) {
            slot.1 = value;
        } else {
            self.0.push((name.to_string(), value));
        }
    }
    /// Non-empty string value (requireFlag's accepted shape). Anything else —
    /// absent, boolean-true, empty — is Node's own refusal path → caller
    /// delegates.
    pub(crate) fn req_str(&self, name: &str) -> Option<&str> {
        match self.get(name) {
            Some(FlagV::S(s)) if !s.is_empty() => Some(s),
            _ => None,
        }
    }
    /// `flags.x ? String(flags.x) : <absent>` for value flags.
    pub(crate) fn truthy_str(&self, name: &str) -> Option<&str> {
        match self.get(name) {
            Some(FlagV::S(s)) if !s.is_empty() => Some(s),
            _ => None,
        }
    }
}

/// bee.mjs parseFlags, byte-faithful: value flags consume the NEXT token
/// verbatim (even one that starts with "--"); any `--json` occurrence as a
/// FLAG sets json and is not stored. Returns None for shapes Node answers
/// itself (non-flag token, missing value) — the probe delegates.
pub(crate) fn parse_flags(tokens: &[&str]) -> Option<(Flags, bool)> {
    let mut flags = Flags(Vec::new());
    let mut json = false;
    let mut i = 0usize;
    while i < tokens.len() {
        let tok = tokens[i];
        if !tok.starts_with("--") {
            return None; // "unexpected argument" — Node's error path
        }
        let (name, value): (&str, FlagV) = match tok.find('=') {
            Some(eq) => (&tok[2..eq], FlagV::S(tok[eq + 1..].to_string())),
            None => {
                let name = &tok[2..];
                if FLAG_ALONE_BOOLEANS.contains(&name) {
                    (name, FlagV::Present)
                } else {
                    match tokens.get(i + 1) {
                        None => return None, // "flag --x requires a value"
                        Some(v) => {
                            i += 1;
                            (name, FlagV::S((*v).to_string()))
                        }
                    }
                }
            }
        };
        i += 1;
        if name == "json" {
            json = true;
            continue;
        }
        flags.insert(name, value);
    }
    Some((flags, json))
}

/// Routing gate for a verb's flag set: every parsed key must be a known flag
/// (or "help", which the dispatcher always allows) — an unknown flag is
/// Node's own emitError path, so the probe delegates.
pub(crate) fn keys_known(flags: &Flags, known: &[&str]) -> bool {
    flags
        .0
        .iter()
        .all(|(k, _)| k == "help" || known.contains(&k.as_str()))
}

/// A registry type:"number" flag through validate() + Number.parseInt(v,10).
/// Ok(Some(n)) — parsed integer part; Ok(None) — validate passes but parseInt
/// yields NaN (e.g. ".5"); Err — outside the plain-decimal grammar this port
/// models (hex, Infinity, overflow, empty) → delegate, Node owns the answer.
pub(crate) fn js_number_flag(raw: &str) -> Ex<Option<f64>> {
    let t = js_trim(raw);
    if t.is_empty() {
        return Err(Exotic); // validate(): invalid type for '' — Node's message
    }
    // grammar: [+-]? ( digits [ '.' digits? ] | '.' digits ) ( [eE][+-]?digits )?
    let bytes = t.as_bytes();
    let mut i = 0usize;
    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    let int_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let int_len = i - int_start;
    let mut frac_len = 0usize;
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let fs = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        frac_len = i - fs;
    }
    if int_len == 0 && frac_len == 0 {
        return Err(Exotic);
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
            i += 1;
        }
        let es = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == es {
            return Err(Exotic);
        }
    }
    if i != bytes.len() {
        return Err(Exotic);
    }
    let num: f64 = t.parse().map_err(|_| Exotic)?;
    if !num.is_finite() {
        return Err(Exotic); // Number(v) not finite → validate() refuses in Node
    }
    // parseInt(t, 10): optional sign + leading digit run.
    if int_len == 0 {
        return Ok(None); // ".5" → NaN
    }
    let int_str = &t[..int_start + int_len];
    let v: f64 = int_str.parse().map_err(|_| Exotic)?;
    Ok(Some(v))
}

// ─── Node path port (subset; provenance: hooks/write_guard.rs) ─────────────

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

pub(crate) fn np_check_modelable(p: &str) -> Ex<()> {
    if cfg!(windows) {
        if has_drive(p) && !p.chars().nth(2).map(is_sep).unwrap_or(false) {
            return Err(Exotic); // drive-relative "C:foo"
        }
        let mut ch = p.chars();
        if let (Some(a), Some(b)) = (ch.next(), ch.next()) {
            if is_sep(a) && is_sep(b) {
                return Err(Exotic); // UNC-ish
            }
        }
    }
    Ok(())
}

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

/// path.resolve over args (last wins), with the process cwd as the implicit
/// final fallback.
pub(crate) fn np_resolve(args: &[&str]) -> Ex<String> {
    let cwd_buf = std::env::current_dir()
        .map_err(|_| Exotic)?
        .to_string_lossy()
        .into_owned();
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
        let (dev, root_end, is_abs): (String, usize, bool) = if cfg!(windows) {
            if has_drive(p) {
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
        if !dev.is_empty() && !device.is_empty() && !dev.eq_ignore_ascii_case(&device) {
            continue;
        }
        if device.is_empty() {
            device = dev;
        }
        if !absolute {
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
        return Err(Exotic);
    }
    if cfg!(windows) && device.is_empty() {
        return Err(Exotic);
    }
    let norm = np_normalize_tail(&tail);
    Ok(format!("{}{}{}", device, SEP, norm))
}

pub(crate) fn np_resolve1(p: &str) -> Ex<String> {
    np_resolve(&[p])
}

pub(crate) fn np_resolve2(base: &str, p: &str) -> Ex<String> {
    np_resolve(&[base, p])
}

pub(crate) fn np_dirname(p: &str) -> String {
    let chars: Vec<char> = p.chars().collect();
    let root_len = if cfg!(windows) {
        if has_drive(p) {
            3
        } else if !chars.is_empty() && is_sep(chars[0]) {
            1
        } else {
            0
        }
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
