use serde::{Deserialize, Serialize};
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
}
