// hooks — `bee hook <name>` subcommands: the native replacements for the
// former `node .bee/bin/hooks/bee-<name>.mjs` wrapper scripts. hooks.json
// wiring is the only host-visible change (contract C3); each hook's
// stdin/stdout/exit contract is unchanged.
//
// stdin is read ONCE here and handed to the hook impl. A hook that meets an
// edge it cannot decide returns `Outcome::Delegate` — which, since the
// cutover, no longer means "re-run the .mjs wrapper" (there is none). It means
// UNDECIDABLE, and `emit_undecidable` below resolves it the only way a hook
// may: fail OPEN (exit 0, the tool call proceeds) with a VISIBLE diagnostic on
// stderr. Failing closed on infrastructure would let a hook bug block every
// tool call in a session; failing open silently would let a guard stop
// guarding without anyone noticing. Loud and open is the only safe pair.

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

/// CUTOVER: a hook that cannot decide fails OPEN and SAYS SO.
///
/// Before the cutover this spawned the `.mjs` wrapper with the same stdin.
/// There is no wrapper any more, and the two remaining choices are both worse
/// if taken silently: a non-zero exit on a PreToolUse hook BLOCKS the tool
/// call (a guard bug would freeze a whole session), and a silent exit 0 lets a
/// guard stop guarding with nothing in the transcript to show for it. So:
/// exit 0, with one line on stderr naming the hook and the payload shape it
/// could not decide. Same posture the rendered hook commands take when the
/// binary is missing (`bee: hook binary missing`) — visible fail-open, spec R2.
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

/// Dispatch `bee hook <name> [args...]`. Returns None when argv is not a
/// hook invocation at all.
pub fn try_native(args: &[OsString]) -> Option<ExitCode> {
    if args.first()?.to_str()? != "hook" {
        return None;
    }
    let name = args.get(1).and_then(|a| a.to_str()).unwrap_or("").to_string();
    let rest: Vec<String> = args[2..]
        .iter()
        .filter_map(|a| a.to_str().map(str::to_string))
        .collect();
    let stdin = read_stdin_once();
    let stdin_str = String::from_utf8_lossy(&stdin).into_owned();
    let outcome = match name.as_str() {
        "tools-logger" => tools_logger::run(&rest, &stdin_str),
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
