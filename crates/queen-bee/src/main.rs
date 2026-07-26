//! queen-bee: the single compiled binary that replaces bee.mjs + hooks in
//! host repos (CONTEXT.md D2).
//!
//! Slice 0 scope: workspace skeleton only (`ping`, `--version`). Slice 1
//! (rust-port-7) adds the `hook <name>` runtime: `queen-bee hook tools-logger`
//! and `queen-bee hook codex-subagent-audit` (the two trivial hooks); the
//! remaining 7 `bee-*.mjs` hooks and the full CLI surface (116 defs / 19
//! groups) land in later slices. See `queen_bee::hooks` for the dispatch and
//! `queen_bee::adapter` for the shared runtime port of
//! `.bee/bin/hooks/adapter.mjs`.

use std::env;
use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        Some("ping") => {
            println!("pong");
            ExitCode::SUCCESS
        }
        Some("--version") => {
            println!("queen-bee {}", bee_core::VERSION);
            ExitCode::SUCCESS
        }
        Some("hook") => run_hook_command(&args[1..]),
        _ => {
            eprintln!("usage: queen-bee <ping|--version|hook NAME>");
            ExitCode::FAILURE
        }
    }
}

fn run_hook_command(rest: &[String]) -> ExitCode {
    let Some(name) = rest.first() else {
        eprintln!("usage: queen-bee hook <name> [--source plugin|repo]");
        return ExitCode::FAILURE;
    };
    let argv = &rest[1..];

    let mut raw_stdin = String::new();
    // A stdin read failure is treated exactly like empty stdin (the
    // adapter's own tolerant-empty-payload path) — never a crash exit,
    // matching the fail-open contract every hook operates under.
    let _ = std::io::stdin().read_to_string(&mut raw_stdin);

    let code = queen_bee::hooks::run_hook(name, argv, &raw_stdin);
    u8::try_from(code).map(ExitCode::from).unwrap_or(ExitCode::FAILURE)
}
