# mv-1 — Cap pending path + feature-verify record verb + close-door gate (D1-D3)

**Status:** [DONE]
**Commit:** 0e0fb5fe
**Outcome:** `cells cap --feature-verify-pending` caps evidence-free stamping `trace.feature_verify: "pending"` (combining with per-cell evidence claims is refused; classic path byte-unchanged); `bee state feature-verify record`/`show` stamps `{feature, command, output_sha256, result, at}` on the active feature's tracked record and its workflow-store record (red storable, never satisfying); `guardFeatureVerifyDebt` refuses BOTH swarming exits (`state set` out + `state scribing-run`) until a green record strictly newer than the newest pending cap exists — typed refusal naming the pending cells and the FIX, immune to every gate_bypass level (guardTestCellDebt mirror).

## Files touched

- `packages/bee/lib/cells.mjs` — capCell pending branch + marker stamp
- `packages/bee/bee.mjs` — cap flag pass-through, FLAG_ALONE_BOOLEANS, guardFeatureVerifyDebt at both doors, record/show handlers, dispatch + usage fallback
- `packages/bee/lib/command-registry.mjs` — cells.cap flag, state.feature-verify.record/.show entries beside state.route
- `packages/bee/lib/workflow-store.mjs` — `feature_verify: null` base default (route precedent)
- `packages/bee/tests/test_bee_cli.mjs` — registry-completeness example checks for the new family

## Verify

`node packages/bee/tests/test_bee_cli.mjs && node packages/bee/tests/test_state.mjs && node packages/bee/tests/test_cells.mjs` — 336+44+126 passed, 0 failed. Full trace and structured evidence: `.bee/cells/mv-1.json`.

## Deviations

- Reserved and edited `packages/bee/tests/test_bee_cli.mjs` (7th file, declared in the cell text as "registry examples updated as needed"): the "every registry entry had its example executed" allowlist lives there.
- Declared `lib/state.mjs` and `tests/test_misc.mjs` needed no change (no state.mjs export-surface touch).

## Notes for the orchestrator

- Vendored `.bee/bin` regen deferred per wave-barrier ack — `test_misc.mjs` byte-identity sweep is expectedly red until wave close.
- Pre-existing red (also on HEAD, verified): `test_workflow_store.mjs` createWorkflow/readWorkflow round-trip (route:null defaults asymmetry from explicit-triage) — needs its own fix cell; recorded as friction on the cap.
