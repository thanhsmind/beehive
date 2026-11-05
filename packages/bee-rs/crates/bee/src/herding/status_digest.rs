use std::path::Path;
use std::process::ExitCode;
use chrono::{SecondsFormat, Utc};
use serde_json::{json, Value};

use super::mailbox;

pub(crate) trait SupervisorSummarizer {
    fn summarize(&self, main_root: &Path, notes: &[ProgressNote]) -> Result<String, String>;
}

pub(crate) struct RealSupervisor;

impl SupervisorSummarizer for RealSupervisor {
    fn summarize(&self, main_root: &Path, notes: &[ProgressNote]) -> Result<String, String> {
        let rt = crate::session_identity::locate_caller()
            .map(|c| c.runtime)
            .filter(|rt| crate::verbs::drivers::DISPATCH_RUNTIMES.contains(&rt.as_str()))
            .unwrap_or_else(|| "claude".to_string());

        let bee_exe = std::env::current_exe().map_err(|e| format!("cannot get current exe: {e}"))?;
        let output = std::process::Command::new(&bee_exe)
            .current_dir(main_root)
            .args(["dispatch", "prepare", "--runtime", &rt, "--kind", "gather", "--role", "supervisor", "--json"])
            .output()
            .map_err(|e| format!("cannot run dispatch prepare: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("dispatch prepare failed: {stderr}"));
        }

        let env_val: Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("dispatch prepare emitted invalid JSON: {e}"))?;

        let tool = env_val.get("tool").and_then(Value::as_str).unwrap_or("");
        if tool != "Bash" {
            return Err("non-bash payload".to_string());
        }

        let cmd_str = env_val
            .get("payload")
            .and_then(|p| p.get("command"))
            .and_then(Value::as_str)
            .ok_or_else(|| "missing payload command".to_string())?;

        let mut task = String::from("Summarize the following worker progress notes into one status line:\n");
        for n in notes {
            if let Some(step) = &n.step {
                task.push_str(&format!("- [{}] {}\n", step, n.note));
            } else {
                task.push_str(&format!("- {}\n", n.note));
            }
        }

        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd_str)
            .current_dir(main_root)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("cannot spawn supervisor command: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(task.as_bytes());
        }

        let run_output = child.wait_with_output().map_err(|e| format!("error waiting on supervisor: {e}"))?;
        if !run_output.status.success() {
            let stderr = String::from_utf8_lossy(&run_output.stderr);
            return Err(format!("supervisor command failed: {stderr}"));
        }

        let stdout_str = String::from_utf8_lossy(&run_output.stdout);
        let summary = if let Ok(val) = serde_json::from_str::<Value>(&stdout_str) {
            val.get("summary")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| stdout_str.to_string())
        } else {
            stdout_str.to_string()
        };

        let first_line = summary
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .ok_or_else(|| "empty supervisor summary".to_string())?;

        let line_120: String = first_line.chars().take(120).collect();
        Ok(line_120)
    }
}

pub(crate) struct ProgressNote {
    pub(crate) ts: Option<String>,
    pub(crate) step: Option<String>,
    pub(crate) note: String,
}

fn read_progress_notes(progress_path: &Path) -> Vec<ProgressNote> {
    let Ok(bytes) = std::fs::read(progress_path) else {
        return Vec::new();
    };
    let mut notes = Vec::new();
    let has_trailing_newline = bytes.ends_with(b"\n");
    let mut lines: Vec<&[u8]> = bytes.split(|&b| b == b'\n').collect();
    if !has_trailing_newline {
        lines.pop();
    }
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Ok(val) = serde_json::from_slice::<Value>(line) else {
            continue;
        };
        let note = match val.get("note").or_else(|| val.get("text")).and_then(Value::as_str) {
            Some(n) if !n.trim().is_empty() => n.trim().to_string(),
            _ => continue,
        };
        let ts = val.get("ts").or_else(|| val.get("at")).and_then(Value::as_str).map(str::to_string);
        let step = val.get("step").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
        notes.push(ProgressNote { ts, step, note });
    }
    notes
}

fn format_verbatim_note(note: &ProgressNote) -> String {
    if let Some(step) = &note.step {
        if !step.trim().is_empty() {
            return format!("{}: {}", step.trim(), note.note);
        }
    }
    note.note.clone()
}

pub(crate) fn status_digest_with_summarizer(
    main_root: &Path,
    job_id: &str,
    _json_out: bool,
    summarizer: &dyn SupervisorSummarizer,
) -> (ExitCode, Value, String) {
    let bee_dir = main_root.join(".bee");
    let job_dir = mailbox::mailbox_dir(&bee_dir, job_id);
    let progress_file = mailbox::progress_path(&bee_dir, job_id);
    let notes = read_progress_notes(&progress_file);

    let mut entry_names = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&job_dir) {
        for entry in rd.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                entry_names.push(name);
            }
        }
    }

    if let Some(round) = mailbox::latest_result_round(&entry_names) {
        let res_path = mailbox::result_path(&bee_dir, job_id, round);
        if let Ok(text) = std::fs::read_to_string(&res_path) {
            if let Ok(res) = mailbox::parse_result_text(round, &text) {
                let status_str = match res.status {
                    mailbox::MailboxStatus::Done => "done",
                    mailbox::MailboxStatus::Blocked => "blocked",
                    mailbox::MailboxStatus::Question => "question",
                };
                let line = format!("{status_str}: {}", res.summary);
                let val = json!({
                    "job": job_id,
                    "line": line,
                    "source": "result",
                    "notes": notes.len(),
                });
                return (ExitCode::SUCCESS, val, line);
            }
        }
    }

    if notes.is_empty() {
        let line = "no progress notes yet".to_string();
        let val = json!({
            "job": job_id,
            "line": line,
            "source": "none",
            "notes": 0,
        });
        return (ExitCode::SUCCESS, val, line);
    }

    if notes.len() <= 3 {
        let last_note = notes.last().unwrap();
        let line = format_verbatim_note(last_note);
        let val = json!({
            "job": job_id,
            "line": line,
            "source": "verbatim",
            "notes": notes.len(),
        });
        return (ExitCode::SUCCESS, val, line);
    }

    let digest_path = job_dir.join("status-digest.json");
    if let Ok(raw) = std::fs::read_to_string(&digest_path) {
        if let Ok(cached) = serde_json::from_str::<Value>(&raw) {
            if cached.get("notes").and_then(Value::as_u64) == Some(notes.len() as u64) {
                if let Some(cached_line) = cached.get("line").and_then(Value::as_str) {
                    let line = cached_line.to_string();
                    let val = json!({
                        "job": job_id,
                        "line": line,
                        "source": "cached",
                        "notes": notes.len(),
                    });
                    return (ExitCode::SUCCESS, val, line);
                }
            }
        }
    }

    match summarizer.summarize(main_root, &notes) {
        Ok(line) => {
            let cache_obj = json!({
                "notes": notes.len(),
                "line": line,
                "at": Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            });
            let _ = crate::fsutil::write_json_atomic(&digest_path, &cache_obj);
            let val = json!({
                "job": job_id,
                "line": line,
                "source": "supervisor",
                "notes": notes.len(),
            });
            (ExitCode::SUCCESS, val, line)
        }
        Err(reason) => {
            let last_note = notes.last().unwrap();
            let line = format_verbatim_note(last_note);
            let val = json!({
                "job": job_id,
                "line": line,
                "source": "verbatim",
                "notes": notes.len(),
                "fallback": reason,
            });
            (ExitCode::SUCCESS, val, line)
        }
    }
}

pub(crate) fn status_job(main_root: &Path, job_id: &str, json_out: bool) -> ExitCode {
    let (code, val, line) = status_digest_with_summarizer(main_root, job_id, json_out, &RealSupervisor);
    if json_out {
        println!("{val}");
    } else {
        println!("{line}");
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeSummarizer {
        result: Result<String, String>,
        calls: AtomicUsize,
    }

    impl FakeSummarizer {
        fn new(result: Result<String, String>) -> Self {
            Self {
                result,
                calls: AtomicUsize::new(0),
            }
        }
    }

    impl SupervisorSummarizer for FakeSummarizer {
        fn summarize(&self, _main_root: &Path, _notes: &[ProgressNote]) -> Result<String, String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.result.clone()
        }
    }

    struct PanickingSummarizer;

    impl SupervisorSummarizer for PanickingSummarizer {
        fn summarize(&self, _main_root: &Path, _notes: &[ProgressNote]) -> Result<String, String> {
            panic!("summarizer should never be called");
        }
    }

    #[test]
    fn red_test_status_job_no_notes_and_no_result() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let job_dir = root.join(".bee/mailbox/J");
        std::fs::create_dir_all(&job_dir).unwrap();
        std::fs::write(job_dir.join("job.json"), r#"{"job_id":"J"}"#).unwrap();
        std::fs::write(job_dir.join("log.txt"), "SENTINEL_OUTPUT_NEVER_LEAK_INTO_STATUS\n").unwrap();

        let (code, val, line) = status_digest_with_summarizer(root, "J", true, &PanickingSummarizer);
        assert_eq!(code, ExitCode::SUCCESS);
        assert_eq!(line, "no progress notes yet");
        assert_eq!(val["job"], "J");
        assert_eq!(val["line"], "no progress notes yet");
        assert_eq!(val["source"], "none");
        assert_eq!(val["notes"], 0);
        assert!(!line.contains("SENTINEL"));
        assert!(!val.to_string().contains("SENTINEL"));
    }

    #[test]
    fn status_job_verbatim_up_to_three_notes_and_never_calls_summarizer() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let job_dir = root.join(".bee/mailbox/job-3");
        std::fs::create_dir_all(&job_dir).unwrap();
        let notes_path = job_dir.join("progress.jsonl");

        std::fs::write(
            &notes_path,
            "{\"ts\":\"2026-01-01T00:00:00Z\",\"step\":\"read\",\"note\":\"read files\"}\n\
             {\"ts\":\"2026-01-01T00:01:00Z\",\"step\":\"edit\",\"note\":\"applied change\"}\n\
             {\"ts\":\"2026-01-01T00:02:00Z\",\"note\":\"testing code\"}\n",
        ).unwrap();

        let (code, val, line) = status_digest_with_summarizer(root, "job-3", true, &PanickingSummarizer);
        assert_eq!(code, ExitCode::SUCCESS);
        assert_eq!(line, "testing code");
        assert_eq!(val["job"], "job-3");
        assert_eq!(val["line"], "testing code");
        assert_eq!(val["source"], "verbatim");
        assert_eq!(val["notes"], 3);

        std::fs::write(
            &notes_path,
            "{\"ts\":\"2026-01-01T00:00:00Z\",\"step\":\"setup\",\"note\":\"init workspace\"}\n",
        ).unwrap();
        let (code2, val2, line2) = status_digest_with_summarizer(root, "job-3", false, &PanickingSummarizer);
        assert_eq!(code2, ExitCode::SUCCESS);
        assert_eq!(line2, "setup: init workspace");
        assert_eq!(val2["source"], "verbatim");
        assert_eq!(val2["notes"], 1);
    }

    #[test]
    fn status_job_result_outranks_notes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let job_dir = root.join(".bee/mailbox/job-res");
        std::fs::create_dir_all(&job_dir).unwrap();
        std::fs::write(
            job_dir.join("progress.jsonl"),
            "{\"ts\":\"2026-01-01T00:00:00Z\",\"note\":\"work in progress\"}\n",
        ).unwrap();
        std::fs::write(
            job_dir.join("result-2.json"),
            "{\"status\":\"done\",\"summary\":\"feature delivered\",\"files_changed\":[],\"proof\":\"cargo test\"}\n",
        ).unwrap();

        let (code, val, line) = status_digest_with_summarizer(root, "job-res", true, &PanickingSummarizer);
        assert_eq!(code, ExitCode::SUCCESS);
        assert_eq!(line, "done: feature delivered");
        assert_eq!(val["job"], "job-res");
        assert_eq!(val["line"], "done: feature delivered");
        assert_eq!(val["source"], "result");
        assert_eq!(val["notes"], 1);
    }

    #[test]
    fn status_job_more_than_three_notes_uses_supervisor_and_caches_result() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let job_dir = root.join(".bee/mailbox/job-4");
        std::fs::create_dir_all(&job_dir).unwrap();
        std::fs::write(
            job_dir.join("progress.jsonl"),
            "{\"ts\":\"2026-01-01T00:00:00Z\",\"note\":\"note 1\"}\n\
             {\"ts\":\"2026-01-01T00:01:00Z\",\"note\":\"note 2\"}\n\
             {\"ts\":\"2026-01-01T00:02:00Z\",\"note\":\"note 3\"}\n\
             {\"ts\":\"2026-01-01T00:03:00Z\",\"note\":\"note 4\"}\n",
        ).unwrap();

        let fake = FakeSummarizer::new(Ok("implementing worker status digest".to_string()));
        let (code, val, line) = status_digest_with_summarizer(root, "job-4", true, &fake);
        assert_eq!(code, ExitCode::SUCCESS);
        assert_eq!(line, "implementing worker status digest");
        assert_eq!(val["source"], "supervisor");
        assert_eq!(val["notes"], 4);
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);

        let digest_file = job_dir.join("status-digest.json");
        assert!(digest_file.exists());
        let digest_val: Value = serde_json::from_str(&std::fs::read_to_string(&digest_file).unwrap()).unwrap();
        assert_eq!(digest_val["notes"], 4);
        assert_eq!(digest_val["line"], "implementing worker status digest");
        assert!(digest_val["at"].is_string());

        let (code2, val2, line2) = status_digest_with_summarizer(root, "job-4", true, &PanickingSummarizer);
        assert_eq!(code2, ExitCode::SUCCESS);
        assert_eq!(line2, "implementing worker status digest");
        assert_eq!(val2["source"], "cached");
        assert_eq!(val2["notes"], 4);
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn status_job_more_than_three_notes_failing_supervisor_falls_back_to_verbatim() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let job_dir = root.join(".bee/mailbox/job-fail");
        std::fs::create_dir_all(&job_dir).unwrap();
        std::fs::write(
            job_dir.join("progress.jsonl"),
            "{\"ts\":\"2026-01-01T00:00:00Z\",\"note\":\"note 1\"}\n\
             {\"ts\":\"2026-01-01T00:01:00Z\",\"note\":\"note 2\"}\n\
             {\"ts\":\"2026-01-01T00:02:00Z\",\"note\":\"note 3\"}\n\
             {\"ts\":\"2026-01-01T00:03:00Z\",\"step\":\"build\",\"note\":\"note 4\"}\n",
        ).unwrap();

        let fake = FakeSummarizer::new(Err("supervisor unreachable".to_string()));
        let (code, val, line) = status_digest_with_summarizer(root, "job-fail", true, &fake);
        assert_eq!(code, ExitCode::SUCCESS);
        assert_eq!(line, "build: note 4");
        assert_eq!(val["source"], "verbatim");
        assert_eq!(val["notes"], 4);
        assert_eq!(val["fallback"], "supervisor unreachable");
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn status_job_skips_torn_last_line() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let job_dir = root.join(".bee/mailbox/job-torn");
        std::fs::create_dir_all(&job_dir).unwrap();
        std::fs::write(
            job_dir.join("progress.jsonl"),
            "{\"ts\":\"2026-01-01T00:00:00Z\",\"note\":\"valid note\"}\n\
             {\"ts\":\"2026-01-01T00:01:00Z\",\"note\":\"torn",
        ).unwrap();

        let (code, val, line) = status_digest_with_summarizer(root, "job-torn", true, &PanickingSummarizer);
        assert_eq!(code, ExitCode::SUCCESS);
        assert_eq!(line, "valid note");
        assert_eq!(val["notes"], 1);
        assert_eq!(val["source"], "verbatim");
    }
}
