use serde_json::{Map, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

const HERDING_COMMAND: &str = "bee herding run";

pub(crate) struct Spawn {
    exit: Option<i32>,
    stdout: String,
}

pub(crate) type Runner<'a> = &'a (dyn Fn(&str, &str, &Path) -> Spawn + Sync);

pub(crate) fn run_wave(root: &Path, items: &[(String, Value)], skipped: &[Value]) -> (Value, u8) {
    run_wave_with(root, items, skipped, &shell_runner)
}

pub(crate) fn run_wave_with(
    root: &Path,
    items: &[(String, Value)],
    skipped: &[Value],
    runner: Runner<'_>,
) -> (Value, u8) {
    let mut herding: Vec<(&str, &Value)> = Vec::new();
    let mut native: Vec<Value> = Vec::new();
    for (id, item) in items {
        if is_herding(item) {
            herding.push((id.as_str(), item));
        } else {
            native.push(item.clone());
        }
    }

    let spawned: Vec<Spawn> = std::thread::scope(|scope| {
        let handles: Vec<_> = herding
            .iter()
            .map(|(_, item)| {
                let command = payload_text(item, "command");
                let stdin = payload_text(item, "stdin");
                scope.spawn(move || runner(command, stdin, root))
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| Spawn { exit: None, stdout: String::new() })
            })
            .collect()
    });

    let mut rows: Vec<Value> = Vec::new();
    for ((id, _), spawn) in herding.iter().zip(spawned.iter()) {
        let path = envelope_path(root, id);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, spawn.stdout.as_bytes());
        rows.push(row(id, spawn, &path));
    }

    let all_done = rows
        .iter()
        .all(|r| r.get("outcome").and_then(Value::as_str) == Some("done"));
    let mut out = Map::new();
    out.insert("ran".into(), Value::Array(rows));
    out.insert("native".into(), Value::Array(native));
    out.insert("skipped".into(), Value::Array(skipped.to_vec()));
    (Value::Object(out), if all_done { 0 } else { 1 })
}

fn is_herding(item: &Value) -> bool {
    let tool = item.get("tool").and_then(Value::as_str);
    tool == Some("Bash") && payload_text(item, "command").contains(HERDING_COMMAND)
}

fn payload_text<'a>(item: &'a Value, key: &str) -> &'a str {
    item.get("payload")
        .and_then(|p| p.get(key))
        .and_then(Value::as_str)
        .unwrap_or("")
}

pub(crate) fn envelope_path(root: &Path, id: &str) -> PathBuf {    root.join(".bee")
        .join("workers")
        .join(format!("{id}-wave-run.json"))
}

fn row(id: &str, spawn: &Spawn, envelope_path: &Path) -> Value {
    let envelope = parse_envelope(&spawn.stdout);
    let env = envelope.as_ref();
    let mut m = Map::new();
    m.insert("id".into(), Value::String(id.to_string()));
    m.insert(
        "exit".into(),
        spawn.exit.map(Value::from).unwrap_or(Value::Null),
    );
    m.insert("outcome".into(), plain(env, &["outcome"]));
    m.insert("job_id".into(), plain(env, &["job_id"]));
    m.insert("report_path".into(), plain(env, &["report_path"]));
    m.insert("head_sha".into(), plain(env, &["git", "head_sha"]));
    m.insert("changed_paths".into(), changed_paths_count(env));
    m.insert("retryable".into(), plain(env, &["retryable"]));
    m.insert(
        "envelope_path".into(),
        Value::String(envelope_path.to_string_lossy().into_owned()),
    );
    Value::Object(m)
}

fn parse_envelope(stdout: &str) -> Option<Value> {
    let text = stdout.trim();
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        return Some(value);
    }
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }
    serde_json::from_str(&text[start..=end]).ok()
}

fn at<'a>(value: Option<&'a Value>, path: &[&str]) -> Option<&'a Value> {
    let mut current = value?;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

fn plain(value: Option<&Value>, path: &[&str]) -> Value {
    at(value, path).cloned().unwrap_or(Value::Null)
}

fn changed_paths_count(value: Option<&Value>) -> Value {
    match at(value, &["git", "changed_paths"]).and_then(Value::as_array) {
        Some(list) => Value::from(list.len() as u64),
        None => Value::Null,
    }
}

fn shell_runner(command: &str, stdin: &str, cwd: &Path) -> Spawn {
    let empty = Spawn { exit: None, stdout: String::new() };
    let Some(mut cmd) = crate::shell::command() else {
        return empty;
    };
    let spawned = cmd
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let Ok(mut child) = spawned else {
        return empty;
    };
    let brief = stdin.to_string();
    let writer = child.stdin.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let _ = pipe.write_all(brief.as_bytes());
        })
    });
    let out = child.wait_with_output();
    if let Some(handle) = writer {
        let _ = handle.join();
    }
    match out {
        Ok(o) => Spawn {
            exit: o.status.code(),
            stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
        },
        Err(_) => Spawn { exit: None, stdout: String::new() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn herding_item(id: &str) -> (String, Value) {
        (
            id.to_string(),
            json!({
                "tool": "Bash",
                "payload": {
                    "command": format!(".bee/bin/bee herding run --task-file - --json --cell-id \"{id}\""),
                    "stdin": format!("brief for {id}"),
                },
                "claimed": true,
                "reserved": [],
            }),
        )
    }

    fn agent_item(id: &str) -> (String, Value) {
        (
            id.to_string(),
            json!({
                "tool": "Agent",
                "payload": {"subagent_type": "bee-worker", "prompt": "go"},
                "claimed": true,
            }),
        )
    }

    fn done_envelope(id: &str) -> String {
        json!({
            "job_id": format!("job-{id}"),
            "outcome": "done",
            "report_path": format!(".bee/mailbox/job-{id}/report-1.md"),
            "git": {"head_sha": "abc123", "changed_paths": ["src/a.rs", "src/b.rs"]},
        })
        .to_string()
    }

    fn blocked_envelope(id: &str) -> String {
        json!({
            "job_id": format!("job-{id}"),
            "outcome": "blocked",
            "retryable": true,
        })
        .to_string()
    }

    #[test]
    fn dispatch_wave_run_runs_herding_items_and_returns_compact_rows() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let items = vec![herding_item("c-1"), herding_item("c-2"), agent_item("c-3")];
        let skipped = vec![json!({"id": "c-4", "reason": "already_claimed", "detail": "busy"})];
        let runner = |_command: &str, stdin: &str, cwd: &Path| {
            assert_eq!(cwd, root, "every child runs from the main root");
            if stdin.contains("c-1") {
                Spawn { exit: Some(0), stdout: done_envelope("c-1") }
            } else {
                Spawn { exit: Some(1), stdout: blocked_envelope("c-2") }
            }
        };

        let (value, code) = run_wave_with(root, &items, &skipped, &runner);
        assert_eq!(code, 1, "one blocked row must not exit 0: {value}");
        let ran = value["ran"].as_array().unwrap_or_else(|| panic!("{value}"));
        assert_eq!(ran.len(), 2, "{value}");
        assert_eq!(ran[0]["id"], json!("c-1"), "{value}");
        assert_eq!(ran[0]["exit"], json!(0), "{value}");
        assert_eq!(ran[0]["outcome"], json!("done"), "{value}");
        assert_eq!(ran[0]["job_id"], json!("job-c-1"), "{value}");
        assert_eq!(
            ran[0]["report_path"],
            json!(".bee/mailbox/job-c-1/report-1.md"),
            "{value}"
        );
        assert_eq!(ran[0]["head_sha"], json!("abc123"), "{value}");
        assert_eq!(ran[0]["changed_paths"], json!(2), "{value}");
        assert_eq!(ran[0]["retryable"], Value::Null, "{value}");
        assert_eq!(ran[1]["id"], json!("c-2"), "{value}");
        assert_eq!(ran[1]["outcome"], json!("blocked"), "{value}");
        assert_eq!(ran[1]["retryable"], json!(true), "{value}");
        assert_eq!(ran[1]["head_sha"], Value::Null, "{value}");
        assert_eq!(ran[1]["report_path"], Value::Null, "{value}");
        assert_eq!(ran[1]["changed_paths"], Value::Null, "{value}");

        let native = value["native"].as_array().unwrap_or_else(|| panic!("{value}"));
        assert_eq!(native.len(), 1, "{value}");
        assert_eq!(native[0], agent_item("c-3").1, "native items come back unchanged");
        assert_eq!(value["skipped"], Value::Array(skipped), "{value}");

        for (id, expected) in [("c-1", done_envelope("c-1")), ("c-2", blocked_envelope("c-2"))] {
            let path = envelope_path(root, id);
            assert_eq!(
                std::fs::read_to_string(&path).expect("the envelope is saved"),
                expected,
                "{} must hold the child's full stdout",
                path.display()
            );
            assert_eq!(
                ran.iter()
                    .find(|r| r["id"] == json!(id))
                    .and_then(|r| r["envelope_path"].as_str()),
                Some(path.to_str().unwrap()),
                "{value}"
            );
        }
    }
}
