[DONE] Fix: 3 impacted-suite tests broken by default-on verify cache

**Outcome:** Fixed three failing assertions in test_run_verify_impacted.mjs by adding `--no-cache` to three run_verify spawn sites. Test suite now passes: 37 passed, 0 failed.

**Files touched:**
- `scripts/tests/test_run_verify_impacted.mjs` — added `--no-cache` flag to three assertions:
  1. Line 79: `--impacted` self-select case
  2. Line 231: `--level 1` case
  3. Line 519: `--only` case

**Context:** These three assertions exercise the selection logic + real execution path in run_verify, expecting to see actual PASS lines from test runs. With tbf-1's cache implementation default-on locally, warm cache was returning "CACHED green" results instead. Adding `--no-cache` ensures these tests validate real execution behavior.

**Verification:** `node scripts/tests/test_run_verify_impacted.mjs` → 37 passed, 0 failed.

**Cell trace:** `.bee/cells/tbf-4.json`
