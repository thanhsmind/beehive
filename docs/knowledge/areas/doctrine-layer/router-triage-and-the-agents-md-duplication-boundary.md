---
type: bee.area
title: Doctrine Layer — router triage and the duplication boundary
description: "How the router lets an obviously-small request pick its lane before a second instruction set loads, why uncertainty resolves toward loading more, and the rule deciding what the router may drop because the always-loaded operating block already carries it."
tags: [doctrine-layer, routing, context-budget]
timestamp: 2026-07-23
bee:
  id: doctrine-layer-router-triage-and-duplication-boundary
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md, areas/verify-pipeline/skill-reference-pointer-integrity.md]
  decisions: [router-cost D5, router-cost D6, router-cost D7, "pstack-gaps D4/D5 (docs/history/pstack-gaps/CONTEXT.md, 2026-09-01 — every rule id gains a spoken line here and a spoken rule obliges the reply to name what it changed)", "pstack-gaps D6 (docs/history/pstack-gaps/CONTEXT.md, 2026-09-01 — tests/rule_index_parity.rs pins the markers, the index rows and the spoken lines against each other)"]
  sources: [docs/history/router-cost/CONTEXT.md, "docs/history/router-cost/ (cells rc-3, rc-4, capped)"]
  authoritative_for: "doctrine-layer: router triage and the duplication boundary"
---

## Purpose

The router is the instruction set an assistant loads first, on every cold entry, before it is allowed
to decide anything. Its size is therefore a tax on every piece of work, including the smallest.

Two separate costs were conflated before this concept existed, and keeping them apart is most of what
it teaches:

1. **What is loaded before the lane is known.** The lane decision happened only after both the router
   and the planning instruction set were in context — so a one-line fix paid a full-project entry fee.
2. **What the router says that the reader already has.** The operating block is loaded into every
   session automatically, always. Anything the router restates from it is paid twice on every cold
   entry.

## Entry Points & Triggers

- **Triage** applies the moment a request arrives and the router is loaded — before any second
  instruction set is opened.
- **The duplication boundary** applies whenever the router is edited: it decides what may live there
  and what must be a pointer.

## Data Dictionary

| Element | Meaning |
|---|---|
| **the operating block** | The instruction document loaded automatically into every session. Its presence is guaranteed; nothing needs to load it. |
| **the router** | The first instruction set loaded on entry. Chooses the lane, the gates, and what to load next. |
| **lane** | The ceremony level a piece of work carries. Decided by two counts: risk flags tripped, and product files touched. |
| **the second load** | The planning instruction set — roughly 21 KB — opened only when the lane genuinely needs shaping. |
| **hard-gate flag** | A risk flag that forces the highest lane regardless of size: authentication, authorization, data loss, audit or security, an external provider, or the removal of existing validation. |
| **pinned wording** | Wording in the router that automated checks require verbatim, in some cases identically across several documents. Ten such strings exist. |

## Behaviors & Operations

**Triaging from the request alone.** The router opens with a compact triage block. A reader counts
risk flags and product files, and lands in one of four rows. Knowledge-only changes, and work with
0–1 flags within the small file caps, route straight to the merged shape-and-execution gate and the
single dispatched worker — **without opening the second load**. Everything else falls through to the
full chain, which does open it.

**Resolving uncertainty downward.** A reader who cannot tell which row they are in takes the *fuller*
path. The block states this explicitly and closes the two evasions a reader under pressure reaches
for: one hard-gate flag is the highest lane even at a single file, and re-counting flags to land
under a threshold is itself the signal that the higher lane already applies.

**Stating its own limit.** The triage block says, in its own text, that it saves nothing on the
router itself — instruction sets load whole, so the router is already fully in context by the time
the block is read. The only saving available is the second load. Writing this down is deliberate: a
later reader cannot mistake the block for permission to stop reading the router.

**Deferring what the operating block already carries.** Where the operating block genuinely states a
rule, the router keeps a one-line statement of the rule and points at where it lives in full. The
rule never disappears — only its elaboration moves.

## Actors & Access

| Actor | Observes |
|---|---|
| an assistant with an obviously-small request | a lane decision and an immediate route, with no second instruction set opened |
| an assistant with an ambiguous request | a rule sending it to the fuller path, not a judgment call |
| an assistant needing a deferred rule's detail | a one-line statement plus a pointer that resolves |
| an author editing the router | a fixed set of pinned wording that must survive verbatim |

## Business Rules

- **R1.** Triage decides the lane from the request alone, before a second instruction set loads
  (router-cost D7).
- **R2.** **Uncertainty resolves downward, into loading more — never upward into skipping.** Triage
  is an early exit for the obviously-small and is never a licence to shortcut.
- **R3.** One hard-gate flag forces the highest lane at any file count.
- **R4.** The router may drop only what the operating block genuinely carries — verified against the
  real document, never assumed.
- **R5.** **Every cut leaves a one-line pointer naming the rule and its home.** A silent deletion is a
  regression even when the rule survives elsewhere, because nothing tells the reader to go look.
  Section headings survive cuts for the same reason: a reader scanning headings must still learn the
  rule exists.
- **R6.** **A pointer chain must terminate.** Where the operating block itself defers *back* to the
  router for a rule's full text, the router may not answer by pointing at the operating block — that
  builds a loop in which the full rule lives nowhere. Such rules move to a reference document
  instead.
- **R7.** Pinned wording survives character-for-character. Some pins are cross-document consistency
  pins: rewording one requires rewording all its siblings, which is out of scope for a size
  reduction.
- **R8.** Prose may not be moved into a reference until the pointer-integrity gate exists to check
  the result (router-cost D5). See `verify-pipeline/skill-reference-pointer-integrity.md`.
- **R9.** **Enforced-rule signposting.** A rule whose prohibition a hook deterministically enforces —
  the deny naming its remedy — may shrink in always-loaded prose to a one-line signpost naming the
  sanctioned path. Semantic every-turn rules (where no enforcement is possible), and any sentence
  that is the canonical home other documents cite, keep their full text (block-lean L1a-L1d,
  `docs/history/block-lean/CONTEXT.md`).
- **R10.** **Every rule id is speakable, and a spoken rule owes an answer** (pstack-gaps D4, D5, D6;
  cells pg-2, pg-3, 2026-09-01). The § AGENTS.md rule homes section below is the rule index, and each row now
  carries an indented `spoken:` line — the sentence a person would actually say. A user may invoke a
  rule mid-run by name ("apply never-build-on-red"); the agent resolves the id here, applies the rule
  to the work in hand, and names in the reply the decision or step it changed — or says plainly that
  it changed nothing. A bare acknowledgement is not an answer. The leverage is the obligation on the
  reply, not the list: without it an invoked id is a no-op an agent can acknowledge and ignore, and
  without the changed-nothing branch the law invites a fabricated change. No second index and no CLI
  verb were added — this registry already existed and `bee knowledge check` already validated it;
  what was missing was one sentence per rule. The spoken line MUST stay indented: `parse_agents_rule_homes`
  reads any line opening with a dash as a new rule row, so a sibling bullet would report twenty rules
  where there are ten. `tests/rule_index_parity.rs` pins the three copies — the markers in `AGENTS.md`,
  the markers in `packages/bee/AGENTS.block.md`, and the rows here — plus the spoken line's presence.

## Edge Cases Settled

- **Three rules read as duplicated but were not.** A scratch-queue working file, a session-end
  review nudge, and four router-specific red flags each appeared to be restatements and each proved
  absent from the operating block on inspection. All were kept. The general lesson: the duplication
  list is a hypothesis, and checking it against the real document is what decides.
- **Two rules could not point at the operating block**, because the operating block already points
  back at the router for their full text (R6). Both moved to a reference document so the chain ends
  somewhere.
- **Restating a pinned string in new text is safe** where the check is a presence test — an extra
  occurrence does not break it. Rewording an existing occurrence does.
- **2026-07-28 — the boundary was applied in the opposite direction, to the operating block itself
  (agents-block-diet), and R6 turned out to be the binding constraint.** The block fell 16,152 →
  12,573 bytes (−22.2%) with every rule intact. Three findings generalise beyond that one edit:
  - **The router's earlier cut made the operating block a terminal home.** Because the router now
    says its own rules 2-4 and 13 "appear in full" in the operating block, and points there for the
    guardrail rules, those rules could not be thinned by pointing outward — that is exactly the R6
    loop. **A document that has been cut toward becomes harder to cut, not easier**, and which
    direction the boundary last ran in has to be checked before any second pass.
  - **The cheapest safe cut is content the reader is handed anyway.** The largest single saving was
    startup steps restating what the session preamble prints unprompted (2,919 → 1,554 bytes), plus
    a step that described itself as optional. Duplication against a *generated* surface is invisible
    to a duplication list built by comparing documents.
  - **A byte budget alone is an unsafe instrument, and this closes the open gap above.** A budget
    rewards cutting and cannot distinguish restated elaboration from a deleted rule. The fence was
    ratcheted onto the achieved size (20,480/18,000 → 15,000/14,000) only together with a structural
    guard: the numbered-rule roster pinned on both the template and the render, a negative control
    proving it bites, and a check refusing any terminal-home rule compressed into a bare
    cross-reference. **The observation outlived the instrument.** The fence was deleted outright in
    2026-07-29 (budget-fence-removal D1/D2) once it became clear a budget alone was not merely
    insufficient but actively harmful — it made an author fund a correct addition by cutting correct
    text. The structural guards named here survive and still block; only the size half is gone.
- **2026-07-26 — the operating block's `## Critical rules` shrank 16 → 14, in judgement form
  (judgement-rules D1-D4)**, without touching this concept's own pinned wording: the fan-out rule
  (old 13, new 12) dropped its `>3 files` numeric proxy in favor of "digest, not verbatim" (D2), and
  the `## Red flags — stop and re-route` section was deleted from the standing sheet (D4) — the
  router's own four router-only red flags are unaffected. `packages/bee/tests/test_misc.mjs`'s
  mutation harness moved in lockstep: `assertFanOutAnchors` now checks the compressed rule-12 anchor
  set, and the reshaped rule-15-pointer census (the unnumbered line under rule 14) keeps running
  `assertOrderedWaitContract` against `routing-and-contracts.md` plus the `bee-swarming` copies.

- **2026-08-04 — the boundary gained a third axis (block-lean): enforced-rule signposting (R9).**
  The boundary's first axis was router-vs-operating-block (this document's origin: what the router
  may drop because the block carries it); prompt-diet added one-rule-one-home across the skills
  (`doctrine-layer/prompt-writing-standard.md`); block-lean adds *enforcement* as a reason a rule
  may leave always-loaded prose. Where a hook deterministically denies the prohibited action and
  the deny message itself names the remedy, the operating block keeps only a one-line signpost to
  the sanctioned path — the tier-transport, guardrail, and reservation-etiquette paragraphs took
  this shape, and the block fell 184 → 174 lines (~8.6 KB), a saving paid back on every session of
  every governed project. The classification was possible only because guard-hardening's audit
  produced the enforcement map — which rule has a deterministic backstop and which is semantic-only
  (`doctrine-layer/unenforced-obedience.md`); full text stays for semantic every-turn rules (L1b)
  and for every canonical-home sentence other documents cite (L1c), per block-lean L1a-L1d
  (`docs/history/block-lean/CONTEXT.md`). One honest caveat: the two wording guards this document
  names as enforcing every-turn reachability and pinned strings (`packages/bee/tests/test_misc.mjs`,
  `scripts/tests/test_gate_bypass_doctrine.mjs`, listed under Pointers) have been dead since the R6
  cutover — both trees were deleted and the instruction-layer suites were never re-pointed
  (`plans/cutover-readiness.md` records the gap) — so the L1d not-every-turn classification was
  applied manually and recorded in the block-lean CONTEXT rather than checked by automation.

## AGENTS.md rule homes

Discipline rules homed in `AGENTS.md` have no YAML frontmatter. Their `applied_at` records are tracked here:

- `agents-proof-at-cap` (AGENTS.md § Prove, then say so):
  spoken: run the proof your change type needs, then record it on the cap as command, result, and why that scope
  - applied_at:
    - `docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md`
    - `docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md`
    - `skills/bee-hive/references/gates-and-delegation.md`
    - `skills/bee-planning/SKILL.md`
    - `skills/bee-planning/references/planning-reference.md`
    - `skills/bee-shaping/references/mini-brief-template.md`
    - `skills/bee-shaping/references/implement-plan-template.md`
    - `skills/bee-swarming/references/swarming-reference.md`
- `agents-never-build-on-red` (AGENTS.md § Prove, then say so):
  spoken: never build on a red base — the red is the work now: fix it first, then carry on
  - applied_at:
    - `docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md`
    - `docs/knowledge/areas/doctrine-layer/unenforced-obedience.md`
    - `skills/bee-hive/SKILL.md`
    - `skills/bee-hive/references/gates-and-delegation.md`
    - `skills/bee-swarming/SKILL.md`
    - `skills/bee-swarming/references/swarming-reference.md`
- `agents-never-zero-execution-workers` (AGENTS.md § Work in parallel, coordinate through the store):
  spoken: from a small lane up, hand every cell to a dispatched worker — only a tiny cell may run inline
  - applied_at:
    - `docs/knowledge/areas/doctrine-layer/helper-classes-and-transports.md`
    - `skills/bee-hive/references/gates-and-delegation.md`
- `agents-capture-line-at-close` (AGENTS.md § Capture what settles):
  spoken: write it down the moment it settles, and close every task with a capture line or a plain 'nothing settled'
  - applied_at:
    - `skills/bee-capturing/SKILL.md`
    - `skills/bee-hive/SKILL.md`
    - `skills/bee-hive/references/routing-and-contracts.md`
- `agents-context-handoff-65` (AGENTS.md § Care for the session):
  spoken: at about 65% context, write the handoff and stop cleanly — this holds mid-wave too
  - applied_at:
    - `docs/knowledge/areas/onboarding/status-display-vendoring.md`
    - `skills/bee-hive/SKILL.md`
    - `skills/bee-hive/references/go-mode.md`
    - `skills/bee-hive/references/routing-and-contracts.md`
    - `skills/bee-swarming/SKILL.md`
    - `skills/bee-swarming/references/swarming-reference.md`
    - `skills/bee-swarming/references/worker-details.md`
- `agents-gates-never-self-approved` (AGENTS.md § Bee workflow):
  spoken: never approve your own gate — gates, decision answers and privacy calls belong to the user
  - applied_at:
    - `skills/bee-herding/README.md`
    - `skills/bee-herding/references/role-dispatch.md`
    - `skills/bee-hive/SKILL.md`
    - `skills/bee-hive/references/gates-and-delegation.md`
- `agents-one-commit-per-cell` (AGENTS.md § Care for the session):
  spoken: one commit per cell — imperative subject, and the cell id on the last line of the body
  - applied_at:
    - `skills/bee-swarming/references/worker-details.md`
- `agents-review-user-invoked` (AGENTS.md § Bee workflow):
  spoken: ask for a review when you want one — it never runs by itself as a step of the flow
  - applied_at:
    - `docs/knowledge/areas/workflow-state/review-sessions.md`
    - `skills/bee-hive/SKILL.md`
    - `skills/bee-hive/references/gates-and-delegation.md`
    - `skills/bee-hive/references/go-mode.md`
    - `skills/bee-hive/references/routing-and-contracts.md`
- `agents-one-next-action` (AGENTS.md § Communication):
  spoken: close on exactly one next action — your own next move or the one call only the user can make
  - applied_at:
    - `docs/knowledge/areas/doctrine-layer/the-communication-contract.md`
    - `skills/bee-hive/references/routing-and-contracts.md`
- `agents-worktree-first` (AGENTS.md § Bee workflow):
  spoken: start code-touching feature work in its own worktree; main takes integration, release and a solo tiny fix
  - applied_at:
    - `docs/knowledge/areas/workflow-state/worktree-isolation.md`
    - `docs/knowledge/areas/worktree-parallelism/routing-and-visibility.md`
    - `skills/bee-hive/SKILL.md`
    - `skills/bee-hive/references/routing-and-contracts.md`
    - `skills/bee-hive/references/scout-and-ticks.md`
    - `skills/bee-planning/SKILL.md`
    - `skills/bee-swarming/SKILL.md`

The list is kept by hand because `AGENTS.md` carries no frontmatter. A
`bee.applied_at` key on this concept would not serve: `applied_at_unlinked`
resolves a target against the rules this concept's own body homes, and these
ten are homed in `AGENTS.md`, not here.

## Open Gaps

- **The realised saving is smaller than first projected.** The router fell 18.7%, about 1,675 tokens
  per cold entry, against an early estimate near 5,000. The estimate counted duplication that
  inspection showed was either genuinely needed or pinned. Going further would mean cutting
  route-critical prose or touching pinned wording.
- **No check measures the *router's* size.** Nothing prevents it from growing back. The operating
  block's own version of this gap closed on 2026-07-28 (see Edge Cases Settled): a size budget is a
  workable instrument, but only paired with a structural guard, because a budget on its own rewards
  deleting a rule exactly as much as deleting a restatement. Whether the router deserves the same
  pairing is still undecided; the threshold-widening worry is unchanged, and is why the budget
  constant carries its reasoning inline — widening it is meant to be a visible act.
- **Triage's effect is unmeasured.** No instrumentation records how often a session takes the early
  exit versus falling through, so the saving is modelled rather than observed.

## Pointers (implementation)

- `skills/bee-hive/SKILL.md` — the router; `## Triage first` sits before onboarding, and the
  pinned region is `## Modes and Lanes`.
- `AGENTS.md` — the operating block; its critical rules are what R4 is checked against.
- `skills/bee-hive/references/routing-and-contracts.md` — where deferred detail lands.
- `scripts/tests/test_gate_bypass_doctrine.mjs`, `packages/bee/tests/test_misc.mjs` — the
  suites enforcing the pinned wording.
- `scripts/tests/test_skill_pointers.mjs` — the gate required by R8.
