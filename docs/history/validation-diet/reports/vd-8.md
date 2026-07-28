# vd-8 — Remove the state validation-cache verbs entirely

**Outcome:** [DONE] — Per D7, `bee state validation-cache record` and
`bee state validation-cache check` are gone: command surface, implementation,
exports, tests, and the gitignore-managed cache file. No deprecation window,
no dormant code.

Re-derived call sites from scratch (`rg -n 'validation.?cache|validationCache|
VALIDATION_CACHE|VALIDATION_SOURCE_ABSENT' packages/bee scripts .bee/bin`)
rather than trusting CONTEXT.md's planning-time inventory, whose line numbers
had already shifted. Removed:

- `packages/bee/lib/state.mjs` — the whole `validationCacheCheck`/
  `writeValidationCache` block (`VALIDATION_CACHE_VERSION`,
  `VALIDATION_SOURCE_ABSENT_SENTINEL`, path/hash/normalize/read/check/write
  helpers), ~311 lines.
- `packages/bee/lib/command-registry.mjs` — both `state.validation-cache.record`
  / `.check` registry entries and their doc comment.
- `packages/bee/bee.mjs` — the `validationCacheCheck`/`writeValidationCache`
  import, both handler functions, the dispatcher branch, the handler-map
  entries, and the `Unknown command` usage line; reworded two prose comments
  (near `FEATURE_VERIFY_RESULTS`) that referenced validation-cache only by
  analogy, so no orphaned mention survives.
- `packages/bee/tests/test_misc.mjs` — the four exports dropped from
  `EXPECTED_STATE_EXPORTS` (the exact-set census cell vd-5 already burned
  this trap on, in the opposite direction).
- `packages/bee/tests/test_bee_cli.mjs` — the entire `state.validation-cache`
  test section (registry examples, staleness/degradation matrix, refusal
  cases), pre-change lines 3453-3731.
- `packages/bee/scripts/onboard_bee.mjs` / `test_onboard_bee.mjs` — the
  `.bee/validation-cache.json` gitignore-managed-paths entry and its fixture.
- Deleted the orphaned, untracked `.bee/validation-cache.json` data file left
  over from before this cut.

Left alone, on purpose: the root `.gitignore`'s own line for
`.bee/validation-cache.json` — it is a mechanically rendered artifact of
`onboard_bee.mjs`'s `renderGitignoreBlock()` (source already fixed), not in
this cell's declared files, and no test in the verify chain checks it; it
self-corrects on the next onboarding regen (`regen_obligation_ack:
wave-barrier`).

All three `packages/bee/` ↔ `.bee/bin/` twin pairs (`state.mjs`,
`command-registry.mjs`, `bee.mjs`) stayed byte-identical throughout — checked
after every edit, not just at the end.

**Before (build-emitted, not re-authored — this is a removal, not a bugfix,
so D9's "before" is the prior behavior's presence):** `git show HEAD` on the
pre-change commit shows `state.mjs:2321,2325,2493,2589` exporting all four
names, `test_misc.mjs:1011-1014` carrying them in the allowlist,
`command-registry.mjs:1176,1194` registering both verbs, and
`bee.mjs:7956-7957` wiring both handlers.

**Verify:** `node packages/bee/tests/test_misc.mjs && node packages/bee/tests/test_bee_cli.mjs && node packages/bee/scripts/test_onboard_bee.mjs`
```
test_misc.mjs:        118 passed, 0 failed
test_bee_cli.mjs:      375 passed, 0 failed
test_onboard_bee.mjs:  PASS - failures: 0, skipped: 1
combined exit: 0
```
Post-edit sweep (`rg -n` over the same pattern across `packages/bee scripts
.bee/bin`) returns zero hits. Full before/after text and the full verify
transcript: `.bee/cells/vd-8.json` trace.verification_evidence /
trace.verify_output.

**Files + commit:** `packages/bee/lib/state.mjs`, `packages/bee/lib/command-registry.mjs`,
`packages/bee/bee.mjs`, `.bee/bin/lib/state.mjs`, `.bee/bin/lib/command-registry.mjs`,
`.bee/bin/bee.mjs`, `packages/bee/tests/test_bee_cli.mjs`, `packages/bee/tests/test_misc.mjs`,
`packages/bee/scripts/onboard_bee.mjs`, `packages/bee/scripts/test_onboard_bee.mjs`.
Full trace/evidence: `.bee/cells/vd-8.json`.

**Deviations:** none — the removal matched the cell's declared action exactly;
no architectural change, no widened allowlist, no touched call sites outside
the four constants/functions and their callers.

**Not mine, left alone:** `.bee/decisions.jsonl` and `docs/decisions/taxonomy.json`
were already dirty before this cell started (Gate 1-3 approval decisions and
a vd-4 ratio-waiver decision, plus a taxonomy tag addition, all predating
vd-8); `.bee/cells/vd-9.json`, `docs/history/validation-diet/CONTEXT.md`,
`docs/history/validation-diet/plan.md` are likewise pre-existing untracked
planning artifacts, none produced by this cell.
