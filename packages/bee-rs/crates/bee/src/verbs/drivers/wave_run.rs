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
    let mut silent = true;
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
            silent &= is_silent(&spawn);
            rows.push(save_row(root, &id, &spawn));
        }
    });
    finish_value(rows, native, &skipped, silent)
}

fn save_row(root: &Path, id: &str, spawn: &Spawn) -> Value {
    let path = envelope_path(root, id);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, spawn.stdout.as_bytes());
    let mut value = row(id, spawn, &path);
    let (judged, errors) = record_judge_blocks(root, id, &value);
    let capped = crate::verbs::cells::read_cell(root, id)
        .ok()
        .flatten()
        .is_some_and(|cell| cell.get("status").and_then(Value::as_str) == Some("capped"));
    if let Value::Object(m) = &mut value {
        m.insert("capped".into(), Value::Bool(capped));
        m.insert("judged".into(), Value::from(judged));
        if !errors.is_empty() {
            m.insert("judge_errors".into(), Value::from(errors));
        }
    }
    value
}

fn record_judge_blocks(root: &Path, id: &str, row: &Value) -> (Vec<String>, Vec<String>) {
    let mut judged = Vec::new();
    let mut errors = Vec::new();
    let Ok(Some(cell)) = crate::verbs::cells::read_cell(root, id) else {
        return (judged, errors);
    };
    if cell.get("role").and_then(Value::as_str) != Some("test") {
        return (judged, errors);
    }
    let deps: Vec<&str> = cell
        .get("deps")
        .and_then(Value::as_array)
        .map(|list| list.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let Some(report) = row.get("report_path").and_then(Value::as_str) else {
        return (judged, errors);
    };
    let Ok(text) = std::fs::read_to_string(root.join(report)) else {
        return (judged, errors);
    };
    let blocks = match crate::verbs::cells::extract_fenced_verdicts(&text) {
        Ok(blocks) => blocks,
        Err(e) => {
            if !e.starts_with("no json") {
                errors.push(e);
            }
            return (judged, errors);
        }
    };
    for (block_id, body) in blocks {
        if !deps.contains(&block_id.as_str()) {
            continue;
        }
        let body = match body {
            Value::String(raw) => raw,
            other => other.to_string(),
        };
        let one = format!("```json {block_id}\n{body}\n```\n");
        match crate::verbs::cells::process_judge_from_text(
            root,
            &one,
            Some(&block_id),
            None,
            None,
            None,
            false,
        ) {
            Ok(crate::verbs::reservations::Out::Emit(_, _, 0)) => judged.push(block_id),
            Ok(crate::verbs::reservations::Out::Emit(_, line, _)) => errors.push(line),
            Ok(crate::verbs::reservations::Out::Thrown(msg)) => errors.push(msg),
            Err(_) => errors.push(format!("{block_id}: judge record failed")),
        }
    }
    (judged, errors)
}

fn is_silent(spawn: &Spawn) -> bool {
    spawn.exit.is_none() && spawn.stdout.trim().is_empty()
}

fn finish_value(
    rows: Vec<Value>,
    native: Vec<Value>,
    skipped: &[Value],
    silent: bool,
) -> (Value, u8) {
    let empty_wave = silent && !rows.is_empty();
    let all_done = rows
        .iter()
        .all(|r| r.get("outcome").and_then(Value::as_str) == Some("done"));
    let mut out = Map::new();
    if empty_wave {
        out.insert("empty_wave".into(), Value::Bool(true));
        out.insert(
            "empty_wave_reason".into(),
            Value::String(
                "every launched worker returned no exit code and no output; nobody answered".into(),
            ),
        );
    }
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

    let silent = spawned.iter().all(is_silent);
    let rows: Vec<Value> = herding
        .iter()
        .zip(spawned.iter())
        .map(|((id, _), spawn)| save_row(root, id, spawn))
        .collect();
    finish_value(rows, native, skipped, silent)
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

    fn judge_block(id: &str, verdict: &str) -> String {
        let body = json!({
            "schema": "judge-verdict/1",
            "verdict": verdict,
            "checks": [{"id": "c1", "status": "PASS", "evidence": "f.rs:1 shows it"}],
            "fixability": "automatic",
            "confidence": "high",
        });
        format!("```json {id}\n{body}\n```\n")
    }

    fn put_cell(root: &Path, id: &str, role: &str, deps: &[&str]) {
        let dir = root.join(".bee").join("cells");
        std::fs::create_dir_all(&dir).unwrap();
        let cell = json!({"id": id, "role": role, "deps": deps, "status": "open", "trace": {}});
        std::fs::write(dir.join(format!("{id}.json")), cell.to_string()).unwrap();
    }

    fn put_report(root: &Path, id: &str, text: &str) {
        let dir = root.join(".bee").join("mailbox").join(format!("job-{id}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("report-1.md"), text).unwrap();
    }

    fn judge_entries(root: &Path, id: &str) -> usize {
        let raw = std::fs::read_to_string(root.join(".bee").join("cells").join(format!("{id}.json"))).unwrap();
        let cell: Value = serde_json::from_str(&raw).unwrap();
        cell["trace"]["semantic_judge"].as_array().map_or(0, Vec::len)
    }

    fn run_one(root: &Path, id: &str) -> Value {
        let items = vec![herding_item(id)];
        let runner = |_c: &str, stdin: &str, _cwd: &Path| {
            let id = stdin.trim_start_matches("brief for ").to_string();
            Spawn { exit: Some(0), stdout: done_envelope(&id) }
        };
        let (value, _) = run_wave_with(root, &items, &[], &runner);
        value["ran"][0].clone()
    }

    #[test]
    fn wave_judge_record_records_one_verdict_per_dep_block_of_a_test_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        put_cell(root, "a-1", "code", &[]);
        put_cell(root, "a-2", "code", &[]);
        put_cell(root, "t-1", "test", &["a-1", "a-2"]);
        let text = format!("{}{}", judge_block("a-1", "PASS"), judge_block("a-2", "PASS"));
        put_report(root, "t-1", &text);
        let row = run_one(root, "t-1");
        assert_eq!(row["judged"], json!(["a-1", "a-2"]), "{row}");
        assert!(row.get("judge_errors").is_none(), "{row}");
        assert_eq!(judge_entries(root, "a-1"), 1);
        assert_eq!(judge_entries(root, "a-2"), 1);
    }

    #[test]
    fn wave_judge_record_ignores_a_block_for_a_cell_that_is_not_a_dep() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        put_cell(root, "a-1", "code", &[]);
        put_cell(root, "x-9", "code", &[]);
        put_cell(root, "t-1", "test", &["a-1"]);
        let text = format!("{}{}", judge_block("a-1", "PASS"), judge_block("x-9", "PASS"));
        put_report(root, "t-1", &text);
        let row = run_one(root, "t-1");
        assert_eq!(row["judged"], json!(["a-1"]), "{row}");
        assert_eq!(judge_entries(root, "a-1"), 1);
        assert_eq!(judge_entries(root, "x-9"), 0);
    }

    #[test]
    fn wave_judge_record_records_nothing_for_an_invalid_block_and_shows_it_on_the_row() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        put_cell(root, "a-1", "code", &[]);
        put_cell(root, "t-1", "test", &["a-1"]);
        put_report(root, "t-1", &judge_block("a-1", "MAYBE"));
        let row = run_one(root, "t-1");
        assert_eq!(row["judged"], json!([]), "{row}");
        let errors = row["judge_errors"].as_array().unwrap_or_else(|| panic!("{row}"));
        assert_eq!(errors.len(), 1, "{row}");
        assert!(errors[0].as_str().unwrap().contains("verdict must be one of"), "{row}");
        assert_eq!(judge_entries(root, "a-1"), 0);
    }

    #[test]
    fn wave_judge_record_never_reads_the_report_of_a_code_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        put_cell(root, "a-1", "code", &[]);
        put_cell(root, "c-1", "code", &["a-1"]);
        put_report(root, "c-1", &judge_block("a-1", "PASS"));
        let row = run_one(root, "c-1");
        assert_eq!(row["judged"], json!([]), "{row}");
        assert!(row.get("judge_errors").is_none(), "{row}");
        assert_eq!(judge_entries(root, "a-1"), 0);
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

    fn silent_spawn() -> Spawn {
        Spawn { exit: None, stdout: String::new() }
    }

    #[test]
    fn wave_flags_empty_wave_when_every_row_is_silent() {
        let tmp = tempfile::tempdir().unwrap();
        let items = vec![herding_item("c-1"), herding_item("c-2")];
        let runner = |_c: &str, _i: &str, _p: &Path| silent_spawn();
        let (value, code) = run_wave_with(tmp.path(), &items, &[], &runner);
        assert_eq!(code, 1, "{value}");
        assert_eq!(value["empty_wave"], json!(true), "{value}");
        assert!(
            value["empty_wave_reason"].as_str().is_some_and(|r| !r.is_empty()),
            "{value}"
        );
    }

    #[test]
    fn wave_with_one_answer_is_not_flagged_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let items = vec![herding_item("c-1"), herding_item("c-2")];
        let runner = |_c: &str, stdin: &str, _p: &Path| {
            if stdin.contains("c-1") {
                Spawn { exit: Some(0), stdout: done_envelope("c-1") }
            } else {
                silent_spawn()
            }
        };
        let (value, _code) = run_wave_with(tmp.path(), &items, &[], &runner);
        assert!(value.get("empty_wave").is_none(), "{value}");
    }

    #[test]
    fn wave_with_zero_rows_is_not_flagged_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let runner = |_c: &str, _i: &str, _p: &Path| -> Spawn { panic!("nothing runs") };
        let (value, _code) = run_wave_with(tmp.path(), &[agent_item("c-3")], &[], &runner);
        assert!(value.get("empty_wave").is_none(), "{value}");
        let mut fill = |_slots: usize, _s: &mut Vec<Value>| Vec::new();
        let (value, _code) = run_rolling_with(tmp.path(), Some(2), &mut fill, &runner);
        assert!(value.get("empty_wave").is_none(), "{value}");
    }

    #[test]
    fn rolling_flags_empty_wave_when_every_row_is_silent() {
        let tmp = tempfile::tempdir().unwrap();
        let capped = std::sync::Mutex::new(Vec::new());
        let started = std::sync::Mutex::new(Vec::new());
        let runner = |_c: &str, _i: &str, _p: &Path| silent_spawn();
        let mut fill = ready_fill(&["a", "b"], &[], &capped, &started, &[]);
        let (value, _code) = run_rolling_with(tmp.path(), Some(2), &mut fill, &runner);
        assert_eq!(value["empty_wave"], json!(true), "{value}");
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
