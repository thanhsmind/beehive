# wl-2 — bee state workflows list/close CLI verb

**[DONE]** Added `bee state workflows list [--json]` and `bee state workflows close (--feature <f> | --id <id> | --all-but-active)` to `packages/bee/bee.mjs` and `packages/bee/lib/command-registry.mjs`, closing the rule-12 gap where the orchestrator had to hand-edit `.bee/runtime/workflows/*/state.json` to clear zombie records.

**Files touched:** `packages/bee/bee.mjs`, `packages/bee/lib/command-registry.mjs`

**Commits:** `99b989bf` (initial implementation), `b6d664c4` (import-path fix after MAIN's mid-work contract correction — see Deviations below for why this is two commits, not one).

Full trace/evidence: `.bee/cells/wl-2.json`.

## Deviations

- Mid-work interface-contract correction from MAIN: `listWorkflowRecords`/`closeWorkflowsForFeature` live in `lib/state.mjs`, not `lib/workflow-store.mjs` as the cell text originally said (that leaf module can't import `controlRootFor`). Updated the import and every call site to pass plain `root` (both helpers resolve `controlRootFor` internally).
- Git hygiene incident: the first commit accidentally swept in unrelated files already staged by a concurrent swarm sharing this checkout (`git commit` commits the whole index, not just newly-`git add`-ed paths). Recovered without losing any commit (a stray `reset --soft` was immediately corrected with `reset --hard` back to the concurrent swarm's own commit); the import-path fix landed as a second, cleanly-scoped commit rather than amending the polluted one, to avoid re-orphaning history a second time.
- Left `ensureWorkflowRecordForFeature` unwired from `bee state set --feature` (MAIN's offered bonus) — out of wl-2's scope (a different verb); recommend a follow-up cell.

R82: capped `--feature-verify-pending`; MAIN verifies at feature close (`node packages/bee/tests/test_bee_cli.mjs`).
