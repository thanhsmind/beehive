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
//   2. Split a pane off the ROOMIEST pane in the caller's tab (hps-12:
//      `herdr pane current --current` locates the caller, `herdr pane
//      layout` reads every pane's rect, then `herdr pane split … --cwd
//      <worktree>` targets the largest one — never always the caller's own,
//      which under fan-out just halves the same pane repeatedly). The width
//      guard (hps-13) checks the RESULTING CHILD, not the parent — a parent
//      only clears it when its split-off half would still be wide enough
//      for a worker to submit into. When no pane in the tab clears it, bee
//      asks herdr for a fresh tab and hands the worker its root pane
//      directly, unsplit, at full width — a refusal is the last resort, only
//      when that tab-create attempt itself fails. The split-then-start order
//      is the only one `spawn-proof.md` records herdr 0.8.0 accepting.
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
// nothing in `PaneTransport` is ever called on that path (proven below by handing
// dry-run execution a fake that panics if any of its methods fire).
//
// STRUCTURAL SPLIT, same shape `control_loop.rs` and `wave.rs` already use:
// every herdr-shaped operation lives behind the `PaneTransport` trait so a test
// drives the spawn sequence and the pane-lifecycle decision with a fake —
// no real `herdr` on PATH anywhere in this crate's test suite (D7's seam,
// "FakeBackend pattern" per the cell). The poll loop's own decision logic
// (`decide_poll`) and its driving loop (`run_poll_loop`) are pure/injected
// too, so the three timing rules — fresh heartbeat extends, a stale one
// times out, the ceiling caps regardless of activity — run as fast unit
// tests with no real sleep.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

use serde_json::{Map, Value};

use super::mailbox::{self, BriefSpec, ExpertiseEntry, MailboxResult, MailboxStatus};
use super::split_lock;
use super::tmux::{RealTmux, TmuxSettings};
use super::wave::{resolve_agent_command, WorkspaceTrust};
use super::TransportKind;
use super::wave_ledger::{self, WaveRow, WorkerRow};

const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 900; // 15 minutes of no heartbeat
const DEFAULT_CEILING_SECS: u64 = 21_600; // 6 hours, the busy-loop backstop
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Case-insensitive substrings that signal a usage limit pause in a worker pane.
/// Extensible per agent kind in future iterations (D1).
const LIMIT_PATTERNS: &[&str] = &["hit your session limit", "usage limit"];

fn find_limit_match(pane_text: &str) -> Option<String> {
    for line in pane_text.lines() {
        let lower = line.to_lowercase();
        for pattern in LIMIT_PATTERNS {
            if lower.contains(&pattern.to_lowercase()) {
                return Some(line.trim().to_string());
            }
        }
    }
    None
}

/// Case-insensitive substrings for `find_prompt_diagnosis` — DIAGNOSIS-ONLY
/// (herding-prompt-stall D3/D5, cell hps-7): this table never decides
/// whether a wait keeps going, only what a wait that has ALREADY given up
/// says. It is checked nowhere near `decide_poll`; a false positive here
/// costs a slightly wrong sentence and nothing else, so it stays entirely
/// separate from `LIMIT_PATTERNS` (a genuine wait-changing signal) and from
/// herdr's own `blocked` classification (D3, checked live against herdr,
/// never guessed from raw pane text).
///
/// Tokens are kept SHORT on purpose. The live acceptance probe (D5) put
/// three concurrent `agy` runs into a genuinely untrusted workspace; all
/// three sat at Antigravity's "Do you trust this folder?" dialog while
/// `herdr agent list` reported the agent as `idle`, never `blocked` — the
/// stall bee had no name for. The SAME probe found that a worker pane split
/// three deep off the same parent renders at roughly EIGHT COLUMNS wide, and
/// `herdr pane read` returns that dialog CLIPPED to one short fragment per
/// line (`Do you t`, `Yes, I`, `No, ex`). A long needle like
/// "do you trust this folder" cannot match a capture that narrow — the tail
/// of every long line is exactly what clipping throws away. A short
/// confirmation cue that lives on ITS OWN line (a `(y/n)` hint, an arrow-key
/// nav footer, a selection caret) needs no clipping margin at all, so it
/// still lands inside a narrow capture even when the question text above it
/// does not.
const PROMPT_DIAGNOSIS_PATTERNS: &[&str] = &["y/n", "↑/↓", "❯"];

/// Scans `pane_text` for `PROMPT_DIAGNOSIS_PATTERNS` (case-insensitive,
/// mirroring `find_limit_match`'s own scan) or a line ending in `?` — both
/// common shapes for an unanswered interactive prompt. Returns the first
/// matching line, trimmed, or `None` on no match. Pure and diagnosis-only:
/// this function decides nothing about waiting, only what a give-up message
/// SAYS once a wait has already ended.
fn find_prompt_diagnosis(pane_text: &str) -> Option<String> {
    for line in pane_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.ends_with('?') {
            return Some(trimmed.to_string());
        }
        let lower = trimmed.to_lowercase();
        for pattern in PROMPT_DIAGNOSIS_PATTERNS {
            if lower.contains(&pattern.to_lowercase()) {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

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
    /// `--agent <name>` (herd-registry D2): resolve the spawn command
    /// through the `herding.agents` registry by name instead of the
    /// default `herding.agent_command` split. `execute_continue` never
    /// reads this field — the pane already exists, so `--continue`
    /// ignores `--agent` by construction.
    agent: Option<String>,
    /// `--expertise <entries>` (worker-brief-expertise D1): dispatcher-picked
    /// knowledge and skill reference files with purpose and read-to guidance.
    expertise: Vec<ExpertiseEntry>,
    /// True when `--expertise` was explicitly passed on the command line.
    has_explicit_expertise: bool,
    /// `--nickname <name>` (herding-prompt-stall D4): the worker-facing
    /// identity recorded in the round's ack file. Falls back to `job_id`
    /// when the caller has no separate reservation identity for this run.
    nickname: String,
    /// `--cell-id <id>` (D4): the cell id this round belongs to, when the
    /// caller has one — carried into the ack schema the brief shows, never
    /// invented when absent.
    cell_id: Option<String>,
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

pub(crate) fn parse_expertise(raw: &str) -> Result<Vec<ExpertiseEntry>, String> {
    let mut entries = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(" :: ").collect();
        if parts.len() != 3 || parts.iter().any(|s| s.trim().is_empty()) {
            return Err(format!(
                "malformed --expertise line (want '<path> :: <purpose> :: <read-to>'): {line}"
            ));
        }
        entries.push(ExpertiseEntry {
            path: parts[0].trim().to_string(),
            purpose: parts[1].trim().to_string(),
            read_to: parts[2].trim().to_string(),
        });
    }
    Ok(entries)
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
    let mut agent: Option<&str> = None;
    let mut expertise_raw: Option<&str> = None;
    let mut nickname: Option<&str> = None;
    let mut cell_id: Option<&str> = None;
    let mut i = 0usize;
    while i < flags.len() {
        match flags[i] {
            "--agent" => {
                agent = flags.get(i + 1).copied();
                i += 2;
            }
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
            "--expertise" => {
                expertise_raw = flags.get(i + 1).copied();
                i += 2;
            }
            "--nickname" => {
                nickname = flags.get(i + 1).copied();
                i += 2;
            }
            "--cell-id" => {
                cell_id = flags.get(i + 1).copied();
                i += 2;
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
    let (expertise, has_explicit_expertise) = match expertise_raw {
        Some(raw) => (parse_expertise(raw)?, true),
        None => (Vec::new(), false),
    };

    let nickname = nickname.map(str::to_string).unwrap_or_else(|| job_id.clone());

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
        agent: agent.map(str::to_string),
        expertise,
        has_explicit_expertise,
        nickname,
        cell_id: cell_id.map(str::to_string),
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// the PaneTransport seam — every herdr-shaped operation, real or faked
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Liveness {
    Alive { pid: u32 },
    Absent,
    Unknown,
}

/// One entry of `herdr pane layout`'s `panes` array — an id and its
/// character-cell rect. hps-12: the split-parent choice is a pure function
/// over a `Vec` of these, never over a live herdr call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PaneGeom {
    pub(crate) pane_id: String,
    pub(crate) width: u64,
    pub(crate) height: u64,
}

/// Every herdr operation `bee herding run` needs, isolated behind a trait so
/// tests inject a fake instead of a real `herdr` on PATH (D7's seam, no
/// process anywhere in this crate's test suite). `RealHerdr` below is the
/// only production implementer.
///
/// The split lock (`split_lock::acquire`) sits OUTSIDE this trait, so D2's
/// spawn serialization is transport-neutral and every implementer inherits it.
pub(crate) trait PaneTransport {
    /// `herdr pane current --current` — the caller's OWN pane id, the pane
    /// this verb splits from.
    fn pane_current(&self) -> Result<String, String>;
    /// `herdr pane layout --pane <id>` — geometry for EVERY pane in the
    /// caller's OWN tab, not just `pane_id`: the split-parent choice, the
    /// direction and the ratio (herding-split-serialize D2) all read it
    /// from the same list. `None` on any trouble (herdr missing,
    /// unparseable body) — the caller falls back to its own pane rather
    /// than failing the whole run over a geometry read.
    fn pane_layout(&self, pane_id: &str) -> Option<Vec<PaneGeom>>;
    /// `herdr pane split <id> --direction <dir> --ratio <r> --cwd <cwd>
    /// --no-focus` — returns the newly split pane's id. `ratio` is the
    /// share the PARENT KEEPS (measured live, `first_split_geometry`), so
    /// D2's one-third worker column travels as a ratio ABOVE 0.5.
    fn pane_split(&self, pane_id: &str, direction: &str, ratio: f64, cwd: &Path) -> Result<String, String>;
    /// `herdr tab create --workspace <ws> --cwd <cwd> --label <label>`
    /// (hps-13): the new-tab fallback for a caller's tab with no pane roomy
    /// enough to split into a usable child — the worker gets a FRESH tab's
    /// root pane directly, unsplit, so it starts at full width instead of a
    /// sliver. Returns the new root pane's id. Never passes `--focus`
    /// (mirrors `pane_split`'s own `--no-focus`): a worker must never steal
    /// the human's focus.
    fn tab_create(&self, workspace: &str, cwd: &Path, label: &str) -> Result<String, String>;
    /// `herdr pane run <pane_id> <command>` (D4) — runs ONE shell line in
    /// the pane and returns; used ONLY to send the `export K='v' …` line a
    /// per-agent-env registry entry carries, AFTER the pane split and
    /// BEFORE `agent_start` (never after — the agent's own process must
    /// inherit the exported vars from its spawning shell). Unlike
    /// `agent_start`'s `agent_pane_busy` retry, this call carries no
    /// special busy-shell tolerance: `pane run` types text into whatever
    /// shell is there, it does not gate on "is this shell available for an
    /// agent" the way `agent start` does, so there is no reported busy
    /// error shape to retry on.
    fn pane_run(&self, pane_id: &str, command: &str) -> Result<(), String>;
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
    /// `herdr agent prompt <job_id> <prompt> --wait --until <state>
    /// --timeout <ms>` (D3 `--continue`; herding-prompt-stall D1) — sends
    /// the round N+1 brief (or the initial pointer) to an ALREADY-RUNNING
    /// agent, never `agent start` (that would spawn a second agent instead
    /// of continuing this one). `--wait` is herdr's own atomic
    /// submit-and-observe: a prompt sent from a non-`working` state that
    /// produces no observed lifecycle change within `timeout_ms` comes back
    /// `agent_prompt_stalled` (`is_agent_prompt_stalled`) instead of herdr
    /// waiting indefinitely. A stall no longer ends the run outright (D6,
    /// hps-14): `deliver_pointer` folds it into the same bounded resend the
    /// ready-with-no-ack path already uses, rather than treating a booting
    /// TUI's first miss as final.
    fn agent_prompt(&self, job_id: &str, prompt: &str, until: &str, timeout_ms: u64) -> Result<(), String>;
    /// `herdr agent wait <job_id> --until idle --until done --timeout <ms>`
    /// (herding-prompt-stall D2) — herdr's own settle-aware wait: blocks
    /// herdr-side up to `timeout_ms` for the agent to settle into `idle`,
    /// `done`, or `blocked`, then returns whatever it observed. `None` on
    /// any trouble (herdr missing, an unparseable body, no settle inside
    /// the window) — same fail-safe shape as `agent_status`: an
    /// unverifiable status never counts as a ready or heartbeat signal.
    fn agent_wait(&self, job_id: &str, timeout_ms: u64) -> Option<String>;
    /// `herdr pane list`, membership-tested against `pane_id` — the D3
    /// `--continue` "is the pane gone" check. Unlike `pane_layout` (which
    /// fails open to the caller's own pane on ANY trouble), this fails
    /// CLOSED: herdr missing, a non-zero exit, or an unparseable body all
    /// read as "not alive" — `--continue` refuses rather than prompting a
    /// pane it cannot confirm still exists.
    fn pane_alive(&self, pane_id: &str) -> bool;
    /// `herdr pane read <pane_id>` — raw stdout of the pane capture, or
    /// error if herdr failed.
    fn pane_read(&self, pane_id: &str) -> Result<String, String>;
    /// `herdr pane process-info --pane <id>` (D1/D2) — liveness of the agent
    /// process inside the pane. Fails OPEN to `Unknown` on any trouble.
    fn process_info(&self, pane_id: &str) -> Liveness;
    /// The transport's own name, as `herding.transport` spells it
    /// (tmux-herding-transport D1) — carried into `--dry-run`'s JSON so a
    /// caller can see WHICH multiplexer the run would have reached for
    /// without spawning anything. Defaults to `"herdr"`, so every
    /// herdr-shaped implementer (including the test fakes) inherits the
    /// pre-tmux answer and only `RealTmux` overrides it.
    fn name(&self) -> &'static str {
        "herdr"
    }
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
        parse_herdr_body(&args.join(" "), &out.stdout)
    }
}

/// The JSON-decode half of `RealHerdr::call`, split out so a test can drive
/// it with a captured reply body and no spawned process.
fn parse_herdr_body(args_desc: &str, stdout: &[u8]) -> Result<Value, String> {
    serde_json::from_slice(stdout).map_err(|_| {
        format!("herdr {args_desc} returned a body that was not valid JSON: {}", String::from_utf8_lossy(stdout))
    })
}

/// `agent_wait`'s extraction, pure: herdr's `agent wait` reply nests the
/// status one level deeper than `agent_status`'s `agent list` reply —
/// `result.agent.agent_status`, not `result.agent_status` — captured live
/// from `herdr agent wait <job> --until idle --until done --timeout <ms>`.
/// Split out so a test can feed the captured reply straight through the
/// same extraction the impl uses.
fn extract_agent_wait_status(v: &Value) -> Option<String> {
    v.get("result")?.get("agent")?.get("agent_status").and_then(Value::as_str).map(str::to_string)
}

/// `pane_layout`'s extraction, pure: herdr's `pane layout` reply nests the
/// full-tab pane array under `result.layout.panes` — captured live from
/// `herdr pane layout --pane w4:p4` against a clean tab: `{"result":
/// {"layout":{"tab_id":"w4:t4","panes":[{"pane_id":"w4:p4","rect":
/// {"height":43,"width":120,"x":36,"y":1}}],"splits":[]}}}`. With siblings
/// present, `panes` holds one entry per pane in the tab. A malformed entry
/// is dropped rather than failing the whole parse; a missing `panes` array
/// (or an unreadable body) is `None`, same fail-open shape as
/// `agent_status`. Split out so a test can feed the captured reply straight
/// through the same extraction the impl uses.
fn extract_pane_layout(v: &Value) -> Option<Vec<PaneGeom>> {
    let panes = v.get("result")?.get("layout")?.get("panes")?.as_array()?;
    Some(
        panes
            .iter()
            .filter_map(|p| {
                let pane_id = p.get("pane_id")?.as_str()?.to_string();
                let rect = p.get("rect")?;
                let width = rect.get("width")?.as_u64()?;
                let height = rect.get("height")?.as_u64()?;
                Some(PaneGeom { pane_id, width, height })
            })
            .collect(),
    )
}

/// `tab_create`'s extraction, pure: herdr's `tab create` reply nests the new
/// root pane under `result.root_pane.pane_id` — captured live from `herdr
/// tab create --workspace w4 --cwd <path> --label bee-probe`, trimmed:
/// `{"id":"cli:tab:create","result":{"root_pane":{"pane_id":"w4:p31","cwd":
/// "<path>","tab_id":"w4:tE","workspace_id":"w4"},"tab":{...}}}`. Split out
/// so a test can feed the captured reply straight through the same
/// extraction the impl uses.
fn extract_tab_create_root_pane(v: &Value) -> Option<String> {
    v.get("result")?.get("root_pane")?.get("pane_id")?.as_str().map(str::to_string)
}

impl PaneTransport for RealHerdr {
    fn pane_current(&self) -> Result<String, String> {
        let v = self.call(&["pane", "current", "--current"])?;
        v.get("result")
            .and_then(|r| r.get("pane"))
            .and_then(|p| p.get("pane_id"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| "herdr pane current --current: missing result.pane.pane_id".to_string())
    }

    fn pane_layout(&self, pane_id: &str) -> Option<Vec<PaneGeom>> {
        let v = self.call(&["pane", "layout", "--pane", pane_id]).ok()?;
        extract_pane_layout(&v)
    }

    fn pane_split(&self, pane_id: &str, direction: &str, ratio: f64, cwd: &Path) -> Result<String, String> {
        let cwd_str = cwd.display().to_string();
        let ratio_str = ratio.to_string();
        let v = self.call(&[
            "pane", "split", pane_id, "--direction", direction, "--ratio", &ratio_str, "--cwd", &cwd_str,
            "--no-focus",
        ])?;
        v.get("result")
            .and_then(|r| r.get("pane"))
            .and_then(|p| p.get("pane_id"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| "herdr pane split: missing result.pane.pane_id".to_string())
    }

    fn tab_create(&self, workspace: &str, cwd: &Path, label: &str) -> Result<String, String> {
        let cwd_str = cwd.display().to_string();
        let v = self.call(&["tab", "create", "--workspace", workspace, "--cwd", &cwd_str, "--label", label])?;
        extract_tab_create_root_pane(&v)
            .ok_or_else(|| "herdr tab create: missing result.root_pane.pane_id".to_string())
    }

    fn pane_run(&self, pane_id: &str, command: &str) -> Result<(), String> {
        // `pane run` emits plain (often empty) stdout, never a JSON
        // envelope — routing it through `call()` made every env export
        // "fail" on a successful run (live: job hlp-1-r1, pane w4:p13).
        // Success is the exit status, exactly like the raw `pane read`
        // capture style (herding-receipt-source D1).
        let out = Command::new("herdr")
            .args(["pane", "run", pane_id, command])
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("herdr pane run {pane_id}: {e}"))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(format!("herdr pane run {pane_id} exited {}: {}", out.status, stderr.trim()));
        }
        Ok(())
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

    fn agent_prompt(&self, job_id: &str, prompt: &str, until: &str, timeout_ms: u64) -> Result<(), String> {
        let timeout = timeout_ms.to_string();
        self.call(&["agent", "prompt", job_id, prompt, "--wait", "--until", until, "--timeout", &timeout])
            .map(|_| ())
    }

    fn agent_wait(&self, job_id: &str, timeout_ms: u64) -> Option<String> {
        let timeout = timeout_ms.to_string();
        let v = self
            .call(&["agent", "wait", job_id, "--until", "idle", "--until", "done", "--timeout", &timeout])
            .ok()?;
        extract_agent_wait_status(&v)
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

    fn pane_read(&self, pane_id: &str) -> Result<String, String> {
        let out = Command::new("herdr")
            .args(["pane", "read", pane_id])
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("herdr pane read {pane_id}: {e}"))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(format!("herdr pane read {pane_id} exited {}: {}", out.status, stderr.trim()));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    fn process_info(&self, pane_id: &str) -> Liveness {
        // D2 fail-open: any error, non-zero exit, error envelope, or unparseable body returns Unknown.
        let Ok(v) = self.call(&["pane", "process-info", "--pane", pane_id]) else {
            return Liveness::Unknown;
        };
        parse_process_info(&v)
    }
}

fn parse_process_info(v: &Value) -> Liveness {
    if v.get("error").is_some_and(|e| !e.is_null()) {
        return Liveness::Unknown;
    }
    let Some(result) = v.get("result") else {
        return Liveness::Unknown;
    };
    let info = result.get("process_info").unwrap_or(result);
    let shell_pid = info.get("shell_pid").and_then(Value::as_u64).map(|p| p as u32);
    let Some(fg_array) = info.get("foreground_processes").and_then(Value::as_array) else {
        return Liveness::Unknown;
    };
    // AGENT-PRESENT: at least one foreground entry whose pid != shell_pid (never a name match)
    let active_pid = fg_array.iter().find_map(|p| {
        let pid = p.get("pid").and_then(Value::as_u64).map(|p| p as u32)?;
        if shell_pid.map_or(true, |sp| pid != sp) {
            Some(pid)
        } else {
            None
        }
    });
    if let Some(pid) = active_pid {
        Liveness::Alive { pid }
    } else {
        Liveness::Absent
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// pure decisions — split direction, poll timing, pane lifecycle
// ═══════════════════════════════════════════════════════════════════════════

/// herding-split-serialize D2's split rule: the direction follows WHICH
/// pane was chosen as the parent, not how many panes the tab holds.
/// Splitting the caller's OWN pane goes `right` — that one split creates
/// the worker column on the main pane's right. Splitting any other pane
/// goes `down` — the worker column grows, stacking each new worker under
/// the previous one, and the main pane is never touched again.
///
/// D2 supersedes herding-pane-slides D1, whose rule answered from the
/// tab's pane count alone. That rule kept the main pane in the candidate
/// set, so five spawns on a 173x50 tab cut the human's own pane from 50
/// rows to 13 while every worker kept 25. The human needs their pane
/// readable at all times; a worker only needs enough width to accept a
/// submission.
///
/// A `down` split still leaves width UNTOUCHED, so width narrows at most
/// once — for the first worker only (`MIN_PANE_WIDTH`'s evidence).
///
/// Pure: takes one already-made comparison, touches no process.
fn split_direction(parent_is_own_pane: bool) -> &'static str {
    if parent_is_own_pane {
        "right"
    } else {
        "down"
    }
}

/// Below this width a submitted prompt never lands, not merely renders
/// oddly: herdr's own `agent prompt --wait` returns `agent_prompt_stalled`
/// before the agent ever processes it. hps-13: this guards the pane the
/// WORKER actually gets — the resulting CHILD of a split, never the parent
/// being split (hps-12 originally checked the parent, the wrong side of a
/// narrowing split, where the child is only a fraction of the parent's
/// width — `first_split_geometry` computes which fraction).
/// Evidence, not taste — live 2026-08-21, tab root 120x43, three concurrent
/// `execute_new` spawns: the first split produced a 60-column child that
/// carried a full `agy` round to a written ack and result file, end to end;
/// the other two, splitting an already-halved 60-wide sibling, produced
/// 30-column children and BOTH died mid-submission with herdr's
/// `agent_prompt_stalled` ("agent prompt produced no observed state change
/// within 5000 ms; status is idle"). 60 is the last width this evidence
/// proves works; 30 is the first it proves stalls; the boundary between
/// them is UNMEASURED, so the minimum takes the known-good value rather
/// than a guess in that gap.
const MIN_PANE_WIDTH: u64 = 60;

/// hps-12/hps-13: `None` when `resulting_child_width` is workable,
/// `Some(message)` naming the parent pane, the width its split-off child
/// would land at, the minimum, and the remedy when it is not. Pure —
/// called BEFORE `pane_split`, so a refused split creates no pane and
/// starts no agent (the caller still has a fresh-tab fallback to try
/// first, hps-13 — this alone is never the final word).
fn narrow_pane_refusal(pane_id: &str, resulting_child_width: u64) -> Option<String> {
    if resulting_child_width >= MIN_PANE_WIDTH {
        return None;
    }
    Some(format!(
        "splitting pane {pane_id} would leave the new child pane {resulting_child_width} columns wide, below \
         the {MIN_PANE_WIDTH}-column minimum a worker needs to render its own prompts and accept submissions"
    ))
}

/// herding-split-serialize D2's first-split geometry, pure: for the ONE
/// `right` split that carves the worker column out of the caller's own
/// pane, the columns the child actually lands at, plus the `--ratio` herdr
/// needs to produce them.
///
/// Columns first, ratio second — the width is the thing D2 constrains and
/// the thing `narrow_pane_refusal` measures; the ratio is only the wire
/// encoding. The child takes one third of the parent, floored at
/// `MIN_PANE_WIDTH` (a worker below it cannot accept a submission at all)
/// and capped at half (the main pane stays strictly the larger share
/// whenever the parent is wide enough for both). Floor before cap: on a
/// parent too narrow for a 60-column child the cap wins, the child lands
/// under the minimum, and `narrow_pane_refusal` refuses it into the
/// `tab_create` fresh-tab path (hps-13) instead of handing a worker a
/// sliver.
///
/// herdr's `--ratio` is the share the PARENT KEEPS, not the share the
/// child gets — measured live 2026-08-21 against herdr 0.8.0 on a fresh
/// 173x50 tab: `pane split --direction right --ratio 0.25` left the parent
/// 43 columns and handed the child 130. So the ratio is
/// `(parent - child) / parent`; a re-split of that 130-column pane with
/// the ratio this function computes for a 60-column child
/// (`70/130 = 0.5384615…`) landed the child at exactly 60. `0.5` was
/// symmetric and could never tell the two readings apart, which is why
/// this had to be measured rather than assumed.
fn first_split_geometry(parent_width: u64) -> (u64, f64) {
    let child = (parent_width / 3).max(MIN_PANE_WIDTH).min(parent_width / 2);
    if parent_width == 0 {
        // Never observed (herdr always reports a real rect); an even split
        // is the harmless answer, and the 0-column child refuses anyway.
        return (child, DOWN_SPLIT_RATIO);
    }
    (child, (parent_width - child) as f64 / parent_width as f64)
}

/// A `down` split divides the parent's HEIGHT and leaves its width alone,
/// so D2 puts no constraint on it: workers stack evenly inside the column
/// they share, exactly as before.
const DOWN_SPLIT_RATIO: f64 = 0.5;

/// The resolved split parent: which pane to split, which direction
/// (herding-split-serialize D2), the `--ratio` that split takes, and
/// whether ITS RESULTING CHILD is too narrow to work (hps-13: the guard
/// moved to the child side of the split).
struct SplitParent {
    pane_id: String,
    direction: &'static str,
    ratio: f64,
    refusal: Option<String>,
}

/// herding-split-serialize D2: the caller's pane is the MAIN pane and stays
/// the largest. It is split EXACTLY ONCE — the first spawn takes a column on
/// its right — and never again. So the parent is the roomiest pane EXCLUDING
/// `own_pane`; only when the tab holds nothing but the caller's own pane is
/// that pane the parent, and that one split is what creates the worker
/// column. Every later spawn therefore lands on a worker pane and splits
/// `down` inside the column (`split_direction`).
///
/// D2 supersedes hps-12's "roomiest pane overall". That rule kept the main
/// pane in the candidate set, so it kept winning on area and kept being
/// split: five spawns on a 173x50 tab left the human's own pane 13 rows tall
/// while every worker held 25.
///
/// hps-13 still holds, now over D2's ratio: the width guard checks the
/// RESULTING CHILD's width, never the chosen parent's own. A `right` split
/// hands the child `first_split_geometry`'s columns; a `down` split leaves
/// width unchanged, so the parent's own width is already the child's.
///
/// Pure, over an already-parsed pane list: `panes` is `None` when the
/// layout could not be read at all, and the parent choice, the direction
/// and the width refusal all fail OPEN to `own_pane`/`right`/no-refusal in
/// that case — the same fail-open habit the geometry read always had. An
/// empty (or candidate-less) list falls back the same way, carrying the
/// old unconditional `0.5`, since a fallback with no readable rect has no
/// width to compute a share from. Ties over area break toward the LAST
/// entry (`Iterator::max_by_key`'s own rule) — an arbitrary but
/// deterministic choice, since herdr's own list order carries no other
/// meaning here.
fn resolve_split_parent(panes: Option<&[PaneGeom]>, own_pane: &str) -> SplitParent {
    let chosen = panes.and_then(|list| {
        // D2: the caller's own pane is a candidate ONLY while it is alone
        // in the tab — that is the one split it ever takes.
        let mut candidates: Vec<&PaneGeom> = list.iter().filter(|p| p.pane_id != own_pane).collect();
        if candidates.is_empty() {
            candidates = list.iter().collect();
        }
        candidates.into_iter().max_by_key(|p| p.width.saturating_mul(p.height))
    });
    match chosen {
        Some(p) => {
            let direction = split_direction(p.pane_id == own_pane);
            let (resulting_child_width, ratio) = if direction == "right" {
                first_split_geometry(p.width)
            } else {
                (p.width, DOWN_SPLIT_RATIO)
            };
            SplitParent {
                pane_id: p.pane_id.clone(),
                direction,
                ratio,
                refusal: narrow_pane_refusal(&p.pane_id, resulting_child_width),
            }
        }
        None => SplitParent {
            pane_id: own_pane.to_string(),
            direction: "right",
            ratio: DOWN_SPLIT_RATIO,
            refusal: None,
        },
    }
}

/// The workspace half of a herdr pane id (`"w4:p31"` → `"w4"`, herdr's own
/// colon-separated shape) — hps-13's `tab_create` fallback hands herdr the
/// SAME workspace the caller's own pane already reports, never a guessed
/// one. Falls back to the whole id on the (never-observed) shape without a
/// colon, rather than panicking over a herdr id format change.
fn pane_workspace(pane_id: &str) -> &str {
    pane_id.split(':').next().unwrap_or(pane_id)
}

/// How long a queued spawn waits for the pane-split lock before it gives up
/// and splits unserialized. Generous ON PURPOSE: the ack wait a worker is
/// judged by is 180s, so a budget well under that lets several queued spawns
/// take their turn one after another and still ack in time, while a budget
/// at or above it would turn a slow queue into the very timeout this lock
/// exists to prevent.
const SPLIT_LOCK_WAIT: Duration = Duration::from_secs(120);

/// hss-2: the whole pane-split decision — read the tab's layout, choose the
/// parent and direction, then create the pane — under ONE cross-process
/// lock, returning the new worker pane's id.
///
/// The lock is the point. Every spawn is its own process, and the layout
/// read is what makes them disagree: five concurrent spawns from one tab
/// (live 2026-08-21) each read a layout still showing the caller's pane
/// ALONE, each answered "right" from `split_direction`, and all five
/// right-split — panes p3R p3S p3T p3V p3W, with worker 5 dying on the 180s
/// ack wait. Reading the layout under the lock and releasing only once the
/// pane EXISTS means the next spawn's read already counts this pane, so it
/// picks that worker pane as the parent (D2) and answers "down".
///
/// Fail OPEN, the same habit the layout read (`resolve_split_parent`'s
/// `None` arm) and the hps-8 workspace-trust pre-flight already follow: a
/// busy budget or a broken lock file warns and splits anyway. A spawn that
/// never happens is worse than a mis-directed one.
///
/// `lock_wait` is the acquire budget — `SPLIT_LOCK_WAIT` at the one
/// production call site. It is a parameter, not a read of the const, so the
/// budget-spent fail-open branch is provable in a test that takes
/// milliseconds instead of two minutes.
fn split_worker_pane(
    herdr: &dyn PaneTransport,
    own_pane: &str,
    cwd: &Path,
    job_id: &str,
    main_root: &Path,
    lock_wait: Duration,
) -> Result<String, String> {
    // Held for the whole body: the guard releases in `Drop`, at the return
    // below, by which point the new pane already exists.
    let _split_lock = match split_lock::acquire(main_root, job_id, lock_wait) {
        Ok(held) => {
            if held.is_none() {
                eprintln!(
                    "bee herding run: the herding pane-split lock stayed busy for {}s — splitting anyway",
                    lock_wait.as_secs()
                );
            }
            held
        }
        Err(e) => {
            eprintln!("bee herding run: could not take the herding pane-split lock ({e}) — splitting anyway");
            None
        }
    };

    let layout = herdr.pane_layout(own_pane);
    let parent = resolve_split_parent(layout.as_deref(), own_pane);
    match parent.refusal {
        None => herdr.pane_split(&parent.pane_id, parent.direction, parent.ratio, cwd),
        // hps-13: no pane in the caller's tab clears the child-width guard
        // — a refusal here would still be a sliver's fault, not a fresh
        // tab's, so try a fresh tab FIRST and refuse only if that fails too.
        // The new tab's root pane is used directly, with no split call at
        // all, so the worker starts at full width.
        Some(width_msg) => {
            let workspace = pane_workspace(own_pane);
            herdr.tab_create(workspace, cwd, job_id).map_err(|create_err| {
                format!("{width_msg}; tried opening a fresh tab instead and that failed too: {create_err}")
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PollDecision {
    Continue,
    ResultReady,
    TimedOutIdle,
    TimedOutCeiling,
    PausedLimit,
    Died { pid: Option<u32> },
    /// D3: herdr reported `blocked` this tick — ends the wait at once,
    /// ahead of the idle timeout and the ceiling.
    Blocked,
}

/// The whole D5 timing rule, pure: a result already present short-circuits
/// everything else; otherwise the absolute ceiling caps the wait REGARDLESS
/// of a fresh heartbeat (checked first, so it wins a tie with the idle
/// check); a died process observation (D1/D3) reports died before idle;
/// a heartbeat gone stale past `idle_timeout_secs` times out (or pauses on
/// a limit if the pane text matches LIMIT_PATTERNS); a fresh one lets the
/// wait continue.
fn decide_poll(
    now_ms: i64,
    started_at_ms: i64,
    last_heartbeat_ms: i64,
    idle_timeout_secs: u64,
    ceiling_secs: u64,
    result_ready: bool,
    pane_text: Option<&str>,
    liveness_died: Option<Option<u32>>,
) -> PollDecision {
    if result_ready {
        return PollDecision::ResultReady;
    }
    if now_ms.saturating_sub(started_at_ms) >= (ceiling_secs as i64).saturating_mul(1000) {
        return PollDecision::TimedOutCeiling;
    }
    if let Some(pid) = liveness_died {
        return PollDecision::Died { pid };
    }
    if now_ms.saturating_sub(last_heartbeat_ms) >= (idle_timeout_secs as i64).saturating_mul(1000) {
        if let Some(text) = pane_text {
            if find_limit_match(text).is_some() {
                return PollDecision::PausedLimit;
            }
        }
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
/// `run_poll_loop` as a pure observation struct.
struct PollTick {
    result_ready: bool,
    heartbeat_fresh: bool,
    pane_text: Option<String>,
    liveness: Option<Liveness>,
    /// D3: this tick observed herdr's `blocked` status.
    blocked: bool,
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
    mut tick: impl FnMut(bool) -> PollTick,
    mut sleep: impl FnMut(Duration),
    mut now: impl FnMut() -> i64,
) -> PollDecision {
    let mut last_heartbeat_ms = started_at_ms;
    let mut absent_reads = 0u32;
    let mut last_seen_pid: Option<u32> = None;
    loop {
        sleep(poll_interval);
        let tick_now_ms = now();
        let heartbeat_already_stale = tick_now_ms.saturating_sub(last_heartbeat_ms)
            >= (idle_timeout_secs as i64).saturating_mul(1000);
        let observed = tick(heartbeat_already_stale);
        if observed.blocked && !observed.result_ready {
            return PollDecision::Blocked;
        }
        if observed.heartbeat_fresh {
            last_heartbeat_ms = tick_now_ms;
        }
        if let Some(liveness) = observed.liveness {
            match liveness {
                Liveness::Alive { pid } => {
                    last_seen_pid = Some(pid);
                    absent_reads = 0;
                }
                Liveness::Unknown => {
                    absent_reads = 0;
                }
                Liveness::Absent => {
                    absent_reads += 1;
                }
            }
        }
        let liveness_died = if absent_reads >= 3 {
            Some(last_seen_pid)
        } else {
            None
        };
        let decision = decide_poll(
            tick_now_ms,
            started_at_ms,
            last_heartbeat_ms,
            idle_timeout_secs,
            ceiling_secs,
            observed.result_ready,
            observed.pane_text.as_deref(),
            liveness_died,
        );
        if decision != PollDecision::Continue {
            return decision;
        }
    }
}

/// D4: single-quotes `value` for a POSIX shell, escaping any embedded
/// single quote as `'\''` (close the quote, an escaped literal quote,
/// reopen) — the standard shell-injection-safe spelling.
fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// D4's one-line env export: `export K='v' K2='v2' …`, keys in `env`'s own
/// (sorted, `BTreeMap`) order, every value single-quoted. Callers only
/// invoke this when `env` is non-empty — an empty map is never sent.
fn build_export_line(env: &BTreeMap<String, String>) -> String {
    let assignments: Vec<String> = env.iter().map(|(k, v)| format!("{k}={}", shell_single_quote(v))).collect();
    format!("export {}", assignments.join(" "))
}

/// herding-start-retry D1: a freshly split pane's shell may not have
/// reached its prompt when `agent start` fires — herdr refuses with
/// `agent_pane_busy` ("not an available shell"; live dogfood hee-1 and the
/// hsr-1 eval both hit it). Retry the start, bounded, ~1s apart; any error
/// that is not the busy shape fails immediately as before.
const START_RETRY_ATTEMPTS: u32 = 10;

fn is_pane_busy_error(e: &str) -> bool {
    e.contains("agent_pane_busy") || e.contains("not an available shell")
}

/// herding-prompt-stall D1: herdr's own five-second stall detector — a
/// prompt sent from a non-`working` state that produces no observed
/// lifecycle change within the `--timeout` window comes back
/// `agent_prompt_stalled` instead of herdr waiting indefinitely. Mirrors
/// `is_pane_busy_error`'s house style: a small predicate over herdr's error
/// string, so the caller can branch a stall away from an ordinary transport
/// error.
fn is_agent_prompt_stalled(e: &str) -> bool {
    e.contains("agent_prompt_stalled")
}

/// herding-prompt-stall D1/D4 (hps-11): herdr's OTHER `agent prompt --wait`
/// failure shape — the submission itself landed (herdr's own five-second
/// stall detector did not fire), only the state did not settle inside
/// bee's `--timeout` window. Distinct from `is_agent_prompt_stalled`: a
/// stall means no submission was observed at all, a timeout means one WAS
/// made. Mirrors `is_agent_prompt_stalled`'s house style — a small
/// substring predicate over herdr's error string — captured live from
/// `{"error":{"code":"timeout","message":"timed out waiting for agent
/// status"}}`.
fn is_agent_prompt_timeout(e: &str) -> bool {
    e.contains("\"code\":\"timeout\"") || e.contains("timed out waiting for agent status")
}

fn start_with_retry(
    start: &mut dyn FnMut() -> Result<(), String>,
    sleep: &mut dyn FnMut(Duration),
) -> Result<(), String> {
    let mut last = String::new();
    for attempt in 1..=START_RETRY_ATTEMPTS {
        match start() {
            Ok(()) => return Ok(()),
            Err(e) if is_pane_busy_error(&e) => {
                last = e;
                if attempt < START_RETRY_ATTEMPTS {
                    sleep(POLL_INTERVAL * 5);
                }
            }
            Err(e) => return Err(e),
        }
    }
    Err(format!("{last} (after {START_RETRY_ATTEMPTS} start attempts, shell never became available)"))
}

/// herdr's `agent_prompt --wait --until working --timeout` window
/// (herding-prompt-stall D1, raised by hps-11): setting this to EXACTLY
/// herdr's own five-second stall detector gave the wait no room at all —
/// bee's own client-side deadline and herdr's internal detector raced for
/// the same instant, so bee's deadline routinely won and bee saw a bare
/// `{"error":{"code":"timeout",...}}` before herdr's detector ever got to
/// fire (or the agent got to settle). Captured live on a healthy pane:
/// `--timeout 5000` returned `timeout`; `--timeout 20000` on the SAME pane
/// returned a `working` observation and the brief landed. Comfortably
/// above herdr's 5s window, not padding it further.
const AGENT_PROMPT_TIMEOUT_MS: u64 = 20_000;

/// herding-prompt-stall D4 (hps-6 narrows the cadence, decisions unchanged):
/// how many times the idempotent pointer is (re)sent while the agent keeps
/// returning to a ready state (`idle`/`done`) with neither the round's ack
/// file nor its result file present — bounded, mirroring
/// `START_RETRY_ATTEMPTS`'s house style, never an infinite resend loop. This
/// bound is NEVER consumed by a healthy `working` agent: polling a `working`
/// agent burns no resend attempts at all, only wall-clock time against
/// `ACK_WAIT_BUDGET_SECS` below.
const DELIVERY_RESEND_ATTEMPTS: u32 = 10;

/// hps-6: the wall-clock ceiling on the WHOLE delivery wait, from the first
/// send to the ack (or result) appearing — about how long a slow first agent
/// turn takes to read a brief and write its ack, never about how fast the
/// poll ticks. Deliberately well under `DEFAULT_IDLE_TIMEOUT_SECS` (900s): a
/// delivery that is still unacked this far in is a stuck submission, not a
/// slow worker, and should fail as `NeverAcked` long before the round's own
/// idle timeout would ever fire.
const ACK_WAIT_BUDGET_SECS: u64 = 180;

/// hps-6: which of `deliver_pointer`'s two independent bounds ran out —
/// named so `DeliveryError::NeverAcked`'s message tells the two apart
/// instead of leaving the reader to guess which one fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NeverAckedBound {
    /// The agent kept returning to `idle`/`done` with no ack, and the
    /// pointer was resent `DELIVERY_RESEND_ATTEMPTS` times.
    ResendAttempts,
    /// The wall-clock `ACK_WAIT_BUDGET_SECS` window elapsed — whether the
    /// agent was still `working`, flapping ready/working, or unreadable.
    AckWaitBudget,
}

/// herding-prompt-stall D1/D3: the delivery outcomes `deliver_pointer` can
/// return, kept distinct because each gets its own message and neither ever
/// triggers a resend loop.
#[derive(Debug)]
enum DeliveryError {
    /// D3: the pane already reported `blocked` (an approval or question UI)
    /// before the pointer was ever sent.
    Blocked,
    /// An ordinary herdr-call failure, unrelated to the agent's lifecycle
    /// (herdr missing, a non-zero exit, an unparseable body).
    Transport(String),
    /// hps-6: one of the two bounds above ran out and neither the round's
    /// ack file nor its result file ever appeared — herdr's own lifecycle
    /// state (a successful "working" observation) is never enough on its
    /// own any more.
    NeverAcked { bound: NeverAckedBound, attempts: u32 },
    /// D6 (hps-14): the resend bound ran out and EVERY send inside it came
    /// back `agent_prompt_stalled` — herdr never observed so much as a
    /// lifecycle change, let alone an ack. Kept distinct from `NeverAcked`:
    /// that variant means the agent took the text and never confirmed it;
    /// this one means the agent never took the text at all.
    NeverDelivered { attempts: u32 },
}

impl std::fmt::Display for DeliveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliveryError::Blocked => write!(
                f,
                "pane is blocked (herdr recognized an approval or question UI) before the prompt was ever sent"
            ),
            DeliveryError::Transport(e) => write!(f, "{e}"),
            DeliveryError::NeverAcked { bound: NeverAckedBound::ResendAttempts, attempts } => write!(
                f,
                "resent the pointer {attempts} time(s) (the agent kept going ready with no ack) but neither the \
                 round's ack file nor its result file ever appeared"
            ),
            DeliveryError::NeverAcked { bound: NeverAckedBound::AckWaitBudget, attempts } => write!(
                f,
                "waited past the {ACK_WAIT_BUDGET_SECS}s ack-wait budget ({attempts} send(s) made) but neither the \
                 round's ack file nor its result file ever appeared"
            ),
            DeliveryError::NeverDelivered { attempts } => write!(
                f,
                "resent the pointer {attempts} time(s) and every submission stalled (herdr observed no lifecycle \
                 change at all) — the agent never took the text, unlike a never-acked submission it did take"
            ),
        }
    }
}

/// herding-prompt-stall D4 (narrows D1; supersedes herding-pointer-delivery
/// D1 / herding-receipt-state D1; hps-6 narrows the cadence below, decisions
/// unchanged): herdr's own atomic submit-and-observe — `herdr agent prompt
/// <job> <text> --wait --until working --timeout <ms>` — still sends the
/// pointer. `agent_prompt`'s reply now sorts into THREE outcomes, not two
/// (hps-11). A pane already `blocked` (checked before the send is even
/// attempted — there is no point submitting into a pane waiting on an
/// unrelated question) still fails FAST as a hard delivery error.
/// `agent_prompt_stalled` (no observed lifecycle change at all within
/// herdr's own five-second window) no longer does (D6, hps-14): a stall
/// proves the submission had not landed AT THAT INSTANT, not that it never
/// will, so it joins the SAME bounded retry the ready-with-no-ack path
/// below already uses instead of ending the run on the first occurrence —
/// live evidence: a fresh, full-width pane's `agent_prompt_stalled` reply,
/// with the IDENTICAL prompt typed by hand seconds later on that same pane
/// delivered at once (docs/history/herding-prompt-stall/CONTEXT.md). A
/// `timeout` reply is different: the submission WAS made, only the state did not
/// settle inside bee's own wait window — that is not a delivery failure,
/// so it falls straight through into the same ack poll a successful send
/// enters, exactly like the `Ok(())` branch below with no ack yet. A
/// successful "working" observation is no longer the receipt either way:
/// herdr lifecycle state
/// proved unreliable on its own (a boot flap through
/// unknown/working/idle/done satisfied the old transition test and
/// receipted a pointer a booting TUI discarded — live: job trust-par-2,
/// docs/history/herding-prompt-stall/CONTEXT.md). The receipt is now the
/// worker's OWN ack file, or the round's result file for an ultra-fast
/// round that finishes before an ack is ever observed (`result_present()`,
/// kept as the pre-existing escape, `ack_present()` added beside it).
///
/// hps-6's cadence, once a send has gone out: `working` is the HEALTHY
/// path, so it is polled, never resent — a worker reading a ten-kilobyte
/// brief must see exactly one send. The pointer is idempotent, so a resend
/// only fires once the agent has gone back to a READY state (`idle`/`done`,
/// D2's vocabulary) with STILL no ack — that is the actual signature of a
/// submission the TUI dropped, not a slow worker. `blocked` still ends the
/// wait at once, whether observed before the first send or mid-poll, never
/// swallowed by another attempt; a stall (D6, hps-14) does not — it is
/// retried under this SAME resend, sleeping the existing `POLL_INTERVAL`
/// backoff before resending the idempotent pointer, checked against the
/// ack and result files between attempts exactly as the ready-with-no-ack
/// path already is, so a worker mid-write on an earlier submission is never
/// resent into needlessly. The wait is bounded two ways —
/// `DELIVERY_RESEND_ATTEMPTS` resends and the wall-clock
/// `ACK_WAIT_BUDGET_SECS` — and exhausting the resend bound ends it as
/// `NeverAcked` when at least one send got past herdr's stall detector, or
/// as `NeverDelivered` when EVERY send in the bound stalled — distinct
/// wording so the reader can tell "the agent never took the text" from
/// "the agent took it and never acked".
fn deliver_pointer(
    pointer: &str,
    prompt: &mut dyn FnMut(&str) -> Result<(), String>,
    status: &mut dyn FnMut() -> Option<String>,
    ack_present: &mut dyn FnMut() -> bool,
    result_present: &mut dyn FnMut() -> bool,
    sleep: &mut dyn FnMut(Duration),
    now: &mut dyn FnMut() -> i64,
) -> Result<(), DeliveryError> {
    let started_ms = now();
    let mut sends = 0u32;
    let mut stalls = 0u32;
    loop {
        if status().as_deref() == Some("blocked") {
            return Err(DeliveryError::Blocked);
        }
        sends += 1;
        if sends > DELIVERY_RESEND_ATTEMPTS {
            return Err(if stalls == DELIVERY_RESEND_ATTEMPTS {
                DeliveryError::NeverDelivered { attempts: DELIVERY_RESEND_ATTEMPTS }
            } else {
                DeliveryError::NeverAcked {
                    bound: NeverAckedBound::ResendAttempts,
                    attempts: DELIVERY_RESEND_ATTEMPTS,
                }
            });
        }
        match prompt(pointer) {
            Ok(()) => {
                if ack_present() || result_present() {
                    return Ok(());
                }
            }
            Err(_) if ack_present() || result_present() => return Ok(()),
            Err(e) if is_agent_prompt_stalled(&e) => {
                // D6 (hps-14): a stall proves the submission had not landed
                // AT THAT INSTANT, not that it never will — sleep the same
                // backoff the ready-with-no-ack path already sleeps, then
                // let the outer loop resend under the SAME bounds, never
                // returning early here.
                stalls += 1;
                sleep(POLL_INTERVAL);
                continue;
            }
            Err(e) if is_agent_prompt_timeout(&e) => {
                // The submission WAS made — herdr's own stall detector
                // never fired, only bee's wait window ran out before the
                // state settled. Not a delivery failure: fall through into
                // the same ack poll a successful send enters below.
            }
            Err(e) => return Err(DeliveryError::Transport(e)),
        }

        // Sent successfully, no ack or result yet: poll (never resend)
        // until the ack/result appears, the pane goes blocked, the agent
        // goes back to ready with still no ack (resend), or the wall-clock
        // budget runs out.
        loop {
            if now().saturating_sub(started_ms) >= (ACK_WAIT_BUDGET_SECS as i64).saturating_mul(1000) {
                return Err(DeliveryError::NeverAcked { bound: NeverAckedBound::AckWaitBudget, attempts: sends });
            }
            sleep(POLL_INTERVAL);
            if ack_present() || result_present() {
                return Ok(());
            }
            match status().as_deref() {
                Some("blocked") => return Err(DeliveryError::Blocked),
                Some("idle") | Some("done") => break, // ready again, still no ack — resend
                _ => {}                                // working/unknown — healthy, keep polling
            }
        }
    }
}

/// herding-prompt-stall D2 (narrows herding-run-ready-wait D1): polls
/// `status()` until the agent reports `idle` OR `done` — herdr states
/// `done` is the same underlying ready-for-input state for a tab that has
/// not been seen in the focused UI, CLI reads never mark a tab seen, and
/// every pane this verb splits carries `--no-focus`, so `done` is the
/// NORMAL resting state of a bee worker pane; idle-only rejected a pane
/// that was ready. D3: a `blocked` status ends the wait at once — the
/// ready-wait ceiling and its sleeps are never burned on a question nobody
/// is going to answer. Check-then-sleep: an agent already idle/done is
/// accepted with zero sleeps, and a 0-second ceiling with no status is
/// exhausted immediately — both drive the tests without a real clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadyOutcome {
    Ready,
    Blocked,
    TimedOut,
}

fn wait_for_agent_ready(
    ready_wait_secs: u64,
    poll_interval: Duration,
    mut status: impl FnMut() -> Option<String>,
    mut sleep: impl FnMut(Duration),
    mut now: impl FnMut() -> i64,
) -> ReadyOutcome {
    let started = now();
    loop {
        match status().as_deref() {
            Some("idle") | Some("done") => return ReadyOutcome::Ready,
            Some("blocked") => return ReadyOutcome::Blocked,
            _ => {}
        }
        if now().saturating_sub(started) >= (ready_wait_secs as i64).saturating_mul(1000) {
            return ReadyOutcome::TimedOut;
        }
        sleep(poll_interval);
    }
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// The last few lines of a pane's capture, for a blocked-pane error's
/// remedy text — enough to show WHAT is being asked without dumping a full
/// screen (reuses `PaneTransport::pane_read`).
fn pane_tail_from_text(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(8);
    lines[start..].join("\n")
}

fn pane_tail(herdr: &dyn PaneTransport, pane_id: &str) -> String {
    let text = herdr.pane_read(pane_id).unwrap_or_default();
    pane_tail_from_text(&text)
}

/// D3: the one message every blocked wait point returns — names the job,
/// the pane, shows its tail, and states the remedy. `blocked` is herdr's
/// whole mechanism for an approval or question UI: bee carries no
/// agent-specific trust-prompt pattern table.
fn blocked_message(job_id: &str, pane_id: &str, tail: &str) -> String {
    format!(
        "job {job_id} pane {pane_id} is blocked (herdr recognized an approval or question UI) — \
remedy: answer the prompt in pane {pane_id}, or pre-authorize whatever it is asking so the agent \
stops asking\npane tail:\n{tail}"
    )
}

/// hps-7 (D3, D5): upgrades a give-up wait's GENERIC message once
/// `find_prompt_diagnosis` names a matching line — same shape as
/// `blocked_message`: names the job and pane, quotes the matched line
/// verbatim, states the remedy, and (hps-9) appends the same `pane_tail`
/// `blocked_message` uses — a clipped narrow-pane capture's matched line can
/// be a short fragment (an arrow-key nav footer, e.g.) that alone does not
/// show what the prompt actually asked; the tail does. Diagnosis-only:
/// called only from a path that has already decided to stop waiting, never
/// as part of deciding whether to keep waiting.
fn diagnosis_message(job_id: &str, pane_id: &str, matched_line: &str, generic: &str, tail: &str) -> String {
    format!(
        "{generic} — pane {pane_id} (job {job_id}) shows what looks like an unanswered prompt: \
\"{matched_line}\" — remedy: answer the prompt in pane {pane_id}, or pre-authorize whatever it is \
asking so the agent stops asking\npane tail:\n{tail}"
    )
}

/// hps-7 (D5), hrc-3 (D3): the one call every give-up wait point makes on its way out,
/// through the existing `pane_read` seam. A match upgrades `generic` via
/// `diagnosis_message`, carrying the same `pane_tail` (hps-9) `blocked_message`
/// uses; no match appends `pane_tail`; a failing `pane_read` returns `generic` UNCHANGED,
/// byte for byte — a `pane_read` failure must never turn a real timeout into
/// a different error.
fn diagnose_giveup(herdr: &dyn PaneTransport, job_id: &str, pane_id: &str, generic: String) -> String {
    let Ok(text) = herdr.pane_read(pane_id) else { return generic };
    let tail = pane_tail_from_text(&text);
    match find_prompt_diagnosis(&text) {
        Some(line) => diagnosis_message(job_id, pane_id, &line, &generic, &tail),
        None => format!("{generic}\npane tail:\n{tail}"),
    }
}

/// Builds the remedy string emitted in JSON output on `RunOutcome::SpawnFailed` (hrc-3, D4).
/// Names how to inspect the forensics pane and unwind claims/reservations.
fn spawn_failed_remedy(cell_id: Option<&str>, agent: Option<&str>, pane_id: Option<&str>) -> String {
    let inspect = match pane_id {
        Some(p) => format!("inspect pane {p} (herdr pane read {p}); "),
        None => String::new(),
    };
    let unwind = match cell_id {
        Some(cell) => match agent {
            Some(a) => format!("unwind: bee cells unclaim --id {cell}; bee reservations release --agent {a} --cell {cell}"),
            None => format!("unwind: bee cells unclaim --id {cell}; bee reservations release --cell {cell}"),
        },
        None => "unwind: no --cell-id was given, release any claim you took by hand".to_string(),
    };
    format!("{inspect}{unwind}")
}

/// hps-8 (D5): the resolved outcome of one workspace-trust pre-flight
/// attempt. `execute_new` turns a `Warning` into one eprintln! line naming
/// the file and what was wrong, then proceeds regardless — tests read this
/// value directly instead of capturing stderr.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TrustPreflightOutcome {
    /// `cwd` was already present in the trusted array — no rewrite.
    AlreadyTrusted,
    /// `cwd` was appended and the file rewritten.
    Appended,
    /// Fail-open: the file could not be read, its contents did not parse
    /// as JSON, the declared key was missing or not an array, or the
    /// rewritten file could not be written back.
    Warning(String),
}

/// hps-8 (D5): pre-seeds a foreign tool's own workspace-trust store so a
/// herd agent that gates on it (Antigravity's `agy`, e.g.) never meets its
/// trust dialog in a brand-new `bee worktree new` directory — `bee` carries
/// no knowledge of what the file means, only that `trust.key` names an
/// array of trusted absolute paths inside `trust.file` (already `~`-expanded
/// by `wave::parse_workspace_trust`). FAIL-OPEN throughout: every branch
/// that cannot proceed returns `Warning` instead of an error, and the
/// caller lets the run continue either way — a foreign tool's config being
/// unreadable or unwritable must never fail a bee run. Never rewrites
/// anything beyond appending `cwd` to the named array.
fn preflight_workspace_trust(trust: &WorkspaceTrust, cwd: &Path) -> TrustPreflightOutcome {
    let file = Path::new(&trust.file);
    let raw = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => return TrustPreflightOutcome::Warning(format!("could not read {} ({e})", trust.file)),
    };
    let mut value: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => return TrustPreflightOutcome::Warning(format!("{} is not valid JSON ({e})", trust.file)),
    };
    let Some(obj) = value.as_object_mut() else {
        return TrustPreflightOutcome::Warning(format!("{} is not a JSON object", trust.file));
    };
    let cwd_str = cwd.display().to_string();
    match obj.get_mut(trust.key.as_str()) {
        Some(Value::Array(paths)) => {
            if paths.iter().any(|p| p.as_str() == Some(cwd_str.as_str())) {
                return TrustPreflightOutcome::AlreadyTrusted;
            }
            paths.push(Value::String(cwd_str));
        }
        _ => {
            return TrustPreflightOutcome::Warning(format!(
                "{}: {:?} is missing or not an array",
                trust.file, trust.key
            ));
        }
    }
    match crate::fsutil::write_json_atomic(file, &value) {
        Ok(()) => TrustPreflightOutcome::Appended,
        Err(e) => TrustPreflightOutcome::Warning(format!("could not write {} ({e})", trust.file)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// the run itself
// ═══════════════════════════════════════════════════════════════════════════

/// The three typed refusals `--continue` can hit before it ever touches
/// `PaneTransport` for real (D3's "refuses typed when…" clause). Each names the
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
    /// `--continue` refused before touching `PaneTransport` for real: the job dir,
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
    /// hps-7 (D5): carries the give-up message — `idle_timeout_message`'s
    /// generic text, or `diagnose_giveup`'s upgrade of it when the pane
    /// shows a matching line.
    TimedOutIdle(String),
    TimedOutCeiling,
    PausedLimit,
    Died { pid: Option<u32> },
    /// D3: herdr reported `blocked` during the round poll — the agent
    /// started and received its prompt, but the pane now shows an approval
    /// or question UI. Distinct from `SpawnFailed`: this fires mid-round,
    /// never before dispatch.
    PaneBlocked(String),
}

struct ExecResult {
    outcome: RunOutcome,
    pane_id: Option<String>,
    closed_pane: bool,
}

/// The round poll's idle-timeout give-up, GENERIC (hps-7): unchanged when
/// `diagnose_giveup` finds no match on the pane, upgraded when it does.
fn idle_timeout_message(idle_timeout_secs: u64) -> String {
    format!("no heartbeat for {idle_timeout_secs}s (idle timeout) — pane kept for inspection")
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

/// The whole verb, generic over `PaneTransport` so tests drive it with a fake
/// (production's only caller, `run()` below, passes `&RealHerdr`). Branches
/// on `--continue` (D3) right at the top: a fresh spawn and a follow-up
/// round share the poll loop (`wait_for_round`) and pane lifecycle, but
/// nothing else — a fresh spawn never reuses a pane, and `--continue` never
/// splits one.
fn execute(opts: &Options, herdr: &dyn PaneTransport) -> ExecResult {
    if opts.is_continue {
        execute_continue(opts, herdr)
    } else {
        execute_new(opts, herdr)
    }
}

fn stamp_paused_limit(bee_dir: &Path, job_id: &str, herdr: &dyn PaneTransport, pane_id: &str) {
    let raw_text = herdr.pane_read(pane_id).unwrap_or_default();
    let limit_reset_hint = find_limit_match(&raw_text).unwrap_or_default();
    let job_file_path = mailbox::job_path(bee_dir, job_id);
    let mut job_value: Value = match std::fs::read_to_string(&job_file_path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
    {
        Some(v) => v,
        None => {
            eprintln!("bee herding run: could not read {} to stamp paused_limit", job_file_path.display());
            return;
        }
    };
    if let Value::Object(ref mut m) = job_value {
        m.insert("paused_limit_at".into(), Value::String(chrono::Utc::now().to_rfc3339()));
        m.insert("limit_reset_hint".into(), Value::String(limit_reset_hint));
    }
    if let Err(e) = crate::fsutil::write_json_atomic(&job_file_path, &job_value) {
        eprintln!("bee herding run: could not stamp paused_limit into {}: {e}", job_file_path.display());
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
    pane_id: &str,
    min_round: u32,
    started_at_ms: i64,
    idle_timeout_secs: u64,
    ceiling_secs: u64,
    herdr: &dyn PaneTransport,
) -> PollDecision {
    let log_file_path = mailbox::log_path(bee_dir, job_id);
    let ack_file_path = mailbox::ack_path(bee_dir, job_id, min_round);
    let mailbox_path = mailbox::mailbox_dir(bee_dir, job_id);
    let mut last_log_mtime: Option<std::time::SystemTime> = None;
    let mut last_ack_mtime: Option<std::time::SystemTime> = None;
    let mut tick_index: u64 = 0;
    run_poll_loop(
        started_at_ms,
        idle_timeout_secs,
        ceiling_secs,
        POLL_INTERVAL,
        |heartbeat_already_stale| {
            tick_index += 1;
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
            // herding-prompt-stall D4: the worker's ack file counts as a
            // heartbeat too, the SAME "mtime advanced since last observed"
            // rule as log.txt above — a worker that acked and is now
            // thinking for a long time is never read as dead off the ack
            // alone appearing once.
            if let Ok(meta) = std::fs::metadata(&ack_file_path) {
                if let Ok(modified) = meta.modified() {
                    if last_ack_mtime.map_or(true, |prev| modified > prev) {
                        last_ack_mtime = Some(modified);
                        heartbeat_fresh = true;
                    }
                }
            }
            // One status read serves both the heartbeat check and the
            // blocked check (D3) — never two herdr calls for one tick.
            let status = herdr.agent_status(job_id);
            if status.as_deref() == Some("working") {
                heartbeat_fresh = true;
            }
            let blocked = status.as_deref() == Some("blocked");
            let liveness = if tick_index % 10 == 0 {
                Some(herdr.process_info(pane_id))
            } else {
                None
            };
            let pane_text = if heartbeat_already_stale {
                herdr.pane_read(pane_id).ok()
            } else {
                None
            };
            PollTick { result_ready, heartbeat_fresh, pane_text, liveness, blocked }
        },
        |d| std::thread::sleep(d),
        now_ms,
    )
}

/// A fresh spawn: split a pane off the caller's own, `agent start` into it,
/// then wait for round 1.
fn execute_new(opts: &Options, herdr: &dyn PaneTransport) -> ExecResult {
    let bee_dir = opts.main_root.join(".bee");
    let files: Vec<String> = Vec::new();
    let spec = BriefSpec {
        job_id: &opts.job_id,
        task: &opts.task,
        worktree_root: &opts.cwd,
        files: &files,
        bee_dir: &bee_dir,
        round: 1,
        expertise: &opts.expertise,
        nickname: &opts.nickname,
        cell_id: opts.cell_id.as_deref(),
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
        "expertise": opts.expertise,
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

    let cfg = read_main_config(&opts.main_root);
    let (kind, args, env, workspace_trust) = match resolve_agent_command(&cfg, opts.agent.as_deref()) {
        Ok(quad) => quad,
        Err(e) => {
            return ExecResult { outcome: RunOutcome::SpawnFailed(e.to_string()), pane_id: None, closed_pane: false }
        }
    };

    // hps-8 (D5): pre-flight BEFORE the pane split and `agent_start`, never
    // after — a herd agent that gates on a per-workspace trust prompt
    // (Antigravity's `agy`, e.g.) must find its own workspace already
    // trusted the moment it boots into the freshly split pane. Fail-open:
    // a `Warning` is reported and the run proceeds regardless.
    if let Some(trust) = &workspace_trust {
        if let TrustPreflightOutcome::Warning(msg) = preflight_workspace_trust(trust, &opts.cwd) {
            eprintln!("bee herding run: workspace-trust pre-flight failed ({msg}) — proceeding anyway");
        }
    }

    let own_pane = match herdr.pane_current() {
        Ok(p) => p,
        Err(e) => return ExecResult { outcome: RunOutcome::SpawnFailed(e), pane_id: None, closed_pane: false },
    };
    // hss-2: the layout read and the split itself run under one
    // cross-process lock, so concurrent spawns from one tab see each other's
    // panes instead of all reading `pane_count` 1 and all splitting `right`.
    let new_pane = match split_worker_pane(
        herdr,
        &own_pane,
        &opts.cwd,
        &opts.job_id,
        &opts.main_root,
        SPLIT_LOCK_WAIT,
    ) {
        Ok(p) => p,
        Err(e) => return ExecResult { outcome: RunOutcome::SpawnFailed(e), pane_id: None, closed_pane: false },
    };

    // D4 + herding-worker-standalone D2: the registry entry's env is
    // exported into the freshly split pane BEFORE the agent starts — so the
    // agent's own process inherits it — merged with the
    // `BEE_HERDING_WORKER=1` marker, which wins over any same-name
    // per-agent value. The marker means this export is sent on EVERY fresh
    // spawn, not only when the per-agent env is non-empty. Treated exactly
    // like a start failure: the agent never started, so the pane this call
    // just split is closed.
    let mut pane_env = env.clone();
    pane_env.insert("BEE_HERDING_WORKER".to_string(), "1".to_string());
    let export_line = build_export_line(&pane_env);
    if let Err(e) = herdr.pane_run(&new_pane, &export_line) {
        let closed = herdr.pane_close(&new_pane).is_ok();
        if !closed {
            eprintln!("bee herding run: could not close pane {new_pane} after a failed env export");
        }
        return ExecResult {
            outcome: RunOutcome::SpawnFailed(format!("env export failed: {e}")),
            pane_id: Some(new_pane),
            closed_pane: closed,
        };
    }

    let start_result = start_with_retry(
        &mut || herdr.agent_start(&opts.job_id, &kind, &new_pane, &args),
        &mut |d| std::thread::sleep(d),
    );
    if let Err(e) = start_result {
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
        || herdr.agent_wait(&opts.job_id, POLL_INTERVAL.as_millis() as u64),
        |d| std::thread::sleep(d),
        now_ms,
    );
    match ready {
        ReadyOutcome::Ready => {}
        ReadyOutcome::Blocked => {
            let tail = pane_tail(herdr, &new_pane);
            return ExecResult {
                outcome: RunOutcome::SpawnFailed(blocked_message(&opts.job_id, &new_pane, &tail)),
                pane_id: Some(new_pane),
                closed_pane: false,
            };
        }
        ReadyOutcome::TimedOut => {
            let generic = format!(
                "agent never reported ready within {}s (ready-wait) — pane kept for inspection",
                opts.ready_wait_secs
            );
            return ExecResult {
                outcome: RunOutcome::SpawnFailed(diagnose_giveup(herdr, &opts.job_id, &new_pane, generic)),
                pane_id: Some(new_pane),
                closed_pane: false,
            };
        }
    }

    // The brief body lives in brief-1.txt and the agent receives a ONE-LINE
    // pointer (herding-brief-file D1): a multi-line prompt is silently
    // dropped by at least one agent kind even when idle and ready. The
    // agent IS running past this point, so a failure here keeps the pane
    // as forensics (the standing failure rule), unlike the start failure
    // above.
    let brief_file = mailbox::brief_path(&bee_dir, &opts.job_id, 1);
    if let Err(e) = crate::fsutil::write_text_atomic(&brief_file, &brief) {
        return ExecResult {
            outcome: RunOutcome::SpawnFailed(format!("could not write {}: {e}", brief_file.display())),
            pane_id: Some(new_pane),
            closed_pane: false,
        };
    }
    let pointer = mailbox::pointer_prompt(&brief_file);
    let round1_result = mailbox::result_path(&bee_dir, &opts.job_id, 1);
    let round1_ack = mailbox::ack_path(&bee_dir, &opts.job_id, 1);
    if let Err(e) = deliver_pointer(
        &pointer,
        &mut |p| herdr.agent_prompt(&opts.job_id, p, "working", AGENT_PROMPT_TIMEOUT_MS),
        &mut || herdr.agent_status(&opts.job_id),
        &mut || round1_ack.exists(),
        &mut || round1_result.exists(),
        &mut |d| std::thread::sleep(d),
        &mut || now_ms(),
    ) {
        let msg = match &e {
            DeliveryError::Blocked => blocked_message(&opts.job_id, &new_pane, &pane_tail(herdr, &new_pane)),
            DeliveryError::NeverAcked { .. } | DeliveryError::NeverDelivered { .. } => {
                diagnose_giveup(herdr, &opts.job_id, &new_pane, format!("brief prompt failed after start: {e}"))
            }
            _ => format!("brief prompt failed after start: {e}"),
        };
        return ExecResult { outcome: RunOutcome::SpawnFailed(msg), pane_id: Some(new_pane), closed_pane: false };
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
        wait_for_round(&bee_dir, &opts.job_id, &new_pane, 1, started_at_ms, opts.idle_timeout_secs, opts.ceiling_secs, herdr);

    let outcome = match decision {
        PollDecision::ResultReady => read_result(&bee_dir, &opts.job_id),
        PollDecision::TimedOutIdle => {
            let generic = idle_timeout_message(opts.idle_timeout_secs);
            RunOutcome::TimedOutIdle(diagnose_giveup(herdr, &opts.job_id, &new_pane, generic))
        }
        PollDecision::TimedOutCeiling => RunOutcome::TimedOutCeiling,
        PollDecision::PausedLimit => {
            stamp_paused_limit(&bee_dir, &opts.job_id, herdr, &new_pane);
            RunOutcome::PausedLimit
        }
        PollDecision::Died { pid } => RunOutcome::Died { pid },
        PollDecision::Blocked => {
            let tail = pane_tail(herdr, &new_pane);
            RunOutcome::PaneBlocked(blocked_message(&opts.job_id, &new_pane, &tail))
        }
        PollDecision::Continue => unreachable!("run_poll_loop only returns on a non-Continue decision"),
    };

    let valid_result = matches!(outcome, RunOutcome::Result(_));
    let close = !matches!(outcome, RunOutcome::PausedLimit) && should_close_pane(valid_result, opts.close_always);
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

fn parse_brief_filename(name: &str) -> Option<u32> {
    let digits = name.strip_prefix("brief-")?.strip_suffix(".txt")?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse::<u32>().ok()
}

/// `--continue <job-id>` (D3): reuse the EXISTING job mailbox — read the
/// highest prior round, render round N+1's brief, `agent prompt` it to the
/// job's recorded pane, then wait for round N+1's result under the same
/// timing and pane-lifecycle rules a fresh spawn uses. Never calls
/// `agent_start` — the whole point is addressing the SAME agent again, not
/// starting a second one.
///
/// When the job carries a `paused_limit_at` stamp, `--continue` takes the
/// same-round resume branch (herding-limit-pause D3) instead of advancing
/// to round N+1.
fn execute_continue(opts: &Options, herdr: &dyn PaneTransport) -> ExecResult {
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

    // herding-limit-pause D3: check for a limit pause stamp before the next-round logic.
    let paused_limit_at = job_value
        .get("paused_limit_at")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());

    if paused_limit_at.is_some() {
        let recorded_pane = job_value.get("pane_id").and_then(Value::as_str).map(str::to_string);
        let resume_round = entries
            .iter()
            .filter_map(|name| parse_brief_filename(name))
            .max()
            .or_else(|| job_value.get("round").and_then(Value::as_u64).map(|r| r as u32))
            .unwrap_or(1);
        let round_result = mailbox::result_path(&bee_dir, job_id, resume_round);
        let pointer = format!(
            "your session was paused by a usage limit; continue the task and write the round-{resume_round} result file at {}",
            round_result.display()
        );

        if opts.dry_run {
            return ExecResult {
                outcome: RunOutcome::DryRun(pointer),
                pane_id: None,
                closed_pane: false,
            };
        }

        // 1a. If the recorded pane is NOT alive: return PaneGone typed refusal
        let pane_id = match &recorded_pane {
            Some(p) if herdr.pane_alive(p) => p.clone(),
            _ => {
                return refused(ContinueRefusal::PaneGone {
                    job_id: job_id.clone(),
                    pane_id: recorded_pane,
                });
            }
        };

        // 1b. Deliver resume nudge through standard deliver_pointer path
        let round_ack = mailbox::ack_path(&bee_dir, job_id, resume_round);
        if let Err(e) = deliver_pointer(
            &pointer,
            &mut |p| herdr.agent_prompt(job_id, p, "working", AGENT_PROMPT_TIMEOUT_MS),
            &mut || herdr.agent_status(job_id),
            &mut || round_ack.exists(),
            &mut || round_result.exists(),
            &mut |d| std::thread::sleep(d),
            &mut || now_ms(),
        ) {
            let msg = match &e {
                DeliveryError::Blocked => blocked_message(job_id, &pane_id, &pane_tail(herdr, &pane_id)),
                DeliveryError::NeverAcked { .. } | DeliveryError::NeverDelivered { .. } => {
                    diagnose_giveup(herdr, job_id, &pane_id, format!("agent prompt failed: {e}"))
                }
                _ => format!("agent prompt failed: {e}"),
            };
            return ExecResult { outcome: RunOutcome::SpawnFailed(msg), pane_id: Some(pane_id), closed_pane: false };
        }

        // On delivered: rewrite job.json WITHOUT paused_limit_at/limit_reset_hint (atomic)
        let mut updated_job = job_value.clone();
        if let Value::Object(ref mut m) = updated_job {
            m.remove("paused_limit_at");
            m.remove("limit_reset_hint");
        }
        if let Err(e) = crate::fsutil::write_json_atomic(&job_file_path, &updated_job) {
            eprintln!("bee herding run --continue: could not update {}: {e}", job_file_path.display());
        }

        // Re-enter wait_for_round for the SAME round
        let started_at_ms = now_ms();
        let decision = wait_for_round(
            &bee_dir,
            job_id,
            &pane_id,
            resume_round,
            started_at_ms,
            opts.idle_timeout_secs,
            opts.ceiling_secs,
            herdr,
        );

        let outcome = match decision {
            PollDecision::ResultReady => read_result(&bee_dir, job_id),
            PollDecision::TimedOutIdle => {
                let generic = idle_timeout_message(opts.idle_timeout_secs);
                RunOutcome::TimedOutIdle(diagnose_giveup(herdr, job_id, &pane_id, generic))
            }
            PollDecision::TimedOutCeiling => RunOutcome::TimedOutCeiling,
            PollDecision::PausedLimit => {
                stamp_paused_limit(&bee_dir, job_id, herdr, &pane_id);
                RunOutcome::PausedLimit
            }
            PollDecision::Died { pid } => RunOutcome::Died { pid },
            PollDecision::Blocked => {
                let tail = pane_tail(herdr, &pane_id);
                RunOutcome::PaneBlocked(blocked_message(job_id, &pane_id, &tail))
            }
            PollDecision::Continue => unreachable!("run_poll_loop only returns on a non-Continue decision"),
        };

        let valid_result = matches!(outcome, RunOutcome::Result(_));
        let close = !matches!(outcome, RunOutcome::PausedLimit) && should_close_pane(valid_result, opts.close_always);
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

        return ExecResult {
            outcome,
            pane_id: Some(pane_id),
            closed_pane,
        };
    }

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

    let recorded_expertise: Vec<ExpertiseEntry> = match job_value.get("expertise") {
        Some(val) => serde_json::from_value(val.clone()).unwrap_or_default(),
        None => Vec::new(),
    };
    let expertise = if opts.has_explicit_expertise {
        opts.expertise.clone()
    } else {
        recorded_expertise
    };

    let files: Vec<String> = Vec::new();
    let spec = BriefSpec {
        job_id,
        task: &opts.task,
        worktree_root: &worktree_root,
        files: &files,
        bee_dir: &bee_dir,
        round: next_round,
        expertise: &expertise,
        nickname: &opts.nickname,
        cell_id: opts.cell_id.as_deref(),
    };
    let brief = mailbox::render_brief(&spec);

    if opts.dry_run {
        // Renders the round N+1 brief and sends nothing — no `PaneTransport` call
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

    let brief_file = mailbox::brief_path(&bee_dir, job_id, next_round);
    if let Err(e) = crate::fsutil::write_text_atomic(&brief_file, &brief) {
        return ExecResult {
            outcome: RunOutcome::SpawnFailed(format!("could not write {}: {e}", brief_file.display())),
            pane_id: Some(pane_id),
            closed_pane: false,
        };
    }
    let pointer = mailbox::pointer_prompt(&brief_file);
    let round_result = mailbox::result_path(&bee_dir, job_id, next_round);
    let round_ack = mailbox::ack_path(&bee_dir, job_id, next_round);
    if let Err(e) = deliver_pointer(
        &pointer,
        &mut |p| herdr.agent_prompt(job_id, p, "working", AGENT_PROMPT_TIMEOUT_MS),
        &mut || herdr.agent_status(job_id),
        &mut || round_ack.exists(),
        &mut || round_result.exists(),
        &mut |d| std::thread::sleep(d),
        &mut || now_ms(),
    ) {
        let msg = match &e {
            DeliveryError::Blocked => blocked_message(job_id, &pane_id, &pane_tail(herdr, &pane_id)),
            DeliveryError::NeverAcked { .. } | DeliveryError::NeverDelivered { .. } => {
                diagnose_giveup(herdr, job_id, &pane_id, format!("agent prompt failed: {e}"))
            }
            _ => format!("agent prompt failed: {e}"),
        };
        return ExecResult { outcome: RunOutcome::SpawnFailed(msg), pane_id: Some(pane_id), closed_pane: false };
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
        if opts.has_explicit_expertise {
            m.insert("expertise".into(), serde_json::to_value(&opts.expertise).unwrap_or_default());
        }
    }
    if let Err(e) = crate::fsutil::write_json_atomic(&job_file_path, &updated_job) {
        eprintln!("bee herding run --continue: could not update {}: {e}", job_file_path.display());
    }

    let started_at_ms = now_ms();
    let decision = wait_for_round(
        &bee_dir,
        job_id,
        &pane_id,
        next_round,
        started_at_ms,
        opts.idle_timeout_secs,
        opts.ceiling_secs,
        herdr,
    );

    let outcome = match decision {
        PollDecision::ResultReady => read_result(&bee_dir, job_id),
        PollDecision::TimedOutIdle => {
            let generic = idle_timeout_message(opts.idle_timeout_secs);
            RunOutcome::TimedOutIdle(diagnose_giveup(herdr, job_id, &pane_id, generic))
        }
        PollDecision::TimedOutCeiling => RunOutcome::TimedOutCeiling,
        PollDecision::PausedLimit => {
            stamp_paused_limit(&bee_dir, job_id, herdr, &pane_id);
            RunOutcome::PausedLimit
        }
        PollDecision::Died { pid } => RunOutcome::Died { pid },
        PollDecision::Blocked => {
            let tail = pane_tail(herdr, &pane_id);
            RunOutcome::PaneBlocked(blocked_message(job_id, &pane_id, &tail))
        }
        PollDecision::Continue => unreachable!("run_poll_loop only returns on a non-Continue decision"),
    };

    let valid_result = matches!(outcome, RunOutcome::Result(_));
    let close = !matches!(outcome, RunOutcome::PausedLimit) && should_close_pane(valid_result, opts.close_always);
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
        RunOutcome::TimedOutIdle(_) => "timed_out_idle",
        RunOutcome::TimedOutCeiling => "timed_out_ceiling",
        RunOutcome::PausedLimit => "paused_limit",
        RunOutcome::Died { .. } => "died",
        RunOutcome::PaneBlocked(_) => "pane_blocked",
    }
}

fn exit_code_for(o: &RunOutcome) -> ExitCode {
    match o {
        RunOutcome::DryRun(_) => ExitCode::SUCCESS,
        RunOutcome::Result(r) if r.status == MailboxStatus::Done => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

/// `<main_root>/.bee/config.json`, parsed — `Value::Null` when the file is
/// absent or unparseable. This is the fail-open read `execute_new` has always
/// done for the agent registry; `run`'s transport selection reads the SAME
/// file for `herding.tmux.*`, so the two share one helper instead of two
/// drifting copies.
fn read_main_config(main_root: &Path) -> Value {
    let cfg_path = main_root.join(".bee").join("config.json");
    std::fs::read_to_string(&cfg_path).ok().and_then(|raw| serde_json::from_str(&raw).ok()).unwrap_or(Value::Null)
}

/// tmux-herding-transport D1: the already-decided transport kind, built into
/// the one object `execute` runs against. Pure over `(kind, cfg)` — no config
/// read, no environment sniff, no process — so a test names a kind and reads
/// the answer back through `name()`.
fn select_transport(kind: TransportKind, cfg: &Value) -> Box<dyn PaneTransport> {
    match kind {
        TransportKind::Herdr => Box::new(RealHerdr),
        TransportKind::Tmux => Box::new(RealTmux::new(TmuxSettings::from_config(cfg))),
    }
}

/// `run`'s whole transport decision: read `herding.transport` out of the main
/// checkout's config, then build it. An illegal value comes back `Err` with
/// the message `transport_kind` wrote (it names both legal spellings), and
/// `run` refuses on it BEFORE the job file, the mailbox, or any pane split —
/// a typo'd transport must never half-start a worker.
fn transport_for_run(main_root: &Path) -> Result<Box<dyn PaneTransport>, String> {
    let kind = crate::herding::transport_kind_at(main_root)?;
    Ok(select_transport(kind, &read_main_config(main_root)))
}

fn emit_result(opts: &Options, result: &ExecResult, transport: &str) {
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
            RunOutcome::SpawnFailed(msg) => {
                m.insert("error".into(), Value::String(msg.clone()));
                m.insert(
                    "remedy".into(),
                    Value::String(spawn_failed_remedy(
                        opts.cell_id.as_deref(),
                        opts.agent.as_deref(),
                        result.pane_id.as_deref(),
                    )),
                );
            }
            RunOutcome::Malformed(msg) | RunOutcome::PaneBlocked(msg) => {
                m.insert("error".into(), Value::String(msg.clone()));
            }
            RunOutcome::ContinueRefused(refusal) => {
                m.insert("error".into(), Value::String(refusal.to_string()));
            }
            RunOutcome::DryRun(brief) => {
                m.insert("brief".into(), Value::String(brief.clone()));
                // tmux-herding-transport D1: additive, dry-run only — which
                // multiplexer the real run would have reached for.
                m.insert("transport".into(), Value::String(transport.to_string()));
                m.insert(
                    "job_path".into(),
                    Value::String(
                        mailbox::job_path(&opts.main_root.join(".bee"), &opts.job_id).display().to_string(),
                    ),
                );
            }
            RunOutcome::Died { pid } => {
                if let Some(p) = pid {
                    m.insert("pid".into(), Value::from(*p));
                }
            }
            RunOutcome::TimedOutIdle(msg) => {
                m.insert("error".into(), Value::String(msg.clone()));
            }
            RunOutcome::TimedOutCeiling | RunOutcome::PausedLimit => {}
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
    // tmux-herding-transport D1: the transport is chosen from config here,
    // before ANY side effect — an illegal `herding.transport` refuses with no
    // job file written and no pane split.
    let transport = match transport_for_run(&opts.main_root) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("bee herding run: {msg}");
            return ExitCode::FAILURE;
        }
    };
    let result = execute(&opts, transport.as_ref());
    let exit = exit_code_for(&result.outcome);
    emit_result(&opts, &result, transport.name());
    exit
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    // ─── pure decisions ─────────────────────────────────────────────────

    #[test]
    fn split_direction_is_right_only_when_the_parent_is_the_callers_own_pane() {
        // D2: the caller's own pane is the MAIN pane. Splitting it is what
        // creates the worker column, and it happens exactly once.
        assert_eq!(split_direction(true), "right");
        // Every other parent is already inside the worker column, so the
        // column grows downward instead of eating into the main pane.
        assert_eq!(split_direction(false), "down");
    }

    // ─── D2: the worker column's width — floor, cap, and the ratio ───────

    #[test]
    fn first_split_gives_the_worker_a_third_of_a_wide_parent_not_half() {
        // The live tab the decision was measured on: 173 columns. A third
        // is 57, under the 60-column floor, so the floor wins — and the
        // main pane keeps 113, far the larger share.
        let (child, ratio) = first_split_geometry(173);
        assert_eq!(child, 60);
        assert!((ratio - (113.0 / 173.0)).abs() < 1e-12, "ratio is the parent's keep-share: {ratio}");

        // A genuinely wide tab, where a third clears the floor on its own:
        // 240 columns hands the worker 80 and keeps 160.
        let (child, ratio) = first_split_geometry(240);
        assert_eq!(child, 80, "a third of 240, not half");
        assert!((ratio - (160.0 / 240.0)).abs() < 1e-12, "{ratio}");
    }

    #[test]
    fn the_sixty_column_floor_wins_where_a_third_would_be_too_thin() {
        // 150 columns: a third is 50, below the minimum a worker needs to
        // accept a submission at all, so the child is floored up to 60 and
        // the main pane still keeps the larger 90.
        let (child, ratio) = first_split_geometry(150);
        assert_eq!(child, MIN_PANE_WIDTH);
        assert!((ratio - (90.0 / 150.0)).abs() < 1e-12, "{ratio}");
        assert!(narrow_pane_refusal("w1:p1", child).is_none(), "a floored 60-column child is workable");
    }

    #[test]
    fn the_half_cap_holds_where_a_third_and_a_half_converge() {
        // 120 columns: a third is 40, floored to 60 — which is also exactly
        // half. The cap keeps the child from ever exceeding half, so the
        // two converge and neither side is starved.
        let (child, ratio) = first_split_geometry(120);
        assert_eq!(child, 60);
        assert!((ratio - 0.5).abs() < 1e-12, "an even split is the narrowest D2 ever allows: {ratio}");
        assert!(narrow_pane_refusal("w1:p1", child).is_none());

        // Below that, the cap wins over the floor and the child lands under
        // the minimum — the refusal fires and the fresh-tab path takes over.
        let (child, _) = first_split_geometry(100);
        assert_eq!(child, 50, "half of 100, capped — never the 60 the floor asked for");
        assert!(narrow_pane_refusal("w1:p1", child).is_some(), "a 50-column child must refuse");
    }

    // ─── D2/hps-13: main-pane-excluding parent + child-side width guard ──

    #[test]
    fn resolve_split_parent_never_picks_the_callers_own_pane_once_another_exists() {
        // D2's core: the caller's own pane is the LARGEST here by area, and
        // the retired hps-12 rule would have split it again. It is excluded
        // outright, so the roomiest WORKER pane takes the split instead.
        let panes = vec![
            PaneGeom { pane_id: "w1:p1".to_string(), width: 113, height: 50 },
            PaneGeom { pane_id: "w1:p2".to_string(), width: 60, height: 25 },
            PaneGeom { pane_id: "w1:p3".to_string(), width: 60, height: 12 },
        ];
        let parent = resolve_split_parent(Some(&panes), "w1:p1");
        assert_eq!(parent.pane_id, "w1:p2", "the main pane is never split twice");
        assert_eq!(parent.direction, "down");
        assert!((parent.ratio - 0.5).abs() < 1e-12, "a down split stays even: {}", parent.ratio);
        assert!(parent.refusal.is_none(), "a down split keeps the parent's 60 columns: {:?}", parent.refusal);
    }

    #[test]
    fn resolve_split_parent_stacks_a_band_instead_of_narrowing_a_populated_tab_again() {
        // Shaped like the live 60/30/15 measurement: the caller's own pane
        // is excluded, so the roomiest of the rest takes a "down" split,
        // which leaves its 60 columns untouched in the child.
        let panes = vec![
            PaneGeom { pane_id: "w1:p1".to_string(), width: 30, height: 43 },
            PaneGeom { pane_id: "w1:p2".to_string(), width: 60, height: 43 },
            PaneGeom { pane_id: "w1:p3".to_string(), width: 15, height: 43 },
        ];
        let parent = resolve_split_parent(Some(&panes), "w1:p1");
        assert_eq!(parent.pane_id, "w1:p2");
        assert_eq!(parent.direction, "down");
        assert!(parent.refusal.is_none(), "a down split keeps the parent's 60 columns: {:?}", parent.refusal);
    }

    #[test]
    fn resolve_split_parent_splits_the_callers_own_pane_only_when_it_is_alone() {
        // The live D5 shape: a 120x43 tab root, the caller's own and the
        // only pane. This is the one split it ever takes — "right", and its
        // child lands at exactly the 60-column minimum, workable.
        let panes = vec![PaneGeom { pane_id: "w1:p1".to_string(), width: 120, height: 43 }];
        let parent = resolve_split_parent(Some(&panes), "w1:p1");
        assert_eq!(parent.pane_id, "w1:p1");
        assert_eq!(parent.direction, "right");
        assert!((parent.ratio - 0.5).abs() < 1e-12, "{}", parent.ratio);
        assert!(parent.refusal.is_none(), "a 60-column right-split child is workable: {:?}", parent.refusal);
    }

    #[test]
    fn resolve_split_parent_carries_the_one_third_ratio_on_the_first_split_of_a_wide_tab() {
        // The uat tab: 173x50, one pane. The worker column is 60 wide and
        // the human keeps 113 — the whole point of D2.
        let panes = vec![PaneGeom { pane_id: "w1:p1".to_string(), width: 173, height: 50 }];
        let parent = resolve_split_parent(Some(&panes), "w1:p1");
        assert_eq!(parent.pane_id, "w1:p1");
        assert_eq!(parent.direction, "right");
        assert!((parent.ratio - (113.0 / 173.0)).abs() < 1e-12, "the main pane keeps 113: {}", parent.ratio);
        assert!(parent.ratio > 0.5, "the main pane must keep MORE than half: {}", parent.ratio);
        assert!(parent.refusal.is_none(), "{:?}", parent.refusal);
    }

    #[test]
    fn resolve_split_parent_a_down_split_never_narrows_so_the_parents_own_width_is_the_guard() {
        // A tab that already holds a worker pane: the parent is that worker
        // and the direction is "down", which leaves width unchanged — the
        // child is exactly as wide as the parent, so a 60-wide worker is
        // workable with no narrowing penalty.
        let panes = vec![
            PaneGeom { pane_id: "w1:p1".to_string(), width: 60, height: 90 },
            PaneGeom { pane_id: "w1:p2".to_string(), width: 60, height: 40 },
        ];
        let parent = resolve_split_parent(Some(&panes), "w1:p1");
        assert_eq!(parent.pane_id, "w1:p2");
        assert_eq!(parent.direction, "down");
        assert!(parent.refusal.is_none(), "a down split keeps width unchanged: {:?}", parent.refusal);
    }

    #[test]
    fn resolve_split_parent_falls_back_to_own_pane_when_the_layout_is_unreadable() {
        let parent = resolve_split_parent(None, "w1:p1");
        assert_eq!(parent.pane_id, "w1:p1");
        assert_eq!(parent.direction, "right");
        assert!((parent.ratio - 0.5).abs() < 1e-12, "a widthless fallback keeps the old even split");
        assert!(parent.refusal.is_none(), "an unreadable layout must never refuse: {:?}", parent.refusal);
    }

    #[test]
    fn resolve_split_parent_falls_back_to_own_pane_when_the_layout_names_no_candidate() {
        let parent = resolve_split_parent(Some(&[]), "w1:p1");
        assert_eq!(parent.pane_id, "w1:p1");
        assert_eq!(parent.direction, "right");
        assert!(parent.refusal.is_none());
    }

    #[test]
    fn resolve_split_parent_refuses_a_parent_below_the_minimum_width_naming_it() {
        // The caller's own pane, alone: a "right" split, and 15 columns cap
        // at a 7-column child — the width the refusal must name.
        let panes = vec![PaneGeom { pane_id: "w1:p3".to_string(), width: 15, height: 43 }];
        let parent = resolve_split_parent(Some(&panes), "w1:p3");
        assert_eq!(parent.pane_id, "w1:p3");
        let msg = parent.refusal.expect("a 7-column child is below the minimum and must refuse");
        assert!(msg.contains("w1:p3"), "{msg}");
        assert!(msg.contains("7"), "{msg}");
        assert!(msg.contains("60"), "{msg}");
    }

    #[test]
    fn narrow_pane_refusal_is_none_at_and_above_the_minimum() {
        assert!(narrow_pane_refusal("w1:p1", 60).is_none());
        assert!(narrow_pane_refusal("w1:p1", 120).is_none());
    }

    #[test]
    fn extract_pane_layout_reads_the_captured_live_reply() {
        // Captured from `herdr pane layout --pane w4:p4` against a clean
        // tab whose only pane was 120x43.
        let body = r#"{"result":{"layout":{"tab_id":"w4:t4","panes":[{"pane_id":"w4:p4","rect":{"height":43,"width":120,"x":36,"y":1}}],"splits":[]}}}"#;
        let v: Value = serde_json::from_str(body).unwrap();
        let panes = extract_pane_layout(&v).expect("captured reply parses");
        assert_eq!(panes, vec![PaneGeom { pane_id: "w4:p4".to_string(), width: 120, height: 43 }]);
    }

    #[test]
    fn extract_pane_layout_reads_multiple_sibling_panes() {
        let body = r#"{"result":{"layout":{"tab_id":"w4:t4","panes":[
            {"pane_id":"w4:p1","rect":{"height":43,"width":30,"x":0,"y":1}},
            {"pane_id":"w4:p2","rect":{"height":43,"width":60,"x":30,"y":1}}
        ],"splits":[]}}}"#;
        let v: Value = serde_json::from_str(body).unwrap();
        let panes = extract_pane_layout(&v).expect("captured reply parses");
        assert_eq!(panes.len(), 2);
        assert_eq!(panes[1].pane_id, "w4:p2");
        assert_eq!(panes[1].width, 60);
    }

    #[test]
    fn extract_pane_layout_is_none_on_the_old_shallow_path_without_the_layout_wrapper() {
        // Guards against regressing to the shape this cell fixed: `panes`
        // sitting straight under `result` (one level too shallow) must not
        // be picked up.
        let v: Value = serde_json::from_str(
            r#"{"result":{"panes":[{"pane_id":"w4:p4","rect":{"height":43,"width":120}}]}}"#,
        )
        .unwrap();
        assert_eq!(extract_pane_layout(&v), None);
    }

    #[test]
    fn extract_tab_create_root_pane_reads_the_captured_live_reply() {
        // Captured from `herdr tab create --workspace w4 --cwd <path>
        // --label bee-probe`, trimmed.
        let body = r#"{"id":"cli:tab:create","result":{"root_pane":{"pane_id":"w4:p31","cwd":"<path>","tab_id":"w4:tE","workspace_id":"w4"},"tab":{"tab_id":"w4:tE"}}}"#;
        let v: Value = serde_json::from_str(body).unwrap();
        assert_eq!(extract_tab_create_root_pane(&v), Some("w4:p31".to_string()));
    }

    #[test]
    fn extract_tab_create_root_pane_is_none_when_root_pane_is_missing() {
        let v: Value = serde_json::from_str(r#"{"result":{"tab":{"tab_id":"w4:tE"}}}"#).unwrap();
        assert_eq!(extract_tab_create_root_pane(&v), None);
    }

    #[test]
    fn pane_workspace_splits_at_the_first_colon() {
        assert_eq!(pane_workspace("w4:p31"), "w4");
        assert_eq!(pane_workspace("no-colon"), "no-colon");
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
        assert_eq!(decide_poll(0, 0, 0, 1, 1, true, None, None), PollDecision::ResultReady);
    }

    #[test]
    fn decide_poll_extends_on_a_fresh_heartbeat() {
        assert_eq!(decide_poll(5_000, 0, 5_000, 60, 3_600, false, None, None), PollDecision::Continue);
    }

    #[test]
    fn decide_poll_times_out_idle_when_the_heartbeat_goes_stale() {
        assert_eq!(decide_poll(61_000, 0, 0, 60, 3_600, false, None, None), PollDecision::TimedOutIdle);
    }

    #[test]
    fn decide_poll_ceiling_caps_even_with_a_fresh_heartbeat() {
        // last heartbeat one second ago (well inside a 60s idle timeout),
        // but the run has now been alive for the full 3600s ceiling — the
        // ceiling caps regardless of activity.
        assert_eq!(decide_poll(3_600_000, 0, 3_599_000, 60, 3_600, false, None, None), PollDecision::TimedOutCeiling);
    }

    #[test]
    fn decide_poll_pauses_on_limit_when_heartbeat_stale_and_pane_matches_limit_pattern() {
        assert_eq!(
            decide_poll(61_000, 0, 0, 60, 3_600, false, Some("You've hit your session limit · resets 6:20pm"), None),
            PollDecision::PausedLimit
        );
        assert_eq!(
            decide_poll(61_000, 0, 0, 60, 3_600, false, Some("warning: usage limit reached"), None),
            PollDecision::PausedLimit
        );
    }

    #[test]
    fn decide_poll_keeps_waiting_with_fresh_heartbeat_even_if_pane_mentions_limit() {
        assert_eq!(
            decide_poll(5_000, 0, 5_000, 60, 3_600, false, Some("hit your session limit"), None),
            PollDecision::Continue
        );
    }

    #[test]
    fn decide_poll_times_out_idle_when_heartbeat_stale_and_pane_does_not_match_limit() {
        assert_eq!(
            decide_poll(61_000, 0, 0, 60, 3_600, false, Some("compilation error: mismatched types"), None),
            PollDecision::TimedOutIdle
        );
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
            |_| {
                ticks += 1;
                PollTick { result_ready: ticks >= 3, heartbeat_fresh: false, pane_text: None, liveness: None, blocked: false }
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
            |_| PollTick { result_ready: false, heartbeat_fresh: false, pane_text: None, liveness: None, blocked: false },
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
            |_| PollTick { result_ready: false, heartbeat_fresh: true, pane_text: None, liveness: None, blocked: false },
            |_| {},
            || {
                clock += 1_000;
                clock
            },
        );
        assert_eq!(decision, PollDecision::TimedOutCeiling);
    }

    #[test]
    fn run_poll_loop_passes_stale_flag_false_on_fresh_ticks_and_true_when_stale() {
        let mut flags = Vec::new();
        let mut clock = 0i64;
        let decision = run_poll_loop(
            0,
            5,
            3_600,
            Duration::from_millis(0),
            |stale| {
                flags.push(stale);
                PollTick {
                    result_ready: flags.len() >= 7,
                    heartbeat_fresh: flags.len() <= 2,
                    pane_text: None,
                    liveness: None,
                    blocked: false,
                }
            },
            |_| {},
            || {
                clock += 1_000;
                clock
            },
        );
        assert_eq!(decision, PollDecision::ResultReady);
        assert_eq!(flags, vec![false, false, false, false, false, false, true]);
    }

    #[test]
    fn run_poll_loop_ends_at_once_on_a_blocked_tick_without_burning_the_idle_timeout() {
        // D3: a blocked observation ends the round poll immediately — a
        // huge idle timeout and ceiling are never burned on a question
        // nobody is going to answer.
        let mut ticks = 0u32;
        let mut clock = 0i64;
        let decision = run_poll_loop(
            0,
            900,
            21_600,
            Duration::from_millis(0),
            |_| {
                ticks += 1;
                PollTick { result_ready: false, heartbeat_fresh: false, pane_text: None, liveness: None, blocked: true }
            },
            |_| {},
            || {
                clock += 1_000;
                clock
            },
        );
        assert_eq!(decision, PollDecision::Blocked);
        assert_eq!(ticks, 1, "blocked ends the wait on the very first tick");
    }

    #[test]
    fn run_poll_loop_lets_a_same_tick_result_win_over_blocked() {
        // A completed round already trumps a stale/simultaneous blocked
        // read — the work finished, so ResultReady wins.
        let mut clock = 0i64;
        let decision = run_poll_loop(
            0,
            900,
            21_600,
            Duration::from_millis(0),
            |_| PollTick { result_ready: true, heartbeat_fresh: false, pane_text: None, liveness: None, blocked: true },
            |_| {},
            || {
                clock += 1_000;
                clock
            },
        );
        assert_eq!(decision, PollDecision::ResultReady);
    }

    // ─── hps-7: prompt diagnosis, pure (D5) ────────────────────────────

    #[test]
    fn find_prompt_diagnosis_finds_a_short_hint_line_that_survives_clipping_where_the_question_would_not() {
        // The live acceptance probe's exact shape: an eight-column-wide
        // capture clips the question itself down to unmatchable fragments,
        // but a short confirmation hint on its OWN line needs no clipping
        // margin at all and still lands whole.
        let clipped_pane = "Do you t\nrust wor\n(y/n)\n";
        assert!(
            !clipped_pane.to_lowercase().contains("do you trust this folder"),
            "the clipped capture must not contain the long needle — that is the whole premise"
        );
        assert_eq!(find_prompt_diagnosis(clipped_pane), Some("(y/n)".to_string()));
    }

    #[test]
    fn find_prompt_diagnosis_matches_the_navigation_arrows() {
        assert_eq!(find_prompt_diagnosis("choose one:\n↑/↓ to move, enter to select\n"), Some("↑/↓ to move, enter to select".to_string()));
    }

    #[test]
    fn find_prompt_diagnosis_matches_a_selection_caret() {
        assert_eq!(find_prompt_diagnosis("  yes\n❯ no\n"), Some("❯ no".to_string()));
    }

    #[test]
    fn find_prompt_diagnosis_matches_a_line_ending_in_a_question_mark() {
        assert_eq!(find_prompt_diagnosis("some banner\nContinue anyway?\n"), Some("Continue anyway?".to_string()));
    }

    #[test]
    fn find_prompt_diagnosis_is_case_insensitive_on_its_pattern_table() {
        assert_eq!(find_prompt_diagnosis("Overwrite the file [Y/n]\n"), Some("Overwrite the file [Y/n]".to_string()));
    }

    #[test]
    fn find_prompt_diagnosis_returns_none_on_ordinary_output() {
        assert_eq!(find_prompt_diagnosis("compiling…\nrunning tests\nall green\n"), None);
    }

    #[test]
    fn find_prompt_diagnosis_returns_none_on_empty_text() {
        assert_eq!(find_prompt_diagnosis(""), None);
    }

    // ─── hps-10: real herdr reply parsing, no process spawned ───────────
    //
    // These feed bodies captured live from a real `herdr` binary through
    // the exact extraction `RealHerdr` uses, so a parse bug (like the one
    // that shipped here — `agent_wait` reading `result.agent_status`
    // instead of `result.agent.agent_status`) fails a test instead of
    // hanging every `bee herding run` for 60s. `FakeHerdr::agent_wait`
    // below returns whatever a test configures, so it exercises the seam,
    // never the parse — these are the only tests that do.

    #[test]
    fn extract_agent_wait_status_reads_the_captured_live_reply() {
        // Captured from `herdr agent wait accept-solo-1 --until idle
        // --until done --timeout 200` against a real, healthy pane.
        let body = r#"{"id":"cli:agent:wait","result":{"agent":{"agent":"agy","agent_status":"done","interactive_ready":true,"name":"accept-solo-1","pane_id":"w4:p2J"},"type":"agent_info"}}"#;
        let v: Value = serde_json::from_str(body).expect("captured reply is valid JSON");
        assert_eq!(extract_agent_wait_status(&v), Some("done".to_string()));
    }

    #[test]
    fn extract_agent_wait_status_is_none_when_the_status_is_missing() {
        let v: Value = serde_json::from_str(r#"{"id":"cli:agent:wait","result":{"agent":{"name":"accept-solo-1"}}}"#).unwrap();
        assert_eq!(extract_agent_wait_status(&v), None);
    }

    #[test]
    fn extract_agent_wait_status_is_none_on_the_old_wrong_shallow_path() {
        // Guards against regressing to the shape this cell fixed: a status
        // sitting at `result.agent_status` (one level too shallow) must
        // NOT be picked up by the real extraction.
        let v: Value = serde_json::from_str(r#"{"result":{"agent_status":"done"}}"#).unwrap();
        assert_eq!(extract_agent_wait_status(&v), None);
    }

    #[test]
    fn parse_herdr_body_accepts_the_captured_agent_prompt_success_envelope() {
        // Captured from `herdr agent prompt <job> <text> --wait --until
        // working --timeout 5000` against a real, healthy pane.
        let body = br#"{"id":"cli:agent:prompt","result":{"agent":{"agent":"agy","agent_status":"working","name":"accept-solo-1"},"type":"agent_info"}}"#;
        assert!(parse_herdr_body("agent prompt", body).is_ok());
    }

    #[test]
    fn parse_herdr_body_rejects_non_json() {
        assert!(parse_herdr_body("agent wait", b"not json").is_err());
    }

    // ─── the PaneTransport seam ─────────────────────────────────────────

    struct FakeHerdr {
        own_pane: &'static str,
        /// hps-12: `pane_layout`'s stand-in — the full tab's pane list, or
        /// `None` for "layout unreadable". Defaults to a single entry for
        /// `own_pane`, matching the old fixed-rect default.
        layout: Option<Vec<PaneGeom>>,
        split_result: Result<String, String>,
        /// Every `pane_split(pane_id, direction, ratio, _)` call, in
        /// order — the seam the "split call receives the chosen parent"
        /// tests read (hps-12's parent choice, D2's ratio).
        split_calls: RefCell<Vec<(String, String, f64)>>,
        /// hps-13: what `tab_create` returns — the new tab's root pane id,
        /// or an error to prove the "refuse only when the fresh tab ALSO
        /// fails" path.
        tab_create_result: Result<String, String>,
        /// Every `tab_create(workspace, cwd, label)` call, in order — the
        /// seam an hps-13 test reads to prove the fallback fired (and, on
        /// the happy path, that no `pane_split` call went with it).
        tab_create_calls: RefCell<Vec<(String, String, String)>>,
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
        /// The `kind` (herdr `--kind`) `agent_start` was last called with —
        /// how a herd-registry test proves a named `--agent` resolution
        /// reached the spawn, not merely that SOME start happened.
        started_kind: RefCell<Option<String>>,
        /// The trailing args `agent_start` was last called with — how a D3
        /// override test proves the CONFIG entry's argv reached the spawn,
        /// not merely that the same-kind built-in did (`started_kind` alone
        /// cannot tell those apart when both name the same herdr kind).
        started_args: RefCell<Option<Vec<String>>>,
        /// Every `agent_wait(job_id, timeout_ms)` call, in order — proves
        /// the ready gate uses herdr's settle-aware wait (herding-prompt-stall
        /// D2), not a raw `agent_status` read. Its RETURN VALUE proxies
        /// `status` (below) unless a test wants a different value, since
        /// `agent_wait`'s fail-safe shape mirrors `agent_status`'s.
        agent_wait_calls: RefCell<Vec<(String, u64)>>,
        /// D4: what `pane_run` returns — `Ok(())` unless a test overrides
        /// it to prove the env-export-failure path.
        pane_run_result: Result<(), String>,
        /// Every `pane_run(pane_id, command)` call, in order — a D4 test
        /// reads this to prove the export line's exact content and that it
        /// went to the newly split pane.
        pane_run_calls: RefCell<Vec<(String, String)>>,
        /// D4: `pane_run` and `agent_start` calls, in order, tagged by
        /// name — the seam a D4 ordering test reads to prove the export
        /// line was sent BEFORE `agent_start`, never after.
        call_log: RefCell<Vec<&'static str>>,
        pane_text: RefCell<Option<String>>,
        /// hps-7: when set, every `pane_read` call returns this error
        /// instead of `pane_text` — the seam a "pane_read failure keeps the
        /// generic message" test uses.
        pane_read_err: RefCell<Option<String>>,
        pane_read_calls: RefCell<Vec<String>>,
        liveness_responses: RefCell<Vec<Liveness>>,
        process_info_calls: RefCell<Vec<String>>,
    }

    impl FakeHerdr {
        fn new() -> Self {
            FakeHerdr {
                own_pane: "w1:p1",
                // hps-13: 120x43 — the same roomy tab shape the live D5
                // probe measured, whose "right" split yields a 60-column
                // child (the minimum, workable) rather than a refusal.
                layout: Some(vec![PaneGeom { pane_id: "w1:p1".to_string(), width: 120, height: 43 }]),
                split_result: Ok("w1:p2".to_string()),
                split_calls: RefCell::new(Vec::new()),
                tab_create_result: Ok("w9:p1".to_string()),
                tab_create_calls: RefCell::new(Vec::new()),
                start_result: Ok(()),
                prompt_result: Ok(()),
                status: RefCell::new(Some("idle".to_string())),
                closed: RefCell::new(Vec::new()),
                alive_panes: RefCell::new(vec!["w1:p2".to_string()]),
                prompt_calls: RefCell::new(Vec::new()),
                start_calls: RefCell::new(Vec::new()),
                started_kind: RefCell::new(None),
                started_args: RefCell::new(None),
                agent_wait_calls: RefCell::new(Vec::new()),
                pane_run_result: Ok(()),
                pane_run_calls: RefCell::new(Vec::new()),
                call_log: RefCell::new(Vec::new()),
                pane_text: RefCell::new(None),
                pane_read_err: RefCell::new(None),
                pane_read_calls: RefCell::new(Vec::new()),
                liveness_responses: RefCell::new(Vec::new()),
                process_info_calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl PaneTransport for FakeHerdr {
        fn pane_current(&self) -> Result<String, String> {
            Ok(self.own_pane.to_string())
        }
        fn pane_layout(&self, _pane_id: &str) -> Option<Vec<PaneGeom>> {
            self.layout.clone()
        }
        fn pane_split(&self, pane_id: &str, direction: &str, ratio: f64, _cwd: &Path) -> Result<String, String> {
            self.split_calls.borrow_mut().push((pane_id.to_string(), direction.to_string(), ratio));
            self.split_result.clone()
        }
        fn tab_create(&self, workspace: &str, cwd: &Path, label: &str) -> Result<String, String> {
            self.tab_create_calls.borrow_mut().push((
                workspace.to_string(),
                cwd.display().to_string(),
                label.to_string(),
            ));
            self.tab_create_result.clone()
        }
        fn pane_run(&self, pane_id: &str, command: &str) -> Result<(), String> {
            self.pane_run_calls.borrow_mut().push((pane_id.to_string(), command.to_string()));
            self.call_log.borrow_mut().push("pane_run");
            self.pane_run_result.clone()
        }
        fn agent_start(
            &self,
            job_id: &str,
            kind: &str,
            _pane_id: &str,
            args: &[String],
        ) -> Result<(), String> {
            self.start_calls.borrow_mut().push(job_id.to_string());
            self.call_log.borrow_mut().push("agent_start");
            *self.started_kind.borrow_mut() = Some(kind.to_string());
            *self.started_args.borrow_mut() = Some(args.to_vec());
            self.start_result.clone()
        }
        fn agent_status(&self, _job_id: &str) -> Option<String> {
            self.status.borrow().clone()
        }
        fn pane_close(&self, pane_id: &str) -> Result<(), String> {
            self.closed.borrow_mut().push(pane_id.to_string());
            Ok(())
        }
        fn agent_prompt(&self, job_id: &str, prompt: &str, _until: &str, _timeout_ms: u64) -> Result<(), String> {
            self.prompt_calls.borrow_mut().push((job_id.to_string(), prompt.to_string()));
            self.prompt_result.clone()
        }
        fn agent_wait(&self, job_id: &str, timeout_ms: u64) -> Option<String> {
            self.agent_wait_calls.borrow_mut().push((job_id.to_string(), timeout_ms));
            self.status.borrow().clone()
        }
        fn pane_alive(&self, pane_id: &str) -> bool {
            self.alive_panes.borrow().iter().any(|p| p == pane_id)
        }
        fn pane_read(&self, pane_id: &str) -> Result<String, String> {
            self.pane_read_calls.borrow_mut().push(pane_id.to_string());
            if let Some(err) = self.pane_read_err.borrow().clone() {
                return Err(err);
            }
            Ok(self.pane_text.borrow().clone().unwrap_or_default())
        }
        fn process_info(&self, pane_id: &str) -> Liveness {
            self.process_info_calls.borrow_mut().push(pane_id.to_string());
            if !self.liveness_responses.borrow().is_empty() {
                self.liveness_responses.borrow_mut().remove(0)
            } else {
                Liveness::Unknown
            }
        }
    }

    // ─── hss-2: the pane split, serialized across processes ─────────────

    /// A `PaneTransport` whose pane list is SHARED and MUTABLE — the stand-in for
    /// the one real terminal layout that concurrent `bee herding run`
    /// processes all read and all change. `pane_split` pushes the pane it
    /// creates into that list, so the NEXT `pane_layout` read counts it;
    /// the short sleep before the push is the race window an unserialized
    /// split falls into (both racers read `pane_count` 1, both answer
    /// "right"). Everything sits behind a `Mutex` instead of `FakeHerdr`'s
    /// `RefCell` so the fake is `Sync` and two threads can share it.
    struct SharedLayoutHerdr {
        panes: std::sync::Mutex<Vec<PaneGeom>>,
        /// The direction of every completed split, in the order the new
        /// pane actually appeared.
        directions: std::sync::Mutex<Vec<String>>,
        next_pane: std::sync::atomic::AtomicUsize,
    }

    impl SharedLayoutHerdr {
        fn new(root_width: u64, root_height: u64) -> Self {
            SharedLayoutHerdr {
                panes: std::sync::Mutex::new(vec![PaneGeom {
                    pane_id: "w1:p1".to_string(),
                    width: root_width,
                    height: root_height,
                }]),
                directions: std::sync::Mutex::new(Vec::new()),
                next_pane: std::sync::atomic::AtomicUsize::new(2),
            }
        }

        fn directions(&self) -> Vec<String> {
            self.directions.lock().expect("directions").clone()
        }

        fn pane(&self, pane_id: &str) -> PaneGeom {
            self.panes
                .lock()
                .expect("panes")
                .iter()
                .find(|p| p.pane_id == pane_id)
                .cloned()
                .unwrap_or_else(|| panic!("no such pane {pane_id}"))
        }

        fn panes(&self) -> Vec<PaneGeom> {
            self.panes.lock().expect("panes").clone()
        }
    }

    impl PaneTransport for SharedLayoutHerdr {
        fn pane_current(&self) -> Result<String, String> {
            Ok("w1:p1".to_string())
        }
        fn pane_layout(&self, _pane_id: &str) -> Option<Vec<PaneGeom>> {
            Some(self.panes.lock().expect("panes").clone())
        }
        fn pane_split(&self, pane_id: &str, direction: &str, ratio: f64, _cwd: &Path) -> Result<String, String> {
            // Creating a pane takes real time, and only once herdr returns
            // does the new pane show up in a layout read. Without the lock,
            // both racers spend this window on the SAME stale read.
            std::thread::sleep(Duration::from_millis(120));
            let id = format!(
                "w1:p{}",
                self.next_pane.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            );
            let mut panes = self.panes.lock().expect("panes");
            let idx = panes
                .iter()
                .position(|p| p.pane_id == pane_id)
                .ok_or_else(|| format!("no such pane {pane_id}"))?;
            // herdr's own arithmetic, measured live against 0.8.0: `ratio`
            // is the share the PARENT KEEPS, rounded to whole cells, and
            // the child takes the remainder. A `right` split divides the
            // width, a `down` split the height; the other dimension is
            // untouched.
            let split_cells = |dim: u64| -> (u64, u64) {
                let kept = (dim as f64 * ratio).round() as u64;
                let kept = kept.min(dim);
                (kept, dim - kept)
            };
            let child = if direction == "right" {
                let (kept, given) = split_cells(panes[idx].width);
                panes[idx].width = kept;
                PaneGeom { pane_id: id.clone(), width: given, height: panes[idx].height }
            } else {
                let (kept, given) = split_cells(panes[idx].height);
                panes[idx].height = kept;
                PaneGeom { pane_id: id.clone(), width: panes[idx].width, height: given }
            };
            panes.push(child);
            self.directions.lock().expect("directions").push(direction.to_string());
            Ok(id)
        }
        fn tab_create(&self, _workspace: &str, _cwd: &Path, _label: &str) -> Result<String, String> {
            panic!("the hss-2 fake tab is roomy enough that no width refusal can fire")
        }
        fn pane_run(&self, _pane_id: &str, _command: &str) -> Result<(), String> {
            unreachable!("split_worker_pane never runs a command")
        }
        fn agent_start(
            &self,
            _job_id: &str,
            _kind: &str,
            _pane_id: &str,
            _args: &[String],
        ) -> Result<(), String> {
            unreachable!("split_worker_pane never starts an agent")
        }
        fn agent_status(&self, _job_id: &str) -> Option<String> {
            unreachable!("split_worker_pane never reads a status")
        }
        fn pane_close(&self, _pane_id: &str) -> Result<(), String> {
            unreachable!("split_worker_pane never closes a pane")
        }
        fn agent_prompt(&self, _job_id: &str, _prompt: &str, _until: &str, _timeout_ms: u64) -> Result<(), String> {
            unreachable!("split_worker_pane never prompts")
        }
        fn agent_wait(&self, _job_id: &str, _timeout_ms: u64) -> Option<String> {
            unreachable!("split_worker_pane never waits on an agent")
        }
        fn pane_alive(&self, _pane_id: &str) -> bool {
            unreachable!("split_worker_pane never probes pane liveness")
        }
        fn pane_read(&self, _pane_id: &str) -> Result<String, String> {
            unreachable!("split_worker_pane never reads a pane")
        }
        fn process_info(&self, _pane_id: &str) -> Liveness {
            unreachable!("split_worker_pane never reads process info")
        }
    }

    /// A private temp main root per test — the split lock lives under
    /// `<root>/.bee/locks`, so each test needs its own.
    fn hss_temp_root(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "bee-herding-split-serialize-{}-{}-{}",
            tag,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).expect("temp root");
        dir
    }

    /// The exact live regression (2026-08-21): five concurrent spawns from
    /// one tab ALL split `right` (panes p3R p3S p3T p3V p3W) and worker 5
    /// died on the 180s ack wait, because every process read a layout still
    /// showing `pane_count` 1. Two threads over one shared layout and one
    /// shared main root: with the lock held across the layout read AND the
    /// split, the second thread's read already counts the first thread's
    /// pane, so the directions are `right` then `down`. Drop the lock and
    /// this asserts `["right", "right"]` instead, and fails.
    #[test]
    fn concurrent_splits_are_serialized_so_the_second_one_stacks_a_band() {
        let root = hss_temp_root("concurrent");
        let herdr = SharedLayoutHerdr::new(240, 40);
        let budget = Duration::from_secs(20);
        std::thread::scope(|s| {
            let a = s.spawn(|| split_worker_pane(&herdr, "w1:p1", &root, "job-a", &root, budget));
            let b = s.spawn(|| split_worker_pane(&herdr, "w1:p1", &root, "job-b", &root, budget));
            a.join().expect("thread a").expect("first split must succeed");
            b.join().expect("thread b").expect("second split must succeed");
        });
        assert_eq!(
            herdr.directions(),
            vec!["right".to_string(), "down".to_string()],
            "two concurrent splits must be one right and one down, never two rights"
        );
        assert!(
            !root.join(".bee").join("locks").join("herding-pane-split.lock").exists(),
            "both guards must have released the lock file"
        );
        // hss-3/D2, on top of the hss-2 serialization: the caller's own pane
        // keeps its FULL height through both spawns, and the second worker
        // landed inside the first worker's column, not beside the human.
        let own = herdr.pane("w1:p1");
        assert_eq!(own.height, 40, "the main pane is never split downward");
        assert_eq!(own.width, 160, "the main pane keeps two thirds of 240");
        for worker in herdr.panes().iter().filter(|p| p.pane_id != "w1:p1") {
            assert_eq!(worker.width, 80, "every worker sits in the one 80-column column: {worker:?}");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The uat regression this cell exists for (herding-split-serialize D2):
    /// five spawns over one 173x50 tab. Before D2 the human's own pane fell
    /// from 50 rows to 13 while every worker kept 25. Now the main pane is
    /// split exactly once — a 60-column worker column on its right — and
    /// every later worker stacks DOWN inside that column, so the caller's
    /// pane stays 113x50 from first spawn to last.
    #[test]
    fn five_spawns_leave_the_callers_pane_whole_and_stack_every_worker_in_one_column() {
        let root = hss_temp_root("five-spawns");
        let herdr = SharedLayoutHerdr::new(173, 50);
        let budget = Duration::from_secs(20);
        for n in 1..=5 {
            split_worker_pane(&herdr, "w1:p1", &root, &format!("job-{n}"), &root, budget)
                .unwrap_or_else(|e| panic!("spawn {n} must succeed: {e}"));
        }

        assert_eq!(
            herdr.directions(),
            vec![
                "right".to_string(),
                "down".to_string(),
                "down".to_string(),
                "down".to_string(),
                "down".to_string(),
            ],
            "the main pane is split exactly once; every later worker stacks down"
        );
        let own = herdr.pane("w1:p1");
        assert_eq!(own.height, 50, "the caller's pane keeps its FULL height through all five spawns");
        assert_eq!(own.width, 113, "and the larger share of the width");

        let workers: Vec<PaneGeom> = herdr.panes().into_iter().filter(|p| p.pane_id != "w1:p1").collect();
        assert_eq!(workers.len(), 5, "five spawns, five worker panes");
        for worker in &workers {
            assert_eq!(worker.width, MIN_PANE_WIDTH, "every worker lands in the 60-column right-hand column: {worker:?}");
        }
        // The column's heights partition the tab's 50 rows exactly — the
        // workers stack, they never overlap the main pane.
        assert_eq!(workers.iter().map(|w| w.height).sum::<u64>(), 50);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Fail OPEN on a busy budget: a lock the caller cannot take inside its
    /// wait budget warns and splits anyway. A spawn that never happens is
    /// worse than a mis-directed one.
    #[test]
    fn a_busy_split_lock_still_splits_once_the_budget_is_spent() {
        let root = hss_temp_root("busy");
        // A live-pid, fresh-mtime holder: never stale, so the budget runs
        // out with the lock still held.
        let held = split_lock::acquire(&root, "job-holder", Duration::from_millis(50))
            .expect("holder acquire")
            .expect("holder holds");
        let herdr = SharedLayoutHerdr::new(240, 40);
        let pane = split_worker_pane(
            &herdr,
            "w1:p1",
            &root,
            "job-queued",
            &root,
            Duration::from_millis(150),
        )
        .expect("a busy lock must never fail the spawn");
        assert_eq!(pane, "w1:p2");
        assert_eq!(herdr.directions(), vec!["right".to_string()]);
        drop(held);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Fail OPEN on a broken lock: a main root whose `.bee` is a FILE means
    /// the locks directory can never be created, so `acquire` returns
    /// `Err`. The split still happens.
    #[test]
    fn an_unusable_lock_directory_still_splits() {
        let root = hss_temp_root("unusable");
        std::fs::write(root.join(".bee"), b"not a directory").expect("write .bee as a file");
        let herdr = SharedLayoutHerdr::new(240, 40);
        let pane = split_worker_pane(
            &herdr,
            "w1:p1",
            &root,
            "job-broken",
            &root,
            Duration::from_millis(150),
        )
        .expect("a broken lock must never fail the spawn");
        assert_eq!(pane, "w1:p2");
        assert_eq!(herdr.directions(), vec!["right".to_string()]);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Every method panics — proves a code path never touches `PaneTransport` at
    /// all (used for `--dry-run`: D1's "spawns no process" claim).
    struct PanicHerdr;
    impl PaneTransport for PanicHerdr {
        fn pane_current(&self) -> Result<String, String> {
            panic!("dry-run must never call PaneTransport::pane_current")
        }
        fn pane_layout(&self, _pane_id: &str) -> Option<Vec<PaneGeom>> {
            panic!("dry-run must never call PaneTransport::pane_layout")
        }
        fn pane_split(&self, _pane_id: &str, _direction: &str, _ratio: f64, _cwd: &Path) -> Result<String, String> {
            panic!("dry-run must never call PaneTransport::pane_split")
        }
        fn tab_create(&self, _workspace: &str, _cwd: &Path, _label: &str) -> Result<String, String> {
            panic!("dry-run must never call PaneTransport::tab_create")
        }
        fn pane_run(&self, _pane_id: &str, _command: &str) -> Result<(), String> {
            panic!("dry-run must never call PaneTransport::pane_run")
        }
        fn agent_start(
            &self,
            _job_id: &str,
            _kind: &str,
            _pane_id: &str,
            _args: &[String],
        ) -> Result<(), String> {
            panic!("dry-run must never call PaneTransport::agent_start")
        }
        fn agent_status(&self, _job_id: &str) -> Option<String> {
            panic!("dry-run must never call PaneTransport::agent_status")
        }
        fn pane_close(&self, _pane_id: &str) -> Result<(), String> {
            panic!("dry-run must never call PaneTransport::pane_close")
        }
        fn agent_prompt(&self, _job_id: &str, _prompt: &str, _until: &str, _timeout_ms: u64) -> Result<(), String> {
            panic!("dry-run must never call PaneTransport::agent_prompt")
        }
        fn agent_wait(&self, _job_id: &str, _timeout_ms: u64) -> Option<String> {
            panic!("dry-run must never call PaneTransport::agent_wait")
        }
        fn pane_alive(&self, _pane_id: &str) -> bool {
            panic!("dry-run must never call PaneTransport::pane_alive")
        }
        fn pane_read(&self, _pane_id: &str) -> Result<String, String> {
            panic!("dry-run must never call PaneTransport::pane_read")
        }
        fn process_info(&self, _pane_id: &str) -> Liveness {
            panic!("dry-run must never call PaneTransport::process_info")
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
            agent: None,
            expertise: Vec::new(),
            has_explicit_expertise: false,
            nickname: "job-1".to_string(),
            cell_id: None,
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

    /// Writes an empty round-N ack file into the job's mailbox — enough for
    /// `deliver_pointer`'s `ack_present()` check to see delivery as already
    /// confirmed in a single send, so a test exercising `wait_for_round`'s
    /// OWN timing (not delivery) never trips the bounded-resend loop's real
    /// sleep (herding-prompt-stall D4).
    fn seed_ack(main_root: &Path, job_id: &str, round: u32) {
        let bee_dir = main_root.join(".bee");
        let dir = mailbox::mailbox_dir(&bee_dir, job_id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(mailbox::ack_path(&bee_dir, job_id, round), "{}").unwrap();
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
            agent: None,
            expertise: Vec::new(),
            has_explicit_expertise: false,
            nickname: "job-1".to_string(),
            cell_id: None,
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

    // ─── tmux-herding-transport D1: which transport `run` builds ─────────

    #[test]
    fn transport_select_herdr_kind_builds_the_herdr_transport() {
        // The absent-key default. `herding.tmux.*` may even be present —
        // the KIND decides, nothing else.
        let cfg = serde_json::json!({"herding": {"tmux": {"quiet_cycles": 9}}});
        assert_eq!(select_transport(TransportKind::Herdr, &cfg).name(), "herdr");
        assert_eq!(select_transport(TransportKind::Herdr, &Value::Null).name(), "herdr");
    }

    #[test]
    fn transport_select_tmux_kind_builds_the_tmux_transport() {
        let cfg = serde_json::json!({"herding": {"transport": "tmux"}});
        assert_eq!(select_transport(TransportKind::Tmux, &cfg).name(), "tmux");
    }

    #[test]
    fn transport_select_run_refuses_an_illegal_value_before_any_split() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        std::fs::create_dir_all(&bee_dir).unwrap();
        std::fs::write(bee_dir.join("config.json"), r#"{"herding":{"transport":"nope"}}"#).unwrap();

        // The message `run` prints on stderr, read through the same helper
        // `run` calls: it names BOTH legal spellings so the typo is fixable
        // from the refusal alone.
        let msg = match transport_for_run(tmp.path()) {
            Err(m) => m,
            Ok(t) => panic!("an illegal transport must refuse, got {}", t.name()),
        };
        assert!(msg.contains("herdr"), "refusal must name herdr: {msg}");
        assert!(msg.contains("tmux"), "refusal must name tmux: {msg}");

        // And the verb itself fails before ANY side effect — no job file, no
        // mailbox, and (by construction, since `execute` never runs) no split.
        let root = tmp.path().display().to_string();
        let exit = run(&["--task", "do the thing", "--job-id", "job-1", "--main-root", &root, "--json"]);
        assert_eq!(exit, ExitCode::FAILURE);
        assert!(!bee_dir.join("mailbox").exists(), "the refusal must not create the mailbox tree");
        assert!(!mailbox::job_path(&bee_dir, "job-1").exists(), "the refusal must not write job.json");
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
    fn a_fresh_spawn_splits_the_roomiest_sibling_pane_not_the_callers_own() {
        let tmp = tempfile::tempdir().unwrap();
        let opts = test_options(tmp.path(), false);
        let mut fake = FakeHerdr::new();
        // Caller's own pane is down to 30; a sibling worker pane holds 130.
        // D2 excludes the caller's own pane outright, so the split targets
        // the sibling and goes "down", and the child keeps all 130
        // columns — hps-13's roomy-tab case, with no narrowing at all.
        fake.layout = Some(vec![
            PaneGeom { pane_id: "w1:p1".to_string(), width: 30, height: 43 },
            PaneGeom { pane_id: "w1:p9".to_string(), width: 130, height: 43 },
        ]);
        seeded_result_dir(&tmp.path().join(".bee"), &opts.job_id);

        let result = execute(&opts, &fake);

        assert!(matches!(result.outcome, RunOutcome::Result(_)), "got {:?}", result.outcome);
        assert_eq!(
            fake.split_calls.borrow().as_slice(),
            [("w1:p9".to_string(), "down".to_string(), DOWN_SPLIT_RATIO)],
            "the split must target the roomiest sibling, not the caller's own w1:p1"
        );
        assert!(fake.tab_create_calls.borrow().is_empty(), "a roomy tab must never open a fresh one");
    }

    #[test]
    fn a_fresh_spawn_with_no_roomy_pane_opens_a_fresh_tab_and_uses_its_root_pane_unsplit() {
        let tmp = tempfile::tempdir().unwrap();
        let opts = test_options(tmp.path(), false);
        let mut fake = FakeHerdr::new();
        // Every pane in the tab is too narrow to split into a usable
        // child — the fresh-tab fallback fires instead of a refusal.
        fake.layout = Some(vec![PaneGeom { pane_id: "w1:p1".to_string(), width: 30, height: 43 }]);
        fake.tab_create_result = Ok("w9:p1".to_string());
        seeded_result_dir(&tmp.path().join(".bee"), &opts.job_id);

        let result = execute(&opts, &fake);

        assert!(matches!(result.outcome, RunOutcome::Result(_)), "got {:?}", result.outcome);
        assert_eq!(
            fake.tab_create_calls.borrow().as_slice(),
            [("w1".to_string(), opts.cwd.display().to_string(), opts.job_id.clone())],
            "tab_create must get the caller's own workspace, the run's --cwd, and a label naming the job"
        );
        assert!(fake.split_calls.borrow().is_empty(), "the fresh tab's root pane is used with no split call");
        assert_eq!(result.pane_id.as_deref(), Some("w9:p1"), "the new tab's root pane becomes the worker's pane");
    }

    #[test]
    fn a_fresh_spawn_refuses_only_when_the_fresh_tab_attempt_also_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let opts = test_options(tmp.path(), false);
        let mut fake = FakeHerdr::new();
        fake.layout = Some(vec![PaneGeom { pane_id: "w1:p1".to_string(), width: 15, height: 43 }]);
        fake.tab_create_result = Err("herdr: no free workspace slot".to_string());

        let result = execute(&opts, &fake);

        match &result.outcome {
            RunOutcome::SpawnFailed(msg) => {
                // One pane, so a first split: 15 columns halve to the
                // 7-column child the refusal names.
                assert!(msg.contains("w1:p1") && msg.contains("7") && msg.contains("60"), "{msg}");
                assert!(msg.contains("no free workspace slot"), "must name why the fresh tab failed too: {msg}");
            }
            other => panic!("expected SpawnFailed(narrow pane + failed fresh tab), got {other:?}"),
        }
        assert!(!fake.tab_create_calls.borrow().is_empty(), "a fresh tab must be attempted before refusing");
        assert!(fake.split_calls.borrow().is_empty(), "a refused parent must never be split");
        assert!(fake.start_calls.borrow().is_empty(), "a refused parent must never start an agent");
        assert!(result.pane_id.is_none());
    }

    #[test]
    fn start_retry_survives_a_busy_shell_then_succeeds() {
        let calls = std::cell::RefCell::new(0u32);
        let out = start_with_retry(
            &mut || {
                *calls.borrow_mut() += 1;
                if *calls.borrow() <= 2 {
                    Err("herdr agent start x exited 1: {\"error\":{\"code\":\"agent_pane_busy\",\"message\":\"agent target pane w1:p9 is not an available shell\"}}".to_string())
                } else {
                    Ok(())
                }
            },
            &mut |_d| {},
        );
        assert!(out.is_ok());
        assert_eq!(*calls.borrow(), 3, "two busy refusals then success");
    }

    #[test]
    fn start_retry_never_retries_a_non_busy_error() {
        let calls = std::cell::RefCell::new(0u32);
        let out = start_with_retry(
            &mut || {
                *calls.borrow_mut() += 1;
                Err("herdr agent start x exited 1: unknown kind".to_string())
            },
            &mut |_d| {},
        );
        assert!(out.is_err());
        assert_eq!(*calls.borrow(), 1, "a non-busy error fails immediately");
    }

    #[test]
    fn start_retry_exhaustion_names_the_attempts_and_keeps_spawn_failure_shape() {
        let out = start_with_retry(
            &mut || Err("agent_pane_busy: still booting".to_string()),
            &mut |_d| {},
        );
        let err = out.unwrap_err();
        assert!(err.contains("start attempts"), "{err}");
        // the caller's existing failure arm closes the pane — proven by the
        // pre-existing spawn_failure_closes_the_pane_it_just_split test,
        // which now flows through start_with_retry with a non-busy error.
    }

    #[test]
    fn deliver_pointer_succeeds_in_one_send_when_the_ack_already_shows() {
        // herding-prompt-stall D4: the worker's ack file is the receipt
        // now, not herdr's own "working" transition — an ack already
        // present right after the send needs no resend.
        let sent = std::cell::RefCell::new(0u32);
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| {
                *sent.borrow_mut() += 1;
                Ok(())
            },
            &mut || Some("idle".to_string()),
            &mut || true,
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        assert!(out.is_ok(), "{out:?}");
        assert_eq!(*sent.borrow(), 1, "exactly one send when the ack already shows");
    }

    #[test]
    fn deliver_pointer_retries_a_stall_and_delivers_on_the_next_send() {
        // D6 (hps-14): a stall proves the submission had not landed AT THAT
        // INSTANT, not that it never will — it joins the same bounded retry
        // the ready-with-no-ack path already uses instead of ending the run
        // on the first occurrence.
        let sent = std::cell::RefCell::new(0u32);
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| {
                *sent.borrow_mut() += 1;
                if *sent.borrow() == 1 {
                    Err("herdr agent prompt job-1 exited 1: agent_prompt_stalled: no observed change within \
                         5000ms"
                        .to_string())
                } else {
                    Ok(())
                }
            },
            &mut || Some("idle".to_string()),
            &mut || *sent.borrow() >= 2, // ack shows once the resend lands
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        assert!(out.is_ok(), "{out:?}");
        assert_eq!(*sent.borrow(), 2, "the stall is retried, not fatal — one resend then delivered");
    }

    #[test]
    fn is_agent_prompt_stalled_matches_the_captured_live_stalled_body() {
        // hps-11: parse-level pin over the exact body captured live
        // (docs/history/herding-prompt-stall/CONTEXT.md) — no process spawned.
        let body = r#"{"error":{"code":"agent_prompt_stalled","message":"agent prompt produced no observed state change within 5000 ms; status is idle and state_change_seq remained 771"},"id":"cli:agent:prompt"}"#;
        assert!(is_agent_prompt_stalled(body));
        assert!(!is_agent_prompt_timeout(body));
    }

    #[test]
    fn is_agent_prompt_timeout_matches_the_captured_live_timeout_body() {
        // hps-11: parse-level pin over the exact body captured live —
        // `--timeout 5000` on a healthy pane — no process spawned.
        let body = r#"{"error":{"code":"timeout","message":"timed out waiting for agent status"}}"#;
        assert!(is_agent_prompt_timeout(body));
        assert!(!is_agent_prompt_stalled(body));
    }

    #[test]
    fn deliver_pointer_falls_through_to_the_ack_poll_on_a_herdr_timeout_reply() {
        // hps-11: a `timeout` reply means the submission WAS made — unlike
        // agent_prompt_stalled it must NOT abort delivery. It polls for the
        // ack exactly as a successful send would, never resending blind.
        let sent = std::cell::RefCell::new(0u32);
        let polls = std::cell::RefCell::new(0u32);
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| {
                *sent.borrow_mut() += 1;
                Err(r#"{"error":{"code":"timeout","message":"timed out waiting for agent status"}}"#.to_string())
            },
            &mut || Some("working".to_string()),
            &mut || {
                *polls.borrow_mut() += 1;
                *polls.borrow() >= 2
            },
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        assert!(out.is_ok(), "{out:?}");
        assert_eq!(*sent.borrow(), 1, "exactly one send — a timeout reply is never resent blind");
    }

    #[test]
    fn deliver_pointer_rescues_a_stalled_send_when_the_result_already_landed() {
        // The result_present() escape survives D4: an ultra-fast round can
        // write its result before herdr ever observes the "working"
        // transition.
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| Err("agent_prompt_stalled: no observed change".to_string()),
            &mut || Some("idle".to_string()),
            &mut || false,
            &mut || true,
            &mut |_d| {},
            &mut || 0,
        );
        assert!(out.is_ok(), "{out:?}");
    }

    #[test]
    fn deliver_pointer_rescues_a_stalled_send_when_the_ack_already_landed() {
        // D4: the ack file is its own escape for a stalled send, the same
        // shape as the pre-existing result_present() escape.
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| Err("agent_prompt_stalled: no observed change".to_string()),
            &mut || Some("idle".to_string()),
            &mut || true,
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        assert!(out.is_ok(), "{out:?}");
    }

    #[test]
    fn deliver_pointer_surfaces_a_transport_error_distinctly_from_a_stall() {
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| Err("could not spawn herdr: No such file or directory".to_string()),
            &mut || Some("idle".to_string()),
            &mut || false,
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        match out {
            Err(DeliveryError::Transport(e)) => assert!(e.contains("could not spawn herdr"), "{e}"),
            other => panic!("expected DeliveryError::Transport, got {other:?}"),
        }
    }

    #[test]
    fn deliver_pointer_refuses_fast_when_the_pane_is_already_blocked() {
        // D3: blocked ends the delivery wait at once — never sent into a
        // pane that is waiting on an unrelated question.
        let sent = std::cell::RefCell::new(0u32);
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| {
                *sent.borrow_mut() += 1;
                Ok(())
            },
            &mut || Some("blocked".to_string()),
            &mut || false,
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        assert!(matches!(out, Err(DeliveryError::Blocked)), "{out:?}");
        assert_eq!(*sent.borrow(), 0, "never sends into an already-blocked pane");
    }

    #[test]
    fn deliver_pointer_resends_bounded_while_neither_ack_nor_result_appears() {
        // D4: the send is idempotent — herdr keeps observing "working" but
        // neither the ack nor the result ever shows, so the pointer is
        // resent, bounded, never infinite.
        let sent = std::cell::RefCell::new(0u32);
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| {
                *sent.borrow_mut() += 1;
                Ok(())
            },
            &mut || Some("idle".to_string()),
            &mut || false,
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        assert!(
            matches!(
                out,
                Err(DeliveryError::NeverAcked { bound: NeverAckedBound::ResendAttempts, attempts: DELIVERY_RESEND_ATTEMPTS })
            ),
            "{out:?}"
        );
        assert_eq!(*sent.borrow(), DELIVERY_RESEND_ATTEMPTS, "bounded, not infinite");
    }

    #[test]
    fn deliver_pointer_succeeds_once_the_ack_appears_after_a_resend() {
        let sent = std::cell::RefCell::new(0u32);
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| {
                *sent.borrow_mut() += 1;
                Ok(())
            },
            &mut || Some("idle".to_string()),
            &mut || *sent.borrow() >= 3, // ack shows on the 3rd send
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        assert!(out.is_ok(), "{out:?}");
        assert_eq!(*sent.borrow(), 3, "stops resending the moment the ack shows");
    }

    #[test]
    fn deliver_pointer_stops_resending_the_instant_the_pane_goes_blocked_mid_resend() {
        // D3/D4: a blocked observation mid-resend must not be swallowed by
        // another attempt — it ends the wait at once, same as on the very
        // first check.
        let sent = std::cell::RefCell::new(0u32);
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| {
                *sent.borrow_mut() += 1;
                Ok(())
            },
            &mut || if *sent.borrow() >= 1 { Some("blocked".to_string()) } else { Some("idle".to_string()) },
            &mut || false,
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        assert!(matches!(out, Err(DeliveryError::Blocked)), "{out:?}");
        assert_eq!(*sent.borrow(), 1, "must not send again once blocked shows up between attempts");
    }

    #[test]
    fn deliver_pointer_retries_past_a_stall_appearing_mid_resend() {
        // D6 (hps-14): a stall mid-resend is retried like any other — it
        // must not end the run early, unlike `blocked` which still does.
        let sent = std::cell::RefCell::new(0u32);
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| {
                *sent.borrow_mut() += 1;
                if *sent.borrow() == 3 {
                    Err("agent_prompt_stalled: no observed change".to_string())
                } else {
                    Ok(())
                }
            },
            &mut || Some("idle".to_string()),
            &mut || *sent.borrow() >= 4, // ack shows only after the send past the stall
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        assert!(out.is_ok(), "{out:?}");
        assert_eq!(*sent.borrow(), 4, "the stall does not end the run — it is retried like any other resend");
    }

    #[test]
    fn deliver_pointer_exhausts_the_resend_bound_when_every_send_stalls() {
        // D6 (hps-14): a stall every single time still ends as a terminal
        // error once the resend bound is spent — but its wording says every
        // submission stalled, distinct from `NeverAcked`'s "kept going
        // ready with no ack": the reader can tell "the agent never took the
        // text" from "the agent took it and never acked".
        let sent = std::cell::RefCell::new(0u32);
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| {
                *sent.borrow_mut() += 1;
                Err("agent_prompt_stalled: no observed change".to_string())
            },
            &mut || Some("idle".to_string()),
            &mut || false,
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        match &out {
            Err(DeliveryError::NeverDelivered { attempts }) => {
                assert_eq!(*attempts, DELIVERY_RESEND_ATTEMPTS);
            }
            other => panic!("expected DeliveryError::NeverDelivered, got {other:?}"),
        }
        let msg = out.unwrap_err().to_string();
        assert!(msg.contains("every submission stalled"), "{msg}");
        assert!(!msg.contains("kept going ready with no ack"), "{msg}");
        assert_eq!(*sent.borrow(), DELIVERY_RESEND_ATTEMPTS, "bounded, not infinite");
    }

    #[test]
    fn deliver_pointer_polls_a_working_agent_to_ack_with_exactly_one_send() {
        // hps-6: a worker that is `working` and has not acked yet is
        // healthy — it gets polled, never resent, no matter how many ticks
        // it takes to write its ack.
        let sent = std::cell::RefCell::new(0u32);
        let polls = std::cell::RefCell::new(0u32);
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| {
                *sent.borrow_mut() += 1;
                Ok(())
            },
            &mut || Some("working".to_string()),
            &mut || {
                *polls.borrow_mut() += 1;
                *polls.borrow() >= 5 // ack shows only after a few poll ticks
            },
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        assert!(out.is_ok(), "{out:?}");
        assert_eq!(*sent.borrow(), 1, "a working agent is polled, never resent");
    }

    #[test]
    fn deliver_pointer_resends_only_once_the_agent_goes_ready_with_still_no_ack() {
        // hps-6: `working` is healthy and burns no resend; the pointer is
        // resent only once the agent settles back to `idle`/`done` with the
        // ack still missing — the actual signature of a dropped submission.
        let sent = std::cell::RefCell::new(0u32);
        let poll = std::cell::RefCell::new(0u32);
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| {
                *sent.borrow_mut() += 1;
                Ok(())
            },
            &mut || {
                *poll.borrow_mut() += 1;
                if *poll.borrow() < 3 { Some("working".to_string()) } else { Some("idle".to_string()) }
            },
            &mut || *sent.borrow() >= 2, // ack shows only after the resend
            &mut || false,
            &mut |_d| {},
            &mut || 0,
        );
        assert!(out.is_ok(), "{out:?}");
        assert_eq!(*sent.borrow(), 2, "exactly one resend, triggered only by the return to ready with no ack");
    }

    #[test]
    fn deliver_pointer_expires_the_ack_wait_budget_distinct_from_the_resend_bound() {
        // hps-6: the agent stays `working` forever (healthy — never
        // triggers a resend), so only the wall-clock ack-wait budget, not
        // the resend bound, can end this wait.
        let sent = std::cell::RefCell::new(0u32);
        let calls = std::cell::RefCell::new(0i64);
        let out = deliver_pointer(
            "Read the file /x/brief-1.txt and follow its instructions exactly.",
            &mut |_p| {
                *sent.borrow_mut() += 1;
                Ok(())
            },
            &mut || Some("working".to_string()),
            &mut || false,
            &mut || false,
            &mut |_d| {},
            &mut || {
                let mut c = calls.borrow_mut();
                *c += 1;
                if *c == 1 { 0 } else { (ACK_WAIT_BUDGET_SECS as i64) * 1000 + 1 }
            },
        );
        match out {
            Err(DeliveryError::NeverAcked { bound: NeverAckedBound::AckWaitBudget, attempts }) => {
                assert_eq!(attempts, 1, "the budget expires after the single healthy send")
            }
            other => panic!("expected NeverAcked{{AckWaitBudget}}, got {other:?}"),
        }
        assert_eq!(*sent.borrow(), 1, "never resends while status stays working — only the budget ends it");
    }

    #[test]
    fn ready_wait_accepts_idle_and_only_then() {
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
        assert_eq!(ready, ReadyOutcome::Ready);
        assert_eq!(polls, 3, "ready only on the third status read");
    }

    #[test]
    fn ready_wait_accepts_done_the_same_as_idle() {
        // herding-prompt-stall D2: `done` is the normal resting state of a
        // never-focused, --no-focus-split bee worker pane — not a rejection.
        let mut clock = 0i64;
        let ready = wait_for_agent_ready(
            60,
            Duration::from_millis(500),
            || Some("done".to_string()),
            |_| {},
            || {
                clock += 500;
                clock
            },
        );
        assert_eq!(ready, ReadyOutcome::Ready);
    }

    #[test]
    fn ready_wait_rejects_working_status_until_idle_or_done() {
        // When an agent process is booting or updating, herdr may report
        // "working". Ready-wait must not accept "working" as ready for
        // input; it must wait until the agent settles into idle or done.
        let mut clock = 0i64;
        let statuses = std::cell::RefCell::new(vec![
            Some("working".to_string()),
            Some("working".to_string()),
            Some("done".to_string()),
        ]);
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
        assert_eq!(ready, ReadyOutcome::Ready);
        assert_eq!(polls, 3, "ready only when status reaches idle or done, not while working");
    }

    #[test]
    fn ready_wait_fails_fast_on_blocked_without_burning_the_ceiling() {
        // D3: a blocked pane ends the ready wait at once — the 60s ceiling
        // and its sleeps are never burned on a question nobody will answer.
        let mut clock = 0i64;
        let mut sleeps = 0u32;
        let ready = wait_for_agent_ready(
            60,
            Duration::from_millis(500),
            || Some("blocked".to_string()),
            |_| sleeps += 1,
            || {
                clock += 500;
                clock
            },
        );
        assert_eq!(ready, ReadyOutcome::Blocked);
        assert_eq!(sleeps, 0, "blocked is observed on the first check, before any sleep");
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
        // herding-brief-file D1: the agent receives a ONE-LINE pointer at
        // brief-1.txt; the brief body lives in the file.
        let prompts = fake.prompt_calls.borrow();
        assert_eq!(prompts.len(), 1, "exactly one pointer prompt for round 1");
        assert_eq!(prompts[0].0, "job-1");
        assert!(!prompts[0].1.contains('\n'), "pointer prompt is one line: {}", prompts[0].1);
        assert!(prompts[0].1.contains("brief-1.txt"), "pointer names the brief file: {}", prompts[0].1);
        let brief_text =
            std::fs::read_to_string(mailbox::brief_path(&bee_dir, &opts.job_id, 1)).unwrap();
        assert!(brief_text.contains("round 1"), "brief file names its round");
        assert!(brief_text.contains("result-1.json"), "brief file names the result file");
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
        seed_ack(tmp.path(), &opts.job_id, 1);
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::TimedOutIdle(_)), "got {:?}", result.outcome);
        assert!(!result.closed_pane);
        assert!(fake.closed.borrow().is_empty());
    }

    #[test]
    fn close_always_closes_the_pane_even_on_a_timeout() {
        let tmp = tempfile::tempdir().unwrap();
        let mut opts = test_options(tmp.path(), false);
        opts.idle_timeout_secs = 1;
        opts.close_always = true;
        seed_ack(tmp.path(), &opts.job_id, 1);
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::TimedOutIdle(_)), "got {:?}", result.outcome);
        assert!(result.closed_pane);
        assert_eq!(fake.closed.borrow().as_slice(), ["w1:p2"]);
    }

    #[test]
    fn stale_heartbeat_with_matching_pane_text_pauses_limit_keeps_pane_even_with_close_always_and_stamps_job_json() {
        let tmp = tempfile::tempdir().unwrap();
        let mut opts = test_options(tmp.path(), false);
        opts.idle_timeout_secs = 1;
        opts.close_always = true;
        seed_ack(tmp.path(), &opts.job_id, 1);
        let fake = FakeHerdr::new();
        *fake.pane_text.borrow_mut() = Some("You've hit your session limit · resets 6:20pm (Asia/Bangkok)\n".to_string());
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::PausedLimit), "got {:?}", result.outcome);
        assert!(!result.closed_pane, "pane must NOT be closed even with close_always");
        assert!(fake.closed.borrow().is_empty(), "pane must remain open in herdr");

        let job_file_path = mailbox::job_path(&tmp.path().join(".bee"), &opts.job_id);
        let job: Value = serde_json::from_str(&std::fs::read_to_string(job_file_path).unwrap()).unwrap();
        assert_eq!(job["limit_reset_hint"], "You've hit your session limit · resets 6:20pm (Asia/Bangkok)");
        assert!(job.get("paused_limit_at").is_some());
    }

    #[test]
    fn stale_heartbeat_with_non_matching_pane_text_times_out_idle_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let mut opts = test_options(tmp.path(), false);
        opts.idle_timeout_secs = 1;
        opts.close_always = true;
        seed_ack(tmp.path(), &opts.job_id, 1);
        let fake = FakeHerdr::new();
        *fake.pane_text.borrow_mut() = Some("waiting for something else...".to_string());
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::TimedOutIdle(_)), "got {:?}", result.outcome);
        assert!(result.closed_pane, "TimedOutIdle closes with close_always");
        assert_eq!(fake.closed.borrow().as_slice(), ["w1:p2"]);
    }

    #[test]
    fn fresh_heartbeat_with_matching_pane_text_keeps_waiting_until_result_ready() {
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
        *fake.pane_text.borrow_mut() = Some("hit your session limit (in discussion only)".to_string());
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::Result(_)), "got {:?}", result.outcome);
        assert!(fake.pane_read_calls.borrow().is_empty(), "pane_read must not be called when result is ready with fresh heartbeat");
    }

    #[test]
    fn pane_read_is_not_called_while_heartbeat_is_fresh_and_polled_when_stale() {
        let tmp = tempfile::tempdir().unwrap();
        let mut opts = test_options(tmp.path(), false);
        opts.idle_timeout_secs = 1;
        seed_ack(tmp.path(), &opts.job_id, 1);
        let fake = FakeHerdr::new();
        *fake.pane_text.borrow_mut() = Some("regular error output".to_string());
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::TimedOutIdle(_)), "got {:?}", result.outcome);
        // Across the fresh-heartbeat ticks before staleness, pane_read was never called.
        // Once heartbeat went stale, pane_read was called once to check for limit patterns,
        // then once more by `diagnose_giveup` (hps-7) on the way out of the give-up path.
        let reads = fake.pane_read_calls.borrow();
        assert_eq!(reads.len(), 2, "pane_read should be called once for the limit check and once for hps-7's diagnosis, never on fresh ticks: {:?}", *reads);
        assert_eq!(reads[0], "w1:p2");
        assert_eq!(reads[1], "w1:p2");
    }

    // ─── hps-7: diagnosis wired into the three give-up paths (D5) ───────

    #[test]
    fn diagnose_giveup_upgrades_the_generic_message_on_a_match() {
        let fake = FakeHerdr::new();
        *fake.pane_text.borrow_mut() = Some("Trust this folder?\n".to_string());
        let msg = diagnose_giveup(&fake, "job-1", "w1:p2", "generic timeout text".to_string());
        assert!(msg.contains("generic timeout text"), "generic prefix dropped: {msg}");
        assert!(msg.contains("w1:p2"), "pane id missing: {msg}");
        assert!(msg.contains("job-1"), "job id missing: {msg}");
        assert!(msg.contains("Trust this folder?"), "matched line not quoted: {msg}");
        assert!(msg.contains("remedy"), "remedy missing: {msg}");
        assert!(msg.contains("pane tail:"), "pane tail missing: {msg}");
    }

    #[test]
    fn diagnose_giveup_carries_the_pane_tail_on_a_clipped_narrow_pane_capture() {
        // The live acceptance probe's exact shape: an eight-column clipped
        // pane leaves only a short arrow-key nav footer for
        // `find_prompt_diagnosis` to match — nothing in that fragment names
        // the workspace-trust dialog above it. The pane tail must carry
        // that context, or the give-up message stays as unreadable as the
        // bare matched line.
        let clipped_pane = "Trust th\nis works\npace\n\n↑/↓ Na\n";
        let fake = FakeHerdr::new();
        *fake.pane_text.borrow_mut() = Some(clipped_pane.to_string());
        let msg = diagnose_giveup(&fake, "job-1", "w1:p2", "generic timeout text".to_string());
        assert!(msg.contains("\"↑/↓ Na\""), "matched line not quoted: {msg}");
        assert!(msg.contains("pane tail:"), "pane tail missing: {msg}");
        assert!(msg.contains("Trust th"), "clipped trust-dialog line dropped from tail: {msg}");
    }

    #[test]
    fn diagnose_giveup_leaves_the_generic_message_byte_for_byte_on_no_match() {
        let fake = FakeHerdr::new();
        *fake.pane_text.borrow_mut() = Some("compiling…\nall green\n".to_string());
        let generic = "generic timeout text".to_string();
        let msg = diagnose_giveup(&fake, "job-1", "w1:p2", generic.clone());
        assert!(msg.starts_with(&generic), "generic text dropped: {msg}");
        assert!(msg.contains("pane tail:"), "pane tail label missing: {msg}");
        assert!(msg.contains("all green"), "pane content missing from tail: {msg}");
    }

    #[test]
    fn spawn_failed_remedy_formats_inspect_unclaim_and_release() {
        let msg = spawn_failed_remedy(Some("hrc-3"), Some("w-hrc-3"), Some("w5:pS"));
        assert_eq!(
            msg,
            "inspect pane w5:pS (herdr pane read w5:pS); unwind: bee cells unclaim --id hrc-3; bee reservations release --agent w-hrc-3 --cell hrc-3"
        );
    }

    #[test]
    fn spawn_failed_remedy_without_cell_id_reports_no_cell_id_given() {
        let msg = spawn_failed_remedy(None, Some("w-hrc-3"), Some("w5:pS"));
        assert_eq!(
            msg,
            "inspect pane w5:pS (herdr pane read w5:pS); unwind: no --cell-id was given, release any claim you took by hand"
        );
    }

    #[test]
    fn spawn_failed_remedy_without_pane_id_omits_inspect_prefix() {
        let msg = spawn_failed_remedy(Some("hrc-3"), Some("w-hrc-3"), None);
        assert_eq!(
            msg,
            "unwind: bee cells unclaim --id hrc-3; bee reservations release --agent w-hrc-3 --cell hrc-3"
        );
    }

    #[test]
    fn spawn_failed_remedy_without_agent_omits_agent_flag() {
        let msg = spawn_failed_remedy(Some("hrc-3"), None, Some("w5:pS"));
        assert_eq!(
            msg,
            "inspect pane w5:pS (herdr pane read w5:pS); unwind: bee cells unclaim --id hrc-3; bee reservations release --cell hrc-3"
        );
    }

    #[test]
    fn spawn_failed_remedy_with_all_none_fields() {
        let msg = spawn_failed_remedy(None, None, None);
        assert_eq!(msg, "unwind: no --cell-id was given, release any claim you took by hand");
    }

    #[test]
    fn diagnose_giveup_leaves_the_generic_message_byte_for_byte_on_a_failing_pane_read() {
        let fake = FakeHerdr::new();
        // Even a pane whose text WOULD match if it could be read must never
        // change which message comes back once `pane_read` itself fails.
        *fake.pane_text.borrow_mut() = Some("Trust this folder?\n".to_string());
        *fake.pane_read_err.borrow_mut() = Some("herdr: pane gone".to_string());
        let generic = "generic timeout text".to_string();
        let msg = diagnose_giveup(&fake, "job-1", "w1:p2", generic.clone());
        assert_eq!(msg, generic, "a pane_read failure must not change which error is returned");
    }

    #[test]
    fn ready_gate_timeout_names_the_question_on_screen() {
        let tmp = tempfile::tempdir().unwrap();
        let mut opts = test_options(tmp.path(), false);
        opts.ready_wait_secs = 0; // exhausted on the first check, no sleep
        let fake = FakeHerdr::new();
        *fake.status.borrow_mut() = Some("working".to_string()); // never idle/done/blocked
        *fake.pane_text.borrow_mut() = Some("Trust this folder?\n".to_string());
        let result = execute(&opts, &fake);
        match &result.outcome {
            RunOutcome::SpawnFailed(msg) => {
                assert!(msg.contains("ready-wait"), "generic ready-wait text dropped: {msg}");
                assert!(msg.contains("Trust this folder?"), "question not named: {msg}");
            }
            other => panic!("expected SpawnFailed, got {other:?}"),
        }
    }

    #[test]
    fn ready_gate_blocked_still_wins_over_a_diagnosis_match_in_the_same_pane_text() {
        // D3's fast path is untouched by hps-7: `blocked` is checked first
        // and produces `blocked_message`, never the diagnosis pass, even
        // when the pane text would also match `find_prompt_diagnosis`.
        let tmp = tempfile::tempdir().unwrap();
        let opts = test_options(tmp.path(), false);
        let fake = FakeHerdr::new();
        *fake.status.borrow_mut() = Some("blocked".to_string());
        *fake.pane_text.borrow_mut() = Some("Trust this folder?\n".to_string());
        let result = execute(&opts, &fake);
        match &result.outcome {
            RunOutcome::SpawnFailed(msg) => {
                assert!(msg.contains("is blocked"), "expected D3's blocked_message shape, got: {msg}");
            }
            other => panic!("expected SpawnFailed, got {other:?}"),
        }
    }

    #[test]
    fn round_poll_idle_timeout_names_the_question_on_screen() {
        let tmp = tempfile::tempdir().unwrap();
        let mut opts = test_options(tmp.path(), false);
        opts.idle_timeout_secs = 1;
        seed_ack(tmp.path(), &opts.job_id, 1);
        let fake = FakeHerdr::new();
        *fake.pane_text.borrow_mut() = Some("Trust this folder?\n".to_string());
        let result = execute(&opts, &fake);
        match &result.outcome {
            RunOutcome::TimedOutIdle(msg) => {
                assert!(msg.contains("idle timeout"), "generic idle-timeout text dropped: {msg}");
                assert!(msg.contains("Trust this folder?"), "question not named: {msg}");
            }
            other => panic!("expected TimedOutIdle, got {other:?}"),
        }
    }

    #[test]
    fn delivery_never_acked_names_the_question_on_screen() {
        let tmp = tempfile::tempdir().unwrap();
        let opts = test_options(tmp.path(), false);
        let fake = FakeHerdr::new();
        // idle/done with no ack ever written: `deliver_pointer` resends
        // until `DELIVERY_RESEND_ATTEMPTS` is exhausted (NeverAcked's
        // ResendAttempts bound).
        *fake.status.borrow_mut() = Some("idle".to_string());
        *fake.pane_text.borrow_mut() = Some("Trust this folder?\n".to_string());
        let result = execute(&opts, &fake);
        match &result.outcome {
            RunOutcome::SpawnFailed(msg) => {
                assert!(msg.contains("brief prompt failed after start"), "generic delivery text dropped: {msg}");
                assert!(msg.contains("Trust this folder?"), "question not named: {msg}");
            }
            other => panic!("expected SpawnFailed, got {other:?}"),
        }
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

    #[test]
    fn parse_options_reads_the_agent_flag() {
        let opts =
            parse_options(&["--task", "x", "--main-root", ".", "--agent", "codex-herd"]).unwrap();
        assert_eq!(opts.agent.as_deref(), Some("codex-herd"));
    }

    #[test]
    fn parse_options_agent_defaults_to_none() {
        let opts = parse_options(&["--task", "x", "--main-root", "."]).unwrap();
        assert_eq!(opts.agent, None);
    }

    #[test]
    fn parse_options_continue_still_parses_agent_but_execute_continue_never_reads_it() {
        // herd-registry D2: `--continue` ignores `--agent` because the pane
        // already exists — proven below by `execute_continue` running to
        // completion with `opts.agent` set and never needing to resolve an
        // agent command at all (a fake `PaneTransport` with no `agent_start` wiring
        // would panic if it were ever called on this path).
        let opts = parse_options(&[
            "--task",
            "round 2",
            "--main-root",
            ".",
            "--continue",
            "job-9",
            "--agent",
            "codex-herd",
        ])
        .unwrap();
        assert!(opts.is_continue);
        assert_eq!(opts.agent.as_deref(), Some("codex-herd"));
    }

    #[test]
    fn a_spawn_resolves_the_named_agent_through_the_registry_over_agent_command() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        std::fs::write(
            main_root.join(".bee/config.json"),
            serde_json::json!({
                "herding": {
                    "agent_command": ["claude", "--model", "sonnet"],
                    "agents": {"codex-herd": ["codex", "--flag"]}
                }
            })
            .to_string(),
        )
        .unwrap();
        let mut opts = test_options(main_root, false);
        opts.agent = Some("codex-herd".to_string());
        let bee_dir = main_root.join(".bee");
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
        assert_eq!(fake.started_kind.borrow().as_deref(), Some("codex"));
    }

    #[test]
    fn a_spawn_with_an_unknown_agent_name_refuses_typed_listing_registry_keys() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        std::fs::write(
            main_root.join(".bee/config.json"),
            serde_json::json!({"herding": {"agents": {"codex-herd": ["codex", "--flag"]}}}).to_string(),
        )
        .unwrap();
        let mut opts = test_options(main_root, false);
        opts.agent = Some("no-such-herd".to_string());
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        match &result.outcome {
            RunOutcome::SpawnFailed(msg) => {
                assert!(msg.contains("codex-herd"), "{msg}");
            }
            other => panic!("expected SpawnFailed, got {other:?}"),
        }
    }

    // ─── D3/D4: built-in registry defaults + per-agent env ─────────────

    fn seeded_result_dir(bee_dir: &Path, job_id: &str) {
        let dir = mailbox::mailbox_dir(bee_dir, job_id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("result-1.json"),
            r#"{"status":"done","summary":"ok","files_changed":[],"proof":"n/a"}"#,
        )
        .unwrap();
    }

    #[test]
    fn built_in_agents_resolve_and_spawn_with_zero_herding_config() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        // No config.json at all — resolve_agent_command's cfg falls back
        // to Value::Null, and the registry still seeds the built-ins.
        let mut opts = test_options(main_root, false);
        opts.agent = Some("agy-flash".to_string());
        seeded_result_dir(&main_root.join(".bee"), &opts.job_id);
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::Result(_)), "got {:?}", result.outcome);
        assert_eq!(fake.started_kind.borrow().as_deref(), Some("agy"));
        assert_eq!(fake.started_args.borrow().as_deref(), Some(&["--dangerously-skip-permissions".to_string()][..]));
    }

    #[test]
    fn a_same_name_config_entry_overrides_the_built_in_agents_argv() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        std::fs::write(
            main_root.join(".bee/config.json"),
            serde_json::json!({"herding": {"agents": {"agy-flash": ["agy", "--custom-flag"]}}}).to_string(),
        )
        .unwrap();
        let mut opts = test_options(main_root, false);
        opts.agent = Some("agy-flash".to_string());
        seeded_result_dir(&main_root.join(".bee"), &opts.job_id);
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::Result(_)), "got {:?}", result.outcome);
        assert_eq!(fake.started_kind.borrow().as_deref(), Some("agy"));
        assert_eq!(fake.started_args.borrow().as_deref(), Some(&["--custom-flag".to_string()][..]));
    }

    #[test]
    fn an_object_shape_registry_entry_carries_env_into_the_pane() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        std::fs::write(
            main_root.join(".bee/config.json"),
            serde_json::json!({
                "herding": {
                    "agents": {
                        "codex-envd": {
                            "argv": ["codex", "--flag"],
                            "env": {"API_KEY": "it's-a-secret"},
                        }
                    }
                }
            })
            .to_string(),
        )
        .unwrap();
        let mut opts = test_options(main_root, false);
        opts.agent = Some("codex-envd".to_string());
        seeded_result_dir(&main_root.join(".bee"), &opts.job_id);
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::Result(_)), "got {:?}", result.outcome);
        let calls = fake.pane_run_calls.borrow();
        assert_eq!(calls.len(), 1, "exactly one env export line must be sent, got {calls:?}");
        assert_eq!(calls[0].0, "w1:p2", "the export line must go to the newly split pane");
        assert_eq!(calls[0].1, r#"export API_KEY='it'\''s-a-secret' BEE_HERDING_WORKER='1'"#);
    }

    #[test]
    fn a_bad_env_key_drops_the_whole_entry_and_the_spawn_refuses_typed() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        std::fs::write(
            main_root.join(".bee/config.json"),
            serde_json::json!({
                "herding": {
                    "agents": {
                        "codex-badkey": {"argv": ["codex"], "env": {"1BAD": "v"}},
                        "codex-badval": {"argv": ["codex"], "env": {"OK": "line1\nline2"}},
                    }
                }
            })
            .to_string(),
        )
        .unwrap();
        for bad in ["codex-badkey", "codex-badval"] {
            let mut opts = test_options(main_root, false);
            opts.agent = Some(bad.to_string());
            let fake = FakeHerdr::new();
            let result = execute(&opts, &fake);
            match &result.outcome {
                RunOutcome::SpawnFailed(msg) => assert!(msg.contains(bad), "{msg}"),
                other => panic!("expected SpawnFailed for {bad}, got {other:?}"),
            }
            assert!(fake.pane_run_calls.borrow().is_empty(), "a dropped entry must never reach pane_run");
            assert!(fake.start_calls.borrow().is_empty(), "a dropped entry must never reach agent_start");
        }
    }

    #[test]
    fn the_export_line_is_always_sent_before_agent_start_marker_present_even_for_array_shape_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        std::fs::write(
            main_root.join(".bee/config.json"),
            serde_json::json!({
                "herding": {
                    "agents": {
                        "codex-envd": {"argv": ["codex", "--flag"], "env": {"K": "v"}},
                        "codex-plain": ["codex", "--flag"],
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        let mut opts = test_options(main_root, false);
        opts.agent = Some("codex-envd".to_string());
        seeded_result_dir(&main_root.join(".bee"), &opts.job_id);
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::Result(_)), "got {:?}", result.outcome);
        let log = fake.call_log.borrow();
        let pane_run_idx = log.iter().position(|c| *c == "pane_run").expect("pane_run must be called");
        let agent_start_idx = log.iter().position(|c| *c == "agent_start").expect("agent_start must be called");
        assert!(pane_run_idx < agent_start_idx, "pane_run must precede agent_start: {log:?}");
        assert_eq!(fake.pane_run_calls.borrow()[0].1, "export BEE_HERDING_WORKER='1' K='v'");

        // herding-worker-standalone D2: a fresh spawn ALWAYS sends the
        // export, marker included, even for an array-shape (env-less)
        // registry entry.
        let mut opts2 = test_options(main_root, false);
        opts2.agent = Some("codex-plain".to_string());
        opts2.job_id = "job-plain".to_string();
        seeded_result_dir(&main_root.join(".bee"), &opts2.job_id);
        let fake2 = FakeHerdr::new();
        let result2 = execute(&opts2, &fake2);
        assert!(matches!(result2.outcome, RunOutcome::Result(_)), "got {:?}", result2.outcome);
        let calls2 = fake2.pane_run_calls.borrow();
        assert_eq!(calls2.len(), 1, "an array-shape entry must still send the marker export, got {calls2:?}");
        assert_eq!(calls2[0].1, "export BEE_HERDING_WORKER='1'");
    }

    // ─── hps-8 (D5): workspace-trust pre-flight ─────────────────────────

    #[test]
    fn preflight_appends_the_cwd_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let trust_file = tmp.path().join("settings.json");
        std::fs::write(&trust_file, r#"{"trustedWorkspaces": ["/already/trusted"], "other": "kept"}"#).unwrap();
        let trust = WorkspaceTrust { file: trust_file.display().to_string(), key: "trustedWorkspaces".to_string() };
        let cwd = Path::new("/some/fresh/worktree");

        let outcome = preflight_workspace_trust(&trust, cwd);
        assert_eq!(outcome, TrustPreflightOutcome::Appended);

        let written: Value = serde_json::from_str(&std::fs::read_to_string(&trust_file).unwrap()).unwrap();
        let arr = written["trustedWorkspaces"].as_array().unwrap();
        assert!(arr.iter().any(|v| v.as_str() == Some("/already/trusted")));
        assert!(arr.iter().any(|v| v.as_str() == Some("/some/fresh/worktree")));
        assert_eq!(written["other"], "kept", "fields outside the named array must survive untouched");
    }

    #[test]
    fn preflight_leaves_an_already_present_path_untouched_no_rewrite() {
        let tmp = tempfile::tempdir().unwrap();
        let trust_file = tmp.path().join("settings.json");
        let original = r#"{"trustedWorkspaces": ["/some/fresh/worktree"]}"#;
        std::fs::write(&trust_file, original).unwrap();
        let trust = WorkspaceTrust { file: trust_file.display().to_string(), key: "trustedWorkspaces".to_string() };
        let cwd = Path::new("/some/fresh/worktree");

        let outcome = preflight_workspace_trust(&trust, cwd);
        assert_eq!(outcome, TrustPreflightOutcome::AlreadyTrusted);
        assert_eq!(std::fs::read_to_string(&trust_file).unwrap(), original, "no rewrite when cwd is already present");
    }

    #[test]
    fn preflight_warns_and_proceeds_on_a_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let trust_file = tmp.path().join("does-not-exist.json");
        let trust = WorkspaceTrust { file: trust_file.display().to_string(), key: "trustedWorkspaces".to_string() };
        let outcome = preflight_workspace_trust(&trust, Path::new("/cwd"));
        assert!(matches!(outcome, TrustPreflightOutcome::Warning(_)), "got {outcome:?}");
    }

    #[test]
    fn preflight_warns_and_proceeds_on_unparsable_json() {
        let tmp = tempfile::tempdir().unwrap();
        let trust_file = tmp.path().join("settings.json");
        std::fs::write(&trust_file, "not json at all").unwrap();
        let trust = WorkspaceTrust { file: trust_file.display().to_string(), key: "trustedWorkspaces".to_string() };
        let outcome = preflight_workspace_trust(&trust, Path::new("/cwd"));
        assert!(matches!(outcome, TrustPreflightOutcome::Warning(_)), "got {outcome:?}");
    }

    #[test]
    fn preflight_warns_and_proceeds_on_a_missing_or_non_array_key() {
        let tmp = tempfile::tempdir().unwrap();
        let missing_key_file = tmp.path().join("missing-key.json");
        std::fs::write(&missing_key_file, r#"{"other": "field"}"#).unwrap();
        let trust =
            WorkspaceTrust { file: missing_key_file.display().to_string(), key: "trustedWorkspaces".to_string() };
        assert!(matches!(preflight_workspace_trust(&trust, Path::new("/cwd")), TrustPreflightOutcome::Warning(_)));

        let non_array_file = tmp.path().join("non-array.json");
        std::fs::write(&non_array_file, r#"{"trustedWorkspaces": "not-an-array"}"#).unwrap();
        let trust =
            WorkspaceTrust { file: non_array_file.display().to_string(), key: "trustedWorkspaces".to_string() };
        assert!(matches!(preflight_workspace_trust(&trust, Path::new("/cwd")), TrustPreflightOutcome::Warning(_)));
    }

    #[cfg(unix)]
    #[test]
    fn preflight_warns_and_proceeds_when_the_file_is_unwritable() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        // A read-only DIRECTORY, not just a read-only file, so the atomic
        // write's tmp-then-rename (which needs to create a file in the
        // parent dir) fails regardless of the target file's own mode.
        let trust_file = tmp.path().join("settings.json");
        std::fs::write(&trust_file, r#"{"trustedWorkspaces": []}"#).unwrap();
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
        let trust = WorkspaceTrust { file: trust_file.display().to_string(), key: "trustedWorkspaces".to_string() };

        let outcome = preflight_workspace_trust(&trust, Path::new("/cwd"));

        // Restore write permission so tempdir cleanup can remove the file.
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(matches!(outcome, TrustPreflightOutcome::Warning(_)), "got {outcome:?}");
    }

    #[test]
    fn a_fresh_spawn_pre_flights_workspace_trust_before_the_pane_split() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        let trust_file = main_root.join("antigravity-settings.json");
        std::fs::write(&trust_file, r#"{"trustedWorkspaces": []}"#).unwrap();
        std::fs::write(
            main_root.join(".bee/config.json"),
            serde_json::json!({
                "herding": {
                    "agents": {
                        "agy-flash": {
                            "argv": ["agy", "--dangerously-skip-permissions"],
                            "workspace_trust": {
                                "file": trust_file.display().to_string(),
                                "key": "trustedWorkspaces",
                            },
                        }
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        let mut opts = test_options(main_root, false);
        opts.agent = Some("agy-flash".to_string());
        seeded_result_dir(&main_root.join(".bee"), &opts.job_id);
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::Result(_)), "got {:?}", result.outcome);

        let written: Value = serde_json::from_str(&std::fs::read_to_string(&trust_file).unwrap()).unwrap();
        let arr = written["trustedWorkspaces"].as_array().unwrap();
        assert!(
            arr.iter().any(|v| v.as_str() == Some(opts.cwd.display().to_string().as_str())),
            "the run's cwd must be appended to the trust store, got {written}"
        );
    }

    #[test]
    fn a_fresh_spawn_proceeds_when_the_workspace_trust_file_is_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        std::fs::write(
            main_root.join(".bee/config.json"),
            serde_json::json!({
                "herding": {
                    "agents": {
                        "agy-flash": {
                            "argv": ["agy", "--dangerously-skip-permissions"],
                            "workspace_trust": {
                                "file": main_root.join("no-such-file.json").display().to_string(),
                                "key": "trustedWorkspaces",
                            },
                        }
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        let mut opts = test_options(main_root, false);
        opts.agent = Some("agy-flash".to_string());
        seeded_result_dir(&main_root.join(".bee"), &opts.job_id);
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        assert!(
            matches!(result.outcome, RunOutcome::Result(_)),
            "a missing trust file must never fail the run, got {:?}",
            result.outcome
        );
    }

    #[test]
    fn the_marker_wins_over_a_same_name_per_agent_env_value() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        std::fs::write(
            main_root.join(".bee/config.json"),
            serde_json::json!({
                "herding": {
                    "agents": {
                        "codex-override": {
                            "argv": ["codex", "--flag"],
                            "env": {"BEE_HERDING_WORKER": "0"},
                        }
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        let mut opts = test_options(main_root, false);
        opts.agent = Some("codex-override".to_string());
        seeded_result_dir(&main_root.join(".bee"), &opts.job_id);
        let fake = FakeHerdr::new();
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::Result(_)), "got {:?}", result.outcome);
        let calls = fake.pane_run_calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].1, "export BEE_HERDING_WORKER='1'", "the marker must win over a same-name per-agent value");
    }

    // ─── --continue (D3) ────────────────────────────────────────────────

    #[test]
    fn continue_sends_a_prompt_to_the_recorded_pane_and_never_calls_agent_start() {
        let tmp = tempfile::tempdir().unwrap();
        seed_job(tmp.path(), "job-1", "w1:p2", 1);
        // Round 2's ack is seeded up front, distinct from round 2's result
        // below — delivery (the ack) and completion (the result) are
        // separate concerns (herding-prompt-stall D4); the result still
        // lands later from the background writer thread, unaffected.
        seed_ack(tmp.path(), "job-1", 2);
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
        // herding-brief-file D1: the prompt is a one-line pointer at
        // brief-2.txt; the round-2 brief body lives in the file.
        assert!(!prompts[0].1.contains('\n'), "pointer prompt is one line:\n{}", prompts[0].1);
        assert!(prompts[0].1.contains("brief-2.txt"), "pointer names brief-2.txt:\n{}", prompts[0].1);
        let brief_text = std::fs::read_to_string(mailbox::brief_path(
            &tmp.path().join(".bee"),
            "job-1",
            2,
        ))
        .unwrap();
        assert!(brief_text.contains("round 2: keep going"), "brief file missing the round 2 task");
        assert!(brief_text.contains("result-2.json"), "brief file does not name result-2.json");

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
        seed_ack(tmp.path(), "job-1", 2);
        let mut opts = continue_options(tmp.path(), false);
        opts.idle_timeout_secs = 1; // no round-2 result ever arrives; trips fast
        let fake = FakeHerdr::new();

        let result = execute(&opts, &fake);

        assert!(matches!(result.outcome, RunOutcome::TimedOutIdle(_)), "got {:?}", result.outcome);
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
        // PanicHerdr: proves --continue --dry-run touches no PaneTransport method,
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

    // ─── herding-limit-pause (D3): same-round resume via --continue ────

    #[test]
    fn continue_resumes_same_round_clears_stamp_and_sends_resume_pointer_when_stamped_and_alive() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        let dir = mailbox::mailbox_dir(&bee_dir, "job-1");
        std::fs::create_dir_all(&dir).unwrap();

        // Seed job.json with paused_limit_at, limit_reset_hint, round 1, pane w1:p2
        let job = serde_json::json!({
            "job_id": "job-1",
            "task": "round 1 task",
            "cwd": tmp.path().join("work").display().to_string(),
            "round": 1,
            "idle_timeout_secs": 3_600,
            "ceiling_secs": 3_600,
            "close_always": false,
            "created_at": "2026-01-01T00:00:00Z",
            "pane_id": "w1:p2",
            "kind": "claude",
            "paused_limit_at": "2026-08-20T12:00:00Z",
            "limit_reset_hint": "You've hit your session limit · resets 6:20pm",
        });
        std::fs::write(mailbox::job_path(&bee_dir, "job-1"), serde_json::to_string(&job).unwrap()).unwrap();

        // Write brief-1.txt
        let brief_file = mailbox::brief_path(&bee_dir, "job-1", 1);
        std::fs::write(&brief_file, "round 1 brief text").unwrap();
        // Round 1's ack already exists from the original delivery, before
        // this pause ever happened — the resume nudge re-points at the
        // SAME round, so its receipt is already on disk (herding-prompt-stall D4).
        seed_ack(tmp.path(), "job-1", 1);

        let opts = continue_options(tmp.path(), false);
        let fake = FakeHerdr::new();

        // Simulate worker finishing round 1 shortly after the resume pointer is received
        let result_path = mailbox::result_path(&bee_dir, "job-1", 1);
        let writer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            std::fs::write(
                &result_path,
                r#"{"status":"done","summary":"round 1 completed after limit pause","files_changed":[],"proof":"n/a"}"#,
            )
            .unwrap();
        });

        let result = execute(&opts, &fake);
        writer.join().unwrap();

        // 1. Agent start is never called
        assert!(fake.start_calls.borrow().is_empty(), "--continue must never call agent_start");

        // 2. Exactly one resume prompt was sent to the recorded pane
        let prompts = fake.prompt_calls.borrow();
        assert_eq!(prompts.len(), 1, "expected exactly one resume prompt call: {prompts:?}");
        assert_eq!(prompts[0].0, "job-1");
        let expected_result_path = mailbox::result_path(&bee_dir, "job-1", 1);
        assert!(
            prompts[0].1.contains("your session was paused by a usage limit; continue the task and write the round-1 result file at"),
            "prompt text mismatch: {}",
            prompts[0].1
        );
        assert!(prompts[0].1.contains(&expected_result_path.display().to_string()));

        // 3. job.json on disk had paused_limit_at and limit_reset_hint cleared
        let updated_job: Value = serde_json::from_str(&std::fs::read_to_string(mailbox::job_path(&bee_dir, "job-1")).unwrap()).unwrap();
        assert!(updated_job.get("paused_limit_at").is_none(), "paused_limit_at must be cleared");
        assert!(updated_job.get("limit_reset_hint").is_none(), "limit_reset_hint must be cleared");
        assert_eq!(updated_job["round"], 1);

        // 4. Execution outcome is the round-1 result
        match &result.outcome {
            RunOutcome::Result(r) => assert_eq!(r.summary, "round 1 completed after limit pause"),
            other => panic!("expected Result, got {other:?}"),
        }
        assert_eq!(result.pane_id.as_deref(), Some("w1:p2"));
    }

    #[test]
    fn continue_refuses_typed_when_stamped_and_pane_is_gone() {
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
            "paused_limit_at": "2026-08-20T12:00:00Z",
            "limit_reset_hint": "usage limit",
        });
        std::fs::write(mailbox::job_path(&bee_dir, "job-1"), serde_json::to_string(&job).unwrap()).unwrap();

        let opts = continue_options(tmp.path(), false);
        let mut fake = FakeHerdr::new();
        fake.alive_panes = RefCell::new(Vec::new()); // w1:p2 is gone

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
    fn continue_resumes_same_round_for_higher_round_when_stamped() {
        let tmp = tempfile::tempdir().unwrap();
        let bee_dir = tmp.path().join(".bee");
        let dir = mailbox::mailbox_dir(&bee_dir, "job-1");
        std::fs::create_dir_all(&dir).unwrap();

        // Round 1 result exists from prior round
        std::fs::write(
            mailbox::result_path(&bee_dir, "job-1", 1),
            r#"{"status":"done","summary":"round 1 done","files_changed":[],"proof":"n/a"}"#,
        )
        .unwrap();

        // Job was paused in round 2
        let job = serde_json::json!({
            "job_id": "job-1",
            "task": "round 2 task",
            "cwd": tmp.path().join("work").display().to_string(),
            "round": 2,
            "idle_timeout_secs": 3_600,
            "ceiling_secs": 3_600,
            "close_always": false,
            "created_at": "2026-01-01T00:00:00Z",
            "pane_id": "w1:p2",
            "kind": "claude",
            "paused_limit_at": "2026-08-20T14:00:00Z",
            "limit_reset_hint": "hit your session limit",
        });
        std::fs::write(mailbox::job_path(&bee_dir, "job-1"), serde_json::to_string(&job).unwrap()).unwrap();

        let brief_file = mailbox::brief_path(&bee_dir, "job-1", 2);
        std::fs::write(&brief_file, "round 2 brief text").unwrap();
        // Round 2's ack already exists from the original delivery, before
        // this pause ever happened (herding-prompt-stall D4).
        seed_ack(tmp.path(), "job-1", 2);

        let opts = continue_options(tmp.path(), false);
        let fake = FakeHerdr::new();

        let result_path = mailbox::result_path(&bee_dir, "job-1", 2);
        let writer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            std::fs::write(
                &result_path,
                r#"{"status":"done","summary":"round 2 completed after limit pause","files_changed":[],"proof":"n/a"}"#,
            )
            .unwrap();
        });

        let result = execute(&opts, &fake);
        writer.join().unwrap();

        let prompts = fake.prompt_calls.borrow();
        assert_eq!(prompts.len(), 1);
        let expected_result_path = mailbox::result_path(&bee_dir, "job-1", 2);
        assert!(prompts[0].1.contains("round-2 result file at"));
        assert!(prompts[0].1.contains(&expected_result_path.display().to_string()));

        let updated_job: Value = serde_json::from_str(&std::fs::read_to_string(mailbox::job_path(&bee_dir, "job-1")).unwrap()).unwrap();
        assert!(updated_job.get("paused_limit_at").is_none());
        assert_eq!(updated_job["round"], 2);

        match &result.outcome {
            RunOutcome::Result(r) => assert_eq!(r.summary, "round 2 completed after limit pause"),
            other => panic!("expected Result, got {other:?}"),
        }
    }

    // ─── liveness signals and died outcome ───────────────────────────────

    #[test]
    fn decide_poll_reports_died_when_liveness_armed() {
        assert_eq!(
            decide_poll(5_000, 0, 5_000, 60, 3_600, false, None, Some(Some(1234))),
            PollDecision::Died { pid: Some(1234) }
        );
        assert_eq!(
            decide_poll(5_000, 0, 5_000, 60, 3_600, false, None, Some(None)),
            PollDecision::Died { pid: None }
        );
    }

    #[test]
    fn decide_poll_ceiling_takes_precedence_over_died_in_same_tick() {
        // Ceiling passed AND armed-died in the same tick yields TimedOutCeiling
        assert_eq!(
            decide_poll(3_600_000, 0, 3_599_000, 60, 3_600, false, None, Some(Some(1234))),
            PollDecision::TimedOutCeiling
        );
    }

    #[test]
    fn decide_poll_result_ready_takes_precedence_over_died_in_same_tick() {
        // Result present AND armed-died in the same tick yields ResultReady
        assert_eq!(
            decide_poll(0, 0, 0, 1, 1, true, None, Some(Some(1234))),
            PollDecision::ResultReady
        );
    }

    #[test]
    fn decide_poll_died_takes_precedence_over_stale_heartbeat_idle() {
        // Died check sits before the stale-heartbeat check
        assert_eq!(
            decide_poll(61_000, 0, 0, 60, 3_600, false, None, Some(Some(1234))),
            PollDecision::Died { pid: Some(1234) }
        );
    }

    #[test]
    fn run_poll_loop_reports_died_after_three_consecutive_absent_reads() {
        let mut ticks = 0u32;
        let mut clock = 0i64;
        let decision = run_poll_loop(
            0,
            60,
            3_600,
            Duration::from_millis(0),
            |_| {
                ticks += 1;
                PollTick {
                    result_ready: false,
                    heartbeat_fresh: true,
                    pane_text: None,
                    liveness: Some(Liveness::Absent),
                    blocked: false,
                }
            },
            |_| {},
            || {
                clock += 1_000;
                clock
            },
        );
        assert_eq!(decision, PollDecision::Died { pid: None });
        assert_eq!(ticks, 3);
    }

    #[test]
    fn run_poll_loop_unknown_resets_absent_counter_preventing_died_on_interleave() {
        let mut ticks = 0u32;
        let mut clock = 0i64;
        // Interleave: Absent -> Absent -> Unknown -> Absent -> Absent -> ResultReady
        let liveness_sequence = [
            Some(Liveness::Absent),
            Some(Liveness::Absent),
            Some(Liveness::Unknown),
            Some(Liveness::Absent),
            Some(Liveness::Absent),
        ];
        let decision = run_poll_loop(
            0,
            60,
            3_600,
            Duration::from_millis(0),
            |_| {
                ticks += 1;
                let liveness = if (ticks as usize) <= liveness_sequence.len() {
                    liveness_sequence[(ticks - 1) as usize]
                } else {
                    None
                };
                PollTick {
                    result_ready: ticks >= 6,
                    heartbeat_fresh: true,
                    pane_text: None,
                    liveness,
                    blocked: false,
                }
            },
            |_| {},
            || {
                clock += 1_000;
                clock
            },
        );
        assert_eq!(decision, PollDecision::ResultReady);
        assert_eq!(ticks, 6);
    }

    #[test]
    fn run_poll_loop_alive_resets_absent_counter_and_tracks_last_seen_pid() {
        let mut ticks = 0u32;
        let mut clock = 0i64;
        let liveness_sequence = [
            Some(Liveness::Alive { pid: 4242 }),
            Some(Liveness::Absent),
            Some(Liveness::Absent),
            Some(Liveness::Absent),
        ];
        let decision = run_poll_loop(
            0,
            60,
            3_600,
            Duration::from_millis(0),
            |_| {
                ticks += 1;
                let liveness = liveness_sequence[(ticks - 1) as usize];
                PollTick {
                    result_ready: false,
                    heartbeat_fresh: true,
                    pane_text: None,
                    liveness,
                    blocked: false,
                }
            },
            |_| {},
            || {
                clock += 1_000;
                clock
            },
        );
        assert_eq!(decision, PollDecision::Died { pid: Some(4242) });
        assert_eq!(ticks, 4);
    }

    #[test]
    fn run_poll_loop_unknown_fails_open_and_continues_polling() {
        let mut ticks = 0u32;
        let mut clock = 0i64;
        let decision = run_poll_loop(
            0,
            60,
            3_600,
            Duration::from_millis(0),
            |_| {
                ticks += 1;
                PollTick {
                    result_ready: ticks >= 5,
                    heartbeat_fresh: true,
                    pane_text: None,
                    liveness: Some(Liveness::Unknown),
                    blocked: false,
                }
            },
            |_| {},
            || {
                clock += 1_000;
                clock
            },
        );
        assert_eq!(decision, PollDecision::ResultReady);
        assert_eq!(ticks, 5);
    }

    #[test]
    fn parse_process_info_identifies_alive_with_non_agent_name_when_pid_not_shell_pid() {
        let json = serde_json::json!({
            "result": {
                "process_info": {
                    "foreground_processes": [
                        {"name": "cargo", "pid": 2898247, "cmdline": "cargo test"}
                    ],
                    "shell_pid": 5952
                }
            }
        });
        assert_eq!(parse_process_info(&json), Liveness::Alive { pid: 2898247 });
    }

    #[test]
    fn parse_process_info_identifies_absent_when_only_shell_pid_in_foreground() {
        let json = serde_json::json!({
            "result": {
                "process_info": {
                    "foreground_processes": [
                        {"name": "bash", "pid": 5952}
                    ],
                    "shell_pid": 5952
                }
            }
        });
        assert_eq!(parse_process_info(&json), Liveness::Absent);
    }

    #[test]
    fn parse_process_info_identifies_absent_when_foreground_processes_empty() {
        let json = serde_json::json!({
            "result": {
                "process_info": {
                    "foreground_processes": [],
                    "shell_pid": 5952
                }
            }
        });
        assert_eq!(parse_process_info(&json), Liveness::Absent);
    }

    #[test]
    fn parse_process_info_fails_open_to_unknown_on_error_envelope_or_malformed_json() {
        let error_envelope = serde_json::json!({
            "error": "daemon unavailable"
        });
        assert_eq!(parse_process_info(&error_envelope), Liveness::Unknown);

        let missing_result = serde_json::json!({
            "status": "ok"
        });
        assert_eq!(parse_process_info(&missing_result), Liveness::Unknown);

        let missing_fg = serde_json::json!({
            "result": {
                "shell_pid": 5952
            }
        });
        assert_eq!(parse_process_info(&missing_fg), Liveness::Unknown);
    }

    #[test]
    fn died_outcome_keeps_the_pane_open_unless_close_always() {
        let tmp = tempfile::tempdir().unwrap();
        let mut opts = test_options(tmp.path(), false);
        opts.idle_timeout_secs = 60;
        opts.close_always = false;
        seed_ack(tmp.path(), &opts.job_id, 1);
        let fake = FakeHerdr::new();
        *fake.liveness_responses.borrow_mut() = vec![
            Liveness::Absent,
            Liveness::Absent,
            Liveness::Absent,
        ];
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::Died { .. }), "got {:?}", result.outcome);
        assert!(!result.closed_pane, "pane must remain open as forensics");
        assert!(fake.closed.borrow().is_empty());
    }

    #[test]
    fn died_outcome_with_close_always_closes_pane() {
        let tmp = tempfile::tempdir().unwrap();
        let mut opts = test_options(tmp.path(), false);
        opts.idle_timeout_secs = 60;
        opts.close_always = true;
        seed_ack(tmp.path(), &opts.job_id, 1);
        let fake = FakeHerdr::new();
        *fake.liveness_responses.borrow_mut() = vec![
            Liveness::Absent,
            Liveness::Absent,
            Liveness::Absent,
        ];
        let result = execute(&opts, &fake);
        assert!(matches!(result.outcome, RunOutcome::Died { .. }), "got {:?}", result.outcome);
        assert!(result.closed_pane, "pane must close when close_always is set");
        assert_eq!(fake.closed.borrow().as_slice(), ["w1:p2"]);
    }

    #[test]
    fn outcome_label_and_exit_code_for_died() {
        let died_outcome = RunOutcome::Died { pid: Some(12345) };
        assert_eq!(outcome_label(&died_outcome), "died");
        assert_eq!(exit_code_for(&died_outcome), ExitCode::FAILURE);
    }

    #[test]
    fn parse_options_parses_expertise_flag() {
        let opts = parse_options(&[
            "--task",
            "do something",
            "--expertise",
            "skills/foo.md :: purpose one :: read this to x\n/abs/bar.md :: purpose two :: read this to y",
        ])
        .unwrap();
        assert!(opts.has_explicit_expertise);
        assert_eq!(
            opts.expertise,
            vec![
                ExpertiseEntry {
                    path: "skills/foo.md".to_string(),
                    purpose: "purpose one".to_string(),
                    read_to: "read this to x".to_string(),
                },
                ExpertiseEntry {
                    path: "/abs/bar.md".to_string(),
                    purpose: "purpose two".to_string(),
                    read_to: "read this to y".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parse_options_refuses_malformed_expertise_line() {
        let err1 = parse_options(&["--task", "x", "--expertise", "only one part"]).unwrap_err();
        assert!(err1.contains("malformed --expertise line"), "got: {err1}");
        assert!(err1.contains("only one part"), "got: {err1}");

        let err2 = parse_options(&["--task", "x", "--expertise", "part1 :: part2"]).unwrap_err();
        assert!(err2.contains("malformed --expertise line"), "got: {err2}");

        let err3 = parse_options(&["--task", "x", "--expertise", "part1 ::  :: part3"]).unwrap_err();
        assert!(err3.contains("malformed --expertise line"), "got: {err3}");
    }

    #[test]
    fn dry_run_brief_carries_expertise_section() {
        let tmp = tempfile::tempdir().unwrap();
        let mut opts = test_options(tmp.path(), true);
        opts.has_explicit_expertise = true;
        opts.expertise = vec![ExpertiseEntry {
            path: "skills/bee-swarming/SKILL.md".to_string(),
            purpose: "swarming contract".to_string(),
            read_to: "follow the worker protocol".to_string(),
        }];
        let result = execute(&opts, &PanicHerdr);
        match &result.outcome {
            RunOutcome::DryRun(brief) => {
                assert!(brief.contains("# Expertise — read these before you start"));
                assert!(brief.contains("skills/bee-swarming/SKILL.md — swarming contract. Read it to follow the worker protocol."));
            }
            other => panic!("expected DryRun, got {other:?}"),
        }
    }

    #[test]
    fn continue_round_trip_keeps_job_json_expertise_when_no_fresh_flag() {
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
            "kind": "fake",
            "expertise": [
                {
                    "path": "skills/ref.md",
                    "purpose": "reference docs",
                    "read_to": "understand rules"
                }
            ]
        });
        std::fs::write(mailbox::job_path(&bee_dir, "job-1"), serde_json::to_string(&job).unwrap()).unwrap();
        std::fs::write(
            mailbox::result_path(&bee_dir, "job-1", 1),
            r#"{"status":"done","summary":"round 1 done","files_changed":[],"proof":"n/a"}"#,
        )
        .unwrap();

        let opts = continue_options(tmp.path(), true);
        let result = execute(&opts, &PanicHerdr);
        match &result.outcome {
            RunOutcome::DryRun(brief) => {
                assert!(brief.contains("# Expertise — read these before you start"));
                assert!(brief.contains("skills/ref.md — reference docs. Read it to understand rules."));
            }
            other => panic!("expected DryRun, got {other:?}"),
        }
    }

    #[test]
    fn continue_round_trip_overrides_job_json_expertise_when_fresh_flag_given() {
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
            "kind": "fake",
            "expertise": [
                {
                    "path": "skills/old.md",
                    "purpose": "old ref",
                    "read_to": "old purpose"
                }
            ]
        });
        std::fs::write(mailbox::job_path(&bee_dir, "job-1"), serde_json::to_string(&job).unwrap()).unwrap();
        std::fs::write(
            mailbox::result_path(&bee_dir, "job-1", 1),
            r#"{"status":"done","summary":"round 1 done","files_changed":[],"proof":"n/a"}"#,
        )
        .unwrap();

        let mut opts = continue_options(tmp.path(), true);
        opts.has_explicit_expertise = true;
        opts.expertise = vec![ExpertiseEntry {
            path: "skills/new.md".to_string(),
            purpose: "new ref".to_string(),
            read_to: "new purpose".to_string(),
        }];
        let result = execute(&opts, &PanicHerdr);
        match &result.outcome {
            RunOutcome::DryRun(brief) => {
                assert!(brief.contains("# Expertise — read these before you start"));
                assert!(brief.contains("skills/new.md — new ref. Read it to new purpose."));
                assert!(!brief.contains("skills/old.md"));
            }
            other => panic!("expected DryRun, got {other:?}"),
        }
    }
}
