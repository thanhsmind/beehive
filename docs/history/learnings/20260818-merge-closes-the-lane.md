---
date: 2026-08-18
feature: merge-closes-the-lane
categories: [pattern, process]
severity: normal
tags: [workflow-state, worktrees, lanes, tests, judging]
---

# Learning: A lifecycle with no terminal writer leaves every finished feature running

**Category:** pattern
**Severity:** normal
**Tags:** [workflow-state, worktrees, lanes]
**Applicable-when:** a state machine's terminal transition is reachable
only by a human typing a command, and every automated path stops one
step short of it.

## What Happened

A sibling session reported three beehive features stuck in a dashboard's
"In Progress" column after their work had shipped. The scan found eleven,
not three: every one fully merged into `main` (`git rev-list --count
main..wt/<f>` = 0), cells capped or archived, and every lane still reading
`swarming`, `planning`, or `exploring`. Five still held a live `uat` gate
mark, so they also read "waiting on the human" — forever.

The dashboard was right. The state was wrong.

## Root Cause

No command wrote a terminal lane phase. `bee worktree merge` performed
**zero** lane-record writes — its only lane touch was a read-only uat
precheck, and its only durable self-record was a `worktree-cleanup` row in
the pending-work queue. `bee close` retired the feature's cells and
committed bookkeeping, but wrote no phase. The only writer was
`bee state set --phase`, typed by hand. Two features closed correctly in
the whole history, both because a human happened to type it.

Two smaller holes fed the same symptom. `bee state waiting-on clear`
refused outright when the feature's workflow record was already `closed`
— so a stranded mark had no CLI repair path at all, even though the
command's own help promised "a no-op, never a refusal". And on a lane
record the clear nulled only `waiting_on`, leaving `run_state:
awaiting-approval` behind; decision `f9fd9d46` had already fixed exactly
that pair rule, but only for the default `state.json` record, and the
stuck value actually lived on the lane projection.

## Resolution

Three writes, each in the command that already owned that half of the
truth (decisions `b61d41ac`, `f220f461`, `500fa2f9`, `D4`):

- A green `bee worktree merge` that actually merged something clears the
  merged feature's lane pair and rewrites `next_action` to name
  `bee close --feature <f>`. It NEVER writes `phase` — a merge can land
  one slice of several, and a phase write from there would call a
  mid-flight feature finished. Load-bearing constraint, not a detail.
- A green, non-dry-run `bee close` sets the lane to `idle`. Never
  `compounding-complete`, which stays gated on a fresh compounding run.
- `waiting-on clear` falls back to the newest record regardless of status
  and clears the lane pair; only a feature with no record at all still
  refuses. `waiting-on set` never widens.

The eleven stranded lanes were repaired through the CLI in the same pass.

## What Made This Expensive

The revision, not the build. An independent semantic judge found that the
new lane write sat just BEFORE the post-commit tracked-files guard — and
lane records are tracked here, so every clean merge printed
`verify_mutated_tracked_files` at itself. All three of the cell's new
tests missed it because their fixture lane file was untracked, and the
guard excludes untracked files. A green suite proved nothing about the
one thing that was broken.

Promoted as a critical pattern:
`docs/knowledge/patterns/20260818-a-write-placed-before-a-self-check-makes-the.md`.

## Process Note

The first dispatch of the merge cell returned `[BLOCKED]` rather than
guessing: the cell's action text cited `merge_finish` at `merge.rs:451`,
but the function had moved to `phases.rs` in an earlier file split, and
`merge.rs:451` was an unrelated helper. The worker checked whether the
write could go somewhere in scope, found that every existing merge test
calls `merge_feature_worktree` directly (so a write one layer up would be
untestable by the file's own convention), and asked for the scope instead
of shipping a weaker implementation. Correct call. A stale line citation
in a cell is worth a blocked dispatch.

## Open

Filed as P3 debt: the lane rewrite still lands outside the merge commit,
sitting modified-but-uncommitted in main until an unrelated bookkeeping
auto-commit sweeps it up. Not a defect in the shipped scope; worth folding
into the merge commit later.

Thirty-three historical lanes with no worktree still read a non-terminal
phase. They predate this defect and are cleanup, not behavior.
