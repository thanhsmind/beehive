---
type: bee.pattern
title: "A best-effort write hung off a verb sits on every exit and outside the verb's own locks"
description: "Two silent, green-passing failure shapes from one feature: a record-keeping write placed only on a verb's success path leaves the previous answer standing when the verb refuses early, and a best-effort side effect that writes through the same lock seam as its host finds the locks busy and vanishes. Both pass every test that checks the happy path; only a reader of the record catches them."
tags: [workflow-state, records, locks, close, side-effects]
timestamp: 2026-08-23
bee:
  id: pattern-20260823-a-best-effort-write-sits-on-every-exit-outside-the-locks
  lifecycle: active
  sources: ["merge-ready-fact cell mrf-2 (close has THREE full-doors vectors, not two — the proof-debt refusal arm assembles its own and returns before the green path; the blocked_by write was wired onto all three, pinned by a_close_stopped_at_the_tests_door_still_records_that_door)", "merge-ready-fact cell mrf-2 (set_uat runs after run_gate_body drops its mutation locks — the helper takes those very locks, so an earlier call would find them busy and fail-open into silence)", "merge-ready-fact cell mrf-2 (worktree unregister clears the fact before taking the worktree-admin lock, so record-mutation locks are never nested under it)"]
  polarity: pitfall
  critical: false
---

# A best-effort write sits on every exit and outside the locks

A fact that a verb maintains as a side effect — "ready except for these
doors", "uat approved", "this worktree is gone" — is easy to hang off the
success path and forget everywhere else. Two things then go wrong, and both
are silent.

**Every exit, not the success path.** A verb that can stop at several doors
has several exits. A write placed only where the verb succeeds leaves the
previous answer standing on every refusal — a list that says "ready except
for nothing" for a close that never happened. The record reads green for a
result that did not occur. Place the write at the moment the answer is known,
before the verb decides whether to refuse, and count the exits in the code,
not in the cell text: the feature that found this had been told "two
vectors" and found three.

**Outside the host's locks, not inside.** A best-effort side effect that
writes through the same record-mutation seam as the verb hosting it takes
the same locks. Called while the host still holds them, it finds them busy
and — because it is best-effort — returns without writing and without a
sound. Run it after the host releases, still strictly after the write it
depends on; and never nest the record locks under an unrelated admin lock.

Both shapes pass a happy-path suite. The proof that catches them is a test
on the refusal arm ("a close stopped at the tests door still records that
door") and a test that drives the side-effect helper on its own.
