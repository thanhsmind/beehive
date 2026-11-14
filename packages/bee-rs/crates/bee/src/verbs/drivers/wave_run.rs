use serde_json::{Map, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::mpsc;

const HERDING_COMMAND: &str = "bee herding run";

pub(crate) struct Spawn {
    exit: Option<i32>,
    stdout: String,
}

pub(crate) type Runner<'a> = &'a (dyn Fn(&str, &str, &Path) -> Spawn + Sync);

pub(crate) type Fill<'a> = &'a mut dyn FnMut(usize, &mut Vec<Value>) -> Vec<(String, Value)>;

pub(crate) fn run_rolling(root: &Path, limit: Option<usize>, fill: Fill<'_>) -> (Value, u8) {
    run_rolling_with(root, limit, fill, &shell_runner)
}

pub(crate) fn run_rolling_with(
    root: &Path,
    limit: Option<usize>,
    fill: Fill<'_>,
    runner: Runner<'_>,
) -> (Value, u8) {
    let mut native: Vec<Value> = Vec::new();
    let mut skipped: Vec<Value> = Vec::new();
    let mut rows: Vec<Value> = Vec::new();
    std::thread::scope(|scope| {
        let (tx, rx) = mpsc::channel::<(String, Spawn)>();
        let mut live = 0usize;
        loop {
            let slots = limit.map_or(usize::MAX, |n| n.saturating_sub(live));
            if slots > 0 {
                for (id, item) in fill(slots, &mut skipped) {
                    if !is_herding(&item) {
                        native.push(item);
                        continue;
                    }
                    let command = payload_text(&item, "command").to_string();
                    let stdin = payload_text(&item, "stdin").to_string();
                    let tx = tx.clone();
                    live += 1;
                    scope.spawn(move || {
                        let spawn = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            runner(&command, &stdin, root)
                        }))
                        .unwrap_or(Spawn { exit: None, stdout: String::new() });
                        let _ = tx.send((id, spawn));
                    });
                }
            }
            if live == 0 {
                break;
            }
            let Ok((id, spawn)) = rx.recv() else { break };
            live -= 1;
            rows.push(save_row(root, &id, &spawn));
        }
    });
    finish_value(rows, native, &skipped)
}

fn save_row(root: &Path, id: &str, spawn: &Spawn) -> Value {
    let path = envelope_path(root, id);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, spawn.stdout.as_bytes());
    row(id, spawn, &path)
}

fn finish_value(rows: Vec<Value>, native: Vec<Value>, skipped: &[Value]) -> (Value, u8) {
    let all_done = rows
        .iter()
        .all(|r| r.get("outcome").and_then(Value::as_str) == Some("done"));
    let mut out = Map::new();
    out.insert("ran".into(), Value::Array(rows));
    out.insert("native".into(), Value::Array(native));
    out.insert("skipped".into(), Value::Array(skipped.to_vec()));
    (Value::Object(out), if all_done { 0 } else { 1 })
}

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

    let rows: Vec<Value> = herding
        .iter()
        .zip(spawned.iter())
        .map(|((id, _), spawn)| save_row(root, id, spawn))
        .collect();
    finish_value(rows, native, skipped)
}

pub(crate) fn is_herding(item: &Value) -> bool {
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

    fn ready_fill<'a>(
        order: &'a [&'a str],
        deps: &'a [(&'a str, &'a str)],
        capped: &'a std::sync::Mutex<Vec<String>>,
        started: &'a std::sync::Mutex<Vec<String>>,
        natives: &'a [&'a str],
    ) -> impl FnMut(usize, &mut Vec<Value>) -> Vec<(String, Value)> + 'a {
        move |slots, _skipped| {
            let mut out = Vec::new();
            let mut herding = 0;
            for id in order {
                if herding >= slots {
                    break;
                }
                let mut started = started.lock().unwrap();
                if started.iter().any(|s| s == id) {
                    continue;
                }
                let capped = capped.lock().unwrap();
                let blocked = deps
                    .iter()
                    .any(|(cell, dep)| cell == id && !capped.iter().any(|c| c == dep));
                if blocked {
                    continue;
                }
                started.push(id.to_string());
                if natives.contains(id) {
                    out.push(agent_item(id));
                } else {
                    herding += 1;
                    out.push(herding_item(id));
                }
            }
            out
        }
    }

    #[test]
    fn rolling_keeps_at_most_limit_live_and_starts_the_third_when_one_ends() {
        let tmp = tempfile::tempdir().unwrap();
        let capped = std::sync::Mutex::new(Vec::new());
        let started = std::sync::Mutex::new(Vec::new());
        let live = std::sync::atomic::AtomicUsize::new(0);
        let peak = std::sync::atomic::AtomicUsize::new(0);
        let runner = |_c: &str, stdin: &str, _cwd: &Path| {
            use std::sync::atomic::Ordering::SeqCst;
            let now = live.fetch_add(1, SeqCst) + 1;
            peak.fetch_max(now, SeqCst);
            let id = stdin.trim_start_matches("brief for ").to_string();
            if id == "c-3" {
                assert_eq!(started.lock().unwrap().len(), 3);
            }
            std::thread::sleep(std::time::Duration::from_millis(if id == "c-1" { 20 } else { 120 }));
            live.fetch_sub(1, SeqCst);
            Spawn { exit: Some(0), stdout: done_envelope(&id) }
        };
        let mut fill = ready_fill(&["c-1", "c-2", "c-3"], &[], &capped, &started, &[]);
        let (value, code) = run_rolling_with(tmp.path(), Some(2), &mut fill, &runner);
        assert_eq!(code, 0, "{value}");
        assert_eq!(value["ran"].as_array().unwrap().len(), 3, "{value}");
        assert_eq!(peak.load(std::sync::atomic::Ordering::SeqCst), 2, "{value}");
    }

    #[test]
    fn rolling_starts_a_cell_whose_dependency_a_finished_run_capped() {
        let tmp = tempfile::tempdir().unwrap();
        let capped = std::sync::Mutex::new(Vec::new());
        let started = std::sync::Mutex::new(Vec::new());
        let runner = |_c: &str, stdin: &str, _cwd: &Path| {
            let id = stdin.trim_start_matches("brief for ").to_string();
            capped.lock().unwrap().push(id.clone());
            Spawn { exit: Some(0), stdout: done_envelope(&id) }
        };
        let mut fill = ready_fill(&["a", "b"], &[("b", "a")], &capped, &started, &[]);
        let (value, code) = run_rolling_with(tmp.path(), Some(2), &mut fill, &runner);
        assert_eq!(code, 0, "{value}");
        let ids: Vec<&str> = value["ran"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, vec!["a", "b"], "{value}");
    }

    #[test]
    fn rolling_returns_native_items_once_and_never_refills_them() {
        let tmp = tempfile::tempdir().unwrap();
        let capped = std::sync::Mutex::new(Vec::new());
        let started = std::sync::Mutex::new(Vec::new());
        let runner = |_c: &str, stdin: &str, _cwd: &Path| {
            let id = stdin.trim_start_matches("brief for ").to_string();
            Spawn { exit: Some(0), stdout: done_envelope(&id) }
        };
        let mut fill = ready_fill(&["n-1", "h-1", "h-2"], &[], &capped, &started, &["n-1"]);
        let (value, _code) = run_rolling_with(tmp.path(), Some(1), &mut fill, &runner);
        assert_eq!(value["native"].as_array().unwrap().len(), 1, "{value}");
        assert_eq!(value["native"][0], agent_item("n-1").1, "{value}");
        assert_eq!(value["ran"].as_array().unwrap().len(), 2, "{value}");
    }

    #[test]
    fn rolling_ends_at_once_when_nothing_is_ready() {
        let tmp = tempfile::tempdir().unwrap();
        let mut fill = |_slots: usize, _s: &mut Vec<Value>| Vec::new();
        let runner = |_c: &str, _i: &str, _p: &Path| -> Spawn { panic!("nothing runs") };
        let (value, code) = run_rolling_with(tmp.path(), Some(2), &mut fill, &runner);
        assert_eq!(code, 0, "{value}");
        assert_eq!(value["ran"], json!([]), "{value}");
    }
}
