---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-07-27 (auto-approved, gate_bypass=total)
---

# gh-fix-batch — fix 3 confirmed bugs from GitHub issues #87, #83, #84

## Mode gate (mechanical record)

- Risk flags: **0-1**. All three are covered bugfixes that keep existing tests
  green and add a new one (D7 narrowing scores 0 on both test flags). No auth,
  no data model, no external systems, no public-contract change. #84 touches a
  destructive teardown path but the fix *narrows* destruction and the artifacts
  involved (symlink, marker file, /tmp session) are recreatable metadata — not
  the data-loss hard gate.
- Product files: **6** (3 source + 3 test) — exceeds the small cap (≤3), and
  the three fixes are independent tasks, so tiny/small are dishonest. → **standard**.
- Why not smaller: 3 independent tasks across 6 product files; why not
  high-risk: zero hard-gate flags.

## Scope

| # | Issue | Bug | Fix site |
|---|---|---|---|
| 1 | #87 bug 1 | `handleReservationsRelease` scopes cross-worktree hold release by `{holder, cell}` only — one session's release clears another session's still-active hold on the same cell | `packages/bee/bee.mjs:1864-1885` |
| 2 | #83 | `checkWrite()` terminal-phase idle-gate branch reads `readConfig(root)` instead of `readConfig(controlRoot)` — companion-mounted paths read the wrong project's config | `packages/bee/lib/guards.mjs:1041` |
| 3 | #84 | `mergeFeatureWorktreeStage` calls `teardownCompanionIfPresent` BEFORE its dirty-tree checks — a refused merge still destroys the companion mount | `packages/bee/lib/worktree-store.mjs:1696-1709` |

**Out of scope:** #87 bug 2 (advisor_ref dropped) — already fixed on main via
the GH #86 repair (`writeState(root, updated)` present in both
`writeStateRecordThroughProjection` and `writeLaneRecordThroughProjection`,
verified by source read). Issue comment will say so.

## Discovery (L1 — verified against source)

- `releaseHolds(mainRoot, { holder, session = null, cell = null })`
  (`packages/bee/lib/worktree-holds.mjs:218`) already supports the `session`
  filter; the reserve side already mirrors `session` onto holds
  (`insertHold`, same file). Fix 1 is purely a call-site change: derive
  `{cell, session}` pairs instead of bare cell ids. Sessionless rows fall back
  to today's cell-only scoping (strictly narrowing, never behavior-changing
  for the single-session case).
- `controlRoot` is already resolved at `guards.mjs:874`
  (`resolveWriteTopology`); fix 2 is a one-argument change plus a fixture
  where `root` and `controlRoot` configs disagree.
- #84's ordering has a *stated* reason (doc comment, `worktree-store.mjs:1584`):
  the companion's mounted symlink is untracked, so `isTreeDirty(worktreeRoot)`
  would see it and refuse every `--with-companion` merge. A naive reorder
  breaks that. **Approach:** read the marker first (without deleting), run
  both dirty checks with the companion's known `mountPath` line excluded from
  the porcelain output, and run the actual teardown only after both checks
  pass — the merge is then genuinely proceeding. Reason 2 of the doc comment
  (session shouldn't outlive the merge) still holds on the proceed path; a
  refused merge now leaves the mount intact, which is the issue's ask.

## Approach

Three independent bugfix cells, red-first each (repro test written and seen
red before the fix, per bugfix discipline — slice-tail batching does not apply
to bugfix cells). Disjoint file sets → parallel dispatch with reservations.

Risk map:
- Cell 1: LOW — call-site change, primitive already supports the filter.
  Proof: CLI-handler-level test (two agents, same cell id, two sessions).
- Cell 2: LOW — one-argument change. Proof: disagreeing-config fixture.
- Cell 3: MEDIUM — dirty-check exclusion must match git porcelain's rendering
  of the symlink path exactly (quoting, trailing slash). Proof: fixture with a
  companion marker + real untracked symlink + a genuinely dirty file; assert
  refusal preserves the mount; clean case still tears down and merges.

## Test matrix (edge dimensions, scaled)

- **Concurrency/identity:** two sessions, same cell id (cell 1's core case).
- **Absence:** sessionless legacy reservation → cell-only fallback (cell 1);
  no companion marker → no-op path unchanged (cell 3).
- **Disagreement:** root vs controlRoot configs differ on `guards.idle_gate`
  and on phase (cell 2).
- **Ordering:** refused-merge-preserves-mount, clean-merge-tears-down (cell 3).

## Verify

Per-cell scoped: `node scripts/run_verify.mjs --only <suite>` (suites:
`test_reservations`/`test_bee_cli`, `test_guards`, `test_worktree_store`).
Session close: `BEE_VERIFY_CONCURRENCY=12 node scripts/run_verify.mjs
--impacted-from-git`. Full suite stays CI-owned.
