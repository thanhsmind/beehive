# Stage: planning (`bee-planning`)

**Purpose** — Turn locked `CONTEXT.md` decisions into the *smallest believable*
execution path: the mode, the approach, the work shape, and the current-slice cells.

**When it runs** — After exploring locks `CONTEXT.md` (Gate 1 approved), or directly
for a clear-scope task that needs a mode decision. The mode gate runs **first**.

## Inputs
- `CONTEXT.md`, critical patterns, recent decisions.
- The `docs/knowledge/` bundle (or `docs/specs/`), `docs/history/learnings/`.
- `status --json`, current cells list.

## Outputs
- `plan.md` (standard/high-risk — **frozen at Gate 2**); opt-in for small; none for
  tiny/spike (the cell is the micro-plan).
- Optional `discovery.md` / `approach.md` (L2+ discovery or high-risk).
- A logged scoping-synthesis decision (small lane).
- Current-slice [cells](../register.md#beecellsfeature-njson) (`cells add`).

## Gate
**Gate 2** — "Work shape is ready. Approve before current-work preparation?"
The old standalone Gate 3 is folded into Gate 2: there is no `validating`
phase to hand off to, so Gate 2 also covers execution, approved via `bee
state gate --merge` (flips `approved_gates.shape` and `approved_gates.execution`
together in one call). Tiny/small ask this as the **merged** gate: "Work
shape + execution: I'm about to do X via Y, verified by Z. Approve?" No
source edits happen until `approved_gates.execution` is true.

## State touched
`cells add/tier`, `state set --owner planning --phase swarming`,
`state gate --name shape` / `--merge`, [`decisions`](../register.md#beedecisionsjsonl)
(scoping synthesis).

## Key rules
- **The mode gate is mechanical flag-counting**, and it runs first (D8).
- **Once approved, `plan.md` is frozen** — only an approval stamp may be written
  after Gate 2.
- **Cells only for the current slice** — future-slice cells are prohibited.
- Stop at Gate 2; no cell creation before shape approval.

## Source
`skills/bee-planning/SKILL.md`
