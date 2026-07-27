//! THE REGISTRATION TABLE (rpl-1).
//!
//! This file is the single place a ported command group is wired in. Landing
//! a group is exactly one `register` line here — `main.rs` and `dispatch.rs`
//! never change for it. That is the whole point of the seam: a red in a group
//! is a red in that group, never a red in the dispatcher.
//!
//! Adding a group looks like:
//!
//! ```ignore
//! d.register(crate::ledger::intent::group()).expect("intent group");
//! ```
//!
//! Slice 4 (`rust-port-ledgers`) lands `intent`, `capture`, `decisions`,
//! `backlog`, `reviews` and `feedback` through here, one cell each. `rpl-1`
//! deliberately registered NONE of them: the seam was proven on its own
//! first, so the first group's parity run could not be confused with a
//! harness fault. `rpl-2` is that first group — `intent`, added below as
//! exactly one line, with no edit to `main.rs` or `dispatch.rs`'s dispatch
//! logic, which is the seam's whole claim being cashed.

use crate::dispatch::Dispatcher;

/// Build the process-wide dispatcher. Registration failures are a programmer
/// error in this file (duplicate group or verb), so they panic loudly at
/// startup rather than silently shadowing a handler.
pub fn dispatcher() -> Dispatcher {
    let mut d = Dispatcher::new();
    register_all(&mut d);
    d
}

/// Every ported group, in registration order. ONE LINE PER GROUP.
pub fn register_all(d: &mut Dispatcher) {
    d.register(crate::ledger::intent::group()).expect("intent group");
    d.register(crate::ledger::capture::group()).expect("capture group");
    d.register(crate::ledger::decisions::group()).expect("decisions group");
    // rpl-6/rpl-9: backlog
    // rpl-7: reviews
    // rpl-8: feedback
}
