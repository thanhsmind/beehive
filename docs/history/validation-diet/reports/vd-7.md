# vd-7 — Fix-first: restore the level-aware gate-bypass tokens the doctrine suite requires

**Outcome:** [DONE] — Derived the required-token set from
`scripts/tests/test_gate_bypass_doctrine.mjs`'s own assertions (not copied
from the dispatch sample) and added the 10 missing tokens to the 3 scoped
skills, each placed inside the step it governs, not bolted on as a keyword
line:

- `skills/bee-exploring/SKILL.md` — Gate 1 handoff step now reads the active
  `gate_bypass_level` and states the `full`/`total` floor-lift before
  presenting the gate; Socratic Locking step now states the info-vs-approval
  litmus ("confident best answer", decision a93994d3) at the point questions
  are filtered.
- `skills/bee-planning/SKILL.md` — Mode Gate section now names "intake
  classification" (D8); the lane-shape table states "request + one cell"
  (tiny, D3) and "plan.md is opt-in" (small, D4); the Gate 2 line reads the
  active `gate_bypass_level` and states the floor-lift.
- `skills/bee-validating/SKILL.md` — Required Inputs gate-in check now reads
  "current-slice cells exist" (D2) and refuses with "stop and return to
  bee-planning" (exact wording the suite pins); the Gate 3 bypass line reads
  the active `gate_bypass_level` before presenting anything.

**Verify:** exact recorded command,
`node scripts/tests/test_gate_bypass_doctrine.mjs && node scripts/skill_lint.mjs`.
All 15 assertions touching the 3 scoped files now pass (0 residual for this
cell's files). The combined command still exits 1: 11 FAILs remain, all in
`skills/bee-hive/SKILL.md`, `skills/bee-swarming/SKILL.md`, and
`packages/bee/AGENTS.block.md` — outside this cell's `files` list, explicitly
owned by not-yet-created slice-2/3 cells `vd-9`/`vd-10` per
`docs/history/validation-diet/plan.md:158-159`. Run standalone,
`node scripts/skill_lint.mjs` exits 0 (1 pre-existing advisory, unrelated to
this cell). Capped via `--feature-verify-pending` rather than a false
passing record — full trace + both run outputs: `.bee/cells/vd-7.json`.

Before-fix repro (captured pre-edit, per dispatch instructions): the same
suite run showed the identical 15 checks as `FAIL` for these 3 files
(`gate_bypass_level`/`full` missing on all three Gate steps; `confident best
answer` missing; D8/D3/D4 planning tokens missing; D2/refusal wording missing
on validating), 21 total FAIL lines at that point in the swarm.

Mirrors: `node scripts/render_plugin_skill_trees.mjs` regenerated
`.claude-plugin/skills/` and `.codex-plugin/skills/` (the two roots this
script renders) plus their `.bee-render.json` sidecars — no hand-edits.

**Files + commit:** `skills/bee-exploring/SKILL.md`,
`skills/bee-planning/SKILL.md`, `skills/bee-validating/SKILL.md`,
`.claude-plugin/skills/{bee-exploring,bee-planning,bee-validating}/SKILL.md`,
`.claude-plugin/skills/.bee-render.json`,
`.codex-plugin/skills/{bee-exploring,bee-planning,bee-validating}/SKILL.md`,
`.codex-plugin/skills/.bee-render.json`. Full trace/evidence:
`.bee/cells/vd-7.json`.

**Deviations:** none. No assertion in `test_gate_bypass_doctrine.mjs` was
weakened, deleted, or replaced.
