# gh-fix-batch — learnings (2026-07-28)

Feature: `gh-fix-batch` (standard, 3 bugfix cells gfb-1/2/3) — fixed the three
confirmed bugs from GitHub issues #87 (bug 1), #83, #84. Issue #87's bug 2
(advisor_ref silently dropped) was already fixed on main by the GH #86 repair.

## What settled

- **Scope a shared-ledger release by every discriminator the row carries.**
  Cell-only hold release wiped a different session's hold on the same cell id
  (#87). The fix pattern: derive `{cell, session}` pairs from the caller's own
  rows and pass each filter the primitive already supports; sessionless rows
  fall back to the old scoping — strictly narrowing (gfb-1).
- **A resolved topology must be used everywhere in the call it was resolved
  for.** `readConfig(root)` under an already-resolved `controlRoot` made the
  idle gate judge a different project than the containment check in the same
  call (#83). Fixing one call site and leaving the same shape at two others
  recreates the bug — align the whole class in one pass (gfb-2).
- **Destructive cleanup belongs after the last zero-mutation refusal.**
  Companion teardown ran before the merge's refusal checks, so refused merges
  destroyed healthy mounts (#84). Reorder + git pathspec `:(exclude)` on the
  dirty check; text-filtering porcelain is wrong because nested untracked paths
  collapse to a parent-directory line (gfb-3).
- **The e2e suite catches what the plan missed:** deferring teardown exposed a
  second dirt source (the marker file itself) only visible in the real
  `--with-companion` merge suite — plan-checked approaches still need their
  end-to-end suite in the cell verify.
- **Plan-check earns its cost on MEDIUM-risk cells:** the opus plan-check
  caught the nested-mount porcelain collapse, the two extra refusals before
  first mutation, the sessionId-null requirement to even reach the buggy
  branch, and the parallel-dispatch/manifest-regen conflict — four defects
  that would each have burned a worker round-trip.

## Knowledge sync

- `worktree-parallelism/cross-worktree-holds.md` (release scoping rule).
- `hook-runtime/governed-paths-and-the-intake-gate.md` (new R13).
- `worktree-parallelism/returning-and-the-merge-gate.md` (companion survival rule).
