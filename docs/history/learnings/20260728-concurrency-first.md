---
date: 2026-07-28
feature: concurrency-first (+ verify-owner-signal)
categories: [orchestration, doctrine-layer]
severity: high
tags: [lanes, concurrency-law, artifact-labelling, first-parallel-features]
---

# concurrency-first — feature close learnings

## What Happened

Two features ran **concurrently in one checkout** for the first time —
`verify-owner-signal` in the main lane, `concurrency-first` in its own lane —
and closed under one shared barrier.

- **R86 (concurrency law):** one rule across three tiers (gather → workers,
  cells → wave, features → lanes), an exhaustive list of the only four legal
  reasons to go serial, and a mandatory one-line concurrency plan computed from
  declared paths, not guessed. Lanes stopped being a footnote and became the
  paved road for new feature work while another feature is live.
- **R87 (artifact labelling):** the cell's `verify` field now renders
  `verify_owner: "main (feature close) — the worker never runs this"`.

## Root Cause of the thing that triggered it

A worker ran the full suite chain hours after verification moved to the
delegator. Not disobedience: its skill said one thing, but the work record
handed it a field literally named `verify` holding a runnable command with no
owner, and its dispatch note said "don't run suites". **The artifact won.**
Laws are read once; artifacts are read at the moment of acting.

## What the live run proved

1. **The lane guard is the concurrency plan's proof.** Opening the second lane
   with the full path set was refused by name —
   `packages/bee/bee.mjs` held by the sibling's claim. Re-opened over the
   disjoint subset, it started immediately. Parallel where disjoint, serial
   exactly where forced, decided by the machine.
2. **A path boundary produces honest follow-up work, not damage.** cf-1 could
   not touch the AGENTS template (sibling held that tree), so the template went
   out of sync; the worker reported it as a deviation and a 3-minute follow-up
   cell closed it once the path freed. Cheaper than serialising both features.
3. **The barrier had a hole.** The shared regen chain missed
   `render_openai_metadata.mjs`; the feature verify caught it as a stale
   projection. *Rule: the wave barrier is every generator that reads what the
   wave wrote — when a new generated projection appears, it joins the barrier
   chain, not just the verify chain.*

## Ran under

R82 (workers ran no suites; main verified once per feature — one red caught,
fixed, re-verified), R83 (one sync, one compounding), R84 (step ticks), R85
(communication contract loaded).
