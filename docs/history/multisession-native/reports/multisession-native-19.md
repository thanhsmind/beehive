# multisession-native-19 — Workspace registry with single write owner

**Status:** [DONE]

**Outcome:** Added `workspace-store.mjs` (workspace registry at `controlRoot/.bee/runtime/workspaces/<id>.json`, O_EXCL-lock-fenced write ownership with heartbeat-staleness reclaim), wired it into `createFeatureWorktree`/`performCleanup` (register/unregister lifecycle), stamped `workspace_id` through `claims.mjs` (`createSession`, `claimCellFile`) and `hooks/bee-session-init.mjs` (also closing the `ctx.controlRoot` re-root gap the msn-18c cell deferred). Proved grant (store topology) and write ownership (session concurrency) are independent, composable axes across all 4 combinations. 19 new tests, full impacted verify green.

**Files touched:** see `.bee/cells/multisession-native-19.json` trace for the full list and structured `verification_evidence`.

**Commit:** `09e1ed0`

Full trace, deviations, and evidence: `.bee/cells/multisession-native-19.json`.
