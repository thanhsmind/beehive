# Stage: planning (`bee-planning`)

**Purpose** — Turn locked `CONTEXT.md` decisions into the *smallest honest*
execution path: the lane, the approach, the work shape, and — after approval — the
current-slice cells.

**When it runs** — After exploring locks `CONTEXT.md` (Gate 1 approved), or directly
for a clear-scope task ("just fix this"). The route runs **first**, before reading
deeply.

## Inputs
- `CONTEXT.md`, critical patterns, recent decisions.
- The `docs/knowledge/` bundle (or `docs/specs/`), `docs/history/learnings/`.
- `bee orient` / `bee status --json`, the current cells list.
- Reading scales with the lane: tiny keeps two targeted reads; small adds
  `CONTEXT.md` and recent decisions; standard/high-risk add area truth, critical
  patterns, and prior learnings. Precedent beats research; unfamiliar territory or
  competing approaches dispatch `bee-researching`.

## Outputs
- A recorded route: `bee route --set --class <c> --lane <l> --flags <f> --files <n>`.
- The feature worktree for any code-touching route — `bee worktree new --feature
  <slug>`, session opened there. `docs` and a solo `tiny` stay in main.
- `plan.md` (standard/high-risk — **frozen at Gate 2**); opt-in for small; none for
  tiny/spike (the cell is the micro-plan).
- Optional `discovery.md` / `approach.md`; a logged scoping-synthesis decision
  (small lane).
- After approval only: current-slice [cells](../register.md#beecellsfeature-njson),
  in **one batched `bee cells add --stdin` call** — later slices keep one-line
  headlines, not cells.

## Gate
**Gate 2** — "Work shape is ready. Approve before current-work preparation?"
The old standalone execution gate is folded into it: Gate 2 covers execution too, approved
via `bee gate --merge` (flips `approved_gates.shape` and
`approved_gates.execution` in one call). Tiny/small ask it as the **merged** gate —
"Work shape + execution: I'm about to do X via Y, verified by Z. Approve?" — with
the cells *previewed in the gate message*, never persisted-then-previewed. No
source edits happen until `approved_gates.execution` is true.

## State touched
[`bee route --set`](../register.md#beestatejson), `bee worktree new`,
[`cells add --stdin` / `cells tier`](../register.md#beecellsfeature-njson),
`bee gate --name shape` / `--merge`,
[`decisions log`](../register.md#beedecisionsjsonl) (scoping synthesis),
`state set --owner planning --phase swarming`.

## Key rules
- **The route is mechanical flag-counting**, and it runs first. Re-route upward on
  new evidence at any time; de-escalate only on cited evidence.
- **The SMALLER PATH check runs in every lane** — one inline question, one line of
  evidence: is there a cheaper shape that still honors every locked decision? FAIL
  → redraft. Standard/high-risk add the review wave before the gate.
- **Once approved, `plan.md` is frozen** — a stamp may follow, a content edit may
  not. The next slice is shaped as new work; an approved plan is never reopened.
- **Cells only for the current slice** — a future-slice cell does not exist yet.
- **A user-visible surface makes slice 1 a walking skeleton** — end to end, real
  behavior, no stubs.
- **Tests are the writer's, TDD-style, inside each cell** — coverage judgment
  first: cite existing tests by file and case, author only the gap
  (`.bee/expertise/tests.md`). There is no trailing test cell per slice, and no
  per-cell proof tier: `bee cells finish` runs the declared `commands.test` at
  every cap. `commands.verify` is the close/merge chain; CI owns the full estate.
- **Scope integrity**: when the shape will not fit the budget, never quietly shrink
  a locked decision or drop a must-have. Answer SPLIT RECOMMENDED and let the user
  choose what waits; a cheaper swap needs a supersede.
- Stop at Gate 2; no cells, no prep artifacts, before approval.

## Source
`skills/bee-planning/SKILL.md` + `references/{planning-reference, edge-dimensions}.md`;
craft in `.bee/expertise/planning.md` and `.bee/expertise/tests.md`
