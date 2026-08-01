// bee — Rust front door for the bee harness (strangler port, plans/rust-port.md).
//
// Routing rule: argv shapes a native verb probe accepts are served in Rust;
// everything else is delegated verbatim to `node packages/bee/bee.mjs` with
// argv, stdin, stdout, stderr, and exit code passed through untouched
// (contract C2: byte-identical output). A native probe that returns None has
// produced NO output — the Node run owns the command end to end.

mod devtools;
mod fsutil;
mod herding;
mod hooks;
mod integration_queue;
mod js_fallback;
mod jsjson;
mod lease_store;
mod lock;
mod onboard;
mod path_identity;
mod registry;
mod roots;
mod router;
mod state;
mod verbs;

use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    // t0 covers argv parsing too, mirroring bee.mjs's direct-run wall timer.
    let t0 = Instant::now();
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();

    match router::try_native(&args, t0) {
        Some(code) => code,
        None => js_fallback::delegate(&args),
    }
}
