---
type: bee.area
title: Doctrine Layer — the communication contract
description: "The single ruleset for what execution reports to the person being served, and in what shape: five reader facts, the open/body/close turn shape, seven standing rules, when the rules deliberately break, and the pre-send acceptance test."
tags: [doctrine-layer, communication, voice]
timestamp: 2026-07-26
bee:
  id: doctrine-layer-communication-contract
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md]
  decisions: [ec9a60ae (comms-contract D1 — a single-home communication contract shaped as reader facts -> turn shape -> seven rules -> break conditions -> pre-send litmus), f6ff3bf5-df05-4af9-9d03-65fd9d0b4735 (auto-approved landing of the contract into routing-and-contracts.md)]
  sources: ["docs/history/comms-contract/ (tiny lane, decisions ec9a60ae + f6ff3bf5-df05-4af9-9d03-65fd9d0b4735, 2026-07-26)", "skills/bee-hive/references/routing-and-contracts.md '## Communication contract' (the governing text this concept describes)"]
  authoritative_for: "doctrine-layer: the communication contract"
---

# Doctrine Layer — the communication contract

## Purpose

The workflow's day-to-day work is invisible to the person it serves unless something is said
about it, in their own terms. Left unspecified, that reporting drifts toward mechanism — step
counts, internal identifiers, procedural narration — because that is what the acting side is
actually tracking while it works. This concept is the single ruleset for what crosses from
execution into the conversation, and in what shape, so voice is never re-invented per skill or
per session, and never drifts back toward mechanism.

## Entry Points & Triggers

- Every user-facing turn during any workflow-governed session — not only inside a named stage; a
  plain conversation turn is still bound by it.
- A gate, a decision point, or a privacy approval — the moments this contract calls out as
  deliberately unmistakable, so they are never mistaken for routine progress chatter.
- An author changing user-facing wording anywhere in the workflow — this is the one place that
  shape is decided; nowhere else legitimately re-derives it.

## Data Dictionary

| Element | Meaning |
|---|---|
| **Reader facts** | The five standing truths about the person on the other side of the conversation that every rule below is derived from: they supervise rather than execute; they drop in and out of long sessions and remember only the last message; they think in outcomes, not mechanism; their rare high-stakes moments must read as visibly different from routine progress; and they trust fresh evidence, never bare assurance. |
| **Turn shape** | The three-part shape of every user-facing turn: an opening state line naming what finished, what is running, and what remains; a body that is the work itself, kept short, with the full record linked rather than pasted in; and a close naming exactly one next action. |
| **The seven rules** | The standing checklist a turn is written against: purpose-first content, concrete estimates, a runnable win, cause-plus-fix-plus-actor on any error, one unmistakable question at a time, a tangent surviving as one closing line instead of a mid-task detour, and evidence beside every claim of completion. |
| **Break conditions** | The three situations where the contract deliberately trades brevity for depth: a destructive or irreversible action, an explicit request to explain, and genuine ambiguity — which still earns only one short question, never a guess. |
| **Pre-send litmus** | The check applied before any user-facing message is sent: the first and last line alone must answer what just happened and what happens next, and every internal term should be strippable without losing anything the reader needed. |

## Behaviors & Operations

**Composing a user-facing turn.** Trigger: any point in a workflow-governed session where
something is about to reach the person being served. The turn is built to the turn shape
(open/body/close) and checked against the seven rules before it is sent; a line that fails a rule
is rewritten or deleted, never softened into ambiguity. What the reader observes: a message that
opens by naming state, stays short, and closes with exactly one thing to do or decide next — never
a menu, never bare mechanism.

**Presenting a high-stakes moment.** Trigger: a gate, a decision, or a privacy approval. These are
visually and structurally set apart from ordinary progress narration, because the reader's
attention for these moments is rare and must not be spent skimming past routine updates. What the
reader observes: an unmistakable, restatable ask, never buried in a paragraph of progress text.

**Breaking the rules on purpose.** Trigger: a destructive/irreversible action, an explicit request
for depth, or genuine ambiguity. The turn trades its normal brevity for full clarity in exactly
these cases, while keeping the same open/close shape. What the reader observes: more words exactly
where more words are warranted, never elsewhere.

## Actors & Access

| Actor | Observes |
|---|---|
| The person being served | Every user-facing turn shaped by this contract: state-first opening, short body, one-action close, and unmistakable high-stakes moments. |
| The acting side (the assistant) | The single ruleset it composes every user-facing turn against — never a second, competing style guide. |
| An author changing user-facing wording | One place — this contract's home — that must be edited; nothing elsewhere legitimately re-derives the shape. |

## Business Rules

1. **One home.** The communication contract has exactly one governing document; it is never
   re-derived, restated as a rival ruleset, or forked per skill (decision ec9a60ae).
2. **Purpose-first, content-required.** Every perceivable unit of work opens by naming what is
   being done and for what outcome; a sentence naming neither is deleted rather than softened
   (ec9a60ae).
3. **Concrete estimates.** Anything expected to take over a minute is given a concrete unit of
   time, never a vague duration (ec9a60ae).
4. **A win is runnable.** A completion line names what now works and how to try it, before any
   narrative account of what changed (ec9a60ae).
5. **Errors carry cause, fix, and actor.** Every error names its cause, its fix, and who acts on
   it — the acting side, by default — quoting the shortest decisive evidence rather than a raw
   dump (ec9a60ae).
6. **Questions are scarce and unmistakable.** One question at a time, set apart from progress
   narration, phrased so the reader can restate what they are deciding in their own words
   (ec9a60ae).
7. **Tangents survive as one line, after the main thread closes.** A side issue is filed and
   mentioned once at the close, never expanded mid-task (ec9a60ae).
8. **Evidence accompanies every claim of completion.** "Done"/"fixed"/"green" appear only beside
   the fresh output that proves it, in the same message (ec9a60ae).
9. **High-stakes moments break the brevity rule on purpose.** A destructive action, an explicit
   request to explain, or genuine ambiguity earns full clarity instead of the usual short form —
   still with exactly one next action at the close (ec9a60ae).
10. **The pre-send litmus is the acceptance test.** A message whose first and last line fail to
    answer what happened and what happens next is rewritten before it is sent, and every internal
    term strippable without losing meaning is stripped (ec9a60ae).

## Edge Cases Settled

- The contract governs plain conversation turns too, not only turns inside a named workflow
  stage — the reader facts do not change just because no stage happens to be running.
- A high-stakes moment is never demoted to ordinary progress narration for the sake of brevity;
  break condition 9 exists precisely so brevity never wins there.

## Open Gaps

- No mechanical check enforces the seven rules or the pre-send litmus today; adherence is
  self-applied per turn, the same way the reader facts themselves are.

## Pointers (implementation)

- The literal, authoritative wording — reader facts, turn shape, the seven rules, break
  conditions, and the pre-send litmus — lives in
  `skills/bee-hive/references/routing-and-contracts.md`, `## Communication contract` section.
  This concept describes it; that section governs it.
- Landed via decision `ec9a60ae` (comms-contract D1) and auto-approved-merge decision
  `f6ff3bf5-df05-4af9-9d03-65fd9d0b4735`.
- The neighboring `## Gate Presentation Contract` section in the same file specializes turn shape
  for the four approval gates.
