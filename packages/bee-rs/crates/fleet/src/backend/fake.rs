//! A fault-injecting fake `WorkerBackend`, for driving the choreography's
//! ordering invariants deterministically — no real backend process, no
//! sleeping, no dependence on real time. Per D7, this trait IS the
//! crate's only test seam for an external command, and this fake is that
//! seam's test support.
//!
//! The source `herdr-agent-comms` skill's own test harness needed an
//! atomic write-then-rename around its on-disk worker state, because
//! concurrent waiters read that state file mid-write and a torn read
//! looked like corruption or a stale value (the tmpfile handoff layer D9
//! deliberately does not recreate;
//! `docs/history/research/herdr-orchestrator-distill.md`). `FakeBackend`
//! has no such hazard by construction: its state lives in a single
//! in-process `Mutex<HashMap<..>>`, never on disk, so a reader can only
//! ever observe the state before or after one complete mutation, never a
//! torn write — the write-then-rename discipline that hazard forced has
//! no analogue to build here, because the mutex already gives every
//! access an all-or-nothing view.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use super::{WorkerBackend, WorkerStatus};
use crate::wave::WorkerSpec;

/// One simulated raw outcome the fake backend can be told to return from a
/// status lookup. Every non-`Value` variant collapses to
/// `WorkerStatus::Unverifiable` when read through `WorkerBackend::status`
/// — modelling the three raw failure shapes the fail-closed status law
/// (D7, Ordering Invariant 4) must survive: a lookup that fails outright,
/// a response missing the status field, and a response whose status
/// wasn't one of the five known values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawStatus {
    /// A normal, well-formed status reading.
    Value(WorkerStatus),
    /// The lookup itself failed (the simulated equivalent of a backend
    /// call returning an error).
    LookupFailed,
    /// The lookup succeeded but the status field was missing/null.
    NullField,
    /// The lookup succeeded but the status value wasn't one of the five
    /// known states.
    OffEnum,
}

#[derive(Debug, Default)]
struct WorkerState {
    /// Statuses queued to be returned, oldest first — one per call to
    /// `status`. Lets a test drive a mid-sequence flip deterministically:
    /// push `Working` then `Blocked` and two calls see the transition.
    status_queue: VecDeque<RawStatus>,
    /// Returned once `status_queue` is empty, so a test doesn't have to
    /// keep re-queueing a settled value.
    steady_status: Option<RawStatus>,
    /// `send` outcomes queued to be returned, oldest first. An empty
    /// queue means `send` succeeds.
    send_queue: VecDeque<Result<(), String>>,
    /// The worker's current output/transcript, as `read_output` returns
    /// it.
    output: String,
    /// Whether `start` has been called for this worker. Recorded for
    /// tests to assert on; the fake never uses this to gate the other
    /// methods, because a test must be able to configure a worker's
    /// status — for example, already `Finished` — BEFORE `start` is ever
    /// called. That is exactly the fault this fake has to model: a
    /// worker that finishes before anyone starts watching it.
    started: bool,
}

/// A fault-injecting, in-memory `WorkerBackend`. Every method is a plain
/// `HashMap` lookup behind a `Mutex`, so nothing here sleeps and nothing
/// depends on real time — a test drives every transition explicitly by
/// calling the `schedule_*`/`set_*` methods below, then reads the result
/// through the `WorkerBackend` trait like any other backend.
#[derive(Debug, Default)]
pub struct FakeBackend {
    workers: Mutex<HashMap<String, WorkerState>>,
}

impl FakeBackend {
    /// Builds an empty fake with no workers registered yet. A worker's
    /// entry is created on first reference by any method below —
    /// `start` included — so a test may configure a worker's state
    /// before ever calling `start`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues `raw` to be the outcome of the next call to `status` for
    /// `worker`, after any statuses already queued. Call this twice with
    /// two different values to model a status that flips between one
    /// call and the next.
    pub fn schedule_status(&self, worker: &str, raw: RawStatus) {
        let mut workers = self.workers.lock().unwrap();
        workers
            .entry(worker.to_string())
            .or_default()
            .status_queue
            .push_back(raw);
    }

    /// Sets the status `status` returns for `worker` once its queue (set
    /// via `schedule_status`) is exhausted — the worker's settled value.
    /// Setting this before `start` is called is how a test models a
    /// worker that finishes before anyone starts watching it: the very
    /// first status read already sees the settled value, with no polling
    /// having happened yet.
    pub fn set_steady_status(&self, worker: &str, raw: RawStatus) {
        let mut workers = self.workers.lock().unwrap();
        workers.entry(worker.to_string()).or_default().steady_status = Some(raw);
    }

    /// Queues `result` to be the outcome of the next call to `send` for
    /// `worker`, after any results already queued.
    pub fn schedule_send_result(&self, worker: &str, result: Result<(), String>) {
        let mut workers = self.workers.lock().unwrap();
        workers
            .entry(worker.to_string())
            .or_default()
            .send_queue
            .push_back(result);
    }

    /// Sets the text `read_output` returns for `worker`.
    pub fn set_output(&self, worker: &str, output: impl Into<String>) {
        let mut workers = self.workers.lock().unwrap();
        workers.entry(worker.to_string()).or_default().output = output.into();
    }

    /// True once `start` has been called for `worker`; false for a
    /// worker never referenced, or referenced only through a
    /// `schedule_*`/`set_*` call.
    pub fn was_started(&self, worker: &str) -> bool {
        self.workers
            .lock()
            .unwrap()
            .get(worker)
            .map(|w| w.started)
            .unwrap_or(false)
    }
}

impl WorkerBackend for FakeBackend {
    fn start(&self, worker: &WorkerSpec) -> anyhow::Result<()> {
        let mut workers = self.workers.lock().unwrap();
        workers.entry(worker.name.clone()).or_default().started = true;
        Ok(())
    }

    fn status(&self, worker: &str) -> WorkerStatus {
        let mut workers = self.workers.lock().unwrap();
        let state = workers.entry(worker.to_string()).or_default();
        // A worker with nothing queued and no steady value set has never
        // been configured at all — that defaults to `Unverifiable`, not
        // to a safe-looking status, the same fail-closed discipline the
        // trait itself documents (D7, Ordering Invariant 4).
        let raw = state
            .status_queue
            .pop_front()
            .or_else(|| state.steady_status.clone())
            .unwrap_or(RawStatus::LookupFailed);
        match raw {
            RawStatus::Value(status) => status,
            RawStatus::LookupFailed | RawStatus::NullField | RawStatus::OffEnum => {
                WorkerStatus::Unverifiable
            }
        }
    }

    fn send(&self, worker: &str, _task: &str) -> anyhow::Result<()> {
        // The fake records nothing about the task's content — a test that
        // needs to assert on what a worker received wires that up through
        // `set_output` in the arrangement it configures.
        let mut workers = self.workers.lock().unwrap();
        let state = workers.entry(worker.to_string()).or_default();
        match state.send_queue.pop_front() {
            Some(Err(message)) => Err(anyhow::anyhow!(message)),
            Some(Ok(())) | None => Ok(()),
        }
    }

    fn read_output(&self, worker: &str) -> anyhow::Result<String> {
        let workers = self.workers.lock().unwrap();
        Ok(workers
            .get(worker)
            .map(|w| w.output.clone())
            .unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_never_configured_worker_reads_unverifiable_not_a_safe_default() {
        let backend = FakeBackend::new();
        assert_eq!(backend.status("ghost"), WorkerStatus::Unverifiable);
    }

    #[test]
    fn a_failed_lookup_reads_unverifiable() {
        let backend = FakeBackend::new();
        backend.schedule_status("w1", RawStatus::LookupFailed);
        assert_eq!(backend.status("w1"), WorkerStatus::Unverifiable);
    }

    #[test]
    fn a_null_status_field_reads_unverifiable() {
        let backend = FakeBackend::new();
        backend.schedule_status("w1", RawStatus::NullField);
        assert_eq!(backend.status("w1"), WorkerStatus::Unverifiable);
    }

    #[test]
    fn an_off_enum_status_value_reads_unverifiable() {
        let backend = FakeBackend::new();
        backend.schedule_status("w1", RawStatus::OffEnum);
        assert_eq!(backend.status("w1"), WorkerStatus::Unverifiable);
    }

    #[test]
    fn status_can_flip_deterministically_between_one_call_and_the_next() {
        let backend = FakeBackend::new();
        backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Working));
        backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Blocked));
        assert_eq!(backend.status("w1"), WorkerStatus::Working);
        assert_eq!(backend.status("w1"), WorkerStatus::Blocked);
    }

    #[test]
    fn steady_status_serves_every_call_once_the_queue_is_exhausted() {
        let backend = FakeBackend::new();
        backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Working));
        backend.set_steady_status("w1", RawStatus::Value(WorkerStatus::Ready));
        assert_eq!(backend.status("w1"), WorkerStatus::Working);
        assert_eq!(backend.status("w1"), WorkerStatus::Ready);
        assert_eq!(backend.status("w1"), WorkerStatus::Ready);
    }

    #[test]
    fn a_worker_that_finishes_before_anyone_starts_watching_it_is_still_detected() {
        // No poll loop exists yet — that's the choreography's job, the
        // next cell — but the fake can already model the fault:
        // configure the settled status FIRST, `start` the worker SECOND,
        // and the very first status read already sees `Finished`,
        // matching Ordering Invariant 1 (fast completion).
        let backend = FakeBackend::new();
        backend.set_steady_status("w1", RawStatus::Value(WorkerStatus::Finished));
        assert!(!backend.was_started("w1"));
        backend.start(&WorkerSpec::new("w1", "task")).unwrap();
        assert!(backend.was_started("w1"));
        assert_eq!(backend.status("w1"), WorkerStatus::Finished);
    }

    #[test]
    fn a_send_can_be_made_to_fail_then_succeed() {
        let backend = FakeBackend::new();
        backend.schedule_send_result("w1", Err("connection refused".to_string()));
        let first = backend.send("w1", "do it");
        assert!(first.is_err());
        let second = backend.send("w1", "do it");
        assert!(second.is_ok());
    }

    #[test]
    fn read_output_returns_whatever_was_set_and_defaults_to_empty() {
        let backend = FakeBackend::new();
        assert_eq!(backend.read_output("ghost").unwrap(), "");
        backend.set_output("w1", "hello world");
        assert_eq!(backend.read_output("w1").unwrap(), "hello world");
    }
}
