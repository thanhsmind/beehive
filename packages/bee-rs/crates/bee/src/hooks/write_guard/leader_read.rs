use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

const DEFAULT_LIMIT: u64 = 400;
const MAX_BYTES: u64 = 25 * 1024 * 1024;

pub(crate) fn resolve_leader_read_max_lines(config: &Map<String, Value>) -> u64 {
    match config.get("leader_read_max_lines") {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(DEFAULT_LIMIT),
        _ => DEFAULT_LIMIT,
    }
}

pub(crate) fn is_worker_session() -> bool {
    crate::hooks::herding_worker_marker_set()
        || std::env::var("PASEO_AGENT_ID").is_ok_and(|v| !v.is_empty())
}

fn unquote(token: &str) -> Option<&str> {
    let bytes = token.as_bytes();
    let inner = match (bytes.first(), bytes.last()) {
        (Some(b'"'), Some(b'"')) | (Some(b'\''), Some(b'\'')) if bytes.len() >= 2 => {
            &token[1..token.len() - 1]
        }
        _ => token,
    };
    if inner.is_empty() || inner.contains(['"', '\'', '\\']) {
        return None;
    }
    Some(inner)
}

fn plain_command(text: &str) -> bool {
    !text.contains(['|', ';', '&', '<', '>', '$', '`', '(', ')', '\n', '*', '?', '[', '{', '~'])
}

fn cat_operand(command: &str) -> Option<(Option<String>, String)> {
    let command = command.trim();
    let (cd_dir, rest) = match command.strip_prefix("cd ") {
        Some(after) => {
            let (dir, rest) = after.split_once("&&")?;
            if !plain_command(dir) {
                return None;
            }
            let dir = unquote(dir.trim())?;
            (Some(dir.to_string()), rest.trim())
        }
        None => (None, command),
    };
    if !plain_command(rest) {
        return None;
    }
    let mut parts = rest.split_whitespace();
    if parts.next()? != "cat" {
        return None;
    }
    let file = parts.next()?;
    if parts.next().is_some() || file.starts_with('-') {
        return None;
    }
    Some((cd_dir, unquote(file)?.to_string()))
}

pub(crate) fn unranged_target(
    tool_name: &str,
    tool_input: &Map<String, Value>,
    cwd: &Path,
) -> Option<PathBuf> {
    match tool_name {
        "Read" => {
            if tool_input.contains_key("offset") || tool_input.contains_key("limit") {
                return None;
            }
            let raw = ["file_path", "path"]
                .iter()
                .filter_map(|k| tool_input.get(*k).and_then(Value::as_str))
                .find(|s| !s.is_empty())?;
            Some(cwd.join(raw))
        }
        "Bash" | "exec" => {
            let command = ["command", "cmd"]
                .iter()
                .filter_map(|k| tool_input.get(*k).and_then(Value::as_str))
                .find(|s| !s.is_empty())?;
            let (dir, file) = cat_operand(command)?;
            let base = match dir {
                Some(d) => cwd.join(d),
                None => cwd.to_path_buf(),
            };
            Some(base.join(file))
        }
        _ => None,
    }
}

pub(crate) fn count_lines(path: &Path) -> Option<usize> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    if bytes.iter().take(8000).any(|&b| b == 0) {
        return None;
    }
    let newlines = bytes.iter().filter(|&&b| b == b'\n').count();
    Some(match bytes.last() {
        Some(&last) if last != b'\n' => newlines + 1,
        _ => newlines,
    })
}

pub(crate) fn leader_read_refusal(
    tool_name: &str,
    tool_input: &Map<String, Value>,
    cwd: &Path,
    worker: bool,
    line_count: impl FnOnce(&Path) -> Option<usize>,
    limit: u64,
) -> Option<String> {
    if worker || limit == 0 {
        return None;
    }
    let target = unranged_target(tool_name, tool_input, cwd)?;
    let lines = line_count(&target)?;
    if (lines as u64) <= limit {
        return None;
    }
    Some(format!(
        "bee leader read guard: \"{}\" is {lines} lines (leader_read_max_lines: {limit}) and this read has no range, so the whole file would enter the leader context and be re-read on every later call. \
FIX: read a range with offset and limit; or dispatch a read gather: .bee/bin/bee dispatch prepare --runtime claude --kind gather --role read --json",
        target.display()
    ))
}
