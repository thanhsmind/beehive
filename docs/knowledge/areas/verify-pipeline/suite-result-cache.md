---
type: bee.area
title: Verify Pipeline — suite-result cache
description: "The local content-hash cache that skips a suite whose green result is already proven for the exact bytes of its dependency closure — why red is never cached, why CI never uses it, and how it degrades open on corruption."
tags: [verify-pipeline, performance]
timestamp: 2026-08-06
bee:
  id: verify-pipeline-suite-result-cache
  lifecycle: active
  areas: [verify-pipeline]
  required_context: [areas/verify-pipeline/suite-topology-and-discovery.md]
  decisions: [test-batching-finish merged-gate audit, "spec #80 P7", review-p1-fixes p1-1/p2-2 (the cache was reworked twice — a leaking fixture and an opt-in declaration table — before the runtime it lived in was retired; 2026-08-04), "962a6490 (the retired runtime's performance work is recorded, never merged into the specs as current behavior)"]
  sources: ["verified 2026-08-06: no suite-result cache exists in the shipped runtime; the test verb reads commands.test from .bee/config.json and runs every declared command sequentially, writing .bee/logs/test-results.json on each call", review-p1-fixes cells p1-1/p2-2 (docs/history/review-p1-fixes/promote-proposals.md — the last two reworks of the retired cache)]
---

> **Retired (2026-08-06).** Everything below describes a subsystem of the
> runtime that has been replaced. That runtime is gone, and with it the whole
> cache: no result caching of any kind ships today. The project declares its
> tests once, and the test verb runs every declared command, in order, on every
> call, writing the results record each time — there is no closure hash, no
> declaration table, no environment switch that disables caching, because there
> is nothing to disable. This page is kept as the historical record of how the
> cache worked and why it was shaped that way; it is not a description of
> current behavior. The live behavior is one line: run what the project
> declares, every time.

## Purpose

A test suite whose entire dependency closure is byte-identical to the last green run proves nothing by running again. This cache records, per suite, a hash over the exact content of every file in its impact-registry closure at the moment the suite last ran green, and skips the suite while that hash holds — cutting repeated local dev-loop runs to the suites that can actually change outcome.

## Entry Points & Triggers

- Active by default on local runs (full and impacted); every green suite result updates its entry.
- Disabled entirely on CI (environment-detected) — CI always runs everything it selects.
- Escape hatches: a no-cache flag forces real execution without touching the store; a cache-clear flag wipes the store and forces a cold run.

## Data Dictionary

- **Closure hash**: sha256 over the sorted content hashes of every file in the suite's registry closure (a suite with no registry closure hashes its own entry file and arguments). Working-tree bytes, not commits — uncommitted edits change the hash.
- **Cache entry**: suite label → closure hash + timestamp + green marker. Only green exists; red is never written.
- **Cache store**: one JSON file under the runtime logs directory; disposable, never committed.

## Behaviors & Operations

- A run reports each skipped suite visibly ("CACHED green … closure unchanged") and summarizes run-vs-cached counts — a cached skip is never silent.
- Editing any closure file re-runs exactly the suites whose closures contain it; unrelated entries stay cached.
- The first green run after a red executes for real (red was never cached, so there is nothing to skip against).
- Diff-vs-test advisory at green cap (finish-advisory fa-1, 2026-08-11): when HEAD's commit body carries the finishing cell's `cell: <id>` trailer and the commit changes more than `finish.advisory_untested_lines` lines (default 150; 0 disables) with no test-shaped path touched, `cells finish` prints one stderr advisory line and appends it to `trace.warnings`. Green path only; never alters exit code or cap outcome; any git failure is a silent skip.

## Business Rules

- **Green only.** A failure is never cached; only proof of green for exact bytes is remembered.
- **CI never caches.** The authoritative full-suite verdict is always freshly executed.
- **Fail open.** A missing or corrupt store reads as an empty cache — a cache defect buys more running, never less; the store repairs itself on the next green.
- **Hermetic tests spawn with the cache off.** Suites that assert "this run really executed" pass the no-cache flag; a warm cache must never change a test's meaning.

## Edge Cases Settled

- Corrupt store: silent cache-miss, then rewritten clean.
- Mixed run: cached and executed suites coexist in one invocation; counts reported.
- Introducing the cache broke three pre-existing selection tests that asserted real execution; settled rule above (hermetic tests opt out) rather than weakening assertions.

## Open Gaps

- Cache entries are label-keyed; renaming a suite orphans its entry harmlessly (stale entries are never pruned automatically).

## Pointers (implementation)

- `scripts/run_verify.mjs` (cache logic, flags), `.bee/logs/verify-cache.json` (store), `scripts/impact-registry.json` (closures), `scripts/tests/test_verify_cache.mjs` (8-behavior proof), `scripts/tests/test_run_verify_impacted.mjs` (hermetic --no-cache spawns).
- History: `docs/history/test-batching-finish/` (tbf-1/2/4 reports).
