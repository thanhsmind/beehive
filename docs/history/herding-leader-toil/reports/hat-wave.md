# Plan check — herding-leader-toil (hat wave, 2026-10-06)

Seats: hat-facts-gaps (opus), hat-risks (fable), hat-value (agy-flash),
hat-alternatives (opus), hat-user-impact (agy-flash). Synthesis by the leader.

```text
PLAN CHECK
Work: slice 1, cells hlt-1..hlt-7

STRUCTURE
BLOCKERS (all fixed in CONTEXT.md and plan.md):
- hlt-1 / private module: herding::mailbox is private and read_result/git_block live in run.rs / FIX: herding.rs gains pub(crate) mod mailbox; local git helper; no run.rs edit.
- hlt-1 / close door: a from-job cap recorded no mistakes answer, so bee close would refuse / FIX: --no-mistakes or --mistake required with --from-job (D1 revised).
- hlt-2 / scope: widening the shared prelude served every verb on that door, wider than D2 / FIX: allow-list opt-in through ctx_serving_granted; unlisted verbs keep refusing (D2 revised).
- hlt-2 / close: close.rs resolved roots itself and was not in the file list; it also read main's docs / FIX: close.rs added with worktree docs reads.
- hlt-2 / verb names: no `cells release` verb exists; unclaim, block, drop, rebind-session were unaddressed / FIX: explicit served and refused lists.
- hlt-3 / silent success: zero verdict blocks exited 0 / FIX: zero blocks refuse (D3 revised).
WARNINGS (applied):
- hlt-4 wait thread could not be stopped and orphaned children / WaitSource arm-poll-cancel with a polled child, cancel on every exit (D4 revised).
- hlt-4 after Error nothing re-read / inspect fallback after Error.
- hlt-6 doctor failed before the first spawn / folder checks only when the folder exists (D6 revised); non-link auth.json fails.
- hlt-1 files source not shown; empty proof text gave a confusing error / output names the source; named refusal.
- hlt-2 JSON arrays would break / control_root only on objects.
- hlt-2 gate history reads already resolve the worktree plan / test only, no second resolver.
- cells finish must keep capping from the worktree / truth and test added.
WARNINGS (noted):
- auth.json may be replaced by a Pi token refresh / doctor fails on a non-link; the live proof checks it.

CELLS (reviewed: 7)
CRITICAL FLAGS: none open after the fixes.
CLEAN CELLS: hlt-5, hlt-7

SUMMARY: Six blockers fixed by five revised decisions (cb3dc85d, 671f60e7, 6824f849, 2e46ebbe, 9e018527) and revised contracts; hat-value judged every item worth its cost.
```
