// herding::job_verbs — interrupt and cancel verbs (herding-cockpit-completeness D1, D4)
//
// D1 (c943feb9): `bee herding interrupt <job-id>` sends Escape to the job's
// recorded pane, keeps the pane open, and marks the job interrupted in
// job.json. The pane stays open so `--continue` still has a pane.
//
// D4 (7172010b): `bee herding cancel <job-id>` is fail-closed: capture the
// pane's foreground process id, close the pane, confirm exit within 5 s,
// then record outcome cancelled in job.json. If exit is unconfirmed within
// 5 s, exit non-zero with `cancel_termination_failed` and leave the mark as
// `cancel_pending`. A second cancel on a cancel_pending job skips the pane
// close and only re-confirms the pid. When process_info is Absent, close the
// pane and record cancelled at once. Unknown process_info refuses typed.

use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

use serde_json::Value;

use super::mailbox::{self, Mark};
use super::run::{self, Liveness, PaneTransport};
use super::{resolve_main_root, tmux, TransportKind};

const DEFAULT_POLL_TIMEOUT: Duration = Duration::from_millis(5000);
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub(crate) fn transport_for_run(main_root: &Path) -> Result<Box<dyn PaneTransport>, String> {
    let kind = crate::herding::transport_kind_at(main_root)?;
    let cfg = run::read_main_config(main_root);
    match kind {
        TransportKind::Herdr => Ok(Box::new(run::RealHerdr)),
        TransportKind::Tmux => Ok(Box::new(tmux::RealTmux::new(tmux::TmuxSettings::from_config(&cfg)))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JobVerbError {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

impl std::fmt::Display for JobVerbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

struct ParsedArgs<'a> {
    job_id: Option<&'a str>,
    main_root: Option<&'a str>,
    json: bool,
}

fn parse_args<'a>(args: &[&'a str]) -> ParsedArgs<'a> {
    let mut job_id = None;
    let mut main_root = None;
    let mut json = false;
    let mut i = 0usize;
    while i < args.len() {
        match args[i] {
            "--json" => {
                json = true;
                i += 1;
            }
            "--main-root" => {
                main_root = args.get(i + 1).copied();
                i += 2;
            }
            arg if arg.starts_with("--main-root=") => {
                main_root = Some(&arg["--main-root=".len()..]);
                i += 1;
            }
            arg if !arg.starts_with('-') && job_id.is_none() => {
                job_id = Some(arg);
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    ParsedArgs { job_id, main_root, json }
}

fn read_job_spec(
    bee_dir: &Path,
    job_id: &str,
    verb: &str,
) -> Result<(Value, String), JobVerbError> {
    let job_path = mailbox::job_path(bee_dir, job_id);
    let raw = match crate::fsutil::read_json(&job_path) {
        crate::fsutil::ReadJson::Parsed(v) => v,
        _ => {
            return Err(JobVerbError {
                code: "job_not_found",
                message: format!(
                    "herding {verb}: job \"{job_id}\" not found (no job.json in .bee/mailbox/{job_id}/). FIX: check the job id with bee herding status"
                ),
            });
        }
    };
    let pane_id = raw
        .get("pane_id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if pane_id.is_empty() {
        return Err(JobVerbError {
            code: "pane_missing",
            message: format!(
                "herding {verb}: job \"{job_id}\" has no recorded pane_id in job.json. FIX: check the job with bee herding status"
            ),
        });
    }
    Ok((raw, pane_id))
}

pub(crate) fn interrupt_with_transport(
    main_root: &Path,
    job_id: &str,
    json: bool,
    transport: &dyn PaneTransport,
) -> Result<(), JobVerbError> {
    let bee_dir = main_root.join(".bee");
    let (_job_spec, pane_id) = read_job_spec(&bee_dir, job_id, "interrupt")?;

    if let Some((mark, _)) = mailbox::read_mark(&bee_dir, job_id) {
        if matches!(mark, Mark::Cancelled | Mark::CancelPending) {
            return Err(JobVerbError {
                code: "already_cancelled",
                message: format!(
                    "herding interrupt: job \"{job_id}\" mark is already {} — cannot interrupt. FIX: start a new job or run bee herding status",
                    mark.as_str()
                ),
            });
        }
    }

    if !transport.pane_alive(&pane_id) {
        return Err(JobVerbError {
            code: "pane_missing",
            message: format!(
                "herding interrupt: pane \"{pane_id}\" for job \"{job_id}\" is missing or dead. FIX: check pane with bee herding pane list"
            ),
        });
    }

    let kind = crate::herding::transport_kind_at(main_root).unwrap_or(TransportKind::Herdr);
    let key = if kind == TransportKind::Tmux || transport.name() == "tmux" {
        "Escape"
    } else {
        "esc"
    };

    transport.pane_send_key(&pane_id, key).map_err(|e| JobVerbError {
        code: "send_key_failed",
        message: format!(
            "herding interrupt: failed to send key to pane \"{pane_id}\": {e}. FIX: check pane with bee herding pane list"
        ),
    })?;

    mailbox::write_mark(&bee_dir, job_id, Mark::Interrupted, "user").map_err(|e| {
        JobVerbError {
            code: "write_mark_failed",
            message: format!(
                "herding interrupt: could not write mark: {e}. FIX: check permissions in .bee/mailbox/{job_id}/"
            ),
        }
    })?;

    if json {
        let out = serde_json::json!({
            "job_id": job_id,
            "outcome": "interrupted",
            "pane_id": pane_id,
            "closed_pane": false,
        });
        println!("{}", serde_json::to_string(&out).unwrap());
    } else {
        println!("herding: interrupted job {job_id} (pane kept open)");
    }

    Ok(())
}

pub(crate) fn cancel_with_transport(
    main_root: &Path,
    job_id: &str,
    json: bool,
    transport: &dyn PaneTransport,
) -> Result<(), JobVerbError> {
    cancel_with_transport_and_timeout(
        main_root,
        job_id,
        json,
        transport,
        DEFAULT_POLL_TIMEOUT,
        DEFAULT_POLL_INTERVAL,
    )
}

pub(crate) fn cancel_with_transport_and_timeout(
    main_root: &Path,
    job_id: &str,
    json: bool,
    transport: &dyn PaneTransport,
    poll_timeout: Duration,
    poll_interval: Duration,
) -> Result<(), JobVerbError> {
    let bee_dir = main_root.join(".bee");
    let (job_spec, pane_id) = read_job_spec(&bee_dir, job_id, "cancel")?;

    let current_mark = mailbox::read_mark(&bee_dir, job_id);
    if let Some((Mark::Cancelled, _)) = current_mark {
        return Err(JobVerbError {
            code: "already_cancelled",
            message: format!(
                "herding cancel: job \"{job_id}\" is already cancelled. FIX: check job status with bee herding status"
            ),
        });
    }

    // A second cancel on a cancel_pending job skips the pane step and only re-confirms the pid.
    let is_cancel_pending = matches!(current_mark, Some((Mark::CancelPending, _)));

    let pid = if is_cancel_pending {
        // Retrieve recorded pid from job.json
        job_spec
            .get("cancel_pid")
            .and_then(Value::as_u64)
            .map(|n| n as u32)
    } else {
        match transport.process_info(&pane_id) {
            Liveness::Unknown => {
                return Err(JobVerbError {
                    code: "cancel_unknown",
                    message: format!(
                        "herding cancel: process info for pane \"{pane_id}\" is unknown — cannot confirm cancel. FIX: inspect pane with bee herding pane list"
                    ),
                });
            }
            Liveness::Absent => {
                let _ = transport.pane_close(&pane_id);
                mailbox::write_mark(&bee_dir, job_id, Mark::Cancelled, "user").map_err(|e| {
                    JobVerbError {
                        code: "write_mark_failed",
                        message: format!("herding cancel: could not write mark: {e}. FIX: check permissions in .bee/mailbox/{job_id}/"),
                    }
                })?;
                if json {
                    let out = serde_json::json!({
                        "job_id": job_id,
                        "outcome": "cancelled",
                        "pane_id": pane_id,
                        "closed_pane": true,
                    });
                    println!("{}", serde_json::to_string(&out).unwrap());
                } else {
                    println!("herding: cancelled job {job_id} (pane closed)");
                }
                return Ok(());
            }
            Liveness::Alive { pid } => {
                // Write mark cancel_pending
                mailbox::write_mark(&bee_dir, job_id, Mark::CancelPending, "user").map_err(|e| {
                    JobVerbError {
                        code: "write_mark_failed",
                        message: format!("herding cancel: could not write mark: {e}. FIX: check permissions in .bee/mailbox/{job_id}/"),
                    }
                })?;
                // Save cancel_pid in job.json
                let job_path = mailbox::job_path(&bee_dir, job_id);
                if let crate::fsutil::ReadJson::Parsed(Value::Object(mut map)) =
                    crate::fsutil::read_json(&job_path)
                {
                    map.insert("cancel_pid".to_string(), Value::Number(pid.into()));
                    let _ = crate::fsutil::write_json_atomic(&job_path, &Value::Object(map));
                }
                // Close pane
                let _ = transport.pane_close(&pane_id);
                Some(pid)
            }
        }
    };

    if let Some(pid) = pid {
        let mut gone = false;
        let steps = (poll_timeout.as_millis() / poll_interval.as_millis().max(1)).max(1);
        for _ in 0..steps {
            if !crate::lock::is_pid_alive(Some(pid as f64)) {
                gone = true;
                break;
            }
            std::thread::sleep(poll_interval);
        }
        if !gone && !crate::lock::is_pid_alive(Some(pid as f64)) {
            gone = true;
        }

        if gone {
            mailbox::write_mark(&bee_dir, job_id, Mark::Cancelled, "user").map_err(|e| {
                JobVerbError {
                    code: "write_mark_failed",
                    message: format!("herding cancel: could not write mark: {e}. FIX: check permissions in .bee/mailbox/{job_id}/"),
                }
            })?;
            if json {
                let out = serde_json::json!({
                    "job_id": job_id,
                    "outcome": "cancelled",
                    "pane_id": pane_id,
                    "closed_pane": true,
                    "pid": pid,
                });
                println!("{}", serde_json::to_string(&out).unwrap());
            } else {
                println!("herding: cancelled job {job_id} (pane closed, pid {pid} exited)");
            }
            Ok(())
        } else {
            // Pid still alive after timeout! Exit non-zero, mark stays cancel_pending.
            if json {
                let err_obj = serde_json::json!({
                    "error": "cancel_termination_failed",
                    "job_id": job_id,
                    "pid": pid,
                    "pane_id": pane_id,
                });
                println!("{}", serde_json::to_string(&err_obj).unwrap());
            } else {
                eprintln!(
                    "herding cancel: pid {pid} still alive after {} s. FIX: kill pid {pid} by hand, then run bee herding cancel {job_id} again",
                    poll_timeout.as_secs()
                );
            }
            Err(JobVerbError {
                code: "cancel_termination_failed",
                message: format!("FIX: kill pid {pid} by hand, then run bee herding cancel {job_id} again"),
            })
        }
    } else {
        // No pid was known and mark was cancel_pending without cancel_pid. Close pane and mark cancelled.
        let _ = transport.pane_close(&pane_id);
        mailbox::write_mark(&bee_dir, job_id, Mark::Cancelled, "user").map_err(|e| {
            JobVerbError {
                code: "write_mark_failed",
                message: format!("herding cancel: could not write mark: {e}. FIX: check permissions in .bee/mailbox/{job_id}/"),
            }
        })?;
        if json {
            let out = serde_json::json!({
                "job_id": job_id,
                "outcome": "cancelled",
                "pane_id": pane_id,
                "closed_pane": true,
            });
            println!("{}", serde_json::to_string(&out).unwrap());
        } else {
            println!("herding: cancelled job {job_id} (pane closed)");
        }
        Ok(())
    }
}

pub(super) fn interrupt(args: &[&str]) -> ExitCode {
    let parsed = parse_args(args);
    let Some(job_id) = parsed.job_id else {
        eprintln!("herding interrupt: missing <job-id> positional argument. FIX: run bee herding interrupt <job-id>");
        return ExitCode::FAILURE;
    };
    let Some(main_root) = resolve_main_root(parsed.main_root) else {
        eprintln!("herding interrupt: could not resolve main checkout root. FIX: pass --main-root <path>");
        return ExitCode::FAILURE;
    };
    let transport = match transport_for_run(&main_root) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("herding interrupt: {e}. FIX: check herding.transport in .bee/config.json");
            return ExitCode::FAILURE;
        }
    };
    match interrupt_with_transport(&main_root, job_id, parsed.json, transport.as_ref()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if parsed.json {
                let err_obj = serde_json::json!({
                    "error": e.code,
                    "message": e.message,
                });
                println!("{}", serde_json::to_string(&err_obj).unwrap());
            } else {
                eprintln!("{}", e.message);
            }
            ExitCode::FAILURE
        }
    }
}

pub(super) fn cancel(args: &[&str]) -> ExitCode {
    let parsed = parse_args(args);
    let Some(job_id) = parsed.job_id else {
        eprintln!("herding cancel: missing <job-id> positional argument. FIX: run bee herding cancel <job-id>");
        return ExitCode::FAILURE;
    };
    let Some(main_root) = resolve_main_root(parsed.main_root) else {
        eprintln!("herding cancel: could not resolve main checkout root. FIX: pass --main-root <path>");
        return ExitCode::FAILURE;
    };
    let transport = match transport_for_run(&main_root) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("herding cancel: {e}. FIX: check herding.transport in .bee/config.json");
            return ExitCode::FAILURE;
        }
    };
    match cancel_with_transport(&main_root, job_id, parsed.json, transport.as_ref()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if e.code != "cancel_termination_failed" {
                if parsed.json {
                    let err_obj = serde_json::json!({
                        "error": e.code,
                        "message": e.message,
                    });
                    println!("{}", serde_json::to_string(&err_obj).unwrap());
                } else {
                    eprintln!("{}", e.message);
                }
            }
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use super::run::PaneGeom;

    struct TestTransport {
        name: &'static str,
        sent_keys: RefCell<Vec<(String, String)>>,
        closed_panes: RefCell<Vec<String>>,
        alive_panes: RefCell<Vec<String>>,
        process_info_results: RefCell<Vec<Liveness>>,
        process_info_calls: RefCell<Vec<String>>,
    }

    impl TestTransport {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                sent_keys: RefCell::new(Vec::new()),
                closed_panes: RefCell::new(Vec::new()),
                alive_panes: RefCell::new(vec!["p1".to_string()]),
                process_info_results: RefCell::new(Vec::new()),
                process_info_calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl PaneTransport for TestTransport {
        fn pane_current(&self) -> Result<String, String> {
            Ok("p0".to_string())
        }
        fn pane_layout(&self, _pane_id: &str) -> Option<Vec<PaneGeom>> {
            None
        }
        fn pane_split(&self, _pane_id: &str, _dir: &str, _ratio: f64, _cwd: &Path) -> Result<String, String> {
            Ok("p1".to_string())
        }
        fn tab_create(&self, _ws: &str, _cwd: &Path, _label: &str) -> Result<String, String> {
            Ok("p1".to_string())
        }
        fn pane_run(&self, _pane: &str, _cmd: &str) -> Result<(), String> {
            Ok(())
        }
        fn agent_start(&self, _job: &str, _kind: &str, _pane: &str, _args: &[String]) -> Result<(), String> {
            Ok(())
        }
        fn agent_status(&self, _job: &str) -> Option<String> {
            Some("working".to_string())
        }
        fn pane_close(&self, pane_id: &str) -> Result<(), String> {
            self.closed_panes.borrow_mut().push(pane_id.to_string());
            Ok(())
        }
        fn pane_send_key(&self, pane_id: &str, key: &str) -> Result<(), String> {
            self.sent_keys.borrow_mut().push((pane_id.to_string(), key.to_string()));
            Ok(())
        }
        fn agent_prompt(&self, _job: &str, _prompt: &str, _until: &str, _timeout_ms: u64) -> Result<(), String> {
            Ok(())
        }
        fn agent_wait(&self, _job: &str, _timeout_ms: u64) -> Option<String> {
            None
        }
        fn pane_alive(&self, pane_id: &str) -> bool {
            self.alive_panes.borrow().iter().any(|p| p == pane_id)
        }
        fn pane_read(&self, _pane_id: &str) -> Result<String, String> {
            Ok(String::new())
        }
        fn process_info(&self, pane_id: &str) -> Liveness {
            self.process_info_calls.borrow_mut().push(pane_id.to_string());
            if self.closed_panes.borrow().contains(&pane_id.to_string()) {
                Liveness::Absent
            } else if !self.process_info_results.borrow().is_empty() {
                self.process_info_results.borrow_mut().remove(0)
            } else {
                Liveness::Unknown
            }
        }
        fn name(&self) -> &'static str {
            self.name
        }
    }

    fn seed_job(bee_dir: &Path, job_id: &str, pane_id: &str) {
        let job_file = mailbox::job_path(bee_dir, job_id);
        let mut obj = serde_json::Map::new();
        obj.insert("job_id".to_string(), Value::String(job_id.to_string()));
        obj.insert("pane_id".to_string(), Value::String(pane_id.to_string()));
        crate::fsutil::write_json_atomic(&job_file, &Value::Object(obj)).unwrap();
    }

    #[test]
    fn interrupt_sends_key_and_writes_mark() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        seed_job(&bee_dir, "job-interrupt-1", "p1");

        let transport = TestTransport::new("herdr");
        let res = interrupt_with_transport(tmp.path(), "job-interrupt-1", false, &transport);
        assert!(res.is_ok(), "interrupt succeeds: {res:?}");

        assert_eq!(
            *transport.sent_keys.borrow(),
            vec![("p1".to_string(), "esc".to_string())]
        );
        assert!(
            transport.closed_panes.borrow().is_empty(),
            "interrupt never closes the pane"
        );

        let mark = mailbox::read_mark(&bee_dir, "job-interrupt-1");
        assert_eq!(mark, Some((Mark::Interrupted, Some("user".to_string()))));
    }

    #[test]
    fn interrupt_on_tmux_sends_escape_key() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        seed_job(&bee_dir, "job-interrupt-tmux", "p1");

        let transport = TestTransport::new("tmux");
        let res = interrupt_with_transport(tmp.path(), "job-interrupt-tmux", false, &transport);
        assert!(res.is_ok());

        assert_eq!(
            *transport.sent_keys.borrow(),
            vec![("p1".to_string(), "Escape".to_string())]
        );
    }

    #[test]
    fn interrupt_refuses_on_a_cancelled_mark() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        seed_job(&bee_dir, "job-cancelled", "p1");
        mailbox::write_mark(&bee_dir, "job-cancelled", Mark::Cancelled, "user").unwrap();

        let transport = TestTransport::new("herdr");
        let err = interrupt_with_transport(tmp.path(), "job-cancelled", false, &transport).unwrap_err();
        assert_eq!(err.code, "already_cancelled");
        assert!(err.message.contains("FIX:"), "must contain FIX line: {err}");

        let err_pending = {
            seed_job(&bee_dir, "job-pending", "p1");
            mailbox::write_mark(&bee_dir, "job-pending", Mark::CancelPending, "user").unwrap();
            interrupt_with_transport(tmp.path(), "job-pending", false, &transport).unwrap_err()
        };
        assert!(err_pending.message.contains("FIX:"));
    }

    #[test]
    fn cancel_confirms_exit_and_marks_cancelled() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        seed_job(&bee_dir, "job-cancel-absent", "p1");

        let transport = TestTransport::new("herdr");
        transport.process_info_results.borrow_mut().push(Liveness::Absent);

        let res = cancel_with_transport(tmp.path(), "job-cancel-absent", false, &transport);
        assert!(res.is_ok(), "cancel on absent process succeeds: {res:?}");
        assert_eq!(*transport.closed_panes.borrow(), vec!["p1".to_string()]);
        assert_eq!(
            mailbox::read_mark(&bee_dir, "job-cancel-absent"),
            Some((Mark::Cancelled, Some("user".to_string())))
        );

        // Also test when process_info returns Alive with an already-dead pid
        seed_job(&bee_dir, "job-cancel-dead-pid", "p1");
        let dead_pid = 999_999_999u32;
        transport.process_info_results.borrow_mut().push(Liveness::Alive { pid: dead_pid });
        let res2 = cancel_with_transport(tmp.path(), "job-cancel-dead-pid", false, &transport);
        assert!(res2.is_ok(), "cancel on dead pid confirms exit: {res2:?}");
        assert_eq!(
            mailbox::read_mark(&bee_dir, "job-cancel-dead-pid"),
            Some((Mark::Cancelled, Some("user".to_string())))
        );
    }

    #[test]
    fn cancel_on_fake_pid_that_stays_alive_reports_cancel_termination_failed() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        seed_job(&bee_dir, "job-alive-pid", "p1");

        let current_pid = std::process::id();
        let transport = TestTransport::new("herdr");
        transport.process_info_results.borrow_mut().push(Liveness::Alive { pid: current_pid });

        let err = cancel_with_transport_and_timeout(
            tmp.path(),
            "job-alive-pid",
            false,
            &transport,
            Duration::from_millis(150),
            Duration::from_millis(50),
        )
        .unwrap_err();

        assert_eq!(err.code, "cancel_termination_failed");
        assert!(err.message.contains("FIX: kill pid"), "message must have FIX line: {err}");

        // Mark stays cancel_pending
        let mark = mailbox::read_mark(&bee_dir, "job-alive-pid");
        assert_eq!(mark, Some((Mark::CancelPending, Some("user".to_string()))));
    }

    #[test]
    fn unknown_job_refuses_with_a_fix_line() {
        let tmp = tempfile::tempdir().unwrap();
        let transport = TestTransport::new("herdr");

        let err_int = interrupt_with_transport(tmp.path(), "nonexistent-job", false, &transport).unwrap_err();
        assert_eq!(err_int.code, "job_not_found");
        assert!(err_int.message.contains("FIX:"), "interrupt unknown job needs FIX line: {err_int}");

        let err_canc = cancel_with_transport(tmp.path(), "nonexistent-job", false, &transport).unwrap_err();
        assert_eq!(err_canc.code, "job_not_found");
        assert!(err_canc.message.contains("FIX:"), "cancel unknown job needs FIX line: {err_canc}");
    }

    #[test]
    fn cancel_second_attempt_on_cancel_pending_skips_pane_and_confirms_exit() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        seed_job(&bee_dir, "job-pending-retry", "p1");

        // Write cancel_pending mark and cancel_pid to job.json
        mailbox::write_mark(&bee_dir, "job-pending-retry", Mark::CancelPending, "user").unwrap();
        let job_path = mailbox::job_path(&bee_dir, "job-pending-retry");
        let dead_pid = 999_999_999u32;
        if let crate::fsutil::ReadJson::Parsed(Value::Object(mut map)) = crate::fsutil::read_json(&job_path) {
            map.insert("cancel_pid".to_string(), Value::Number(dead_pid.into()));
            crate::fsutil::write_json_atomic(&job_path, &Value::Object(map)).unwrap();
        }

        let transport = TestTransport::new("herdr");
        let res = cancel_with_transport(tmp.path(), "job-pending-retry", false, &transport);
        assert!(res.is_ok());

        assert!(
            transport.closed_panes.borrow().is_empty(),
            "second cancel skips pane close"
        );
        assert!(
            transport.process_info_calls.borrow().is_empty(),
            "second cancel skips process_info"
        );
        assert_eq!(
            mailbox::read_mark(&bee_dir, "job-pending-retry"),
            Some((Mark::Cancelled, Some("user".to_string())))
        );
    }
}
