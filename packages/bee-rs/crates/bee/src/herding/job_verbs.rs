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
    text: Option<&'a str>,
    json: bool,
}

fn parse_args<'a>(args: &[&'a str]) -> ParsedArgs<'a> {
    let mut job_id = None;
    let mut main_root = None;
    let mut text = None;
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
            "--text" => {
                text = args.get(i + 1).copied();
                i += 2;
            }
            arg if arg.starts_with("--text=") => {
                text = Some(&arg["--text=".len()..]);
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
    ParsedArgs { job_id, main_root, text, json }
}

fn read_job_json(
    bee_dir: &Path,
    job_id: &str,
    verb: &str,
) -> Result<Value, JobVerbError> {
    let job_path = mailbox::job_path(bee_dir, job_id);
    match crate::fsutil::read_json(&job_path) {
        crate::fsutil::ReadJson::Parsed(v) => Ok(v),
        _ => Err(JobVerbError {
            code: "job_not_found",
            message: format!(
                "herding {verb}: job \"{job_id}\" not found (no job.json in .bee/mailbox/{job_id}/). FIX: check the job id with bee herding status"
            ),
        }),
    }
}

#[allow(dead_code)]
pub(crate) fn interrupt_with_transport(
    main_root: &Path,
    job_id: &str,
    json: bool,
    transport: &dyn PaneTransport,
) -> Result<(), JobVerbError> {
    interrupt_with_backends(main_root, job_id, json, Some(transport), None)
}

pub(crate) fn interrupt_with_backends(
    main_root: &Path,
    job_id: &str,
    json: bool,
    transport: Option<&dyn PaneTransport>,
    paseo_cli: Option<&dyn crate::herding::paseo::PaseoCli>,
) -> Result<(), JobVerbError> {
    let bee_dir = main_root.join(".bee");
    let job_raw = read_job_json(&bee_dir, job_id, "interrupt")?;

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

    let is_paseo = job_raw.get("transport").and_then(Value::as_str) == Some("paseo");
    if is_paseo {
        let paseo_agent_id = match job_raw
            .get("paseo_agent_id")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
        {
            Some(id) => id,
            None => {
                return Err(JobVerbError {
                    code: "paseo_agent_missing",
                    message: format!(
                        "herding interrupt: job \"{job_id}\" has no recorded paseo_agent_id in job.json. FIX: check the job with bee herding status"
                    ),
                });
            }
        };

        let stop_cmd = crate::herding::paseo::stop_argv(paseo_agent_id);
        let real_cli;
        let cli = match paseo_cli {
            Some(c) => c,
            None => {
                let cfg = run::read_main_config(main_root);
                let cmd = crate::herding::paseo::paseo_command(&cfg);
                real_cli = crate::herding::paseo::RealPaseoCli::new(cmd).with_timeout(Duration::from_secs(5));
                &real_cli
            }
        };

        cli.call(&stop_cmd).map_err(|e| JobVerbError {
            code: "stop_failed",
            message: format!(
                "herding interrupt: failed to stop agent \"{paseo_agent_id}\": {e}. FIX: check the Paseo daemon"
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
                "paseo_agent_id": paseo_agent_id,
                "closed_pane": false,
            });
            println!("{}", serde_json::to_string(&out).unwrap());
        } else {
            println!("herding: interrupted job {job_id} (agent kept)");
        }

        Ok(())
    } else {
        let pane_id = job_raw
            .get("pane_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if pane_id.is_empty() {
            return Err(JobVerbError {
                code: "pane_missing",
                message: format!(
                    "herding interrupt: job \"{job_id}\" has no recorded pane_id in job.json. FIX: check the job with bee herding status"
                ),
            });
        }

        let real_trans;
        let trans = match transport {
            Some(t) => t,
            None => {
                real_trans = transport_for_run(main_root).map_err(|e| JobVerbError {
                    code: "transport_error",
                    message: format!("herding interrupt: {e}. FIX: check herding.transport in .bee/config.json"),
                })?;
                real_trans.as_ref()
            }
        };

        if !trans.pane_alive(&pane_id) {
            return Err(JobVerbError {
                code: "pane_missing",
                message: format!(
                    "herding interrupt: pane \"{pane_id}\" for job \"{job_id}\" is missing or dead. FIX: check pane with bee herding pane list"
                ),
            });
        }

        let kind = crate::herding::transport_kind_at(main_root).unwrap_or(TransportKind::Herdr);
        let key = if kind == TransportKind::Tmux || trans.name() == "tmux" {
            "Escape"
        } else {
            "esc"
        };

        trans.pane_send_key(&pane_id, key).map_err(|e| JobVerbError {
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
}

const TREE_KILL_GRACE: Duration = Duration::from_millis(2000);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TreeSignal {
    Term,
    Kill,
}

pub(crate) trait ProcessTree {
    fn read(&self) -> Option<Vec<(u32, u32, u32)>>;
    fn signal(&self, pid: u32, sig: TreeSignal);
}

pub(crate) struct RealProcessTree;

impl ProcessTree for RealProcessTree {
    #[cfg(unix)]
    fn read(&self) -> Option<Vec<(u32, u32, u32)>> {
        let out = std::process::Command::new("ps")
            .args(["-A", "-o", "pid=,ppid=,pgid="])
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let rows = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|line| {
                let mut cols = line.split_whitespace().map(|c| c.parse::<u32>().ok());
                Some((cols.next()??, cols.next()??, cols.next()??))
            })
            .collect();
        Some(rows)
    }

    #[cfg(not(unix))]
    fn read(&self) -> Option<Vec<(u32, u32, u32)>> {
        None
    }

    #[cfg(unix)]
    fn signal(&self, pid: u32, sig: TreeSignal) {
        let signo = match sig {
            TreeSignal::Term => libc::SIGTERM,
            TreeSignal::Kill => libc::SIGKILL,
        };
        unsafe { libc::kill(pid as i32, signo) };
    }

    #[cfg(not(unix))]
    fn signal(&self, _pid: u32, _sig: TreeSignal) {}
}

fn tree_members(rows: &[(u32, u32, u32)], root: u32) -> Vec<u32> {
    let own = std::process::id();
    let mut members = vec![root];
    loop {
        let before = members.len();
        for &(pid, ppid, pgid) in rows {
            if pid <= 1 || pid == own || members.contains(&pid) {
                continue;
            }
            if members.contains(&ppid) || (pgid != pid && members.contains(&pgid)) {
                members.push(pid);
            }
        }
        if members.len() == before {
            return members;
        }
    }
}

pub(crate) fn kill_tree(tree: &dyn ProcessTree, root: u32, grace: Duration) {
    let Some(rows) = tree.read() else { return };
    let members = tree_members(&rows, root);
    for &pid in &members {
        tree.signal(pid, TreeSignal::Term);
    }
    let deadline = std::time::Instant::now() + grace;
    loop {
        let alive: Vec<u32> = match tree.read() {
            Some(now) => members
                .iter()
                .copied()
                .filter(|m| now.iter().any(|&(pid, _, _)| pid == *m))
                .collect(),
            None => members.clone(),
        };
        if alive.is_empty() {
            return;
        }
        if std::time::Instant::now() >= deadline {
            for pid in alive {
                tree.signal(pid, TreeSignal::Kill);
            }
            return;
        }
        std::thread::sleep(DEFAULT_POLL_INTERVAL);
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cancel_with_backends_and_timeout(
    main_root: &Path,
    job_id: &str,
    json: bool,
    transport: Option<&dyn PaneTransport>,
    paseo_cli: Option<&dyn crate::herding::paseo::PaseoCli>,
    tree: &dyn ProcessTree,
    poll_timeout: Duration,
    poll_interval: Duration,
) -> Result<(), JobVerbError> {
    let bee_dir = main_root.join(".bee");
    let job_raw = read_job_json(&bee_dir, job_id, "cancel")?;

    let current_mark = mailbox::read_mark(&bee_dir, job_id);
    if let Some((Mark::Cancelled, _)) = current_mark {
        return Err(JobVerbError {
            code: "already_cancelled",
            message: format!(
                "herding cancel: job \"{job_id}\" is already cancelled. FIX: check job status with bee herding status"
            ),
        });
    }

    let is_paseo = job_raw.get("transport").and_then(Value::as_str) == Some("paseo");
    if is_paseo {
        let paseo_agent_id = match job_raw
            .get("paseo_agent_id")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
        {
            Some(id) => id,
            None => {
                return Err(JobVerbError {
                    code: "paseo_agent_missing",
                    message: format!(
                        "herding cancel: job \"{job_id}\" has no recorded paseo_agent_id in job.json. FIX: check the job with bee herding status"
                    ),
                });
            }
        };

        let is_cancel_pending = matches!(current_mark, Some((Mark::CancelPending, _)));
        if !is_cancel_pending {
            mailbox::write_mark(&bee_dir, job_id, Mark::CancelPending, "user").map_err(|e| {
                JobVerbError {
                    code: "write_mark_failed",
                    message: format!("herding cancel: could not write mark: {e}. FIX: check permissions in .bee/mailbox/{job_id}/"),
                }
            })?;
        }

        let is_own_agent = std::env::var("PASEO_AGENT_ID")
            .map(|own| !own.trim().is_empty() && own.trim() == paseo_agent_id.trim())
            .unwrap_or(false);

        if is_own_agent {
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
                    "paseo_agent_id": paseo_agent_id,
                    "skipped_own_agent": true,
                    "message": format!("skipped stop and archive for caller's own agent {paseo_agent_id}"),
                });
                println!("{}", serde_json::to_string(&out).unwrap());
            } else {
                println!("herding: cancelled job {job_id} (skipped stop and archive for caller's own agent {paseo_agent_id})");
            }
            return Ok(());
        }

        let real_cli;
        let cli = match paseo_cli {
            Some(c) => c,
            None => {
                let cfg = run::read_main_config(main_root);
                let cmd = crate::herding::paseo::paseo_command(&cfg);
                real_cli = crate::herding::paseo::RealPaseoCli::new(cmd).with_timeout(Duration::from_secs(5));
                &real_cli
            }
        };

        let stop_cmd = crate::herding::paseo::stop_argv(paseo_agent_id);
        cli.call(&stop_cmd).map_err(|e| JobVerbError {
            code: "stop_failed",
            message: format!("herding cancel: failed to stop agent \"{paseo_agent_id}\": {e}. FIX: check the Paseo daemon"),
        })?;

        let archive_cmd = crate::herding::paseo::archive_argv(paseo_agent_id);
        cli.call(&archive_cmd).map_err(|e| JobVerbError {
            code: "archive_failed",
            message: format!("herding cancel: failed to archive agent \"{paseo_agent_id}\": {e}. FIX: check the Paseo daemon"),
        })?;

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
                "paseo_agent_id": paseo_agent_id,
                "closed_pane": false,
            });
            println!("{}", serde_json::to_string(&out).unwrap());
        } else {
            println!("herding: cancelled job {job_id} (agent stopped and archived)");
        }

        Ok(())
    } else {
        let pane_id = job_raw
            .get("pane_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if pane_id.is_empty() {
            return Err(JobVerbError {
                code: "pane_missing",
                message: format!(
                    "herding cancel: job \"{job_id}\" has no recorded pane_id in job.json. FIX: check the job with bee herding status"
                ),
            });
        }

        let real_trans;
        let trans = match transport {
            Some(t) => t,
            None => {
                real_trans = transport_for_run(main_root).map_err(|e| JobVerbError {
                    code: "transport_error",
                    message: format!("herding cancel: {e}. FIX: check herding.transport in .bee/config.json"),
                })?;
                real_trans.as_ref()
            }
        };

        let is_cancel_pending = matches!(current_mark, Some((Mark::CancelPending, _)));

        let pid = if is_cancel_pending {
            job_raw
                .get("cancel_pid")
                .and_then(Value::as_u64)
                .map(|n| n as u32)
        } else {
            match trans.process_info(&pane_id) {
                Liveness::Unknown => {
                    return Err(JobVerbError {
                        code: "cancel_unknown",
                        message: format!(
                            "herding cancel: process info for pane \"{pane_id}\" is unknown — cannot confirm cancel. FIX: inspect pane with bee herding pane list"
                        ),
                    });
                }
                Liveness::Absent => {
                    let _ = trans.pane_close(&pane_id);
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
                    mailbox::write_mark(&bee_dir, job_id, Mark::CancelPending, "user").map_err(|e| {
                        JobVerbError {
                            code: "write_mark_failed",
                            message: format!("herding cancel: could not write mark: {e}. FIX: check permissions in .bee/mailbox/{job_id}/"),
                        }
                    })?;
                    let job_path = mailbox::job_path(&bee_dir, job_id);
                    if let crate::fsutil::ReadJson::Parsed(Value::Object(mut map)) =
                        crate::fsutil::read_json(&job_path)
                    {
                        map.insert("cancel_pid".to_string(), Value::Number(pid.into()));
                        let _ = crate::fsutil::write_json_atomic(&job_path, &Value::Object(map));
                    }
                    kill_tree(tree, pid, TREE_KILL_GRACE);
                    let _ = trans.pane_close(&pane_id);
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
            let _ = trans.pane_close(&pane_id);
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
    match interrupt_with_backends(&main_root, job_id, parsed.json, None, None) {
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
    match cancel_with_backends_and_timeout(
        &main_root,
        job_id,
        parsed.json,
        None,
        None,
        &RealProcessTree,
        DEFAULT_POLL_TIMEOUT,
        DEFAULT_POLL_INTERVAL,
    ) {
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

fn find_job_cell_id(job_dir: &Path, job_raw: &Value) -> Option<String> {
    if let Some(id) = job_raw.get("cell_id").and_then(Value::as_str) {
        if !id.trim().is_empty() {
            return Some(id.to_string());
        }
    }
    if let Some(id) = job_raw.get("cell").and_then(Value::as_str) {
        if !id.trim().is_empty() {
            return Some(id.to_string());
        }
    }
    if let Ok(rd) = std::fs::read_dir(job_dir) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("ack-") && name.ends_with(".json") {
                if let Ok(raw) = std::fs::read_to_string(entry.path()) {
                    if let Ok(val) = serde_json::from_str::<Value>(&raw) {
                        if let Some(id) = val.get("cell_id").and_then(Value::as_str) {
                            if !id.trim().is_empty() {
                                return Some(id.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    if let Ok(rd) = std::fs::read_dir(job_dir) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("brief-") && name.ends_with(".txt") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("\"cell_id\":") {
                            let parts: Vec<&str> = trimmed.split('"').collect();
                            if parts.len() >= 4 && !parts[3].trim().is_empty() {
                                return Some(parts[3].to_string());
                            }
                        }
                        if let Some(rest) = trimmed.strip_prefix("Assigned cell id:") {
                            let id = rest.trim();
                            if !id.is_empty() {
                                return Some(id.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn steer_job(
    main_root: &Path,
    job_id: &str,
    text: &str,
) -> Result<u32, JobVerbError> {
    steer_job_with(main_root, job_id, text, &crate::herding::paseo_steer::RealSteerRunner)
}

pub(crate) fn steer_job_with(
    main_root: &Path,
    job_id: &str,
    text: &str,
    runner: &dyn crate::herding::paseo_steer::SteerRunner,
) -> Result<u32, JobVerbError> {
    let bee_dir = main_root.join(".bee");
    let mbox = mailbox::mailbox_dir(&bee_dir, job_id);
    if !mbox.is_dir() {
        return Err(JobVerbError {
            code: "job_not_found",
            message: format!(
                "herding steer: job \"{job_id}\" not found (no job directory .bee/mailbox/{job_id}/). FIX: check the job id with bee herding status"
            ),
        });
    }

    let job_path = mailbox::job_path(&bee_dir, job_id);
    let job_raw = match crate::fsutil::read_json(&job_path) {
        crate::fsutil::ReadJson::Parsed(Value::Object(v)) => Value::Object(v),
        _ => {
            return Err(JobVerbError {
                code: "job_not_found",
                message: format!(
                    "herding steer: job \"{job_id}\" has no readable job.json in .bee/mailbox/{job_id}/. FIX: check the job with bee herding status"
                ),
            });
        }
    };

    if let Some(cell_id) = find_job_cell_id(&mbox, &job_raw) {
        let cell_path = bee_dir.join("cells").join(format!("{cell_id}.json"));
        if cell_path.is_file() {
            if let Ok(raw) = std::fs::read_to_string(&cell_path) {
                if let Ok(cell_json) = serde_json::from_str::<Value>(&raw) {
                    let is_capped = cell_json.get("status").and_then(Value::as_str) == Some("capped")
                        || cell_json
                            .get("trace")
                            .and_then(|t| t.get("capped_at"))
                            .and_then(Value::as_str)
                            .is_some_and(|s| !s.trim().is_empty());
                    if is_capped {
                        return Err(JobVerbError {
                            code: "cell_capped",
                            message: format!(
                                "herding steer: cell \"{cell_id}\" for job \"{job_id}\" is already capped — cannot steer. FIX: start a new job or cell"
                            ),
                        });
                    }
                }
            }
        }
    }

    let entries: Vec<String> = match std::fs::read_dir(&mbox) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(e) => {
            return Err(JobVerbError {
                code: "mailbox_read_failed",
                message: format!("herding steer: could not read directory {}: {e}", mbox.display()),
            });
        }
    };

    let base_round = job_raw.get("round").and_then(Value::as_u64).map(|r| r as u32).unwrap_or(1);
    let max_brief_round = entries
        .iter()
        .filter_map(|name| {
            let digits = name.strip_prefix("brief-")?.strip_suffix(".txt")?;
            digits.parse::<u32>().ok()
        })
        .max()
        .unwrap_or(1);
    let current_round = base_round.max(max_brief_round);

    let has_result = entries.iter().any(|name| {
        if let Some(digits) = name.strip_prefix("result-").and_then(|s| s.strip_suffix(".json")) {
            if let Ok(r) = digits.parse::<u32>() {
                return r >= current_round;
            }
        }
        false
    });
    if has_result {
        return Err(JobVerbError {
            code: "result_already_written",
            message: format!(
                "herding steer: job \"{job_id}\" already has a result for round {current_round} — cannot steer. FIX: continue the job with bee herding run --continue {job_id}"
            ),
        });
    }

    let max_steer_n = entries
        .iter()
        .filter_map(|name| {
            let rest = name.strip_prefix("steer-")?;
            let digits = rest.split('.').next()?;
            digits.parse::<u32>().ok()
        })
        .max()
        .unwrap_or(0);
    let next_n = max_steer_n + 1;

    let is_paseo = job_raw.get("transport").and_then(Value::as_str) == Some("paseo");
    if is_paseo {
        let config_path = main_root.join(".bee").join("config.json");
        let cfg = match crate::fsutil::read_json(&config_path) {
            crate::fsutil::ReadJson::Parsed(v) => v,
            _ => Value::Null,
        };
        let agent = job_raw
            .get("agent")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("");
        let spec = match crate::herding::paseo::PaseoSpec::from_config(&cfg, agent) {
            Some(Ok(s)) => s,
            Some(Err(e)) => {
                return Err(JobVerbError {
                    code: "paseo_steer_unsupported",
                    message: format!(
                        "herding steer: failed to read paseo configuration for agent \"{agent}\": {e}. FIX: check herding.agents in .bee/config.json"
                    ),
                });
            }
            None => {
                return Err(JobVerbError {
                    code: "paseo_steer_unsupported",
                    message: format!(
                        "herding steer: agent \"{agent}\" has no paseo configuration in .bee/config.json. FIX: configure herding.agents.{agent}.paseo in .bee/config.json"
                    ),
                });
            }
        };

        if spec.provider != "pi" {
            if !crate::herding::paseo_steer::steerable(&spec.provider) {
                return Err(JobVerbError {
                    code: "paseo_steer_unsupported",
                    message: format!(
                        "herding steer: provider \"{}\" does not support steering on Paseo. FIX: wait for the worker to finish or use a supported provider (claude, codex, opencode, pi)",
                        spec.provider
                    ),
                });
            }

            let paseo_agent_id = match job_raw
                .get("paseo_agent_id")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                Some(id) => id,
                None => {
                    return Err(JobVerbError {
                        code: "paseo_steer_failed",
                        message: format!(
                            "herding steer: job \"{job_id}\" missing paseo_agent_id in job.json. FIX: check the job with bee herding status"
                        ),
                    });
                }
            };

            let paseo_cmd = crate::herding::paseo::paseo_command(&cfg);
            let cli_dir = match crate::herding::paseo_steer::cli_package_dir(&paseo_cmd, |k| std::env::var(k).ok()) {
                Some(d) => d,
                None => {
                    return Err(JobVerbError {
                        code: "paseo_steer_failed",
                        message: "herding steer: could not locate @getpaseo/cli package directory. FIX: install @getpaseo/cli via npm or check your PATH".to_string(),
                    });
                }
            };

            runner
                .steer(&cli_dir, paseo_agent_id, text)
                .map_err(|e| JobVerbError {
                    code: "paseo_steer_failed",
                    message: format!("herding steer: paseo steer failed: {e}. FIX: check the Paseo daemon"),
                })?;

            return Ok(next_n);
        }
    }

    let steer_obj = serde_json::json!({
        "n": next_n,
        "text": text,
        "at": chrono::Utc::now().to_rfc3339(),
    });
    let tmp_file = mbox.join(format!("steer-{next_n}.json.tmp"));
    let final_file = mbox.join(format!("steer-{next_n}.json"));
    let payload = match serde_json::to_string_pretty(&steer_obj) {
        Ok(s) => format!("{s}\n"),
        Err(e) => {
            return Err(JobVerbError {
                code: "json_serialize_failed",
                message: format!("herding steer: failed to serialize steer object: {e}"),
            });
        }
    };
    if let Err(e) = std::fs::write(&tmp_file, payload) {
        return Err(JobVerbError {
            code: "write_steer_failed",
            message: format!("herding steer: failed to write {}: {e}", tmp_file.display()),
        });
    }
    if let Err(e) = std::fs::rename(&tmp_file, &final_file) {
        let _ = std::fs::remove_file(&tmp_file);
        return Err(JobVerbError {
            code: "rename_steer_failed",
            message: format!(
                "herding steer: failed to rename {} to {}: {e}",
                tmp_file.display(),
                final_file.display()
            ),
        });
    }

    Ok(next_n)
}

pub(super) fn steer(args: &[&str]) -> ExitCode {
    let parsed = parse_args(args);
    let Some(job_id) = parsed.job_id else {
        eprintln!("herding steer: missing <job-id> positional argument. FIX: run bee herding steer <job-id> --text <text>");
        return ExitCode::FAILURE;
    };
    let Some(text) = parsed.text else {
        eprintln!("herding steer: missing required --text argument. FIX: run bee herding steer <job-id> --text <text>");
        return ExitCode::FAILURE;
    };
    let Some(main_root) = resolve_main_root(parsed.main_root) else {
        eprintln!("herding steer: could not resolve main checkout root. FIX: pass --main-root <path>");
        return ExitCode::FAILURE;
    };
    match steer_job(&main_root, job_id, text) {
        Ok(n) => {
            if parsed.json {
                let out = serde_json::json!({
                    "job_id": job_id,
                    "steer": n,
                    "n": n,
                });
                println!("{}", serde_json::to_string(&out).unwrap());
            } else {
                println!("herding: steered job {job_id} (steer #{n})");
            }
            ExitCode::SUCCESS
        }
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

pub(crate) fn permit_job(
    main_root: &Path,
    job: &str,
    allow: bool,
    request: Option<&str>,
    all: bool,
    cli: &dyn crate::herding::paseo::PaseoCli,
) -> Result<String, JobVerbError> {
    let bee_dir = main_root.join(".bee");
    let job_raw = read_job_json(&bee_dir, job, "permit")?;

    if job_raw.get("transport").and_then(Value::as_str) != Some("paseo") {
        return Err(JobVerbError {
            code: "not_a_paseo_job",
            message: format!(
                "herding permit: job \"{job}\" is not a Paseo job. FIX: permit only applies to Paseo workers"
            ),
        });
    }

    let paseo_agent_id = match job_raw
        .get("paseo_agent_id")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
    {
        Some(id) => id,
        None => {
            return Err(JobVerbError {
                code: "paseo_agent_missing",
                message: format!(
                    "herding permit: job \"{job}\" has no recorded paseo_agent_id in job.json. FIX: check the job with bee herding status"
                ),
            });
        }
    };

    let argv = crate::herding::paseo::permit_argv(paseo_agent_id, allow, request, all);
    cli.call(&argv).map_err(|e| JobVerbError {
        code: "permit_failed",
        message: format!(
            "herding permit: failed to answer permission for agent \"{paseo_agent_id}\": {e}. FIX: check the Paseo daemon"
        ),
    })
}

struct ParsedPermitArgs<'a> {
    job_id: Option<&'a str>,
    action: Option<&'a str>,
    request: Option<&'a str>,
    all: bool,
    json: bool,
    main_root: Option<&'a str>,
}

fn parse_permit_args<'a>(args: &[&'a str]) -> ParsedPermitArgs<'a> {
    let mut job_id = None;
    let mut action = None;
    let mut request = None;
    let mut all = false;
    let mut json = false;
    let mut main_root = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i] {
            "--json" => {
                json = true;
                i += 1;
            }
            "--all" => {
                all = true;
                i += 1;
            }
            "--request" => {
                request = args.get(i + 1).copied();
                i += 2;
            }
            arg if arg.starts_with("--request=") => {
                request = Some(&arg["--request=".len()..]);
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
            arg if !arg.starts_with('-') => {
                if job_id.is_none() {
                    job_id = Some(arg);
                } else if action.is_none() {
                    action = Some(arg);
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    ParsedPermitArgs {
        job_id,
        action,
        request,
        all,
        json,
        main_root,
    }
}

pub(super) fn permit(args: &[&str]) -> ExitCode {
    let parsed = parse_permit_args(args);
    let Some(job_id) = parsed.job_id else {
        eprintln!("herding permit: missing <job-id> positional argument. FIX: run bee herding permit <job-id> allow|deny");
        return ExitCode::FAILURE;
    };
    let Some(action_str) = parsed.action else {
        eprintln!("herding permit: missing <allow|deny> positional argument. FIX: run bee herding permit {job_id} allow|deny");
        return ExitCode::FAILURE;
    };
    let allow = match action_str {
        "allow" => true,
        "deny" => false,
        other => {
            eprintln!("herding permit: invalid action \"{other}\". FIX: pass allow or deny");
            return ExitCode::FAILURE;
        }
    };
    let Some(main_root) = resolve_main_root(parsed.main_root) else {
        eprintln!("herding permit: could not resolve main checkout root. FIX: pass --main-root <path>");
        return ExitCode::FAILURE;
    };
    let cfg = run::read_main_config(&main_root);
    let cmd = crate::herding::paseo::paseo_command(&cfg);
    let cli = crate::herding::paseo::RealPaseoCli::new(cmd).with_timeout(Duration::from_secs(5));
    match permit_job(&main_root, job_id, allow, parsed.request, parsed.all, &cli) {
        Ok(output) => {
            if parsed.json {
                let out = serde_json::json!({
                    "job_id": job_id,
                    "action": action_str,
                    "output": output.trim(),
                });
                println!("{}", serde_json::to_string(&out).unwrap());
            } else if !output.trim().is_empty() {
                println!("{}", output.trim());
            } else {
                println!("herding: permitted job {job_id} ({action_str})");
            }
            ExitCode::SUCCESS
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use super::run::PaneGeom;

    struct NoopTree;

    impl ProcessTree for NoopTree {
        fn read(&self) -> Option<Vec<(u32, u32, u32)>> {
            None
        }
        fn signal(&self, _pid: u32, _sig: TreeSignal) {}
    }

    fn cancel_with_transport(
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

    fn cancel_with_transport_and_timeout(
        main_root: &Path,
        job_id: &str,
        json: bool,
        transport: &dyn PaneTransport,
        poll_timeout: Duration,
        poll_interval: Duration,
    ) -> Result<(), JobVerbError> {
        cancel_with_backends_and_timeout(
            main_root,
            job_id,
            json,
            Some(transport),
            None,
            &NoopTree,
            poll_timeout,
            poll_interval,
        )
    }

    struct FakeTree {
        rows: Vec<(u32, u32, u32)>,
        signals: RefCell<Vec<(u32, TreeSignal)>>,
    }

    impl ProcessTree for FakeTree {
        fn read(&self) -> Option<Vec<(u32, u32, u32)>> {
            Some(self.rows.clone())
        }
        fn signal(&self, pid: u32, sig: TreeSignal) {
            self.signals.borrow_mut().push((pid, sig));
        }
    }

    #[test]
    fn kill_tree_terms_then_kills_descendants_and_group_members_only() {
        let tree = FakeTree {
            rows: vec![(100, 1, 100), (101, 100, 100), (102, 101, 102), (103, 1, 101), (200, 1, 200)],
            signals: RefCell::new(Vec::new()),
        };
        kill_tree(&tree, 100, Duration::from_millis(0));
        let signals = tree.signals.borrow();
        let termed: Vec<u32> = signals.iter().filter(|s| s.1 == TreeSignal::Term).map(|s| s.0).collect();
        let killed: Vec<u32> = signals.iter().filter(|s| s.1 == TreeSignal::Kill).map(|s| s.0).collect();
        assert_eq!(termed, vec![100, 101, 102, 103]);
        assert_eq!(killed, vec![100, 101, 102, 103]);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn kill_tree_stops_a_detached_setsid_child() {
        use std::io::BufRead;
        let mut child = std::process::Command::new("sh")
            .args(["-c", "setsid sleep 60 & echo $!; wait"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let sh_pid = child.id();
        let mut line = String::new();
        std::io::BufReader::new(child.stdout.take().unwrap()).read_line(&mut line).unwrap();
        let sleep_pid: u32 = line.trim().parse().unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while RealProcessTree
            .read()
            .is_none_or(|rows| !rows.iter().any(|&(pid, ppid, _)| pid == sleep_pid && ppid == sh_pid))
        {
            assert!(std::time::Instant::now() < deadline, "sleep never showed up under sh");
            std::thread::sleep(Duration::from_millis(20));
        }

        kill_tree(&RealProcessTree, sh_pid, TREE_KILL_GRACE);

        child.wait().unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while crate::lock::is_pid_alive(Some(sleep_pid as f64)) {
            assert!(std::time::Instant::now() < deadline, "detached sleep {sleep_pid} survived the tree kill");
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(!crate::lock::is_pid_alive(Some(sh_pid as f64)));
    }

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

    struct FakeSteerRunner {
        calls: std::sync::atomic::AtomicUsize,
        should_fail: bool,
    }

    impl FakeSteerRunner {
        fn new(should_fail: bool) -> Self {
            Self {
                calls: std::sync::atomic::AtomicUsize::new(0),
                should_fail,
            }
        }

        fn call_count(&self) -> usize {
            self.calls.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl crate::herding::paseo_steer::SteerRunner for FakeSteerRunner {
        fn steer(&self, _cli_dir: &Path, _agent_id: &str, _text: &str) -> Result<(), String> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if self.should_fail {
                Err("fake runner error".to_string())
            } else {
                Ok(())
            }
        }
    }

    fn setup_fake_cli(tmp: &Path) -> std::path::PathBuf {
        let pkg_dir = tmp.join("fake_cli_pkg");
        let bin_dir = pkg_dir.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(
            pkg_dir.join("package.json"),
            r#"{"name":"@getpaseo/cli","version":"0.10.3"}"#,
        )
        .unwrap();
        let bin_path = bin_dir.join("paseo");
        std::fs::write(&bin_path, "#!/bin/sh\nexit 0\n").unwrap();
        bin_path
    }

    #[test]
    fn steer_job_with_fake_runner_for_claude() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        let fake_bin = setup_fake_cli(tmp.path());

        let cfg = serde_json::json!({
            "herding": {
                "paseo": {
                    "command": fake_bin.to_str().unwrap()
                },
                "agents": {
                    "w-claude": {
                        "paseo": {
                            "provider": "claude"
                        }
                    }
                }
            }
        });
        std::fs::create_dir_all(&bee_dir).unwrap();
        std::fs::write(bee_dir.join("config.json"), cfg.to_string()).unwrap();

        let job_dir = bee_dir.join("mailbox").join("job-claude-test");
        std::fs::create_dir_all(&job_dir).unwrap();
        let job_spec = serde_json::json!({
            "job_id": "job-claude-test",
            "transport": "paseo",
            "agent": "w-claude",
            "paseo_agent_id": "agent-uuid-claude",
            "round": 1
        });
        std::fs::write(job_dir.join("job.json"), job_spec.to_string()).unwrap();

        let runner = FakeSteerRunner::new(false);
        let res = steer_job_with(tmp.path(), "job-claude-test", "steer prompt", &runner);
        assert!(res.is_ok());
        assert_eq!(runner.call_count(), 1);
        assert!(!job_dir.join("steer-1.json").exists());
    }

    #[test]
    fn steer_job_with_fake_runner_for_pi() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");

        let cfg = serde_json::json!({
            "herding": {
                "agents": {
                    "w-pi": {
                        "paseo": {
                            "provider": "pi"
                        }
                    }
                }
            }
        });
        std::fs::create_dir_all(&bee_dir).unwrap();
        std::fs::write(bee_dir.join("config.json"), cfg.to_string()).unwrap();

        let job_dir = bee_dir.join("mailbox").join("job-pi-test");
        std::fs::create_dir_all(&job_dir).unwrap();
        let job_spec = serde_json::json!({
            "job_id": "job-pi-test",
            "transport": "paseo",
            "agent": "w-pi",
            "paseo_agent_id": "agent-uuid-pi",
            "round": 1
        });
        std::fs::write(job_dir.join("job.json"), job_spec.to_string()).unwrap();

        let runner = FakeSteerRunner::new(false);
        let res = steer_job_with(tmp.path(), "job-pi-test", "steer prompt", &runner);
        assert!(res.is_ok());
        assert_eq!(runner.call_count(), 0);
        assert!(job_dir.join("steer-1.json").is_file());
    }

    #[test]
    fn steer_job_with_unknown_provider() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");

        let cfg = serde_json::json!({
            "herding": {
                "agents": {
                    "w-other": {
                        "paseo": {
                            "provider": "unknown_provider"
                        }
                    }
                }
            }
        });
        std::fs::create_dir_all(&bee_dir).unwrap();
        std::fs::write(bee_dir.join("config.json"), cfg.to_string()).unwrap();

        let job_dir = bee_dir.join("mailbox").join("job-other-test");
        std::fs::create_dir_all(&job_dir).unwrap();
        let job_spec = serde_json::json!({
            "job_id": "job-other-test",
            "transport": "paseo",
            "agent": "w-other",
            "paseo_agent_id": "agent-uuid-other",
            "round": 1
        });
        std::fs::write(job_dir.join("job.json"), job_spec.to_string()).unwrap();

        let runner = FakeSteerRunner::new(false);
        let res = steer_job_with(tmp.path(), "job-other-test", "steer prompt", &runner);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.code, "paseo_steer_unsupported");
        assert!(err.message.contains("FIX:"));
        assert_eq!(runner.call_count(), 0);
        assert!(!job_dir.join("steer-1.json").exists());
    }

    #[test]
    fn steer_job_with_runner_error() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        let fake_bin = setup_fake_cli(tmp.path());

        let cfg = serde_json::json!({
            "herding": {
                "paseo": {
                    "command": fake_bin.to_str().unwrap()
                },
                "agents": {
                    "w-claude": {
                        "paseo": {
                            "provider": "claude"
                        }
                    }
                }
            }
        });
        std::fs::create_dir_all(&bee_dir).unwrap();
        std::fs::write(bee_dir.join("config.json"), cfg.to_string()).unwrap();

        let job_dir = bee_dir.join("mailbox").join("job-err-test");
        std::fs::create_dir_all(&job_dir).unwrap();
        let job_spec = serde_json::json!({
            "job_id": "job-err-test",
            "transport": "paseo",
            "agent": "w-claude",
            "paseo_agent_id": "agent-uuid-claude",
            "round": 1
        });
        std::fs::write(job_dir.join("job.json"), job_spec.to_string()).unwrap();

        let runner = FakeSteerRunner::new(true);
        let res = steer_job_with(tmp.path(), "job-err-test", "steer prompt", &runner);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.code, "paseo_steer_failed");
        assert!(err.message.contains("FIX:"));
        assert_eq!(runner.call_count(), 1);
        assert!(!job_dir.join("steer-1.json").exists());
    }

    struct FakePaseoCli {
        calls: std::sync::Mutex<Vec<Vec<String>>>,
        result: Result<String, String>,
    }

    impl FakePaseoCli {
        fn new() -> Self {
            Self {
                calls: std::sync::Mutex::new(Vec::new()),
                result: Ok("ok".to_string()),
            }
        }

        fn get_calls(&self) -> Vec<Vec<String>> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl crate::herding::paseo::PaseoCli for FakePaseoCli {
        fn call(&self, args: &[String]) -> Result<String, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            self.result.clone()
        }
    }

    fn seed_paseo_job(bee_dir: &Path, job_id: &str, agent_id: &str) {
        let job_file = mailbox::job_path(bee_dir, job_id);
        let mut obj = serde_json::Map::new();
        obj.insert("job_id".to_string(), Value::String(job_id.to_string()));
        obj.insert("transport".to_string(), Value::String("paseo".to_string()));
        obj.insert("paseo_agent_id".to_string(), Value::String(agent_id.to_string()));
        crate::fsutil::write_json_atomic(&job_file, &Value::Object(obj)).unwrap();
    }

    #[test]
    fn interrupt_paseo_calls_stop_and_keeps_agent() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        seed_paseo_job(&bee_dir, "job-paseo-int", "agent-int-123");

        let cli = FakePaseoCli::new();
        let res = interrupt_with_backends(tmp.path(), "job-paseo-int", false, None, Some(&cli));
        assert!(res.is_ok(), "{res:?}");
        assert_eq!(cli.get_calls(), vec![vec!["stop".to_string(), "agent-int-123".to_string()]]);
        let mark = mailbox::read_mark(&bee_dir, "job-paseo-int");
        assert_eq!(mark, Some((Mark::Interrupted, Some("user".to_string()))));
    }

    #[test]
    fn cancel_paseo_calls_stop_then_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        seed_paseo_job(&bee_dir, "job-paseo-cancel", "agent-cancel-456");

        let cli = FakePaseoCli::new();
        let res = cancel_with_backends_and_timeout(
            tmp.path(),
            "job-paseo-cancel",
            false,
            None,
            Some(&cli),
            &NoopTree,
            DEFAULT_POLL_TIMEOUT,
            DEFAULT_POLL_INTERVAL,
        );
        assert!(res.is_ok(), "{res:?}");
        assert_eq!(
            cli.get_calls(),
            vec![
                vec!["stop".to_string(), "agent-cancel-456".to_string()],
                vec!["archive".to_string(), "--force".to_string(), "agent-cancel-456".to_string()],
            ]
        );
        let mark = mailbox::read_mark(&bee_dir, "job-paseo-cancel");
        assert_eq!(mark, Some((Mark::Cancelled, Some("user".to_string()))));
    }

    #[test]
    fn cancel_paseo_never_archives_callers_own_agent() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        seed_paseo_job(&bee_dir, "job-paseo-own", "agent-own-999");

        unsafe { std::env::set_var("PASEO_AGENT_ID", "agent-own-999") };
        let cli = FakePaseoCli::new();
        let res = cancel_with_backends_and_timeout(
            tmp.path(),
            "job-paseo-own",
            false,
            None,
            Some(&cli),
            &NoopTree,
            DEFAULT_POLL_TIMEOUT,
            DEFAULT_POLL_INTERVAL,
        );
        unsafe { std::env::remove_var("PASEO_AGENT_ID") };

        assert!(res.is_ok(), "{res:?}");
        assert!(cli.get_calls().is_empty());
        let mark = mailbox::read_mark(&bee_dir, "job-paseo-own");
        assert_eq!(mark, Some((Mark::Cancelled, Some("user".to_string()))));
    }

    #[test]
    fn permit_sends_allow_or_deny_with_job_agent_id() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        seed_paseo_job(&bee_dir, "job-paseo-permit", "agent-permit-789");

        let cli = FakePaseoCli::new();
        let res1 = permit_job(tmp.path(), "job-paseo-permit", true, Some("req-1"), false, &cli);
        assert!(res1.is_ok(), "{res1:?}");
        let res2 = permit_job(tmp.path(), "job-paseo-permit", false, None, true, &cli);
        assert!(res2.is_ok(), "{res2:?}");

        assert_eq!(
            cli.get_calls(),
            vec![
                vec!["permit".to_string(), "allow".to_string(), "agent-permit-789".to_string(), "req-1".to_string()],
                vec!["permit".to_string(), "deny".to_string(), "agent-permit-789".to_string(), "--all".to_string()],
            ]
        );
    }

    #[test]
    fn permit_refuses_a_herdr_job() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        seed_job(&bee_dir, "job-herdr-permit", "p1");

        let cli = FakePaseoCli::new();
        let err = permit_job(tmp.path(), "job-herdr-permit", true, None, false, &cli).unwrap_err();
        assert_eq!(err.code, "not_a_paseo_job");
        assert!(err.message.contains("FIX:"), "{err}");
        assert!(cli.get_calls().is_empty());
    }

    #[test]
    fn permit_refuses_missing_job_and_missing_agent_id() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");

        let cli = FakePaseoCli::new();
        let err1 = permit_job(tmp.path(), "nonexistent-job", true, None, false, &cli).unwrap_err();
        assert_eq!(err1.code, "job_not_found");
        assert!(err1.message.contains("FIX:"), "{err1}");

        let job_file = mailbox::job_path(&bee_dir, "job-no-agent");
        let mut obj = serde_json::Map::new();
        obj.insert("job_id".to_string(), Value::String("job-no-agent".to_string()));
        obj.insert("transport".to_string(), Value::String("paseo".to_string()));
        crate::fsutil::write_json_atomic(&job_file, &Value::Object(obj)).unwrap();

        let err2 = permit_job(tmp.path(), "job-no-agent", true, None, false, &cli).unwrap_err();
        assert_eq!(err2.code, "paseo_agent_missing");
        assert!(err2.message.contains("FIX:"), "{err2}");
    }

    #[test]
    fn interrupt_and_cancel_paseo_never_build_pane_transport() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        seed_paseo_job(&bee_dir, "job-paseo-no-pane", "agent-no-pane");

        let cli = FakePaseoCli::new();
        let res_int = interrupt_with_backends(tmp.path(), "job-paseo-no-pane", false, None, Some(&cli));
        assert!(res_int.is_ok());

        seed_paseo_job(&bee_dir, "job-paseo-no-pane-cancel", "agent-no-pane-cancel");
        let res_cancel = cancel_with_backends_and_timeout(
            tmp.path(),
            "job-paseo-no-pane-cancel",
            false,
            None,
            Some(&cli),
            &NoopTree,
            DEFAULT_POLL_TIMEOUT,
            DEFAULT_POLL_INTERVAL,
        );
        assert!(res_cancel.is_ok());
    }
}
