//! hooks — ported hook handlers, dispatched by `queen-bee hook <name>`
//! (rust-port-7, CONTEXT.md D2: "hooks invoke the same binary (subcommand
//! per lifecycle event), not separate scripts").
//!
//! rust-port-7/9/10 ported the four lighter hooks (`tools-logger`,
//! `codex-subagent-audit`, `write-guard`, `model-guard`); rust-port-17 adds
//! the two remaining heavy hooks whose dependencies are now all in Rust:
//! `chain-nudge` and `state-sync`. The other 5 `bee-*.mjs` hooks land in
//! later slices (per the epic map).

pub mod chain_nudge;
pub mod codex_subagent_audit;
pub mod model_guard;
pub mod state_sync;
pub mod tools_logger;
pub mod write_guard;

/// Runs the named hook with the given argv tail (everything after `hook
/// <name>`) and raw stdin text, returning the process exit code. An
/// unrecognized name is a caller/wiring error (never wired by this cell —
/// see CONTEXT.md D7's flip discipline), not a fail-open runtime condition,
/// so it exits non-zero with a usage message rather than silently exiting 0.
pub fn run_hook(name: &str, argv: &[String], raw_stdin: &str) -> i32 {
    match name {
        "tools-logger" => tools_logger::run(argv, raw_stdin),
        "codex-subagent-audit" => codex_subagent_audit::run(argv, raw_stdin),
        "write-guard" => write_guard::run(argv, raw_stdin),
        "model-guard" => model_guard::run(argv, raw_stdin),
        "chain-nudge" => chain_nudge::run(argv, raw_stdin),
        "state-sync" => state_sync::run(argv, raw_stdin),
        other => {
            eprintln!("queen-bee: unknown hook \"{other}\"");
            2
        }
    }
}
