# Approach: Worktree Concurrency Guard — Port to bee 1.18.2

## Recommended path

Perform one real `git merge origin/main --no-commit --no-ff` on `wt/worktree-concurrency-guard` (a safety tag `pre-1.18.2-port-backup` already exists at the pre-merge tip, `56b437aa`), then resolve every conflict in a single continuous pass — mechanical/generated files first (accept upstream, then regen), then the three real code files (`packages/bee/lib/guards.mjs`, `packages/bee/hooks/bee-write-guard.mjs`, `packages/bee/bee.mjs`), then the two relocated test files, then run the full verify suite before committing the merge. This must be ONE cell, not several: a live, uncommitted `git merge --no-commit` is shared mutable state across the whole working tree — it cannot be split across independently-claimed cells without them stomping on each other's uncommitted resolution. The cell is scoped as a single ceiling-tier unit of work (per D1-D8, cites CONTEXT.md).

## Rejected alternatives

- **Rebase instead of merge** — rejected per Port-D1: would force-push an already-reviewed, open PR branch.
- **Splitting the merge into multiple cells** (one per conflicted file) — rejected: a `git merge --no-commit` conflict state is one shared working tree, not independently claimable per-file; two "cells" both touching the live merge would race on the same uncommitted git state, exactly the class of bug this whole feature exists to prevent.
- **Hand-merging generated/bookkeeping files** — rejected per Port-D2: regenerate instead, avoids shipping an internally-inconsistent ledger.

## Risk map

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| controlRoot threading through 2 functions | Medium | Port-D4/D5/D6 are evidence-backed but not yet proven against a real merge conflict resolution — the exact signature shape (new param vs opts field) is explicitly deferred to this cell's own judgment (CONTEXT.md Outstanding Questions) | The cell must pick one shape, apply it consistently to both `isSharedNestedCheckoutTarget` and `hasAnySharedNestedCheckout`, and prove it with the ported tests plus a new controlRoot-specific test case (sibling-worktree-shares-controlRoot scenario) |
| Scan-vs-concurrency root scoping (Port-D4) | Medium | Fresh-eyes review flagged this as the single most likely mistake — a naive `root`→`controlRoot` swap would scan the wrong physical tree | Explicit test: a target physically under `root` but with a DIFFERENT `controlRoot` (simulating linked-worktree topology) is still detected on the filesystem scan, while the concurrency check correctly consults `controlRoot`'s session records |
| The ~20-file merge conflict resolves without losing any of D1-D6/D3/D4/D5 (original feature) or the P1 fixes | High | This is effectively re-applying 5 already-shipped, already-reviewed cells' worth of logic by hand during conflict resolution — a transcription error would silently regress a fixed P1 | Full existing test suite (85+17 rows across both relocated test files) must pass unchanged in verdict, plus the new controlRoot-specific rows |
| Mechanical/generated file regen produces a byte-consistent result | Low | Same regen chain already proven 5 times this session (wcg-1 through wcg-fix-2) | `ledger_parity.mjs --check`, `release_manifest.mjs --check`, `knowledge check` all clean |

## Files and order

1. `git merge origin/main --no-commit --no-ff` (real, not preview).
2. Mechanical/generated conflicts: accept upstream (`git checkout --theirs <path>` where safe, or the regen command's own output) for manifest/ledger/index files; re-run `render_plugin_skill_trees.mjs` (if it still exists at 1.18.2 — verify first), `onboard_bee.mjs --apply`, `knowledge index`, `backlog render --write`.
3. `packages/bee/lib/guards.mjs` — re-apply the 10 relocated exports/helpers, controlRoot-scoped per Port-D4.
4. `packages/bee/hooks/bee-write-guard.mjs` — re-apply the pre-`checkWrite` check using `ctx.controlRoot` (Port-D5).
5. `packages/bee/bee.mjs` `handleWorktreeNew` — re-apply the pre-creation refusal using `controlRootFor(mainRoot)` + `resolveSessionId` + plain-Error convention (Port-D6/D7).
6. `packages/bee/hooks/test_write_guard.mjs`, `scripts/tests/test_worktree_companion.mjs` — re-apply all proven rows at the new paths (Port-D8), plus new controlRoot-specific rows (risk map above).
7. Full verify, then commit the merge.

## Relevant learnings

- This session's own promoted pattern `docs/knowledge/patterns/20260724-scheduler-blind-to-regen-side-effects.md` — not directly applicable here (no parallel cells), but the underlying lesson (declare/verify every side-effect explicitly) applies to the regen step.
- `docs/history/worktree-concurrency-guard/reports/review-20260724.md` — the two P1s already found and fixed; this port must not reintroduce either (self-exclusion, fail-closed-on-error).

## Questions for validating (resolved during planning)

- **Resolved:** `render_plugin_skill_trees.mjs` still exists at the same path (`scripts/render_plugin_skill_trees.mjs`), still runnable, but its `TARGET_ROOTS` now only lists `claude: .claude-plugin/skills` and `codex: .codex-plugin/skills` — confirmed via `git show origin/main:scripts/render_plugin_skill_trees.mjs`. **`.agents/skills/` and plain `.claude/skills/` are retired as render targets entirely** — upstream consolidated to 2 plugin mirrors instead of 4. `onboard_bee.mjs` relocated to `packages/bee/scripts/onboard_bee.mjs` (still exists, same role).
- This changes step 2 of Files and order: mechanical conflict resolution for `.agents/skills/*` and `.claude/skills/*` paths is **deletion, not regeneration** (matching upstream's own removal) — only `.claude-plugin/`, `.codex-plugin/`, and `packages/bee/` need the regen chain re-run.
