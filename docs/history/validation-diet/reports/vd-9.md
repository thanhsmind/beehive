# vd-9 — Delete the bee-validating skill and fold its two survivors into bee-planning

**Outcome:** [DONE] — Deleted `skills/bee-validating/` whole (SKILL.md, both
references, `CREATION-LOG.md`, `agents/openai.yaml`; per D1, no replacement
for the feasibility matrix/delta rule per D6). Folded the two survivors into
`skills/bee-planning/SKILL.md` §3 Shape, each as a step a reader acts on, not
a glossary entry:

- **SMALLER PATH check (D1)** — one inline question the moment the shape is
  drafted, every lane: is there a cheaper shape that still honors every
  locked decision. Trimmed the tiny/small merged gate's old 5-item reality
  check (`references/planning-reference.md` "Tiny/small merged gate") down
  to this one survivor.
- **Review wave (D5)** — merged reviewer (structure + cold-pickup) dispatched
  when the shape is drafted, findings held until Gate 2, cost
  `max(reviewer, planning)`. Ported the lane-scaling thresholds and the full
  merged-reviewer prompt + persona panel into a new
  `references/planning-reference.md` § "Review Wave in full" /
  "Merged Reviewer Subagent Prompt" section (Gate 3 → Gate 2 throughout).
- **Spikes opt-in by change class (D8)** — added to §1 Mode Gate: owed only
  for `migration`/`security`/external-side-effect/no-precedent; everything
  else builds directly, no matrix, no delta rule.

**Verify:** exact recorded command,
`node scripts/skill_lint.mjs && node packages/bee/tests/test_misc.mjs`. Both
green — `skill_lint.mjs` exits 0 (1 pre-existing advisory on
`bee-hive/SKILL.md`, untouched by this cell); `test_misc.mjs` 118 passed, 0
failed. Capped via `--feature-verify-pending` per the wave-barrier dispatch.
Also ran `node scripts/tests/test_gate_bypass_doctrine.mjs` (outside this
cell's own verify, but pinned by the must-haves): 11 residual FAILs, all
confined to `skills/bee-hive/SKILL.md`, `skills/bee-swarming/SKILL.md`, and
`packages/bee/AGENTS.block.md` — confirmed by `git diff --stat` untouched by
this cell — exactly the files slice 3 owns per `plan.md`'s Files and order
section. Expected, not a failure of this cell.

**Files + commit:** deleted `skills/bee-validating/{SKILL.md,CREATION-LOG.md,
agents/openai.yaml,references/{validation-reference.md,provenance.md}}`;
edited `skills/bee-planning/SKILL.md`,
`skills/bee-planning/references/planning-reference.md`,
`scripts/tests/test_gate_bypass_doctrine.mjs`, `scripts/skill_lint.mjs`,
`scripts/skill-body-budget.json`. Full trace/evidence: `.bee/cells/vd-9.json`.
No mirror regen — `regen_obligation_ack: wave-barrier`, owed by the
orchestrator at wave close.

**Deviations:** one, recorded on the cell trace. Fixed three dead references
in `skills/bee-planning/SKILL.md` left by the deletion — the Hard Gates
handoff line, the Prep phase hand-off line ("phase `validating`"), and the
footer — since `bee-validating` and the `validating` phase no longer exist
(confirmed against `packages/bee/lib/state.mjs` post-vd-1). Kept these
factual/mechanical (routing text only); did not touch Gate 2's approval CLI
mechanics or redesign the shape+execution merge presentation for
standard/high-risk, since that is D2/vd-3's territory and not cited in this
cell's decisions (D1/D5/D6/D8 only).

**Note for the orchestrator:** `trace.behavior_change` landed `false` after
capping via `--feature-verify-pending` without `--behavior-change` — the
cell record itself carries no top-level `behavior_change` field for `cells
cap` to honor, and `cells update`/a second `cells cap` both refuse post-cap
(door-validated / already-capped). The cell's real content is a behavior
change (a skill deleted, planning's chain rewired); flagging in case the
swarming-close judge tier keys off this trace field.
