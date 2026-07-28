# diet-6 — P6 regrowth law text in bee-writing-skills + bee-evolving (D5)

**Status:** [DONE]
**Outcome:** D5 compounding law added to both skills: `bee-writing-skills` gets
a checklist item ("Regrowth law") in its PHASE 2 SKILL.md checklist;
`bee-evolving` gets a "Learning placement" rule under step 3 (the fix
hand-off). Both bodies stayed under their grandfathered baselines by trimming
reassurance/duplication in the same body (one-in-one-out, no rule weakened):
`bee-writing-skills` 9154 -> 8968 bytes, `bee-evolving` 8850 -> 8828 bytes.
Both baseline entries lowered via `--update-baseline`. No provenance
citations added (both skills stay unmigrated and bare). Regen obligation ran
in-cell: plugin mirror render, `onboard_bee.mjs --apply`, release manifest
`--write`/`--check`. Full verify chain green.

**Files:** `skills/bee-writing-skills/SKILL.md`, `skills/bee-evolving/SKILL.md`,
`scripts/skill-body-budget.json`, `docs/history/codex-harness-hardening/release-manifest.json`,
plus regenerated mirrors (`.claude/skills`, `.claude-plugin/skills`,
`.codex-plugin/skills`, `.agents/skills`, their `.bee-render.json` stamps,
`.bee/onboarding.json`).

Full trace and verify output: `.bee/cells/diet-6.json`.

## Notes

- `bee-writing-skills` trims: dropped a marketing-style N=28,000 citation
  parenthetical, removed a rationalization-table row duplicating row 1's
  "agents differ from you" point, and collapsed PHASE 4's "Manual checks"
  list (which restated the PHASE 2 checklist verbatim) into a pointer back to
  that checklist — single source instead of two copies.
- `bee-evolving` trims: dropped meta-commentary from the intro paragraph,
  tightened the step-0 "No exceptions" paragraph (kept all four excuses,
  shorter phrasing), removed a rationalization-table row duplicating step 6's
  "any remote ref is a push" bullet, and tightened the three Gate A bullets
  (kept all three claims, shorter phrasing).
- Both additions are bare — no `(D\d`, `AO\d`, `decision <hash>`,
  `hardening-\d`, or `plan \d` citation — consistent with the cell's
  prohibition even though the fence's provenance grep only fires on skills
  listed in `migrated[]` (neither skill is).
