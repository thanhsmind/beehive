//! queen-bee: the single compiled binary that replaces bee.mjs + hooks in
//! host repos (CONTEXT.md D2).
//!
//! Slice 0 scope: workspace skeleton only. `ping` and `--version` prove the
//! binary builds, links against `bee-core`, and runs; the full CLI surface
//! (116 defs / 19 groups, per `command-registry.mjs`) lands in later slices.

use std::env;
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
        _ => {
            eprintln!("usage: queen-bee <ping|--version>");
            ExitCode::FAILURE
        }
    }
}
