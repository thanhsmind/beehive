# uat-gate-before-merge — learnings (2026-08-17)

Feature: the `uat` gate — the user's acceptance stop between
execution-complete and the merge to main. Cells ug-1..3, merged to main
(verify green) after the user's explicit acceptance in-session.

## What settled

- uat-gate-before-merge D1: merge refuses standard/high-risk features with an
  unapproved uat gate; `--skip-uat` / `uat_before_merge: false` escape
  hatches; no bypass level auto-approves it. Specs synced:
  `workflow-state/gates.md` (R106), `worktree-parallelism/returning-and-the-merge-gate.md`.
- staging-lane D0/D0a logged mid-feature (user): main/staging/worktree
  topology becomes first-class mechanics — next feature.

## Learnings

1. **Lane projection drops `route`; the claim guard's own remedy cannot
   satisfy it.** `.bee/lanes/<feature>.json` is rewritten without `route`
   while `route --set` writes only the default record; `read_lane_route`
   short-circuits on the lane file, so `NO_ROUTE_RECORD`'s FIX line loops
   forever. Worked around with `claim --session-id <fresh>`; filed P2
   backlog friction (fix: projection carries route, or lane read falls back
   to the default record).
2. **A feature worktree cannot see bookkeeping written after its creation.**
   Cells and plan.md written in main's working tree post-`worktree new` are
   invisible to the worker until committed on main and merged into the
   branch. First dispatch died on it; a path-scoped bookkeeping commit +
   `git merge main` in the worktree fixed it. Ordering candidate: commit
   feature bookkeeping before dispatching, always.
3. **The shared-index pattern got teeth the same day it was recorded** — the
   concurrent-worker git guard refused a bare `git add` in main and named the
   temp-index remedy; the remedy worked as written. Pattern→guard promotion
   loop closed in under a day.
4. **Self-hosting lag is a real UAT concern**: the running binary can neither
   record the new gate nor enforce the new refusal until a release ships;
   this feature's own merge used the conversation's approval plus
   `--no-cleanup`, and enforcement starts with the next release.
