# fx-2 — Split test_worktree_store.mjs (D3)

**Status:** [DONE]

**Outcome:** Split `packages/bee/tests/test_worktree_store.mjs` (13 tests)
verbatim at the D3 topology/merge boundary: 4 store/topology tests stay in
`test_worktree_store.mjs`; 9 merge-path tests (checkProcessorLease/
onVerifyTick hooks, companion-teardown ordering) moved to new
`test_worktree_store_merge.mjs`. Fixture helpers (`git`,
`makeOrdinaryRepoFixture`, plus merge-only `gitText`/`lstatExists`/companion
helpers) are duplicated rather than shared across files, so the runner never
double-executes either half — this repo has no pre-existing cross-suite
fixture-module pattern to follow instead. Impact registry regenerated
(`--write` then `--check` green). No test deleted/weakened, no timeout
raised, no CI yml touched. 4 + 9 = 13/13 green.

**Deviation:** cell's `change_class` was set to `"refactor"` at planning,
which test-economy D1 unconditionally blocks from adding a new `test_*.mjs`
file (no override) — the split necessarily creates one. Reported
`[BLOCKED]` with the conflict and an advisor consult (fable, recommending
block-and-escalate over a worker-side unclaim/reclaim workaround); the
orchestrator reclassified `change_class` to `"test"` (audited decision) and
re-claimed the cell under exec-fx2, unblocking the cap with no further
implementation change needed. Also fixed a small leftover: the split
orphaned an unused `gitText` helper in `test_worktree_store.mjs` (only the
merge file still needs it) — removed, header comment updated to match.

**Files touched:**
- `packages/bee/tests/test_worktree_store.mjs`
- `packages/bee/tests/test_worktree_store_merge.mjs` (new)
- `scripts/impact-registry.json`

Full trace/evidence: `.bee/cells/fx-2.json`.

## Consults

1 consult — advisor: fable. Ask: given the change_class/test-economy-D1
classification conflict, should I self-fix via unclaim/patch/reclaim, or
block and escalate? Answer: block-and-escalate — reconciling D3's split
mandate against test-economy D1's unconditional no-override guard is
planning/orchestrator altitude, not worker-owned; the unclaim→update→reclaim
path is a workaround around a guard that ships with no override by design.
The orchestrator subsequently performed the reclassification itself,
confirming the recommendation.
