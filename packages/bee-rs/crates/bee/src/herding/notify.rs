use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{json, Map, Value};

use super::job_verbs::JobVerbError;
use super::{mailbox, resolve_main_root};

pub(crate) const KINDS: [&str; 5] = ["question", "blocked", "dependency", "progress", "reopen"];

pub(crate) fn progress_path(job_dir: &Path) -> PathBuf {
    job_dir.join("progress.jsonl")
}

pub(crate) fn messages_path(job_dir: &Path) -> PathBuf {
    job_dir.join("messages.jsonl")
}

fn refuse(code: &'static str, message: String) -> JobVerbError {
    JobVerbError { code, message: format!("herding notify: {message}") }
}

pub(crate) fn append_message(
    main_root: &Path,
    job_id: Option<&str>,
    kind: &str,
    text: &str,
    correlation_id: Option<&str>,
    at: DateTime<Utc>,
) -> Result<Value, JobVerbError> {
    let job_id = job_id.map(str::trim).filter(|j| !j.is_empty()).ok_or_else(|| {
        refuse(
            "not_herded",
            "BEE_HERDING_JOB_ID is not set; notify runs only inside a herded worker. FIX: put the note in your result file".into(),
        )
    })?;
    if matches!(job_id, "." | "..") || job_id.contains(['/', '\\']) {
        return Err(refuse("bad_job_id", format!("\"{job_id}\" is not a job id. FIX: run notify from the worker bee started")));
    }
    if !KINDS.contains(&kind) {
        return Err(refuse(
            "unknown_kind",
            format!("unknown kind \"{kind}\". FIX: pass --kind one of {}", KINDS.join("|")),
        ));
    }
    if text.trim().is_empty() {
        return Err(refuse("text_missing", "--text is empty. FIX: pass --text <text>".into()));
    }
    let bee_dir = main_root.join(".bee");
    let job_dir = mailbox::mailbox_dir(&bee_dir, job_id);
    let crate::fsutil::ReadJson::Parsed(job) = crate::fsutil::read_json(&mailbox::job_path(&bee_dir, job_id)) else {
        return Err(refuse(
            "job_not_found",
            format!("job \"{job_id}\" has no job.json in {}. FIX: check BEE_HERDING_JOB_ID", job_dir.display()),
        ));
    };
    let line = json!({
        "job_id": job_id,
        "cell_id": job.get("cell_id").cloned().unwrap_or(Value::Null),
        "kind": kind,
        "text": text,
        "correlation_id": correlation_id,
        "at": at.to_rfc3339_opts(SecondsFormat::Millis, true),
    });
    let mut bytes = serde_json::to_vec(&line).unwrap_or_default();
    bytes.push(b'\n');
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(messages_path(&job_dir))
        .and_then(|mut f| f.write_all(&bytes))
        .map_err(|e| refuse("write_failed", format!("cannot append to {}: {e}. FIX: check the job mailbox", job_dir.display())))?;
    Ok(line)
}

pub(crate) fn append_progress(
    main_root: &Path,
    job_id: Option<&str>,
    step: Option<&str>,
    note: &str,
    at: DateTime<Utc>,
) -> Result<Value, JobVerbError> {
    let job_id = job_id.map(str::trim).filter(|j| !j.is_empty()).ok_or_else(|| {
        refuse(
            "not_herded",
            "BEE_HERDING_JOB_ID is not set; notify runs only inside a herded worker. FIX: put the note in your result file".into(),
        )
    })?;
    if matches!(job_id, "." | "..") || job_id.contains(['/', '\\']) {
        return Err(refuse("bad_job_id", format!("\"{job_id}\" is not a job id. FIX: run notify from the worker bee started")));
    }
    if note.trim().is_empty() {
        return Err(refuse("text_missing", "--text is empty. FIX: pass --text <text>".into()));
    }
    if note.chars().count() > 120 {
        return Err(refuse(
            "note_too_long",
            format!("note is {} characters (maximum 120). FIX: shorten note to 120 characters or less", note.chars().count()),
        ));
    }
    let bee_dir = main_root.join(".bee");
    let job_dir = mailbox::mailbox_dir(&bee_dir, job_id);
    std::fs::create_dir_all(&job_dir)
        .map_err(|e| refuse("dir_failed", format!("cannot create {}: {e}. FIX: check permissions", job_dir.display())))?;
    let step_val = step.map(str::trim).filter(|s| !s.is_empty()).map(Value::from).unwrap_or(Value::Null);
    let line = json!({
        "ts": at.to_rfc3339_opts(SecondsFormat::Millis, true),
        "step": step_val,
        "note": note,
    });
    let mut bytes = serde_json::to_vec(&line).unwrap_or_default();
    bytes.push(b'\n');
    let progress_file = mailbox::progress_path(&bee_dir, job_id);
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&progress_file)
        .and_then(|mut f| f.write_all(&bytes))
        .map_err(|e| refuse("write_failed", format!("cannot append to {}: {e}. FIX: check the job mailbox", progress_file.display())))?;
    Ok(line)
}

fn complete_lines(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
    let end = bytes.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
    bytes[..end].split_inclusive(|&b| b == b'\n')
}

pub(crate) fn insert_messages(job_map: &mut Map<String, Value>, job_dir: &Path) -> String {
    let bytes = std::fs::read(messages_path(job_dir)).unwrap_or_default();
    let cursor = std::fs::read_to_string(job_dir.join("messages.cursor"))
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0)
        .min(bytes.len());
    let total = complete_lines(&bytes).count();
    let unread = complete_lines(&bytes[cursor..]).count();
    let last = complete_lines(&bytes)
        .last()
        .and_then(|l| serde_json::from_slice::<Value>(l).ok())
        .map(|m| json!({"kind": m.get("kind"), "text": m.get("text")}));
    job_map.insert("messages".into(), total.into());
    job_map.insert("unread_messages".into(), unread.into());
    job_map.insert("last_message".into(), last.unwrap_or(Value::Null));
    if total == 0 {
        return String::new();
    }
    format!(" messages={total} unread_messages={unread}")
}

pub(super) fn notify(args: &[&str]) -> ExitCode {
    let (mut kind, mut text, mut correlation, mut main_root, mut json_out) = (None, None, None, None, false);
    let mut step: Option<&str> = None;
    let mut job_arg: Option<&str> = None;
    let mut it = args.iter().copied();
    while let Some(arg) = it.next() {
        match arg {
            "--kind" => kind = it.next(),
            "--text" | "--note" => text = it.next(),
            "--step" => step = it.next(),
            "--job" => job_arg = it.next(),
            "--correlation-id" => correlation = it.next(),
            "--main-root" => main_root = it.next(),
            "--json" => json_out = true,
            _ if arg.starts_with("--kind=") => kind = Some(&arg["--kind=".len()..]),
            _ if arg.starts_with("--text=") => text = Some(&arg["--text=".len()..]),
            _ if arg.starts_with("--note=") => text = Some(&arg["--note=".len()..]),
            _ if arg.starts_with("--step=") => step = Some(&arg["--step=".len()..]),
            _ if arg.starts_with("--job=") => job_arg = Some(&arg["--job=".len()..]),
            _ if arg.starts_with("--correlation-id=") => correlation = Some(&arg["--correlation-id=".len()..]),
            _ if arg.starts_with("--main-root=") => main_root = Some(&arg["--main-root=".len()..]),
            _ => {}
        }
    }
    let env_job = std::env::var("BEE_HERDING_JOB_ID").ok();
    let job = job_arg.or(env_job.as_deref());
    let root = match resolve_main_root(main_root) {
        Some(r) => r,
        None => {
            let err = refuse("main_root_missing", "could not resolve main checkout root. FIX: pass --main-root <path>".into());
            if json_out {
                println!("{}", json!({"error": err.code, "message": err.message}));
            } else {
                eprintln!("{}", err.message);
            }
            return ExitCode::FAILURE;
        }
    };
    let kind_str = kind.unwrap_or("");
    let result = if kind_str == "progress" {
        append_progress(&root, job, step, text.unwrap_or(""), Utc::now())
    } else {
        append_message(&root, job, kind_str, text.unwrap_or(""), correlation, Utc::now())
    };
    match result {
        Ok(line) => {
            if json_out {
                println!("{line}");
            } else if kind_str == "progress" {
                println!("herding: sent a progress note to the leader; keep working");
            } else {
                println!("herding: sent a {} note to the leader; keep working", line["kind"].as_str().unwrap_or(""));
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            if json_out {
                println!("{}", json!({"error": e.code, "message": e.message}));
            } else {
                eprintln!("{}", e.message);
            }
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-12T08:00:00.123Z").unwrap().with_timezone(&Utc)
    }

    fn fixture() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".bee/mailbox/job-1");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("job.json"), r#"{"job_id":"job-1","cell_id":"pl-5"}"#).unwrap();
        (tmp, dir)
    }

    #[test]
    fn notify_refuses_without_the_job_env() {
        let (tmp, dir) = fixture();
        let err = append_message(tmp.path(), None, "progress", "hi", None, now()).unwrap_err();
        assert_eq!(err.code, "not_herded");
        assert!(err.message.contains("BEE_HERDING_JOB_ID"), "{}", err.message);
        assert!(!messages_path(&dir).exists());
    }

    #[test]
    fn notify_refuses_an_unknown_kind() {
        let (tmp, dir) = fixture();
        let err = append_message(tmp.path(), Some("job-1"), "chat", "hi", None, now()).unwrap_err();
        assert_eq!(err.code, "unknown_kind");
        assert!(!messages_path(&dir).exists());
    }

    #[test]
    fn notify_refuses_a_job_id_that_is_a_path() {
        let (tmp, _) = fixture();
        let err = append_message(tmp.path(), Some("../job-1"), "progress", "hi", None, now()).unwrap_err();
        assert_eq!(err.code, "bad_job_id");
    }

    #[test]
    fn notify_appends_one_newline_terminated_line_per_call() {
        let (tmp, dir) = fixture();
        append_message(tmp.path(), Some("job-1"), "blocked", "need the schema", Some("q-1"), now()).unwrap();
        append_message(tmp.path(), Some("job-1"), "reopen", "half done", None, now()).unwrap();
        let raw = std::fs::read_to_string(messages_path(&dir)).unwrap();
        assert!(raw.ends_with('\n'));
        let lines: Vec<Value> = raw.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0],
            json!({
                "job_id": "job-1", "cell_id": "pl-5", "kind": "blocked", "text": "need the schema",
                "correlation_id": "q-1", "at": "2026-10-12T08:00:00.123Z",
            })
        );
        assert_eq!(lines[1]["kind"], "reopen");
        assert_eq!(lines[1]["correlation_id"], Value::Null);
    }

    #[test]
    fn notify_progress_writes_progress_jsonl_and_leaves_messages_jsonl_absent() {
        let (tmp, dir) = fixture();
        let code = notify(&[
            "--main-root", tmp.path().to_str().unwrap(),
            "--job", "job-1",
            "--kind", "progress",
            "--step", "step1",
            "--text", "doing work",
        ]);
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(!messages_path(&dir).exists());
        let progress_file = dir.join("progress.jsonl");
        assert!(progress_file.exists());
        let raw = std::fs::read_to_string(&progress_file).unwrap();
        let val: Value = serde_json::from_str(raw.trim()).unwrap();
        assert_eq!(val["step"], "step1");
        assert_eq!(val["note"], "doing work");
        assert!(val["ts"].is_string());
    }

    #[test]
    fn notify_refuses_note_over_120_chars() {
        let (tmp, _) = fixture();
        let long_note = "a".repeat(121);
        let code = notify(&[
            "--main-root", tmp.path().to_str().unwrap(),
            "--job", "job-1",
            "--kind", "progress",
            "--text", &long_note,
        ]);
        assert_eq!(code, ExitCode::FAILURE);
    }

    #[test]
    fn notify_progress_with_job_and_no_job_json_succeeds() {
        let tmp = tempfile::tempdir().unwrap();
        let code = notify(&[
            "--main-root", tmp.path().to_str().unwrap(),
            "--job", "scratch-job",
            "--kind", "progress",
            "--text", "first note",
        ]);
        assert_eq!(code, ExitCode::SUCCESS);
        let progress_file = tmp.path().join(".bee/mailbox/scratch-job/progress.jsonl");
        assert!(progress_file.exists());
    }

    #[test]
    fn notify_blocked_still_writes_messages_jsonl() {
        let (tmp, dir) = fixture();
        let code = notify(&[
            "--main-root", tmp.path().to_str().unwrap(),
            "--job", "job-1",
            "--kind", "blocked",
            "--text", "stuck on permissions",
        ]);
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(messages_path(&dir).exists());
    }

    #[test]
    fn notify_status_counts_unread_past_the_cursor_and_skips_a_partial_line() {
        let (tmp, dir) = fixture();
        let mut m = Map::new();
        assert_eq!(insert_messages(&mut m, &dir), "");
        assert_eq!((m["messages"].clone(), m["unread_messages"].clone()), (json!(0), json!(0)));
        assert_eq!(m["last_message"], Value::Null);

        append_message(tmp.path(), Some("job-1"), "progress", "one", None, now()).unwrap();
        let first_len = std::fs::metadata(messages_path(&dir)).unwrap().len();
        append_message(tmp.path(), Some("job-1"), "blocked", "two", None, now()).unwrap();
        let mut f = std::fs::OpenOptions::new().append(true).open(messages_path(&dir)).unwrap();
        f.write_all(br#"{"kind":"progress","te"#).unwrap();

        let mut m = Map::new();
        assert_eq!(insert_messages(&mut m, &dir), " messages=2 unread_messages=2");
        assert_eq!(m["last_message"], json!({"kind": "blocked", "text": "two"}));

        std::fs::write(dir.join("messages.cursor"), first_len.to_string()).unwrap();
        let mut m = Map::new();
        insert_messages(&mut m, &dir);
        assert_eq!((m["messages"].clone(), m["unread_messages"].clone()), (json!(2), json!(1)));
    }
}
