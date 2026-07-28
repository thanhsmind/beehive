# diet-1 — skill_budget_fence.mjs blocking fence

**[DONE]**

Created `scripts/skill_budget_fence.mjs` as a new blocking chain-fail fence
(okf-fence pattern) that owns the skill body byte-budget ratchet plus the D8
provenance grep end to end, promoting them from `skill_lint.mjs`'s
always-advisory checks to a blocking check per D6. Re-seeded
`scripts/skill-body-budget.json` to exact current byte sizes (`migrated: []`,
`notes.<skill> = "pending migration"` for every skill over 8192), registered
the selftest+bare pair in `scripts/run_verify.mjs` beside the okf fences, and
trimmed `scripts/skill_lint.mjs` to keep only its advisory anchor-integrity
and ordered-list checks.

## Files touched

- `scripts/skill_budget_fence.mjs` (new)
- `scripts/skill-body-budget.json`
- `scripts/skill_lint.mjs`
- `scripts/run_verify.mjs`

## Verify

`node scripts/skill_budget_fence.mjs --selftest && node scripts/skill_budget_fence.mjs && node scripts/skill_lint.mjs` — all green (recorded in `.bee/cells/diet-1.json`).

Full trace, evidence, and verify output: `.bee/cells/diet-1.json`.

Commit: `5a183153` (`feat(skill-token-diet): skill_budget_fence.mjs blocking fence [diet-1]`).
