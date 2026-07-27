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
//! deliberately registers NONE of them: the seam is proven on its own first,
//! so the first group's parity run cannot be confused with a harness fault.

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
pub fn register_all(_d: &mut Dispatcher) {
    // rpl-2: intent
    // rpl-3: capture
    // rpl-4/rpl-5: decisions
    // rpl-6/rpl-9: backlog
    // rpl-7: reviews
    // rpl-8: feedback
}
