---
date: 2026-07-28
feature: test-batching-finish
categories: [verify-pipeline, tests]
severity: low
tags: [cache, performance, hermetic-tests]
---

# test-batching-finish — feature close learnings

## What Happened

Closed the two open gaps of spec #80 (test-batching): tbf-1 shipped the
content-hash suite-result cache in `run_verify.mjs` (green-only, CI-disabled,
fail-open, `--no-cache`/`--cache-clear`; second identical run: 0 executed, all
cached); tbf-2 proved it with an 8-behavior hermetic suite; tbf-3 added the
≤40-line evidence-report template to `bee-executing`'s worker reference. Spec
#77 (validation-speedup) was closed by audit: P1-P4+P6 already shipped in
1.19.2, P5 rejected by decision. Run summary note: this close synthesized from
first-hand session evidence without the 3-analyst wave (small 4-cell feature,
all evidence in-session) — recorded as a deliberate lean deviation.

## Root Cause / Finding

**A default-on cache is a behavior change for every existing caller.** tbf-1's
worker ran the regression net before the cache was warm — 37/37 green — then its
own demo warmed the live cache, and three pre-existing selection tests (which
assert a suite REALLY runs) started failing with `CACHED green` lines. Caught at
the orchestrator's goal-check re-run, fixed forward as tbf-4 (`--no-cache` in
the suite's three spawns; assertions untouched).

## Recommendation

- When a change alters default behavior of shared infrastructure, re-run its
  regression net AFTER the new state is warm (cache populated, daemon started,
  flag flipped) — a net run against the cold state proves the old world.
- Tests that assert real execution must pin their environment (`--no-cache`,
  `CI=1`) — hermeticity against ambient state is part of the test's contract.
- Audit-before-build pays: one gather pass per spec showed 11 of 18 P-items
  already shipped, collapsing three planned features into one small feature +
  two decisions.
