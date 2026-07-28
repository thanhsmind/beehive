# vd-1 — Remove the validating phase from the enum and retarget both hardcoded guards

**Outcome:** [DONE] — Per D3, dropped `'validating'` from `PHASES`
(state.mjs), narrowed `GATED_PHASES` to `{'exploring','planning'}`
(guards.mjs), and narrowed `PHASE_GATE` to `{ planning: "execution" }`
(bee-session-close.mjs, value moved from `"shape"` to `"execution"`), each
mirrored byte-identically into its `.bee/bin` twin. Adjacent stale
comments describing the old phase/gate sets updated to match (no logic
change). `isDebtGuardedDeparture`, `TERMINAL_PHASES`, `SCRIBING_RUN_FROM`
untouched, as required.

**Verify:** `node packages/bee/tests/test_misc.mjs && node packages/bee/tests/test_cli_state.mjs && node packages/bee/tests/test_state.mjs && node packages/bee/tests/test_guards.mjs`
```
test_misc.mjs: 118 passed, 0 failed
test_cli_state.mjs: 120 passed, 0 failed
test_state.mjs: 44 passed, 0 failed
test_guards.mjs: 58 passed, 0 failed
(combined exit 0)
```
Red-first evidence (captured before the test migrations, per dispatch
instructions): after editing only the 3 core files, `test_cli_state.mjs`
ran 117 passed/3 failed on the unknown-phase refusal for hand-written
`phase: 'validating'` fixtures; `test_guards.mjs` ran 57 passed/1 failed
("NET branch 6 ... gate deny at validating, got {\"allow\":true}"). Full
before/after text: `.bee/cells/vd-1.json` trace.verification_evidence.

**Files + commit:** `packages/bee/lib/state.mjs`, `packages/bee/lib/guards.mjs`,
`packages/bee/hooks/bee-session-close.mjs` + their 3 `.bee/bin` twins,
`packages/bee/tests/test_cli_state.mjs`, `packages/bee/tests/test_guards.mjs`.
Commit `4a03006ecab29ad9f764f3165c1ce6ee66fa0803`. Full trace/evidence:
`.bee/cells/vd-1.json`.

**Deviations:**
- Migrated a 7th hand-written `'validating'` phase site in
  `test_cli_state.mjs` (line ~3417, `--phase validating` in the p2-1
  departure-guard test) that the advisor's six named sites didn't cover —
  found via a full-file grep. Retargeted to `--phase swarming` (intent is
  "debt-guard fires on every departure," not specifically `validating`).
- Edited `packages/bee/tests/test_guards.mjs` (outside this cell's
  declared `files`, reserved before writing) to migrate NET branch 6's
  loop fixture off the retired 3-member `GATED_PHASES` set — explicitly
  anticipated by the dispatch prompt as "yours to migrate."
