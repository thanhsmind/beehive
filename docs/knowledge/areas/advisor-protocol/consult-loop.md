---
type: bee.area
title: Advisor Protocol — worker consult loop
description: "How a stuck worker consults inside its own turn, the two-per-claim budget, and why advice never approves, overrides, or writes."
timestamp: 2026-07-19
bee:
  id: advisor-protocol-consult-loop
  lifecycle: active
  areas: [advisor-protocol]
  required_context: [areas/advisor-protocol/overview.md]
  decisions: [advisor D1-D3, f1ca79b9 (AO15 — attribution fields)]
  sources: ["advisor cells adv-1..adv-3 (worker consult loop, 2026-07-13)", "advisor-and-orchestration Slices 2A-i..2A-iv, 2B, 3A, 3B, 4, 5 (cells ao-2ai-1..ao-5-1, traces in .bee/cells/, reports docs/history/advisor-and-orchestration/reports/, 2026-07-17)", "docs/specs/advisor-protocol.md#B2", "docs/specs/advisor-protocol.md#B4", "docs/specs/advisor-protocol.md#R7", "docs/specs/advisor-protocol.md#P1"]
  authoritative_for: "advisor-protocol: worker consult loop"
---

# Advisor Protocol — Worker Consult Loop

## Behaviors & Operations

**B2 — A stuck worker consults inside its own turn.** After its first serious
failed verification attempt, the worker sends the evidence bundle and applies
the reply itself. Advice that contradicts a locked decision converts to a
block citing both. The consult and its outcome land in the work unit's trace,
so the cap record shows what was asked and what came back.

**B4 — Advice is advice.** It never approves a gate, never overrides a locked
decision, never edits anything, and is never accepted as verification — the
orchestrator still re-runs every verify itself. The mechanism meant to catch a
mistake is not allowed to make one: advice-class transports are refused write
privileges at configuration checking.

## Business Rules

- **R7 —** The worker budget is two per claim; exhaustion returns blocked.

## Pointers (implementation)

- **P1 —** Worker loop: `skills/bee-swarming/references/worker-details.md`
  ("Advisor consult in full"); dispatch-time offer + same-model no-op:
  `skills/bee-swarming/SKILL.md`.
