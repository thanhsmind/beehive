---
type: bee.area
title: "Workflow State — the attempt history, lifetime budgets, and the audited reset door"
description: "The append-only record of every verification and block on a unit of work, the claim-door budgets that stop a unit from looping forever (including the two-identical-failures rule no autopilot level can waive), and the single audited door that reopens an exhausted unit."
timestamp: 2026-07-22
bee:
  id: workflow-state-cells-attempt-budgets
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["self-correcting-loop D2 with Validating amendments Δ1-Δ3 (append-only attempt ledger, claim-door lifetime budgets, picker skip)", gh-issue-fixes-172 D-GHF-B (heartbeat-invariant acquisition identity in the attempt ledger and claim counting)]
  sources: ["self-correcting-loop cells scl-1..scl-5 (traces in .bee/cells/, reports docs/history/self-correcting-loop/reports/, 2026-07-19)", "gh-issue-fixes-172 cells ghf-1/ghf-3..ghf-6 (traces in .bee/cells/, GH #23/#27, 2026-07-20)", "docs/specs/workflow-state.md#B26", "docs/specs/workflow-state.md#B27", "docs/specs/workflow-state.md#B28", "docs/specs/workflow-state.md#R41", "docs/specs/workflow-state.md#R42", "docs/specs/workflow-state.md#R43", "docs/specs/workflow-state.md#R44", "docs/specs/workflow-state.md#R45", "docs/specs/workflow-state.md#E23", "docs/specs/workflow-state.md#E24", "docs/specs/workflow-state.md#P20"]
  authoritative_for: "workflow-state: unit attempt history, lifetime budgets, and budget reset"
---

# Workflow State — the attempt history, lifetime budgets, and the audited reset door

A unit that fails the same way forever is not persistence, it is a loop — and
the only way to see one is to remember. This concept owns that memory: one
append-only entry per verification or block, budgets measured over the unit's
whole life and enforced inside the very operation that grants a claim, and one
door back in that always costs a stated reason and a durable decision.

## Behaviors & Operations

**B26 — Every verification or block appends one entry to a unit's attempt
history.** Trigger: recording a verification result (pass or fail) or blocking
a unit of work. What happens: exactly one entry is appended — a sequence
number, when, the owning session and the moment it acquired its claim, the
worker, the outcome, a failure signature (worker-supplied or mechanically
derived from the verification output), and an optional note. The history is
append-only: no revision path may edit or remove an entry, and entries survive
completion — the same execution-trace freeze that already refuses a plan
revision touching the trace (B7) covers the whole history. What each actor
observes: a unit with no history yet (every legacy unit) behaves exactly as it
always did (self-correcting-loop D1, Δ1).

**B27 — A unit's lifetime budget is enforced at the moment of claiming, inside
the same exclusive operation that decides ownership (B11).** Trigger: a
session claims a unit, whether by identity or through the automatic picker.
What happens: the claim is granted first, then its budgets are checked against
the attempt history; exceeding the claim-count or failed-attempt ceiling, or
two failed attempts sharing an identical failure signature, refuses the claim
and releases the just-granted claim in the same operation, so the caller never
ends up holding a claim it was refused — the enforcement therefore lands at
the very next claim attempt, bounded to at most one attempt of overrun. The
refusal names the exhausted budget (or the repeated signature), a summary of
the attempt history, and the sanctioned door (a budget reset). The automatic
work picker skips a budget-exhausted or repeated-failure unit when selecting
the next unit, so one looping unit never bricks the whole pool of ready work —
only claiming a unit by its identity surfaces the refusal directly. No
autopilot level ever overrides either refusal: these are loop-safety stops on
a unit's own history, not human approval gates, and they answer the same "no"
whether a human or the agent is driving. A unit with no attempt history
(legacy) is measured against the unstated defaults and behaves exactly as
before until it actually starts looping. What each actor observes: normal
single-attempt work is unaffected; a unit that keeps failing the same way
stops being reclaimable until someone explicitly resets it with a reason
(self-correcting-loop D2, Δ1-Δ3).

**B28 — A budget reset is the sole door back into an exhausted unit, and it is
never silent.** Trigger: an operator decides a unit's approach has genuinely
changed and it deserves a fresh budget. What happens: the reset requires a
reason, records a durable decision, and appends a reset marker to the unit's
own record; it never rewrites or removes a prior attempt-history entry. What
each actor observes: after a reset, lifetime-budget accounting (B27) resumes
counting from the marker forward, so attempts before the reset stop being held
against the unit's new attempts (self-correcting-loop D2).

## Business Rules

- R41 — A unit's lifetime budget is checked inside the same exclusive
  operation that grants its claim, not before or after it; exceeding it
  releases the just-granted claim in the same step and answers with a typed
  refusal naming the exhausted budget, the attempt history, and the reset
  door — enforcement is therefore bounded to at most one attempt of overrun
  (self-correcting-loop D2, Δ2).
- R42 — Two failed attempts sharing an identical failure signature exhaust a
  unit immediately: the signal is that the approach must change, not that
  another try is warranted (self-correcting-loop D2).
- R43 — The automatic work picker skips a budget-exhausted or
  repeated-failure unit rather than surfacing the refusal, so one looping
  unit never bricks the pool of ready work; only claiming a unit by identity
  gets the typed refusal directly (self-correcting-loop D2, Δ3).
- R44 — No autopilot level ever overrides a budget or repeated-failure
  refusal: these are loop-safety stops on a unit's own history, not human
  approval gates (self-correcting-loop D2).
- R45 — A budget reset is the only door back into an exhausted unit: it
  requires a stated reason, records a durable decision, and appends a marker
  without ever touching the attempt history itself (self-correcting-loop D2).

## Edge Cases Settled

- A unit with no attempt history yet (every legacy unit) is measured against
  the default lifetime budgets and behaves exactly as it always did — until it
  actually loops (self-correcting-loop D2, D6).
- An autopilot level set to `total` still does not waive a budget or
  repeated-failure refusal — proven by a dedicated test row exercising that
  exact combination (self-correcting-loop D2).

## Pointers (implementation)

- Self-correcting loop (B26-B32, R41-R50): attempt ledger `appendAttempt` /
  exported `normalizeFailureSignature` in
  `packages/bee/lib/cells.mjs` (byte-mirrored to `.bee/bin/lib/`),
  invoked from `recordVerify` (both outcomes) and `blockCell`; lifetime
  budgets `checkCellBudgets` runs inside `claimCellCrossSession`'s O_EXCL
  critical section with unwind-on-refusal, typed `CELL_BUDGET_EXHAUSTED` /
  `REPEATED_FAILURE`, skip-on-select wired into `claimNextCell`, reset verb
  `resetCellBudget` / CLI `cells reset-budget --id --reason`; change
  classification `deriveChangeClass`, authoring advisory
  `JUDGE_STANDARD_INSUFFICIENT` (STDERR only via the `bee.mjs` handler layer,
  pah-2 precedent), behavior-class completion teeth (evidence length +
  tolerant duplicate scan) in `capCell`; judge-verdict schema
  `validateJudgeVerdict` / `deriveModelIndependence` in new
  `packages/bee/lib/judge.mjs` (byte-mirrored to
  `.bee/bin/lib/`), recorder `recordJudgeVerdict` in `cells.mjs`, CLI verb
  `cells judge-record --id --file <verdict.json> [--builder-model]
  [--judge-model]`; goal-check judge-tier table single-homed in
  `skills/bee-hive/references/routing-and-contracts.md` with a one-line
  565e68d0-scoping clause mirrored onto the seven adjacent surfaces
  (`bee-swarming` SKILL + reference, `bee-hive` SKILL x2 sites, go-mode,
  `AGENTS.md` + `templates/AGENTS.block.md`, `bee-scribing` SKILL). Evidence:
  `docs/history/self-correcting-loop/CONTEXT.md` (D1-D6, Validating
  amendments Δ1-Δ6, decisions 84e49851/1cb27fbf); traces
  `.bee/cells/scl-{1..5}.json`; reports
  `docs/history/self-correcting-loop/reports/scl-{1..5}.md`.
