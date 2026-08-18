//! The `Wave` value: a set of worker specs, timeouts, and a failure
//! policy, described as data rather than a sequence of calls (D11). A
//! wave is at most a handful of workers dispatched and waited on together,
//! aggregating to a single verdict.

use std::time::Duration;

/// One worker to include in a wave: the name it is addressed by through
/// the backend trait (`crate::backend::WorkerBackend`), and the task text
/// it is to be given once the choreography sends to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerSpec {
    /// The name the backend resolves this worker by. Distinct worker
    /// specs that resolve to the same underlying target must be deduped
    /// before dispatch (Ordering Invariant 8) — that dedupe is the
    /// choreography's job, not this type's.
    pub name: String,
    /// The task text this worker is sent once the choreography dispatches
    /// to it.
    pub task: String,
}

impl WorkerSpec {
    /// Builds a `WorkerSpec` from a name and a task.
    pub fn new(name: impl Into<String>, task: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            task: task.into(),
        }
    }
}

/// The timeouts that govern one wave's waiting phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaveTimeouts {
    /// The wall-clock ceiling on how long the choreography waits for one
    /// worker to move from `Working` to a terminal status (`Finished` or
    /// `Unverifiable`) after dispatch. Reproduces the ceiling GNU
    /// `timeout` gave the bash control loop this crate replaces (D9).
    pub worker_settle: Duration,
    /// How often the choreography re-reads a worker's status while
    /// waiting for it to settle. Keeping this short relative to
    /// `worker_settle` is what makes bounded working→finished polling
    /// possible (Ordering Invariant 7): a worker that settles well before
    /// the ceiling is not made to wait out the rest of it.
    pub poll_interval: Duration,
}

/// What a wave does when one of the targets it actually sent to fails to
/// reach a successful terminal status. Present as an enum from day one
/// (D11) so the shape carrying a scenario never needs to change when a
/// later variant is implemented — even though only `WaitForAll` is
/// implemented today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailurePolicy {
    /// Wait for every sent target to reach a terminal status — `Finished`
    /// or `Unverifiable` — before aggregating a verdict, and fail the
    /// whole wave if any sent target did not settle successfully
    /// (Ordering Invariant 6, mixed-result aggregation). The only variant
    /// the choreography implements today.
    WaitForAll,
    /// Stop waiting the moment any one sent target reaches a successful
    /// terminal status, and cancel the still-outstanding waits on the
    /// rest. **Not implemented yet** — only this variant and its meaning
    /// exist today, so a scenario can already declare this intent before
    /// the choreography is built to honour it.
    FirstSuccessCancelRest,
    /// Wait out every sent target's full settle window regardless of
    /// individual outcome, then report every target's result without
    /// failing the whole wave for any one target's failure — the caller
    /// reads the per-target results itself. **Not implemented yet**, for
    /// the same reason as `FirstSuccessCancelRest`.
    BestEffort,
}

/// A wave: a set of worker specs, timeouts, and a failure policy,
/// constructed as a single value rather than assembled through a sequence
/// of calls (D11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wave {
    /// The workers this wave dispatches to and waits on.
    pub workers: Vec<WorkerSpec>,
    /// The timeouts governing this wave's waiting phase.
    pub timeouts: WaveTimeouts,
    /// What this wave does if a sent target fails.
    pub failure_policy: FailurePolicy,
}

impl Wave {
    /// Builds a `Wave` from its three parts as a single value (D11) —
    /// never a sequence of `.add_worker()`-style calls.
    pub fn new(
        workers: Vec<WorkerSpec>,
        timeouts: WaveTimeouts,
        failure_policy: FailurePolicy,
    ) -> Self {
        Self {
            workers,
            timeouts,
            failure_policy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_is_constructed_as_one_value_holding_its_three_parts() {
        let wave = Wave::new(
            vec![
                WorkerSpec::new("alpha", "do the thing"),
                WorkerSpec::new("beta", "do the other thing"),
            ],
            WaveTimeouts {
                worker_settle: Duration::from_secs(30),
                poll_interval: Duration::from_millis(200),
            },
            FailurePolicy::WaitForAll,
        );

        assert_eq!(wave.workers.len(), 2);
        assert_eq!(wave.workers[0].name, "alpha");
        assert_eq!(wave.workers[0].task, "do the thing");
        assert_eq!(wave.timeouts.worker_settle, Duration::from_secs(30));
        assert_eq!(wave.timeouts.poll_interval, Duration::from_millis(200));
        assert_eq!(wave.failure_policy, FailurePolicy::WaitForAll);
    }

    #[test]
    fn failure_policy_carries_all_three_variants_from_day_one() {
        let variants = [
            FailurePolicy::WaitForAll,
            FailurePolicy::FirstSuccessCancelRest,
            FailurePolicy::BestEffort,
        ];
        assert_eq!(variants.len(), 3);
        assert_ne!(
            FailurePolicy::WaitForAll,
            FailurePolicy::FirstSuccessCancelRest
        );
        assert_ne!(FailurePolicy::WaitForAll, FailurePolicy::BestEffort);
        assert_ne!(
            FailurePolicy::FirstSuccessCancelRest,
            FailurePolicy::BestEffort
        );
    }
}
