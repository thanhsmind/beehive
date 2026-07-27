# diet-2 — fence selftest extended to the plan's full test matrix

**[DONE]**

Extended `scripts/skill_budget_fence.mjs`'s `--selftest` from diet-1's 9
bite-proving fixtures to the plan's full test matrix (`plan.md` §Test Matrix):
added 5 new fixtures — body exactly 8,192 bytes passes; 8,193 fails with
delta 1; a budget entry of exactly 8,192 needs no note but still turns the
provenance grep on when the skill is listed in `migrated[]`; a second
`--update-baseline` run on a stable baseline reports "nothing to lower"
verbatim; and a grown entry is never raised across two consecutive
`--update-baseline` runs. No diet-1 fixture (A-I) or fence logic changed —
net new coverage only, all fixtures build in `os.tmpdir()` temp dirs, the
live `skills/` tree and baseline JSON are never touched by tests.

## Files touched

- `scripts/skill_budget_fence.mjs`

## Verify

`node scripts/skill_budget_fence.mjs --selftest && node scripts/skill_budget_fence.mjs`
— 14 selftest fixtures pass (9 diet-1 + 5 new), live fence run: 18 skills
checked, 0 findings. Recorded in `.bee/cells/diet-2.json`.

Full trace, evidence, and verify output: `.bee/cells/diet-2.json`.
