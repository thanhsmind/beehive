// herding::mailbox — the file mailbox worker-completion contract
// (herding-executor: mailbox layout is the D1-numbered decision in
// .bee/decisions.jsonl feature=herding-executor; the self-contained-brief
// requirement is that feature's D3; docs/history/herding-executor/CONTEXT.md
// had not been written when this cell ran — the decision log is the
// authoritative source cited here instead of re-deriving either rule).
//
// Worker completion travels through a file mailbox, one directory per job:
//
//   .bee/mailbox/<job-id>/job.json      <- job spec, written once by the
//                                          orchestrator (a later cell's
//                                          concern; not built here)
//   .bee/mailbox/<job-id>/result-N.json <- round-numbered result, written by
//                                          the WORKER, tmp-then-rename; its
//                                          appearance at the final name IS
//                                          the done signal — never a fixed
//                                          single filename, so a re-briefed
//                                          worker on round 2 can never
//                                          collide with round 1's file
//   .bee/mailbox/<job-id>/log.txt       <- worker's own append-only log,
//                                          read by the orchestrator's
//                                          heartbeat liveness check
//
// The worker is bee-ignorant by design: it may be any of 21 agent kinds
// that has never seen bee and cannot safely run bee state verbs from
// inside its own pane. So the brief `render_brief` produces below is the
// WHOLE contract — task text, absolute paths, file constraints, the result
// JSON schema, and the tmp-then-rename write gesture, spelled out in the
// prompt itself, with the round number named explicitly. Every piece of
// bee bookkeeping (cells finish, proof line, reservations, dispatch row)
// is done by the orchestrator AFTER reading the result — never delegated
// into the pane.
//
// This module is PURE: path arithmetic, string rendering, and JSON
// parsing — no `std::fs` call anywhere below. A later cell's control-loop
// integration owns the real directory listing and file reads, and feeds
// their output into `select_latest_round` / `parse_result_text`; that
// split is what lets every case here — including "no result file yet" and
// "malformed result" — run as a unit test with no filesystem.

use std::path::{Path, PathBuf};

use serde_json::Value;

// ═══════════════════════════════════════════════════════════════════════════
// Path layout — .bee/mailbox/<job-id>/…
// ═══════════════════════════════════════════════════════════════════════════

/// `.bee/mailbox/<job-id>/` under the given `.bee` directory. `bee_dir` is
/// the already-resolved `.bee` directory (e.g. `store_root.join(".bee")`),
/// not the repo root — this module does no root resolution of its own.
pub(crate) fn mailbox_dir(bee_dir: &Path, job_id: &str) -> PathBuf {
    bee_dir.join("mailbox").join(job_id)
}

/// `.bee/mailbox/<job-id>/job.json` — the job spec, written once by the
/// orchestrator before the worker is dispatched.
pub(crate) fn job_path(bee_dir: &Path, job_id: &str) -> PathBuf {
    mailbox_dir(bee_dir, job_id).join("job.json")
}

/// `.bee/mailbox/<job-id>/log.txt` — the worker's own append-only log.
pub(crate) fn log_path(bee_dir: &Path, job_id: &str) -> PathBuf {
    mailbox_dir(bee_dir, job_id).join("log.txt")
}

/// `.bee/mailbox/<job-id>/result-N.json` for a given round.
pub(crate) fn brief_path(bee_dir: &Path, job_id: &str, round: u32) -> PathBuf {
    mailbox_dir(bee_dir, job_id).join(format!("brief-{round}.txt"))
}

/// herding-brief-file D1: the ONE-LINE prompt actually delivered to the
/// agent. A multi-line prompt is silently dropped by at least one agent
/// kind (agy, live smokes 4/5: agent idle and ready, brief lost; a
/// single-line prompt landed instantly) — so the brief body lives in
/// `brief-N.txt` and the prompt only points at it.
pub(crate) fn pointer_prompt(brief_abs_path: &Path) -> String {
    format!(
        "Read the file {} and follow its instructions exactly.",
        brief_abs_path.display()
    )
}

pub(crate) fn result_path(bee_dir: &Path, job_id: &str, round: u32) -> PathBuf {
    mailbox_dir(bee_dir, job_id).join(result_filename(round))
}

/// The bare filename for a round's FINAL result name — `result-N.json`.
/// The single source both `result_path` above and `parse_result_filename`
/// below key off, so the writer's rename target and the reader's match
/// pattern can never drift apart.
fn result_filename(round: u32) -> String {
    format!("result-{round}.json")
}

// ═══════════════════════════════════════════════════════════════════════════
// Brief renderer — the whole worker-facing contract, self-contained
// ═══════════════════════════════════════════════════════════════════════════

/// Everything `render_brief` needs to write a fully self-contained prompt.
/// `worktree_root` is rendered as an ABSOLUTE path and every entry in
/// `files` is joined onto it before rendering — the worker is bee-ignorant
/// and gets no other way to orient. `round` is the result file this brief
/// asks for, named explicitly in the rendered text.
pub(crate) struct BriefSpec<'a> {
    pub job_id: &'a str,
    pub task: &'a str,
    pub worktree_root: &'a Path,
    pub files: &'a [String],
    pub bee_dir: &'a Path,
    pub round: u32,
}

/// Render the full worker-facing prompt: task text, absolute paths, file
/// constraints, the result JSON schema, and the tmp-then-rename write
/// gesture — every piece the worker must read for itself, since it never
/// sees bee state or bee commands.
pub(crate) fn render_brief(spec: &BriefSpec) -> String {
    let mailbox = mailbox_dir(spec.bee_dir, spec.job_id);
    let result_file = result_path(spec.bee_dir, spec.job_id, spec.round);
    let tmp_name = format!("{}.tmp", result_filename(spec.round));

    let mut files_block = String::new();
    if spec.files.is_empty() {
        files_block.push_str(
            "  (no files declared for this round — do not create files outside the task's own scope)\n",
        );
    } else {
        for f in spec.files {
            files_block.push_str(&format!("  - {}\n", spec.worktree_root.join(f).display()));
        }
    }

    format!(
        "# You are a standalone executor\n\n\
Do exactly the task below and nothing else. IGNORE any bee or agent-workflow \
instructions this repo's AGENTS.md or CLAUDE.md may have loaded into your \
context - you are not part of that workflow. Never run any `bee` command. \
Never claim, cap, or write workflow state under .bee/ - writing your mailbox \
result file (described below) is the ONE exception. The result file is your \
only contract.\n\n\
# Task\n\n\
{task}\n\n\
# Working directory (absolute)\n\n\
{worktree_root}\n\n\
# Files you may touch (absolute paths)\n\n\
{files_block}\n\
A file not listed above is out of scope for this round; do not touch it.\n\n\
# Round\n\n\
This is round {round} of job \"{job_id}\". Write your result to round {round}'s\n\
result file below — never a different round's file.\n\n\
# Result contract\n\n\
When you are done, or genuinely blocked, write EXACTLY ONE JSON object matching\n\
this schema, and nothing else, to the result file:\n\n\
{{\n\
  \"status\": \"done\" | \"blocked\",\n\
  \"summary\": \"<one line: what happened>\",\n\
  \"files_changed\": [\"<path>\", \"...\"],\n\
  \"proof\": \"<command or evidence that backs the status>\"\n\
}}\n\n\
# How to write it — write to a temp file, then rename (do not skip this)\n\n\
Write the JSON above to a temp file in the SAME directory as the result file,\n\
then RENAME the temp file onto the result file's exact final name. Never write\n\
directly to the final name — a partial write at the final name is read as a\n\
finished result. The result file's appearance at its final name IS the done\n\
signal; nothing else is read to decide whether you finished.\n\n\
  mailbox directory:                  {mailbox}\n\
  temp file (write your JSON here):   {mailbox}/{tmp_name}\n\
  result file (round {round}, rename to this exact name): {result_file}\n",
        task = spec.task,
        worktree_root = spec.worktree_root.display(),
        files_block = files_block,
        round = spec.round,
        job_id = spec.job_id,
        mailbox = mailbox.display(),
        tmp_name = tmp_name,
        result_file = result_file.display(),
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// Result reader — find highest round, parse, schema-validate
// ═══════════════════════════════════════════════════════════════════════════

/// One validated result, matching the schema `render_brief` asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MailboxResult {
    pub status: MailboxStatus,
    pub summary: String,
    pub files_changed: Vec<String>,
    pub proof: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MailboxStatus {
    Done,
    Blocked,
}

/// Every way reading a mailbox result can fail — missing or malformed is
/// always a typed error, never a silent green.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MailboxError {
    /// No entry matching `result-N.json` exists in the given listing.
    NoResultFile,
    /// The file at the given round is not valid JSON.
    NotJson { round: u32, detail: String },
    /// The parsed JSON is not an object.
    NotAnObject { round: u32 },
    /// A required field is absent or the wrong type.
    MissingField { round: u32, field: &'static str },
    /// `status` is present but not `"done"` or `"blocked"`.
    InvalidStatus { round: u32, value: String },
}

impl std::fmt::Display for MailboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MailboxError::NoResultFile => write!(f, "no result-N.json file found in mailbox"),
            MailboxError::NotJson { round, detail } => {
                write!(f, "result-{round}.json is not valid JSON: {detail}")
            }
            MailboxError::NotAnObject { round } => {
                write!(f, "result-{round}.json is not a JSON object")
            }
            MailboxError::MissingField { round, field } => {
                write!(f, "result-{round}.json is missing required field \"{field}\"")
            }
            MailboxError::InvalidStatus { round, value } => {
                write!(
                    f,
                    "result-{round}.json has invalid status \"{value}\" (want \"done\" or \"blocked\")"
                )
            }
        }
    }
}

impl std::error::Error for MailboxError {}

/// The highest round with a `result-N.json` entry among `entries`, or
/// `None` if there is none. Matches the filename EXACTLY —
/// `result-3.json.tmp`, `result-3.json.bak`, `job.json`, `log.txt`, or any
/// name a partial or in-flight atomic write could leave behind is ignored,
/// never mistaken for a finished round.
pub(crate) fn latest_result_round(entries: &[String]) -> Option<u32> {
    entries.iter().filter_map(|name| parse_result_filename(name)).max()
}

/// `select_latest_round`'s pure refusal half: the same lookup as
/// `latest_result_round`, but a typed `MailboxError::NoResultFile` in place
/// of `None` — the "missing = typed error, never silent green" case,
/// exercised with no filesystem: the caller supplies `entries` from a real
/// directory listing.
pub(crate) fn select_latest_round(entries: &[String]) -> Result<u32, MailboxError> {
    latest_result_round(entries).ok_or(MailboxError::NoResultFile)
}

fn parse_result_filename(name: &str) -> Option<u32> {
    let digits = name.strip_prefix("result-")?.strip_suffix(".json")?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse::<u32>().ok()
}

/// Parse and schema-validate one round's result text (already read by the
/// caller) into a `MailboxResult`, or a typed `MailboxError` — invalid
/// JSON, a non-object body, a missing field, or a `status` outside the
/// enum are each their own variant, never collapsed into a generic
/// failure.
pub(crate) fn parse_result_text(round: u32, text: &str) -> Result<MailboxResult, MailboxError> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| MailboxError::NotJson { round, detail: e.to_string() })?;
    let obj = value.as_object().ok_or(MailboxError::NotAnObject { round })?;

    let status_str = obj
        .get("status")
        .and_then(Value::as_str)
        .ok_or(MailboxError::MissingField { round, field: "status" })?;
    let status = match status_str {
        "done" => MailboxStatus::Done,
        "blocked" => MailboxStatus::Blocked,
        other => return Err(MailboxError::InvalidStatus { round, value: other.to_string() }),
    };

    let summary = obj
        .get("summary")
        .and_then(Value::as_str)
        .ok_or(MailboxError::MissingField { round, field: "summary" })?
        .to_string();

    let files_changed_arr = obj
        .get("files_changed")
        .and_then(Value::as_array)
        .ok_or(MailboxError::MissingField { round, field: "files_changed" })?;
    let files_changed = files_changed_arr
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();

    let proof = obj
        .get("proof")
        .and_then(Value::as_str)
        .ok_or(MailboxError::MissingField { round, field: "proof" })?
        .to_string();

    Ok(MailboxResult { status, summary, files_changed, proof })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_files() -> Vec<String> {
        vec!["src/lib.rs".to_string(), "src/main.rs".to_string()]
    }

    fn sample_spec<'a>(worktree_root: &'a Path, bee_dir: &'a Path, files: &'a [String], round: u32) -> BriefSpec<'a> {
        BriefSpec {
            job_id: "job-42",
            task: "Fix the off-by-one in the paginator.",
            worktree_root,
            files,
            bee_dir,
            round,
        }
    }

    // ─── path layout ─────────────────────────────────────────────────────

    #[test]
    fn mailbox_paths_are_under_bee_mailbox_job_id() {
        let bee_dir = Path::new("/repo/.bee");
        assert_eq!(mailbox_dir(bee_dir, "job-1"), PathBuf::from("/repo/.bee/mailbox/job-1"));
        assert_eq!(job_path(bee_dir, "job-1"), PathBuf::from("/repo/.bee/mailbox/job-1/job.json"));
        assert_eq!(log_path(bee_dir, "job-1"), PathBuf::from("/repo/.bee/mailbox/job-1/log.txt"));
        assert_eq!(result_path(bee_dir, "job-1", 5), PathBuf::from("/repo/.bee/mailbox/job-1/result-5.json"));
    }

    // ─── brief renderer ─────────────────────────────────────────────────

    #[test]
    fn render_brief_includes_absolute_paths() {
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files = sample_files();
        let spec = sample_spec(worktree_root, bee_dir, &files, 1);
        let text = render_brief(&spec);

        assert!(text.contains("/repo/work"), "missing absolute worktree root:\n{text}");
        assert!(text.contains("/repo/work/src/lib.rs"), "missing absolute file path:\n{text}");
        assert!(text.contains("/repo/work/src/main.rs"), "missing absolute file path:\n{text}");
        assert!(text.contains("/repo/.bee/mailbox/job-42"), "missing absolute mailbox dir:\n{text}");
    }

    #[test]
    fn render_brief_includes_the_result_schema_block() {
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files = sample_files();
        let spec = sample_spec(worktree_root, bee_dir, &files, 1);
        let text = render_brief(&spec);

        assert!(text.contains("\"status\": \"done\" | \"blocked\""), "missing status schema line:\n{text}");
        assert!(text.contains("\"summary\""), "missing summary field:\n{text}");
        assert!(text.contains("\"files_changed\""), "missing files_changed field:\n{text}");
        assert!(text.contains("\"proof\""), "missing proof field:\n{text}");
    }

    #[test]
    fn render_brief_names_the_round_number_explicitly() {
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files = sample_files();
        let spec = sample_spec(worktree_root, bee_dir, &files, 3);
        let text = render_brief(&spec);

        assert!(text.contains("round 3"), "round number not named explicitly:\n{text}");
        assert!(text.contains("result-3.json"), "final result filename for the round missing:\n{text}");
    }

    #[test]
    fn render_brief_includes_the_tmp_then_rename_gesture() {
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files = sample_files();
        let spec = sample_spec(worktree_root, bee_dir, &files, 2);
        let text = render_brief(&spec);

        assert!(text.to_lowercase().contains("rename"), "no rename instruction:\n{text}");
        assert!(text.contains("result-2.json.tmp"), "no named temp file:\n{text}");
        assert!(text.contains("result-2.json"), "no named final result file:\n{text}");
    }

    #[test]
    fn render_brief_opens_with_the_standalone_executor_block_before_task() {
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files = sample_files();
        let spec = sample_spec(worktree_root, bee_dir, &files, 1);
        let text = render_brief(&spec);

        let standalone_pos = text
            .find("# You are a standalone executor")
            .expect("missing standalone-executor block");
        let task_pos = text.find("# Task").expect("missing # Task heading");
        assert!(standalone_pos < task_pos, "standalone-executor block must come before # Task:\n{text}");
        assert!(text.contains("IGNORE any bee or agent-workflow"), "missing ignore-workflow wording:\n{text}");
        assert!(text.contains("Never run any `bee` command"), "missing never-run-bee wording:\n{text}");
    }

    #[test]
    fn render_brief_with_no_files_still_renders_a_scope_line() {
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files: Vec<String> = Vec::new();
        let spec = sample_spec(worktree_root, bee_dir, &files, 1);
        let text = render_brief(&spec);

        assert!(text.contains("no files declared"));
    }

    // ─── latest_result_round / select_latest_round ─────────────────────

    #[test]
    fn latest_result_round_picks_the_highest_n() {
        let entries = vec!["result-1.json".to_string(), "result-3.json".to_string(), "result-2.json".to_string()];
        assert_eq!(latest_result_round(&entries), Some(3));
    }

    #[test]
    fn latest_result_round_ignores_tmp_and_unrelated_names() {
        let entries = vec![
            "result-3.json.tmp".to_string(),
            "result-x.json".to_string(),
            "result-.json".to_string(),
            "job.json".to_string(),
            "log.txt".to_string(),
            "result-1.json".to_string(),
        ];
        assert_eq!(latest_result_round(&entries), Some(1));
    }

    #[test]
    fn latest_result_round_is_none_for_an_empty_mailbox() {
        let entries: Vec<String> = Vec::new();
        assert_eq!(latest_result_round(&entries), None);
    }

    #[test]
    fn select_latest_round_refuses_with_a_typed_error_when_missing() {
        let entries: Vec<String> = vec!["job.json".to_string(), "log.txt".to_string()];
        assert_eq!(select_latest_round(&entries), Err(MailboxError::NoResultFile));
    }

    #[test]
    fn select_latest_round_returns_the_round_when_present() {
        let entries = vec!["result-7.json".to_string()];
        assert_eq!(select_latest_round(&entries), Ok(7));
    }

    // ─── parse_result_text ───────────────────────────────────────────────

    #[test]
    fn parse_result_text_accepts_a_valid_done_result() {
        let text = r#"{"status":"done","summary":"fixed it","files_changed":["a.rs"],"proof":"cargo test — green"}"#;
        let result = parse_result_text(1, text).expect("valid result should parse");
        assert_eq!(result.status, MailboxStatus::Done);
        assert_eq!(result.summary, "fixed it");
        assert_eq!(result.files_changed, vec!["a.rs".to_string()]);
        assert_eq!(result.proof, "cargo test — green");
    }

    #[test]
    fn parse_result_text_accepts_a_valid_blocked_result() {
        let text = r#"{"status":"blocked","summary":"stuck on missing input","files_changed":[],"proof":"n/a"}"#;
        let result = parse_result_text(1, text).expect("valid result should parse");
        assert_eq!(result.status, MailboxStatus::Blocked);
        assert!(result.files_changed.is_empty());
    }

    #[test]
    fn parse_result_text_refuses_text_that_is_not_json() {
        let err = parse_result_text(2, "not json at all").unwrap_err();
        assert!(matches!(err, MailboxError::NotJson { round: 2, .. }), "got: {err:?}");
    }

    #[test]
    fn parse_result_text_refuses_a_missing_field() {
        let text = r#"{"status":"done","summary":"ok"}"#;
        let err = parse_result_text(4, text).unwrap_err();
        assert_eq!(err, MailboxError::MissingField { round: 4, field: "files_changed" });
    }

    #[test]
    fn parse_result_text_refuses_an_invalid_status() {
        let text = r#"{"status":"maybe","summary":"x","files_changed":[],"proof":"x"}"#;
        let err = parse_result_text(1, text).unwrap_err();
        assert_eq!(err, MailboxError::InvalidStatus { round: 1, value: "maybe".to_string() });
    }

    #[test]
    fn parse_result_text_refuses_non_object_json() {
        let err = parse_result_text(1, "[1,2,3]").unwrap_err();
        assert_eq!(err, MailboxError::NotAnObject { round: 1 });
    }

    #[test]
    fn mailbox_error_display_names_the_round() {
        let err = MailboxError::MissingField { round: 9, field: "proof" };
        assert!(err.to_string().contains("result-9.json"));
        assert!(err.to_string().contains("proof"));
    }
}
