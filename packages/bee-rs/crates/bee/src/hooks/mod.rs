// hooks — `bee hook <name>` subcommands. Each hook's stdin/stdout/exit
// contract is fixed by hooks.json wiring.
//
// stdin is read ONCE here and handed to the hook impl. A hook that meets an
// edge it cannot decide returns `Outcome::Delegate`, which means UNDECIDABLE:
// `emit_undecidable` below resolves it the only way a hook may — fail OPEN
// (exit 0, the tool call proceeds) with a VISIBLE diagnostic on stderr.
// Failing closed on infrastructure would let a hook bug block every tool call
// in a session; failing open silently would let a guard stop guarding
// without anyone noticing. Loud and open is the only safe pair.

pub mod activity;
pub mod adapter;
pub mod chain_nudge;
pub mod cli_shape;
pub mod codex_subagent_audit;
pub mod compaction;
pub mod model_guard;
pub mod prompt_context;
pub mod session_close;
pub mod session_init;
pub mod session_preamble;
pub mod state_sync;
pub mod tools_logger;
pub mod write_guard;

use std::ffi::OsString;
use std::process::ExitCode;

pub enum Outcome {
    Done(ExitCode),
    /// This hook cannot decide the payload it was given. Native code must have
    /// produced NO output before returning this; `emit_undecidable` then fails
    /// open, loudly. (Named `Delegate` for its whole history — kept so the
    /// hundreds of sites that return it keep reading the same, and because
    /// "delegate" still describes what happens: the decision is handed back to
    /// the host, which is now the only place left to hand it.)
    Delegate,
}

fn read_stdin_once() -> Vec<u8> {
    use std::io::Read;
    let mut buf = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut buf);
    buf
}

/// A hook that cannot decide fails OPEN and SAYS SO.
///
/// The two possible choices are both worse if taken silently: a non-zero
/// exit on a PreToolUse hook BLOCKS the tool call (a guard bug would freeze a
/// whole session), and a silent exit 0 lets a guard stop guarding with
/// nothing in the transcript to show for it. So: exit 0, with one line on
/// stderr naming the hook and the payload shape it could not decide. Same
/// posture the rendered hook commands take when the binary is missing
/// (`bee: hook binary missing`) — visible fail-open, spec R2.
///
/// `BEE_HOOK_NO_DELEGATE` stays: it is the test tripwire that proves a hook ran
/// NATIVE, and it turns this arm into a loud exit 42 that no fixture can
/// mistake for success.
fn emit_undecidable(name: &str) -> ExitCode {
    if std::env::var_os("BEE_HOOK_NO_DELEGATE").is_some() {
        eprintln!("bee(rs): hook {name} UNDECIDABLE (BEE_HOOK_NO_DELEGATE tripwire)");
        return ExitCode::from(42);
    }
    eprintln!(
        "bee: hook {name} could not decide this payload — allowing the operation (fail-open).          The guard did NOT run on it."
    );
    ExitCode::SUCCESS
}

/// The hook names `bee hook <name>` dispatches to. Kept as one list so the
/// usage line and the dispatch match arm can never drift apart silently —
/// add a hook to both, or the usage line lies.
const HOOK_NAMES: [&str; 10] = [
    "tools-logger",
    "activity",
    "codex-subagent-audit",
    "chain-nudge",
    "state-sync",
    "prompt-context",
    "session-init",
    "session-close",
    "model-guard",
    "write-guard",
];

/// The herded-worker marker (herding/run.rs D2): set and non-empty means
/// this process runs inside a pane `bee herding run` opened.
pub(crate) fn herding_worker_marker_set() -> bool {
    std::env::var("BEE_HERDING_WORKER").is_ok_and(|v| !v.is_empty())
}

/// herding-activity-hook D1, as one testable predicate: under the marker,
/// which hook names still exit 0 before stdin is read? Every one of them
/// except `activity` — including names this build does not know, because an
/// unknown name in a herded pane is still a hook invocation the pane must
/// not answer.
fn marker_short_circuits(name: &str) -> bool {
    name != "activity"
}

fn print_hook_usage() {
    println!("usage: bee hook <name> [args...]");
    println!();
    println!("hooks:");
    for hook_name in HOOK_NAMES {
        println!("  {hook_name}");
    }
}

/// `BEE_HERDING_WORKER` and `BEE_HERDING_JOB_ID` are PROCESS environment,
/// and cargo runs this crate's tests on parallel threads of ONE process.
/// Every test that reads or writes them takes THIS lock — here and in
/// `hooks::activity`'s tests — because two module-local mutexes would not
/// serialize against each other. Same shape as
/// `verbs/status_full/tests.rs`'s `session_env_lock`.
#[cfg(test)]
pub(crate) fn herding_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Dispatch `bee hook <name> [args...]`. Returns None when argv is not a
/// hook invocation at all.
pub fn try_native(args: &[OsString]) -> Option<ExitCode> {
    if args.first()?.to_str()? != "hook" {
        return None;
    }
    let name = args.get(1).and_then(|a| a.to_str()).unwrap_or("").to_string();
    if name.is_empty() {
        // Bare `bee hook`: no name given. Print usage and exit deliberately —
        // nonzero (this is a missing-argument error) but never a panic.
        print_hook_usage();
        return Some(ExitCode::FAILURE);
    }
    if name == "--help" || name == "-h" {
        // `bee hook --help`: print usage and exit 0 — the user asked, not erred.
        print_hook_usage();
        return Some(ExitCode::SUCCESS);
    }
    // herding-worker-standalone D3: a herded worker pane carries
    // BEE_HERDING_WORKER=1 (herding/run.rs D2). Every hook invocation in
    // that pane exits 0 silently, before stdin is read or any hook runs —
    // the worker's posture is already fully-open (herding-adopt D7), so it
    // gets zero bee preamble, zero guards, zero nudges.
    //
    // herding-activity-hook D1 cuts ONE hole in that: `activity` passes
    // through to its handler. It is the only hook that can never deny,
    // never prints, and always exits 0 — so the guard silence D3 buys is
    // untouched, and the pane gains the one thing the orchestrator needs
    // back: its own state, written into the job mailbox (D2).
    if herding_worker_marker_set() && marker_short_circuits(&name) {
        return Some(ExitCode::SUCCESS);
    }
    let rest: Vec<String> = args
        .get(2..)
        .unwrap_or(&[])
        .iter()
        .filter_map(|a| a.to_str().map(str::to_string))
        .collect();
    let stdin = read_stdin_once();
    let stdin_str = String::from_utf8_lossy(&stdin).into_owned();
    let outcome = match name.as_str() {
        "tools-logger" => tools_logger::run(&rest, &stdin_str),
        "activity" => activity::run(&rest, &stdin_str),
        "codex-subagent-audit" => codex_subagent_audit::run(&rest, &stdin_str),
        "chain-nudge" => chain_nudge::run(&rest, &stdin_str),
        "state-sync" => state_sync::run(&rest, &stdin_str),
        "prompt-context" => prompt_context::run(&rest, &stdin_str),
        "session-init" => session_init::run(&rest, &stdin_str),
        "session-close" => session_close::run(&rest, &stdin_str),
        "model-guard" => model_guard::run(&rest, &stdin_str),
        "write-guard" => write_guard::run(&rest, &stdin_str),
        _ => {
            eprintln!("bee hook: unknown hook \"{name}\"");
            return Some(ExitCode::FAILURE);
        }
    };
    Some(match outcome {
        Outcome::Done(code) => code,
        Outcome::Delegate => emit_undecidable(&name),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bare `bee hook` (no name argument) must not panic — it must print
    /// usage and exit cleanly, nonzero because no hook name was given.
    #[test]
    fn bare_hook_prints_usage_and_does_not_panic() {
        let args = vec![OsString::from("hook")];
        let code = try_native(&args).expect("bee hook is a hook invocation");
        assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::FAILURE));
    }

    /// `bee hook --help` must print usage and exit 0 — the user asked for
    /// help, that is not an error.
    #[test]
    fn hook_help_prints_usage_and_exits_zero() {
        let args = vec![OsString::from("hook"), OsString::from("--help")];
        let code = try_native(&args).expect("bee hook --help is a hook invocation");
        assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::SUCCESS));
    }

    #[test]
    fn hook_short_help_flag_also_exits_zero() {
        let args = vec![OsString::from("hook"), OsString::from("-h")];
        let code = try_native(&args).expect("bee hook -h is a hook invocation");
        assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::SUCCESS));
    }

    /// The ONE process-wide lock for the herding env vars — shared with
    /// `hooks::activity`'s tests, which set the same variables.
    fn herding_worker_env_lock() -> std::sync::MutexGuard<'static, ()> {
        super::herding_env_lock()
    }

    /// herding-worker-standalone D3: under the marker, a hook invocation
    /// exits 0 silently — no dispatch, no output. An unknown hook name
    /// would otherwise dispatch to `ExitCode::FAILURE` (see the
    /// `_ =>` arm), so a SUCCESS here proves the marker short-circuited
    /// before dispatch.
    #[test]
    fn under_the_marker_a_hook_invocation_exits_zero_silently() {
        let _guard = herding_worker_env_lock();
        let prior = std::env::var_os("BEE_HERDING_WORKER");
        // SAFETY: `herding_worker_env_lock` serializes every test in this
        // module that touches this var.
        unsafe { std::env::set_var("BEE_HERDING_WORKER", "1") };

        let args = vec![OsString::from("hook"), OsString::from("some-unknown-hook")];
        let code = try_native(&args).expect("bee hook <name> is a hook invocation");
        assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::SUCCESS));

        // SAFETY: see above.
        match prior {
            Some(v) => unsafe { std::env::set_var("BEE_HERDING_WORKER", v) },
            None => unsafe { std::env::remove_var("BEE_HERDING_WORKER") },
        }
    }

    /// herding-activity-hook D1: the hole in the marker exit is EXACTLY one
    /// name wide. Asserted over the whole `HOOK_NAMES` list rather than one
    /// sampled guard — a predicate tested on a single state is a law with a
    /// hole (docs/knowledge/patterns/20260713).
    #[test]
    fn under_the_marker_only_activity_passes_through_to_its_handler() {
        assert!(!marker_short_circuits("activity"), "activity must reach its handler");
        for hook_name in HOOK_NAMES {
            if hook_name == "activity" {
                continue;
            }
            assert!(
                marker_short_circuits(hook_name),
                "{hook_name} must still exit 0 before stdin is read under the marker"
            );
        }
        // An unknown name is not a hole either: it never reaches dispatch.
        assert!(marker_short_circuits("some-unknown-hook"));
    }

    /// The guard hooks stay silent through the real entry point too:
    /// `tools-logger` under the marker returns SUCCESS from the early exit,
    /// which is the only path in `try_native` that produces no output at
    /// all — dispatch is never reached, so no hook prints and no stdin is
    /// read.
    #[test]
    fn under_the_marker_a_guard_hook_exits_zero_without_dispatch() {
        let _guard = herding_worker_env_lock();
        let prior = std::env::var_os("BEE_HERDING_WORKER");
        // SAFETY: `herding_worker_env_lock` serializes every test that
        // touches this var.
        unsafe { std::env::set_var("BEE_HERDING_WORKER", "1") };

        for hook_name in ["tools-logger", "write-guard", "model-guard"] {
            let args = vec![OsString::from("hook"), OsString::from(hook_name)];
            let code = try_native(&args).expect("bee hook <name> is a hook invocation");
            assert_eq!(
                format!("{code:?}"),
                format!("{:?}", ExitCode::SUCCESS),
                "{hook_name} must exit 0 silently under the marker"
            );
        }

        // SAFETY: see above.
        match prior {
            Some(v) => unsafe { std::env::set_var("BEE_HERDING_WORKER", v) },
            None => unsafe { std::env::remove_var("BEE_HERDING_WORKER") },
        }
    }

    /// herding-worker-standalone D3: without the marker, the same
    /// invocation reaches dispatch — an unknown hook name hits the `_ =>`
    /// arm and exits `FAILURE`, proving dispatch ran rather than the
    /// marker's early return.
    #[test]
    fn without_the_marker_the_same_invocation_reaches_dispatch() {
        let _guard = herding_worker_env_lock();
        let prior = std::env::var_os("BEE_HERDING_WORKER");
        // SAFETY: see `under_the_marker_a_hook_invocation_exits_zero_silently`.
        unsafe { std::env::remove_var("BEE_HERDING_WORKER") };

        let args = vec![OsString::from("hook"), OsString::from("some-unknown-hook")];
        let code = try_native(&args).expect("bee hook <name> is a hook invocation");
        assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::FAILURE));

        // SAFETY: see above.
        match prior {
            Some(v) => unsafe { std::env::set_var("BEE_HERDING_WORKER", v) },
            None => unsafe { std::env::remove_var("BEE_HERDING_WORKER") },
        }
    }
}
