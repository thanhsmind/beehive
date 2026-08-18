//! The worker-backend trait: start a worker, read its status, send it a
//! task, read its output. The trait's status model is the choreography's
//! own — `Ready`, `Working`, `Blocked`, `Finished`, `Unverifiable` — with
//! `Unverifiable` a first-class value, never coerced to an error (D7).
//!
//! The trait is also the test seam: the whole choreography becomes
//! testable against a fake backend (`fake::FakeBackend`), with no running
//! herdr server and no naming of herdr in this crate.

use crate::wave::WorkerSpec;

pub mod fake;

/// One worker's current status, as read from a backend. Exactly the five
/// states the choreography needs — no more, no fewer (D7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerStatus {
    /// The worker exists, is addressable, and is not currently processing
    /// a dispatched task — it can be sent to.
    Ready,
    /// The worker is actively processing a task already sent to it.
    Working,
    /// The worker exists but cannot currently be sent to, for a reason
    /// outside this wave's control (for example the backend reports it
    /// waiting on something else).
    Blocked,
    /// The backend reports this worker's most recent task complete.
    ///
    /// **This must never be read as proof that work happened.** In the
    /// first backend that implements this trait (herdr, driving the
    /// `agent_status` field), the distinction the backend actually
    /// surfaces between a `Ready`-shaped reading and a `Finished`-shaped
    /// reading tracks the individual pane's own UI-focus field, not work
    /// completion: a worker whose pane has just been "seen" again reads
    /// back as `Ready`-shaped even though it only just settled, and a
    /// worker that settles while its pane goes unseen reads
    /// `Finished`-shaped for what is otherwise the same underlying
    /// settle event. See `skills/bee-herding/references/spawn-proof.md`,
    /// Step 4 and Takeaway 3, for the live round trip this was observed
    /// in. A caller that needs real completion proof, not a focus
    /// artifact, compares a `CompletionSignal` against a `Baseline`
    /// instead of trusting this variant.
    Finished,
    /// The status could not be determined: the lookup itself failed, the
    /// backend's response was missing the status field, or the backend's
    /// response held a value outside these five states. `Unverifiable` is
    /// a first-class value returned by `WorkerBackend::status`, never an
    /// `Err`, never wrapped in `Option`, and never silently coerced to
    /// any other variant — fail-closed status is Ordering Invariant 4.
    Unverifiable,
}

/// A transcript/output snapshot for one worker, captured BEFORE any task
/// is dispatched to it. The anchor every completion check is measured
/// against (Terms: Baseline) — necessary precisely because
/// `WorkerStatus::Finished` alone cannot be trusted as completion proof
/// (see its documentation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Baseline(String);

impl Baseline {
    /// Captures `output` as a baseline. Call this BEFORE the worker it
    /// describes is dispatched to — a baseline taken after dispatch can
    /// no longer distinguish "was already there" from "arrived because of
    /// this send" (Ordering Invariant 1, fast completion; Ordering
    /// Invariant 2, stale marker rejection).
    pub fn capture(output: impl Into<String>) -> Self {
        Self(output.into())
    }

    /// The captured text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One dispatch's completion evidence: the marker embedded in that
/// dispatch's task text, and the worker's output read back afterward.
/// Carries what a completion check compares — see `confirmed_against`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionSignal {
    /// The marker text embedded in the dispatched task, split so that
    /// echoing the prompt back cannot reproduce it (Terms: Completion
    /// marker).
    pub marker: String,
    /// The worker's output, read after dispatch.
    pub output: String,
}

impl CompletionSignal {
    /// True only when `marker` is present in `output` AND absent from
    /// `baseline` — the exact shape of Ordering Invariant 2 (stale marker
    /// rejection): a marker that was already there before this dispatch
    /// proves nothing about this dispatch.
    pub fn confirmed_against(&self, baseline: &Baseline) -> bool {
        self.output.contains(&self.marker) && !baseline.as_str().contains(&self.marker)
    }
}

/// Start a worker, read its status, send it a task, read its output.
/// Deliberately small and entirely synchronous (D9 fixes `std::thread`
/// plus channels for wave concurrency; no async signature belongs on a
/// single backend call either).
///
/// Shaped as the crate's test seam (D7): every method here has a natural
/// fake (`fake::FakeBackend`), so the choreography that drives this trait
/// is testable with no running external process.
pub trait WorkerBackend {
    /// Starts `worker`, making it addressable by name for the other three
    /// methods. Does not send it a task.
    fn start(&self, worker: &WorkerSpec) -> anyhow::Result<()>;

    /// Reads `worker`'s current status. Never fails: a lookup that fails
    /// outright, a response missing the status field, or a response
    /// holding a value outside `WorkerStatus` all arrive here as
    /// `WorkerStatus::Unverifiable`, not as an `Err` — fail-closed status
    /// is a property of this method's return type, not of its caller's
    /// discipline (D7, Ordering Invariant 4).
    fn status(&self, worker: &str) -> WorkerStatus;

    /// Sends `task` to `worker`. An `Err` here is that one target's own
    /// failure; the choreography built on this trait is what isolates it
    /// from targets already dispatched to (Ordering Invariant 5,
    /// partial-failure isolation) — this method itself does no isolating.
    fn send(&self, worker: &str, task: &str) -> anyhow::Result<()>;

    /// Reads `worker`'s current output/transcript — the substrate a
    /// `Baseline` is captured from before dispatch, and a
    /// `CompletionSignal` is built from after it.
    fn read_output(&self, worker: &str) -> anyhow::Result<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_model_has_exactly_five_states() {
        // Exhaustive match: this fails to compile the moment a variant is
        // added or removed, which is the point — the status model is
        // fixed by D7, not open for a caller to extend.
        fn describe(status: WorkerStatus) -> &'static str {
            match status {
                WorkerStatus::Ready => "ready",
                WorkerStatus::Working => "working",
                WorkerStatus::Blocked => "blocked",
                WorkerStatus::Finished => "finished",
                WorkerStatus::Unverifiable => "unverifiable",
            }
        }
        assert_eq!(describe(WorkerStatus::Ready), "ready");
        assert_eq!(describe(WorkerStatus::Unverifiable), "unverifiable");
    }

    #[test]
    fn completion_signal_confirmed_only_when_present_now_and_absent_from_baseline() {
        let baseline = Baseline::capture("hello\n$ ");
        let signal = CompletionSignal {
            marker: "XMARK-123".to_string(),
            output: "hello\n$ XMARK-123 done".to_string(),
        };
        assert!(signal.confirmed_against(&baseline));
    }

    #[test]
    fn completion_signal_rejects_a_marker_already_present_in_the_baseline() {
        // Ordering Invariant 2: a marker present before this dispatch is
        // never credited to it, even if it's also present in the output
        // read afterward.
        let baseline = Baseline::capture("hello\n$ XMARK-123 leftover");
        let signal = CompletionSignal {
            marker: "XMARK-123".to_string(),
            output: "hello\n$ XMARK-123 leftover".to_string(),
        };
        assert!(!signal.confirmed_against(&baseline));
    }

    #[test]
    fn completion_signal_rejects_a_marker_absent_from_the_current_output() {
        let baseline = Baseline::capture("hello\n$ ");
        let signal = CompletionSignal {
            marker: "XMARK-123".to_string(),
            output: "hello\n$ still waiting".to_string(),
        };
        assert!(!signal.confirmed_against(&baseline));
    }
}
