# vd-5 — Un-export coerceLegacyPhase so the state export census stops drifting

**Outcome:** [DONE] — vd-2 introduced `coerceLegacyPhase` as an exported
binding at `packages/bee/lib/state.mjs:76`, but `test_misc.mjs` asserts
exact-set equality between `lib/state.mjs`'s export surface and
`EXPECTED_STATE_EXPORTS`, so the extra export turned it red. Confirmed all
three call sites (`:1115`, `:1169`, `:1727`) live inside `state.mjs` itself
via a repo-wide `rg -n coerceLegacyPhase` sweep — no external consumer, so
dropping the `export` keyword is sufficient. `EXPECTED_STATE_EXPORTS` was
**not** widened (per the cell's prohibition — the allowlist is a deliberate
exact-set fence). Mirrored the one-line change into `.bee/bin/lib/state.mjs`
in the same commit; the two files remain byte-identical.

Ran the mandated regen chain in order: `render_plugin_skill_trees.mjs`
(no-op, projections already matched), `onboard_bee.mjs --repo-root . --apply`
(also re-synced a stale `.bee/bin/lib/state-projection.mjs` vendoring
mirror — a pre-existing drift from an earlier cell, unrelated to this fix,
now closed by the "vendored source" byte-identical guard test),
`release_manifest.mjs --write`, `impact_registry.mjs --write`.

**Before (red, cited in the cell, not re-authored):**
```
FAIL  readConfig strips a stale advisor key and never throws; advisor exports are gone
```

**Verify:** `node packages/bee/tests/test_misc.mjs && node packages/bee/tests/test_guards.mjs && node scripts/tests/test_impact_registry.mjs && node scripts/release_manifest.mjs --check`
```
test_misc.mjs: 118 passed, 0 failed
test_guards.mjs: 62 passed, 0 failed
test_impact_registry.mjs: 21 passed, 0 failed
release_manifest --check: 463 file(s) match stored manifest
```
Full before/after text: `.bee/cells/vd-5.json` trace.verification_evidence.

**Files + commit:** `packages/bee/lib/state.mjs`, `.bee/bin/lib/state.mjs`,
`.bee/bin/lib/state-projection.mjs` (regen side-effect), `docs/history/codex-harness-hardening/release-manifest.json`,
`scripts/impact-registry.json`, `.bee/onboarding.json` (regen side-effect).
Commits carry `vd-5`: `db11b709` (fix + regen), `80fbd4f4` (cell trace).
Full trace/evidence: `.bee/cells/vd-5.json`.

**Deviations:** none — the fix matched the cell's declared action exactly;
no architectural change, no widened allowlist, no touched call sites.

**Not mine, left alone:** `scripts/tests/test_state_projection_race.mjs` and
`scripts/tests/test_gate_bypass_doctrine.mjs` are red on baseline `33d58a7e`
already, from work predating this feature; outside this cell's verify.
Also left untouched: `.bee/decisions.jsonl`, `docs/history/validation-diet/CONTEXT.md`,
`docs/history/validation-diet/plan.md` — all three were already dirty in
the working tree before this cell started (confirmed via mtime, predating
my edits), not produced by this cell's action or regen chain.
