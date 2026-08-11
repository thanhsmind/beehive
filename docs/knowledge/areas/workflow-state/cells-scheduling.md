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
dependencies with their reasons, and units declaring no paths. Collision
packing also weighs the REGEN OBLIGATION since schedule-regen-awareness cells
sra-1/sra-2 (2026-08-11, PBI p-50f3af4d): two units whose declared paths are
disjoint but which both trigger a regeneration obligation over the same
obligated root — derived through the very same derivation the authoring
refusal uses, never a hand-kept list — conflict exactly like a path overlap
and serialize into different waves, because the shared regeneration chain is
a whole-tree side effect two parallel workers cannot both run. The schedule
names each such split (deferred unit, blocking unit, shared root) in its
payload and its text, so the orchestrator sees why the wave broke. In the
`--json` payload those splits ride a dedicated `obligation_conflicts` array,
one entry per split naming the deferred unit, the blocking unit, and the
shared obligated root (sra-2); when no split occurs the array is empty and
every other byte of the schedule output is identical to the pre-awareness
rendering, so existing consumers see no change until a conflict actually
exists. The query never
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

- R78 — **Parallel is the default dispatch posture, every lane.** Units of the
  same wave dispatch concurrently (3–4 live workers) whenever their product
  file sets are disjoint — reservations prove and police the disjointness;
  serial dispatch is the exception and must name its conflict. A unit touching
  shared generated artifacts (release manifest, plugin mirrors, onboarding
  ledger) may defer their regeneration to the **wave barrier** by declaring
  the recognized skip value on its regen obligation: the orchestrator then
  owes the full regeneration chain exactly once at wave close, in the
  wave-close commit, before the wave is declared clean — removing those
  artifacts from the overlap comparison so the schedule computes truly
  disjoint waves (parallel-default D1/D2, user philosophy decision
  2026-07-28; supersedes the serial-default parallel criterion and rescopes
  the small-lane serial doctrine).

## Pointers (implementation)

- Computed schedule (B17/B18, R26/R27): `packages/bee/lib/schedule.mjs`
  (`computeSchedule`, `detectCycles` — pure, Kahn layering + greedy `pathsOverlap`
  packing, Tarjan SCC for cycles; byte-mirrored to `.bee/bin/lib/`); cycle refusal
  wired in `cells.mjs` `addCell`/`addCells`/`updateCell` via `assertNoCycle`;
  CLI verb `bee cells schedule` (`command-registry.mjs` `cells.schedule`,
  `handleCellsSchedule` in both dispatcher copies); consumer prose in
  `skills/bee-swarming/SKILL.md` (wave analysis),
  `skills/bee-planning/references/planning-reference.md`
  (files-authoring note; the standalone validating skill is deleted — validation-diet
  D1 — its feasibility matrix retired with no replacement, D6). Tests: schedule + cycle-refusal rows in
  `templates/tests/test_lib.mjs` (321 passing), verb example in
  `templates/tests/test_bee_cli.mjs` (132 passing). Evidence: commits 390165a,
  9e2156e, 5003503, 79217ae; traces `.bee/cells/parallel-scheduler-{1..4}.json`.
