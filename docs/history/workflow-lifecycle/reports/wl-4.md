# wl-4: Declare three new state.mjs exports in census

**Status:** DONE

**Outcome:** Added three workflow-lifecycle exports (closeWorkflowsForFeature, ensureWorkflowRecordForFeature, listWorkflowRecords) to EXPECTED_STATE_EXPORTS in alphabetical position. Test result: 118 passed, 0 failed.

**Files touched:**
- `packages/bee/tests/test_misc.mjs`

**Evidence:** [.bee/cells/wl-4.json](.bee/cells/wl-4.json)

Test output: `node packages/bee/tests/test_misc.mjs` → 118 passed, 0 failed
