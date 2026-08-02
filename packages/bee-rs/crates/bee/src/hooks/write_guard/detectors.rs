// apply_patch extraction, agent-name inference, AskUserQuestion and the delegation detectors
//
// Split out of the single 5.9k-line hooks/write_guard.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::hooks::adapter::{append_hook_log, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson;
use crate::state::hook_enabled;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ─── apply_patch extraction (provenance: bee-write-guard.mjs codex-parity) ─

pub(crate) fn apply_patch_text(tool_input: &Map<String, Value>) -> Option<String> {
    for key in ["input", "patch", "command"] {
        if let Some(Value::String(s)) = tool_input.get(key) {
            if s.contains("*** Begin Patch") {
                return Some(s.clone());
            }
        }
    }
    None
}

/// provenance: bee-write-guard.mjs PATCH_TARGET_RE + extractApplyPatchTargets.
pub(crate) fn extract_apply_patch_targets(patch_text: &str) -> Vec<String> {
    let mut targets = Vec::new();
    for line in patch_text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let Some(after_stars) = line.strip_prefix("***") else { continue };
        let after_ws = after_stars.trim_start_matches(js_is_ws);
        if after_ws.len() == after_stars.len() {
            continue; // \s+ requires at least one whitespace char
        }
        let mut matched = None;
        for verb in ["Add File", "Update File", "Delete File", "Move to"] {
            if let Some(rest) = after_ws.strip_prefix(verb) {
                if let Some(rest) = rest.strip_prefix(':') {
                    matched = Some(rest);
                    break;
                }
            }
        }
        let Some(rest) = matched else { continue };
        if rest.is_empty() {
            continue; // `\s*(.+?)\s*$` needs at least one char after ':'
        }
        targets.push(js_trim(rest).to_string());
    }
    targets
}

// ─── agent-name inference (provenance: bee-write-guard.mjs inferAgentName) ─

pub(crate) fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// /\bBEE_AGENT_NAME=(["']?)([^"'\s]+)\1/ hand-rolled.
pub(crate) fn agent_name_from_command(command: &str) -> Option<String> {
    const NEEDLE: &str = "BEE_AGENT_NAME=";
    let chars: Vec<char> = command.chars().collect();
    let n: Vec<char> = NEEDLE.chars().collect();
    let mut i = 0usize;
    while i + n.len() <= chars.len() {
        if chars[i..i + n.len()] != n[..] {
            i += 1;
            continue;
        }
        // \b before 'B'
        if i > 0 && is_word_char(chars[i - 1]) {
            i += 1;
            continue;
        }
        let mut j = i + n.len();
        let quote = match chars.get(j) {
            Some(&q @ ('"' | '\'')) => {
                j += 1;
                Some(q)
            }
            _ => None,
        };
        let start = j;
        while j < chars.len() {
            let c = chars[j];
            if c == '"' || c == '\'' || js_is_ws(c) {
                break;
            }
            j += 1;
        }
        if j > start {
            let ok = match quote {
                Some(q) => chars.get(j) == Some(&q), // backreference
                None => true,                        // empty \1 always matches
            };
            if ok {
                return Some(chars[start..j].iter().collect());
            }
        }
        i += 1;
    }
    None
}

pub(crate) fn infer_agent_name(payload: &Map<String, Value>, tool_input: &Map<String, Value>) -> Option<String> {
    for key in ["agent_name", "agentName", "agent_nickname", "subagent_type"] {
        if let Some(s) = str_trim_nonempty(payload.get(key)) {
            return Some(s);
        }
    }
    let command = match tool_input.get("command") {
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    };
    if let Some(m) = agent_name_from_command(&command) {
        return Some(m);
    }
    match std::env::var("BEE_AGENT_NAME") {
        Ok(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

// ─── AskUserQuestion (provenance: guards.mjs checkAskUserQuestion +
// bee-write-guard.mjs ask-guard-autofix emission) ──────────────────────────

pub(crate) enum AskResult {
    Allow,
    Deny(String),
    Fixed { fixed: Value, notes: Vec<String> },
}

pub(crate) fn check_ask_user_question(tool_input: &Map<String, Value>) -> R<AskResult> {
    let questions = match tool_input.get("questions") {
        Some(Value::Array(a)) => a.clone(),
        _ => return Ok(AskResult::Allow),
    };
    let n = questions.len();
    if n < 1 || n > 4 {
        return Ok(AskResult::Deny(format!(
            "bee AskUserQuestion guard: {n} question(s) — the tool takes 1–4 per call. Split into separate calls."
        )));
    }
    let mut fixes: Vec<(usize, String)> = Vec::new();
    for (i, q) in questions.iter().enumerate() {
        let q = match q {
            Value::Object(m) => m,
            _ => continue,
        };
        let where_ = if n > 1 { format!(" (question {})", i + 1) } else { String::new() };
        if let Some(Value::String(h)) = q.get("header") {
            if utf16_len(h) > 12 {
                fixes.push((i, h.clone()));
            }
        }
        if let Some(Value::Array(options)) = q.get("options") {
            let on = options.len();
            if on < 2 || on > 4 {
                return Ok(AskResult::Deny(format!(
                    "bee AskUserQuestion guard: {on} option(s){where_} — each question needs 2–4 options (an \"Other\" free-text choice is added automatically). Fold overflow into a follow-up question."
                )));
            }
            for (j, o) in options.iter().enumerate() {
                let o = match o {
                    Value::Object(m) => m,
                    _ => continue,
                };
                let label = match o.get("label") {
                    Some(Value::String(l)) if !js_trim(l).is_empty() => l.clone(),
                    _ => {
                        return Ok(AskResult::Deny(format!(
                            "bee AskUserQuestion guard: option {}{} is missing a non-empty \"label\". Every option needs a label and a description.",
                            j + 1,
                            where_
                        )));
                    }
                };
                match o.get("description") {
                    Some(Value::String(d)) if !js_trim(d).is_empty() => {}
                    _ => {
                        return Ok(AskResult::Deny(format!(
                            "bee AskUserQuestion guard: option \"{label}\"{where_} is missing a non-empty \"description\". Every option needs a label and a description."
                        )));
                    }
                }
            }
        }
    }
    if fixes.is_empty() {
        return Ok(AskResult::Allow);
    }
    let mut fixed = Value::Object(tool_input.clone());
    let mut notes = Vec::new();
    for (idx, old) in fixes {
        if !old.is_ascii() {
            // UTF-16 slice(0,11)+trimEnd parity for non-ASCII headers is
            // unproven (surrogate-pair splits) — delegate.
            return Err(Nd);
        }
        let truncated = js_trim_end(&old[..11]);
        let new_header = format!("{truncated}…");
        fixed["questions"][idx]["header"] = Value::String(new_header.clone());
        notes.push(format!(
            "header \"{}\" ({} chars) → \"{}\"",
            old,
            utf16_len(&old),
            new_header
        ));
    }
    Ok(AskResult::Fixed { fixed, notes })
}

// ─── CLI-shape / internals-reach delegation detectors ──────────────────────

/// True when a shell segment invokes node/nodejs with an inline-eval script
/// (-e/--eval/-p or --eval=…) — checkBinLibImportBashCommand's regex scan is
/// not ported → the caller delegates.
pub(crate) fn has_node_inline_eval(command: &str) -> bool {
    let tokens = tokenize(command);
    let mut i = 0usize;
    while i < tokens.len() {
        if is_separator(&tokens[i]) {
            i += 1;
            continue;
        }
        let mut end = i;
        while end < tokens.len() && !is_separator(&tokens[end]) {
            end += 1;
        }
        let segment = &tokens[i..end];
        i = end;
        let cmd = segment
            .first()
            .map(|t| t.replace('\\', "/"))
            .unwrap_or_default();
        let cmd = cmd.rsplit('/').next().unwrap_or("");
        if cmd != "node" && cmd != "nodejs" {
            continue;
        }
        for (k, token) in segment.iter().enumerate() {
            if matches!(token.as_str(), "-e" | "--eval" | "-p") {
                if segment.get(k + 1).is_some() {
                    return true;
                }
            }
            if token.starts_with("--eval=") {
                return true;
            }
        }
    }
    false
}

// ─── buffered emission ─────────────────────────────────────────────────────

#[derive(Default)]
pub(crate) struct Emit {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) code: u8,
    /// Deferred coverage-gap log lines: (root, gap, detail, ts).
    pub(crate) gaps: Vec<(PathBuf, &'static str, String, String)>,
}

pub(crate) fn push_gap(emit: &mut Emit, root: &Path, gap: &'static str, detail: String) {
    emit.gaps.push((root.to_path_buf(), gap, detail, now_iso()));
}

pub(crate) const DETAIL_MAX: usize = 300;

pub(crate) fn flush(emit: Emit, source: Option<&str>) -> Outcome {
    for (root, gap, detail, ts) in &emit.gaps {
        // Same field order as adapter.mjs logCoverageGap: ts, hook, event,
        // gap, detail, source?.
        let truncated: String = if detail.chars().count() <= DETAIL_MAX {
            detail.clone()
        } else {
            format!("{}...", detail.chars().take(DETAIL_MAX).collect::<String>())
        };
        let mut entry = Map::new();
        entry.insert("ts".into(), Value::String(ts.clone()));
        entry.insert("hook".into(), Value::String(HOOK_NAME.to_string()));
        entry.insert("event".into(), Value::String("coverage-gap".to_string()));
        entry.insert("gap".into(), Value::String(gap.to_string()));
        entry.insert("detail".into(), Value::String(truncated));
        if let Some(s) = source {
            entry.insert("source".into(), Value::String(s.to_string()));
        }
        append_hook_log(root, &Value::Object(entry));
    }
    use std::io::Write;
    if !emit.stdout.is_empty() {
        let _ = std::io::stdout().write_all(emit.stdout.as_bytes());
    }
    // Corrupt-JSON warnings precede everything the evaluation itself wrote,
    // matching where Node's readJson emitted them.
    let corrupt = take_corrupt_json_warnings();
    if !corrupt.is_empty() {
        let _ = std::io::stderr().write_all(corrupt.as_bytes());
    }
    if !emit.stderr.is_empty() {
        let _ = std::io::stderr().write_all(emit.stderr.as_bytes());
    }
    Outcome::Done(ExitCode::from(emit.code))
}
