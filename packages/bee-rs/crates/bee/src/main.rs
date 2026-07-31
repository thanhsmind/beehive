// bee — Rust front door for the bee harness (strangler port, plans/rust-port.md).
//
// Routing rule: a (group, verb) pair listed in router::PORTED is served natively;
// everything else is delegated verbatim to `node packages/bee/bee.mjs` with
// argv, stdin, stdout, stderr, and exit code passed through untouched (contract
// C2: byte-identical output). As verbs are ported (R3), they move from the
// fallback to PORTED one at a time, each behind a green diff-harness run.

mod js_fallback;
mod router;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();

    match router::route(&args) {
        router::Route::Native(cmd) => router::run_native(cmd, &args),
        router::Route::Delegate => js_fallback::delegate(&args),
    }
}
