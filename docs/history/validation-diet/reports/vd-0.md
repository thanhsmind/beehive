# vd-0 — Fix-first: retire the pre-diet AGENTS-block literal in the onboarding assertion

**Outcome:** [DONE] — `test_onboard_bee.mjs:303` pinned a pre-diet literal
(`node .bee/bin/bee.mjs status --json`) that a16db517 (agents-block-diet)
retired everywhere in the shipped block. Updated the assertion literal to
`bee.mjs status --json`, matching the bare-command convention now live at
`packages/bee/AGENTS.block.md:9`. No edit to `AGENTS.block.md` or root
`AGENTS.md`; `check()` title unchanged.

**Verify:** `node packages/bee/scripts/test_onboard_bee.mjs && node scripts/release_manifest.mjs --check`
```
ok    - AGENTS block mentions bee.mjs status first step
PASS - failures: 0, skipped: 1
release_manifest --check: 463 file(s) match stored manifest
```
Before-fix repro (captured pre-edit, per dispatch instructions):
```
FAIL - AGENTS block mentions bee.mjs status first step
FAIL - failures: 1, skipped: 1
```

**Files + commit:** `packages/bee/scripts/test_onboard_bee.mjs`,
`docs/history/codex-harness-hardening/release-manifest.json` (regen chain
picked up the manifest diff; `render_plugin_skill_trees.mjs` and
`onboard_bee.mjs --apply` were no-ops content-wise). Full trace/evidence:
`.bee/cells/vd-0.json`.

**Deviations:** none.
