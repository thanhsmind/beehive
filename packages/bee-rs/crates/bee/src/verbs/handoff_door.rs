use crate::hooks::write_guard::{waggledance_binary_for_root, waggledance_list_projects};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoffDoor {
    Absent,
    Present { binary: PathBuf, project_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffRequest {
    pub project_id: String,
    pub follow_up_of: Option<String>,
    pub request_id: String,
    pub task_file: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoffOutcome {
    Sent { run_id: String, request_id: String },
    Failed { reason: String },
}

pub fn build_request(
    project_id: &str,
    follow_up_of: Option<String>,
    session_id: &str,
    feature: &str,
    task_file: PathBuf,
) -> HandoffRequest {
    HandoffRequest {
        project_id: project_id.to_string(),
        follow_up_of: follow_up_of.filter(|s| !s.trim().is_empty()),
        request_id: format!("bee-handoff-{session_id}-{feature}"),
        task_file,
    }
}

pub fn dispatch_argv(req: &HandoffRequest) -> Vec<String> {
    let mut argv = vec![
        "dispatch".to_string(),
        "--project".to_string(),
        req.project_id.clone(),
        "--leader".to_string(),
    ];
    if let Some(ref run_id) = req.follow_up_of {
        argv.push("--follow-up-of".to_string());
        argv.push(run_id.clone());
        argv.push("--handoff".to_string());
    }
    argv.push("--request-id".to_string());
    argv.push(req.request_id.clone());
    argv.push("--task-file".to_string());
    argv.push(req.task_file.to_string_lossy().into_owned());
    argv
}

pub fn find_follow_up(runs_json: &str, agent_id: &str) -> Option<String> {
    let agent = agent_id.trim();
    if agent.is_empty() {
        return None;
    }
    let target_pane = format!("paseo:{agent}");
    let doc: Value = serde_json::from_str(runs_json).ok()?;
    let runs = doc
        .get("runs")
        .and_then(Value::as_array)
        .or_else(|| doc.as_array())?;
    for row in runs {
        let pane = row.get("pane_id").and_then(Value::as_str);
        if pane == Some(target_pane.as_str())
            && let Some(id) = row.get("id").and_then(Value::as_str)
            && !id.trim().is_empty()
        {
            return Some(id.to_string());
        }
    }
    None
}

pub fn compose_task_text(
    merged: &str,
    feature: &str,
    class: Option<&str>,
    lane: Option<&str>,
    files: Option<&str>,
    flags: Option<&str>,
    owner_text: &str,
) -> String {
    let mut cmd = format!("bee feature start --feature {feature}");
    if let Some(c) = class {
        cmd.push_str(&format!(" --class {c}"));
    }
    if let Some(l) = lane {
        cmd.push_str(&format!(" --lane {l}"));
    }
    if let Some(n) = files {
        cmd.push_str(&format!(" --files {n}"));
    }
    if let Some(f) = flags
        && !f.trim().is_empty()
    {
        cmd.push_str(&format!(" --flags {f}"));
    }
    format!(
        "Handoff from previous leader session (merged feature: {merged}).\n\
         Requested feature: {feature}\n\
         Route: class={} lane={} files={} flags={}\n\
         Run: {cmd}\n\n\
         {}\n",
        class.unwrap_or("-"),
        lane.unwrap_or("-"),
        files.unwrap_or("-"),
        flags.filter(|f| !f.trim().is_empty()).unwrap_or("-"),
        owner_text.trim()
    )
}

pub fn probe_door(main_root: &Path) -> HandoffDoor {
    let Some(binary) = waggledance_binary_for_root(main_root) else {
        return HandoffDoor::Absent;
    };
    let Some(projects) = waggledance_list_projects(&binary) else {
        return HandoffDoor::Absent;
    };
    let canon_root = std::fs::canonicalize(main_root).unwrap_or_else(|_| main_root.to_path_buf());
    for project in projects {
        let canon_proj =
            std::fs::canonicalize(&project.root_path).unwrap_or_else(|_| project.root_path.clone());
        if canon_proj == canon_root {
            return HandoffDoor::Present {
                binary,
                project_id: project.id,
            };
        }
    }
    HandoffDoor::Absent
}

pub(crate) fn parse_dispatch_run_id(stdout: &str) -> Option<String> {
    let doc: Value = serde_json::from_str(stdout.trim()).ok()?;
    doc.get("run_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn tail_lines(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.is_empty() {
        return String::new();
    }
    let start = lines.len().saturating_sub(5);
    lines[start..].join(" | ")
}

pub(crate) fn stderr_tail(stderr: &[u8], stdout: &[u8]) -> String {
    let err = tail_lines(stderr);
    if !err.is_empty() {
        return err;
    }
    let out = tail_lines(stdout);
    if !out.is_empty() {
        return out;
    }
    "waggledance dispatch failed with no output".to_string()
}

fn sanitize_path_segment(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub fn execute_handoff(
    door: &HandoffDoor,
    main_root: &Path,
    session_id: &str,
    merged: &str,
    feature: &str,
    class: Option<&str>,
    lane: Option<&str>,
    files: Option<&str>,
    flags: Option<&str>,
    owner_text: &str,
) -> HandoffOutcome {
    let HandoffDoor::Present { binary, project_id } = door else {
        return HandoffOutcome::Failed {
            reason: "waggledance door is absent".to_string(),
        };
    };

    let follow_up_of = std::env::var("PASEO_AGENT_ID")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .and_then(|agent_id| {
            let output = Command::new(binary)
                .args(["runs", "--project", project_id, "--full"])
                .stdin(Stdio::null())
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            find_follow_up(&stdout, &agent_id)
        });

    let tmp_dir = main_root.join(".bee").join("tmp");
    if let Err(e) = std::fs::create_dir_all(&tmp_dir) {
        return HandoffOutcome::Failed {
            reason: format!("could not create {}: {e}", tmp_dir.display()),
        };
    }
    let safe_sid = sanitize_path_segment(session_id);
    let safe_feat = sanitize_path_segment(feature);
    let task_file = tmp_dir.join(format!("handoff-{safe_sid}-{safe_feat}.md"));
    let task_text = compose_task_text(merged, feature, class, lane, files, flags, owner_text);
    if let Err(e) = std::fs::write(&task_file, &task_text) {
        return HandoffOutcome::Failed {
            reason: format!("could not write handoff task file: {e}"),
        };
    }

    let req = build_request(project_id, follow_up_of, session_id, feature, task_file);
    let argv = dispatch_argv(&req);
    let output = match Command::new(binary)
        .args(&argv)
        .stdin(Stdio::null())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return HandoffOutcome::Failed {
                reason: format!("failed to spawn waggledance dispatch: {e}"),
            };
        }
    };

    if !output.status.success() {
        return HandoffOutcome::Failed {
            reason: stderr_tail(&output.stderr, &output.stdout),
        };
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(run_id) = parse_dispatch_run_id(&stdout) else {
        return HandoffOutcome::Failed {
            reason: format!(
                "waggledance dispatch returned unparsable output: {}",
                tail_lines(&output.stdout)
            ),
        };
    };

    HandoffOutcome::Sent {
        run_id,
        request_id: req.request_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_formats_deterministic_request_id() {
        let req = build_request(
            "beehive",
            Some("run-123".to_string()),
            "sess-abc",
            "next-feat",
            PathBuf::from("/tmp/task.md"),
        );
        assert_eq!(req.project_id, "beehive");
        assert_eq!(req.follow_up_of.as_deref(), Some("run-123"));
        assert_eq!(req.request_id, "bee-handoff-sess-abc-next-feat");
        assert_eq!(req.task_file, PathBuf::from("/tmp/task.md"));
    }

    #[test]
    fn dispatch_argv_omits_follow_up_and_handoff_when_absent() {
        let req = build_request(
            "beehive",
            None,
            "sess-1",
            "feat-a",
            PathBuf::from("/repo/.bee/tmp/task.md"),
        );
        assert_eq!(
            dispatch_argv(&req),
            vec![
                "dispatch",
                "--project",
                "beehive",
                "--leader",
                "--request-id",
                "bee-handoff-sess-1-feat-a",
                "--task-file",
                "/repo/.bee/tmp/task.md",
            ]
        );
    }

    #[test]
    fn dispatch_argv_includes_follow_up_and_handoff_when_present() {
        let req = build_request(
            "beehive",
            Some("run-999".to_string()),
            "sess-1",
            "feat-a",
            PathBuf::from("/repo/.bee/tmp/task.md"),
        );
        assert_eq!(
            dispatch_argv(&req),
            vec![
                "dispatch",
                "--project",
                "beehive",
                "--leader",
                "--follow-up-of",
                "run-999",
                "--handoff",
                "--request-id",
                "bee-handoff-sess-1-feat-a",
                "--task-file",
                "/repo/.bee/tmp/task.md",
            ]
        );
    }

    #[test]
    fn find_follow_up_matches_paseo_pane_id_and_ignores_others() {
        let json = r#"{
            "runs": [
                {"id": "run-other", "pane_id": "paseo:other-agent"},
                {"id": "run-match", "pane_id": "paseo:agent-42"}
            ]
        }"#;
        assert_eq!(
            find_follow_up(json, "agent-42").as_deref(),
            Some("run-match")
        );
        assert_eq!(find_follow_up(json, "missing"), None);
        assert_eq!(find_follow_up(json, "  "), None);
    }

    #[test]
    fn compose_task_text_includes_header_command_and_owner_text() {
        let text = compose_task_text(
            "old-feat",
            "new-feat",
            Some("feature"),
            Some("standard"),
            Some("4"),
            Some("multi-domain"),
            "Build the new feature and keep it simple.",
        );
        assert!(text.contains("old-feat"));
        assert!(text.contains("new-feat"));
        assert!(text.contains(
            "bee feature start --feature new-feat --class feature --lane standard --files 4 --flags multi-domain"
        ));
        assert!(text.contains("Build the new feature and keep it simple."));
    }

    #[test]
    fn parse_dispatch_run_id_extracts_id_and_rejects_missing() {
        assert_eq!(
            parse_dispatch_run_id(r#"{"run_id":"run-abc","status":"queued"}"#).as_deref(),
            Some("run-abc")
        );
        assert_eq!(parse_dispatch_run_id(r#"{"status":"queued"}"#), None);
        assert_eq!(parse_dispatch_run_id("not json"), None);
    }
}
