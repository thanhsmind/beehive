---
date: 2026-07-28
feature: status-diet
categories: [cli, orchestration]
severity: low
tags: [status, brief, dispatch, tokens]
---

# status-diet — feature close learnings

## What Happened

Every worker startup paid full `status --json`: measured 372ms + 15,171B
(353 cell files scanned; models/handoff/review blocks a worker never reads).
Shipped `status --brief` — 7 keys, state-layer reads only, measured live at
**73ms / 545B** (5x faster, 28x smaller) — and changed the worker contract:
the dispatch prompt embeds a `State at dispatch:` line, startup re-validates
with brief, `cells show` stays the claim authority. Wave ran st-1 ∥ st-2
parallel; one [BLOCKED] resolved by amending the cell's verify.

## Findings

1. **A wave-barrier cell's verify must not name barrier-dependent checks.**
   st-1's verify included `test_misc`, whose vendored-parity check is
   necessarily red until the orchestrator's barrier regen — the cell's own
   ack forbids the worker from fixing it. Worker blocked correctly (advisor
   concurred: a verify-pass claim is machine-readable; exceptions belong to
   the orchestrator). *Rule: when authoring a wave-barrier cell, exclude
   parity/manifest checks from its verify — they are the barrier's proof,
   not the cell's.*
2. **The orientation payload was 96% waste for workers.** The fix was not
   making the big thing faster but giving the hot path a small thing.

## Context for the next feature

This close is the last one under per-cell worker verify. The user's
feature-verify philosophy (main verifies once per feature, workers never run
suites) is decision-logged and lands next as `main-verifies`.
