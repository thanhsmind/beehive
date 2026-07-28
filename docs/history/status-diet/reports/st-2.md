# st-2 — Worker contract: embed state line, startup uses brief (D2)

[DONE] Worker Prompt Template (`skills/bee-swarming/references/swarming-reference.md`)
now embeds an orchestrator-filled "State at dispatch" line (phase/feature/
gates.execution) in Identity, and Startup step 2 switched from full
`status --json` to `status --brief --json` — `cells show` stays the claim
authority. Aligned `skills/bee-executing/SKILL.md` and
`skills/bee-executing/references/worker-details.md` the same way (rg-verified
no other worker-startup mention of full status remains). Orchestrator/
session-scout status mentions (tier resolution at line 422, orchestrator's
own compaction resume commands at line 561) left untouched, per D2 scope.

## Verify

`node scripts/skill_budget_fence.mjs && node scripts/skill_lint.mjs && node scripts/okf_instructions_fence.mjs`

```
PASS skill_budget_fence: 18 skill(s) checked, 0 findings
OK — skill tree clean
PASS okf_instructions_fence: 18 instruction-surface file(s) ... 0 unbranched misroutes
```

`bee-executing/SKILL.md` had zero budget headroom (budget == prior size,
10225B, "pending migration" note); trimmed two incidental words elsewhere in
the body (unrelated to status wording) to stay at budget after adding
`--brief`.

## Files + commit

- `skills/bee-swarming/references/swarming-reference.md`
- `skills/bee-executing/SKILL.md`
- `skills/bee-executing/references/worker-details.md`

Commit: 811080e8

## Deviations

None — no architectural change, no bug found, no package install.
