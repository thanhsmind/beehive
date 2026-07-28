# mv-3 — Slice test: feature-verify law net behavior (D1-D3)

**Status:** [DONE]
**Commit:** f0599d2b
**Outcome:** Trailing net over mv-1's relocated proof law, in the owning suites (extended, never forked). `test_cells.mjs`: (a) pending cap stores `trace.feature_verify: "pending"` with zero per-cell evidence demanded; (b) classic evidence path stays byte-identical — still refuses without verify, no stray `feature_verify` field on a classically-capped trace. `test_bee_cli.mjs`: (c) `state feature-verify record` green/red round-trips with a REAL computed `output_sha256` (never caller-supplied), `--show` reads it back; (d) the close door refuses on no-record / red-record / stale-record, passes on a fresh green record, and stays untouched with zero pending cells — proven at BOTH swarming exits (`state set` + `state scribing-run`); (e) `gate_bypass: "total"` does NOT lift the door — refused identically to no bypass at all.

## Files touched

- `packages/bee/tests/test_cells.mjs` — 2 new checks: pending-cap assertion (a), classic-path byte-identity assertion (b)
- `packages/bee/tests/test_bee_cli.mjs` — 7 new checks: record verb round-trip (c), 5 door-refusal/pass/untouched shapes + both-doors mirror (d), gate_bypass total non-lift (e)

## Verify

`node packages/bee/tests/test_bee_cli.mjs && node packages/bee/tests/test_cells.mjs` — 344 + 128 passed, 0 failed. Full trace and structured evidence: `.bee/cells/mv-3.json`.

## Deviations

None — mv-1's cap pending path, record/show verbs, and `guardFeatureVerifyDebt` were exercised exactly as landed; no source touched, tests only.

## Notes for the orchestrator

- Classic verify+cap law used per dispatch override (this cell runs under the OLD law, not the new main-verifies doctrine it is itself testing).
- Ratio waiver recorded at cap (test-economy D3, standard lane, test-only diff) — audited decision logged, ceiling itself untouched.
- Vendored `.bee/bin` regen deferred per wave-barrier ack, per this cell's own prohibition.
