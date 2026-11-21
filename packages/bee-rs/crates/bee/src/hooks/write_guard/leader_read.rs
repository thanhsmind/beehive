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

fn cat_operands(command: &str) -> Option<(Option<String>, Vec<String>)> {
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
    let files = parts
        .map(|file| {
            if file.starts_with('-') {
                return None;
            }
            Some(unquote(file)?.to_string())
        })
        .collect::<Option<Vec<_>>>()?;
    if files.is_empty() {
        return None;
    }
    Some((cd_dir, files))
}

pub(crate) fn unranged_target(
    tool_name: &str,
    tool_input: &Map<String, Value>,
    cwd: &Path,
) -> Option<Vec<PathBuf>> {
    match tool_name {
        "Read" => {
            if tool_input.contains_key("offset") || tool_input.contains_key("limit") {
                return None;
            }
            let raw = ["file_path", "path"]
                .iter()
                .filter_map(|k| tool_input.get(*k).and_then(Value::as_str))
                .find(|s| !s.is_empty())?;
            Some(vec![cwd.join(raw)])
        }
        "Bash" | "exec" => {
            let command = ["command", "cmd"]
                .iter()
                .filter_map(|k| tool_input.get(*k).and_then(Value::as_str))
                .find(|s| !s.is_empty())?;
            let (dir, files) = cat_operands(command)?;
            let base = match dir {
                Some(d) => cwd.join(d),
                None => cwd.to_path_buf(),
            };
            Some(files.iter().map(|file| base.join(file)).collect())
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
    mut line_count: impl FnMut(&Path) -> Option<usize>,
    limit: u64,
) -> Option<String> {
    if worker || limit == 0 {
        return None;
    }
    let targets = unranged_target(tool_name, tool_input, cwd)?;
    let mut lines = 0usize;
    for target in &targets {
        lines += line_count(target)?;
    }
    if (lines as u64) <= limit {
        return None;
    }
    let subject = match targets.as_slice() {
        [one] => format!("\"{}\" is {lines} lines", one.display()),
        many => format!(
            "{} are {lines} lines together",
            many.iter()
                .map(|t| format!("\"{}\"", t.display()))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    Some(format!(
        "bee leader read guard: {subject} (leader_read_max_lines: {limit}) and this read has no range, so the whole content would enter the leader context and be re-read on every later call. \
FIX: read a range with offset and limit; or dispatch a read gather: .bee/bin/bee dispatch prepare --runtime claude --kind gather --role read --json"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn input(value: Value) -> Map<String, Value> {
        value.as_object().cloned().unwrap()
    }

    fn file_with(dir: &Path, name: &str, lines: usize) {
        std::fs::write(dir.join(name), "x\n".repeat(lines)).unwrap();
    }

    fn refusal(tool: &str, value: Value, dir: &Path, worker: bool, limit: u64) -> Option<String> {
        leader_read_refusal(tool, &input(value), dir, worker, count_lines, limit)
    }

    #[test]
    fn leader_read_cap_refuses_unranged_read_of_401_lines_naming_both_remedies() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "long.txt", 401);
        let text = refusal("Read", json!({"file_path": "long.txt"}), dir.path(), false, 400)
            .expect("401 lines must be refused");
        assert!(text.contains("offset and limit"), "{text}");
        assert!(text.contains("bee dispatch prepare"), "{text}");
    }

    #[test]
    fn leader_read_cap_passes_400_lines() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "edge.txt", 400);
        assert!(refusal("Read", json!({"file_path": "edge.txt"}), dir.path(), false, 400).is_none());
    }

    #[test]
    fn leader_read_cap_passes_ranged_read() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "long.txt", 401);
        for ranged in [
            json!({"file_path": "long.txt", "offset": 1, "limit": 50}),
            json!({"file_path": "long.txt", "limit": 50}),
            json!({"file_path": "long.txt", "offset": 10}),
        ] {
            assert!(refusal("Read", ranged, dir.path(), false, 400).is_none());
        }
    }

    #[test]
    fn leader_read_cap_passes_worker_session() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "long.txt", 401);
        assert!(refusal("Read", json!({"file_path": "long.txt"}), dir.path(), true, 400).is_none());
        assert!(refusal("Bash", json!({"command": "cat long.txt"}), dir.path(), true, 400).is_none());
    }

    #[test]
    fn leader_read_cap_limit_zero_passes_every_size() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "huge.txt", 5000);
        assert!(refusal("Read", json!({"file_path": "huge.txt"}), dir.path(), false, 0).is_none());
        assert!(refusal("Bash", json!({"command": "cat huge.txt"}), dir.path(), false, 0).is_none());
    }

    #[test]
    fn leader_read_cap_refuses_bare_cat_of_401_lines() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "long.txt", 401);
        let text = refusal("Bash", json!({"command": "cat long.txt"}), dir.path(), false, 400)
            .expect("bare cat must be refused");
        assert!(text.contains("offset and limit"), "{text}");
        assert!(text.contains("bee dispatch prepare"), "{text}");
    }

    #[test]
    fn leader_read_cap_passes_piped_or_ranged_cat() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "long.txt", 401);
        for command in ["cat long.txt | head", "cat long.txt | head -n 20", "head -n 20 long.txt"] {
            assert!(
                refusal("Bash", json!({"command": command}), dir.path(), false, 400).is_none(),
                "{command}"
            );
        }
    }

    #[test]
    fn leader_read_multi_cat_refuses_sum_past_limit_naming_both_files() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "a.txt", 250);
        file_with(dir.path(), "b.txt", 151);
        let text = refusal("Bash", json!({"command": "cat a.txt b.txt"}), dir.path(), false, 400)
            .expect("401 summed lines must be refused");
        assert!(text.contains("a.txt"), "{text}");
        assert!(text.contains("b.txt"), "{text}");
        assert!(text.contains("401 lines together"), "{text}");
    }

    #[test]
    fn leader_read_multi_cat_passes_sum_at_limit() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "a.txt", 200);
        file_with(dir.path(), "b.txt", 200);
        assert!(refusal("Bash", json!({"command": "cat a.txt b.txt"}), dir.path(), false, 400).is_none());
    }

    #[test]
    fn leader_read_multi_cat_passes_worker_session() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "a.txt", 250);
        file_with(dir.path(), "b.txt", 151);
        assert!(refusal("Bash", json!({"command": "cat a.txt b.txt"}), dir.path(), true, 400).is_none());
    }

    #[test]
    fn leader_read_multi_cat_limit_zero_passes() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "a.txt", 250);
        file_with(dir.path(), "b.txt", 151);
        assert!(refusal("Bash", json!({"command": "cat a.txt b.txt"}), dir.path(), false, 0).is_none());
    }

    #[test]
    fn leader_read_multi_cat_piped_passes() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "a.txt", 250);
        file_with(dir.path(), "b.txt", 151);
        assert!(
            refusal("Bash", json!({"command": "cat a.txt b.txt | head"}), dir.path(), false, 400).is_none()
        );
    }

    #[test]
    fn leader_read_multi_cat_single_file_boundary_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        file_with(dir.path(), "long.txt", 401);
        file_with(dir.path(), "edge.txt", 400);
        assert!(refusal("Bash", json!({"command": "cat long.txt"}), dir.path(), false, 400).is_some());
        assert!(refusal("Bash", json!({"command": "cat edge.txt"}), dir.path(), false, 400).is_none());
    }

    #[test]
    fn leader_read_cap_config_default_and_override() {
        assert_eq!(resolve_leader_read_max_lines(&Map::new()), 400);
        assert_eq!(resolve_leader_read_max_lines(&input(json!({"leader_read_max_lines": 0}))), 0);
        assert_eq!(resolve_leader_read_max_lines(&input(json!({"leader_read_max_lines": 50}))), 50);
    }
}
