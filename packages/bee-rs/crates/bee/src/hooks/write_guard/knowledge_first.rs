use super::*;
use crate::hooks::adapter::HookContext;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

const READ_VERBS: &[&str] = &["rg", "grep", "cat", "sed", "head", "tail", "fd", "find", "less", "bat"];
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "mjs", "js", "ts", "tsx", "py", "go", "sh", "java", "kt", "swift", "rb", "c", "h", "cpp",
];
const NON_CODE_PREFIXES: &[&str] = &["docs/", ".bee/", "skills/"];
const CODE_PREFIXES: &[&str] = &["packages/", "src/"];
const SEARCHED: &str = "searched";
const NUDGED: &str = "nudged";
const NUDGE: &str = "bee rule agents-knowledge-before-code: before reading code, search the knowledge store — run `.bee/bin/bee knowledge search --text \"<what you need>\"`.";

pub(crate) fn session_key(payload: &Map<String, Value>) -> String {
    let sid = str_trim_nonempty(payload.get("session_id")).unwrap_or_else(|| "default".to_string());
    sid.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

fn turn_state_path(store_root: &Path, key: &str) -> PathBuf {
    store_root.join(".bee").join("cache").join(format!("knowledge-first-{key}.json"))
}

pub(crate) fn reset_turn(store_root: &Path, payload: &Map<String, Value>) {
    let _ = std::fs::remove_file(turn_state_path(store_root, &session_key(payload)));
}

fn write_turn_state(path: &Path, state: &str) -> bool {
    if let Some(dir) = path.parent() {
        if std::fs::create_dir_all(dir).is_err() {
            return false;
        }
    }
    std::fs::write(path, format!("{{\"state\":\"{state}\"}}\n")).is_ok()
}

fn is_subagent(payload: &Map<String, Value>) -> bool {
    str_trim_nonempty(payload.get("agent_id")).is_some()
        || str_trim_nonempty(payload.get("agent_type")).is_some()
}

fn relative_to_root(root: &str, raw: &str) -> Option<String> {
    let mut p = raw.replace('\\', "/");
    if p.starts_with('/') {
        let base = format!("{}/", root.replace('\\', "/").trim_end_matches('/'));
        p = p.strip_prefix(&base)?.to_string();
    }
    while let Some(rest) = p.strip_prefix("./") {
        p = rest.to_string();
    }
    Some(p)
}

fn is_code_path(root: &str, raw: &str) -> bool {
    let Some(rel) = relative_to_root(root, raw) else { return false };
    if rel.ends_with(".md") || NON_CODE_PREFIXES.iter().any(|p| rel.starts_with(p)) {
        return false;
    }
    if CODE_PREFIXES.iter().any(|p| rel.starts_with(p)) {
        return true;
    }
    let name = rel.rsplit('/').next().unwrap_or("");
    name.rsplit_once('.')
        .is_some_and(|(_, ext)| SOURCE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
}

fn command_basename(token: &str) -> &str {
    token.rsplit(['/', '\\']).next().unwrap_or("")
}

fn runs_knowledge_lookup(tokens: &[String]) -> bool {
    tokens.windows(3).any(|w| {
        command_basename(&w[0]) == "bee" && w[1] == "knowledge" && (w[2] == "search" || w[2] == "context")
    })
}

fn bash_reads_code(root: &str, tokens: &[String]) -> bool {
    let mut i = 0usize;
    while i < tokens.len() {
        if !READ_VERBS.contains(&command_basename(&tokens[i])) {
            i += 1;
            continue;
        }
        i += 1;
        while i < tokens.len() && !is_separator(&tokens[i]) {
            if !tokens[i].starts_with('-') && is_code_path(root, &tokens[i]) {
                return true;
            }
            i += 1;
        }
    }
    false
}

fn tool_reads_code(root: &str, tool_name: &str, tool_input: &Map<String, Value>) -> bool {
    let mut keys = vec!["file_path", "path", "glob"];
    if tool_name == "Glob" {
        keys.push("pattern");
    }
    keys.iter()
        .filter_map(|k| str_trim_nonempty(tool_input.get(*k)))
        .any(|p| is_code_path(root, &p))
}

pub(crate) fn observe(
    ctx: &HookContext,
    root: &str,
    store_root: &Path,
    tool_name: &str,
    tool_input: &Map<String, Value>,
) -> Option<String> {
    if crate::hooks::herding_worker_marker_set() || is_subagent(&ctx.payload) {
        return None;
    }
    let state_path = turn_state_path(store_root, &session_key(&ctx.payload));
    let reads_code = match tool_name {
        "Read" | "Glob" | "Grep" => tool_reads_code(root, tool_name, tool_input),
        "Bash" | "exec" => {
            let command = match first_truthy(tool_input, &["command", "cmd"]) {
                Some(Value::String(s)) => s.clone(),
                _ => return None,
            };
            let tokens = tokenize_deep(&fence_heredocs(&command)).tokens;
            if runs_knowledge_lookup(&tokens) {
                write_turn_state(&state_path, SEARCHED);
                return None;
            }
            bash_reads_code(root, &tokens)
        }
        _ => false,
    };
    if !reads_code || state_path.exists() {
        return None;
    }
    write_turn_state(&state_path, NUDGED).then(|| NUDGE.to_string())
}
