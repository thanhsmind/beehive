// bee — native front door for the bee harness (plans/rust-port.md, R6 cutover).
//
// There is no second runtime. Until the cutover an argv shape no native probe
// claimed was handed to `node packages/bee/bee.mjs`; that tree is gone, so a
// probe returning None is now the END of the line and must SAY SO. Every door
// in the tree follows the same rule: refuse out loud, never fall silent.
//
// Routing rule (unchanged): a verb serves only the argv shapes its port has
// proven. What changed is the consequence of "not proven" — an emitted
// `unsupported command shape` with a non-zero exit instead of a delegation.

mod devtools;
mod fsutil;
mod herding;
mod hooks;
mod integration_queue;
mod jsjson;
mod lease_store;
mod link_invalid;
mod lock;
mod nested_checkout;
mod onboard;
mod path_identity;
mod registry;
mod roots;
mod router;
mod state;
mod verbs;
mod version;

use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    // t0 covers argv parsing too, mirroring bee.mjs's direct-run wall timer.
    let t0 = Instant::now();
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();

    match router::try_native(&args, t0) {
        Some(code) => code,
        None => router::emit_unsupported_shape(&args),
    }
}
