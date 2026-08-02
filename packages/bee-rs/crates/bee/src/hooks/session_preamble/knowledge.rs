// the knowledge bundle reads
//
// Split out of the single 3.1k-line hooks/session_preamble.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, warn_corrupt_json, ReadJson};
use crate::jsjson;
use crate::state::{bypass_level, ship_visibility};
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::version::BEE_VERSION;

// ─── knowledge bundle (knowledge.mjs bundleDir/collectConcepts/bundleMode) ─
//
// provenance: lib/knowledge.mjs l. 144-146, 499-655, 754-816; Rust lift of
// verbs/knowledge.rs:220-700 (key_re_ok, is_reserved_basename,
// parse_frontmatter and its scalar/flow-list helpers, list_bundle_markdown).
// The two delegate arms that port carries (non-sortable filenames,
// lone-surrogate escapes) are collapsed to the fail-open direction here.

pub(crate) fn bundle_dir(root: &Path) -> PathBuf {
    resolve_product_root(root).join("docs").join("knowledge")
}

pub(crate) fn key_re_ok(key: &str) -> bool {
    // /^[A-Za-z_][A-Za-z0-9_.-]*$/
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

pub(crate) fn is_reserved_basename(base: &str) -> bool {
    base == "index.md" || base == "log.md"
}

pub(crate) fn parse_scalar_token(raw: &str) -> Option<Value> {
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

pub(crate) fn parse_flow_list(raw: &str) -> Option<Value> {
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

pub(crate) fn parse_key_value_line(line: &str, target: &mut JMap) -> Option<()> {
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
pub(crate) fn parse_frontmatter(text: &str) -> Option<JMap> {
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

/// knowledge.mjs listBundleMarkdown — never leaves docs/knowledge/ (D23).
pub(crate) fn list_bundle_markdown(dir: &Path) -> Vec<String> {
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

pub(crate) fn read_file_lossy(path: &Path) -> Option<String> {
    std::fs::read(path).ok().map(|b| String::from_utf8_lossy(&b).into_owned())
}

pub(crate) struct Concept {
    pub(crate) path: String,
    pub(crate) data: JMap,
}

/// knowledge.mjs collectConcepts — the ONE inventory path (D12).
pub(crate) fn collect_concepts(root: &Path) -> Vec<Concept> {
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

pub(crate) fn join_rel(dir: &Path, rel: &str) -> PathBuf {
    let mut p = dir.to_path_buf();
    for seg in rel.split('/') {
        p.push(seg);
    }
    p
}

/// knowledge.mjs bundleMode (G8) — the ONE "does this repo have a bundle?"
/// predicate. Never throws; a missing root, an unreadable tree, or a FILE
/// where the bundle directory should be all read as `false`.
pub(crate) fn bundle_mode(root: &Path) -> bool {
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
