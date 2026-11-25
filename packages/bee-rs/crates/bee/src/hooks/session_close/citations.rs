use super::claims::{inline_numbers, is_sources_heading, last_turn_assistant_text, leading_number};
use serde_json::Value;
use std::path::Path;

pub(crate) fn citation_source_warning(root: &Path, events: &[Value]) -> Option<String> {
    let text = last_turn_assistant_text(events);
    if text.is_empty() {
        return None;
    }

    let mut in_sources = false;
    let mut reply_lines: Vec<&str> = Vec::new();
    let mut source_lines: Vec<(&str, &str)> = Vec::new();

    for line in text.lines() {
        if in_sources {
            if let Some(n) = leading_number(line) {
                let trimmed = line.trim();
                let rest = trimmed[n.len() + 2..].trim();
                source_lines.push((n, rest));
                continue;
            }
        } else if is_sources_heading(line) {
            in_sources = true;
            continue;
        }
        reply_lines.push(line);
    }

    if source_lines.is_empty() {
        return None;
    }

    let commands = collect_shell_commands(events);
    let mut failures: Vec<(&str, String, String)> = Vec::new();

    for (n, rest) in source_lines {
        if rest.contains("http://") || rest.contains("https://") {
            continue;
        }

        if let Some(tok) = find_path_token(rest) {
            if let Some(reason) = check_path_source(root, n, rest, &reply_lines, &tok) {
                failures.push((n, tok.label, reason));
            }
            continue;
        }

        if let Some(cmd) = find_command_source(rest)
            && !command_was_run(&cmd, &commands)
        {
            failures.push((n, cmd, "command not run".to_string()));
        }
    }

    if failures.is_empty() {
        return None;
    }

    let count = failures.len();
    let shown: Vec<String> = failures
        .into_iter()
        .take(3)
        .map(|(n, source, reason)| format!("[{n}] {source}: {reason}"))
        .collect();

    Some(format!(
        "bee: {count} cited source(s) did not check out — rule agents-verify-before-agreeing: {}",
        shown.join("; ")
    ))
}

struct PathToken {
    label: String,
    path: String,
    range: Option<(usize, usize)>,
}

fn find_path_token(rest: &str) -> Option<PathToken> {
    if let Some(after) = rest.strip_prefix('`')
        && let Some(end) = after.find('`')
        && after[..end].chars().any(char::is_whitespace)
    {
        return None;
    }
    let word = rest.split_whitespace().next()?;
    let cleaned = word
        .trim_matches('`')
        .trim_end_matches([',', '.', ';', ')'])
        .trim_matches('`');
    if cleaned.is_empty() {
        return None;
    }
    let (path_part, range) = split_line_range(cleaned);
    if looks_like_path(path_part) {
        return Some(PathToken {
            label: cleaned.to_string(),
            path: path_part.to_string(),
            range,
        });
    }
    None
}

fn split_line_range(token: &str) -> (&str, Option<(usize, usize)>) {
    let Some((left, right)) = token.rsplit_once(':') else {
        return (token, None);
    };
    if left.is_empty() || right.is_empty() {
        return (token, None);
    }
    if let Some((a, b)) = right.split_once('-') {
        if !a.is_empty()
            && !b.is_empty()
            && a.bytes().all(|c| c.is_ascii_digit())
            && b.bytes().all(|c| c.is_ascii_digit())
            && let (Ok(start), Ok(end)) = (a.parse::<usize>(), b.parse::<usize>())
        {
            return (left, Some((start, end)));
        }
    } else if right.bytes().all(|c| c.is_ascii_digit())
        && let Ok(n) = right.parse::<usize>()
    {
        return (left, Some((n, n)));
    }
    (token, None)
}

fn looks_like_path(s: &str) -> bool {
    if s.is_empty() || s == "/" {
        return false;
    }
    if s.contains('/') {
        return true;
    }
    Path::new(s)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| {
            !e.is_empty()
                && e.chars().all(|c| c.is_ascii_alphanumeric())
                && e.chars().any(|c| c.is_ascii_alphabetic())
        })
}

fn check_path_source(
    root: &Path,
    n: &str,
    rest: &str,
    reply_lines: &[&str],
    tok: &PathToken,
) -> Option<String> {
    let path = Path::new(&tok.path);
    let full_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let Ok(bytes) = std::fs::read(&full_path) else {
        return Some("file not found".to_string());
    };
    let content = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len();

    let window = if let Some((start, end)) = tok.range {
        if start == 0 || start > line_count {
            return Some(format!("line {start} past end (file has {line_count} lines)"));
        }
        if end == 0 || end > line_count {
            return Some(format!("line {end} past end (file has {line_count} lines)"));
        }
        let (lo, hi) = (start.min(end), start.max(end));
        let win_start = lo.saturating_sub(21);
        let win_end = hi.saturating_add(20).min(line_count);
        &lines[win_start..win_end]
    } else {
        &lines[..]
    };

    let tokens = collect_tokens(n, rest, reply_lines, &tok.label, &tok.path);
    if tokens.is_empty() {
        return None;
    }

    let found = tokens.iter().any(|t| {
        let lower = t.to_lowercase();
        window.iter().any(|line| line.to_lowercase().contains(&lower))
    });

    if found {
        None
    } else if let Some((start, _)) = tok.range {
        Some(format!("text not found near line {start}: {}", tokens[0]))
    } else {
        Some(format!("text not found: {}", tokens[0]))
    }
}

fn collect_tokens(
    n: &str,
    rest: &str,
    reply_lines: &[&str],
    source_label: &str,
    path_part: &str,
) -> Vec<String> {
    let mut tokens = Vec::new();
    push_backtick_tokens(rest, source_label, path_part, &mut tokens);
    for line in reply_lines {
        if inline_numbers(line).contains(&n) {
            push_backtick_tokens(line, source_label, path_part, &mut tokens);
        }
    }
    tokens
}

fn push_backtick_tokens(text: &str, source_label: &str, path_part: &str, out: &mut Vec<String>) {
    let mut rest = text;
    while let Some(start) = rest.find('`') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('`') else {
            break;
        };
        let span = &after_start[..end];
        if span.chars().count() >= 3 && span != source_label && span != path_part {
            out.push(span.to_string());
        }
        rest = &after_start[end + 1..];
    }
}

fn find_command_source(rest: &str) -> Option<String> {
    if let Some(start) = rest.find('`') {
        let after_start = &rest[start + 1..];
        if let Some(end) = after_start.find('`') {
            let span = after_start[..end].trim();
            if !span.is_empty() {
                return Some(span.to_string());
            }
        }
    }
    if let Some((before, _)) = rest.split_once(" — ") {
        let cmd = before.trim();
        if !cmd.is_empty() {
            return Some(cmd.to_string());
        }
    }
    None
}

fn collect_shell_commands(events: &[Value]) -> Vec<String> {
    let mut commands = Vec::new();
    for event in events {
        match event.get("type").and_then(Value::as_str).unwrap_or("") {
            "assistant" => {
                let blocks = event.get("message").and_then(|m| m.get("content")).and_then(Value::as_array);
                for block in blocks.into_iter().flatten() {
                    if block.get("type").and_then(Value::as_str) == Some("tool_use")
                        && let Some(cmd) = block.get("input").and_then(|i| i.get("command")).and_then(Value::as_str)
                    {
                        commands.push(cmd.to_string());
                    }
                }
            }
            "response_item" => {
                let Some(payload) = event.get("payload") else { continue };
                if payload.get("type").and_then(Value::as_str) != Some("function_call") {
                    continue;
                }
                let Some(args_str) = payload.get("arguments").and_then(Value::as_str) else { continue };
                let Ok(args) = serde_json::from_str::<Value>(args_str) else { continue };
                match args.get("cmd").or_else(|| args.get("command")) {
                    Some(Value::String(s)) => commands.push(s.clone()),
                    Some(Value::Array(arr)) => {
                        let joined: Vec<&str> = arr.iter().filter_map(Value::as_str).collect();
                        if !joined.is_empty() {
                            commands.push(joined.join(" "));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    commands
}

fn command_was_run(cited: &str, commands: &[String]) -> bool {
    let words: Vec<&str> = cited.split_whitespace().collect();
    let Some(&first) = words.first() else {
        return true;
    };
    let rest = &words[1..];
    commands.iter().any(|ran| {
        ran.contains(first) && (rest.is_empty() || rest.iter().any(|w| ran.contains(w)))
    })
}
