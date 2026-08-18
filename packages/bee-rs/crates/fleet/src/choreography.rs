//! The five-phase choreography (D3): resolve+dedupe every target to a
//! canonical identity, a fail-closed status filter, a baseline snapshot
//! of every surviving target before any dispatch, a per-target re-check
//! immediately before that target's own send, then a concurrent wait and
//! a folded-in aggregation. The ORDER of these five phases is the
//! specification — see
//! `docs/history/herding-orchestration/CONTEXT.md`, decision D3 and its
//! § Ordering Invariants, and `crates/fleet/tests/choreography.rs`, which
//! pins all eight of them plus the resolution-abort truth.
//!
//! Calls only `WorkerBackend` (D7) — never a concrete backend. Waiting is
//! concurrent across targets using `std::thread` and `std::sync::mpsc`
//! only (D9); no async runtime, no temporary-file handoff layer — each
//! waiter thread returns its `TargetOutcome` directly through the
//! channel.

use std::collections::HashSet;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::backend::{Baseline, CompletionSignal, WorkerBackend, WorkerStatus};
use crate::wave::{Wave, WaveTimeouts, WorkerSpec};

/// A wave's aggregated verdict, in named buckets rather than one boolean
/// — the source corpus recorded a real regression where a dropped target
/// was omitted from the final aggregation, so `is_success` below folds in
/// every bucket, not only the send/wait ones (Ordering Invariant 6).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WaveResult {
    /// Targets sent to and confirmed complete.
    pub succeeded: Vec<String>,
    /// Targets whose `WorkerBackend::start` failed during resolution
    /// (phase 1). Non-empty here means the WHOLE wave aborted before
    /// phase 2 ever ran for ANY target, resolved or not.
    pub resolution_failed: Vec<String>,
    /// Targets dropped by the bulk fail-closed status filter (phase 2),
    /// before any baseline was captured or anything was sent.
    pub unsafe_at_preflight: Vec<String>,
    /// Targets dropped because their per-target re-check, immediately
    /// before their own send (phase 4), found them no longer safe —
    /// flipped to `Working` or `Blocked` since phase 2 (Ordering
    /// Invariant 3).
    pub flipped_before_send: Vec<String>,
    /// Targets whose `WorkerBackend::send` returned `Err`. Isolated from
    /// targets dispatched earlier in the same fan-out (Ordering Invariant
    /// 5, partial-failure isolation).
    pub send_failed: Vec<String>,
    /// Targets sent to but never confirmed complete within
    /// `WaveTimeouts::worker_settle` — either because they stayed
    /// `Working`/`Blocked` the whole window, or because a terminal
    /// status was seen but its completion marker never confirmed against
    /// the baseline (Ordering Invariant 2, stale-marker rejection, is
    /// what keeps a stale marker from short-circuiting this into
    /// `succeeded`).
    pub timed_out: Vec<String>,
    /// Targets sent to whose status turned `Unverifiable` while waiting —
    /// fail-closed, never credited as success (Ordering Invariant 4).
    pub unverifiable_after_send: Vec<String>,
}

impl WaveResult {
    /// True only when every bucket other than `succeeded` is empty. A
    /// dropped target (never sent, for any reason) fails the wave
    /// exactly like a sent-and-failed one (Ordering Invariant 6): this
    /// method is the one place that folds both kinds in, so no caller
    /// can accidentally check only the send/wait buckets and miss a
    /// dropped target.
    pub fn is_success(&self) -> bool {
        self.resolution_failed.is_empty()
            && self.unsafe_at_preflight.is_empty()
            && self.flipped_before_send.is_empty()
            && self.send_failed.is_empty()
            && self.timed_out.is_empty()
            && self.unverifiable_after_send.is_empty()
    }
}

/// Ordering Invariant 8 (dedupe before preflight): collapse `workers` to
/// one entry per distinct `WorkerSpec::name`, keeping the first
/// occurrence's task text and the original order. Within this generic
/// core, `WorkerSpec::name` IS the canonical identity — the trait exposes
/// no separate name-to-canonical-id resolution, so a name and an alias
/// referring to the same backend-side target (for example a herdr pane
/// id) collapse only in a later, backend-specific phase, not here. This
/// MUST run before any status is read (phase 2) or anything is sent
/// (phase 4/5).
fn resolve_and_dedupe(workers: &[WorkerSpec]) -> Vec<WorkerSpec> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::with_capacity(workers.len());
    for worker in workers {
        if seen.insert(worker.name.clone()) {
            deduped.push(worker.clone());
        }
    }
    deduped
}

/// True for the two statuses safe to send to: `Ready`, and `Finished`
/// (which — per `WorkerStatus::Finished`'s own documentation — can mean
/// "settled since I was last looked at", not "currently busy", so it is
/// as sendable as `Ready`). `Working`, `Blocked` and `Unverifiable` are
/// never safe (Ordering Invariant 4, fail-closed status).
fn is_safe_to_send(status: WorkerStatus) -> bool {
    matches!(status, WorkerStatus::Ready | WorkerStatus::Finished)
}

/// One target that made it through resolve, preflight, baseline, the
/// dispatch-time re-check, and `send` — everything a waiter thread needs
/// to confirm or time it out.
struct SentTarget {
    name: String,
    /// The exact task text sent — doubles as this send's completion
    /// marker (Terms: Completion marker). `confirmed_against` requires it
    /// present in the current output AND absent from the baseline, so a
    /// caller's task text should be distinguishing enough not to already
    /// appear in the target's transcript.
    marker: String,
    baseline: Baseline,
}

/// What one waiter thread reports back through the channel.
enum TargetOutcome {
    Succeeded,
    TimedOut,
    UnverifiableAfterSend,
}

/// Polls one already-dispatched target's status until it settles on a
/// terminal, marker-confirmed outcome or `timeouts.worker_settle`
/// elapses — bounded (Ordering Invariant 7): a target confirmed complete
/// on its very first status read returns immediately, with no sleep at
/// all, regardless of how large `worker_settle` is.
fn wait_for_target<B: WorkerBackend + ?Sized>(
    backend: &B,
    name: &str,
    baseline: &Baseline,
    marker: &str,
    timeouts: WaveTimeouts,
) -> TargetOutcome {
    let start = Instant::now();
    loop {
        match backend.status(name) {
            WorkerStatus::Unverifiable => return TargetOutcome::UnverifiableAfterSend,
            WorkerStatus::Ready | WorkerStatus::Finished => {
                match backend.read_output(name) {
                    Ok(output) => {
                        let signal = CompletionSignal {
                            marker: marker.to_string(),
                            output,
                        };
                        if signal.confirmed_against(baseline) {
                            return TargetOutcome::Succeeded;
                        }
                        // Terminal-looking, but not yet confirmed — fall
                        // through to the timeout check below and keep
                        // polling; a stale marker must never short-circuit
                        // this into success (Ordering Invariant 2).
                    }
                    Err(_) => return TargetOutcome::UnverifiableAfterSend,
                }
            }
            WorkerStatus::Working | WorkerStatus::Blocked => {}
        }
        if start.elapsed() >= timeouts.worker_settle {
            return TargetOutcome::TimedOut;
        }
        thread::sleep(timeouts.poll_interval);
    }
}

/// Runs one wave to completion: resolve+dedupe, fail-closed filter,
/// baseline every survivor, dispatch each with its own re-check, wait on
/// all dispatched targets concurrently, aggregate. Calls only
/// `WorkerBackend` — never a concrete backend (D7) — and only
/// `std::thread`/`std::sync::mpsc` for concurrency (D9).
///
/// Only `FailurePolicy::WaitForAll`'s semantics are implemented, matching
/// `Wave::failure_policy`'s own documentation that the other two variants
/// exist as a value from day one (D11) but are not yet built.
pub fn run_wave<B: WorkerBackend + Sync + ?Sized>(backend: &B, wave: &Wave) -> WaveResult {
    // Phase 1 — resolve and dedupe every target to a canonical identity,
    // aborting the WHOLE wave, before anything is sent, if any target
    // fails to resolve.
    let targets = resolve_and_dedupe(&wave.workers);
    let mut resolution_failed = Vec::new();
    for target in &targets {
        if backend.start(target).is_err() {
            resolution_failed.push(target.name.clone());
        }
    }
    if !resolution_failed.is_empty() {
        return WaveResult {
            resolution_failed,
            ..Default::default()
        };
    }

    // Phase 2 — fail-closed status filter, a bulk pass over every
    // resolved target, before any baseline is captured (Ordering
    // Invariant 4).
    let mut unsafe_at_preflight = Vec::new();
    let mut survivors = Vec::new();
    for target in &targets {
        if is_safe_to_send(backend.status(&target.name)) {
            survivors.push(target.clone());
        } else {
            unsafe_at_preflight.push(target.name.clone());
        }
    }

    // Phase 3 — capture a baseline snapshot of EVERY surviving target
    // BEFORE any of them is dispatched to (Ordering Invariants 1 and 2).
    // No `send` call happens anywhere in this loop.
    let mut baselined = Vec::with_capacity(survivors.len());
    for target in survivors {
        let output = backend.read_output(&target.name).unwrap_or_default();
        let baseline = Baseline::capture(output);
        baselined.push((target, baseline));
    }

    // Phase 4 — dispatch: for each surviving target, in order, re-check
    // its status immediately before ITS OWN send (Ordering Invariant 3),
    // then send. One target's send failing does not stop the loop
    // (Ordering Invariant 5, partial-failure isolation).
    let mut flipped_before_send = Vec::new();
    let mut send_failed = Vec::new();
    let mut sent = Vec::new();
    for (target, baseline) in baselined {
        if !is_safe_to_send(backend.status(&target.name)) {
            flipped_before_send.push(target.name.clone());
            continue;
        }
        match backend.send(&target.name, &target.task) {
            Ok(()) => sent.push(SentTarget {
                name: target.name.clone(),
                marker: target.task.clone(),
                baseline,
            }),
            Err(_) => send_failed.push(target.name.clone()),
        }
    }

    // Phase 5 — wait on every sent target concurrently, one
    // `std::thread` per target, collected through an `mpsc` channel (D9);
    // each thread returns its `TargetOutcome` directly rather than
    // writing to a temporary file.
    let (tx, rx) = mpsc::channel::<(String, TargetOutcome)>();
    thread::scope(|scope| {
        for target in &sent {
            let tx = tx.clone();
            let name = target.name.clone();
            let marker = target.marker.clone();
            let baseline = target.baseline.clone();
            let timeouts = wave.timeouts;
            scope.spawn(move || {
                let outcome = wait_for_target(backend, &name, &baseline, &marker, timeouts);
                let _ = tx.send((name, outcome));
            });
        }
    });
    drop(tx);

    let mut succeeded = Vec::new();
    let mut timed_out = Vec::new();
    let mut unverifiable_after_send = Vec::new();
    for (name, outcome) in rx {
        match outcome {
            TargetOutcome::Succeeded => succeeded.push(name),
            TargetOutcome::TimedOut => timed_out.push(name),
            TargetOutcome::UnverifiableAfterSend => unverifiable_after_send.push(name),
        }
    }

    WaveResult {
        succeeded,
        resolution_failed,
        unsafe_at_preflight,
        flipped_before_send,
        send_failed,
        timed_out,
        unverifiable_after_send,
    }
}
