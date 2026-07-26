---
type: bee.area
title: Workflow State — the computed dispatch schedule and the cycle refused at the door
description: "The dispatch plan derived fresh from declared dependencies and declared touched paths — numbered waves plus diagnostics, never stored, never guessed — and the write-time refusal that makes an impossible dependency graph impossible to record."
timestamp: 2026-07-22
bee:
  id: workflow-state-cells-scheduling
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["parallel-scheduler D1-D4 (docs/history/parallel-scheduler/CONTEXT.md; logged a648ea2a/b4740f68/ecc8862d/eec223d9, D2 clarified 0746db88)"]
  sources: ["parallel-scheduler cells parallel-scheduler-1..5 (traces in .bee/cells/, reports docs/history/parallel-scheduler/reports/, 2026-07-15/16; -5: review fix scoping refusal to introduced/participating cycles)", "docs/specs/workflow-state.md#B17", "docs/specs/workflow-state.md#B18", "docs/specs/workflow-state.md#R26", "docs/specs/workflow-state.md#R27", "docs/specs/workflow-state.md#P19"]
  authoritative_for: "workflow-state: the computed dispatch schedule and dependency-cycle refusal"
---

# Workflow State — the computed dispatch schedule and the cycle refused at the door

Two units that declare overlapping paths are not a mistake — they are a wave
apart. This concept owns the derivation that says so: a schedule computed on
demand from what the units themselves declare, using the very same overlap
meaning the runtime write refusal enforces, so prediction and enforcement can
never disagree. And it owns the one thing the schedule refuses to tolerate at
all: a dependency cycle, stopped at the write rather than diagnosed later.

## Behaviors & Operations

**B17 — The schedule is computed, not guessed.** Trigger: anyone asks for a
feature's dispatch plan (a read-only schedule query, filtered to one feature or
spanning all work). What happens: the schedule is derived fresh from the
declared record — dependency layering first (a dependency on a completed unit
counts as satisfied), then collision packing (declared-path overlap within a
layer defers the colliding unit to a later wave, in deterministic id order) —
and answered as numbered waves plus diagnostics: dependency cycles, unsatisfiable
dependencies with their reasons, and units declaring no paths. The query never
writes anything. What each consumer observes: the orchestrator dispatches wave
by wave and deviates only with a stated reason; feasibility validation of a
multi-unit slice requires the diagnostics to be clean before execution is
approved; a planner sees that overlapping declared paths are legal but cost a
wave, so partitioning quality is visible at plan time instead of surfacing as
mid-flight write refusals. The runtime write refusal stays in place unchanged —
the schedule predicts it; it never replaces it (parallel-scheduler D1/D2/D4).

**B18 — A dependency cycle is refused at the door.** Trigger: any write that
creates or changes a unit's declared dependencies — adding one unit, adding a
batch, or updating an existing unit's dependencies. What happens: the write is
checked against the union of the existing record and the incoming change; the
refusal is scoped to cycles the write itself introduces or participates in
(self-dependency included) — if any member of a resulting cycle is part of the
incoming change, the entire write is refused before anything lands — a batch
is all-or-nothing — and the refusal names the cycle's member ids. The
structural check spans units of every status. A cycle that exists only among
untouched pre-existing records never blocks an unrelated write: a legacy store
with a cycle is reported by the schedule query's diagnostics, and the only
writes it refuses are ones that would keep one of its own members inside the
cycle — a change that breaks the cycle is always allowed. What the caller
observes: an immediate, specific "no" at write time, so an impossible plan is
impossible to record; pre-existing records are never mutated by the check
(parallel-scheduler D2; scope sharpened by review fix parallel-scheduler-5).

## Business Rules

- R26 — No dependency cycle can ever be recorded: every write that creates or
  changes declared dependencies is refused, all-or-nothing and naming the
  cycle, when the union of the record and the change would contain one. A
  cycle that predates the rule is surfaced by the schedule query's
  diagnostics, never silently scheduled around (parallel-scheduler D2,
  decisions b4740f68/0746db88).
- R27 — One overlap semantics, two consumers: the computed schedule judges
  declared-path collisions with exactly the same meaning the runtime write
  refusal enforces. Collision between ready units is legal and auto-serializes
  into a later wave — it is never refused, and never dispatched concurrently.
  The computed schedule is the default dispatch order; deviating requires a
  stated reason, and execution of a multi-unit slice is not validated while
  the schedule's diagnostics report cycles (parallel-scheduler D1/D2/D3,
  decisions a648ea2a/ecc8862d).

## Pointers (implementation)

- Computed schedule (B17/B18, R26/R27): `packages/bee/lib/schedule.mjs`
  (`computeSchedule`, `detectCycles` — pure, Kahn layering + greedy `pathsOverlap`
  packing, Tarjan SCC for cycles; byte-mirrored to `.bee/bin/lib/`); cycle refusal
  wired in `cells.mjs` `addCell`/`addCells`/`updateCell` via `assertNoCycle`;
  CLI verb `bee cells schedule` (`command-registry.mjs` `cells.schedule`,
  `handleCellsSchedule` in both dispatcher copies); consumer prose in
  `skills/bee-swarming/SKILL.md` (wave analysis), `skills/bee-validating/SKILL.md`
  (feasibility matrix), `skills/bee-planning/references/planning-reference.md`
  (files-authoring note). Tests: schedule + cycle-refusal rows in
  `templates/tests/test_lib.mjs` (321 passing), verb example in
  `templates/tests/test_bee_cli.mjs` (132 passing). Evidence: commits 390165a,
  9e2156e, 5003503, 79217ae; traces `.bee/cells/parallel-scheduler-{1..4}.json`.
