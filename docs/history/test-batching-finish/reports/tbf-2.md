# tbf-2 — Slice test: verify-cache behavior suite

**Status:** [DONE]

**Outcome:** Added `scripts/tests/test_verify_cache.mjs`, a fixture-based suite proving run_verify.mjs's tbf-1 suite-result cache end to end: cold populate, warm CACHED skip, closure-file-edit invalidation (exactly the dependent suite, not the unrelated one), red-never-cached (including the first green run after reds still executes for real), corrupt-cache fail-open (never a crash, cache repaired), CI env var and `--no-cache` both bypass (cache file left byte-untouched), and `--cache-clear` wipes + forces a real run. Each of the 8 checks builds a throwaway temp-dir mini-repo (byte copies of `run_verify.mjs` + `impact_registry.mjs` plus fake controllable-exit-code suites); the live `.bee/logs/verify-cache.json` and real `scripts/tests` are never read or written.

**Files:** `scripts/tests/test_verify_cache.mjs` (new), `scripts/impact-registry.json` (regenerated — required so the new suite's own closure is registered; `impact_registry.mjs --check` demanded it).

**Commit:** `c5e3a566`

**Verify:** `node scripts/tests/test_verify_cache.mjs` → 8 passed, 0 failed. Regression: `test_verify_manifest.mjs` and `test_impact_registry.mjs` both green after the registry regen. Full trace/evidence: `.bee/cells/tbf-2.json`.

**Deviations:** None.
