---
date: 2026-07-27
feature: test-runs-lean
categories: [process, verification]
severity: P2
tags: [verify, tests, lanes, ceremony]
---

# test-runs-lean close — learnings

## What Happened

User audit of the compounding-gate session: the same suites ran 5+ times
(worker + orchestrator duplicate, test_misc diagnostics, mutation cycle), an
impacted run executed mid-slice because the orchestrator authored it into a
cell's verify (violating the existing targeted-only law), and a ~40-line guard
shipped with a ~390-line test/fixture diff. Fixed same-day as skill text:
verify-once for serial tiny/small, falsifiability scoped to hard-gate suites,
impacted-in-cell-verify made a worker-refusable defect.

## Root Cause

The double-verify default ("never the worker's word") was written for parallel
untrusted swarm waves and silently taxed the serial supervised case; the
existing targeted-only law had no refusal tooth on the authoring side, so the
orchestrator's own cell-writing habit could violate it unchecked.

## Recommendation

- When a proof rule exists to distrust parallel strangers, do not apply it to a
  serial worker whose full transcript the orchestrator just read — trust the
  recorded output, re-run on smell only.
- When a law binds the worker but the defect is authored upstream, give the
  downstream party an explicit refusal (worker [BLOCKED] on a broad verify) —
  laws without a refusing party get violated by their own authors.
- Mechanized follow-up filed: addCell warning when a verify field matches
  run_verify/commands.test.
