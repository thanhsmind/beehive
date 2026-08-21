---
type: bee.area
title: Doctrine Layer — unenforced obedience and the human boundary
description: "The rules with no runtime behind them: obey where no guard covers the action, amend doctrine even when the phase gate is shut, run the machinery yourself, and keep its vocabulary out of the conversation."
timestamp: 2026-08-04
bee:
  id: doctrine-layer-unenforced-obedience
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md]
  decisions: [c2c46488 (an unblocked write is not an approved write), 1689af1b (silent bookkeeping), 4439bd7e (purpose-first narration + intent-carrying dispatch descriptions), tick-contract-inline T6 (a green reachability check is not evidence of obedience; nothing observes agent chat output), "guard-hardening E5 + E1/E2/E3 (docs/history/guard-hardening/CONTEXT.md, 2026-08-04 — the markdown-only set is recorded by necessity; three formerly prose-only rules moved to enforcement)"]
  sources: ["terminal-phase-gate (cell tpg-2, 2026-07-13)", "docs/specs/doctrine-layer.md#B5", "docs/specs/doctrine-layer.md#R6", "docs/specs/doctrine-layer.md#R7", "docs/specs/doctrine-layer.md#R8", "docs/specs/doctrine-layer.md#E3", "tick-contract-inline (cells tci-1/tci-2/tci-3, decisions T1-T7, traces .bee/cells/tci-{1,2,3}.json, reports docs/history/tick-contract-inline/reports/, 2026-07-29)", "guard-hardening (cell gh-3, docs/history/guard-hardening/CONTEXT.md, 2026-08-04)"]
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

**B5a — What stays markdown-only stays by necessity, and the record names the
necessity.** The unenforced layer is not a backlog of guards nobody built; a
rule stays prose-only exactly when no mechanism *can* observe the action it
governs (guard-hardening E5). The recorded set:

- *Gate self-approval.* Gates are approved by the user, never by the assistant —
  but actor identity is unknowable to the CLI: the same process writes the same
  approval whether a human said yes or the assistant invented the yes. No hook
  can tell them apart, so the rule binds only as B5 obedience.
- *Independent review is never an automatic stage.* "The user asked for a
  review" is a fact about the conversation, and nothing mechanical reads the
  conversation (the same blindness T6 records for obedience itself).
- *Cross-session work is claimed only via `bee cells claim-next`.* At the file
  level a browsed-and-taken cell is indistinguishable from a handed one; the
  difference is intent, which lives where no guard looks.

Not in the set, though for a narrower reason than it once had: *never build
on a red base* keeps no prose home because the cap door refuses a proof line
whose recorded result is red. That is a check on what the worker REPORTS, not
a test run — the door runs nothing itself, and the worker picks and runs the
proof. So the enforcement is real but partial: a worker that runs no proof at
all is caught, and a worker that misreports one is not. The remaining gap is
covered by evidence discipline, not by a guard. And the set shrinks when a mechanism becomes possible: in the same
feature, three formerly markdown-only rules moved to enforcement — the
containment deny's harness-owned allowlist judges the resolved write target
(guard-hardening E1), hand-edits to the CLI-owned stores `.bee/cells/*.json`,
`.bee/lanes/*.json`, and `.bee/onboarding.json` are refused with the owning
verb named (E2), and `grep`/`find` invocations are denied by this repo's
`.claude/settings.json` permissions rather than by CLAUDE.md prose alone (E3).
Each entry above carries its necessity so that the next audit can re-ask the
question instead of re-litigating it: a rule still here without a necessity is
a defect, not a tradition.

## Business Rules

- **R6** — An unblocked action is not an approved action. Automated guards catch
  what the assistant forgets; their silence grants nothing (c2c46488).
- **R7** — The workflow's own machinery is run by the assistant, never handed to
  the human. The human's only actions are approvals, decisions, and permissions.
- **R8** — The workflow's internal vocabulary stays out of the conversation. The
  human hears the work in their own terms (1689af1b, narrowed): this constrains the
  WORDS, not whether a step is mentioned — every perceivable step is still ticked,
  in work language. The earlier "run it, never narrate it" reading of 1689af1b is
  retired; it contradicted the tick contract, which is the rule that stands.
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
