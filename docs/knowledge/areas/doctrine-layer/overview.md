---
type: bee.area
title: "Doctrine Layer — purpose, vocabulary, and actors"
description: "Why an always-loaded rule layer exists, the vocabulary it is argued in, who is bound by it, and what it does not yet answer."
timestamp: 2026-07-21
bee:
  id: doctrine-layer-overview
  lifecycle: active
  areas: [doctrine-layer]
  decisions: ["lane-ceremony-v3 D1-D10 (docs/history/lane-ceremony-v3/CONTEXT.md, 2026-07-19)", ba5a35f1-981d-4cb5-8a57-234a187f122d (placement rule), c2c46488 (an unblocked write is not an approved write), 1689af1b (silent bookkeeping), D1/D2/D3 delegation contract, "0023 + 6cd34376 (explicit-tier transport rides critical rule 12, B3a)", codex-agent-wait-loop D1-D5 + ebb70b72-e5e5-43f2-a692-beb371b99f6c (native empty-wait discipline and live Codex surface), "040f8ef0 (read-only analyst spawn + partial-return fan-out, B7/R11)", "f21efe6e (tree-hygiene D1/D4 — one canonical scratch home, the write-guard that enforces it)"]
  sources: ["lane-ceremony-v3 cells lcv3-1..lcv3-4 (traces in .bee/cells/, reports docs/history/lane-ceremony-v3/reports/, 2026-07-19 — plan freeze, lane work-packet shapes, product-file caps, test-anchored flags, intake-first classification; each RED-first against the doctrine assertion suite)", "fanout-doctrine (cell fanout-doctrine-1, 2026-07-13, flushed capture stub 2f796f40)", "terminal-phase-gate (cell tpg-2, 2026-07-13)", "tier-transport-doctrine (cell tier-transport-doctrine-1, 2026-07-13)", "codex-agent-wait-loop (cells codex-agent-wait-loop-2/codex-agent-wait-loop-3, 2026-07-15/2026-07-19 — native wait rule plus independently reviewed D6/D7 repair)", "compounding-fanout-hardening (cell cfh-1, 2026-07-17, flushed capture stub d3417cb2)", "advisor-and-orchestration Slice 2A-ii (cells ao-2aii-1/ao-2aii-2, 2026-07-17)", "advisor-and-orchestration Slice 2A-iii (cells ao-2aiii-1/ao-2aiii-2 — dispatch-boundary enforcement + gather-purpose routing prose, 2026-07-17)", "advisor-and-orchestration Slice 5 (cell ao-5-1 — execution-worker class, tiny/small single-worker execution, AO14, 2026-07-17)", "tree-hygiene (cell th-6, 2026-07-21 — write-guard scratch-shape denial + the three competing prose homes collapsed into one doctrine rule)"]
  authoritative_for: "doctrine-layer: purpose, vocabulary, and actors"
  owns.code: [packages/bee-rs/crates/bee/src/devtools/prompts.rs, packages/bee-rs/crates/bee/src/devtools/skill_trees.rs, packages/bee-rs/crates/bee/src/hooks/prompt_context.rs]
  owns.skills: ["skills/bee-hive/*", "skills/bee-writing-skills/*", "skills/bee-researching/*"]
  owns.tests: [packages/bee-rs/crates/bee/tests/instruction_laws.rs]
---

# Doctrine Layer (the standing instructions an assistant always carries)

## Purpose

The workflow governs an assistant that does not remember. Every rule the
assistant must obey has to reach it *inside the turn it is needed*, and the
assistant's attention is loaded from two very different kinds of source:

- a **standing instruction sheet**, read at the start of every session and again
  after every memory compaction — always present, in every turn, whether or not
  any part of the workflow was invoked; and
- **procedure references**, opened only when a specific stage of the workflow is
  invoked and needs them.

The doctrine layer is the first of those two. It is the set of rules that hold
*unconditionally* — including in ordinary conversation turns, where the
assistant is only talking to the human and no stage of the workflow is running.
This area owns what belongs in that layer, how it reaches every project, and how
it is kept from silently emptying out.

## How this area is split

The split follows what each rule *governs*, not the section it was written in:

- What belongs on the standing sheet, how it reaches every project, and how a
  rule is kept from disappearing: `placement-and-anchoring.md`.
- What doctrine demands where no guard, gate, or stage enforces it — including
  the two rules the human experiences directly: `unenforced-obedience.md`.
- When mechanical gathering delegates and what never does:
  `delegation-threshold.md`.
- The two helper classes, their capability surfaces, and the external-command
  transport: `helper-classes-and-transports.md`.
- The native Codex empty-wait discipline: `native-wait-discipline.md`.
- Lane classification, work-packet shapes, product-file caps, the canonical
  scratch home and the verify ladder: `lane-and-working-discipline.md`.
- What the acting side reports to the person being served, and in what shape:
  `the-communication-contract.md`.

## Entry Points & Triggers

| Trigger | What happens |
|---|---|
| A session begins | The assistant reads the standing instruction sheet before doing anything else. |
| Memory is compacted mid-session | The assistant re-reads it — compaction destroys everything the sheet is not part of. |
| A project is onboarded or upgraded | The current doctrine is written into that project's own instruction sheet, replacing the previous copy in place. |
| A workflow stage is invoked | Procedure references for that stage load *on top of* doctrine; they never substitute for it. |
| The suite runs | Each doctrine rule that must never disappear is checked for by name; a missing rule fails the suite. |

## Data Dictionary

| Element | Meaning |
|---|---|
| **Standing instruction sheet** | The always-loaded rule document that lives in each governed project and is read every session. Carries: startup steps, the stage chain and its approval gates, the numbered critical rules, the working-file inventory, the guardrail rules, and the session-finish checklist. It no longer carries a red-flag list: that section was deleted from the standing sheet (judgement-rules D4); the surviving router-only red flags live in `skills/bee-hive/SKILL.md`. |
| **Critical rule** | A numbered, unconditional instruction on the standing sheet. Holds in every phase, every lane, and every turn — including turns where no workflow stage is running. |
| **Procedure reference** | A detailed document belonging to one workflow stage, loaded only when that stage is invoked. Correct home for *how* a stage does its job; wrong home for anything that must hold when that stage is not running. |
| **Anchor** | A distinctive phrase from a doctrine rule that the suite asserts is still present on the standing sheet. An anchor is how a rule is prevented from quietly drifting back down into a procedure reference. |
| **Always-applies rule** | A rule whose answer to *"must this hold when no workflow stage is running?"* is yes. Always-applies rules belong on the standing sheet, without exception. |
| **Decide-altitude** | Work that requires judgment the orchestrating model must not hand off: approval gates, the mode decision, synthesis of findings, accepting or rejecting a helper's result, durable state writes, and conversation with the human. |
| **Gather-altitude** | Mechanical work whose output the orchestrator needs only as a summary: file hunts, codebase scans, multi-file inventories, routine rendering. |
| **Empty wait** | A native Codex `wait_agent` call that returns timeout or no completed agent. It reports only that no completion arrived in that interval; it is not a worker failure or ownership signal. |
| **Progress interval** | Before another bounded native wait: perform at least one material task-local action if work remains (without needing to exhaust it), otherwise take exactly one `list_agents` snapshot; handle any completion exactly once, recompute relevant liveness, then send concise commentary naming both the live agent state and the next action. Zero live agents ends collection. |

## Actors & Access

- **The orchestrating assistant** — reads the standing sheet every session and
  after every compaction; bound by every rule on it in every turn; owns all
  decide-altitude work.
- **A delegated helper** — receives one bounded mechanical task and returns a
  summary. Never receives decide-altitude work, and never inherits the
  orchestrator's conversation. When its task is purely to read and analyze, it
  is spawned read-only (B7).
- **The human owner** — authors and amends doctrine; approves the gates doctrine
  reserves for them. Never asked to run the workflow's own machinery.
- **A governed project** — receives doctrine by copy at onboarding; may carry its
  own instructions outside the doctrine block, which are never overwritten.

## Open Gaps

- No stated ceiling on doctrine length. The standing sheet is loaded in full,
  every session, in every governed project — its cost is paid on every turn and
  nothing currently bounds its growth, nor defines what would justify retiring a
  rule to make room.
- The three-source delegation threshold is asserted, not measured. No evidence
  records where the real break-even sits, or whether it differs by how large the
  sources are.
- No mechanism detects the B2 failure in the other direction: a rule correctly
  placed on the standing sheet that *should* have been a procedure reference
  (bloat), as opposed to one wrongly buried in a reference (silence). Only the
  silence direction is guarded.

## Concepts in this area

- [Doctrine Layer — agent-facing state query CLI surface](agent-facing-state-query-cli-surface.md) — The three read-only query verbs bee exposes over its own state store, so agents stop grepping .bee/*.jsonl or importing internals — decisions active/search --cell/--feature, backlog findings --feature, and state scribing-run --show — plus the word-boundary discipline that makes the text-matching verbs correct.
- [Doctrine Layer — model roles, fall-through, and escalation](model-roles-and-escalation.md) — How work says which model should run it: an open set of job-named roles resolved through one parser, an ordered fall-through that warns instead of failing, cost held apart as an explicit escalation flag, and an explicit-only retry chain that never fires on a semantic failure.
- [Doctrine Layer — native Codex wait discipline](native-wait-discipline.md) — What an empty wait means, the mandatory progress interval before another bounded wait, and why a timeout never changes worker or ownership state.
- [Doctrine Layer — the prompt-writing standard](prompt-writing-standard.md) — The standard every edit to bee's instruction text is judged by: the four-question line filter, add-on-failure, one rule one home, verifiable-imperative style, the deterministic-backstop preference, and the standing record that no size ceiling exists or may be introduced.
- [Doctrine Layer — report shape and counted numbers](report-shape-and-counted-numbers.md) — The shape a report-shaped skill's output takes: one required closing count line with its empty case written verbatim, a one-line stamp over a closed tag vocabulary for countable findings, a Boundaries block that routes every concern it refuses, and the rule that a skill printing numbers must name the figure it may never invent.

## Patterns in this area

- [A hit list is only as complete as the search that produced it](../../patterns/20260819-a-hit-list-is-only-as-complete-as-the-search-that-produced-it.md) — A cell scoped by a guessed grep inherits the guess: naming the list complete makes the worker stop looking, so the worker is right by its instructions while the result is wrong. Measured on herding-orchestration cell ho-14: a four-file grep left nine live references standing — seven in the very document that describes the permission posture and the runtime adapter seam the feature's riskiest decision protects — while the full-tree sweep in ho-15 found 43 files, most of them mirrors and several of them history that must not be rewritten.
- [A two-sided parity fence is blind to a change made on both sides](../../patterns/20260902-a-two-sided-parity-fence-is-blind-to-a-change-made-on-both-sides.md) — A test that pins file A to file B proves they agree, never that either says anything. Delete the same line from both and the fence stays green while the rule it guarded is gone.
