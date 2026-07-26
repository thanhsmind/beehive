# packages-restructure-3 — report

**Status:** [DONE]

**Outcome:** Zero-diff proof cell. Confirmed cells 1-2's move (`skills/bee-hive/templates` -> `packages/bee/`,
`hooks/` -> `packages/bee/hooks/`) is fully settled and needed no further fixes here. Re-ran the full regen
obligation chain in the cell's specified order (`impact_registry.mjs --write`, `render_plugin_skill_trees.mjs`,
`onboard_bee.mjs --apply`, `release_manifest.mjs --write`) and every artifact came back byte-identical to what
cells 1-2 already produced. Self-onboard reports `up_to_date` with an empty plan (no drift, no orphan lib-module
removals). Distribution-proof suites and the full suite all green.

**Files touched:** none in source scope (true zero diff on all 6 declared paths: `scripts/impact-registry.json`,
`scripts/tests/test_installers_e2e.mjs`, `skills/bee-hive/scripts/test_plugin_distribution.mjs`,
`.claude-plugin/`, `.codex-plugin/`, `docs/history/codex-harness-hardening/release-manifest.json`). Only this
cell's own trace file was written: `.bee/cells/packages-restructure-3.json`.

**Verification:** `BEE_VERIFY_CONCURRENCY=12 node scripts/run_verify.mjs && node scripts/release_manifest.mjs --check`
→ PASS, 105 suites green, 404-file manifest match. Distribution-proof suites re-run explicitly and independently
green (`test_installers_e2e --installer bash`, `test_plugin_distribution`, `test_lib_mirror`, `packages/bee/tests/test_misc.mjs`).
Full trace/evidence: `.bee/cells/packages-restructure-3.json`.

**Deviations:** none — no source edits were needed; nothing to fix-forward.

**Note on a must_haves inaccuracy (not a defect, left as-is):** the cell text expected `.bee/bin/lib` module
count "unchanged vs pre-refactor 34". Checked directly against the pre-refactor commit (`4ce99d1`, before cell 1)
— it already had 35 files, not 34, and `test_lib_mirror` independently proves `packages/bee/lib` and
`.bee/bin/lib` are still byte-identical at 35 files today. The real invariant (parity, no orphan add/remove from
the move) holds; the "34" figure in the cell's must_haves was simply an inaccurate baseline recorded during
planning, not a real drift.

Full trace and evidence: `.bee/cells/packages-restructure-3.json`.
