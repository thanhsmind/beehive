---
type: bee.area
title: Advisor Protocol — consult triggers
description: "The two consult triggers: the dispatcher's budgeted worker offer and the mandatory orchestrator consult before high-risk execution approval."
timestamp: 2026-07-19
bee:
  id: advisor-protocol-triggers
  lifecycle: active
  areas: [advisor-protocol]
  required_context: [areas/advisor-protocol/overview.md]
  decisions: ["AO2(b)/AO3/AO13 (one orchestrator trigger; execution-gate precondition, folded from the old standalone Gate 3 into Gate 2 by validation-diet D2/D14; event-based staleness, never a TTL)", AO4 (call paths split by trigger class), AO14 (execution-worker class), "126412b9 (precondition keys on the selected record's mode)"]
  sources: ["advisor-and-orchestration Slices 2A-i..2A-iv, 2B, 3A, 3B, 4, 5 (cells ao-2ai-1..ao-5-1, traces in .bee/cells/, reports docs/history/advisor-and-orchestration/reports/, 2026-07-17)", first live orchestrator consult digest .bee/spikes/advisor-and-orchestration/slice5-advisor-digest.txt, "docs/specs/advisor-protocol.md#B1", "docs/specs/advisor-protocol.md#B3", "docs/specs/advisor-protocol.md#E3", "docs/specs/advisor-protocol.md#P2", "docs/specs/advisor-protocol.md#P6"]
  authoritative_for: "advisor-protocol: consult triggers"
---

# Advisor Protocol — Consult Triggers

## Entry Points & Triggers

- **Worker trigger (available, budgeted):** a worker that has just hit its
  first serious failed verification attempt may consult the adviser named in
  its dispatch — at most twice per claim, then it must return blocked.
- **Orchestrator trigger (mandatory, mechanical):** before the execution gate
  opens for work in the high-risk mode, the orchestrator must hold a live
  (non-stale) consult record. The approval verb itself refuses otherwise.
  This is machinery, not a human stop: every autopilot level still runs it.
- No other trigger exists. Conflict-between-decisions and scope-creep triggers
  were considered and explicitly deferred/dropped (they lack a mechanical
  detector today).

## Behaviors & Operations

**B1 — The dispatcher offers the adviser; the worker never self-assesses.**
At dispatch the orchestrator resolves the configured adviser and applies the
one honest no-op; otherwise the dispatch names the adviser and exactly how to
reach it (its proven transport). Workers on the session's strongest tier are
offered advisers too — configuration outranks any strength intuition.

**B3 — The orchestrator consults before high-risk execution approval.** The
orchestrator builds the evidence bundle, runs the adviser **read-only**
(external command: exactly as configured, bundle on standard input, printed
output is the advice; model-shaped: a review-class read-only dispatch), and
records the consult. The approval verb then verifies the record is live; a
missing or stale record refuses the approval with a corrective message naming
each failed condition and the exact consult flow. A workspace with no adviser
configured records that fact and proceeds — the rule adds one trigger, not a
dependency on configuration.

## Edge Cases Settled

- **E3 —** Corrupt or hand-edited consult record → reads as missing; the verb
  never crashes; the approval refuses with the standard message.

## Open Gaps

- The conflict-between-decisions trigger waits on structured decision records
  (its prerequisite feature), and the scope-creep trigger has no source of
  truth; neither is built, neither is silently substituted.

## Pointers (implementation)

- **P2 —** Orchestrator consult + throw: `handleStateGate`'s
  `requireFreshAdvisorForHighRisk` (shared by the standalone `--name execution`
  path and the merged `--merge` path) + `state advisor-ref record/show` in
  `packages/bee/bee.mjs`; helpers `advisorRefAnchors` /
  `advisorRefStale` in `packages/bee/lib/state.mjs`. There is no
  standalone validating skill (deleted, validation-diet D1) — the consult now
  has to happen during planning/briefing, before Gate 2's execution component
  is approved (validation-diet D14).
- **P6 —** Gate precondition spec detail: `docs/specs/workflow-state.md` B9/B9a.
