# wl-1 — Record on every start path + close by feature (library side)

[DONE]

**Outcome:** `packages/bee/lib/state.mjs` now routes every workflow-record creation through one
seam (`ensureWorkflowRecordForFeature`) and closes the outgoing work BY FEATURE
(`closeWorkflowsForFeature({ keepFeature })`) instead of by record presence, so a feature that
never got a record can no longer stay `status: "active"` forever and be resurrected by the
projection's idle-bootstrap picker.

**Interface contract exported (for wl-2, from `lib/state.mjs`):**

- `listWorkflowRecords(root)` → array of full records (control-root resolved, fail-open).
- `closeWorkflowsForFeature(root, { keepFeature })` → array of closed `{id, feature}`; idempotent.
- (bonus seam, also exported) `ensureWorkflowRecordForFeature(root, {...})` → `{record, created}`.

**Files touched:** `packages/bee/lib/state.mjs` (only — `lib/workflow-store.mjs` needed no change).

**Verify owner:** main, at feature close (R82) — capped `--feature-verify-pending`.

Full trace/evidence: `.bee/cells/wl-1.json`.
