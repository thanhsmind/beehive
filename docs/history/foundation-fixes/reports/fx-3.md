# fx-3 — Slice test: workflow-close regression net + split verification (D4)

**Status:** [DONE]

**Outcome:** Located the real owning suites for each assertion (not the
cell's hedged single-file guess) and extended them: (a) startFeature-chain-
of-3 in `packages/bee/tests/test_state.mjs`; (b) pickNewestActiveWorkflow
zombie-exclusion and (c) rebuildStateProjection no-resurrection in
`packages/bee/tests/test_state_projection.mjs`; fx-2's split-suite registry
assertion in `scripts/tests/test_impact_registry.mjs`. All four suites
green: 15 + 44 + 26 + 21 passed, 0 failed. `test_workflow_store.mjs`
untouched (self-declared session/state-free; wrong home for these
assertions).

**Deviation:** the cell's `verify` field (`test_workflow_store.mjs &&
test_state.mjs`) and `files` list assumed one suite would own all four
assertions. Investigation showed `test_workflow_store.mjs` structurally
excludes state.mjs/state-projection.mjs coverage by its own contract, so the
real owners are three separate existing suites. Ran the cell's literal
verify command exactly (recorded via `cells verify`, green) plus the two
additional owning suites as supplementary proof that assertions b/c/registry
are actually green. Ratio-ceiling waiver recorded (test-only cell, zero
source lines by design). No fx-1 logic touched; no regen run (wave-barrier).

**Files touched:**
- `packages/bee/tests/test_state.mjs`
- `packages/bee/tests/test_state_projection.mjs`
- `scripts/tests/test_impact_registry.mjs`

Full trace/evidence: `.bee/cells/fx-3.json`.
