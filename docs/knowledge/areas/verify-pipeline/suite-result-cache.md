---
type: bee.area
title: Verify Pipeline — suite-result cache
description: "The local content-hash cache that skips a suite whose green result is already proven for the exact bytes of its dependency closure — why red is never cached, why CI never uses it, and how it degrades open on corruption."
tags: [verify-pipeline, performance]
timestamp: 2026-07-28
bee:
  id: verify-pipeline-suite-result-cache
  lifecycle: active
  areas: [verify-pipeline]
  required_context: [areas/verify-pipeline/suite-topology-and-discovery.md]
  decisions: [test-batching-finish merged-gate audit, "spec #80 P7"]
---

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
