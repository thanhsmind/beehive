# mv-4 — Fix workflow-store create/read round-trip asymmetry

[DONE]

**Outcome:** `createWorkflow`'s record literal now spreads `baseWorkflowDefaults()`
first, before the caller-value overrides, so `route`/`feature_verify` (and any
future default field) land on the created record the same way `readWorkflow`
already synthesizes them on read — created and read are now byte-symmetric.
No test assertion was weakened; the round-trip `deepEqual(read, created)`
check is unchanged and now passes for the correct reason.

**Files touched:**
- `packages/bee/lib/workflow-store.mjs`

**Verify:** `node packages/bee/tests/test_workflow_store.mjs` — 15 passed, 0 failed (was 14/1).

**Commit:** `e7a79063` — fix(main-verifies): mv-4 — symmetrize createWorkflow record with baseWorkflowDefaults

Full trace/evidence: `.bee/cells/mv-4.json`.
