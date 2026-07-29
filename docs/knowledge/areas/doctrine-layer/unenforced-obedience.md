---
type: bee.area
title: Doctrine Layer — unenforced obedience and the human boundary
description: "The rules with no runtime behind them: obey where no guard covers the action, amend doctrine even when the phase gate is shut, run the machinery yourself, and keep its vocabulary out of the conversation."
timestamp: 2026-07-29
bee:
  id: doctrine-layer-unenforced-obedience
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md]
  decisions: [c2c46488 (an unblocked write is not an approved write), 1689af1b (silent bookkeeping), 4439bd7e (purpose-first narration + intent-carrying dispatch descriptions), tick-contract-inline T6 (a green reachability check is not evidence of obedience; nothing observes agent chat output)]
  sources: ["terminal-phase-gate (cell tpg-2, 2026-07-13)", "docs/specs/doctrine-layer.md#B5", "docs/specs/doctrine-layer.md#R6", "docs/specs/doctrine-layer.md#R7", "docs/specs/doctrine-layer.md#R8", "docs/specs/doctrine-layer.md#E3", "tick-contract-inline (cells tci-1/tci-2/tci-3, decisions T1-T7, traces .bee/cells/tci-{1,2,3}.json, reports docs/history/tick-contract-inline/reports/, 2026-07-29)"]
  authoritative_for: "doctrine-layer: unenforced obedience and the human boundary"
---

# Doctrine Layer — Unenforced Obedience and the Human Boundary

Doctrine has no runtime: nothing *executes* these rules, so nothing fails loudly
when one is broken. Each rule here is observed only by the human — obedience
where no automated guard covers the action, a layer that stays amendable when
every source gate is shut, and the two rules about who runs the machinery and
whose words the conversation is held in.

## Behaviors & Operations

**B5 — Doctrine binds the assistant even where no mechanism enforces it.**
Trigger: an action doctrine forbids, in a project or runtime where no automated
guard covers that action. What happens: the assistant obeys anyway. What each
actor observes: nothing — which is the point. A guard's silence is not an
approval, and a gap in a guard is not a gap in the rules; treating the guard as
the authority makes the guard's coverage the real protocol and quietly deletes
every rule it fails to cover (decision c2c46488).

## Business Rules

- **R6** — An unblocked action is not an approved action. Automated guards catch
  what the assistant forgets; their silence grants nothing (c2c46488).
- **R7** — The workflow's own machinery is run by the assistant, never handed to
  the human. The human's only actions are approvals, decisions, and permissions.
- **R8** — The workflow's internal vocabulary stays out of the conversation. The
  human hears the work in their own terms; the machinery runs silently (1689af1b).
  This carries a positive duty — purpose-first narration (4439bd7e, work-visibility
  D1/D2): every perceivable work unit (a phase of real work starting, a worker sent
  out, a long-running step, a change of direction) opens with one work-language
  sentence naming what is being done and for what outcome, and every dispatched
  worker's description is one work-language intent sentence plus the model name —
  never a model name or codename alone. Twin litmus pair: strip the bee terms — if
  nothing is lost, they didn't belong; strip the message entirely — if the human
  loses the thread of what/why, the sentence was owed. Silence about mechanics is
  never silence about purpose.

## Edge Cases Settled

- **Doctrine is not gated.** Amending the standing sheet is knowledge work, and
  knowledge locations stay writable in every phase, including the terminal ones
  where source edits are shut (hook-runtime B12).

- **A green check is not evidence of obedience.** A standing check now proves
  that a rule which applies every turn can be reached from what is always
  loaded. Whether the assistant then followed it is observed nowhere: no guard
  reads the conversation and no suite asserts on it. This is R6 applied to
  checks rather than guards — the check's silence about obedience grants
  nothing, and citing a reachability result as coverage of obedience would
  quietly delete the very rule the check was built to protect
  (tick-contract-inline T6).
