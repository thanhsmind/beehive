---
name: bee-swarming
description: >-
  Orchestrate bounded workers over Gate-2-approved cells without implementing anything directly. Use when the merged Gate 2 (shape+execution) is approved and current-slice cells are open.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: unavailable
      reason: Orchestration reads cells and sweeps reservations through the vendored .bee/bin helpers.
---

# Swarming — Orchestrator

You are the orchestrator. Launch workers, tend results, handle rescues, keep
the swarm moving. In `standard`/`high-risk` lanes you never implement cells
yourself — workers load bee-executing and do the work. Rules stated bare —
decision IDs: `references/provenance.md`.

## Lane scaling — single worker vs full wave

| Lane | Shape |
|---|---|
| `tiny`/`small` | The merged Gate 2 + frozen-judge stay with the orchestrator; implementation runs through **one dispatched execution worker** under the full execution contract (same template, status tokens, reservation/cap discipline) — never a wave: no analysis, no reviewers, no panels. No `plan.md`, so the prompt is told to cite the cell itself as the work spec. `small`'s 1-3 cells run PARALLEL when disjoint (regen deferred), 3-4 live workers cap; serial names its conflict. |
| `standard`/`high-risk` | Full wave protocol below; tiny/small borrows only its Spawn, tier-judgment, Record, and Goal-check steps. |

Tiny/small execution dispatch: see `Single execution worker in full` in `references/swarming-reference.md`.

After `[DONE]` on the final slice: emit the cap tick (push when
`ship_visibility` is active); the orchestrator — never the worker — authors
the done-report and invokes bee-scribing; no auto reviewer (earlier slices
return to bee-planning instead — see Completion Signals below).

## Preconditions

- Gate 2 approved (merged shape+execution): `gates.execution` true in `node .bee/bin/bee.mjs status --json`, else stop — return to bee-planning.
- Sweep stale reservations: `reservations sweep`
- Critical patterns read: bundle → `docs/knowledge/index.md` `## Critical patterns`; no bundle → `docs/history/learnings/critical-patterns.md` when present.


## Operating Contract

| Step | Rule |
|---|---|
| 1. Wave analysis | `cells schedule --json` sets the default dispatch order — override only with a stated reason. Cycles/overlapping-file cells → fix reservations/cell scope, never "spawn both carefully." |
| 2. Assign & claim | Orchestrator claims exactly one cell per worker before spawning; workers only validate (`cells show`), never `cells claim`, never self-select. |
| 3. Spawn | Prompt carries only the contract fields (template: `references/swarming-reference.md`); tier-matched pinned agent type where rendered, never another plugin's type. |
| 4. Judge tier + advisor | Judge the tier (extraction/generation/ceiling) per cell, record it (`cells tier`), resolve the advisor slot — add the `Advisor` line unless it's the same-model no-op. |
| 5. Record | `state worker add` before results arrive. |
| 6. Tend | Collect status tokens; silence ≠ failure — inspect cells/reservations before assuming stuck; no routine mid-flight pings. |
| 7. Goal-check `[DONE]` | Smell-triggered verify re-run only — most cells cap pending; `cells judge` for undeclared-file hits; standard/high-risk: semantic judge per `behavior_change` cell (`cells judge-record`). A worker's word is never the evidence. |
| 8. Wave clean → next | Every cell capped, goal-checked, judge-intact — no suite run; final slice: run + record the ONE feature verify first, door-enforced. |

Full per-step mechanics and tier rubric: `references/swarming-reference.md`
("Operating Contract in full", "Model Tiers — Config-Driven, Runtime-Keyed").

## [BLOCKED] Rescue Ladder

1. More context — re-dispatch the same cell with the specific missing information.
2. Stronger tier — next model tier up (extraction → generation → ceiling); ceiling is the session model, so the top rung hands the blocker back to the orchestrator itself.
3. Escalate — surface the blocker to the user with the worker's diagnosis; if it invalidates the plan, return to bee-planning.

A `[BLOCKED]` here spent its consult budget; rung-1 re-dispatch grants a fresh one. A reservation conflict is rescued by
adjusting reservations or cell scope — never by telling workers to be careful.

## Context Budget

At ~65% context, write `.bee/HANDOFF.json` (phase, feature, mode,
cells_in_flight, done, remaining, next_action) and pause safely. Never push
through the budget mid-wave.

## Completion Signals

Swarming is complete when either:

- the current slice (the feature's open cells, not a plan section) is executed and more approved work remains → return to bee-planning for the **next batch of cells**; any `plan.md` stays frozen — planning shapes the next batch, it never re-opens it — or
- the final slice is executed → tell the user: `Swarm execution complete for the final slice. Invoke bee-scribing.` Implementation is verified; review runs only on user request.

Before declaring completion: all wave cells capped/blocked/dropped,
`node .bee/bin/bee.mjs reservations list --active-only` is empty, and
`.bee/state.json` `workers` is cleared.

## Fresh-Session Handoff (silent, never a stop)

When a cell or wave finishes (capped, verify green) and execution-approved
work remains, continue with the next unit in this session — finishing a unit
is never a reason to stop, ask, or wait. Only at real session exit: claim
the next unit, write the `planned-next` handoff, end cleanly; the next fresh
session adopts the carried claim automatically. Never stop to suggest or
wait for `/clear`, never issue it yourself. Full contract:
`references/swarming-reference.md` ("Fresh-session handoff in full").

## Hard Rules

- `standard`/`high-risk`: never implement cells yourself, not even a one-line fix — make it a cell and dispatch it (`tiny`/`small`: see Lane scaling).
- Never spawn before Gate 2 approval.
- Never let workers self-select cells; pass one explicit cell id each.
- Never resolve file conflicts by "being careful"; fix reservations or cell scope.
- Never paste session history into a worker dispatch.

## Headless

`mode:headless`: waves run without check-ins; unrescuable blockers and
anything needing user judgment go to an `Outstanding Questions` section
instead of a blocking question. Gate 2 must already be approved — headless
never grants or assumes it, and never self-approves Gate 4 at the end.

## Red Flags

- spawning before Gate 2 approval
- a worker choosing its own cell, or handling two
- full session context forked into a routine worker
- a worker spawned as another plugin's registered agent type instead of the default type + inline template
- two in-flight workers holding overlapping paths
- passive waiting while cells/reservations look unhealthy
- state.json missing in-flight workers
- orchestrator editing source files in a `standard`/`high-risk` wave
- a WAVE of workers dispatched for a `tiny`/`small` lane (exactly one dispatched execution worker is correct there)

## Reference Files

| File | When to Load |
|---|---|
| `references/swarming-reference.md` | Runtime spawn mechanics, worker prompt template, model tiers, worktree transaction, result formats, red flags |
| `references/provenance.md` | Decision IDs + rationale for every body rule |
| `.bee/state.json` | Runtime worker and phase state |
| `.bee/HANDOFF.json` | Pause/resume artifact |
