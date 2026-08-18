//! Pins the eight ordering invariants CONTEXT.md's § Ordering Invariants
//! names, plus one supporting test for the resolution-failure abort a
//! `must_haves` truth requires but which is not itself one of the eight
//! (`docs/history/herding-orchestration/CONTEXT.md`, decision D3).
//!
//! Every test drives `fleet::choreography::run_wave` against
//! `FakeBackend` only — no sleeping, no dependence on real elapsed time
//! for its pass/fail verdict. Where a test needs to prove a bound
//! (Ordering Invariant 7), it counts backend calls via
//! `FakeBackend::status_call_count` rather than measuring a duration.

use std::time::Duration;

use fleet::backend::fake::{FakeBackend, RawStatus};
use fleet::backend::WorkerStatus;
use fleet::choreography::run_wave;
use fleet::wave::{FailurePolicy, Wave, WaveTimeouts, WorkerSpec};

fn timeouts(worker_settle: Duration, poll_interval: Duration) -> WaveTimeouts {
    WaveTimeouts {
        worker_settle,
        poll_interval,
    }
}

fn small_timeouts() -> WaveTimeouts {
    timeouts(Duration::from_millis(50), Duration::from_millis(2))
}

fn one_worker_wave(spec: WorkerSpec, timeouts: WaveTimeouts) -> Wave {
    Wave::new(vec![spec], timeouts, FailurePolicy::WaitForAll)
}

/// Ordering Invariant 1 — fast completion: a worker whose backend settles
/// on `Finished` (with its completion marker confirmed) on the very
/// FIRST status read after dispatch is still detected as a success,
/// because its baseline was captured before dispatch ever happened.
#[test]
fn invariant_1_fast_completion_is_detected_via_a_pre_dispatch_baseline() {
    let backend = FakeBackend::new();
    backend.set_output("w1", "prompt\n$ ");
    backend.schedule_output_on_send("w1", "prompt\n$ MARKERTASK done");
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Ready)); // phase 2
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Ready)); // phase 4 re-check
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Finished)); // first wait poll

    let wave = one_worker_wave(WorkerSpec::new("w1", "MARKERTASK"), small_timeouts());
    let result = run_wave(&backend, &wave);

    assert!(
        result.is_success(),
        "a worker that settles before its waiter's first poll must still be detected as \
         successful, because its baseline predates dispatch; got {result:?}"
    );
    assert_eq!(
        result.succeeded,
        vec!["w1".to_string()],
        "the fast-settling worker must land in the succeeded bucket; got {result:?}"
    );
    assert_eq!(
        backend.status_call_count("w1"),
        3,
        "exactly one preflight read, one re-check, and one confirming wait poll — no extra \
         polling once the first wait poll already confirms; got {} calls",
        backend.status_call_count("w1")
    );
}

/// Ordering Invariant 2 — stale-marker rejection: a marker already
/// present in the baseline (leftover from before this send) must never
/// be credited as proof of THIS send, even though the current output
/// still contains it.
#[test]
fn invariant_2_a_marker_already_present_in_the_baseline_is_never_credited() {
    let backend = FakeBackend::new();
    // The marker is already in the transcript before this wave even
    // starts — a leftover from an earlier, unrelated turn.
    backend.set_output("w1", "boot\n$ STALEMARK leftover\n$ ");
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Ready)); // phase 2
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Ready)); // phase 4 re-check
    backend.set_steady_status("w1", RawStatus::Value(WorkerStatus::Finished)); // every wait poll

    let wave = one_worker_wave(
        WorkerSpec::new("w1", "STALEMARK"),
        timeouts(Duration::from_millis(12), Duration::from_millis(2)),
    );
    let result = run_wave(&backend, &wave);

    assert!(
        !result.is_success(),
        "a marker already present in the baseline must never be credited as proof of this \
         send; got {result:?}"
    );
    assert_eq!(
        result.timed_out,
        vec!["w1".to_string()],
        "an unconfirmable worker settles into timed_out, never succeeded; got {result:?}"
    );
    assert!(
        result.succeeded.is_empty(),
        "the stale-marker worker must never appear in succeeded; got {result:?}"
    );
}

/// Ordering Invariant 3 — the dispatch-time re-check: a target that
/// flips to `Working` between the bulk preflight pass and its own send
/// must not be sent to.
#[test]
fn invariant_3_a_target_that_flips_to_working_before_its_own_send_is_not_sent() {
    let backend = FakeBackend::new();
    backend.set_output("w1", "baseline text, no marker");
    backend.schedule_output_on_send("w1", "SHOULD NEVER APPEAR — send must not be called");
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Ready)); // phase 2: safe
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Working)); // phase 4: flipped

    let wave = one_worker_wave(WorkerSpec::new("w1", "T3"), small_timeouts());
    let result = run_wave(&backend, &wave);

    assert!(
        !result.is_success(),
        "a target that flips unsafe right before its own send must fail the wave; got {result:?}"
    );
    assert_eq!(
        result.flipped_before_send,
        vec!["w1".to_string()],
        "the flipped target must land in flipped_before_send; got {result:?}"
    );
    assert!(
        result.send_failed.is_empty(),
        "the target must never reach `send` at all once its re-check finds it unsafe — a \
         send_failed entry here would mean send was called anyway; got {result:?}"
    );
    assert_eq!(
        backend.status_call_count("w1"),
        2,
        "only the phase-2 read and the phase-4 re-check should happen — no wait polling for a \
         target that was never sent; got {} calls",
        backend.status_call_count("w1")
    );
}

/// Ordering Invariant 4 — fail-closed status everywhere: a lookup
/// failure, a null field, or an off-enum value must never be treated as
/// safe, whether encountered at the bulk preflight pass or while waiting
/// after a successful send.
#[test]
fn invariant_4_fail_closed_status_is_never_treated_as_safe() {
    let backend = FakeBackend::new();

    // Three raw fault shapes, all at the bulk preflight pass.
    backend.schedule_status("w1", RawStatus::LookupFailed);
    backend.schedule_status("w2", RawStatus::NullField);
    backend.schedule_status("w3", RawStatus::OffEnum);

    // A fourth target that starts safely, gets sent to, then its status
    // goes Blocked and then unreadable while waiting — it must not
    // "stabilise" into success just because Blocked alone didn't fail it.
    backend.set_output("w4", "baseline, no marker");
    backend.schedule_status("w4", RawStatus::Value(WorkerStatus::Ready)); // phase 2
    backend.schedule_status("w4", RawStatus::Value(WorkerStatus::Ready)); // phase 4
    backend.schedule_status("w4", RawStatus::Value(WorkerStatus::Blocked)); // wait poll 1
    // Nothing else queued and no steady status set for w4: the next read
    // defaults to LookupFailed -> Unverifiable (fail-closed default).

    let wave = Wave::new(
        vec![
            WorkerSpec::new("w1", "T1"),
            WorkerSpec::new("w2", "T2"),
            WorkerSpec::new("w3", "T3"),
            WorkerSpec::new("w4", "T4"),
        ],
        timeouts(Duration::from_millis(20), Duration::from_millis(2)),
        FailurePolicy::WaitForAll,
    );
    let result = run_wave(&backend, &wave);

    assert!(
        !result.is_success(),
        "a wave with any unverifiable target must fail; got {result:?}"
    );
    let mut preflight = result.unsafe_at_preflight.clone();
    preflight.sort();
    assert_eq!(
        preflight,
        vec!["w1".to_string(), "w2".to_string(), "w3".to_string()],
        "a lookup failure, a null field, and an off-enum value must all drop their target at \
         preflight as unsafe, never as a panic and never as a safe status; got {result:?}"
    );
    assert_eq!(
        result.unverifiable_after_send,
        vec!["w4".to_string()],
        "a Blocked worker whose status lookup later fails must not stabilise into success — it \
         must resolve to unverifiable_after_send; got {result:?}"
    );
    assert!(
        result.succeeded.is_empty(),
        "no target configured with a fault shape may ever be credited success; got {result:?}"
    );
}

/// Ordering Invariant 5 — partial-failure isolation: one target's `send`
/// failing mid-fan-out must not abandon a target dispatched earlier (or
/// later) in the same wave.
#[test]
fn invariant_5_one_send_failing_does_not_abandon_other_dispatched_targets() {
    let backend = FakeBackend::new();

    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Ready)); // phase 2
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Ready)); // phase 4
    backend.schedule_send_result("w1", Err("connection refused".to_string()));

    backend.set_output("w2", "baseline, no marker");
    backend.schedule_output_on_send("w2", "baseline, no marker T5W2 done");
    backend.schedule_status("w2", RawStatus::Value(WorkerStatus::Ready)); // phase 2
    backend.schedule_status("w2", RawStatus::Value(WorkerStatus::Ready)); // phase 4
    backend.schedule_status("w2", RawStatus::Value(WorkerStatus::Finished)); // wait poll

    let wave = Wave::new(
        vec![
            WorkerSpec::new("w1", "T5W1"),
            WorkerSpec::new("w2", "T5W2"),
        ],
        small_timeouts(),
        FailurePolicy::WaitForAll,
    );
    let result = run_wave(&backend, &wave);

    assert_eq!(
        result.send_failed,
        vec!["w1".to_string()],
        "w1's send failure must be isolated to w1; got {result:?}"
    );
    assert_eq!(
        result.succeeded,
        vec!["w2".to_string()],
        "w2, dispatched after w1's send failed, must still be waited on and succeed; got \
         {result:?}"
    );
    assert!(
        !result.is_success(),
        "the wave as a whole still fails because w1's send failed; got {result:?}"
    );
}

/// Ordering Invariant 6 — mixed-result aggregation: a wave in which every
/// SENT target succeeded must still fail if any target was dropped
/// before ever being sent.
#[test]
fn invariant_6_a_dropped_target_fails_the_wave_even_if_every_sent_target_succeeded() {
    let backend = FakeBackend::new();

    backend.schedule_status("w1", RawStatus::LookupFailed); // dropped at preflight

    backend.set_output("w2", "baseline, no marker");
    backend.schedule_output_on_send("w2", "baseline, no marker T6W2 done");
    backend.schedule_status("w2", RawStatus::Value(WorkerStatus::Ready));
    backend.schedule_status("w2", RawStatus::Value(WorkerStatus::Ready));
    backend.schedule_status("w2", RawStatus::Value(WorkerStatus::Finished));

    let wave = Wave::new(
        vec![
            WorkerSpec::new("w1", "T6W1"),
            WorkerSpec::new("w2", "T6W2"),
        ],
        small_timeouts(),
        FailurePolicy::WaitForAll,
    );
    let result = run_wave(&backend, &wave);

    assert_eq!(
        result.succeeded,
        vec!["w2".to_string()],
        "the only sent target must still be recorded as succeeded; got {result:?}"
    );
    assert_eq!(
        result.unsafe_at_preflight,
        vec!["w1".to_string()],
        "the dropped target must be recorded, not silently omitted; got {result:?}"
    );
    assert!(
        !result.is_success(),
        "a wave where every SENT target succeeded must still fail overall because one target \
         was dropped and never sent — a verdict that only checked sent targets would wrongly \
         report success here; got {result:?}"
    );
}

/// Ordering Invariant 7 — bounded working-to-finished polling: a worker
/// that settles on `Finished` (confirmed) on its first wait poll must not
/// be made to wait out the rest of a much longer `worker_settle` ceiling.
/// Proven by counting backend calls, never by measuring elapsed time.
#[test]
fn invariant_7_settling_on_finished_early_does_not_wait_out_the_full_ceiling() {
    let backend = FakeBackend::new();
    backend.set_output("w1", "prompt\n$ ");
    backend.schedule_output_on_send("w1", "prompt\n$ T7MARK done");
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Ready)); // phase 2
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Ready)); // phase 4
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Finished)); // first wait poll

    // A ceiling far larger than what a correct implementation should ever
    // need to consume for a worker that confirms on its first poll.
    let wave = one_worker_wave(
        WorkerSpec::new("w1", "T7MARK"),
        timeouts(Duration::from_secs(5), Duration::from_millis(1)),
    );
    let result = run_wave(&backend, &wave);

    assert!(
        result.is_success(),
        "a worker settling on Finished must still succeed; got {result:?}"
    );
    assert_eq!(
        backend.status_call_count("w1"),
        3,
        "a worker confirmed complete on its first wait poll must not be polled again just \
         because the worker_settle ceiling has not elapsed yet; got {} calls",
        backend.status_call_count("w1")
    );
}

/// Ordering Invariant 8 — dedupe before preflight: a name naming one
/// target that appears twice in the wave's worker list must be sent to
/// exactly once, and must cost exactly one preflight read.
#[test]
fn invariant_8_a_duplicate_target_name_is_sent_to_exactly_once() {
    let backend = FakeBackend::new();
    backend.set_output("w1", "prompt\n$ ");
    backend.schedule_output_on_send("w1", "prompt\n$ T8MARK done");
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Ready)); // phase 2
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Ready)); // phase 4
    backend.schedule_status("w1", RawStatus::Value(WorkerStatus::Finished)); // wait poll

    let wave = Wave::new(
        vec![
            WorkerSpec::new("w1", "T8MARK"),
            WorkerSpec::new("w1", "T8MARK"),
        ],
        small_timeouts(),
        FailurePolicy::WaitForAll,
    );
    let result = run_wave(&backend, &wave);

    assert!(
        result.is_success(),
        "the deduped single target must succeed; got {result:?}"
    );
    assert_eq!(
        result.succeeded,
        vec!["w1".to_string()],
        "a name appearing twice in the wave must produce exactly one succeeded entry, not two; \
         got {result:?}"
    );
    assert_eq!(
        backend.status_call_count("w1"),
        3,
        "a name appearing twice must still cost exactly one preflight read, one re-check, and \
         one wait poll — not two of each; got {} calls",
        backend.status_call_count("w1")
    );
}

/// Supporting truth (not one of the eight numbered invariants, but a
/// `must_haves` truth D3's phase 1 requires): a target that fails to
/// resolve aborts the WHOLE wave before anything is sent — including
/// targets that would otherwise have resolved fine.
#[test]
fn resolution_failure_aborts_the_whole_wave_before_anything_is_sent() {
    let backend = FakeBackend::new();
    backend.schedule_start_result("w1", Err("no such target".to_string()));

    // w2 would succeed cleanly if the wave ever reached it.
    backend.set_output("w2", "prompt\n$ ");
    backend.schedule_output_on_send("w2", "prompt\n$ RESOLVEMARK done");
    backend.schedule_status("w2", RawStatus::Value(WorkerStatus::Ready));
    backend.schedule_status("w2", RawStatus::Value(WorkerStatus::Ready));
    backend.schedule_status("w2", RawStatus::Value(WorkerStatus::Finished));

    let wave = Wave::new(
        vec![
            WorkerSpec::new("w1", "T9W1"),
            WorkerSpec::new("w2", "RESOLVEMARK"),
        ],
        small_timeouts(),
        FailurePolicy::WaitForAll,
    );
    let result = run_wave(&backend, &wave);

    assert!(
        !result.is_success(),
        "a wave with an unresolvable target must fail; got {result:?}"
    );
    assert_eq!(
        result.resolution_failed,
        vec!["w1".to_string()],
        "the unresolvable target must be recorded; got {result:?}"
    );
    assert!(
        result.succeeded.is_empty(),
        "no target may be sent once ANY target fails to resolve; got {result:?}"
    );
    assert_eq!(
        backend.status_call_count("w2"),
        0,
        "a resolvable target's status must never be read once a different target in the same \
         wave fails to resolve — the whole wave aborts before phase 2; got {} calls",
        backend.status_call_count("w2")
    );
}
