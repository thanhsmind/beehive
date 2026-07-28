---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-07-28 (gate bypass TOTAL — audit decision logged)
---

# Foundation Fixes — Plan

CONTEXT: `docs/history/foundation-fixes/CONTEXT.md` (D1–D4).

## Mode-Gate Record

Flags: 2 (data model — workflow lifecycle gains a terminal status; changes
behavior existing tests may assert — projection/bootstrap paths are covered)
→ standard. Product files ≈ 5.

## Approach

One slice. **Wave 1 (parallel, wave-barrier acks): fx-1 ∥ fx-2** — disjoint
file sets (state machine vs test suite split). **Wave 2: fx-3** trailing test
cell, net behavior of both fixes.

- **fx-1 (D1+D2):** `startFeature` closes the outgoing live workflow
  (`status: 'closed'`) in the same guarded mutation that creates the new one;
  `pickNewestActiveWorkflow` excludes `phase === 'compounding-complete'`.
  Existing suites must stay green (run the workflow/projection-owning suites
  targeted).
- **fx-2 (D3):** split `test_worktree_store.mjs` at the topology/merge
  boundary into itself + `test_worktree_store_merge.mjs` (auto-discovered);
  shared fixture helpers stay importable by both; impact-registry regen in the
  same commit; both halves green locally.
- **fx-3 (D4):** one test cell — regression net: (a) startFeature closes the
  prior workflow (status flips, exactly one live per feature chain); (b) a
  seeded zombie (`active` + `compounding-complete`) is never picked by the
  bootstrap; (c) a state-sync rebuild with zombies present preserves the
  current feature; (d) both split suites run green and the registry maps them.
- **Barrier (orchestrator, wave close):** mirror render → onboard --apply →
  manifest --write/--check once, in the close commit.

## Risk Map

| Component | Risk | Proof |
|---|---|---|
| Workflow close transition | MEDIUM | fx-3 net + existing workflow-store suites |
| Picker filter | LOW | fx-3 zombie case |
| Suite split | LOW | both halves green; registry check |

## Test Matrix (targeted)

Close-on-start (chain of 3 features → 2 closed, 1 live) · zombie never picked ·
rebuild idempotent with current feature live · split suites: no test lost
(count 13 preserved across both), fixtures shared, both green.
