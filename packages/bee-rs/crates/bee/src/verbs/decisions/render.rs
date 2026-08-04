// the index renderer and the citation sweep
//
// Split out of the single 3.5k-line verbs/decisions.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{append_jsonl, ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, AcquireOnce};
use crate::textutil::{char_len, truncate_chars_head};
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

// ═══════════════════════════════════════════════════════════════════════════
// decisions render / decisions supersede
// ═══════════════════════════════════════════════════════════════════════════

// ─── String.prototype.localeCompare(b) — non-numeric arm ───────────────────
//
// provenance: re-derived from verbs/cells.rs `natural_cmp`/`primary_cmp`/
// `tertiary_case_cmp` and verbs/status_full.rs `locale_cmp(a, b, false)` —
// both are private to their modules, so the calibrated model is restated
// here rather than made public (the campaign keeps each verb file's ported
// surface self-contained). `locale_cmp_agrees_with_the_calibrated_probes`
// below asserts this copy answers the same measured V8/ICU probe vectors
// those two ports were calibrated against.
//
// buildDecisionIndexBody's two sorts are `a.localeCompare(b)` with NO locale
// and NO options: default collation, numeric OFF. The model:
//   primary: whitespace < punctuation < digits < letters, with ICU's
//            '_' < '-' < '.' inside punctuation and letters compared
//            case-insensitively; a shorter string that is a prefix sorts first.
//   tertiary (only on a primary tie): the first case difference decides,
//            lowercase before uppercase.
// Numeric mode is deliberately absent — "10" < "9" here, which is what
// `a.localeCompare(b)` does.
pub(crate) fn locale_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let av: Vec<char> = a.chars().collect();
    let bv: Vec<char> = b.chars().collect();
    let n = av.len().min(bv.len());
    for k in 0..n {
        let ord = lc_primary_key(av[k]).cmp(&lc_primary_key(bv[k]));
        if ord != Ordering::Equal {
            return ord;
        }
    }
    let ord = av.len().cmp(&bv.len());
    if ord != Ordering::Equal {
        return ord;
    }
    // Tertiary (case) pass — only reached when every primary key tied.
    for k in 0..n {
        let (x, y) = (av[k], bv[k]);
        if x != y && x.is_alphabetic() && y.is_alphabetic() {
            let (lx, ly) = (x.is_lowercase(), y.is_lowercase());
            if lx != ly {
                return if lx { Ordering::Less } else { Ordering::Greater };
            }
        }
    }
    Ordering::Equal
}

/// ICU primary-strength key (probe-calibrated; see `locale_cmp`).
pub(crate) fn lc_primary_key(c: char) -> (u8, u32) {
    if c.is_whitespace() {
        return (0, c as u32);
    }
    match c {
        '_' => (1, 0),
        '-' => (1, 1),
        ',' => (1, 2),
        ';' => (1, 3),
        ':' => (1, 4),
        '!' => (1, 5),
        '?' => (1, 6),
        '.' => (1, 7),
        _ if c.is_ascii_digit() => (2, c as u32 - '0' as u32),
        _ if c.is_alphabetic() => (3, c.to_lowercase().next().unwrap_or(c) as u32),
        _ => (1, 100 + c as u32),
    }
}

/// The alphabet the collation model above is CALIBRATED on: ASCII letters,
/// digits, space, and the three anchored punctuation marks. A group key with
/// anything else (accents, CJK, other punctuation, exotic whitespace) leaves
/// the proven region — the whole verb delegates before any output rather
/// than guess at an ICU weight this port never measured.
pub(crate) fn collation_safe(key: &str) -> bool {
    key.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-' | '.'))
}

// ─── decisions.mjs writeTextAtomic ─────────────────────────────────────────

pub(crate) fn write_text_atomic(file: &Path, text: &str) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        ensure_dir(dir)?;
    }
    let unique = format!(
        "{}-{}-{:08x}",
        std::process::id(),
        to_base36(TEXT_ATOMIC_COUNTER.fetch_add(1, Ordering::Relaxed)),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    );
    let mut name = file.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{unique}.tmp"));
    let tmp = file.with_file_name(name);
    let result = std::fs::write(&tmp, text).and_then(|()| std::fs::rename(&tmp, file));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

pub(crate) static TEXT_ATOMIC_COUNTER: AtomicU64 = AtomicU64::new(0);

// ─── decisions.mjs DECISION_INDEX_HEADER / formatIndexLine ─────────────────

pub(crate) const DECISION_INDEX_HEADER: &str = concat!(
    "<!--\n",
    "GENERATED FILE — do not hand-edit.\n",
    "Rendered by `bee decisions render` from the decisions store (decision-propagation D4b/D8a).\n",
    "Regenerate: `bee decisions render`. Check freshness: `bee decisions render --check`.\n",
    "Deterministic: byte-identical for the same store contents — this file never includes a\n",
    "generation timestamp or any other wall-clock value, only the dates already recorded on\n",
    "each decision event.\n",
    "-->\n",
    "\n",
    "# Decision Index",
);

pub(crate) fn decision_index_path(root: &Path) -> PathBuf {
    root.join("docs").join("decisions").join("index.md")
}

/// formatIndexLine: `- short8 · YYYY-MM-DD · first line of decision text`.
pub(crate) fn format_index_line(event: &Value) -> String {
    let short8 = truncate_chars_head(&js_disp_opt(jget(event, "id")), 8);
    let date = match jget(event, "date") {
        Some(Value::String(s)) => truncate_chars_head(s, 10),
        _ => "0000-00-00".to_string(),
    };
    // String(event.decision ?? '').split(/\r?\n/)[0]
    let decision = match jget(event, "decision") {
        None | Some(Value::Null) => String::new(),
        Some(v) => js_disp(v),
    };
    let first_line = split_crlf_first(&decision);
    format!("- {short8} · {date} · {first_line}")
}

/// `text.split(/\r?\n/)[0]` — everything before the first LF, minus one
/// trailing CR when the LF was CRLF.
pub(crate) fn split_crlf_first(text: &str) -> &str {
    match text.find('\n') {
        None => text,
        Some(i) => {
            let head = &text[..i];
            head.strip_suffix('\r').unwrap_or(head)
        }
    }
}

/// buildDecisionIndexBody (lib/decisions.mjs). Returns None when a group key
/// leaves the calibrated collation alphabet (delegate).
pub(crate) fn build_decision_index_body(root: &Path, all: bool) -> Ex<Option<(String, usize)>> {
    let decisions = active_decisions(root, all)?;
    // Insertion-ordered Map<scope, events[]>.
    let mut by_scope: Vec<(String, Vec<Value>)> = Vec::new();
    for event in decisions {
        let scope = match jget(&event, "scope") {
            Some(Value::String(s)) if !js_trim(s).is_empty() => js_trim(s).to_string(),
            _ => "repo".to_string(),
        };
        match by_scope.iter_mut().find(|(k, _)| *k == scope) {
            Some(slot) => slot.1.push(event),
            None => by_scope.push((scope, vec![event])),
        }
    }
    let mut scope_names: Vec<String> = by_scope.iter().map(|(k, _)| k.clone()).collect();
    if !scope_names.iter().all(|k| collation_safe(k)) {
        return Ok(None);
    }
    scope_names.sort_by(|a, b| locale_cmp(a, b)); // JS sort is stable (ES2019+)

    let mut blocks: Vec<String> = Vec::new();
    let mut count = 0usize;
    for scope in &scope_names {
        let mut scope_lines: Vec<String> = vec![format!("## {scope}")];
        let events = &by_scope.iter().find(|(k, _)| k == scope).unwrap().1;
        let mut by_tag: Vec<(String, Vec<&Value>)> = Vec::new();
        let mut untagged: Vec<&Value> = Vec::new();
        for event in events.iter() {
            // `Array.isArray(tags) && tags.length ? String(tags[0]) : null`
            let tag = match jget(event, "tags") {
                Some(Value::Array(a)) if !a.is_empty() => {
                    let t = js_disp(&a[0]);
                    if t.is_empty() {
                        None // falsy string -> the untagged bucket
                    } else {
                        Some(t)
                    }
                }
                _ => None,
            };
            match tag {
                Some(tag) => match by_tag.iter_mut().find(|(k, _)| *k == tag) {
                    Some(slot) => slot.1.push(event),
                    None => by_tag.push((tag, vec![event])),
                },
                None => untagged.push(event),
            }
        }
        let mut tag_names: Vec<String> = by_tag.iter().map(|(k, _)| k.clone()).collect();
        if !tag_names.iter().all(|k| collation_safe(k)) {
            return Ok(None);
        }
        tag_names.sort_by(|a, b| locale_cmp(a, b));
        for tag in &tag_names {
            scope_lines.push(String::new());
            scope_lines.push(format!("### {tag}"));
            scope_lines.push(String::new());
            for event in &by_tag.iter().find(|(k, _)| k == tag).unwrap().1 {
                scope_lines.push(format_index_line(event));
                count += 1;
            }
        }
        if !untagged.is_empty() {
            scope_lines.push(String::new());
            scope_lines.push("### untagged".to_string());
            scope_lines.push(String::new());
            for event in &untagged {
                scope_lines.push(format_index_line(event));
                count += 1;
            }
        }
        blocks.push(scope_lines.join("\n"));
    }

    let body = if blocks.is_empty() {
        "No active decisions.".to_string()
    } else {
        blocks.join("\n\n")
    };
    Ok(Some((body, count)))
}

pub(crate) fn decision_index_content(root: &Path, all: bool) -> Ex<Option<(String, usize)>> {
    Ok(build_decision_index_body(root, all)?
        .map(|(body, count)| (format!("{DECISION_INDEX_HEADER}\n\n{body}\n"), count)))
}

// ─── decisions render ──────────────────────────────────────────────────────

pub(crate) fn run_render(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["all", "check"]) {
        return None;
    }
    let all = bool_flag_present(&flags, "all")?;
    let check = bool_flag_present(&flags, "check")?;

    let ctx = match crate::verbs::reservations::prelude("decisions render", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let out = do_render(&ctx.root, all, check);
    finish(&ctx, out)
}

/// handleDecisionsRender + renderDecisionIndex/decisionIndexDrift.
/// `--check`'s drift refusal is deterministic (path + fixed wording, no V8
/// text), so it is reproduced natively rather than delegated.
pub(crate) fn do_render(root: &Path, all: bool, check: bool) -> R2<Out> {
    let Some((content, count)) = decision_index_content(root, all)? else {
        return Err(Err2::Ex); // collation outside the calibrated alphabet
    };
    let file = decision_index_path(root);
    let rel = path_relative(root, &file);
    if check {
        let on_disk = std::fs::read(&file).ok().map(|b| String::from_utf8_lossy(&b).into_owned());
        let drift = on_disk.as_deref() != Some(content.as_str());
        if drift {
            return Ok(Out::Thrown(format!(
                "decisions render --check: {rel} is out of date — run `bee decisions render` to regenerate (never hand-edit it)."
            )));
        }
        return Ok(Out::Emit(
            json!({ "drift": false, "path": rel }),
            format!("{rel} is up to date."),
            0,
        ));
    }
    write_text_atomic(&file, &content).map_err(|_| Err2::Ex)?;
    let text = format!("Wrote {rel} ({count} decision(s)).");
    Ok(Out::Emit(
        json!({ "path": rel, "content": content, "count": count as f64 }),
        text,
        0,
    ))
}

/// Node path.relative for the only shape this file needs (file under root).
pub(crate) fn path_relative(root: &Path, file: &Path) -> String {
    match file.strip_prefix(root) {
        Ok(rel) => rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(std::path::MAIN_SEPARATOR_STR),
        Err(_) => file.display().to_string(),
    }
}

// ─── decisions.mjs sweepDecisionCitations ──────────────────────────────────

pub(crate) const SWEEP_EXCERPT_MAX: usize = 160;

/// Node path.extname over a bare basename (no separator, no drive prefix) —
/// ported loop-for-loop from lib/path (win32 extname).
pub(crate) fn extname(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut start_dot: isize = -1;
    let start_part: isize = 0;
    let mut end: isize = -1;
    let mut pre_dot_state: i32 = 0;
    let mut i: isize = chars.len() as isize - 1;
    while i >= 0 {
        let c = chars[i as usize];
        if end == -1 {
            end = i + 1;
        }
        if c == '.' {
            if start_dot == -1 {
                start_dot = i;
            } else if pre_dot_state != 1 {
                pre_dot_state = 1;
            }
        } else if start_dot != -1 {
            pre_dot_state = -1;
        }
        i -= 1;
    }
    if start_dot == -1
        || end == -1
        || pre_dot_state == 0
        || (pre_dot_state == 1 && start_dot == end - 1 && start_dot == start_part + 1)
    {
        return String::new();
    }
    chars[start_dot as usize..end as usize].iter().collect()
}

pub(crate) fn is_sweep_text_ext(name: &str) -> bool {
    matches!(
        extname(name).to_lowercase().as_str(),
        ".md" | ".json" | ".yaml" | ".yml" | ".txt"
    )
}

/// collectSweepFiles: readdirSync(withFileTypes) order, depth-first, symlinks
/// skipped (a Dirent symlink is neither isDirectory() nor isFile()).
pub(crate) fn collect_sweep_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else { continue };
        let full = entry.path();
        if ft.is_dir() {
            collect_sweep_files(&full, out);
        } else if ft.is_file() && is_sweep_text_ext(&entry.file_name().to_string_lossy()) {
            out.push(full);
        }
    }
}

pub(crate) fn is_re_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// `new RegExp('\\b' + escapeRegExp(needle) + '\\b', 'i').test(line)` for an
/// ASCII `needle` (escapeRegExp neutralizes every metacharacter, so the body
/// is a literal). ASCII-only is enforced by the caller — V8's `i`-flag
/// Canonicalize is not provably ASCII-folding outside it.
pub(crate) fn word_bounded_ci_test(line: &str, needle: &str) -> bool {
    let l: Vec<char> = line.chars().collect();
    let n: Vec<char> = needle.chars().collect();
    if n.is_empty() {
        return true;
    }
    if n.len() > l.len() {
        return false;
    }
    for start in 0..=(l.len() - n.len()) {
        let matched = (0..n.len())
            .all(|k| l[start + k].to_ascii_lowercase() == n[k].to_ascii_lowercase());
        if !matched {
            continue;
        }
        let end = start + n.len();
        let before_word = start > 0 && is_re_word(l[start - 1]);
        let at_start_word = is_re_word(l[start]);
        if before_word == at_start_word {
            continue; // no \b at the left edge
        }
        let last_word = is_re_word(l[end - 1]);
        let after_word = end < l.len() && is_re_word(l[end]);
        if last_word == after_word {
            continue; // no \b at the right edge
        }
        return true;
    }
    false
}

/// sweepDecisionCitations — read-only docs/** scan. Returns
/// {scanned_at, hit_count, files[]}.
pub(crate) fn sweep_decision_citations(root: &Path, id: &str, short8: &str) -> Value {
    let mut candidates: Vec<PathBuf> = Vec::new();
    collect_sweep_files(&root.join("docs"), &mut candidates);
    let mut files: Vec<Value> = Vec::new();
    for file in &candidates {
        let Ok(bytes) = std::fs::read(file) else { continue };
        let text = String::from_utf8_lossy(&bytes);
        for (index, line) in split_crlf_lines(&text).into_iter().enumerate() {
            if word_bounded_ci_test(line, id) || word_bounded_ci_test(line, short8) {
                let trimmed = js_trim(line);
                let excerpt = if char_len(trimmed) > SWEEP_EXCERPT_MAX {
                    format!("{}...", truncate_chars_head(trimmed, SWEEP_EXCERPT_MAX - 3))
                } else {
                    trimmed.to_string()
                };
                files.push(json!({
                    "file": path_relative(root, file),
                    "line": (index + 1) as f64,
                    "excerpt": excerpt,
                }));
            }
        }
    }
    json!({
        "scanned_at": now_iso(),
        "hit_count": files.len() as f64,
        "files": files,
    })
}

/// `text.split(/\r?\n/)`
pub(crate) fn split_crlf_lines(text: &str) -> Vec<&str> {
    text.split('\n')
        .map(|seg| seg.strip_suffix('\r').unwrap_or(seg))
        .collect()
}
