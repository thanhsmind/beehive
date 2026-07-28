# vd-6 — Classify the five unclassified state verbs in the projection-race coverage gate

**Outcome:** [DONE] — `scripts/tests/test_state_projection_race.mjs` check
(f) refused 5 state verbs added by commits `0e0fb5fe`/`99b989bf` without the
suite being extended: `state.route`, `state.feature-verify.record`,
`state.feature-verify.show`, `state.workflows.list`, `state.workflows.close`.
Read each handler in `packages/bee/bee.mjs` rather than guessing:

- `state.route` (`handleStateRoute`, `--set` mode) and
  `state.feature-verify.record` (`handleStateFeatureVerifyRecord`) both run
  `withMutationLock(root, null, false, ...)` → `target.write(state)` — the
  identical read-modify-write path `state.set`/`worker.add` already use.
  → **`STATE_WRITING_VERBS`**, each given a held-lock probe (`route-set`,
  `feature-verify-record`) mirroring the existing `worker-add` probe;
  `feature-verify-record`'s fixture writes a real `--output-file` so the
  handler's sha256 hashing path is genuinely exercised.
- `state.feature-verify.show` is a **separate** manifest entry
  (`handleStateFeatureVerifyShow`) that only calls `resolveMutationTarget`
  and reads `.feature_verify` back — `target.write()` is never called.
- `state.workflows.list` (`handleStateWorkflowsList` →
  `listWorkflowRecords`) is a plain read of
  `.bee/runtime/workflows/*/state.json`, no write at all.
- `state.workflows.close` (`handleStateWorkflowsClose`) does write, but only
  the workflow-store record via `updateWorkflow`/`updateWorkflowAssumingLock`
  under that record's own `workflow:<id>` lock (`closeWorkflowRecordById`,
  `closeWorkflowsForFeature` in `lib/state.mjs`) — never `.bee/state.json`
  or a lane file, and never through `target.write()`.
  → all three **`NON_PROJECTION_VERBS`**, each with a comment citing the
  exact code path read.

No blanket dump into `NON_PROJECTION_VERBS` — the two writers were placed
correctly and given real probes, closing the escape hatch check (f) exists
to guard. `packages/bee/` source untouched.

**Before (red, captured live — not re-authored, cell noted this reproduces
on baseline `33d58a7e`):**
```
✗ (f) canonical packages/bee/bee.mjs: unclassified state verb(s) state.route,
  state.feature-verify.record, state.feature-verify.show, state.workflows.list,
  state.workflows.close — a new writer of .bee/state.json could be escaping this suite.
✗ (f) vendored .bee/bin/bee.mjs: unclassified state verb(s) state.route,
  state.feature-verify.record, state.feature-verify.show, state.workflows.list,
  state.workflows.close — a new writer of .bee/state.json could be escaping this suite.
```

**Verify:** `node scripts/tests/test_state_projection_race.mjs`
```
(c) HELD-LOCK  route-set            held=state  wrote-during-hold=false expect-blocked=true  verb-exit=0
(c) HELD-LOCK  feature-verify-record held=state  wrote-during-hold=false expect-blocked=true  verb-exit=0
(f) COVERAGE    31 state verbs in the CLI manifest, all classified
PASS — every production writer serializes on the lock for the record it writes
```
Run twice (race-based suite) — both clean, exit 0. Full before/after text:
`.bee/cells/vd-6.json` trace.verify_output.

**Files + commit:** `scripts/tests/test_state_projection_race.mjs` only.
Commit carries `vd-6`: `93f27928`. Full trace/evidence: `.bee/cells/vd-6.json`.

**Deviations:** none — matched the cell's declared action exactly; no
architectural change, no weakening of check (f).

**Not mine, left alone:** a sibling worker (vd-7) has `skills/**` and
`scripts/tests/test_gate_bypass_doctrine.mjs` in flight in the same
checkout; those and other pre-existing dirty files
(`.bee/decisions.jsonl`, `docs/history/validation-diet/CONTEXT.md`,
`docs/history/validation-diet/plan.md`, the rendered skill mirrors) were
left untouched — committed path-scoped (`git commit -m ... --
scripts/tests/test_state_projection_race.mjs`) per the concurrent-worker
git guard, not `git add`.
