# Finish Phase — Context

Asked: "b" (2026-11-04), choosing to fix the feature-finish friction seen
while closing lean-context.

## Feature Boundary

In scope: D1. Out of scope: running `cells claim` or `dispatch prepare
--claim` from a granted worktree; decision 671f60e7 keeps that refusal on
purpose.

## Locked Decisions

- **D1 (e715bc3a).** After its worktree merge succeeds, `bee feature
  finish` moves a lane whose phase is before execution (exploring or
  planning) to swarming before the scribing stamp, and prints one line
  that names the move. A failed merge leaves the phase as it was.

## Evidence

- lean-context: the merge proved 2 cells, then `state scribing-run`
  refused from phase "exploring"; the leader needed two `state set` calls.

## Outstanding Questions

None.
