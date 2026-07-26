# Stage: validating (`bee-validating`)

**Purpose** — Prove the plan against repo reality with concrete evidence *before*
any code is written. The hard gate between planning and execution.

**When it runs** — After Gate 2 (plan approved) for standard/high-risk lanes. For
tiny/small the reality check collapses inline into planning and this skill is not
separately invoked — but it never *disappears*.

## Inputs
- `CONTEXT.md`, the frozen `plan.md`, discovery/approach content.
- Current-slice [cells](../register.md#beecellsfeature-njson) (`cells list`).

## Outputs
- Reality-gate scores, a feasibility matrix, spike results, plan-checker and
  cell-review findings.
- Machine report `docs/history/<feature>/reports/validation-<slice>.md`.
- An `advisor_ref` record for high-risk work.

## Gate
**Gate 3** — "Feasibility validated. Approve execution?" **No source-editing
execution happens before this gate.**

## State touched
`state gate --name execution`, `state set --owner validating --phase swarming`,
[`state advisor-ref record`](../register.md#beestatejson), `.bee/spikes/<feature>/`.

## Key rules
- **Accepted evidence only** — "should work" / "likely" / model knowledge scores
  **NOT READY**. A failed reality gate or a NO spike halts and returns to planning.
- Plan-checker runs **max 3 iterations**, then escalates.
- Verify scripts and executable code never live in `docs/history/` — only `.md`.
- High-risk work requires a **non-stale `advisor_ref`** before Gate 3 can be
  approved (enforced as a throw, not a warning).

## Source
`skills/bee-validating/SKILL.md`
