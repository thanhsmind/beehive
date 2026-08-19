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
}

fn absolute_path(p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().map(|cwd| cwd.join(p)).unwrap_or_else(|_| p.to_path_buf())
    }
}

fn resolve_task(task: Option<&str>, task_file: Option<&str>) -> Result<String, String> {
    if let Some(t) = task {
        if !t.is_empty() {
            return Ok(t.to_string());
        }
    }
    if let Some(f) = task_file {
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
    let job_id = job_id
        .map(str::to_string)
        .unwrap_or_else(|| format!("job-{}", chrono::Utc::now().timestamp_millis()));

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
    /// 60000 -- <args…> <prompt>` — the split pane IS the agent's pane
    /// (spawn-proof.md), never a second one.
    fn agent_start(
        &self,
        job_id: &str,
        kind: &str,
        pane_id: &str,
        args: &[String],
        prompt: &str,
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
        prompt: &str,
    ) -> Result<(), String> {
        let mut argv: Vec<&str> =
            vec!["agent", "start", job_id, "--kind", kind, "--pane", pane_id, "--timeout", "60000", "--"];
        for a in args {
            argv.push(a);
        }
        argv.push(prompt);
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

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

// ═══════════════════════════════════════════════════════════════════════════
// the run itself
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
enum RunOutcome {
    /// `--dry-run`: the rendered brief, for inspection — nothing spawned.
    DryRun(String),
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
/// (production's only caller, `run()` below, passes `&RealHerdr`).
fn execute(opts: &Options, herdr: &dyn Herdr) -> ExecResult {
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

    let job_value = serde_json::json!({
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

    if let Err(e) = herdr.agent_start(&opts.job_id, &kind, &new_pane, &args, &brief) {
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

    record_dispatch(&opts.main_root, opts, &kind, &new_pane);

    let started_at_ms = now_ms();
    let job_id = opts.job_id.clone();
    let log_file_path = mailbox::log_path(&bee_dir, &job_id);
    let mailbox_path = mailbox::mailbox_dir(&bee_dir, &job_id);
    let mut last_log_mtime: Option<std::time::SystemTime> = None;

    let decision = run_poll_loop(
        started_at_ms,
        opts.idle_timeout_secs,
        opts.ceiling_secs,
        POLL_INTERVAL,
        || {
            let result_ready = std::fs::read_dir(&mailbox_path)
                .ok()
                .map(|rd| {
                    rd.filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
                        .collect::<Vec<_>>()
                })
                .map(|names| mailbox::latest_result_round(&names).is_some())
                .unwrap_or(false);
            let mut heartbeat_fresh = false;
            if let Ok(meta) = std::fs::metadata(&log_file_path) {
                if let Ok(modified) = meta.modified() {
                    if last_log_mtime.map_or(true, |prev| modified > prev) {
                        last_log_mtime = Some(modified);
                        heartbeat_fresh = true;
                    }
                }
            }
            if herdr.agent_status(&job_id).as_deref() == Some("working") {
                heartbeat_fresh = true;
            }
            PollTick { result_ready, heartbeat_fresh }
        },
        |d| std::thread::sleep(d),
        now_ms,
    );

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

fn outcome_label(o: &RunOutcome) -> &'static str {
    match o {
        RunOutcome::DryRun(_) => "dry_run",
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
        status: RefCell<Option<String>>,
        closed: RefCell<Vec<String>>,
    }

    impl FakeHerdr {
        fn new() -> Self {
            FakeHerdr {
                own_pane: "w1:p1",
                rect: Some((100, 50)),
                split_result: Ok("w1:p2".to_string()),
                start_result: Ok(()),
                status: RefCell::new(None),
                closed: RefCell::new(Vec::new()),
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
            _job_id: &str,
            _kind: &str,
            _pane_id: &str,
            _args: &[String],
            _prompt: &str,
        ) -> Result<(), String> {
            self.start_result.clone()
        }
        fn agent_status(&self, _job_id: &str) -> Option<String> {
            self.status.borrow().clone()
        }
        fn pane_close(&self, pane_id: &str) -> Result<(), String> {
            self.closed.borrow_mut().push(pane_id.to_string());
            Ok(())
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
            _prompt: &str,
        ) -> Result<(), String> {
            panic!("dry-run must never call Herdr::agent_start")
        }
        fn agent_status(&self, _job_id: &str) -> Option<String> {
            panic!("dry-run must never call Herdr::agent_status")
        }
        fn pane_close(&self, _pane_id: &str) -> Result<(), String> {
            panic!("dry-run must never call Herdr::pane_close")
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
}
