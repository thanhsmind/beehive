// herding::run — `bee herding run` (herding-executor D1, D2, D5, D6, D9;
// docs/history/herding-executor/CONTEXT.md).
//
// The scope-A verb the whole feature exists to ship: start ONE bee-ignorant
// external agent (any herdr-supported kind) in a fresh pane, hand it a
// fully self-contained brief over the file mailbox (`super::mailbox`), wait
// on that mailbox with health-check liveness at zero token cost, and return
// one structured result.
//
//   1. Resolve `herding.agent_command` (D14, `super::wave::resolve_agent_command`
//      — the SAME split `bee herding wave` already uses) into (kind, args).
//   2. Split a pane from the caller's OWN runtime pane
//      (`herdr pane current --current`, then `herdr pane split … --cwd
//      <worktree>`) — the split-then-start order `spawn-proof.md` records
//      as the only one herdr 0.8.0 accepts.
//   3. Start the agent into that pane (`herdr agent start … --pane …`),
//      handing it the rendered brief as its opening prompt. A start failure
//      closes the pane it just created (role-dispatch.md §8's own cleanup
//      rule) and refuses; nothing else touches herdr on that path.
//   4. Append the D9 dispatch.jsonl row and a wave-ledger row (the same
//      ledger `record-worker` writes into) so occupancy counts this worker.
//   5. Poll NATIVELY for completion (D5): `result-N.json` presence,
//      `log.txt` mtime, `herdr agent list` status — no LLM call anywhere on
//      this path. A stale heartbeat past `--idle-timeout` or the absolute
//      `--ceiling` (a hard cap regardless of activity) ends the wait.
//   6. Pane lifecycle (D6): a valid result closes the pane; a failure or a
//      timeout leaves it open as forensics; `--close-always` closes it on
//      every outcome.
//
// `--dry-run` stops after writing `job.json` and rendering the brief —
// nothing in `Herdr` is ever called on that path (proven below by handing
// dry-run execution a fake that panics if any of its methods fire).
//
// STRUCTURAL SPLIT, same shape `control_loop.rs` and `wave.rs` already use:
// every herdr-shaped operation lives behind the `Herdr` trait so a test
// drives the spawn sequence and the pane-lifecycle decision with a fake —
// no real `herdr` on PATH anywhere in this crate's test suite (D7's seam,
// "FakeBackend pattern" per the cell). The poll loop's own decision logic
// (`decide_poll`) and its driving loop (`run_poll_loop`) are pure/injected
// too, so the three timing rules — fresh heartbeat extends, a stale one
// times out, the ceiling caps regardless of activity — run as fast unit
// tests with no real sleep.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

use serde_json::{Map, Value};

use super::mailbox::{self, BriefSpec, MailboxResult, MailboxStatus};
use super::wave::resolve_agent_command;
use super::wave_ledger::{self, WaveRow, WorkerRow};

const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 900; // 15 minutes of no heartbeat
const DEFAULT_CEILING_SECS: u64 = 21_600; // 6 hours, the busy-loop backstop
const POLL_INTERVAL: Duration = Duration::from_millis(200);

// ═══════════════════════════════════════════════════════════════════════════
// CLI parsing
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
struct Options {
    task: String,
    cwd: PathBuf,
    job_id: String,
    idle_timeout_secs: u64,
    ceiling_secs: u64,
    close_always: bool,
    main_root: PathBuf,
    json: bool,
    dry_run: bool,
    /// `--continue <job-id>` (D3): reuse the EXISTING job mailbox instead of
    /// spawning a fresh pane. `job_id` above already carries the target id
    /// in this mode — `execute` branches on this flag alone.
    is_continue: bool,
    /// Ready-wait ceiling (herding-run-ready-wait D1): how long a fresh
    /// spawn waits for the started agent to report ready-for-input before
    /// the round-1 brief is sent. `agent start`'s own success can fire
    /// before the agent accepts input (live smoke smoke-agy-2: the brief
    /// landed above the boot banner and was lost), so the prompt waits for
    /// an observed ready status.
    ready_wait_secs: u64,
}

fn absolute_path(p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().map(|cwd| cwd.join(p)).unwrap_or_else(|_| p.to_path_buf())
    }
}

fn resolve_task(task: Option<&str>, task_file: Option<&str>) -> Result<String, String> {
    resolve_task_with_stdin(task, task_file, super::read_stdin)
}

/// `resolve_task`'s real logic, with the stdin read injected (herding-tier
/// D4): `--task-file -` reads the whole task text from stdin via the SAME
/// `super::read_stdin` helper `bee herding wave` already uses for its
/// worker-specs read. A file path keeps reading from disk exactly as
/// before. Empty stdin refuses with the SAME message an empty `--task` and
/// a missing `--task`/`--task-file` both already produce — "empty" is
/// never a silently-accepted brief.
fn resolve_task_with_stdin(
    task: Option<&str>,
    task_file: Option<&str>,
    read_stdin: impl FnOnce() -> String,
) -> Result<String, String> {
    if let Some(t) = task {
        if !t.is_empty() {
            return Ok(t.to_string());
        }
    }
    if let Some(f) = task_file {
        if f == "-" {
            let text = read_stdin().trim_end_matches('\n').to_string();
            if text.is_empty() {
                return Err("--task or --task-file is required".to_string());
            }
            return Ok(text);
        }
        return std::fs::read_to_string(f)
            .map(|s| s.trim_end_matches('\n').to_string())
            .map_err(|e| format!("could not read --task-file {f}: {e}"));
    }
    Err("--task or --task-file is required".to_string())
}

fn parse_options(flags: &[&str]) -> Result<Options, String> {
    let mut task: Option<&str> = None;
    let mut task_file: Option<&str> = None;
    let mut cwd: Option<&str> = None;
    let mut job_id: Option<&str> = None;
    let mut continue_job_id: Option<&str> = None;
    let mut idle_timeout_secs = DEFAULT_IDLE_TIMEOUT_SECS;
    let mut ceiling_secs = DEFAULT_CEILING_SECS;
    let mut close_always = false;
    let mut explicit_root: Option<&str> = None;
    let mut json = false;
    let mut dry_run = false;
    let mut i = 0usize;
    while i < flags.len() {
        match flags[i] {
            "--task" => {
                task = flags.get(i + 1).copied();
                i += 2;
            }
            "--task-file" => {
                task_file = flags.get(i + 1).copied();
                i += 2;
            }
            "--cwd" => {
                cwd = flags.get(i + 1).copied();
                i += 2;
            }
            "--job-id" => {
                job_id = flags.get(i + 1).copied();
                i += 2;
            }
            "--continue" => {
                continue_job_id = flags.get(i + 1).copied();
                i += 2;
            }
            "--idle-timeout" => {
                if let Some(n) = flags.get(i + 1).and_then(|s| s.parse().ok()) {
                    idle_timeout_secs = n;
                }
                i += 2;
            }
            "--ceiling" => {
                if let Some(n) = flags.get(i + 1).and_then(|s| s.parse().ok()) {
                    ceiling_secs = n;
                }
                i += 2;
            }
            "--close-always" => {
                close_always = true;
                i += 1;
            }
            "--main-root" => {
                explicit_root = flags.get(i + 1).copied();
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            "--dry-run" => {
                dry_run = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let task_text = resolve_task(task, task_file)?;
    let main_root = super::resolve_main_root(explicit_root).ok_or_else(|| {
        "could not resolve the MAIN checkout root (no --main-root given and `git rev-parse \
         --git-common-dir` failed)"
            .to_string()
    })?;
    let cwd_path = match cwd {
        Some(c) => absolute_path(Path::new(c)),
        None => std::env::current_dir()
            .map_err(|e| format!("could not resolve the current directory: {e}"))?,
    };
    let (job_id, is_continue) = match continue_job_id {
        Some(id) => (id.to_string(), true),
        None => (
            job_id.map(str::to_string).unwrap_or_else(|| format!("job-{}", chrono::Utc::now().timestamp_millis())),
            false,
        ),
    };

    Ok(Options {
        task: task_text,
        cwd: cwd_path,
        job_id,
        idle_timeout_secs,
        ceiling_secs,
        close_always,
        main_root,
        json,
        dry_run,
        is_continue,
        ready_wait_secs: 60,
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// the Herdr seam — every herdr-shaped operation, real or faked
// ═══════════════════════════════════════════════════════════════════════════

/// Every herdr operation `bee herding run` needs, isolated behind a trait so
/// tests inject a fake instead of a real `herdr` on PATH (D7's seam, no
/// process anywhere in this crate's test suite). `RealHerdr` below is the
/// only production implementer.
trait Herdr {
    /// `herdr pane current --current` — the caller's OWN pane id, the pane
    /// this verb splits from.
    fn pane_current(&self) -> Result<String, String>;
    /// `herdr pane layout --pane <id>` — best-effort geometry for the
    /// split-direction rule (`role-dispatch.md` §8); `None` on any trouble
    /// (herdr missing, unparseable body, the pane absent from the reply) —
    /// the caller falls back to a default direction rather than failing the
    /// whole run over a geometry read.
    fn pane_rect(&self, pane_id: &str) -> Option<(u64, u64)>;
    /// `herdr pane split <id> --direction <dir> --ratio 0.5 --cwd <cwd>
    /// --no-focus` — returns the newly split pane's id.
    fn pane_split(&self, pane_id: &str, direction: &str, cwd: &Path) -> Result<String, String>;
    /// `herdr agent start <job_id> --kind <kind> --pane <pane_id> --timeout
    /// 60000 -- <args…>` — the split pane IS the agent's pane
    /// (spawn-proof.md), never a second one. The brief is NEVER an argv
    /// token here: a multi-line brief cannot be encoded for the target
    /// shell (live smoke smoke-agy-1, herdr `invalid_agent_argument`) — it
    /// travels through `agent_prompt` after the start succeeds
    /// (herding-run-prompt-delivery D1).
    fn agent_start(
        &self,
        job_id: &str,
        kind: &str,
        pane_id: &str,
        args: &[String],
    ) -> Result<(), String>;
    /// `herdr agent list`'s `agent_status` for `job_id`, or `None` on any
    /// trouble (herdr missing, the target absent, an unparseable body) —
    /// never a "safe" guess (Ordering Invariant 4's rule, mirrored from
    /// `fleet::backend::herdr`): an unverifiable status never counts as a
    /// heartbeat.
    fn agent_status(&self, job_id: &str) -> Option<String>;
    /// `herdr pane close <id>` — best-effort; a failure here is reported,
    /// never allowed to hide the run's own result.
    fn pane_close(&self, pane_id: &str) -> Result<(), String>;
    /// `herdr agent prompt <job_id> <prompt>` (D3 `--continue`) — sends the
    /// round N+1 brief to an ALREADY-RUNNING agent, never `agent start`
    /// (that would spawn a second agent instead of continuing this one).
    fn agent_prompt(&self, job_id: &str, prompt: &str) -> Result<(), String>;
    /// `herdr pane list`, membership-tested against `pane_id` — the D3
    /// `--continue` "is the pane gone" check. Unlike `pane_rect` (which
    /// fails open to a default direction on ANY trouble), this fails
    /// CLOSED: herdr missing, a non-zero exit, or an unparseable body all
    /// read as "not alive" — `--continue` refuses rather than prompting a
    /// pane it cannot confirm still exists.
    fn pane_alive(&self, pane_id: &str) -> bool;
}

struct RealHerdr;

impl RealHerdr {
    fn call(&self, args: &[&str]) -> Result<Value, String> {
        let out = Command::new("herdr")
            .args(args)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("could not spawn herdr: {e}"))?;
        if !out.status.success() {
            let code = out.status.code().map_or_else(|| "unknown".to_string(), |c| c.to_string());
            return Err(format!(
                "herdr {} exited {code}: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        serde_json::from_slice(&out.stdout).map_err(|_| {
            format!("herdr {} returned a body that was not valid JSON: {}", args.join(" "), String::from_utf8_lossy(&out.stdout))
        })
    }
}

impl Herdr for RealHerdr {
    fn pane_current(&self) -> Result<String, String> {
        let v = self.call(&["pane", "current", "--current"])?;
        v.get("result")
            .and_then(|r| r.get("pane"))
            .and_then(|p| p.get("pane_id"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| "herdr pane current --current: missing result.pane.pane_id".to_string())
    }

    fn pane_rect(&self, pane_id: &str) -> Option<(u64, u64)> {
        let v = self.call(&["pane", "layout", "--pane", pane_id]).ok()?;
        let result = v.get("result")?;
        let panes = match result {
            Value::Array(a) => a.clone(),
            Value::Object(o) => match o.get("panes") {
                Some(Value::Array(a)) => a.clone(),
                _ => vec![result.clone()],
            },
            _ => return None,
        };
        let entry = panes
            .iter()
            .find(|p| p.get("pane_id").and_then(Value::as_str) == Some(pane_id))
            .or_else(|| panes.first())?;
        let rect = entry.get("rect")?;
        let w = rect.get("width")?.as_u64()?;
        let h = rect.get("height")?.as_u64()?;
        Some((w, h))
    }

    fn pane_split(&self, pane_id: &str, direction: &str, cwd: &Path) -> Result<String, String> {
        let cwd_str = cwd.display().to_string();
        let v = self.call(&[
            "pane", "split", pane_id, "--direction", direction, "--ratio", "0.5", "--cwd", &cwd_str,
            "--no-focus",
        ])?;
        v.get("result")
            .and_then(|r| r.get("pane"))
            .and_then(|p| p.get("pane_id"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| "herdr pane split: missing result.pane.pane_id".to_string())
    }

    fn agent_start(
        &self,
        job_id: &str,
        kind: &str,
        pane_id: &str,
        args: &[String],
    ) -> Result<(), String> {
        let mut argv: Vec<&str> =
            vec!["agent", "start", job_id, "--kind", kind, "--pane", pane_id, "--timeout", "60000"];
        if !args.is_empty() {
            argv.push("--");
            for a in args {
                argv.push(a);
            }
        }
        self.call(&argv).map(|_| ())
    }

    fn agent_status(&self, job_id: &str) -> Option<String> {
        let v = self.call(&["agent", "list"]).ok()?;
        let agents = v.get("result")?.get("agents")?.as_array()?;
        let entry = agents.iter().find(|a| {
            a.get("name").and_then(Value::as_str) == Some(job_id)
                || a.get("pane_id").and_then(Value::as_str) == Some(job_id)
        })?;
        entry.get("agent_status").and_then(Value::as_str).map(str::to_string)
    }

    fn pane_close(&self, pane_id: &str) -> Result<(), String> {
        self.call(&["pane", "close", pane_id]).map(|_| ())
    }

    fn agent_prompt(&self, job_id: &str, prompt: &str) -> Result<(), String> {
        self.call(&["agent", "prompt", job_id, prompt]).map(|_| ())
    }

    fn pane_alive(&self, pane_id: &str) -> bool {
        // Mirrors `wave::live_pane_ids_via_herdr`'s parse: a bare array or
        // `{"panes": […]}`, an explicit `error` envelope, or any other
        // shape all fail CLOSED (not alive) — this is a refusal gate, not
        // an occupancy estimate.
        let Ok(v) = self.call(&["pane", "list"]) else { return false };
        if v.get("error").is_some_and(|e| !e.is_null()) {
            return false;
        }
        let result = v.get("result").cloned().unwrap_or(Value::Null);
        let panes = match &result {
            Value::Array(a) => a.clone(),
            Value::Object(o) => match o.get("panes") {
                Some(Value::Array(a)) => a.clone(),
                _ => return false,
            },
            _ => return false,
        };
        panes.iter().any(|p| p.get("pane_id").and_then(Value::as_str) == Some(pane_id))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// pure decisions — split direction, poll timing, pane lifecycle
// ═══════════════════════════════════════════════════════════════════════════

/// `role-dispatch.md` §8's geometry rule: wider than tall (or square) splits
/// right, taller than wide splits down. Pure: takes an already-read rect.
fn split_direction(width: u64, height: u64) -> &'static str {
    if width >= height {
        "right"
    } else {
        "down"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PollDecision {
    Continue,
    ResultReady,
    TimedOutIdle,
    TimedOutCeiling,
}

/// The whole D5 timing rule, pure: a result already present short-circuits
/// everything else; otherwise the absolute ceiling caps the wait REGARDLESS
/// of a fresh heartbeat (checked first, so it wins a tie with the idle
/// check); a heartbeat gone stale past `idle_timeout_secs` times out; a
/// fresh one lets the wait continue.
fn decide_poll(
    now_ms: i64,
    started_at_ms: i64,
    last_heartbeat_ms: i64,
    idle_timeout_secs: u64,
    ceiling_secs: u64,
    result_ready: bool,
) -> PollDecision {
    if result_ready {
        return PollDecision::ResultReady;
    }
    if now_ms.saturating_sub(started_at_ms) >= (ceiling_secs as i64).saturating_mul(1000) {
        return PollDecision::TimedOutCeiling;
    }
    if now_ms.saturating_sub(last_heartbeat_ms) >= (idle_timeout_secs as i64).saturating_mul(1000) {
        return PollDecision::TimedOutIdle;
    }
    PollDecision::Continue
}

/// D6's pane lifecycle, pure: a valid (well-formed) result closes the pane;
/// a failure or timeout leaves it open as forensics; `close_always`
/// overrides in every outcome.
fn should_close_pane(valid_result: bool, close_always: bool) -> bool {
    close_always || valid_result
}

/// One poll tick's raw observations, gathered by the caller (real fs +
/// herdr reads in production, a scripted fake in tests) and handed to
/// `run_poll_loop` as a pure `bool` pair.
struct PollTick {
    result_ready: bool,
    heartbeat_fresh: bool,
}

/// The loop `decide_poll` drives: sleep, observe, decide, repeat until a
/// non-`Continue` decision. `tick`/`sleep`/`now` are injected so a test
/// drives the whole loop — not just one call to `decide_poll` — with a
/// simulated clock and no real sleep (see `tests` below).
fn run_poll_loop(
    started_at_ms: i64,
    idle_timeout_secs: u64,
    ceiling_secs: u64,
    poll_interval: Duration,
    mut tick: impl FnMut() -> PollTick,
    mut sleep: impl FnMut(Duration),
    mut now: impl FnMut() -> i64,
) -> PollDecision {
    let mut last_heartbeat_ms = started_at_ms;
    loop {
        sleep(poll_interval);
        let tick_now_ms = now();
        let observed = tick();
        if observed.heartbeat_fresh {
            last_heartbeat_ms = tick_now_ms;
        }
        let decision = decide_poll(
            tick_now_ms,
            started_at_ms,
            last_heartbeat_ms,
            idle_timeout_secs,
            ceiling_secs,
            observed.result_ready,
        );
        if decision != PollDecision::Continue {
            return decision;
        }
    }
}

/// herding-run-ready-wait D1: polls `status()` until the agent reports a
/// real status (`idle` ready-for-input, or already `working`/`done`) or the
/// ready-wait ceiling passes. Check-then-sleep: an agent already ready is
/// accepted with zero sleeps, and a 0-second ceiling with no status is
/// exhausted immediately — both drive the tests without a real clock.
fn wait_for_agent_ready(
    ready_wait_secs: u64,
    poll_interval: Duration,
    mut status: impl FnMut() -> Option<String>,
    mut sleep: impl FnMut(Duration),
    mut now: impl FnMut() -> i64,
) -> bool {
    let started = now();
    loop {
        if matches!(status().as_deref(), Some("idle") | Some("working") | Some("done")) {
            return true;
        }
        if now().saturating_sub(started) >= (ready_wait_secs as i64).saturating_mul(1000) {
            return false;
        }
        sleep(poll_interval);
    }
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

// ═══════════════════════════════════════════════════════════════════════════
// the run itself
// ═══════════════════════════════════════════════════════════════════════════

/// The three typed refusals `--continue` can hit before it ever touches
/// `Herdr` for real (D3's "refuses typed when…" clause). Each names the
/// job id so the caller can tell which job it asked about.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ContinueRefusal {
    /// No `.bee/mailbox/<job-id>/` directory, or no readable `job.json`
    /// inside it — there is no job to continue.
    JobDirMissing { job_id: String },
    /// The job dir exists but holds no `result-N.json` at all — a round
    /// this job never finished has nothing to continue FROM.
    NoPriorResult { job_id: String },
    /// `job.json` recorded no pane, or `herdr pane list` no longer shows
    /// the one it did record — the agent this job would prompt is gone.
    PaneGone { job_id: String, pane_id: Option<String> },
}

impl std::fmt::Display for ContinueRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContinueRefusal::JobDirMissing { job_id } => write!(
                f,
                "--continue {job_id}: no job mailbox found (no job.json under .bee/mailbox/{job_id}/) — \
                 run without --continue to start a new job"
            ),
            ContinueRefusal::NoPriorResult { job_id } => write!(
                f,
                "--continue {job_id}: no prior result-N.json in this job's mailbox — nothing to continue from"
            ),
            ContinueRefusal::PaneGone { job_id, pane_id: Some(p) } => write!(
                f,
                "--continue {job_id}: pane {p} is no longer alive (absent from `herdr pane list`) — cannot continue"
            ),
            ContinueRefusal::PaneGone { job_id, pane_id: None } => write!(
                f,
                "--continue {job_id}: job.json has no recorded pane — cannot continue"
            ),
        }
    }
}

#[derive(Debug)]
enum RunOutcome {
    /// `--dry-run`: the rendered brief, for inspection — nothing spawned.
    DryRun(String),
    /// `--continue` refused before touching `Herdr` for real: the job dir,
    /// prior result, or recorded pane was missing (D3).
    ContinueRefused(ContinueRefusal),
    /// Could not even start: agent-command resolution, a pane operation, or
    /// `agent start` itself failed. Any pane this created is already closed
    /// by the time this variant is returned.
    SpawnFailed(String),
    /// A `result-N.json` was found and parsed — `status` may be `done` or
    /// `blocked`; either is a WELL-FORMED completion signal (D6's "valid
    /// result").
    Result(MailboxResult),
    /// A result file appeared but could not be read or did not pass the
    /// mailbox schema — never a valid result.
    Malformed(String),
    TimedOutIdle,
    TimedOutCeiling,
}

struct ExecResult {
    outcome: RunOutcome,
    pane_id: Option<String>,
    closed_pane: bool,
}

fn read_result(bee_dir: &Path, job_id: &str) -> RunOutcome {
    let dir = mailbox::mailbox_dir(bee_dir, job_id);
    let entries: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned())).collect(),
        Err(e) => return RunOutcome::Malformed(format!("could not list {}: {e}", dir.display())),
    };
    let round = match mailbox::select_latest_round(&entries) {
        Ok(r) => r,
        Err(e) => return RunOutcome::Malformed(e.to_string()),
    };
    let path = mailbox::result_path(bee_dir, job_id, round);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return RunOutcome::Malformed(format!("could not read {}: {e}", path.display())),
    };
    match mailbox::parse_result_text(round, &text) {
        Ok(r) => RunOutcome::Result(r),
        Err(e) => RunOutcome::Malformed(e.to_string()),
    }
}

/// Appends the D9 dispatch.jsonl row and a wave-ledger row (the same
/// occupancy-facing ledger `bee herding record-worker` writes into) —
/// called exactly once, right after a real `agent start` succeeds. Both
/// appends are fail-open: a logging failure is reported to stderr, never
/// allowed to hide (or undo) the dispatch it is recording.
fn record_dispatch(main_root: &Path, opts: &Options, kind: &str, pane_id: &str) {
    let mut m = Map::new();
    m.insert("ts".into(), Value::String(chrono::Utc::now().to_rfc3339()));
    m.insert("source".into(), Value::String("herding-run".into()));
    m.insert("job_id".into(), Value::String(opts.job_id.clone()));
    m.insert("kind".into(), Value::String(kind.to_string()));
    m.insert("pane_id".into(), Value::String(pane_id.to_string()));
    m.insert("cwd".into(), Value::String(opts.cwd.display().to_string()));
    m.insert("task".into(), Value::String(opts.task.clone()));
    if let Err(e) =
        crate::fsutil::append_jsonl(&main_root.join(".bee").join("logs").join("dispatch.jsonl"), &Value::Object(m))
    {
        eprintln!("bee herding run: could not append the dispatch log: {e}");
    }

    let worker = WorkerRow {
        name: opts.job_id.clone(),
        pane_id: pane_id.to_string(),
        worktree: opts.cwd.display().to_string(),
        task: opts.task.clone(),
        outcome: None,
        evidence: None,
    };
    let row = WaveRow {
        wave_id: opts.job_id.clone(),
        started_at: chrono::Utc::now().to_rfc3339(),
        workers: vec![worker],
    };
    if let Err(e) = wave_ledger::append_wave(main_root, &row) {
        eprintln!("bee herding run: could not append the wave ledger row: {e}");
    }
}

/// The whole verb, generic over `Herdr` so tests drive it with a fake
/// (production's only caller, `run()` below, passes `&RealHerdr`). Branches
/// on `--continue` (D3) right at the top: a fresh spawn and a follow-up
/// round share the poll loop (`wait_for_round`) and pane lifecycle, but
/// nothing else — a fresh spawn never reuses a pane, and `--continue` never
/// splits one.
fn execute(opts: &Options, herdr: &dyn Herdr) -> ExecResult {
    if opts.is_continue {
        execute_continue(opts, herdr)
    } else {
        execute_new(opts, herdr)
    }
}

/// The `wait_for_round` a fresh spawn and `--continue` share: sleep, poll
/// the mailbox for a `result-N.json` with `round >= min_round`, and check
/// heartbeat freshness (`log.txt` mtime, `herdr agent list` status) — the
/// SAME timing rule (`decide_poll`) either path runs under. A fresh spawn
/// passes `min_round: 1` (any round satisfies it, since none exist yet); a
/// `--continue` round passes the NEXT round explicitly, so an
/// already-present prior round's result file can never be mistaken for
/// this round's completion (D3's own "not the already-present round N").
fn wait_for_round(
    bee_dir: &Path,
    job_id: &str,
    min_round: u32,
    started_at_ms: i64,
    idle_timeout_secs: u64,
    ceiling_secs: u64,
    herdr: &dyn Herdr,
) -> PollDecision {
    let log_file_path = mailbox::log_path(bee_dir, job_id);
    let mailbox_path = mailbox::mailbox_dir(bee_dir, job_id);
    let mut last_log_mtime: Option<std::time::SystemTime> = None;
    run_poll_loop(
        started_at_ms,
        idle_timeout_secs,
        ceiling_secs,
        POLL_INTERVAL,
        || {
            let result_ready = std::fs::read_dir(&mailbox_path)
                .ok()
                .map(|rd| {
                    rd.filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
                        .collect::<Vec<_>>()
                })
                .and_then(|names| mailbox::latest_result_round(&names))
                .is_some_and(|round| round >= min_round);
            let mut heartbeat_fresh = false;
            if let Ok(meta) = std::fs::metadata(&log_file_path) {
                if let Ok(modified) = meta.modified() {
                    if last_log_mtime.map_or(true, |prev| modified > prev) {
                        last_log_mtime = Some(modified);
                        heartbeat_fresh = true;
                    }
                }
            }
            if herdr.agent_status(job_id).as_deref() == Some("working") {
                heartbeat_fresh = true;
            }
            PollTick { result_ready, heartbeat_fresh }
        },
        |d| std::thread::sleep(d),
        now_ms,
    )
}

/// A fresh spawn: split a pane off the caller's own, `agent start` into it,
/// then wait for round 1.
fn execute_new(opts: &Options, herdr: &dyn Herdr) -> ExecResult {
    let bee_dir = opts.main_root.join(".bee");
    let files: Vec<String> = Vec::new();
    let spec = BriefSpec {
        job_id: &opts.job_id,
        task: &opts.task,
        worktree_root: &opts.cwd,
        files: &files,
        bee_dir: &bee_dir,
        round: 1,
    };
    let brief = mailbox::render_brief(&spec);

    let mut job_value = serde_json::json!({
        "job_id": opts.job_id,
        "task": opts.task,
        "cwd": opts.cwd.display().to_string(),
        "round": 1,
        "idle_timeout_secs": opts.idle_timeout_secs,
        "ceiling_secs": opts.ceiling_secs,
        "close_always": opts.close_always,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });
    let job_file_path = mailbox::job_path(&bee_dir, &opts.job_id);
    if let Err(e) = crate::fsutil::write_json_atomic(&job_file_path, &job_value) {
        return ExecResult {
            outcome: RunOutcome::SpawnFailed(format!("could not write {}: {e}", job_file_path.display())),
            pane_id: None,
            closed_pane: false,
        };
    }

    if opts.dry_run {
        return ExecResult { outcome: RunOutcome::DryRun(brief), pane_id: None, closed_pane: false };
    }

    let cfg_path = opts.main_root.join(".bee").join("config.json");
    let cfg: Value = std::fs::read_to_string(&cfg_path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or(Value::Null);
    let (kind, args) = match resolve_agent_command(&cfg) {
        Ok(pair) => pair,
        Err(e) => {
            return ExecResult { outcome: RunOutcome::SpawnFailed(e.to_string()), pane_id: None, closed_pane: false }
        }
    };

    let own_pane = match herdr.pane_current() {
        Ok(p) => p,
        Err(e) => return ExecResult { outcome: RunOutcome::SpawnFailed(e), pane_id: None, closed_pane: false },
    };
    let direction = herdr.pane_rect(&own_pane).map(|(w, h)| split_direction(w, h)).unwrap_or("right");
    let new_pane = match herdr.pane_split(&own_pane, direction, &opts.cwd) {
        Ok(p) => p,
        Err(e) => return ExecResult { outcome: RunOutcome::SpawnFailed(e), pane_id: None, closed_pane: false },
    };

    if let Err(e) = herdr.agent_start(&opts.job_id, &kind, &new_pane, &args) {
        // role-dispatch.md §8's own cleanup rule: a start failure closes the
        // pane THIS call just split, before reporting.
        let closed = herdr.pane_close(&new_pane).is_ok();
        if !closed {
            eprintln!("bee herding run: could not close pane {new_pane} after a failed start");
        }
        return ExecResult {
            outcome: RunOutcome::SpawnFailed(format!("agent start failed: {e}")),
            pane_id: Some(new_pane),
            closed_pane: closed,
        };
    }

    // herding-run-ready-wait D1: the started agent must REPORT ready before
    // the brief is sent — `agent start`'s success can precede real input
    // readiness, and a brief typed into the boot banner is lost.
    let ready = wait_for_agent_ready(
        opts.ready_wait_secs,
        POLL_INTERVAL,
        || herdr.agent_status(&opts.job_id),
        |d| std::thread::sleep(d),
        now_ms,
    );
    if !ready {
        return ExecResult {
            outcome: RunOutcome::SpawnFailed(format!(
                "agent never reported ready within {}s (ready-wait) — pane kept for inspection",
                opts.ready_wait_secs
            )),
            pane_id: Some(new_pane),
            closed_pane: false,
        };
    }

    // The brief travels through `agent prompt`, never `agent start` argv —
    // a multi-line brief cannot be encoded for the target shell
    // (herding-run-prompt-delivery D1). The agent IS running past this
    // point, so a prompt failure keeps the pane as forensics (the standing
    // failure rule), unlike the start failure above.
    if let Err(e) = herdr.agent_prompt(&opts.job_id, &brief) {
        return ExecResult {
            outcome: RunOutcome::SpawnFailed(format!("brief prompt failed after start: {e}")),
            pane_id: Some(new_pane),
            closed_pane: false,
        };
    }

    record_dispatch(&opts.main_root, opts, &kind, &new_pane);

    // Persist pane+agent identity into job.json (D3): the ONLY way a later
    // `--continue` can find this agent's pane again. Best-effort — a
    // failure here is reported, never allowed to hide the agent that IS
    // now running.
    if let Value::Object(ref mut m) = job_value {
        m.insert("pane_id".into(), Value::String(new_pane.clone()));
        m.insert("kind".into(), Value::String(kind.clone()));
    }
    if let Err(e) = crate::fsutil::write_json_atomic(&job_file_path, &job_value) {
        eprintln!("bee herding run: could not record pane/kind into {}: {e}", job_file_path.display());
    }

    let started_at_ms = now_ms();
    let decision =
        wait_for_round(&bee_dir, &opts.job_id, 1, started_at_ms, opts.idle_timeout_secs, opts.ceiling_secs, herdr);

    let outcome = match decision {
        PollDecision::ResultReady => read_result(&bee_dir, &opts.job_id),
        PollDecision::TimedOutIdle => RunOutcome::TimedOutIdle,
        PollDecision::TimedOutCeiling => RunOutcome::TimedOutCeiling,
        PollDecision::Continue => unreachable!("run_poll_loop only returns on a non-Continue decision"),
    };

    let valid_result = matches!(outcome, RunOutcome::Result(_));
    let close = should_close_pane(valid_result, opts.close_always);
    let closed_pane = if close {
        match herdr.pane_close(&new_pane) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("bee herding run: could not close pane {new_pane}: {e}");
                false
            }
        }
    } else {
        false
    };

    ExecResult { outcome, pane_id: Some(new_pane), closed_pane }
}

/// `--continue <job-id>` (D3): reuse the EXISTING job mailbox — read the
/// highest prior round, render round N+1's brief, `agent prompt` it to the
/// job's recorded pane, then wait for round N+1's result under the same
/// timing and pane-lifecycle rules a fresh spawn uses. Never calls
/// `agent_start` — the whole point is addressing the SAME agent again, not
/// starting a second one.
fn execute_continue(opts: &Options, herdr: &dyn Herdr) -> ExecResult {
    let bee_dir = opts.main_root.join(".bee");
    let job_id = &opts.job_id;
    let mailbox_path = mailbox::mailbox_dir(&bee_dir, job_id);
    let job_file_path = mailbox::job_path(&bee_dir, job_id);

    let refused = |refusal: ContinueRefusal| ExecResult {
        outcome: RunOutcome::ContinueRefused(refusal),
        pane_id: None,
        closed_pane: false,
    };

    // Job dir missing: no directory listing, or no readable job.json in it.
    let entries: Vec<String> = match std::fs::read_dir(&mailbox_path) {
        Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned())).collect(),
        Err(_) => return refused(ContinueRefusal::JobDirMissing { job_id: job_id.clone() }),
    };
    let job_value: Value = match std::fs::read_to_string(&job_file_path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
    {
        Some(v) => v,
        None => return refused(ContinueRefusal::JobDirMissing { job_id: job_id.clone() }),
    };

    // No prior result: nothing to continue FROM.
    let prior_round = match mailbox::latest_result_round(&entries) {
        Some(r) => r,
        None => return refused(ContinueRefusal::NoPriorResult { job_id: job_id.clone() }),
    };
    let next_round = prior_round + 1;

    let recorded_pane = job_value.get("pane_id").and_then(Value::as_str).map(str::to_string);
    let worktree_root = job_value
        .get("cwd")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| opts.cwd.clone());

    let files: Vec<String> = Vec::new();
    let spec = BriefSpec {
        job_id,
        task: &opts.task,
        worktree_root: &worktree_root,
        files: &files,
        bee_dir: &bee_dir,
        round: next_round,
    };
    let brief = mailbox::render_brief(&spec);

    if opts.dry_run {
        // Renders the round N+1 brief and sends nothing — no `Herdr` call
        // of any kind, the same contract a fresh spawn's `--dry-run` keeps.
        return ExecResult { outcome: RunOutcome::DryRun(brief), pane_id: None, closed_pane: false };
    }

    // Pane gone: no recorded pane, or `herdr pane list` no longer shows it.
    let pane_id = match &recorded_pane {
        Some(p) if herdr.pane_alive(p) => p.clone(),
        _ => {
            return refused(ContinueRefusal::PaneGone { job_id: job_id.clone(), pane_id: recorded_pane });
        }
    };

    if let Err(e) = herdr.agent_prompt(job_id, &brief) {
        return ExecResult {
            outcome: RunOutcome::SpawnFailed(format!("agent prompt failed: {e}")),
            pane_id: Some(pane_id),
            closed_pane: false,
        };
    }

    let kind = job_value.get("kind").and_then(Value::as_str).unwrap_or("unknown").to_string();
    record_dispatch(&opts.main_root, opts, &kind, &pane_id);

    // Advance job.json's round so a THIRD `--continue` finds this round as
    // its prior one — everything else in job.json (pane_id, kind, cwd…)
    // stays as spawn recorded it.
    let mut updated_job = job_value.clone();
    if let Value::Object(ref mut m) = updated_job {
        m.insert("round".into(), Value::from(next_round));
        m.insert("continued_at".into(), Value::String(chrono::Utc::now().to_rfc3339()));
    }
    if let Err(e) = crate::fsutil::write_json_atomic(&job_file_path, &updated_job) {
        eprintln!("bee herding run --continue: could not update {}: {e}", job_file_path.display());
    }

    let started_at_ms = now_ms();
    let decision = wait_for_round(
        &bee_dir,
        job_id,
        next_round,
        started_at_ms,
        opts.idle_timeout_secs,
        opts.ceiling_secs,
        herdr,
    );

    let outcome = match decision {
        PollDecision::ResultReady => read_result(&bee_dir, job_id),
        PollDecision::TimedOutIdle => RunOutcome::TimedOutIdle,
        PollDecision::TimedOutCeiling => RunOutcome::TimedOutCeiling,
        PollDecision::Continue => unreachable!("run_poll_loop only returns on a non-Continue decision"),
    };

    let valid_result = matches!(outcome, RunOutcome::Result(_));
    let close = should_close_pane(valid_result, opts.close_always);
    let closed_pane = if close {
        match herdr.pane_close(&pane_id) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("bee herding run --continue: could not close pane {pane_id}: {e}");
                false
            }
        }
    } else {
        false
    };

    ExecResult { outcome, pane_id: Some(pane_id), closed_pane }
}

fn outcome_label(o: &RunOutcome) -> &'static str {
    match o {
        RunOutcome::DryRun(_) => "dry_run",
        RunOutcome::ContinueRefused(_) => "continue_refused",
        RunOutcome::SpawnFailed(_) => "spawn_failed",
        RunOutcome::Result(r) => match r.status {
            MailboxStatus::Done => "done",
            MailboxStatus::Blocked => "blocked",
        },
        RunOutcome::Malformed(_) => "malformed_result",
        RunOutcome::TimedOutIdle => "timed_out_idle",
        RunOutcome::TimedOutCeiling => "timed_out_ceiling",
    }
}

fn exit_code_for(o: &RunOutcome) -> ExitCode {
    match o {
        RunOutcome::DryRun(_) => ExitCode::SUCCESS,
        RunOutcome::Result(r) if r.status == MailboxStatus::Done => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

fn emit_result(opts: &Options, result: &ExecResult) {
    if opts.json {
        let mut m = Map::new();
        m.insert("job_id".into(), Value::String(opts.job_id.clone()));
        m.insert("outcome".into(), Value::String(outcome_label(&result.outcome).to_string()));
        m.insert(
            "pane_id".into(),
            result.pane_id.clone().map(Value::String).unwrap_or(Value::Null),
        );
        m.insert("closed_pane".into(), Value::Bool(result.closed_pane));
        m.insert("dry_run".into(), Value::Bool(opts.dry_run));
        match &result.outcome {
            RunOutcome::Result(r) => {
                m.insert("summary".into(), Value::String(r.summary.clone()));
                m.insert(
                    "files_changed".into(),
                    Value::Array(r.files_changed.iter().cloned().map(Value::String).collect()),
                );
                m.insert("proof".into(), Value::String(r.proof.clone()));
            }
            RunOutcome::SpawnFailed(msg) | RunOutcome::Malformed(msg) => {
                m.insert("error".into(), Value::String(msg.clone()));
            }
            RunOutcome::ContinueRefused(refusal) => {
                m.insert("error".into(), Value::String(refusal.to_string()));
            }
            RunOutcome::DryRun(brief) => {
                m.insert("brief".into(), Value::String(brief.clone()));
                m.insert(
                    "job_path".into(),
                    Value::String(
                        mailbox::job_path(&opts.main_root.join(".bee"), &opts.job_id).display().to_string(),
                    ),
                );
            }
            RunOutcome::TimedOutIdle | RunOutcome::TimedOutCeiling => {}
        }
        println!("{}", Value::Object(m));
    } else {
        println!(
            "bee herding run {}: {}{}",
            opts.job_id,
            outcome_label(&result.outcome),
            if result.closed_pane { " (pane closed)" } else { "" }
        );
    }
}

pub(super) fn run(flags: &[&str]) -> ExitCode {
    let opts = match parse_options(flags) {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("bee herding run: {msg}");
            return ExitCode::FAILURE;
        }
    };
    let result = execute(&opts, &RealHerdr);
    let exit = exit_code_for(&result.outcome);
    emit_result(&opts, &result);
    exit
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    // ─── pure decisions ─────────────────────────────────────────────────

    #[test]
    fn split_direction_prefers_right_when_wider_or_square() {
        assert_eq!(split_direction(173, 50), "right");
        assert_eq!(split_direction(50, 50), "right");
        assert_eq!(split_direction(40, 100), "down");
    }

    #[test]
    fn should_close_pane_matches_the_d6_lifecycle_rule() {
        assert!(should_close_pane(true, false), "a valid result closes the pane");
        assert!(!should_close_pane(false, false), "a failure keeps the pane open");
        assert!(should_close_pane(false, true), "--close-always closes on a failure too");
        assert!(should_close_pane(true, true), "--close-always closes on a valid result too");
    }

    #[test]
    fn decide_poll_reports_result_ready_regardless_of_timers() {
        assert_eq!(decide_poll(0, 0, 0, 1, 1, true), PollDecision::ResultReady);
    }

    #[test]
    fn decide_poll_extends_on_a_fresh_heartbeat() {
        assert_eq!(decide_poll(5_000, 0, 5_000, 60, 3_600, false), PollDecision::Continue);
    }

    #[test]
    fn decide_poll_times_out_idle_when_the_heartbeat_goes_stale() {
        assert_eq!(decide_poll(61_000, 0, 0, 60, 3_600, false), PollDecision::TimedOutIdle);
    }

    #[test]
    fn decide_poll_ceiling_caps_even_with_a_fresh_heartbeat() {
        // last heartbeat one second ago (well inside a 60s idle timeout),
        // but the run has now been alive for the full 3600s ceiling — the
        // ceiling caps regardless of activity.
        assert_eq!(decide_poll(3_600_000, 0, 3_599_000, 60, 3_600, false), PollDecision::TimedOutCeiling);
    }

    #[test]
    fn run_poll_loop_returns_result_ready_as_soon_as_a_tick_reports_it() {
        let mut ticks = 0u32;
        let mut clock = 0i64;
        let decision = run_poll_loop(
            0,
            60,
            3_600,
            Duration::from_millis(0),
            || {
                ticks += 1;
                PollTick { result_ready: ticks >= 3, heartbeat_fresh: false }
            },
            |_| {},
            || {
                clock += 1_000;
                clock
            },
        );
        assert_eq!(decision, PollDecision::ResultReady);
        assert_eq!(ticks, 3);
    }

    #[test]
    fn run_poll_loop_times_out_idle_when_the_heartbeat_never_refreshes() {
        let mut clock = 0i64;
        let decision = run_poll_loop(
            0,
            5,
            3_600,
            Duration::from_millis(0),
            || PollTick { result_ready: false, heartbeat_fresh: false },
            |_| {},
            || {
                clock += 1_000;
                clock
            },
        );
        assert_eq!(decision, PollDecision::TimedOutIdle);
    }

    #[test]
    fn run_poll_loop_hits_the_ceiling_even_with_a_heartbeat_every_tick() {
        let mut clock = 0i64;
        let decision = run_poll_loop(
            0,
            3_600,
            5,
            Duration::from_millis(0),
            || PollTick { result_ready: false, heartbeat_fresh: true },
            |_| {},
            || {
                clock += 1_000;
                clock
            },
        );
        assert_eq!(decision, PollDecision::TimedOutCeiling);
    }

    // ─── the Herdr seam ─────────────────────────────────────────────────

    struct FakeHerdr {
        own_pane: &'static str,
        rect: Option<(u64, u64)>,
        split_result: Result<String, String>,
        start_result: Result<(), String>,
        prompt_result: Result<(), String>,
        status: RefCell<Option<String>>,
        closed: RefCell<Vec<String>>,
        /// Pane ids `pane_alive` answers `true` for — the FakeHerdr's
        /// stand-in for `herdr pane list`'s membership set.
        alive_panes: RefCell<Vec<String>>,
        /// Every `agent_prompt(job_id, prompt)` call, in order — the seam a
        /// `--continue` test reads to prove a prompt was sent (and
        /// `start_calls` stays empty, proving `agent_start` was NOT).
        prompt_calls: RefCell<Vec<(String, String)>>,
        start_calls: RefCell<Vec<String>>,
    }

    impl FakeHerdr {
        fn new() -> Self {
            FakeHerdr {
                own_pane: "w1:p1",
                rect: Some((100, 50)),
                split_result: Ok("w1:p2".to_string()),
                start_result: Ok(()),
                prompt_result: Ok(()),
                status: RefCell::new(Some("idle".to_string())),
                closed: RefCell::new(Vec::new()),
                alive_panes: RefCell::new(vec!["w1:p2".to_string()]),
                prompt_calls: RefCell::new(Vec::new()),
                start_calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl Herdr for FakeHerdr {
        fn pane_current(&self) -> Result<String, String> {
            Ok(self.own_pane.to_string())
        }
        fn pane_rect(&self, _pane_id: &str) -> Option<(u64, u64)> {
            self.rect
        }
        fn pane_split(&self, _pane_id: &str, _direction: &str, _cwd: &Path) -> Result<String, String> {
            self.split_result.clone()
        }
        fn agent_start(
            &self,
            job_id: &str,
            _kind: &str,
            _pane_id: &str,
            _args: &[String],
        ) -> Result<(), String> {
            self.start_calls.borrow_mut().push(job_id.to_string());
            self.start_result.clone()
        }
        fn agent_status(&self, _job_id: &str) -> Option<String> {
            self.status.borrow().clone()
        }
        fn pane_close(&self, pane_id: &str) -> Result<(), String> {
            self.closed.borrow_mut().push(pane_id.to_string());
            Ok(())
        }
        fn agent_prompt(&self, job_id: &str, prompt: &str) -> Result<(), String> {
            self.prompt_calls.borrow_mut().push((job_id.to_string(), prompt.to_string()));
            self.prompt_result.clone()
        }
        fn pane_alive(&self, pane_id: &str) -> bool {
            self.alive_panes.borrow().iter().any(|p| p == pane_id)
        }
    }

    /// Every method panics — proves a code path never touches `Herdr` at
    /// all (used for `--dry-run`: D1's "spawns no process" claim).
    struct PanicHerdr;
    impl Herdr for PanicHerdr {
        fn pane_current(&self) -> Result<String, String> {
            panic!("dry-run must never call Herdr::pane_current")
        }
        fn pane_rect(&self, _pane_id: &str) -> Option<(u64, u64)> {
            panic!("dry-run must never call Herdr::pane_rect")
        }
        fn pane_split(&self, _pane_id: &str, _direction: &str, _cwd: &Path) -> Result<String, String> {
            panic!("dry-run must never call Herdr::pane_split")
        }
        fn agent_start(
            &self,
            _job_id: &str,
            _kind: &str,
            _pane_id: &str,
            _args: &[String],
        ) -> Result<(), String> {
            panic!("dry-run must never call Herdr::agent_start")
        }
        fn agent_status(&self, _job_id: &str) -> Option<String> {
            panic!("dry-run must never call Herdr::agent_status")
        }
        fn pane_close(&self, _pane_id: &str) -> Result<(), String> {
            panic!("dry-run must never call Herdr::pane_close")
        }
        fn agent_prompt(&self, _job_id: &str, _prompt: &str) -> Result<(), String> {
            panic!("dry-run must never call Herdr::agent_prompt")
        }
        fn pane_alive(&self, _pane_id: &str) -> bool {
            panic!("dry-run must never call Herdr::pane_alive")
        }
    }

    fn test_options(main_root: &Path, dry_run: bool) -> Options {
        Options {
            task: "do the thing".to_string(),
            cwd: main_root.join("work"),
            job_id: "job-1".to_string(),
            idle_timeout_secs: 3_600,
            ceiling_secs: 3_600,
            close_always: false,
            main_root: main_root.to_path_buf(),
            json: true,
            dry_run,
            is_continue: false,
            ready_wait_secs: 60,
        }
    }

    /// Writes a job.json + result-N.json pair matching what a real spawn
    /// (`execute_new`) and a real worker would have left behind, so a
    /// `--continue` test can start from "round N already finished" without
    /// running the whole spawn path first.
    fn seed_job(main_root: &Path, job_id: &str, pane_id: &str, round: u32) {
        let bee_dir = main_root.join(".bee");
        let dir = mailbox::mailbox_dir(&bee_dir, job_id);
        std::fs::create_dir_all(&dir).unwrap();
        let job = serde_json::json!({
            "job_id": job_id,
            "task": "round 1 task",
            "cwd": main_root.join("work").display().to_string(),
            "round": round,
            "idle_timeout_secs": 3_600,
            "ceiling_secs": 3_600,
            "close_always": false,
            "created_at": "2026-01-01T00:00:00Z",
            "pane_id": pane_id,
            "kind": "claude",
        });
        std::fs::write(mailbox::job_path(&bee_dir, job_id), serde_json::to_string(&job).unwrap()).unwrap();
        std::fs::write(
            mailbox::result_path(&bee_dir, job_id, round),
            r#"{"status":"done","summary":"round done","files_changed":[],"proof":"n/a"}"#,
        )
        .unwrap();
    }

    fn continue_options(main_root: &Path, dry_run: bool) -> Options {
        Options {
            task: "round 2: keep going".to_string(),
            cwd: main_root.join("work"),
            job_id: "job-1".to_string(),
            idle_timeout_secs: 3_600,
            ceiling_secs: 3_600,
            close_always: false,
            main_root: main_root.to_path_buf(),
            json: true,
            dry_run,
            is_continue: true,
            ready_wait_secs: 60,
        }
    }

    #[test]
    fn dry_run_writes_job_json_renders_the_brief_and_touches_no_herdr() {
        let tmp = tempfile::tempdir().unwrap();
        let opts = test_options(tmp.path(), true);
        let result = execute(&opts, &PanicHerdr);
        match &result.outcome {
            RunOutcome::DryRun(brief) => {
                assert!(brief.contains("do the thing"), "brief missing the task text:\n{brief}");
                assert!(
                    brief.contains(&opts.cwd.display().to_string()),
                    "brief missing the absolute worktree root:\n{brief}"
                );
            }
            other => panic!("expected DryRun, got {other:?}"),
        }
        assert!(result.pane_id.is_none());
        assert!(!result.closed_pane);

        let job_file_path = mailbox::job_path(&tmp.path().join(".bee"), &opts.job_id);
        let job: Value = serde_json::from_str(&std::fs::read_to_string(job_file_path).unwrap()).unwrap();
        assert_eq!(job["job_id"], "job-1");
        assert_eq!(job["task"], "do the thing");
        assert_eq!(job["round"], 1);
    }

    #[test]
    fn spawn_failure_closes_the_pane_it_just_split() {
        let tmp = tempfile::tempdir().unwrap();
        let opts = test_options(tmp.path(), false);
        let mut fake = FakeHerdr::new();
        fake.start_result = Err("herdr: agent kind unrecognised".to_string());
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::SpawnFailed(_)), "got {:?}", result.outcome);
        assert_eq!(fake.closed.borrow().as_slice(), ["w1:p2"]);
        assert!(result.closed_pane);
    }

    #[test]
    fn ready_wait_accepts_an_agent_once_its_status_flips_and_only_then() {
        // herding-run-ready-wait D1, injected clock: status None for the
        // first two polls (booting — the smoke's lost-brief window), idle on
        // the third. No real sleep.
        let mut clock = 0i64;
        let statuses = std::cell::RefCell::new(vec![None, None, Some("idle".to_string())]);
        let mut polls = 0u32;
        let ready = wait_for_agent_ready(
            60,
            Duration::from_millis(500),
            || {
                polls += 1;
                let mut v = statuses.borrow_mut();
                if v.is_empty() { Some("idle".to_string()) } else { v.remove(0) }
            },
            |_| {},
            || {
                clock += 500;
                clock
            },
        );
        assert!(ready);
        assert_eq!(polls, 3, "ready only on the third status read");
    }

    #[test]
    fn ready_wait_exhaustion_is_a_typed_spawn_failure_that_keeps_the_pane() {
        let tmp = tempfile::tempdir().unwrap();
        let mut opts = test_options(tmp.path(), false);
        // 0-second ready-wait + an agent that never reports a status:
        // exhausted on the first check, before any sleep.
        opts.ready_wait_secs = 0;
        let mut fake = FakeHerdr::new();
        fake.status = RefCell::new(None);
        let result = execute(&opts, &fake);
        match &result.outcome {
            RunOutcome::SpawnFailed(msg) => assert!(msg.contains("ready-wait"), "{msg}"),
            other => panic!("expected SpawnFailed(ready-wait), got {other:?}"),
        }
        assert!(!result.closed_pane, "pane stays for inspection");
        assert!(fake.closed.borrow().is_empty());
        assert!(fake.prompt_calls.borrow().is_empty(), "no brief before readiness");
    }

    #[test]
    fn a_new_run_sends_the_brief_via_agent_prompt_and_start_argv_never_carries_it() {
        // herding-run-prompt-delivery D1: the live smoke proved a multi-line
        // brief cannot ride `agent start`'s argv (herdr
        // `invalid_agent_argument`); the brief travels via `agent prompt`.
        let tmp = tempfile::tempdir().unwrap();
        let opts = test_options(tmp.path(), false);
        let bee_dir = tmp.path().join(".bee");
        let dir = mailbox::mailbox_dir(&bee_dir, &opts.job_id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("result-1.json"),
            r#"{"status":"done","summary":"ok","files_changed":[],"proof":"n/a"}"#,
        )
        .unwrap();
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::Result(_)), "got {:?}", result.outcome);
        // start happened exactly once, addressed by job id — argv content is
        // agent_command tokens only by construction (the trait no longer
        // even accepts a prompt parameter).
        assert_eq!(fake.start_calls.borrow().as_slice(), ["job-1"]);
        // the round-1 brief arrived via agent_prompt before polling.
        let prompts = fake.prompt_calls.borrow();
        assert_eq!(prompts.len(), 1, "exactly one brief prompt for round 1");
        assert_eq!(prompts[0].0, "job-1");
        assert!(prompts[0].1.contains("round 1"), "brief names its round: {}", prompts[0].1);
        assert!(prompts[0].1.contains("result-1.json"), "brief names the result file");
    }

    #[test]
    fn a_failed_brief_prompt_keeps_the_pane_as_forensics() {
        let tmp = tempfile::tempdir().unwrap();
        let opts = test_options(tmp.path(), false);
        let mut fake = FakeHerdr::new();
        fake.prompt_result = Err("send failed".to_string());
        let result = execute(&opts, &fake);
        match &result.outcome {
            RunOutcome::SpawnFailed(msg) => {
                assert!(msg.contains("brief prompt failed"), "{msg}")
            }
            other => panic!("expected SpawnFailed, got {other:?}"),
        }
        // unlike a start failure, the agent is running: pane stays open.
        assert!(!result.closed_pane);
        assert!(fake.closed.borrow().is_empty());
    }

    #[test]
    fn a_valid_result_closes_the_pane_and_records_dispatch_and_ledger_rows() {
        let tmp = tempfile::tempdir().unwrap();
        let opts = test_options(tmp.path(), false);
        let bee_dir = tmp.path().join(".bee");
        let dir = mailbox::mailbox_dir(&bee_dir, &opts.job_id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("result-1.json"),
            r#"{"status":"done","summary":"ok","files_changed":[],"proof":"n/a"}"#,
        )
        .unwrap();
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        match &result.outcome {
            RunOutcome::Result(r) => assert_eq!(r.status, MailboxStatus::Done),
            other => panic!("expected Result(done), got {other:?}"),
        }
        assert!(result.closed_pane);
        assert_eq!(fake.closed.borrow().as_slice(), ["w1:p2"]);

        let dispatch_log = std::fs::read_to_string(bee_dir.join("logs").join("dispatch.jsonl")).unwrap();
        assert!(dispatch_log.contains("\"job_id\":\"job-1\""), "{dispatch_log}");
        assert!(dispatch_log.contains("\"source\":\"herding-run\""), "{dispatch_log}");
        let ledger = std::fs::read_to_string(bee_dir.join("wave-ledger.jsonl")).unwrap();
        assert!(ledger.contains("\"wave_id\":\"job-1\""), "{ledger}");
        assert!(ledger.contains("\"pane_id\":\"w1:p2\""), "{ledger}");
    }

    #[test]
    fn a_blocked_result_is_still_valid_and_closes_the_pane() {
        let tmp = tempfile::tempdir().unwrap();
        let opts = test_options(tmp.path(), false);
        let bee_dir = tmp.path().join(".bee");
        let dir = mailbox::mailbox_dir(&bee_dir, &opts.job_id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("result-1.json"),
            r#"{"status":"blocked","summary":"stuck","files_changed":[],"proof":"n/a"}"#,
        )
        .unwrap();
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        match &result.outcome {
            RunOutcome::Result(r) => assert_eq!(r.status, MailboxStatus::Blocked),
            other => panic!("expected Result(blocked), got {other:?}"),
        }
        assert!(result.closed_pane, "a blocked-but-well-formed result is still a valid result (D6)");
    }

    #[test]
    fn timeout_keeps_the_pane_open_unless_close_always() {
        let tmp = tempfile::tempdir().unwrap();
        let mut opts = test_options(tmp.path(), false);
        opts.idle_timeout_secs = 1; // no heartbeat ever arrives; trips fast
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::TimedOutIdle), "got {:?}", result.outcome);
        assert!(!result.closed_pane);
        assert!(fake.closed.borrow().is_empty());
    }

    #[test]
    fn close_always_closes_the_pane_even_on_a_timeout() {
        let tmp = tempfile::tempdir().unwrap();
        let mut opts = test_options(tmp.path(), false);
        opts.idle_timeout_secs = 1;
        opts.close_always = true;
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::TimedOutIdle), "got {:?}", result.outcome);
        assert!(result.closed_pane);
        assert_eq!(fake.closed.borrow().as_slice(), ["w1:p2"]);
    }

    // ─── flag parsing ───────────────────────────────────────────────────

    #[test]
    fn parse_options_requires_task_or_task_file() {
        let err = parse_options(&["--main-root", "."]).unwrap_err();
        assert!(err.contains("--task"), "{err}");
    }

    #[test]
    fn parse_options_reads_task_from_a_file() {
        let tmp = tempfile::tempdir().unwrap();
        let task_file = tmp.path().join("task.txt");
        std::fs::write(&task_file, "fix the thing\n").unwrap();
        let opts = parse_options(&["--task-file", task_file.to_str().unwrap(), "--main-root", "."]).unwrap();
        assert_eq!(opts.task, "fix the thing");
    }

    #[test]
    fn resolve_task_file_dash_reads_the_brief_from_stdin() {
        let task = resolve_task_with_stdin(None, Some("-"), || "the full brief\n".to_string()).unwrap();
        assert_eq!(task, "the full brief");
    }

    #[test]
    fn resolve_task_file_dash_ignores_an_explicit_task_flag_first() {
        // `--task` still wins over `--task-file -` when both are given —
        // the stdin read never fires (proven by a fake that panics).
        let task = resolve_task_with_stdin(Some("inline task"), Some("-"), || {
            panic!("stdin should not be read when --task is given")
        })
        .unwrap();
        assert_eq!(task, "inline task");
    }

    #[test]
    fn resolve_task_file_path_behavior_is_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let task_file = tmp.path().join("task.txt");
        std::fs::write(&task_file, "fix the thing\n").unwrap();
        let task = resolve_task_with_stdin(None, Some(task_file.to_str().unwrap()), || {
            panic!("stdin should not be read for an ordinary file path")
        })
        .unwrap();
        assert_eq!(task, "fix the thing");
    }

    #[test]
    fn resolve_task_file_dash_with_empty_stdin_refuses_like_an_empty_task() {
        let err = resolve_task_with_stdin(None, Some("-"), || String::new()).unwrap_err();
        assert_eq!(err, "--task or --task-file is required");
    }

    #[test]
    fn resolve_task_file_dash_with_whitespace_only_stdin_refuses() {
        let err = resolve_task_with_stdin(None, Some("-"), || "\n\n".to_string()).unwrap_err();
        assert_eq!(err, "--task or --task-file is required");
    }

    #[test]
    fn parse_options_applies_close_always_json_and_dry_run_flags() {
        let opts = parse_options(&[
            "--task",
            "x",
            "--main-root",
            ".",
            "--close-always",
            "--json",
            "--dry-run",
            "--idle-timeout",
            "30",
            "--ceiling",
            "60",
        ])
        .unwrap();
        assert!(opts.close_always);
        assert!(opts.json);
        assert!(opts.dry_run);
        assert_eq!(opts.idle_timeout_secs, 30);
        assert_eq!(opts.ceiling_secs, 60);
    }

    #[test]
    fn parse_options_continue_sets_is_continue_and_the_job_id() {
        let opts = parse_options(&["--task", "round 2", "--main-root", ".", "--continue", "job-9"]).unwrap();
        assert!(opts.is_continue);
        assert_eq!(opts.job_id, "job-9");
    }

    // ─── --continue (D3) ────────────────────────────────────────────────

    #[test]
    fn continue_sends_a_prompt_to_the_recorded_pane_and_never_calls_agent_start() {
        let tmp = tempfile::tempdir().unwrap();
        seed_job(tmp.path(), "job-1", "w1:p2", 1);
        let opts = continue_options(tmp.path(), false);
        let fake = FakeHerdr::new();

        // Round 2's result lands from a second thread shortly after the
        // wait starts — NOT before `execute` is called, which would let
        // the directory listing see it as round 2 already being the
        // "prior" round and ask for round 3 instead. `execute_continue`
        // sends its prompt (and writes the updated job.json) before the
        // wait loop starts, so this thread racing the write is safe: the
        // prompt-recording assertions below read state `execute` already
        // finished setting before it ever started waiting.
        let bee_dir = tmp.path().join(".bee");
        let result_path = mailbox::result_path(&bee_dir, "job-1", 2);
        let writer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            std::fs::write(
                &result_path,
                r#"{"status":"done","summary":"round 2 done","files_changed":[],"proof":"n/a"}"#,
            )
            .unwrap();
        });

        let result = execute(&opts, &fake);
        writer.join().unwrap();

        assert!(fake.start_calls.borrow().is_empty(), "--continue must never call agent_start");
        let prompts = fake.prompt_calls.borrow();
        assert_eq!(prompts.len(), 1, "expected exactly one agent_prompt call: {prompts:?}");
        assert_eq!(prompts[0].0, "job-1");
        assert!(prompts[0].1.contains("round 2: keep going"), "prompt missing the round 2 task:\n{}", prompts[0].1);
        assert!(prompts[0].1.contains("round 2"), "prompt does not name round 2:\n{}", prompts[0].1);
        assert!(prompts[0].1.contains("result-2.json"), "prompt does not name result-2.json:\n{}", prompts[0].1);

        match &result.outcome {
            RunOutcome::Result(r) => assert_eq!(r.summary, "round 2 done"),
            other => panic!("expected Result, got {other:?}"),
        }
        assert_eq!(result.pane_id.as_deref(), Some("w1:p2"));
    }

    #[test]
    fn continue_waits_on_the_incremented_round_not_the_already_present_round_1() {
        let tmp = tempfile::tempdir().unwrap();
        // Round 1's result already exists (seed_job writes it) — a poll
        // that merely checked "any result present" would return
        // immediately with round 1's stale result. This test proves it
        // instead times out waiting specifically for round 2.
        seed_job(tmp.path(), "job-1", "w1:p2", 1);
        let mut opts = continue_options(tmp.path(), false);
        opts.idle_timeout_secs = 1; // no round-2 result ever arrives; trips fast
        let fake = FakeHerdr::new();

        let result = execute(&opts, &fake);

        assert!(matches!(result.outcome, RunOutcome::TimedOutIdle), "got {:?}", result.outcome);
    }

    #[test]
    fn continue_refuses_when_the_job_dir_is_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let opts = continue_options(tmp.path(), false);
        let result = execute(&opts, &PanicHerdr);
        match &result.outcome {
            RunOutcome::ContinueRefused(ContinueRefusal::JobDirMissing { job_id }) => {
                assert_eq!(job_id, "job-1")
            }
            other => panic!("expected ContinueRefused(JobDirMissing), got {other:?}"),
        }
    }

    #[test]
    fn continue_refuses_when_there_is_no_prior_result() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        let dir = mailbox::mailbox_dir(&bee_dir, "job-1");
        std::fs::create_dir_all(&dir).unwrap();
        let job = serde_json::json!({
            "job_id": "job-1",
            "task": "round 1 task",
            "cwd": tmp.path().join("work").display().to_string(),
            "round": 1,
            "pane_id": "w1:p2",
            "kind": "claude",
        });
        std::fs::write(mailbox::job_path(&bee_dir, "job-1"), serde_json::to_string(&job).unwrap()).unwrap();
        let opts = continue_options(tmp.path(), false);
        let result = execute(&opts, &PanicHerdr);
        match &result.outcome {
            RunOutcome::ContinueRefused(ContinueRefusal::NoPriorResult { job_id }) => {
                assert_eq!(job_id, "job-1")
            }
            other => panic!("expected ContinueRefused(NoPriorResult), got {other:?}"),
        }
    }

    #[test]
    fn continue_refuses_when_the_recorded_pane_is_gone() {
        let tmp = tempfile::tempdir().unwrap();
        seed_job(tmp.path(), "job-1", "w1:p2", 1);
        let opts = continue_options(tmp.path(), false);
        let mut fake = FakeHerdr::new();
        fake.alive_panes = RefCell::new(Vec::new()); // w1:p2 no longer alive
        let result = execute(&opts, &fake);
        match &result.outcome {
            RunOutcome::ContinueRefused(ContinueRefusal::PaneGone { job_id, pane_id }) => {
                assert_eq!(job_id, "job-1");
                assert_eq!(pane_id.as_deref(), Some("w1:p2"));
            }
            other => panic!("expected ContinueRefused(PaneGone), got {other:?}"),
        }
        assert!(fake.prompt_calls.borrow().is_empty(), "a gone pane must never receive a prompt");
    }

    #[test]
    fn continue_with_dry_run_renders_the_round_n_plus_one_brief_and_sends_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        seed_job(tmp.path(), "job-1", "w1:p2", 1);
        let opts = continue_options(tmp.path(), true);
        // PanicHerdr: proves --continue --dry-run touches no Herdr method,
        // including pane_alive — the same "spawns no process" contract a
        // fresh --dry-run keeps.
        let result = execute(&opts, &PanicHerdr);
        match &result.outcome {
            RunOutcome::DryRun(brief) => {
                assert!(brief.contains("round 2"), "brief does not name round 2:\n{brief}");
                assert!(brief.contains("result-2.json"), "brief does not name result-2.json:\n{brief}");
                assert!(brief.contains("round 2: keep going"), "brief missing the round 2 task:\n{brief}");
            }
            other => panic!("expected DryRun, got {other:?}"),
        }
        assert!(result.pane_id.is_none());
        assert!(!result.closed_pane);
    }
}
