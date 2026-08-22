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

**The island's cell inventory is feature-scoped at bootstrap — but the prune never touches a
tracked file (worktree-store-hygiene cell wsh-1, corrected same-day by island-prune-safety
cell ips-1, 2026-08-10; PBI p-9c48a67c).** Cell files and archives are git-TRACKED, so `git
worktree add` checks the whole `.bee/cells` tree out into a fresh island before the store
bootstrap runs. The bootstrap reconciles `.bee/cells` by the cell's own feature field: the
granted feature's cells are filled in from the main store when missing (main read-only), and
foreign-feature STRAYS are pruned — but only UNTRACKED ones. A tracked foreign cell stays on
disk untouched: wsh-1's first ship pruned tracked files too, which manufactured tracked
deletions in the island that a later `worktree merge` would have applied to MAIN, wiping the
cell archive — caught live pre-merge the same day. The invariant is pinned by test: after
bootstrap from a real worktree checkout, `git status` inside the island is EMPTY; when git is
unavailable the prune runs not at all (fail safe). A SYMLINKED store is refused outright:
symlink metadata is checked on the island `.bee`, `.bee/cells`, the source cells dir, the
destination archive dir, and every archive dir before any prune or fill — one symlink skips the
whole sync, named in the bootstrap report, never followed (review B-P1-1: a symlinked
`.bee/cells` made git track only the link, emptied the tracked-set shield, and the prune deleted
through into the target; review-p1-batch-fixes cell rpb-1, 2026-08-11; the destination-archive
join and the `worktree new` surface joined under review-p2-hardening cell rph-2). Further
hardening from rph-2: `worktree register` validates `--feature` against the same slug rule
`worktree new` uses BEFORE any path join; the tracked-set lookup fails CLOSED — one unparseable
`ls-files` line means prune-nothing, never the empty-set prune-everything branch; a prune that
removed anything names the pruned files in the bootstrap report; and `worktree new` prints the
cellsSync skip note `register` already printed (review D-P2-1: the refusal used to be invisible
on the primary entry point). The read-side filter shipped
(island-read-filter cell irf-1, 2026-08-11, closing PBI p-9c48a67c): inside a granted island,
cell ENUMERATION scopes to the granted feature at both read seams (the cells listing and the
status/archive-aware listing) — foreign tracked files stay on disk but stop appearing as work
in status counts, ready lists, and claim scans; an ungranted or main-checkout read is
byte-identical, the filter never engages there. Known residue, named on the cell: the
`cells list/ready/claim-next` CLI verbs still refuse-and-delegate from a granted worktree at
root resolution, so the filter is live through `status`/`orient`/`finish` today and unit-level
for the rest until those doors widen.

## A worktree's entry point is the main checkout's binary

A worktree session's bee entry point resolves to the main checkout's
binary, not to the build the worktree just made. Any check that must judge
the change under test — a manifest render, a parity check, a live probe —
runs the freshly built binary by its path; running the entry point instead
silently checks the old behavior and passes. Observed on agent-activity-hook
cell aah-2 (2026-08-22): a manifest check through the entry point compared
against the pre-change catalog until the worker pointed it at the new build.

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
  corrupt-check); seam wiring in `templates/bee` reservations handlers +
  `performCleanup`; claim-next tap in `templates/lib/cells.mjs`; guard tap in
  `templates/lib/guards.mjs` (`resolveHoldTopology` in all three, same shape).
- Tests: resolver P40 regression, grant-resolve, worktree-store unit, worktree CLI e2e,
  `scripts/tests/test_worktree_holds.mjs` (seam), `scripts/tests/test_worktree_holds_race.mjs`
  (concurrency), claim-next foreign-skip rows in `test_cli_cells.mjs`, guard net + foreign
  rows in `templates/tests/test_guards.mjs` (all discovered by the verify pipeline).
