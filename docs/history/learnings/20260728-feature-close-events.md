---
date: 2026-07-28
feature: feature-close-events
categories: [orchestration, scribing]
severity: medium
tags: [feature-close, r82-first-run, census]
---

# feature-close-events — feature close learnings

## What Happened

Rescoped scribing + compounding from per-cell/per-execution triggers to
feature-close events (R83): mid-feature keeps only same-turn capture stubs;
sync + learnings run once, after the feature verify. Surfaces: bee-scribing
sync trigger, bee-swarming completion paragraph, routing-and-contracts capture
law; AGENTS.md rule 8 needed no change (already stub-only). Both touched
migrated bodies net-shrank (scribing 8151→8135, swarming →8121).

## R82's first live run — and its first catch

This was the first feature governed by main-verifies. The loop worked exactly
as designed, including the failure path:
- Workers ran ZERO suites (fc-1 sonnet, fc-2 haiku), capped
  `--feature-verify-pending`.
- MAIN's single feature verify came back RED: fc-1 had dropped the AO14
  pointer phrase from the swarming body — a break the worker could not have
  seen (it ran nothing, and its by-inspection census pass missed the pattern).
- D5 path: fix cell fc-2 (never un-capped fc-1), re-verify green, record
  stamped (sha in the workflow record), and the close door let scribing-run
  through only after the fresh green record existed.

## Findings

1. **By-inspection census checking is weaker than running the census.** The
   new law shifts that risk to main's boundary run — where it was caught at
   the cost of one extra fix cell, not a mid-flight suite run per cell.
   Accepted trade, working as intended.
2. **Byte-budget trims near the ratchet edge drop load-bearing phrases.**
   Twice today a body trim severed a census-pinned pointer (et-4's three, now
   fc-1's one). *Rule: before trimming a migrated body, grep the census
   (test_misc) for phrases pinned to that file and treat them as immovable.*
