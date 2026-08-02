// the claimed-cells, reservations and bundle-mode reads
//
// Split out of the single 2.8k-line hooks/session_close.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, ReadJson};
use crate::hooks::adapter::{emit_hook_output, encode_block, log_crash, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson::{self, js_to_string};
use crate::state::{bypass_level, read_config_raw};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ─── cells.mjs listCells({status:'claimed'}) ───────────────────────────────

pub(crate) fn list_claimed_cells(root: &Path) -> Result<Vec<Map<String, Value>>, Flow> {
    let dir = root.join(".bee").join("cells");
    let mut cells: Vec<Map<String, Value>> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                continue; // `archive` (or any dir) is never a cell
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            // A corrupt cell warns and is skipped (readJson null → `!cell`).
            match read_json_failopen(&entry.path()) {
                ReadJson::Corrupt => unreachable!("read_json_failopen never returns Corrupt"),
                ReadJson::Missing => continue,
                ReadJson::Parsed(Value::Object(cell)) => {
                    if cell.get("status") == Some(&Value::String("claimed".into())) {
                        cells.push(cell);
                    }
                }
                // Arrays pass the .mjs's typeof-object filter but can never
                // carry status === 'claimed'; everything else is skipped.
                ReadJson::Parsed(_) => continue,
            }
        }
    }
    cells.sort_by(|a, b| {
        cmp_locale_numeric(
            &js_to_string(a.get("id").unwrap_or(&Value::Null)),
            &js_to_string(b.get("id").unwrap_or(&Value::Null)),
        )
    });
    Ok(cells)
}

/// String#localeCompare(x, 'en', {numeric: true}) approximation: digit runs
/// compare numerically, letters case-insensitively (lowercase-first tiebreak),
/// exact for the lowercase slug ids bee generates.
pub(crate) fn cmp_locale_numeric(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut ac = a.chars().peekable();
    let mut bc = b.chars().peekable();
    // Case is an ICU tertiary difference: recorded at the first divergence but
    // applied only when everything primary-level compares equal.
    let mut case_tiebreak = Ordering::Equal;
    loop {
        match (ac.peek().copied(), bc.peek().copied()) {
            (None, None) => break,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                if x.is_ascii_digit() && y.is_ascii_digit() {
                    let mut xs = String::new();
                    let mut ys = String::new();
                    while ac.peek().is_some_and(|c| c.is_ascii_digit()) {
                        xs.push(ac.next().unwrap());
                    }
                    while bc.peek().is_some_and(|c| c.is_ascii_digit()) {
                        ys.push(bc.next().unwrap());
                    }
                    let xt = xs.trim_start_matches('0');
                    let yt = ys.trim_start_matches('0');
                    let ord = xt.len().cmp(&yt.len()).then_with(|| xt.cmp(yt));
                    if ord != Ordering::Equal {
                        return ord;
                    }
                } else {
                    let xl = x.to_lowercase().next().unwrap_or(x);
                    let yl = y.to_lowercase().next().unwrap_or(y);
                    if xl != yl {
                        return xl.cmp(&yl);
                    }
                    if x != y && case_tiebreak == Ordering::Equal {
                        // lowercase before uppercase (en tertiary)
                        case_tiebreak = x.is_uppercase().cmp(&y.is_uppercase());
                    }
                    ac.next();
                    bc.next();
                }
            }
        }
    }
    case_tiebreak
}

// ─── reservations.mjs listReservations({activeOnly:true}) ──────────────────

pub(crate) struct ReservationRow {
    pub(crate) agent: Value,
    pub(crate) path: String,
    pub(crate) cell: Option<Value>,
}

pub(crate) fn list_active_reservations(root: &Path, ctx: &HookContext) -> Vec<ReservationRow> {
    // reservations.mjs's own controlRootFor: fail-open findMainRoot ?? root.
    let ctl = if ctx.worktree_resolution == "linked-valid" {
        ctx.control_root.clone().unwrap_or_else(|| root.to_path_buf())
    } else {
        root.to_path_buf()
    };
    let leases_root = ctl.join(".bee").join("runtime").join("leases");
    let now = now_ms();
    let mut rows = Vec::new();
    for kind_dir in ["cells", "paths"] {
        let dir = leases_root.join(kind_dir);
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
            let name = entry.file_name().to_string_lossy().into_owned();
            if !is_file || !name.ends_with(".json") {
                continue;
            }
            // readLeaseSafe: JSON.parse in a try — silent on corruption.
            let Ok(text) = std::fs::read(entry.path()) else { continue };
            let Ok(record) = serde_json::from_str::<Value>(&String::from_utf8_lossy(&text)) else {
                continue;
            };
            // isPathLease: string resource with the 'path:' prefix.
            let Some(resource) = record.get("resource").and_then(Value::as_str) else { continue };
            let Some(path_part) = resource.strip_prefix("path:") else { continue };
            // activeOnly: isLeaseRecordExpired on the raw expires_at.
            let expired = match record.get("expires_at") {
                None | Some(Value::Null) => false,
                Some(v) => match js_date_parse_value(v) {
                    Some(ms) => ms <= now,
                    None => false,
                },
            };
            if expired {
                continue;
            }
            let workspace_id = record.get("workspace_id").cloned().unwrap_or(Value::Null);
            let agent = match &workspace_id {
                Value::String(s) if s.starts_with("agent:") => {
                    Value::String(s["agent:".len()..].to_string())
                }
                other => other.clone(),
            };
            rows.push(ReservationRow {
                agent,
                path: path_part.to_string(),
                cell: record.get("workflow_id").cloned(),
            });
        }
    }
    rows
}

// ─── knowledge.mjs bundleMode (parseFrontmatter subset) ────────────────────

pub(crate) fn bundle_mode(root: &Path, config: &Map<String, Value>, stderr: &mut String) -> bool {
    // bundleDir(root) = resolveProductRoot(root)/docs/knowledge — the .mjs
    // re-runs resolveProductRoot here, re-emitting any warning.
    let dir = resolve_product_root(root, config, stderr).join("docs").join("knowledge");
    let Ok(meta) = std::fs::metadata(&dir) else { return false };
    if !meta.is_dir() {
        return false;
    }
    for rel in list_bundle_markdown(&dir) {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if base == "index.md" || base == "log.md" {
            continue;
        }
        let file = dir.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        let Ok(bytes) = std::fs::read(&file) else { continue };
        let text = String::from_utf8_lossy(&bytes);
        if frontmatter_has_type(&text) {
            return true;
        }
    }
    false
}

pub(crate) fn list_bundle_markdown(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(abs: &Path, rel: &str, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(abs) else { return };
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_symlink() {
                continue; // never follow (D23)
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let child_rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            if ft.is_dir() {
                walk(&entry.path(), &child_rel, out);
            } else if ft.is_file() && name.ends_with(".md") {
                out.push(child_rel);
            }
        }
    }
    if dir.exists() {
        walk(dir, "", &mut out);
    }
    out.sort();
    out
}

/// parseFrontmatter reduced to bundleMode's one question: does this file
/// parse ok, carry frontmatter, and hold a non-empty string `type`?
pub(crate) fn frontmatter_has_type(text: &str) -> bool {
    let open_len = if text.starts_with("---\r\n") {
        5
    } else if text.starts_with("---\n") {
        4
    } else {
        return false; // present: false
    };
    // Locate the closing --- line.
    let mut cursor = open_len;
    let mut inner_end: Option<usize> = None;
    while cursor <= text.len() {
        let nl = text[cursor..].find('\n').map(|i| cursor + i);
        let line_end = nl.unwrap_or(text.len());
        let mut line = &text[cursor..line_end];
        if let Some(stripped) = line.strip_suffix('\r') {
            line = stripped;
        }
        if line == "---" {
            inner_end = Some(cursor);
            break;
        }
        let Some(nl) = nl else { break };
        cursor = nl + 1;
    }
    let Some(inner_end) = inner_end else { return false }; // unclosed
    let inner_raw = &text[open_len..inner_end];
    let inner_lines: Vec<&str> = if inner_raw.is_empty() {
        Vec::new()
    } else {
        let mut v: Vec<&str> = inner_raw.split('\n').collect();
        v.pop(); // split(...).slice(0, -1)
        v
    };

    let mut root_keys: HashSet<String> = HashSet::new();
    let mut bee_keys: HashSet<String> = HashSet::new();
    let mut in_bee_map = false;
    let mut type_is_concept = false;
    for raw_line in inner_lines {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.is_empty() || line.contains('\t') {
            return false;
        }
        if let Some(inner) = line.strip_prefix("  ") {
            if !in_bee_map || inner.starts_with(' ') {
                return false;
            }
            if !parse_kv_line(inner, &mut bee_keys, &mut None) {
                return false;
            }
            continue;
        }
        if line.starts_with(' ') {
            return false;
        }
        in_bee_map = false;
        // /^([^:\s]+):$/ — a bee: map header.
        if let Some(key) = line.strip_suffix(':') {
            if !key.is_empty() && !key.contains(':') && !key.chars().any(is_js_ws) {
                if !fm_key_ok(key) || key != "bee" || root_keys.contains("bee") {
                    return false;
                }
                root_keys.insert("bee".into());
                bee_keys.clear();
                in_bee_map = true;
                continue;
            }
        }
        let mut type_slot = Some(&mut type_is_concept);
        if !parse_kv_line(line, &mut root_keys, &mut type_slot) {
            return false;
        }
    }
    type_is_concept
}

pub(crate) fn fm_key_ok(key: &str) -> bool {
    // /^[A-Za-z_][A-Za-z0-9_.-]*$/
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

/// JS regex \s (no /u): the class used by the frontmatter header rule.
pub(crate) fn is_js_ws(c: char) -> bool {
    matches!(c,
        '\t' | '\n' | '\u{000b}' | '\u{000c}' | '\r' | ' ' | '\u{00a0}' | '\u{1680}'
        | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}'
        | '\u{3000}' | '\u{feff}')
}

/// parseKeyValueLine — returns false on any typed parse failure. When
/// `type_slot` is set (root level) and the key is "type", records whether the
/// value is a non-empty string.
pub(crate) fn parse_kv_line(line: &str, keys: &mut HashSet<String>, type_slot: &mut Option<&mut bool>) -> bool {
    let Some(sep) = line.find(": ") else { return false };
    let key = &line[..sep];
    if !fm_key_ok(key) || keys.contains(key) {
        return false;
    }
    keys.insert(key.to_string());
    let raw = &line[sep + 2..];
    if raw.is_empty() {
        return false;
    }
    let value: Option<String> = if raw.starts_with('[') {
        if !parse_flow_list(raw) {
            return false;
        }
        None
    } else {
        match parse_scalar_token(raw) {
            ScalarParse::Fail => return false,
            ScalarParse::Bool => None,
            ScalarParse::Str(s) => Some(s),
        }
    };
    if key == "type" {
        if let Some(flag) = type_slot.as_deref_mut() {
            *flag = value.map(|s| !s.is_empty()).unwrap_or(false);
        }
    }
    true
}

pub(crate) enum ScalarParse {
    Fail,
    Bool,
    Str(String),
}

pub(crate) fn parse_scalar_token(raw: &str) -> ScalarParse {
    if raw == "true" || raw == "false" {
        return ScalarParse::Bool;
    }
    if raw.starts_with('"') {
        return match serde_json::from_str::<Value>(raw) {
            Ok(Value::String(s)) => ScalarParse::Str(s),
            _ => ScalarParse::Fail,
        };
    }
    if raw.starts_with('\'') {
        return ScalarParse::Fail;
    }
    if raw.starts_with(['&', '*', '!', '|', '>', '%', '@', '`', '{', '}']) {
        return ScalarParse::Fail;
    }
    ScalarParse::Str(raw.to_string())
}

pub(crate) fn parse_flow_list(raw: &str) -> bool {
    if !raw.ends_with(']') {
        return false;
    }
    let inner = raw[1..raw.len() - 1].trim_matches(is_js_ws);
    if inner.is_empty() {
        return true;
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
        return false;
    }
    segments.push(current);
    for segment in segments {
        let token = segment.trim_matches(is_js_ws);
        if token.is_empty() || matches!(parse_scalar_token(token), ScalarParse::Fail) {
            return false;
        }
    }
    true
}
