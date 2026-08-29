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
//   .bee/mailbox/<job-id>/ack-N.json    <- round-numbered delivery receipt,
//                                          written by the WORKER as its
//                                          FIRST step, tmp-then-rename;
//                                          unambiguous evidence the brief
//                                          was actually read — herdr
//                                          lifecycle state is never trusted
//                                          for this (herding-prompt-stall D4)
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

/// `.bee/mailbox/<job-id>/ack-N.json` for a given round — herding-prompt-stall
/// D4's delivery receipt: unlike `result_path`, this file is written by the
/// worker as its FIRST step, before it has done any of the task, so its
/// appearance means only "the brief was read," never "the round is done."
pub(crate) fn ack_path(bee_dir: &Path, job_id: &str, round: u32) -> PathBuf {
    mailbox_dir(bee_dir, job_id).join(ack_filename(round))
}

/// The bare filename for a round's delivery-receipt ack — `ack-N.json`,
/// the same round-numbering `result_filename` uses so a re-briefed worker
/// on round 2 can never collide with round 1's ack.
fn ack_filename(round: u32) -> String {
    format!("ack-{round}.json")
}

// ═══════════════════════════════════════════════════════════════════════════
// Activity record — the herded agent's OWN reported state
// ═══════════════════════════════════════════════════════════════════════════

/// `.bee/mailbox/<job-id>/activity.json` — the ONE activity record a herded
/// pane keeps (herding-activity-hook D2). Unlike `ack_path` and
/// `result_path` it is NOT round-numbered: there is exactly one truth per
/// job, rewritten tmp-then-rename by the pane's `activity` hook, and the
/// round it belongs to rides INSIDE the record as the `round` field, which
/// `parse_activity_text` fences on. D3 keeps this file strictly an upgrade
/// to the screen classifier: `ack-N.json` and `result-N.json` stay the only
/// truth for delivered and done, and nothing here ever touches them.
pub(crate) fn activity_path(bee_dir: &Path, job_id: &str) -> PathBuf {
    mailbox_dir(bee_dir, job_id).join("activity.json")
}

/// The agent-reported states an activity record can carry — the SAME
/// vocabulary `hooks/activity.rs` writes, never a second spelling of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActivityState {
    Working,
    WaitingInput,
    Blocked,
    Idle,
    Exited,
}

/// How old an activity record may be and still count (the "fresh" bound
/// CONTEXT.md left to the agent's discretion; recorded here as the one
/// number). 120s: comfortably longer than a slow agent turn between hook
/// events, far shorter than `ACK_WAIT_BUDGET_SECS` (180s) or the round's
/// own idle timeout — so a pane whose hook died mid-round falls back to the
/// screen classifier well before either of those bounds could fire off a
/// state nobody is refreshing.
pub(crate) const ACTIVITY_FRESHNESS_SECS: i64 = 120;

/// Parse one activity record's text (already read by the caller — this
/// module still touches no filesystem) into the agent's own reported state,
/// or `None` when the record must not be trusted for `current_round`.
///
/// `None` is the FAIL-SAFE answer and every refusal returns it, because the
/// caller's fallback is exactly today's screen classifier: a `None` here can
/// only ever mean "behave as if the hook were not installed". The refusals:
///
/// * the text is not JSON, is not an object, or is missing/mistyped `round`,
///   `at`, or `state` — a half-written or foreign file never speaks for the
///   agent;
/// * `state` is outside the known vocabulary — an unknown word is never
///   guessed at;
/// * `round` is BELOW `current_round` (D2: the round is the launch-id
///   fence) — a record left over from an earlier round of the same job
///   cannot describe this one. A record from a LATER round is kept: the
///   pane genuinely moved on, and its state is still this agent's state.
/// * `at` is more than `ACTIVITY_FRESHNESS_SECS` older than `now_ms` — a
///   stale record is a dead hook, not a blocked agent. A record stamped in
///   the FUTURE is kept: clock skew between the hook's own stamp and the
///   reader is jitter, and treating it as stale would silently disable the
///   signal on exactly the machines that need it.
pub(crate) fn parse_activity_text(text: &str, current_round: u32, now_ms: i64) -> Option<ActivityState> {
    let value: Value = serde_json::from_str(text).ok()?;
    let obj = value.as_object()?;

    let round = u32::try_from(obj.get("round").and_then(Value::as_u64)?).ok()?;
    if round < current_round {
        return None;
    }

    let at_ms = chrono::DateTime::parse_from_rfc3339(obj.get("at").and_then(Value::as_str)?)
        .ok()?
        .timestamp_millis();
    if now_ms.saturating_sub(at_ms) > ACTIVITY_FRESHNESS_SECS.saturating_mul(1000) {
        return None;
    }

    match obj.get("state").and_then(Value::as_str)? {
        "working" => Some(ActivityState::Working),
        "waiting_input" => Some(ActivityState::WaitingInput),
        "blocked" => Some(ActivityState::Blocked),
        "idle" => Some(ActivityState::Idle),
        "exited" => Some(ActivityState::Exited),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Brief renderer — the whole worker-facing contract, self-contained
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ExpertiseEntry {
    pub path: String,
    pub purpose: String,
    pub read_to: String,
}

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
    pub expertise: &'a [ExpertiseEntry],
    /// herding-prompt-stall D4: the worker-facing identity recorded in the
    /// round's ack file — who took the job. Never omitted from the ack
    /// schema the brief shows.
    pub nickname: &'a str,
    /// D4: the cell id this round belongs to, when the caller has one —
    /// omitted from the ack schema entirely when `None`, never rendered as
    /// a null.
    pub cell_id: Option<&'a str>,
}

/// Render the full worker-facing prompt: task text, absolute paths, file
/// constraints, the result JSON schema, and the tmp-then-rename write
/// gesture — every piece the worker must read for itself, since it never
/// sees bee state or bee commands.
pub(crate) fn render_brief(spec: &BriefSpec) -> String {
    let mailbox = mailbox_dir(spec.bee_dir, spec.job_id);
    let result_file = result_path(spec.bee_dir, spec.job_id, spec.round);
    let tmp_name = format!("{}.tmp", result_filename(spec.round));
    let ack_file = ack_path(spec.bee_dir, spec.job_id, spec.round);
    let ack_tmp_name = format!("{}.tmp", ack_filename(spec.round));
    let cell_id_line = match spec.cell_id {
        Some(id) => format!("  \"cell_id\": \"{id}\",\n"),
        None => String::new(),
    };

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

    let mut expertise_block = String::new();
    if !spec.expertise.is_empty() {
        expertise_block.push_str("# Expertise — read these before you start\n\n");
        expertise_block.push_str(
            "The dispatcher picked these files for this task. Read each one before\n\
working; they carry the know-how the task needs. Reading them is allowed\n\
and expected — they do not pull you into any workflow.\n\n",
        );
        for entry in spec.expertise {
            let p = Path::new(&entry.path);
            let abs_path = if p.is_absolute() {
                p.to_path_buf()
            } else {
                spec.worktree_root.join(p)
            };
            expertise_block.push_str(&format!(
                "  - {} — {}. Read it to {}.\n",
                abs_path.display(),
                entry.purpose,
                entry.read_to
            ));
        }
        expertise_block.push('\n');
    }

    format!(
        "# Before any other step — write your delivery ack\n\n\
Before you read the Task below, before touching any other file: write an ack \
file for this round. It is the ONLY proof anyone has that this brief was \
ever received, so it comes first — write it atomically, the SAME gesture \
the result file below uses: a temp file in the SAME directory, then RENAME \
the temp file onto the ack file's exact final name. Never write directly to \
the final name.\n\n\
Write this JSON object to the ack file, filling in \"agent\" with your own \
name for yourself and \"received_at\" with the current time as an ISO-8601 \
timestamp:\n\n\
{{\n\
  \"nickname\": \"{nickname}\",\n\
{cell_id_line}\
  \"job_id\": \"{job_id}\",\n\
  \"round\": {round},\n\
  \"agent\": \"<your own name for yourself>\",\n\
  \"received_at\": \"<ISO-8601 timestamp, e.g. 2026-01-01T00:00:00Z>\"\n\
}}\n\n\
  temp file (write your ack JSON here):   {mailbox}/{ack_tmp_name}\n\
  ack file (round {round}, rename to this exact final name): {ack_file}\n\n\
# You are a standalone executor\n\n\
Do exactly the task below and nothing else. Ignore any bee or agent-workflow \
instructions (gates, cells, claims, state) this repo's AGENTS.md or CLAUDE.md \
may have loaded into your context — you are not part of that workflow, and \
files listed under the Expertise section are yours to read. Never run any `bee` \
command. Never claim, cap, or write workflow state under .bee/ - writing your \
mailbox result file (described below) is the ONE exception. The result file is \
your only contract.\n\n\
# Task\n\n\
{task}\n\n\
{expertise_block}\
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
  \"proof\": \"<command or evidence that backs the status>\",\n\
  \"options\": [\"<one self-contained sentence per way forward>\", \"...\"],\n\
  \"leaning\": \"<the one option you would pick, repeated word for word>\",\n\
  \"dissent\": {{ \"claim\": \"<what is wrong with the task as it was handed to you>\", \"alternative\": \"<what you would do instead>\", \"severity\": \"blocker\" | \"consider\" }}\n\
}}\n\n\
When \"blocked\" leaves a choice to make, fill \"options\" with one \
self-contained sentence per way forward and \"leaning\" with the one you would \
pick, repeated word for word; leave both out when there is no choice.\n\
Fill \"dissent\" only when you disagree with the TASK ITSELF — \"claim\" says what \
is wrong with it, \"alternative\" says what you would do instead, and \
\"severity\" is \"blocker\" when the work should stop until someone answers or \
\"consider\" when it should not; leave \"dissent\" out entirely when you agree \
with the task.\n\n\
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
        expertise_block = expertise_block,
        worktree_root = spec.worktree_root.display(),
        files_block = files_block,
        round = spec.round,
        job_id = spec.job_id,
        mailbox = mailbox.display(),
        tmp_name = tmp_name,
        result_file = result_file.display(),
        nickname = spec.nickname,
        cell_id_line = cell_id_line,
        ack_tmp_name = ack_tmp_name,
        ack_file = ack_file.display(),
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
    /// StopAndAsk (a2affcba): the ways forward the worker offers instead of
    /// guessing — one self-contained sentence each. OPTIONAL at parse and
    /// empty when absent, because a foreign agent that skips it still owes a
    /// usable result: strict validation here would turn a good blocked answer
    /// into `Malformed` and cost a whole round.
    pub options: Vec<String>,
    /// StopAndAsk (a2affcba): the option the worker would pick, written as a
    /// verbatim repeat of one `options` entry. Free text, never an index —
    /// membership is NOT enforced, for the same reason `options` is optional.
    pub leaning: Option<String>,
    /// slp-followup-gaps D3: the worker's structured disagreement with the
    /// TASK it was handed, carried as DATA — a herding worker is
    /// bee-ignorant and never runs a command, so this object on the result
    /// file is its only voice. OPTIONAL at parse and read leniently for the
    /// same reason `options`/`leaning` are: a partial or wrong-typed object
    /// reads as absent and never costs the round its result.
    pub dissent: Option<MailboxDissent>,
}

/// One carried dissent — the three fields `record_dissent` needs, and
/// nothing else. `severity` is a PLAIN STRING here on purpose (D4): the
/// closed set (`blocker` / `consider`) is checked by `record_dissent` alone,
/// and a second copy of a closed set is the drift a boundary listed twice
/// always earns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MailboxDissent {
    pub claim: String,
    pub alternative: String,
    pub severity: String,
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

/// True when the round's ack file — `ack-N.json` exactly — appears in
/// `entries` (a directory listing): the pure half of the ack-presence
/// check, mirroring `latest_result_round`'s "entries in, no filesystem"
/// shape. Unlike `latest_result_round` there is only ever one round to ask
/// about at a time (the round currently in flight), so this checks the
/// exact filename rather than hunting for a maximum — `ack-1.json.tmp` or
/// any other partial-write name never counts.
pub(crate) fn ack_present(entries: &[String], round: u32) -> bool {
    let name = ack_filename(round);
    entries.iter().any(|e| e == &name)
}

fn parse_result_filename(name: &str) -> Option<u32> {
    let digits = name.strip_prefix("result-")?.strip_suffix(".json")?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse::<u32>().ok()
}

/// The `dissent` value off a result object, read leniently: `None` unless
/// the value is an object carrying all three of `claim`, `alternative` and
/// `severity` as strings. Nothing here validates the severity — that is
/// `record_dissent`'s single check (D4).
fn parse_dissent(value: Option<&Value>) -> Option<MailboxDissent> {
    let obj = value?.as_object()?;
    let field = |k: &str| obj.get(k).and_then(Value::as_str).map(str::to_string);
    Some(MailboxDissent {
        claim: field("claim")?,
        alternative: field("alternative")?,
        severity: field("severity")?,
    })
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

    // StopAndAsk: both fields are OPTIONAL and never checked for membership —
    // absent, wrong-typed, or a leaning matching no option all parse, exactly
    // as a result parsed before this pair existed.
    let options = obj
        .get("options")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(|v| v.as_str().unwrap_or_default().to_string()).collect())
        .unwrap_or_default();

    let leaning = obj.get("leaning").and_then(Value::as_str).map(str::to_string);

    // slp-followup-gaps D3: read exactly as leniently as the pair above.
    // Absent, not an object, or missing any one of the three fields all read
    // as NO dissent — never a `Malformed` round. The severity string is
    // passed through unchecked (D4): `record_dissent` owns that closed set.
    let dissent = parse_dissent(obj.get("dissent"));

    Ok(MailboxResult { status, summary, files_changed, proof, options, leaning, dissent })
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
            expertise: &[],
            nickname: "w-job-42",
            cell_id: None,
        }
    }

    /// Every `bee <word>` COMMAND the brief names, for D6's negative pin.
    /// A list of bee's command groups would leak the group nobody thought
    /// of, so this works the other way round: it finds every standalone
    /// `bee` token followed by a space and reports the word after it,
    /// unless that word is one of the brief's two prose neighbours. A new
    /// `bee anything` line therefore fails on its own, with no list to keep.
    fn bee_command_mentions(text: &str) -> Vec<String> {
        const PROSE_NEIGHBOURS: [&str; 1] = ["or"];
        let bytes = text.as_bytes();
        let mut found = Vec::new();
        let mut idx = 0usize;
        while let Some(pos) = text[idx..].find("bee") {
            let at = idx + pos;
            idx = at + 3;
            // A `bee` glued to a word or path segment (`.bee/`, `beehive`)
            // is never a command.
            if at > 0 && !matches!(bytes[at - 1], b' ' | b'\n' | b'`' | b'(') {
                continue;
            }
            let rest = &text[at + 3..];
            let Some(tail) = rest.strip_prefix(' ') else { continue };
            let word = tail.split_whitespace().next().unwrap_or("");
            if word.is_empty() || PROSE_NEIGHBOURS.contains(&word) {
                continue;
            }
            found.push(word.to_string());
        }
        found
    }

    // ─── path layout ─────────────────────────────────────────────────────

    #[test]
    fn mailbox_paths_are_under_bee_mailbox_job_id() {
        let bee_dir = Path::new("/repo/.bee");
        assert_eq!(mailbox_dir(bee_dir, "job-1"), PathBuf::from("/repo/.bee/mailbox/job-1"));
        assert_eq!(job_path(bee_dir, "job-1"), PathBuf::from("/repo/.bee/mailbox/job-1/job.json"));
        assert_eq!(log_path(bee_dir, "job-1"), PathBuf::from("/repo/.bee/mailbox/job-1/log.txt"));
        assert_eq!(result_path(bee_dir, "job-1", 5), PathBuf::from("/repo/.bee/mailbox/job-1/result-5.json"));
        assert_eq!(ack_path(bee_dir, "job-1", 5), PathBuf::from("/repo/.bee/mailbox/job-1/ack-5.json"));
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
        assert!(
            text.contains(&worktree_root.join("src/lib.rs").display().to_string()),
            "missing absolute file path:\n{text}"
        );
        assert!(
            text.contains(&worktree_root.join("src/main.rs").display().to_string()),
            "missing absolute file path:\n{text}"
        );
        assert!(
            text.contains(&mailbox_dir(bee_dir, "job-42").display().to_string()),
            "missing absolute mailbox dir:\n{text}"
        );
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
        // StopAndAsk (a2affcba): the brief teaches the two optional fields, so
        // a blocked worker can hand back a choice instead of prose.
        assert!(text.contains("\"options\""), "missing options field:\n{text}");
        assert!(text.contains("\"leaning\""), "missing leaning field:\n{text}");
        assert!(
            text.contains("leave both out when there is no choice"),
            "brief never says the two fields are optional:\n{text}"
        );
        // slp-followup-gaps D3: a bee-ignorant worker can fill the dissent
        // fields off the brief alone, without running anything.
        assert!(text.contains("\"dissent\""), "missing dissent field:\n{text}");
        assert!(text.contains("\"claim\""), "missing dissent claim field:\n{text}");
        assert!(text.contains("\"alternative\""), "missing dissent alternative field:\n{text}");
        assert!(text.contains("\"severity\""), "missing dissent severity field:\n{text}");
        assert!(
            text.contains("\"blocker\" | \"consider\""),
            "the brief never names the two severities:\n{text}"
        );
        assert!(
            text.contains("leave \"dissent\" out entirely when you agree with the task"),
            "brief never says the dissent object is optional:\n{text}"
        );
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
        assert!(text.contains("Ignore any bee or agent-workflow instructions (gates, cells, claims, state)"), "missing ignore-workflow wording:\n{text}");
        assert!(text.contains("files listed under the Expertise section are yours to read"), "missing expertise-reading wording:\n{text}");
        assert!(text.contains("Never run any `bee` command"), "missing never-run-bee wording:\n{text}");
        // slp-followup-gaps D6: the brief must still never name a bee
        // COMMAND — that half of the old pin is unchanged, and the general
        // scan below makes it stronger than the one literal it used to be.
        // The other half banned the bare substring "dissent", which pinned
        // 6a6b9975's OLD boundary (herding-lane dissent was out of scope).
        // This feature IS that boundary moving, so the FIELD NAME is now
        // expected in the result schema, and only the command form is banned.
        assert!(!text.contains("bee cells"), "the brief names a bee verb:\n{text}");
        assert_eq!(
            bee_command_mentions(&text),
            Vec::<String>::new(),
            "the brief names a bee command:\n{text}"
        );
        assert!(
            text.contains("\"dissent\""),
            "the dissent field name belongs in the result schema now:\n{text}"
        );
        assert!(text.contains("Never claim, cap, or write workflow state under .bee/ - writing your mailbox result file (described below) is the ONE exception."), "missing state-exception wording:\n{text}");
    }

    // ─── first-step ack block (herding-prompt-stall D4) ─────────────────

    #[test]
    fn render_brief_opens_with_the_first_step_ack_block_before_everything_else() {
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files = sample_files();
        let spec = sample_spec(worktree_root, bee_dir, &files, 2);
        let text = render_brief(&spec);

        let ack_pos = text
            .find("# Before any other step — write your delivery ack")
            .expect("missing the first-step ack heading");
        assert_eq!(ack_pos, 0, "the ack block must be the very first thing in the brief:\n{text}");
        let standalone_pos = text
            .find("# You are a standalone executor")
            .expect("missing standalone-executor block");
        assert!(ack_pos < standalone_pos, "ack block must precede the standalone-executor block:\n{text}");
    }

    #[test]
    fn render_brief_ack_block_names_the_absolute_ack_path_and_the_tmp_then_rename_gesture() {
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files = sample_files();
        let spec = sample_spec(worktree_root, bee_dir, &files, 2);
        let text = render_brief(&spec);

        assert!(text.to_lowercase().contains("rename"), "no rename instruction for the ack file:\n{text}");
        assert!(text.contains("ack-2.json.tmp"), "no named ack temp file:\n{text}");
        assert!(
            text.contains(&ack_path(bee_dir, "job-42", 2).display().to_string()),
            "no absolute ack file path:\n{text}"
        );
        assert!(text.to_lowercase().contains("before any other step") || text.to_lowercase().contains("before you read the task"), "brief does not say the ack comes first:\n{text}");
    }

    #[test]
    fn render_brief_ack_block_carries_nickname_job_id_round_agent_and_received_at() {
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files = sample_files();
        let spec = sample_spec(worktree_root, bee_dir, &files, 3);
        let text = render_brief(&spec);

        assert!(text.contains("\"nickname\": \"w-job-42\""), "missing nickname field:\n{text}");
        assert!(text.contains("\"job_id\": \"job-42\""), "missing job_id field in the ack schema:\n{text}");
        assert!(text.contains("\"round\": 3"), "missing round field in the ack schema:\n{text}");
        assert!(text.contains("\"agent\""), "missing agent field in the ack schema:\n{text}");
        assert!(text.contains("\"received_at\""), "missing received_at field in the ack schema:\n{text}");
    }

    #[test]
    fn render_brief_ack_block_includes_cell_id_only_when_the_spec_carries_one() {
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files = sample_files();

        let mut spec = sample_spec(worktree_root, bee_dir, &files, 1);
        let without_cell = render_brief(&spec);
        assert!(!without_cell.contains("\"cell_id\""), "cell_id must be omitted when the spec carries none:\n{without_cell}");

        spec.cell_id = Some("hps-3");
        let with_cell = render_brief(&spec);
        assert!(with_cell.contains("\"cell_id\": \"hps-3\""), "missing cell_id field when the spec carries one:\n{with_cell}");
    }

    #[test]
    fn render_brief_keeps_the_result_form_block_byte_identical_apart_from_the_new_ack_block() {
        // D4's must_haves: existing Result-form block, round numbering and
        // schema block stay byte-identical apart from the new block.
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files = sample_files();
        let spec = sample_spec(worktree_root, bee_dir, &files, 1);
        let text = render_brief(&spec);

        // StopAndAsk (a2affcba) added two OPTIONAL schema lines and one
        // sentence saying when to fill them; slp-followup-gaps D3 added the
        // OPTIONAL `dissent` object and one sentence saying when to fill it.
        // Every other byte of this block — heading, lead-in, and the four
        // original fields — is unchanged.
        assert!(text.contains("# Result contract\n\nWhen you are done, or genuinely blocked, write EXACTLY ONE JSON object matching\nthis schema, and nothing else, to the result file:\n\n{\n\"status\": \"done\" | \"blocked\",\n\"summary\": \"<one line: what happened>\",\n\"files_changed\": [\"<path>\", \"...\"],\n\"proof\": \"<command or evidence that backs the status>\",\n\"options\": [\"<one self-contained sentence per way forward>\", \"...\"],\n\"leaning\": \"<the one option you would pick, repeated word for word>\",\n\"dissent\": { \"claim\": \"<what is wrong with the task as it was handed to you>\", \"alternative\": \"<what you would do instead>\", \"severity\": \"blocker\" | \"consider\" }\n}\n\nWhen \"blocked\" leaves a choice to make, fill \"options\" with one self-contained sentence per way forward and \"leaning\" with the one you would pick, repeated word for word; leave both out when there is no choice.\nFill \"dissent\" only when you disagree with the TASK ITSELF — \"claim\" says what is wrong with it, \"alternative\" says what you would do instead, and \"severity\" is \"blocker\" when the work should stop until someone answers or \"consider\" when it should not; leave \"dissent\" out entirely when you agree with the task.\n\n"), "Result-form block drifted:\n{text}");
        assert!(
            text.contains(&format!(
                "temp file (write your JSON here):   {}/result-1.json.tmp\n",
                mailbox_dir(bee_dir, "job-42").display()
            )),
            "result temp-file line drifted:\n{text}"
        );
    }

    #[test]
    fn render_brief_with_two_expertise_entries_renders_expertise_section() {
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files = sample_files();
        let expertise = vec![
            ExpertiseEntry {
                path: "skills/bee-swarming/references/swarming-reference.md".to_string(),
                purpose: "prompt template details".to_string(),
                read_to: "structure worker prompt".to_string(),
            },
            ExpertiseEntry {
                path: "/abs/path/docs/knowledge/foo.md".to_string(),
                purpose: "domain background".to_string(),
                read_to: "understand area rules".to_string(),
            },
        ];
        let spec = BriefSpec {
            job_id: "job-42",
            task: "Fix the off-by-one in the paginator.",
            worktree_root,
            files: &files,
            bee_dir,
            round: 1,
            expertise: &expertise,
            nickname: "w-job-42",
            cell_id: None,
        };
        let text = render_brief(&spec);

        let task_pos = text.find("# Task").expect("missing # Task heading");
        let exp_pos = text.find("# Expertise — read these before you start").expect("missing # Expertise heading");
        let cwd_pos = text.find("# Working directory (absolute)").expect("missing # Working directory heading");
        assert!(task_pos < exp_pos, "# Task must come before # Expertise");
        assert!(exp_pos < cwd_pos, "# Expertise must come before # Working directory");

        assert!(text.contains(&format!(
            "  - {} — prompt template details. Read it to structure worker prompt.",
            worktree_root.join("skills/bee-swarming/references/swarming-reference.md").display()
        )));
        assert!(text.contains("  - /abs/path/docs/knowledge/foo.md — domain background. Read it to understand area rules."));
    }

    #[test]
    fn render_brief_with_zero_expertise_is_byte_identical_to_empty_spec() {
        let worktree_root = Path::new("/repo/work");
        let bee_dir = Path::new("/repo/.bee");
        let files = sample_files();
        let spec_no_exp = BriefSpec {
            job_id: "job-42",
            task: "Fix the off-by-one in the paginator.",
            worktree_root,
            files: &files,
            bee_dir,
            round: 1,
            expertise: &[],
            nickname: "w-job-42",
            cell_id: None,
        };
        let text = render_brief(&spec_no_exp);
        assert!(!text.contains("# Expertise"));
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

    // ─── ack_present ──────────────────────────────────────────────────────

    #[test]
    fn ack_present_is_true_only_for_the_exact_round_filename() {
        let entries = vec!["ack-1.json".to_string(), "ack-3.json".to_string()];
        assert!(ack_present(&entries, 1));
        assert!(ack_present(&entries, 3));
        assert!(!ack_present(&entries, 2), "round 2 has no ack file");
    }

    #[test]
    fn ack_present_ignores_tmp_and_unrelated_names() {
        let entries = vec![
            "ack-1.json.tmp".to_string(),
            "ack-x.json".to_string(),
            "job.json".to_string(),
            "result-1.json".to_string(),
        ];
        assert!(!ack_present(&entries, 1), "a partial write must never count as present");
    }

    #[test]
    fn ack_present_is_false_for_an_empty_mailbox() {
        let entries: Vec<String> = Vec::new();
        assert!(!ack_present(&entries, 1));
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

    // ─── StopAndAsk: options[] and leaning (a2affcba) ────────────────────

    #[test]
    fn parse_result_text_reads_options_and_leaning_off_a_blocked_result() {
        let text = r#"{"status":"blocked","summary":"two ways to do this","files_changed":[],"proof":"n/a","options":["Widen the cell to cover the parser too.","Split the parser into its own cell."],"leaning":"Split the parser into its own cell."}"#;
        let result = parse_result_text(1, text).expect("valid result should parse");
        assert_eq!(result.status, MailboxStatus::Blocked);
        assert_eq!(
            result.options,
            vec![
                "Widen the cell to cover the parser too.".to_string(),
                "Split the parser into its own cell.".to_string(),
            ]
        );
        assert_eq!(result.leaning.as_deref(), Some("Split the parser into its own cell."));
    }

    #[test]
    fn parse_result_text_leaves_options_and_leaning_empty_when_a_result_carries_neither() {
        // The negative half of the round trip: a result written before this
        // pair existed parses exactly as it always did.
        let text = r#"{"status":"done","summary":"fixed it","files_changed":["a.rs"],"proof":"cargo test — green"}"#;
        let result = parse_result_text(1, text).expect("valid result should parse");
        assert!(result.options.is_empty(), "options must be empty when absent: {:?}", result.options);
        assert_eq!(result.leaning, None);
    }

    #[test]
    fn parse_result_text_accepts_a_leaning_that_matches_no_option() {
        // Membership is NEVER enforced: strict validation of foreign-agent
        // output would turn a useful blocked answer into a whole lost round.
        let text = r#"{"status":"blocked","summary":"stuck","files_changed":[],"proof":"n/a","options":["Do A."],"leaning":"Actually, do C."}"#;
        let result = parse_result_text(1, text).expect("a mismatched leaning still parses");
        assert_eq!(result.options, vec!["Do A.".to_string()]);
        assert_eq!(result.leaning.as_deref(), Some("Actually, do C."));
    }

    #[test]
    fn parse_result_text_ignores_wrong_typed_options_and_leaning() {
        // Same lenience one level down: a wrong-typed pair is absent, never a
        // refusal, and a non-string element degrades to "" like files_changed.
        let text = r#"{"status":"blocked","summary":"stuck","files_changed":[],"proof":"n/a","options":"not an array","leaning":7}"#;
        let result = parse_result_text(1, text).expect("wrong-typed fields must not refuse");
        assert!(result.options.is_empty());
        assert_eq!(result.leaning, None);

        let mixed = r#"{"status":"blocked","summary":"stuck","files_changed":[],"proof":"n/a","options":["Do A.",7]}"#;
        let result = parse_result_text(1, mixed).expect("a non-string option must not refuse");
        assert_eq!(result.options, vec!["Do A.".to_string(), String::new()]);
    }

    // ─── the carried dissent (slp-followup-gaps D3/D4) ───────────────────

    #[test]
    fn parse_result_text_reads_a_full_dissent_object_off_a_result() {
        let text = r#"{"status":"blocked","summary":"the cell is wrong","files_changed":[],"proof":"n/a","dissent":{"claim":"The paginator is not the bug.","alternative":"Fix the cursor encoder instead.","severity":"blocker"}}"#;
        let result = parse_result_text(1, text).expect("a result carrying a dissent parses");
        assert_eq!(
            result.dissent,
            Some(MailboxDissent {
                claim: "The paginator is not the bug.".to_string(),
                alternative: "Fix the cursor encoder instead.".to_string(),
                severity: "blocker".to_string(),
            })
        );
    }

    #[test]
    fn parse_result_text_leaves_dissent_absent_when_a_result_carries_none() {
        let text = r#"{"status":"done","summary":"did it","files_changed":["a.rs"],"proof":"cargo test"}"#;
        let result = parse_result_text(1, text).expect("a result with no dissent still parses");
        assert_eq!(result.dissent, None);
    }

    #[test]
    fn parse_result_text_reads_a_partial_or_wrong_typed_dissent_as_absent() {
        // Missing `alternative`, missing `severity`, missing `claim`, and a
        // non-object body: every one of them reads as NO dissent and the
        // round still comes back green.
        for body in [
            r#""dissent":{"claim":"c","severity":"blocker"}"#,
            r#""dissent":{"claim":"c","alternative":"a"}"#,
            r#""dissent":{"alternative":"a","severity":"blocker"}"#,
            r#""dissent":"just a sentence""#,
            r#""dissent":["c","a","blocker"]"#,
            r#""dissent":{"claim":7,"alternative":"a","severity":"blocker"}"#,
            r#""dissent":null"#,
        ] {
            let text = format!(
                r#"{{"status":"blocked","summary":"stuck","files_changed":[],"proof":"n/a",{body}}}"#
            );
            let result = parse_result_text(1, &text).expect("a malformed dissent never fails the round");
            assert_eq!(result.dissent, None, "malformed dissent must read as absent: {text}");
        }
    }

    #[test]
    fn parse_result_text_passes_an_unknown_severity_straight_through() {
        // D4: the closed set is `record_dissent`'s single check. A second
        // copy here would be the drift a boundary listed twice always earns,
        // so the parser never looks at the value.
        let text = r#"{"status":"blocked","summary":"stuck","files_changed":[],"proof":"n/a","dissent":{"claim":"c","alternative":"a","severity":"catastrophic"}}"#;
        let result = parse_result_text(1, text).expect("an unknown severity still parses");
        assert_eq!(result.dissent.expect("dissent present").severity, "catastrophic");
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

    // ── activity record (herding-activity-hook D2/D3) ────────────────────

    /// The record shape `hooks/activity.rs` writes under the herded-worker
    /// marker — one helper so every case below varies exactly one field.
    fn activity_text(state: &str, round: u32, at: &str) -> String {
        format!(
            r#"{{"state":"{state}","event":"PreToolUse","tool_name":"Bash","at":"{at}","job_id":"job-42","round":{round}}}"#
        )
    }

    fn at_secs_ago(now_ms: i64, secs: i64) -> String {
        chrono::DateTime::from_timestamp_millis(now_ms - secs * 1000).unwrap().to_rfc3339()
    }

    #[test]
    fn activity_path_is_one_unnumbered_file_beside_the_round_numbered_ones() {
        let bee_dir = Path::new("/repo/.bee");
        assert_eq!(activity_path(bee_dir, "job-42"), Path::new("/repo/.bee/mailbox/job-42/activity.json"));
    }

    #[test]
    fn parse_activity_text_reads_every_state_in_the_hook_vocabulary() {
        let now_ms = 1_700_000_000_000i64;
        let at = at_secs_ago(now_ms, 1);
        for (word, want) in [
            ("working", ActivityState::Working),
            ("waiting_input", ActivityState::WaitingInput),
            ("blocked", ActivityState::Blocked),
            ("idle", ActivityState::Idle),
            ("exited", ActivityState::Exited),
        ] {
            assert_eq!(parse_activity_text(&activity_text(word, 1, &at), 1, now_ms), Some(want), "state {word}");
        }
    }

    #[test]
    fn parse_activity_text_refuses_malformed_input() {
        let now_ms = 1_700_000_000_000i64;
        let at = at_secs_ago(now_ms, 1);
        // Not JSON, not an object, and a half-written prefix — every one
        // falls back rather than speaking for the agent.
        assert_eq!(parse_activity_text("not json at all", 1, now_ms), None);
        assert_eq!(parse_activity_text("[1,2,3]", 1, now_ms), None);
        assert_eq!(parse_activity_text(r#"{"state":"blocked","at":"#, 1, now_ms), None);
        // Missing round, missing at, missing state.
        assert_eq!(parse_activity_text(&format!(r#"{{"state":"blocked","at":"{at}"}}"#), 1, now_ms), None);
        assert_eq!(parse_activity_text(r#"{"state":"blocked","round":1}"#, 1, now_ms), None);
        assert_eq!(parse_activity_text(&format!(r#"{{"round":1,"at":"{at}"}}"#), 1, now_ms), None);
        // Mistyped round, unparseable timestamp, unknown state word.
        assert_eq!(parse_activity_text(&format!(r#"{{"state":"blocked","round":"1","at":"{at}"}}"#), 1, now_ms), None);
        assert_eq!(parse_activity_text(r#"{"state":"blocked","round":1,"at":"yesterday"}"#, 1, now_ms), None);
        assert_eq!(parse_activity_text(&activity_text("thinking", 1, &at), 1, now_ms), None);
    }

    #[test]
    fn parse_activity_text_fences_a_record_from_an_older_round() {
        // D2: the round is the launch-id fence — round 1's leftover record
        // can never describe round 2.
        let now_ms = 1_700_000_000_000i64;
        let at = at_secs_ago(now_ms, 1);
        assert_eq!(parse_activity_text(&activity_text("blocked", 1, &at), 2, now_ms), None);
        // The current round passes, and so does a LATER one: the pane moved
        // on, and its state is still this agent's state.
        assert_eq!(parse_activity_text(&activity_text("blocked", 2, &at), 2, now_ms), Some(ActivityState::Blocked));
        assert_eq!(parse_activity_text(&activity_text("blocked", 3, &at), 2, now_ms), Some(ActivityState::Blocked));
    }

    #[test]
    fn parse_activity_text_fences_a_stale_record_but_keeps_a_future_stamped_one() {
        let now_ms = 1_700_000_000_000i64;
        let fresh = at_secs_ago(now_ms, ACTIVITY_FRESHNESS_SECS - 1);
        let edge = at_secs_ago(now_ms, ACTIVITY_FRESHNESS_SECS);
        let stale = at_secs_ago(now_ms, ACTIVITY_FRESHNESS_SECS + 1);
        let future = at_secs_ago(now_ms, -30);
        assert_eq!(parse_activity_text(&activity_text("blocked", 1, &fresh), 1, now_ms), Some(ActivityState::Blocked));
        assert_eq!(
            parse_activity_text(&activity_text("blocked", 1, &edge), 1, now_ms),
            Some(ActivityState::Blocked),
            "exactly at the bound still counts"
        );
        assert_eq!(parse_activity_text(&activity_text("blocked", 1, &stale), 1, now_ms), None);
        assert_eq!(
            parse_activity_text(&activity_text("blocked", 1, &future), 1, now_ms),
            Some(ActivityState::Blocked),
            "clock skew is jitter, never staleness"
        );
    }
}
