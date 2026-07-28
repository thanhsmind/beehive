---
date: 2026-07-28
feature: parallel-default
categories: [orchestration, workflow-state]
severity: medium
tags: [parallel, dispatch, wave-barrier, philosophy]
---

# parallel-default — feature close learnings

## What Happened

User philosophy directive: bee was built for parallel — serial-mindedness
contradicted the original design. Flipped the doctrine at four sites (swarming
body + reference, routing-and-contracts lane row + execution-worker class):
cells of a wave dispatch concurrently (3–4 workers) on disjoint product file
sets; serial names its conflict. Shipped the **wave-barrier regen** convention
(`regen_obligation_ack: "wave-barrier"` — orchestrator owes the regen chain
once at wave close) and named it in the guard's refusal message. Verify
concurrency deliberately unchanged at `min(5, cpus)` — measured optimum
(6 flaked, 16 flaky; evidence in `run_verify.mjs:1275`).

Dogfooded in its own build: pd-1 (guard message) + pd-3 (doctrine) ran as a
真 parallel wave — the scheduler itself computed `Wave 1: pd-1, pd-3` once the
wave-barrier acks removed manifest/mirror regen from their file sets.

## Root Cause (of the old serialization)

Not the cell design — the **per-cell regen obligation**. Every cell touching a
manifest-hashed root had to regenerate the same shared artifacts (manifest,
mirrors, onboarding ledger), making every such pair's file sets overlap, which
the scheduler correctly auto-serialized. All six skill migrations of
skill-token-diet serialized on exactly this.

## Recommendations

- **Find the shared artifact before blaming the architecture.** One deferred
  regen turned a forced-serial pipeline into scheduler-computed parallel waves
  with zero scheduler changes.
- **A barrier debt must be typed and owed by name**: the ack records who skips
  and the doctrine records who pays (orchestrator, wave close, close commit,
  before "clean") — an untyped skip is just a missing regen.
- The suite-result cache (spec #80 P7) compounds with parallel waves: the
  wave-close impacted rerun hit 25/25 cached at wall=0ms.

## Open thread

State-clobber friction escalated to P2 (3 occurrences in one day: post-resume,
during sv-1, during pd-1 — feature/phase reverting to a prior feature). Needs
its own repro hunt before the next heavy multi-worker session.
