use crate::verbs::handoff_door::{parse_dispatch_run_id, stderr_tail, HandoffDoor};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Expectation {
    BranchMerged { branch: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FollowUp {
    pub run_id: String,
    pub request_id: String,
    pub sent_at: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LedgerStatus {
    Open,
    Met,
    NeedsOwner,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffLedgerRow {
    pub request_id: String,
    pub run_id: String,
    pub project_id: String,
    pub session_id: String,
    pub feature: String,
    pub sent_at: String,
    pub task_file: String,
    pub task_text: String,
    pub expects: Vec<Expectation>,
    pub follow_ups: Vec<FollowUp>,
    pub status: LedgerStatus,
}

pub fn ledger_dir(main_root: &Path) -> PathBuf {
    main_root.join(".bee").join("handoffs")
}

pub fn write_row(main_root: &Path, row: &HandoffLedgerRow) -> std::io::Result<PathBuf> {
    let path = ledger_dir(main_root).join(format!("{}.json", row.request_id));
    let value = serde_json::to_value(row)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    crate::fsutil::write_json_atomic(&path, &value)?;
    Ok(path)
}

#[allow(dead_code)]
pub fn read_open_rows(main_root: &Path) -> Vec<HandoffLedgerRow> {
    let dir = ledger_dir(main_root);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(row) = serde_json::from_str::<HandoffLedgerRow>(&raw) else {
            continue;
        };
        if row.status == LedgerStatus::Open {
            rows.push(row);
        }
    }
    rows.sort_by(|a, b| a.sent_at.cmp(&b.sent_at).then_with(|| a.request_id.cmp(&b.request_id)));
    rows
}

pub fn branch_merged(main_root: &Path, branch: &str) -> Option<bool> {
    let b = branch.trim();
    if b.is_empty() {
        return None;
    }
    let exists = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", b])
        .current_dir(main_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()?;
    if !exists.success() {
        return None;
    }
    let out = Command::new("git")
        .args(["merge-base", "--is-ancestor", b, "main"])
        .current_dir(main_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()?;
    match out.code() {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckVerdict {
    Pending,
    Met,
    FollowUp { unmet: Vec<String> },
    NeedsOwner { unmet: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "kebab-case")]
pub enum CheckOutcome {
    Pending,
    Met,
    FollowUpSent { new_run_id: String, unmet: Vec<String> },
    NeedsOwner { unmet: Vec<String> },
    NoDoor { unmet: Vec<String> },
    ReportOnly { unmet: Vec<String> },
    DispatchFailed { error: String, unmet: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckReport {
    pub request_id: String,
    pub run_id: String,
    pub status: String,
    #[serde(flatten)]
    pub outcome: CheckOutcome,
}

pub fn decide(
    row: &HandoffLedgerRow,
    run_status: &str,
    merged: &dyn Fn(&str) -> Option<bool>,
) -> CheckVerdict {
    let norm = run_status.trim().to_ascii_lowercase();
    if norm != "done" && norm != "lost" {
        return CheckVerdict::Pending;
    }
    let mut unmet = Vec::new();
    for exp in &row.expects {
        match exp {
            Expectation::BranchMerged { branch } => {
                if merged(branch) != Some(true) {
                    unmet.push(branch.clone());
                }
            }
        }
    }
    if unmet.is_empty() {
        CheckVerdict::Met
    } else if row.follow_ups.len() >= 3 {
        CheckVerdict::NeedsOwner { unmet }
    } else {
        CheckVerdict::FollowUp { unmet }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RunInfo {
    pub status: String,
    pub task_text: String,
}

pub fn read_run_info(binary: &Path, run_id: &str) -> Option<RunInfo> {
    let out = Command::new(binary)
        .args(["runs", "show", run_id])
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let doc: Value = serde_json::from_slice(&out.stdout).ok()?;
    let row = doc
        .get("runs")
        .and_then(Value::as_array)
        .and_then(|a| a.first())?;
    let status = row
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let task_text = row
        .get("task")
        .and_then(Value::as_str)
        .or_else(|| doc.get("task").and_then(Value::as_str))
        .unwrap_or("")
        .trim()
        .to_string();
    Some(RunInfo { status, task_text })
}

pub fn format_report_line(r: &CheckReport) -> String {
    let detail = match &r.outcome {
        CheckOutcome::Pending => "pending".to_string(),
        CheckOutcome::Met => "met".to_string(),
        CheckOutcome::FollowUpSent { new_run_id, .. } => {
            format!("follow-up sent run {new_run_id}")
        }
        CheckOutcome::NeedsOwner { unmet } => format!("needs-owner: {}", unmet.join(", ")),
        CheckOutcome::NoDoor { unmet } => format!("no door: {}", unmet.join(", ")),
        CheckOutcome::ReportOnly { unmet } => {
            format!("unmet (no-dispatch): {}", unmet.join(", "))
        }
        CheckOutcome::DispatchFailed { error, .. } => format!("dispatch failed: {error}"),
    };
    format!("[handoff_check] {} run {}: {detail}", r.request_id, r.run_id, )
}

pub fn check_all(main_root: &Path, door: &HandoffDoor, dispatch: bool) -> Vec<CheckReport> {
    let rows = read_open_rows(main_root);
    if rows.is_empty() {
        return Vec::new();
    }
    let mut reports = Vec::with_capacity(rows.len());
    for mut row in rows {
        let checked_run = row.run_id.clone();
        let HandoffDoor::Present { binary, project_id } = door else {
            let mut unmet = Vec::new();
            for exp in &row.expects {
                match exp {
                    Expectation::BranchMerged { branch } => {
                        if branch_merged(main_root, branch) != Some(true) {
                            unmet.push(branch.clone());
                        }
                    }
                }
            }
            let outcome = if unmet.is_empty() {
                CheckOutcome::Pending
            } else {
                CheckOutcome::NoDoor { unmet }
            };
            reports.push(CheckReport {
                request_id: row.request_id,
                run_id: checked_run,
                status: "unknown".to_string(),
                outcome,
            });
            continue;
        };

        let run_info = read_run_info(binary, &checked_run).unwrap_or_default();
        let run_status = if run_info.status.is_empty() {
            "unknown".to_string()
        } else {
            run_info.status.clone()
        };
        let verdict = decide(&row, &run_status, &|b| branch_merged(main_root, b));
        let outcome = match verdict {
            CheckVerdict::Pending => CheckOutcome::Pending,
            CheckVerdict::Met => {
                row.status = LedgerStatus::Met;
                let _ = write_row(main_root, &row);
                CheckOutcome::Met
            }
            CheckVerdict::NeedsOwner { unmet } => {
                row.status = LedgerStatus::NeedsOwner;
                let _ = write_row(main_root, &row);
                CheckOutcome::NeedsOwner { unmet }
            }
            CheckVerdict::FollowUp { unmet } => {
                if !dispatch {
                    CheckOutcome::ReportOnly { unmet }
                } else {
                    dispatch_followup(main_root, binary, project_id, &mut row, &run_status, unmet)
                }
            }
        };
        reports.push(CheckReport {
            request_id: row.request_id,
            run_id: checked_run,
            status: run_status,
            outcome,
        });
    }
    reports
}

fn dispatch_followup(
    main_root: &Path,
    binary: &Path,
    door_project_id: &str,
    row: &mut HandoffLedgerRow,
    run_status: &str,
    unmet: Vec<String>,
) -> CheckOutcome {
    let old_run = row.run_id.clone();
    let n = row.follow_ups.len() + 1;
    let tmp_dir = main_root.join(".bee").join("tmp");
    if let Err(e) = std::fs::create_dir_all(&tmp_dir) {
        return CheckOutcome::DispatchFailed {
            error: format!("could not create {}: {e}", tmp_dir.display()),
            unmet,
        };
    }
    let deliverables = unmet
        .iter()
        .map(|b| format!("branch {b} is not merged into main"))
        .collect::<Vec<_>>()
        .join("; ");
    let task_file = tmp_dir.join(format!("handoff-followup-{old_run}-{n}.md"));
    let task_body = format!(
        "Follow-up of handoff run {old_run}: it ended {run_status} but these deliverables are unmet: {deliverables}. Finish them through the bee chain (worktree merge from main), then report. Original task: {}",
        row.task_text
    );
    if let Err(e) = std::fs::write(&task_file, &task_body) {
        return CheckOutcome::DispatchFailed {
            error: format!("could not write follow-up task file: {e}"),
            unmet,
        };
    }
    let project = if row.project_id.trim().is_empty() {
        door_project_id
    } else {
        row.project_id.as_str()
    };
    let req_id = format!("bee-handoff-followup-{old_run}-{n}");
    let task_file_str = task_file.to_string_lossy().into_owned();
    let out = match Command::new(binary)
        .args([
            "dispatch",
            "--project",
            project,
            "--leader",
            "--follow-up-of",
            &old_run,
            "--handoff",
            "--request-id",
            &req_id,
            "--task-file",
            &task_file_str,
        ])
        .stdin(Stdio::null())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return CheckOutcome::DispatchFailed {
                error: format!("failed to spawn waggledance dispatch: {e}"),
                unmet,
            };
        }
    };
    if !out.status.success() {
        return CheckOutcome::DispatchFailed {
            error: stderr_tail(&out.stderr, &out.stdout),
            unmet,
        };
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let Some(new_run_id) = parse_dispatch_run_id(&stdout) else {
        return CheckOutcome::DispatchFailed {
            error: "waggledance dispatch returned no run_id".to_string(),
            unmet,
        };
    };
    row.follow_ups.push(FollowUp {
        run_id: new_run_id.clone(),
        request_id: req_id,
        sent_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        reason: deliverables,
    });
    row.run_id = new_run_id.clone();
    if let Err(e) = write_row(main_root, row) {
        return CheckOutcome::DispatchFailed {
            error: format!("dispatched run {new_run_id} but failed to update ledger row: {e}"),
            unmet,
        };
    }
    CheckOutcome::FollowUpSent { new_run_id, unmet }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git(cwd: &Path, args: &[&str]) {
        let out = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git on PATH");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    #[test]
    fn ledger_row_round_trips_with_expected_json_shape() {
        let tmp = tempfile::tempdir().unwrap();
        let row = HandoffLedgerRow {
            request_id: "bee-handoff-sess-1-mark-abc123".to_string(),
            run_id: "run-42".to_string(),
            project_id: "proj-1".to_string(),
            session_id: "sess-1".to_string(),
            feature: "demo".to_string(),
            sent_at: "2026-11-24T10:00:00.000Z".to_string(),
            task_file: "/tmp/task.md".to_string(),
            task_text: "Finish demo".to_string(),
            expects: vec![Expectation::BranchMerged {
                branch: "wt/demo".to_string(),
            }],
            follow_ups: vec![FollowUp {
                run_id: "run-43".to_string(),
                request_id: "bee-handoff-followup-run-42-1".to_string(),
                sent_at: "2026-11-24T10:05:00.000Z".to_string(),
                reason: "branch wt/demo not merged".to_string(),
            }],
            status: LedgerStatus::Open,
        };
        let path = write_row(tmp.path(), &row).unwrap();
        assert_eq!(path, ledger_dir(tmp.path()).join("bee-handoff-sess-1-mark-abc123.json"));
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(val["expects"][0]["kind"], "branch-merged");
        assert_eq!(val["expects"][0]["branch"], "wt/demo");
        assert_eq!(val["status"], "open");

        let mut closed = row.clone();
        closed.request_id = "bee-handoff-sess-1-mark-closed".to_string();
        closed.status = LedgerStatus::Met;
        write_row(tmp.path(), &closed).unwrap();

        let open = read_open_rows(tmp.path());
        assert_eq!(open, vec![row]);
    }

    #[test]
    fn branch_merged_reports_none_false_and_true() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        git(repo, &["init", "-b", "main"]);
        git(repo, &["config", "user.email", "test@example.com"]);
        git(repo, &["config", "user.name", "Test"]);
        std::fs::write(repo.join("a.txt"), "1\n").unwrap();
        git(repo, &["add", "a.txt"]);
        git(repo, &["commit", "-m", "init"]);

        assert_eq!(branch_merged(repo, "wt/missing"), None);

        git(repo, &["checkout", "-b", "wt/feat"]);
        std::fs::write(repo.join("b.txt"), "2\n").unwrap();
        git(repo, &["add", "b.txt"]);
        git(repo, &["commit", "-m", "feat"]);
        git(repo, &["checkout", "main"]);

        assert_eq!(branch_merged(repo, "wt/feat"), Some(false));

        git(repo, &["merge", "--no-ff", "wt/feat", "-m", "merge feat"]);
        assert_eq!(branch_merged(repo, "wt/feat"), Some(true));
    }

    fn sample_row(follow_up_count: usize) -> HandoffLedgerRow {
        let follow_ups = (1..=follow_up_count)
            .map(|i| FollowUp {
                run_id: format!("run-{i}"),
                request_id: format!("bee-handoff-followup-run-0-{i}"),
                sent_at: "2026-11-24T10:00:00.000Z".to_string(),
                reason: "branch wt/demo is not merged into main".to_string(),
            })
            .collect();
        HandoffLedgerRow {
            request_id: "bee-handoff-s-mark-1".to_string(),
            run_id: "run-0".to_string(),
            project_id: "proj-1".to_string(),
            session_id: "s".to_string(),
            feature: "demo".to_string(),
            sent_at: "2026-11-24T10:00:00.000Z".to_string(),
            task_file: String::new(),
            task_text: "Finish demo".to_string(),
            expects: vec![Expectation::BranchMerged {
                branch: "wt/demo".to_string(),
            }],
            follow_ups,
            status: LedgerStatus::Open,
        }
    }

    #[test]
    fn decide_covers_pending_met_followup_and_needs_owner() {
        let row0 = sample_row(0);
        assert_eq!(decide(&row0, "working", &|_| Some(false)), CheckVerdict::Pending);
        assert_eq!(decide(&row0, "queued", &|_| None), CheckVerdict::Pending);
        assert_eq!(decide(&row0, "done", &|_| Some(true)), CheckVerdict::Met);
        assert_eq!(
            decide(&row0, "done", &|_| Some(false)),
            CheckVerdict::FollowUp {
                unmet: vec!["wt/demo".to_string()]
            }
        );
        assert_eq!(
            decide(&row0, "lost", &|_| None),
            CheckVerdict::FollowUp {
                unmet: vec!["wt/demo".to_string()]
            }
        );
        let row3 = sample_row(3);
        assert_eq!(
            decide(&row3, "done", &|_| Some(false)),
            CheckVerdict::NeedsOwner {
                unmet: vec!["wt/demo".to_string()]
            }
        );
    }

    #[test]
    fn check_all_with_no_door_lists_unmet_and_dispatches_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        git(repo, &["init", "-b", "main"]);
        git(repo, &["config", "user.email", "test@example.com"]);
        git(repo, &["config", "user.name", "Test"]);
        std::fs::write(repo.join("a.txt"), "1\n").unwrap();
        git(repo, &["add", "a.txt"]);
        git(repo, &["commit", "-m", "init"]);

        assert!(check_all(repo, &HandoffDoor::Absent, true).is_empty());

        let row = sample_row(0);
        write_row(repo, &row).unwrap();
        let reports = check_all(repo, &HandoffDoor::Absent, true);
        assert_eq!(reports.len(), 1);
        assert_eq!(
            reports[0].outcome,
            CheckOutcome::NoDoor {
                unmet: vec!["wt/demo".to_string()]
            }
        );
        assert_eq!(
            format_report_line(&reports[0]),
            "[handoff_check] bee-handoff-s-mark-1 run run-0: no door: wt/demo"
        );
        let open = read_open_rows(repo);
        assert_eq!(open.len(), 1);
        assert!(open[0].follow_ups.is_empty());
    }
}
