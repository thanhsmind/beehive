//! `fleet` — a generic worker-coordination core.
//!
//! This crate knows workers, tasks, waiting, result collection and failure
//! aggregation. It does not know cells, lanes, worktrees, gates or proof —
//! any bee-shaped vocabulary here is a defect, not a convenience
//! (`docs/history/herding-orchestration/CONTEXT.md`, decision D2).
//!
//! `fleet` links into the shipped `bee` binary as a library dependency; it
//! is never a second shipped binary (D5). The crate boundary — this crate
//! declaring no dependency on `bee` — is the only mechanism that enforces
//! that separation, and is proven by the manifest boundary test in
//! `tests/manifest_boundary.rs`.
//!
//! Concurrency inside a wave uses `std::thread` and `std::sync::mpsc` only;
//! no async runtime or thread-pool crate is added here (D9).

/// The `Wave` value: a set of worker specs, timeouts, and a failure policy,
/// described as data rather than a sequence of calls (D11). A wave is at
/// most a handful of workers dispatched and waited on together, aggregating
/// to a single verdict.
pub mod wave;

/// The worker-backend trait: start a worker, read its status, send it a
/// task, read its output. The trait's status model is the choreography's
/// own — ready, working, blocked, finished, unverifiable — with
/// `unverifiable` a first-class value, never coerced to an error (D7).
///
/// The trait is also the test seam: the whole choreography becomes
/// testable against a fake backend, with no running herdr server and no
/// naming of herdr in this crate.
pub mod backend;

/// The choreography: the state machine that drives a wave of workers from
/// dispatch through waiting to a single aggregated verdict, using
/// `std::thread` fan-out and `std::sync::mpsc` for result collection (D9).
pub mod choreography {
    // Intentionally empty for now — the state machine lands in a later
    // cell, built and proven against a fake backend first.
}

#[cfg(test)]
mod tests {
    /// Proves the crate is wired into the workspace build, not merely
    /// present on disk.
    #[test]
    fn crate_is_wired_into_the_build() {
        assert_eq!(2 + 2, 4);
    }
}
