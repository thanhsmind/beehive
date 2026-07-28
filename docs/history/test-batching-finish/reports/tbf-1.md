# tbf-1 — Suite-result cache in run_verify.mjs

**Status:** [DONE]

**Outcome:** Added a closure-hash-keyed suite-result cache to `scripts/run_verify.mjs`. A suite whose impact-registry closure content is byte-identical to its last GREEN run is skipped ("CACHED green `<label>` (closure unchanged)"). Reuses the existing closure walk (`buildRegistry()`) rather than reimplementing it. Red is never cached; disabled under CI env / `--no-cache`; `--cache-clear` wipes the file; missing/corrupt cache fails open. Applies to both full/`--only` and `--impacted` runs via the shared `runSelectedSuites` tail.

**Files:** `scripts/run_verify.mjs`, `scripts/impact-registry.json` (regenerated — pre-existing drift, required for the chain's own `--check` suite).

**Commit:** `e7e1caba`

**Verify:** Cell's declared `verify` (`node scripts/tests/test_verify_cache.mjs`) doesn't exist yet — it is tbf-2's own deliverable. This cell was manually demoed per dispatch instruction: cold run → green real run → identical rerun → CACHED; closure edit → forces rerun; forced red → never cached; `CI=true`/`--no-cache` → real run; `--cache-clear` → wipes; corrupt cache → fails open. Regression: `test_run_verify_impacted.mjs` 37/37 green, `impact_registry.mjs --check` up to date. Full trace/evidence: `.bee/cells/tbf-1.json`.

**Deviations:** Regenerated `scripts/impact-registry.json` (pre-existing drift unrelated to this cell, needed to keep the `impact_registry.mjs --check` chain member green). Details in the cell trace.

Next: tbf-2 (`scripts/tests/test_verify_cache.mjs`, dep on tbf-1) owes the automated coverage.
