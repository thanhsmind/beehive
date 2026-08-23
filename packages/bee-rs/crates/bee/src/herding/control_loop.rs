// herding::control_loop — the Rust replacement for control-loop.sh
// (herding-orchestration D8, docs/history/herding-orchestration/CONTEXT.md).
//
// WHAT SURVIVES FROM THE BASH SCRIPT (it is the specification;
// `skills/bee-herding/scripts/control-loop.sh` was deleted by ho-14, which
// rewired bootstrap-cockpit.sh onto this verb — read the script in git
// history, not on disk):
//   - parse a required `--role dispatch|merge` plus the optional
//     `--main-root`, `--interval`, `--timeout`, `--max-iterations`,
//     `--max-consecutive-failures`, `--turn-ceiling` and `--once`;
//   - read the role's opening prompt from `skills/bee-herding/references`;
//   - build the control agent's argv (D13 — see `build_control_argv` below);
//   - check the stop file (`<main-root>/.bee/tmp/bee-herding.stop`) BOTH
//     before and after every iteration;
//   - run one iteration under a wall-clock ceiling;
//   - count consecutive failures and back off, capped, on failure;
//   - stop at the iteration ceiling (default 10_000 — never unbounded);
//   - exit clean on a normal stop, non-zero when the failure ceiling is hit.
//
// WHAT DOES NOT SURVIVE, DELIBERATELY. The bash script's `--command CMD`
// flag was "test-only: evaluated as a shell string via `bash -c`" — exactly
// the shell dependency this rewrite exists to drop (D8: GNU coreutils
// `timeout` and a bash-4.3 nameref were the two Windows blockers; a bare
// `bash -c` would just be a THIRD one, reintroduced). Tests here drive the
// loop through the `ArgvProvider`/`IterationSpawner` seams instead (see
// `tests` below) — no claude binary, no herdr, and no shell anywhere on the
// path a real iteration spawns through.
//
// D13, THE PART THAT HOLDS THE LANE AT STANDARD. With no
// `herding.control_command` key in `<main-root>/.bee/config.json`, the argv
// this loop spawns must be byte-identical to the bash default (the
// `--allowedTools` surface it built and the `--permission-mode
// bypassPermissions` tail on the WORKING agent this loop is not the one
// that sets — see `skills/bee-herding/references/operational-invariants.md`,
// "Permission posture"). `build_control_argv` is the ONE function that
// builds it, called by both the real spawn path (`resolve_iteration_argv`)
// and the `tests::default_argv_is_byte_identical_to_the_bash_default` test
// — a named function, not a closure literal only one of the two could reach
// (the lesson this feature has paid for eight times: CONTEXT.md, and the
// cell that built this file).

use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, ExitStatus};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::TransportKind;

// ═══════════════════════════════════════════════════════════════════════════
// role, options, CLI parsing
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    Dispatch,
    Merge,
}

impl Role {
    fn parse(s: &str) -> Option<Role> {
        match s {
            "dispatch" => Some(Role::Dispatch),
            "merge" => Some(Role::Merge),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Role::Dispatch => "dispatch",
            Role::Merge => "merge",
        }
    }
}

const DEFAULT_INTERVAL: u64 = 60;
const DEFAULT_TIMEOUT: u64 = 900;
const DEFAULT_MAX_CONSECUTIVE_FAILURES: u64 = 20;
const DEFAULT_TURN_CEILING: u64 = 50;
// A loop with no explicit --max-iterations is still bounded (D4/D12 in
// CONTEXT.md): this cap keeps a cold, unattended process from running
// literally forever if every other guard somehow lets it.
const DEFAULT_MAX_ITERATIONS: u64 = 10_000;
// Backoff on consecutive failures is capped so it never grows unbounded.
const BACKOFF_CAP: u64 = 600;
// The grace window between "asked nicely" (SIGTERM / the one Windows
// TerminateProcess call) and "asked with SIGKILL" — control-loop.sh's
// `timeout -k 30s`.
const KILL_GRACE: Duration = Duration::from_secs(30);
const DEFAULT_MODEL: &str = "sonnet";

#[derive(Debug)]
pub(crate) struct Options {
    role: Role,
    main_root: Option<String>,
    interval: u64,
    timeout: u64,
    max_iterations: u64,
    max_consecutive_failures: u64,
    turn_ceiling: u64,
    once: bool,
}

impl Options {
    fn parse(flags: &[&str]) -> Result<Options, String> {
        let mut role: Option<Role> = None;
        let mut main_root: Option<String> = None;
        let mut interval = DEFAULT_INTERVAL;
        let mut timeout = DEFAULT_TIMEOUT;
        let mut max_iterations: Option<u64> = None;
        let mut max_consecutive_failures = DEFAULT_MAX_CONSECUTIVE_FAILURES;
        let mut turn_ceiling = DEFAULT_TURN_CEILING;
        let mut once = false;

        let mut i = 0usize;
        while i < flags.len() {
            let flag = flags[i];
            match flag {
                "--role" => {
                    let v = need_value(flag, flags, i)?;
                    role = Some(
                        Role::parse(v)
                            .ok_or_else(|| format!("unknown role '{v}' (expected dispatch or merge)"))?,
                    );
                    i += 2;
                }
                "--main-root" => {
                    let v = need_value(flag, flags, i)?;
                    main_root = Some(v.to_string());
                    i += 2;
                }
                "--interval" => {
                    let v = need_value(flag, flags, i)?;
                    interval = positive_int(flag, v)?;
                    i += 2;
                }
                "--timeout" => {
                    let v = need_value(flag, flags, i)?;
                    timeout = positive_int(flag, v)?;
                    i += 2;
                }
                "--max-iterations" => {
                    let v = need_value(flag, flags, i)?;
                    max_iterations = Some(positive_int(flag, v)?);
                    i += 2;
                }
                "--max-consecutive-failures" => {
                    let v = need_value(flag, flags, i)?;
                    max_consecutive_failures = positive_int(flag, v)?;
                    i += 2;
                }
                "--turn-ceiling" => {
                    let v = need_value(flag, flags, i)?;
                    turn_ceiling = positive_int(flag, v)?;
                    i += 2;
                }
                "--once" => {
                    once = true;
                    i += 1;
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        let role = role.ok_or_else(|| "--role dispatch|merge is required".to_string())?;
        Ok(Options {
            role,
            main_root,
            interval,
            timeout,
            max_iterations: max_iterations.unwrap_or(DEFAULT_MAX_ITERATIONS),
            max_consecutive_failures,
            turn_ceiling,
            once,
        })
    }
}

/// Refuses a value-taking flag with no value left on the line — the fix for
/// the trailing-flag spin control-loop.sh itself carries a comment about.
fn need_value<'a>(flag: &str, flags: &[&'a str], i: usize) -> Result<&'a str, String> {
    flags.get(i + 1).copied().ok_or_else(|| format!("{flag} requires a value"))
}

/// Rejects empty, non-numeric, and zero — a zero interval is a hot loop.
fn positive_int(flag: &str, raw: &str) -> Result<u64, String> {
    raw.parse::<u64>()
        .ok()
        .filter(|n| *n >= 1)
        .ok_or_else(|| format!("{flag} must be a positive integer (got: '{raw}')"))
}

fn print_usage() {
    eprintln!(
        "Usage: bee herding control-loop --role dispatch|merge [--main-root PATH] \
         [--interval N] [--timeout N] [--max-iterations N] \
         [--max-consecutive-failures N] [--turn-ceiling N] [--once]"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// D13 — the control agent's argv (the crossing that holds the lane)
// ═══════════════════════════════════════════════════════════════════════════

/// One control-pane invocation's argv, ready for
/// `Command::new(argv[0]).args(&argv[1..])` — never a shell.
pub(crate) type Argv = Vec<String>;

/// The ENUMERATED control-pane command surface (D7-FINAL in CONTEXT.md).
/// The `Herdr` arms are byte-identical to control-loop.sh's
/// `allowed_tools_for` — this is part of the D13 crossing, so they are copied
/// character for character, not paraphrased.
///
/// The `Tmux` arms (tmux-herding-cockpit D1) are those same strings with the
/// ONE multiplexer entry swapped — `Bash(herdr:*)` becomes `Bash(tmux:*)` —
/// and nothing else touched. The surface stays enumerated and stays the same
/// width: a control pane driving tmux needs the tmux client for exactly the
/// pane work it used herdr for, and gains no other tool.
fn allowed_tools_for(role: Role, kind: TransportKind) -> &'static str {
    match (role, kind) {
        (Role::Dispatch, TransportKind::Herdr) => {
            "Bash(herdr:*),Bash(.bee/bin/bee:*),Bash(git rev-parse:*),Bash(git status:*),Bash(git -C:*),Read"
        }
        (Role::Merge, TransportKind::Herdr) => {
            "Bash(herdr:*),Bash(.bee/bin/bee:*),Bash(git:*),Bash(ls:*),Bash(mkdir:*),Bash(touch:*),Read"
        }
        (Role::Dispatch, TransportKind::Tmux) => {
            "Bash(tmux:*),Bash(.bee/bin/bee:*),Bash(git rev-parse:*),Bash(git status:*),Bash(git -C:*),Read"
        }
        (Role::Merge, TransportKind::Tmux) => {
            "Bash(tmux:*),Bash(.bee/bin/bee:*),Bash(git:*),Bash(ls:*),Bash(mkdir:*),Bash(touch:*),Read"
        }
    }
}

/// Per-token placeholder substitution — never a join-then-split, never
/// `eval`. A `{PROMPT}` value carrying spaces, quotes or shell
/// metacharacters lands as the literal content of exactly this one argv
/// element and can never spill into another.
fn substitute_token(tok: &str, prompt: &str, model: &str, max_turns: u64, allowed_tools: &str) -> String {
    tok.replace("{PROMPT}", prompt)
        .replace("{MODEL}", model)
        .replace("{MAX_TURNS}", &max_turns.to_string())
        .replace("{ALLOWED_TOOLS}", allowed_tools)
}

/// Builds the control pane's real invocation (D13). `template: None` (no
/// `herding.control_command` config, or a malformed one — the caller already
/// folded that into `None`) returns the exact pre-D4 hardcoded default,
/// BYTE-IDENTICAL to control-loop.sh's `run_iteration` fallback:
/// `claude -p "$PROMPT" --model sonnet --max-turns "$TURN_CEILING"
/// --allowedTools "$ALLOWED_TOOLS"`. `template: Some(tokens)` substitutes
/// every token in place and returns that instead. This is the ONE function
/// both the real spawn path and the D13 test call.
pub(crate) fn build_control_argv(
    template: Option<&[String]>,
    prompt: &str,
    model: &str,
    max_turns: u64,
    allowed_tools: &str,
) -> Argv {
    match template {
        Some(tokens) if !tokens.is_empty() => tokens
            .iter()
            .map(|t| substitute_token(t, prompt, model, max_turns, allowed_tools))
            .collect(),
        _ => vec![
            "claude".to_string(),
            "-p".to_string(),
            prompt.to_string(),
            "--model".to_string(),
            model.to_string(),
            "--max-turns".to_string(),
            max_turns.to_string(),
            "--allowedTools".to_string(),
            allowed_tools.to_string(),
        ],
    }
}

/// `<main-root>/skills/bee-herding/references/<role>-prompt.md` — the same
/// file control-loop.sh's `$SCRIPT_DIR/../references/${ROLE}-prompt.md`
/// resolves to when `$SCRIPT_DIR` is (as it always is) `skills/bee-herding/
/// scripts` under that same repo root: `scripts/../references` collapses to
/// `references` under `skills/bee-herding`, exactly this path anchored at
/// main-root instead. A compiled binary has no "the directory this script
/// lives in" to resolve against, so this anchors at `main_root` instead —
/// the same anchor the stop file already uses — rather than reinventing one.
fn read_prompt_file(main_root: &Path, role: Role) -> Result<String, String> {
    let path = main_root
        .join("skills")
        .join("bee-herding")
        .join("references")
        .join(format!("{}-prompt.md", role.as_str()));
    std::fs::read_to_string(&path).map_err(|e| format!("prompt file not found: {}: {e}", path.display()))
}

/// The ONE function that computes what a real iteration spawns: reads the
/// prompt file, builds the allowedTools surface, reads any config template,
/// and calls `build_control_argv`. Both the real CLI path
/// (`RealArgvProvider`) and the D13 tests call this exact function — never a
/// second, drifted copy.
pub(crate) fn resolve_iteration_argv(main_root: &Path, role: Role, turn_ceiling: u64) -> Result<Argv, String> {
    let prompt = read_prompt_file(main_root, role)?;
    // The transport picks the allowlist (tmux-herding-cockpit D1). A missing
    // key reads as herdr, so a repo with no key spawns the byte-identical
    // pre-tmux argv; a typo'd key is `transport_kind`'s typed refusal and
    // stops the loop here rather than quietly arming the other multiplexer.
    let allowed_tools = allowed_tools_for(role, super::transport_kind_at(main_root)?);
    let template = super::read_command_template_tokens(main_root, "control_command");
    Ok(build_control_argv(template.as_deref(), &prompt, DEFAULT_MODEL, turn_ceiling, allowed_tools))
}

// ═══════════════════════════════════════════════════════════════════════════
// the wall-clock ceiling (the hard technical part)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub(crate) enum IterationOutcome {
    Success,
    Failed(i32),
    /// The ceiling was reached; the child was signalled to die (see
    /// `run_iteration_with_ceiling`'s doc comment for exactly how far that
    /// signal reaches).
    TimedOut,
    /// Could not even be run: argv resolution failed, or the spawn itself
    /// failed (e.g. the program named in argv[0] does not exist).
    Errored(String),
}

pub(crate) struct IterationResult {
    #[allow(dead_code)] // read by tests; production only inspects `.outcome`
    pub(crate) pid: Option<u32>,
    pub(crate) outcome: IterationOutcome,
}

/// The spawn seam tests inject through instead of a real control agent —
/// production never needs a fake here, only `RealSpawner`.
pub(crate) trait IterationSpawner {
    fn spawn(&self, argv: &Argv) -> std::io::Result<Child>;
}

pub(crate) struct RealSpawner;

impl IterationSpawner for RealSpawner {
    fn spawn(&self, argv: &Argv) -> std::io::Result<Child> {
        let (prog, rest) = argv.split_first().expect("argv is never empty (build_control_argv always returns >=1 token)");
        Command::new(prog).args(rest).spawn()
    }
}

fn status_to_outcome(status: ExitStatus) -> IterationOutcome {
    if status.success() {
        IterationOutcome::Success
    } else {
        IterationOutcome::Failed(status.code().unwrap_or(-1))
    }
}

/// Reproduces the wall-clock ceiling `timeout -k 30s "${TIMEOUT}s"` gave the
/// bash loop — without GNU `timeout`, without a shell, on both platforms
/// (CONTEXT.md D13's "Deferred To Planning" item, now due). `std` has no
/// timed wait on a child, so the shape is: hand the spawned `Child` to a
/// thread that does the (blocking) `wait()`, and bound how long THIS
/// function waits for that thread's answer with `recv_timeout`. Killing
/// never goes through the `Child` handle (which is moved into the waiter
/// thread) — it goes by raw pid instead, which needs no ownership of
/// `Child` at all, so the two never contend for it.
///
/// TERMINATE, THEN KILL. On Unix: SIGTERM, then — if the waiter thread has
/// not reported the child dead within `KILL_GRACE` — SIGKILL. On Windows
/// there is no signal distinct from a hard kill: `TerminateProcess` IS the
/// terminate step, so `send_terminate` and `send_kill` are the IDENTICAL
/// call there. The grace period that follows on Unix therefore does not
/// "escalate" on Windows, it just repeats the same call — documented here
/// rather than implied as a two-step signal the way SIGTERM/SIGKILL are.
/// Once SIGKILL (or the Windows call) has been sent, the final `rx.recv()`
/// is unbounded rather than a further timeout: neither signal can be
/// blocked or ignored by the child, so the waiter thread's `wait()` is
/// guaranteed to return — it is just a question of when the OS gets to it.
///
/// SCOPE — DESCENDANTS ARE NOT REACHED. This kills only the immediate child
/// by pid. A control agent spawns processes of its own (it IS one), and
/// neither platform's kill here reaches them: no process group is created
/// for the child on Unix, and no Job Object is created for it on Windows, so
/// a grandchild the agent started keeps running as an orphan after this
/// call returns. This matches `std::process::Child::kill()`'s own scope.
/// Reaching descendants would need that extra grouping; it is not built.
pub(crate) fn run_iteration_with_ceiling(
    argv: &Argv,
    timeout: Duration,
    spawner: &dyn IterationSpawner,
) -> IterationResult {
    let mut child = match spawner.spawn(argv) {
        Ok(c) => c,
        Err(e) => return IterationResult { pid: None, outcome: IterationOutcome::Errored(e.to_string()) },
    };
    let pid = child.id();

    let outcome = thread::scope(|scope| {
        let (tx, rx) = mpsc::channel();
        scope.spawn(move || {
            let result = child.wait();
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(Ok(status)) => status_to_outcome(status),
            Ok(Err(e)) => IterationOutcome::Errored(format!("wait failed: {e}")),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                IterationOutcome::Errored("waiter thread ended without a result".to_string())
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                send_terminate(pid);
                if rx.recv_timeout(KILL_GRACE).is_err() {
                    send_kill(pid);
                    // SIGKILL/TerminateProcess cannot be blocked or ignored:
                    // this always eventually completes (see doc comment).
                    let _ = rx.recv();
                }
                IterationOutcome::TimedOut
            }
        }
    });

    IterationResult { pid: Some(pid), outcome }
}

#[cfg(unix)]
fn send_terminate(pid: u32) {
    unsafe {
        libc::kill(pid as libc::pid_t, libc::SIGTERM);
    }
}

#[cfg(unix)]
fn send_kill(pid: u32) {
    unsafe {
        libc::kill(pid as libc::pid_t, libc::SIGKILL);
    }
}

#[cfg(windows)]
fn send_terminate(pid: u32) {
    terminate_by_pid(pid);
}

#[cfg(windows)]
fn send_kill(pid: u32) {
    terminate_by_pid(pid);
}

#[cfg(windows)]
fn terminate_by_pid(pid: u32) {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if !handle.is_null() {
            TerminateProcess(handle, 1);
            CloseHandle(handle);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// the loop itself: stop file, ceilings, backoff
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoopOutcome {
    /// A stop file, an iteration ceiling, or `--once` — every exit that is
    /// not "gave up after too many consecutive failures". Matches
    /// control-loop.sh's `exit 0`.
    NormalStop,
    /// `max_consecutive_failures` reached. Matches control-loop.sh's `exit 1`.
    FailureCeiling,
}

/// What one iteration spawns, injected so tests need neither a real `claude`
/// binary nor a running herdr (per-cell requirement).
pub(crate) trait ArgvProvider {
    fn argv_for(&self, iteration: u64) -> Result<Argv, String>;
}

/// The real wiring `control_loop()` (the CLI entry point) uses — a named
/// type, not a closure literal, so this crossing is exactly as reachable by
/// a test as the real spawn path is.
pub(crate) struct RealArgvProvider {
    main_root: PathBuf,
    role: Role,
    turn_ceiling: u64,
}

impl ArgvProvider for RealArgvProvider {
    fn argv_for(&self, _iteration: u64) -> Result<Argv, String> {
        resolve_iteration_argv(&self.main_root, self.role, self.turn_ceiling)
    }
}

pub(crate) trait Sleeper {
    fn sleep(&self, d: Duration);
}

pub(crate) struct RealSleeper;

impl Sleeper for RealSleeper {
    fn sleep(&self, d: Duration) {
        thread::sleep(d);
    }
}

/// Backoff sleep after a failed iteration: grows with the consecutive-
/// failure count, capped at `BACKOFF_CAP`.
fn backoff_duration(interval: u64, consecutive: u64) -> Duration {
    Duration::from_secs(interval.saturating_mul(consecutive).min(BACKOFF_CAP))
}

/// The loop mechanics: stop-file checks before AND after every iteration,
/// the iteration ceiling, consecutive-failure counting with capped backoff,
/// the failure ceiling, and `--once`. Generic over `ArgvProvider` /
/// `IterationSpawner` / `Sleeper` so tests drive it with fast fakes instead
/// of real time and a real control agent.
pub(crate) fn run_loop(
    opts: &Options,
    stop_file: &Path,
    argv_provider: &dyn ArgvProvider,
    spawner: &dyn IterationSpawner,
    sleeper: &dyn Sleeper,
) -> LoopOutcome {
    let mut count: u64 = 0;
    let mut consecutive_failures: u64 = 0;

    loop {
        // Stop check BEFORE the iteration: a stop created between cycles
        // ends the loop before any further work.
        if stop_file.exists() {
            eprintln!("control-loop: stop file found at {}; exiting", stop_file.display());
            return LoopOutcome::NormalStop;
        }

        if count >= opts.max_iterations {
            eprintln!("control-loop: reached max-iterations ({}); exiting", opts.max_iterations);
            return LoopOutcome::NormalStop;
        }

        let outcome = match argv_provider.argv_for(count) {
            Ok(argv) => run_iteration_with_ceiling(&argv, Duration::from_secs(opts.timeout), spawner).outcome,
            Err(e) => IterationOutcome::Errored(e),
        };

        match &outcome {
            IterationOutcome::Success => consecutive_failures = 0,
            IterationOutcome::TimedOut => {
                consecutive_failures += 1;
                eprintln!(
                    "control-loop: iteration timed out after {}s; failed iteration {consecutive_failures}/{}",
                    opts.timeout, opts.max_consecutive_failures
                );
            }
            IterationOutcome::Failed(rc) => {
                consecutive_failures += 1;
                eprintln!(
                    "control-loop: iteration failed with exit code {rc}; failed iteration {consecutive_failures}/{}",
                    opts.max_consecutive_failures
                );
            }
            IterationOutcome::Errored(msg) => {
                consecutive_failures += 1;
                eprintln!(
                    "control-loop: iteration errored ({msg}); failed iteration {consecutive_failures}/{}",
                    opts.max_consecutive_failures
                );
            }
        }

        count += 1;

        // Stop check AFTER the iteration: a stop created DURING the
        // iteration takes effect now, not a full interval later.
        if stop_file.exists() {
            eprintln!("control-loop: stop file found at {}; exiting", stop_file.display());
            return LoopOutcome::NormalStop;
        }

        // Consecutive-failure ceiling: give up rather than retry a broken
        // world forever. Checked before the sleep so the exit is prompt.
        if consecutive_failures >= opts.max_consecutive_failures {
            eprintln!(
                "control-loop: {consecutive_failures} consecutive failed iterations (ceiling {}); giving up",
                opts.max_consecutive_failures
            );
            return LoopOutcome::FailureCeiling;
        }

        if opts.once {
            return LoopOutcome::NormalStop;
        }

        if count >= opts.max_iterations {
            eprintln!("control-loop: reached max-iterations ({}); exiting", opts.max_iterations);
            return LoopOutcome::NormalStop;
        }

        if consecutive_failures > 0 {
            sleeper.sleep(backoff_duration(opts.interval, consecutive_failures));
        } else {
            sleeper.sleep(Duration::from_secs(opts.interval));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CLI entry point
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn control_loop(flags: &[&str]) -> ExitCode {
    if flags.iter().any(|f| *f == "-h" || *f == "--help") {
        print_usage();
        return ExitCode::SUCCESS;
    }

    let opts = match Options::parse(flags) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("control-loop: {e}");
            print_usage();
            return ExitCode::FAILURE;
        }
    };

    let Some(main_root) = super::resolve_main_root(opts.main_root.as_deref()) else {
        eprintln!(
            "control-loop: could not resolve the MAIN checkout root (no --main-root given and \
             `git rev-parse --git-common-dir` failed) — refusing to start"
        );
        return ExitCode::FAILURE;
    };

    let stop_file = main_root.join(".bee").join("tmp").join("bee-herding.stop");

    eprintln!(
        "control-loop: role={} interval={}s timeout={}s max-iterations={} max-consecutive-failures={} turn-ceiling={}",
        opts.role.as_str(),
        opts.interval,
        opts.timeout,
        opts.max_iterations,
        opts.max_consecutive_failures,
        opts.turn_ceiling,
    );

    let provider = RealArgvProvider { main_root, role: opts.role, turn_ceiling: opts.turn_ceiling };

    match run_loop(&opts, &stop_file, &provider, &RealSpawner, &RealSleeper) {
        LoopOutcome::NormalStop => ExitCode::SUCCESS,
        LoopOutcome::FailureCeiling => ExitCode::FAILURE,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    // ── D13: the byte-identical default argv ───────────────────────────────

    #[test]
    fn default_argv_is_byte_identical_to_the_bash_default() {
        let tmp = tempfile::tempdir().unwrap();
        let refs = tmp.path().join("skills").join("bee-herding").join("references");
        std::fs::create_dir_all(&refs).unwrap();
        std::fs::write(refs.join("dispatch-prompt.md"), "PROMPT BODY dispatch\n").unwrap();

        let argv = resolve_iteration_argv(tmp.path(), Role::Dispatch, 50).expect("argv resolves");

        // The WHOLE token vector, not a prefix or a count — the crossing
        // production actually spawns through.
        assert_eq!(
            argv,
            vec![
                "claude".to_string(),
                "-p".to_string(),
                "PROMPT BODY dispatch\n".to_string(),
                "--model".to_string(),
                "sonnet".to_string(),
                "--max-turns".to_string(),
                "50".to_string(),
                "--allowedTools".to_string(),
                "Bash(herdr:*),Bash(.bee/bin/bee:*),Bash(git rev-parse:*),Bash(git status:*),Bash(git -C:*),Read"
                    .to_string(),
            ]
        );
    }

    #[test]
    fn merge_role_gets_the_merge_allowlist_and_prompt_file() {
        let tmp = tempfile::tempdir().unwrap();
        let refs = tmp.path().join("skills").join("bee-herding").join("references");
        std::fs::create_dir_all(&refs).unwrap();
        std::fs::write(refs.join("merge-prompt.md"), "PROMPT BODY merge\n").unwrap();

        let argv = resolve_iteration_argv(tmp.path(), Role::Merge, 7).expect("argv resolves");
        assert_eq!(
            argv,
            vec![
                "claude".to_string(),
                "-p".to_string(),
                "PROMPT BODY merge\n".to_string(),
                "--model".to_string(),
                "sonnet".to_string(),
                "--max-turns".to_string(),
                "7".to_string(),
                "--allowedTools".to_string(),
                "Bash(herdr:*),Bash(.bee/bin/bee:*),Bash(git:*),Bash(ls:*),Bash(mkdir:*),Bash(touch:*),Read"
                    .to_string(),
            ]
        );
    }

    // ── the transport swaps exactly one allowlist entry ───────────────────

    #[test]
    fn allowlist_herdr_arms_stay_the_byte_identical_bash_strings() {
        assert_eq!(
            allowed_tools_for(Role::Dispatch, TransportKind::Herdr),
            "Bash(herdr:*),Bash(.bee/bin/bee:*),Bash(git rev-parse:*),Bash(git status:*),Bash(git -C:*),Read"
        );
        assert_eq!(
            allowed_tools_for(Role::Merge, TransportKind::Herdr),
            "Bash(herdr:*),Bash(.bee/bin/bee:*),Bash(git:*),Bash(ls:*),Bash(mkdir:*),Bash(touch:*),Read"
        );
    }

    #[test]
    fn allowlist_tmux_arms_name_tmux_and_never_herdr() {
        for role in [Role::Dispatch, Role::Merge] {
            let tools = allowed_tools_for(role, TransportKind::Tmux);
            assert!(tools.contains("Bash(tmux:*)"), "{role:?}: {tools}");
            assert!(!tools.contains("Bash(herdr:*)"), "{role:?}: {tools}");
        }
    }

    #[test]
    fn allowlist_tmux_arms_swap_that_one_entry_and_widen_nothing() {
        // The surface stays ENUMERATED and stays the same width: the tmux
        // string must be the herdr string with the single multiplexer token
        // rewritten, never a superset.
        for role in [Role::Dispatch, Role::Merge] {
            assert_eq!(
                allowed_tools_for(role, TransportKind::Tmux),
                allowed_tools_for(role, TransportKind::Herdr).replace("Bash(herdr:*)", "Bash(tmux:*)"),
                "{role:?}"
            );
        }
    }

    #[test]
    fn allowlist_transport_tmux_config_reaches_the_spawned_argv() {
        // The whole point of threading the kind: `herding.transport: tmux`
        // in the main checkout's config lands in the argv the control pane
        // is really spawned with.
        let tmp = tempfile::tempdir().unwrap();
        let refs = tmp.path().join("skills").join("bee-herding").join("references");
        std::fs::create_dir_all(&refs).unwrap();
        std::fs::write(refs.join("dispatch-prompt.md"), "p").unwrap();
        let bee_dir = tmp.path().join(".bee");
        std::fs::create_dir_all(&bee_dir).unwrap();
        std::fs::write(bee_dir.join("config.json"), r#"{"herding":{"transport":"tmux"}}"#).unwrap();

        let argv = resolve_iteration_argv(tmp.path(), Role::Dispatch, 50).expect("argv resolves");
        assert_eq!(
            argv.last().unwrap(),
            "Bash(tmux:*),Bash(.bee/bin/bee:*),Bash(git rev-parse:*),Bash(git status:*),Bash(git -C:*),Read"
        );
    }

    #[test]
    fn missing_prompt_file_is_an_error_not_a_panic() {
        let tmp = tempfile::tempdir().unwrap();
        let err = resolve_iteration_argv(tmp.path(), Role::Dispatch, 50).unwrap_err();
        assert!(err.contains("prompt file not found"), "{err}");
    }

    // ── config-driven substitution: per-token, never join-then-split ──────

    #[test]
    fn config_present_substitutes_per_token_never_join_then_split() {
        let tmp = tempfile::tempdir().unwrap();
        let refs = tmp.path().join("skills").join("bee-herding").join("references");
        std::fs::create_dir_all(&refs).unwrap();
        // A prompt value carrying spaces, quotes and shell metacharacters —
        // proof it lands as the literal content of ONE argv element.
        let prompt_text = r#"do the thing; rm -rf / && echo "gotcha" $(whoami)"#;
        std::fs::write(refs.join("dispatch-prompt.md"), prompt_text).unwrap();

        let bee_dir = tmp.path().join(".bee");
        std::fs::create_dir_all(&bee_dir).unwrap();
        std::fs::write(
            bee_dir.join("config.json"),
            r#"{"herding":{"control_command":["codex","exec","-m","{MODEL}","-s","workspace-write","{PROMPT}","--turns","{MAX_TURNS}","--tools","{ALLOWED_TOOLS}"]}}"#,
        )
        .unwrap();

        let argv = resolve_iteration_argv(tmp.path(), Role::Dispatch, 12).expect("argv resolves");
        assert_eq!(
            argv,
            vec![
                "codex".to_string(),
                "exec".to_string(),
                "-m".to_string(),
                "sonnet".to_string(),
                "-s".to_string(),
                "workspace-write".to_string(),
                prompt_text.to_string(),
                "--turns".to_string(),
                "12".to_string(),
                "--tools".to_string(),
                "Bash(herdr:*),Bash(.bee/bin/bee:*),Bash(git rev-parse:*),Bash(git status:*),Bash(git -C:*),Read"
                    .to_string(),
            ]
        );
        // Nine tokens in, nine tokens out — the metacharacter-laden prompt
        // never split into, or swallowed, a neighbouring argv element.
        assert_eq!(argv.len(), 11);
    }

    #[test]
    fn empty_or_malformed_template_falls_back_to_the_default_never_panics() {
        let tmp = tempfile::tempdir().unwrap();
        let refs = tmp.path().join("skills").join("bee-herding").join("references");
        std::fs::create_dir_all(&refs).unwrap();
        std::fs::write(refs.join("dispatch-prompt.md"), "p").unwrap();
        let bee_dir = tmp.path().join(".bee");
        std::fs::create_dir_all(&bee_dir).unwrap();
        std::fs::write(bee_dir.join("config.json"), r#"{"herding":{"control_command":[]}}"#).unwrap();

        let argv = resolve_iteration_argv(tmp.path(), Role::Dispatch, 5).expect("argv resolves");
        assert_eq!(argv[0], "claude"); // fell back to the default, per D4
    }

    // ── the ceiling: actually killed, not just "a timer elapsed" ──────────

    const SLEEP_FOREVER_ENV: &str = "BEE_HERDING_CONTROL_LOOP_TEST_SLEEP_FOREVER";
    const QUICK_EXIT_ENV: &str = "BEE_HERDING_CONTROL_LOOP_TEST_QUICK_EXIT";

    /// A real, externally-killable long-running process for the kill test
    /// below: only blocks when the env var is set, so a normal whole-suite
    /// `cargo test` run (the var absent) passes through this test instantly
    /// like any other.
    #[test]
    fn sleep_forever_when_asked() {
        if std::env::var(SLEEP_FOREVER_ENV).is_ok() {
            loop {
                thread::sleep(Duration::from_secs(3600));
            }
        }
    }

    /// A real, fast-exiting process for the loop-mechanics tests below:
    /// panics (nonzero exit) only when told to via env var, otherwise
    /// returns immediately — safe to run as part of a normal whole-suite
    /// `cargo test` too (it just passes).
    #[test]
    fn quick_exit_helper() {
        if std::env::var(QUICK_EXIT_ENV).as_deref() == Ok("fail") {
            panic!("bee-herding-control-loop test: intentional failure");
        }
    }

    /// Spawns a copy of THIS test binary (`current_exe()`), targeting one
    /// named test function by `--exact`, with an env var set so tests need
    /// neither a `claude` binary nor herdr (per-cell requirement) while
    /// still exercising a REAL OS process on the kill path. The command it
    /// actually runs is fixed (it must be a controllable self-exec, not
    /// whatever argv happens to carry), but it still OBSERVES the argv it
    /// was handed — `received()` is read by
    /// `overrunning_iteration_is_actually_killed` below, so a mutation that
    /// swaps out what `run_iteration_with_ceiling` passes to the spawner is
    /// still caught even on the kill path.
    struct SelfExecSpawner {
        test_name: &'static str,
        env_var: &'static str,
        env_value: &'static str,
        received: Mutex<Vec<Argv>>,
    }

    impl SelfExecSpawner {
        fn new(test_name: &'static str, env_var: &'static str, env_value: &'static str) -> Self {
            SelfExecSpawner { test_name, env_var, env_value, received: Mutex::new(Vec::new()) }
        }
        fn received(&self) -> Vec<Argv> {
            self.received.lock().unwrap().clone()
        }
    }

    impl IterationSpawner for SelfExecSpawner {
        fn spawn(&self, argv: &Argv) -> std::io::Result<Child> {
            self.received.lock().unwrap().push(argv.clone());
            let exe = std::env::current_exe().expect("test binary path");
            // Null the child's stdio: the child is a full libtest harness, and a
            // deliberately FAILING child would otherwise print its own
            // "test ... FAILED" report into THIS suite's inherited output stream,
            // where it reads as a failure of the parent suite.
            Command::new(exe)
                .args([self.test_name, "--exact"])
                .env(self.env_var, self.env_value)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
        }
    }

    fn is_pid_alive(pid: u32) -> bool {
        crate::lock::is_pid_alive(Some(pid as f64))
    }

    #[test]
    fn overrunning_iteration_is_actually_killed() {
        let spawner = SelfExecSpawner::new(
            "herding::control_loop::tests::sleep_forever_when_asked",
            SLEEP_FOREVER_ENV,
            "1",
        );
        // The command actually run is fixed (a controllable self-exec), but
        // the argv passed through `run_iteration_with_ceiling` must still be
        // OBSERVED unaltered — asserted below via `spawner.received()`.
        let argv: Argv = vec!["placeholder".to_string()];

        let result = run_iteration_with_ceiling(&argv, Duration::from_millis(250), &spawner);
        assert!(matches!(result.outcome, IterationOutcome::TimedOut), "{:?}", result.outcome);
        assert_eq!(
            spawner.received(),
            vec![argv.clone()],
            "the spawner must receive exactly the argv run_iteration_with_ceiling was given"
        );
        let pid = result.pid.expect("a pid was captured before the kill");

        // Prove it is ACTUALLY dead — not merely that our timer elapsed.
        // SIGKILL/TerminateProcess are not blockable, but reaping can lag
        // by a hair after `run_iteration_with_ceiling` returns; give it a
        // short, bounded window before asserting.
        let mut alive = is_pid_alive(pid);
        for _ in 0..20 {
            if !alive {
                break;
            }
            thread::sleep(Duration::from_millis(50));
            alive = is_pid_alive(pid);
        }
        assert!(!alive, "pid {pid} is still alive after the ceiling kill");
    }

    #[test]
    fn an_iteration_finishing_well_inside_the_ceiling_succeeds() {
        let spawner =
            SelfExecSpawner::new("herding::control_loop::tests::quick_exit_helper", QUICK_EXIT_ENV, "succeed");
        let argv: Argv = vec!["placeholder".to_string()];
        let result = run_iteration_with_ceiling(&argv, Duration::from_secs(30), &spawner);
        assert!(matches!(result.outcome, IterationOutcome::Success), "{:?}", result.outcome);
        assert_eq!(spawner.received(), vec![argv.clone()]);
    }

    #[test]
    fn a_failing_iteration_reports_failed_not_timed_out() {
        let spawner = SelfExecSpawner::new("herding::control_loop::tests::quick_exit_helper", QUICK_EXIT_ENV, "fail");
        let argv: Argv = vec!["placeholder".to_string()];
        let result = run_iteration_with_ceiling(&argv, Duration::from_secs(30), &spawner);
        assert!(matches!(result.outcome, IterationOutcome::Failed(_)), "{:?}", result.outcome);
        assert_eq!(spawner.received(), vec![argv.clone()]);
    }

    // ── loop mechanics: stop file, ceilings, backoff, --once ───────────────

    struct FixedArgvProvider(Argv);
    impl ArgvProvider for FixedArgvProvider {
        fn argv_for(&self, _iteration: u64) -> Result<Argv, String> {
            Ok(self.0.clone())
        }
    }

    struct NeverCalledArgvProvider;
    impl ArgvProvider for NeverCalledArgvProvider {
        fn argv_for(&self, _iteration: u64) -> Result<Argv, String> {
            panic!("argv_for must not be called when the stop file already exists");
        }
    }

    /// Records every argv it is handed and how many times it was called,
    /// then delegates the actual spawn to an inner spawner — lets tests
    /// assert on the full value crossing into the spawner, not just a count.
    /// `seen()` is read by `real_argv_provider_output_reaches_the_spawner_unaltered`
    /// below, comparing the WHOLE token vector, not a prefix or a count.
    struct CountingSpawner<'a> {
        inner: &'a dyn IterationSpawner,
        calls: AtomicUsize,
        seen: Mutex<Vec<Argv>>,
    }

    impl<'a> CountingSpawner<'a> {
        fn new(inner: &'a dyn IterationSpawner) -> Self {
            CountingSpawner { inner, calls: AtomicUsize::new(0), seen: Mutex::new(Vec::new()) }
        }
        fn call_count(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
        fn seen(&self) -> Vec<Argv> {
            self.seen.lock().unwrap().clone()
        }
    }

    impl<'a> IterationSpawner for CountingSpawner<'a> {
        fn spawn(&self, argv: &Argv) -> std::io::Result<Child> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.seen.lock().unwrap().push(argv.clone());
            self.inner.spawn(argv)
        }
    }

    struct RecordingSleeper {
        calls: Mutex<Vec<Duration>>,
    }
    impl RecordingSleeper {
        fn new() -> Self {
            RecordingSleeper { calls: Mutex::new(Vec::new()) }
        }
        fn recorded(&self) -> Vec<Duration> {
            self.calls.lock().unwrap().clone()
        }
    }
    impl Sleeper for RecordingSleeper {
        fn sleep(&self, d: Duration) {
            self.calls.lock().unwrap().push(d);
        }
    }

    fn quick_success_provider() -> FixedArgvProvider {
        FixedArgvProvider(vec!["placeholder".to_string()])
    }

    fn opts(overrides: impl FnOnce(&mut Options)) -> Options {
        let mut o = Options {
            role: Role::Dispatch,
            main_root: None,
            interval: 1,
            timeout: 30,
            max_iterations: DEFAULT_MAX_ITERATIONS,
            max_consecutive_failures: DEFAULT_MAX_CONSECUTIVE_FAILURES,
            turn_ceiling: DEFAULT_TURN_CEILING,
            once: false,
        };
        overrides(&mut o);
        o
    }

    #[test]
    fn preexisting_stop_file_stops_the_loop_before_any_iteration() {
        let tmp = tempfile::tempdir().unwrap();
        let stop_file = tmp.path().join("stop");
        std::fs::write(&stop_file, b"").unwrap();

        let sleeper = RecordingSleeper::new();
        let outcome = run_loop(
            &opts(|_| {}),
            &stop_file,
            &NeverCalledArgvProvider,
            &SelfExecSpawner::new("herding::control_loop::tests::quick_exit_helper", QUICK_EXIT_ENV, "succeed"),
            &sleeper,
        );
        assert_eq!(outcome, LoopOutcome::NormalStop);
        assert!(sleeper.recorded().is_empty(), "no iteration ran, so no sleep should have happened");
    }

    /// The stop-file check AFTER an iteration takes effect at that
    /// iteration's boundary, not a full interval later: a provider that
    /// creates the stop file as a side effect of resolving iteration 0's
    /// argv must stop the loop after exactly one spawn, never a second.
    struct TouchStopFileThenProvide {
        stop_file: PathBuf,
        argv: Argv,
    }
    impl ArgvProvider for TouchStopFileThenProvide {
        fn argv_for(&self, iteration: u64) -> Result<Argv, String> {
            if iteration == 0 {
                std::fs::write(&self.stop_file, b"").map_err(|e| e.to_string())?;
            }
            Ok(self.argv.clone())
        }
    }

    #[test]
    fn stop_file_created_mid_iteration_is_honoured_right_after_that_iteration() {
        let tmp = tempfile::tempdir().unwrap();
        let stop_file = tmp.path().join("stop");
        let real_spawner = SelfExecSpawner::new("herding::control_loop::tests::quick_exit_helper", QUICK_EXIT_ENV, "succeed");
        let counting = CountingSpawner::new(&real_spawner);
        let fixed_argv = vec!["placeholder".to_string()];
        let provider = TouchStopFileThenProvide { stop_file: stop_file.clone(), argv: fixed_argv.clone() };
        let sleeper = RecordingSleeper::new();

        let outcome = run_loop(&opts(|_| {}), &stop_file, &provider, &counting, &sleeper);

        assert_eq!(outcome, LoopOutcome::NormalStop);
        assert_eq!(counting.call_count(), 1, "the loop must stop after the iteration that created the stop file, not run more");
        assert_eq!(
            counting.seen(),
            vec![fixed_argv],
            "the whole argv the provider returned must reach the spawner unaltered"
        );
        assert!(sleeper.recorded().is_empty(), "the after-iteration stop check must fire before any sleep");
    }

    #[test]
    fn iteration_ceiling_stops_the_loop_after_exactly_max_iterations() {
        let tmp = tempfile::tempdir().unwrap();
        let stop_file = tmp.path().join("stop"); // never created
        let real_spawner = SelfExecSpawner::new("herding::control_loop::tests::quick_exit_helper", QUICK_EXIT_ENV, "succeed");
        let counting = CountingSpawner::new(&real_spawner);
        let provider = quick_success_provider();
        let sleeper = RecordingSleeper::new();

        let outcome = run_loop(&opts(|o| o.max_iterations = 3), &stop_file, &provider, &counting, &sleeper);

        assert_eq!(outcome, LoopOutcome::NormalStop);
        assert_eq!(counting.call_count(), 3);
    }

    #[test]
    fn once_runs_exactly_one_iteration_then_exits_clean_even_on_a_lone_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let stop_file = tmp.path().join("stop");
        let real_spawner = SelfExecSpawner::new("herding::control_loop::tests::quick_exit_helper", QUICK_EXIT_ENV, "fail");
        let counting = CountingSpawner::new(&real_spawner);
        let provider = quick_success_provider();
        let sleeper = RecordingSleeper::new();

        let outcome =
            run_loop(&opts(|o| o.once = true), &stop_file, &provider, &counting, &sleeper);

        // A single failure is below any real failure ceiling — --once still
        // exits clean after its one iteration, exactly as control-loop.sh's
        // ordering (failure-ceiling check, THEN once check) requires.
        assert_eq!(outcome, LoopOutcome::NormalStop);
        assert_eq!(counting.call_count(), 1);
        assert!(sleeper.recorded().is_empty());
    }

    #[test]
    fn consecutive_failure_ceiling_gives_up_and_backoff_is_capped() {
        let tmp = tempfile::tempdir().unwrap();
        let stop_file = tmp.path().join("stop");
        let real_spawner = SelfExecSpawner::new("herding::control_loop::tests::quick_exit_helper", QUICK_EXIT_ENV, "fail");
        let counting = CountingSpawner::new(&real_spawner);
        let provider = quick_success_provider();
        let sleeper = RecordingSleeper::new();

        let outcome = run_loop(
            &opts(|o| {
                o.max_consecutive_failures = 3;
                o.interval = 400; // > BACKOFF_CAP/2 so the cap is exercised
            }),
            &stop_file,
            &provider,
            &counting,
            &sleeper,
        );

        assert_eq!(outcome, LoopOutcome::FailureCeiling);
        assert_eq!(counting.call_count(), 3, "gives up AT the ceiling, not one iteration past it");
        // Backoff after failures 1 and 2 (the loop gives up before sleeping
        // a third time): interval*1, then min(interval*2, BACKOFF_CAP).
        assert_eq!(
            sleeper.recorded(),
            vec![Duration::from_secs(400), Duration::from_secs(BACKOFF_CAP)]
        );
    }

    #[test]
    fn backoff_duration_grows_with_consecutive_failures_and_caps() {
        assert_eq!(backoff_duration(10, 1), Duration::from_secs(10));
        assert_eq!(backoff_duration(10, 5), Duration::from_secs(50));
        assert_eq!(backoff_duration(1000, 1), Duration::from_secs(BACKOFF_CAP));
        assert_eq!(backoff_duration(100, 100), Duration::from_secs(BACKOFF_CAP));
    }

    // ── production wiring: the real argv provider, spawner, sleeper and
    //    the loop entry point, each reached by at least one test ─────────

    /// THE mutation-catching test. `RealArgvProvider` is the production
    /// wiring `control_loop()` actually uses, `CountingSpawner` records the
    /// WHOLE argv token vector the spawner received — not a count, not a
    /// prefix — and this asserts it equals what `resolve_iteration_argv`
    /// (the same function `RealArgvProvider::argv_for` calls) built for the
    /// identical inputs. A mutation that discards the argv `run_loop` /
    /// `run_iteration_with_ceiling` built and hands the spawner a
    /// hardcoded vector instead fails this test.
    #[test]
    fn real_argv_provider_output_reaches_the_spawner_unaltered() {
        let tmp = tempfile::tempdir().unwrap();
        let refs = tmp.path().join("skills").join("bee-herding").join("references");
        std::fs::create_dir_all(&refs).unwrap();
        std::fs::write(refs.join("dispatch-prompt.md"), "PROMPT BODY dispatch\n").unwrap();

        let provider = RealArgvProvider { main_root: tmp.path().to_path_buf(), role: Role::Dispatch, turn_ceiling: 50 };
        let expected = resolve_iteration_argv(tmp.path(), Role::Dispatch, 50).expect("argv resolves");

        let stop_file = tmp.path().join("stop");
        let real_spawner = SelfExecSpawner::new("herding::control_loop::tests::quick_exit_helper", QUICK_EXIT_ENV, "succeed");
        let counting = CountingSpawner::new(&real_spawner);
        let sleeper = RecordingSleeper::new();

        let outcome = run_loop(&opts(|o| o.once = true), &stop_file, &provider, &counting, &sleeper);

        assert_eq!(outcome, LoopOutcome::NormalStop);
        assert_eq!(
            counting.seen(),
            vec![expected],
            "the whole argv token vector RealArgvProvider built must reach the spawner unaltered — \
             not a prefix, not a length, not the first flag"
        );
    }

    #[test]
    fn real_spawner_actually_spawns_the_given_program() {
        let exe = std::env::current_exe().expect("test binary path");
        let argv: Argv = vec![
            exe.to_string_lossy().to_string(),
            "herding::control_loop::tests::quick_exit_helper".to_string(),
            "--exact".to_string(),
        ];
        let mut child = RealSpawner.spawn(&argv).expect("RealSpawner must be able to spawn a real program");
        let status = child.wait().expect("spawned child must be waitable");
        assert!(status.success(), "{status:?}");
    }

    #[test]
    fn real_sleeper_actually_sleeps() {
        let start = std::time::Instant::now();
        RealSleeper.sleep(Duration::from_millis(20));
        assert!(start.elapsed() >= Duration::from_millis(20));
    }

    #[test]
    fn control_loop_entry_point_prints_usage_and_exits_clean_on_help() {
        // Reaches `control_loop()`, the CLI entry point, without needing a
        // real main-root or a `claude` binary — `--help` returns before
        // either is touched.
        let _: ExitCode = control_loop(&["--help"]);
    }

    #[test]
    fn control_loop_entry_point_fails_without_required_role() {
        let _: ExitCode = control_loop(&[]);
    }

    // ── CLI parsing ──────────────────────────────────────────────────────

    #[test]
    fn role_is_required() {
        let err = Options::parse(&[]).unwrap_err();
        assert!(err.contains("--role"), "{err}");
    }

    #[test]
    fn unknown_role_is_rejected() {
        let err = Options::parse(&["--role", "nope"]).unwrap_err();
        assert!(err.contains("unknown role"), "{err}");
    }

    #[test]
    fn trailing_flag_with_no_value_is_a_hard_error_not_a_spin() {
        let err = Options::parse(&["--role", "dispatch", "--interval"]).unwrap_err();
        assert!(err.contains("--interval requires a value"), "{err}");
    }

    #[test]
    fn zero_interval_is_rejected() {
        let err = Options::parse(&["--role", "dispatch", "--interval", "0"]).unwrap_err();
        assert!(err.contains("positive integer"), "{err}");
    }

    #[test]
    fn non_numeric_timeout_is_rejected() {
        let err = Options::parse(&["--role", "dispatch", "--timeout", "soon"]).unwrap_err();
        assert!(err.contains("positive integer"), "{err}");
    }

    #[test]
    fn defaults_match_the_bash_script() {
        let o = Options::parse(&["--role", "merge"]).unwrap();
        assert_eq!(o.role, Role::Merge);
        assert_eq!(o.interval, DEFAULT_INTERVAL);
        assert_eq!(o.timeout, DEFAULT_TIMEOUT);
        assert_eq!(o.max_iterations, DEFAULT_MAX_ITERATIONS);
        assert_eq!(o.max_consecutive_failures, DEFAULT_MAX_CONSECUTIVE_FAILURES);
        assert_eq!(o.turn_ceiling, DEFAULT_TURN_CEILING);
        assert!(!o.once);
    }

    #[test]
    fn unknown_flag_is_rejected() {
        let err = Options::parse(&["--role", "dispatch", "--nope"]).unwrap_err();
        assert!(err.contains("unknown argument"), "{err}");
    }

    // ── herding-limit-pause D4: occupancy pin for limit-paused jobs ──────
    //
    // The control loop's dispatch role derives slot occupancy and re-dispatch
    // eligibility via `bee herding occupancy` (backed by `wave_ledger::live_worker_count`).
    // Occupancy derives purely from unresolved ledger rows (`outcome: None`) crossed
    // against the backend's live pane list (`herdr pane list`). Because a limit-paused
    // worker's pane stays alive (hlp-1 never closes it on PausedLimit) and its ledger row
    // remains unresolved, the intersection in `live_worker_count` already counts the
    // paused job as OCCUPYING its slot. No new dispatch logic is required in
    // control_loop.rs; the inference holds directly from the ledger-pane cross-check.
    #[test]
    fn paused_limit_job_with_live_pane_counts_as_occupying_slot() {
        let tmp = tempfile::tempdir().unwrap();
        let main_root = tmp.path();
        let bee_dir = main_root.join(".bee");
        std::fs::create_dir_all(&bee_dir).unwrap();

        // 1. Dispatch recorded worker row with outcome: None
        let worker = super::super::wave_ledger::WorkerRow {
            name: "job-paused-1".to_string(),
            pane_id: "w1:p3".to_string(),
            worktree: main_root.join("work").display().to_string(),
            task: "do task".to_string(),
            outcome: None,
            evidence: None,
        };
        let row = super::super::wave_ledger::WaveRow {
            wave_id: "job-paused-1".to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
            workers: vec![worker],
        };
        super::super::wave_ledger::append_wave(main_root, &row).unwrap();

        // 2. The job is stamped paused_limit in job.json
        let job_dir = super::super::mailbox::mailbox_dir(&bee_dir, "job-paused-1");
        std::fs::create_dir_all(&job_dir).unwrap();
        let job = serde_json::json!({
            "job_id": "job-paused-1",
            "task": "do task",
            "cwd": main_root.join("work").display().to_string(),
            "round": 1,
            "pane_id": "w1:p3",
            "kind": "claude",
            "paused_limit_at": "2026-08-20T12:00:00Z",
            "limit_reset_hint": "You've hit your session limit",
        });
        std::fs::write(
            super::super::mailbox::job_path(&bee_dir, "job-paused-1"),
            serde_json::to_string(&job).unwrap(),
        )
        .unwrap();

        // 3. Live panes includes the paused worker's pane
        let mut live_panes = std::collections::HashSet::new();
        live_panes.insert("w1:p3".to_string());

        // 4. Verify occupancy counts the paused job as occupied (Live(1))
        let occ = super::super::wave_ledger::live_worker_count(
            main_root,
            Some(&live_panes),
            0,
            super::super::wave_ledger::DEFAULT_STALE_AFTER_MS,
        );
        assert_eq!(occ, super::super::wave_ledger::Occupancy::Live(1));
    }
}
