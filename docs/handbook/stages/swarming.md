# Stage: swarming (`bee-swarming` — Orchestrate)

**Purpose** — Orchestrate bounded workers over gate-approved cells. The
orchestrator launches and tends; it does not implement.

**When it runs** — After Gate 2's execution approval, with current-slice cells
open, inside the feature's worktree (worktree-first — the main checkout stays clean
for integration).

## Inputs
- `bee cells schedule --json` — the dispatch order; override only with a stated
  reason.
- `bee orient` / [`state.json`](../register.md#beestatejson),
  [reservations](../register.md#beereservationsjson), `CONTEXT.md` / `plan.md`.

## Outputs
- Capped cells with their test records, worker status tokens.
- `.bee/logs/dispatch.jsonl` traces. (Active workers are *derived* — live
  heartbeat sessions joined with cell claims; `state worker.*` verbs are compat
  no-ops.)
- A green `bee close --feature <slug>` at the final slice, and an
  orchestrator-authored done report.

## Gate
None directly — it relies on Gate 2's execution component
(`approved_gates.execution`) already being approved.

## State touched
[`cells schedule/claim/claim-next/show/tier/judge/judge-record`](../register.md#beecellsfeature-njson),
[`bee dispatch prepare --claim`](../register.md#beeclaimscell-idjson) (claims the
cell **and** reserves its files under the worker nickname in one verb — on a
reservation conflict the claim is unwound and nothing is left half-done),
[`reservations reserve/release/sweep/list`](../register.md#beereservationsjson)
(backed by the sharded lease store), `bee close`,
[`HANDOFF.json`](../register.md#beehandoffjson) via the per-workflow mailbox.

## Key rules
- **Never spawn before the execution gate is approved.** In a standard/high-risk
  wave the orchestrator never edits source itself.
- **Concurrency is the default.** Disjoint cells fan out concurrently (reservations
  prove disjointness, 3–4 live workers cap it); serial needs a *named* file
  conflict, dependency, or the user's say-so. From two cells up, state the one-line
  concurrency plan before dispatching. Overlapping-file cells are fixed by scope or
  reservations — never by "spawn both carefully".
- **A `tiny` cell may run inline** in this session; `small` and up always dispatches
  — one worker per cell, and never two cells to one worker.
- **Spawn with exactly the prepared payload.** Never paste session history. Judge
  and record the model tier first (`bee cells tier`).
- **Silence is not failure** — inspect `bee cells list` and `bee reservations list`
  before assuming a worker is stuck.
- **Goal-check every `[DONE]` yourself** — a worker's word is never the evidence.
  `bee cells judge` for undeclared-file hits; at standard/high-risk, one semantic
  judge per slice over its `behavior_change` cells.
- **`[BLOCKED]` has a rescue ladder**: re-dispatch with the missing context → next
  model tier up (the ceiling is this session, so the top rung hands it to you) →
  surface to the user with the worker's diagnosis. If it invalidates the plan,
  return to planning.
- **Slice clean is a door set, not a feeling**: `bee close --feature <slug>
  --dry-run` names every remaining door with the command that settles it; the final
  slice runs `bee close --feature <slug>`, which re-runs the declared tests. Doors
  are never waived.
- **Completion**: more approved work remaining → back to planning for the next
  batch (the approved plan stays frozen). Final slice green → tell the user
  execution is complete, capture is recorded as *pending*, and landing is
  `bee worktree merge` from main. Before declaring done: no active reservations, no
  in-flight workers.
- At ~65% context, write `.bee/HANDOFF.json` and pause cleanly — never push through
  the budget mid-wave. Finishing a unit is never on its own a reason to stop.

## Source
`skills/bee-swarming/SKILL.md` ("Orchestrate") + `references/swarming-reference.md`
