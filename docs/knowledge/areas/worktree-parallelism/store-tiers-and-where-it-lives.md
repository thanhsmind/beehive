---
type: bee.area
title: "Worktree Parallelism — the three store tiers, and where the mechanism lives"
description: "The log/cache/runtime classification that decides what a git merge is allowed to carry back from a worktree and what must never travel, realized by git config rather than a directory move — plus the module, resolver, CLI and test map for the whole area."
timestamp: 2026-07-22
bee:
  id: worktree-parallelism-store-tiers-and-where-it-lives
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [areas/worktree-parallelism/returning-and-the-merge-gate.md]
  decisions: ["worktree-feature-parallelism (three lifecycle tiers realized by git config, no directory move)", cross-worktree-holds D1-D6 (the shared ledger module and its three seam taps)]
  sources: [docs/history/worktree-feature-parallelism/, "docs/specs/worktree-parallelism.md#S-the-three-tiers-what-merges-what-does-not", "docs/specs/worktree-parallelism.md#S-where-it-lives-reading-map"]
  authoritative_for: "worktree-parallelism: store lifecycle tiers, merge safety, and the implementation map"
---

# Worktree Parallelism — Store Tiers and Where It Lives

The merge gate decides *whether* a worktree's work returns. This concept owns *what* returns
with it: which parts of a worktree's own store are tracked and union-merged, which are
rebuilt, and which must never cross a merge at all — plus the map of where every piece of
the mechanism is implemented and proven.

## The three tiers (what merges, what does not)

The store is classified into three lifecycle tiers, realized by git config (no directory move):
- **log tier** — append-only event logs (decisions, backlog, review-candidates). **Tracked**,
  with a `merge=union` git attribute so parallel worktree branches union-merge their provenance
  on `git merge` instead of conflicting. Readers/`replayLog` dedup by event id, so interleaved
  duplicates fold. This is how a worktree's decisions/provenance travel back to main.
- **cache tier** — derived, disposable state (phase/gate state, lanes). Gitignored; rebuilt by
  replaying the log. Never merged.
- **runtime tier** — live coordination (sessions, claims, reservations, the worktree grant
  registry). Gitignored; TTL/heartbeat lifetimes. Never merged — a merged stale hold is a bug.

## Where it lives (reading map)

- Decision + replay logic: `worktree-store.mjs` (`decideWorktreeStore`, `replayLog`,
  `readGrants`, `writeGrant`, `bootstrapWorktreeStore`, `createFeatureWorktree`,
  `mergeFeatureWorktree` — the last two dependency-free; the CLI handler resolves
  config/roots and passes them in).
- Resolution: `resolveRoots` in the state library (throwing) and the hook adapter
  (non-throwing, import-light — grant read inlined). Both expose `{id, mainRoot, worktreeRoot}`
  for a linked-valid worktree.
- CLI: the `worktree` command group.
- Merge safety: `.gitattributes` (log tier) + the onboarding gitignore block (runtime/cache tiers).
- Shared ledger: `templates/lib/worktree-holds.mjs` (mirror/release/foreign-lookup/sweep,
  corrupt-check); seam wiring in `templates/bee.mjs` reservations handlers +
  `performCleanup`; claim-next tap in `templates/lib/cells.mjs`; guard tap in
  `templates/lib/guards.mjs` (`resolveHoldTopology` in all three, same shape).
- Tests: resolver P40 regression, grant-resolve, worktree-store unit, worktree CLI e2e,
  `scripts/tests/test_worktree_holds.mjs` (seam), `scripts/tests/test_worktree_holds_race.mjs`
  (concurrency), claim-next foreign-skip rows in `test_cli_cells.mjs`, guard net + foreign
  rows in `templates/tests/test_guards.mjs` (all discovered by the verify pipeline).
