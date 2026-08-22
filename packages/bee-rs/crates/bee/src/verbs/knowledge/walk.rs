// the bundle walk, the log/index checks, and path resolution
//
// Split out of the single 4.4k-line verbs/knowledge.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── bundle walk (listBundleMarkdown — never leaves docs/knowledge/, D23) ──

/// lstat-level symlink test matching Node's dirent.isSymbolicLink(): on
/// Windows any reparse point (symlink OR junction) counts, like libuv.
pub(crate) fn is_symlinkish(path: &Path) -> bool {
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

/// None => delegate (non-UTF-16-sortable names or unrepresentable OsStrings).
pub(crate) fn list_bundle_markdown(dir: &Path) -> Option<Vec<String>> {
    fn walk(abs: &Path, rel: &str, out: &mut Vec<String>) -> Option<()> {
        let entries = match std::fs::read_dir(abs) {
            Ok(rd) => rd,
            Err(_) => return Some(()),
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_str()?.to_string();
            let child_abs = entry.path();
            if is_symlinkish(&child_abs) {
                continue; // a symlink could escape the bundle — never follow (D23)
            }
            let Ok(ft) = entry.file_type() else { continue };
            let child_rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            if ft.is_dir() {
                walk(&child_abs, &child_rel, out)?;
            } else if ft.is_file() && name.ends_with(".md") {
                out.push(child_rel);
            }
        }
        Some(())
    }
    let mut out = Vec::new();
    if dir.exists() {
        walk(dir, "", &mut out)?;
    }
    // JS Array#sort compares UTF-16 code units; UTF-8 byte order agrees below
    // U+E000 (supplementary chars sort before U+E000..U+FFFF under UTF-16).
    if out.iter().any(|rel| rel.chars().any(|c| c >= '\u{e000}')) {
        return None;
    }
    out.sort();
    Some(out)
}

pub(crate) fn read_file_lossy(path: &Path) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

// ─── log.md / index.md checks ──────────────────────────────────────────────

pub(crate) fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// isIsoDateHeading — the ISO_HEADING_RE match + the Date.UTC round-trip
/// check (which also rejects years < 100: Date.UTC maps 0–99 to 1900+y).
pub(crate) fn is_iso_date_heading(text: &str) -> bool {
    let b = text.as_bytes();
    let digit = |i: usize| i < b.len() && b[i].is_ascii_digit();
    if !(digit(0) && digit(1) && digit(2) && digit(3) && b.get(4) == Some(&b'-') && digit(5) && digit(6)
        && b.get(7) == Some(&b'-') && digit(8) && digit(9))
    {
        return false;
    }
    let mut i = 10usize;
    if i < b.len() {
        // optional time part: [T ]HH:MM(:SS(.frac)?)?(Z|[+-]HH:?MM)?
        if !(b[i] == b'T' || b[i] == b' ') {
            return false;
        }
        i += 1;
        if !(digit(i) && digit(i + 1) && b.get(i + 2) == Some(&b':') && digit(i + 3) && digit(i + 4)) {
            return false;
        }
        i += 5;
        if b.get(i) == Some(&b':') {
            if !(digit(i + 1) && digit(i + 2)) {
                return false;
            }
            i += 3;
            if b.get(i) == Some(&b'.') {
                i += 1;
                let start = i;
                while digit(i) {
                    i += 1;
                }
                if i == start {
                    return false;
                }
            }
        }
        match b.get(i) {
            None => {}
            Some(b'Z') => {
                i += 1;
            }
            Some(b'+') | Some(b'-') => {
                i += 1;
                if !(digit(i) && digit(i + 1)) {
                    return false;
                }
                i += 2;
                if b.get(i) == Some(&b':') {
                    i += 1;
                }
                if !(digit(i) && digit(i + 1)) {
                    return false;
                }
                i += 2;
            }
            Some(_) => return false,
        }
        if i != b.len() {
            return false;
        }
    }
    let y: i64 = text[0..4].parse().unwrap_or(-1);
    let m: i64 = text[5..7].parse().unwrap_or(-1);
    let d: i64 = text[8..10].parse().unwrap_or(-1);
    y >= 100 && (1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, m)
}

pub(crate) fn finding(file: &str, code: &str, message: String) -> Value {
    let mut m = Map::new();
    m.insert("file".into(), Value::String(file.to_string()));
    m.insert("code".into(), Value::String(code.to_string()));
    m.insert("message".into(), Value::String(message));
    Value::Object(m)
}

pub(crate) fn check_index_file(rel: &str, text: &str, errors: &mut Vec<Value>) -> Option<()> {
    let parsed = parse_frontmatter(text);
    let is_root = rel == "index.md";
    if !is_root {
        if !matches!(parsed, Fm::Absent) {
            // presence alone decides — parseability does not rescue it
            errors.push(finding(
                rel,
                "index_frontmatter",
                "a non-root index.md must not carry frontmatter (OKF §6; D4)".to_string(),
            ));
        }
        return Some(());
    }
    match parsed {
        Fm::Absent => Some(()),
        Fm::Failed { code, message, line } => {
            errors.push(finding(
                rel,
                "unparseable_frontmatter",
                format!("root index.md frontmatter is unparseable — {code}: {message} (line {line})"),
            ));
            Some(())
        }
        Fm::Parsed { data, .. } => {
            let extra: Vec<&String> = data.keys().filter(|k| k.as_str() != "okf_version").collect();
            if !extra.is_empty() {
                errors.push(finding(
                    rel,
                    "root_index_extra_keys",
                    format!(
                        "root index.md may carry only okf_version (OKF §9); found extra key(s): {}",
                        extra.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                    ),
                ));
            }
            Some(())
        }
    }
}

pub(crate) fn check_log_file(rel: &str, text: &str, errors: &mut Vec<Value>) {
    for (i, line) in text.split('\n').enumerate() {
        // /^##\s+(.*?)\s*$/ — '##', >=1 JS-\s, content trimmed of trailing \s.
        let Some(rest) = line.strip_prefix("##") else { continue };
        let after_ws = rest.trim_start_matches(js_is_space);
        if after_ws.len() == rest.len() {
            continue; // no whitespace after '##' — the regex requires \s+
        }
        let content = after_ws.trim_end_matches(js_is_space);
        if !is_iso_date_heading(content) {
            errors.push(finding(
                rel,
                "log_heading_not_iso",
                format!(
                    "log.md date heading {} (line {}) is not ISO 8601 (OKF §7 MUST)",
                    js_quote_str(content),
                    i + 1
                ),
            ));
        }
    }
}

// ─── path resolution inside the bundle (resolveInsideBundle subset) ────────

/// resolveInsideBundle + normalizeBundleTarget: lexically resolve `target`
/// against the ABSOLUTE bundle `dir` exactly like path.resolve (pops through
/// '..' and re-entry, clamps at the filesystem root, case-sensitive prefix
/// compare), and return the bundle-relative path with '/' separators when the
/// result is a strict descendant of `dir`; None when it escapes (never
/// followed, D23). Err(()) => delegate (drive-letter / rooted shapes whose
/// win32 path.resolve semantics are not fully modeled here).
pub(crate) fn normalize_bundle_target(dir: &Path, target: &str) -> Result<Option<String>, ()> {
    if target.is_empty() {
        return Ok(None);
    }
    if target.contains(':') || target.starts_with('/') || target.starts_with('\\') {
        return Err(()); // drive-relative / rooted forms — Node decides
    }
    // The bundle dir's own normal components are the containment prefix
    // (path.resolve(dir) — dir is already absolute and '..'-free here).
    let base: Vec<String> = dir
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(os) => os.to_str().map(str::to_string),
            _ => None,
        })
        .collect();
    let mut stack: Vec<String> = base.clone();
    for seg in target.split(['/', '\\']) {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            stack.pop(); // at the root, path.resolve clamps — pop of empty is a no-op
        } else {
            stack.push(seg.to_string());
        }
    }
    if stack.len() <= base.len() || stack[..base.len()] != base[..] {
        return Ok(None); // not a strict descendant of the bundle dir
    }
    Ok(Some(stack[base.len()..].join("/")))
}

/// resolveInsideBundle for existence checks: absolute path when contained.
pub(crate) fn resolve_inside_bundle(dir: &Path, target: &str) -> Result<Option<PathBuf>, ()> {
    Ok(normalize_bundle_target(dir, target)?.map(|rel| join_rel(dir, &rel)))
}

// ─── concept inventory (collectConcepts) ───────────────────────────────────

pub(crate) struct Concept {
    pub(crate) path: String,
    pub(crate) data: Map<String, Value>,
}

/// None => delegate (walk/name issues).
pub(crate) fn collect_concepts(dir: &Path) -> Option<Vec<Concept>> {
    let mut concepts = Vec::new();
    for rel in list_bundle_markdown(dir)? {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if is_reserved_basename(base) {
            continue;
        }
        let data = match read_file_lossy(&join_rel(dir, &rel)) {
            Err(_) => Map::new(), // unreadable: keep the row with empty data
            Ok(text) => match parse_frontmatter(&text) {
                Fm::Parsed { data, .. } => data,
                _ => Map::new(),
            },
        };
        concepts.push(Concept { path: rel, data });
    }
    Some(concepts)
}

pub(crate) fn join_rel(dir: &Path, rel: &str) -> PathBuf {
    let mut p = dir.to_path_buf();
    for seg in rel.split('/') {
        p.push(seg);
    }
    p
}

/// beeOf(data) — the bee map when it is a plain object, else empty.
pub(crate) fn bee_of(data: &Map<String, Value>) -> Map<String, Value> {
    match data.get("bee") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    }
}

pub(crate) fn dir_of(rel: &str) -> &str {
    match rel.rfind('/') {
        Some(p) => &rel[..p],
        None => "",
    }
}

pub(crate) fn str_field<'a>(map: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    match map.get(key) {
        Some(Value::String(s)) if !s.is_empty() => Some(s),
        _ => None,
    }
}

// ─── checkBundle (D4/D13 + G14 layer 3) ────────────────────────────────────

pub(crate) struct CheckReport {
    pub(crate) okf_errors: Vec<Value>,
    pub(crate) profile_errors: Vec<Value>,
    pub(crate) warnings: Vec<Value>,
    pub(crate) files: usize,
    pub(crate) concepts: usize,
    pub(crate) ok: bool,
    pub(crate) notes: Vec<String>,
}
