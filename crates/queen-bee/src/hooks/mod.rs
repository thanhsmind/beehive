//! hooks — ported hook handlers, dispatched by `queen-bee hook <name>`
//! (rust-port-7, CONTEXT.md D2: "hooks invoke the same binary (subcommand
//! per lifecycle event), not separate scripts").
//!
//! Only the two trivial hooks are ported this cell: `tools-logger`
//! (`.bee/bin/hooks/bee-tools-logger.mjs`) and `codex-subagent-audit`
//! (`.bee/bin/hooks/bee-codex-subagent-audit.mjs`). The other 7 `bee-*.mjs`
//! hooks land in later slices (per the epic map, Slice 1).

pub mod codex_subagent_audit;
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
        other => {
            eprintln!("queen-bee: unknown hook \"{other}\"");
            2
        }
    }
}
