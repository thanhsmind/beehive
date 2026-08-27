---
type: bee.pattern
title: A dead worker has the code and is missing the last mechanical step
description: A dead worker has the code and is missing the last mechanical step
tags: [failure, dispatch, recovery, registry, tests]
timestamp: 2026-08-27
bee:
  id: pattern-20260827-a-dead-worker-has-the-code-and-is-missing-the-last-mechanical-step
  lifecycle: active
  areas: [hook-runtime, rust-runtime]
  sources: ["slp-supervisor-heartbeat cell sup-11 — dispatched worker died on an API rate limit with implementation and tests on disk, uncommitted, 2026-08-27"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "before re-dispatching a dead worker's cell, run the cell's own verify against the working tree; then diff the cell's new surface against every declaration file it must appear in (for a new bee verb: `src/generated/registry_payload.json`) — the registry contract tests only walk what IS declared, so an undeclared verb is green"
---

# A dead worker has the code and is missing the last mechanical step

A dispatched execution worker died mid-cell on an API rate limit. Its whole
implementation and its whole test file were already on disk. Nothing was
committed. The reflex is to throw the work away and re-dispatch the cell.

That reflex is expensive and wrong. Orchestrator recovery beats re-dispatch
here: run the cell's verify against the working tree first. If it compiles and
only the dependent assertions fail, finish the cell — fix, commit, cap — rather
than paying for the same code twice.

What a half-finished worker most reliably has *not* reached is the **last
mechanical step**. The cell had every test written, and had never declared its
new verb in the generated registry payload. No test caught it, because the
registry contract tests only walk what **is** declared. An undeclared verb is
not a failing row; it is an absent row, and an absent row is green.

So the recovery has two halves, and the second is the one that gets skipped:

1. Run the verify. Judge whether the code is substantially there.
2. Check every **declaration** surface the change was supposed to appear in —
   registries, manifests, catalogs, indexes. A contract test that iterates a
   declared set can never fail on an omission from that set.
