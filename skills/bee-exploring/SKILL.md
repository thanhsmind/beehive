---
name: bee-exploring
description: >-
  Turn a fuzzy feature request into locked decisions in docs/history/<feature>/CONTEXT.md. Use when a request has gray areas or unstated product decisions that would make planning guess. Not for implementation research, cell creation, or code.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies: []
---

# exploring

If `.bee/onboarding.json` is missing or stale, stop and invoke `bee-hive`.

Turns fuzzy intent into locked decisions in `docs/history/<feature>/CONTEXT.md`.
Scout bees find the flowers; they do not build the comb.

## Hard Gates

- Batch independent questions into one message; serialize only dependent
  ones. Mechanics, `AskUserQuestion` schema, ordering:
  `references/exploring-reference.md` ("Batching mechanics").
- Do not answer your own question — even when sure of the answer.
- No implementation research, architecture proposals, cell creation, or
  code — except a throwaway SEE mock under
  `.bee/spikes/<feature>/mocks/`:
  `references/exploring-reference.md` ("SEE mock").
- Do not invoke planning yourself. End by handing the user to `bee-planning`.
- Gather-altitude steps (scope reads, gray-area scout digest) delegate as
  I/O workers per the Delegation contract
  (`bee-hive/references/routing-and-contracts.md` ("Delegation contract"));
  ad-hoc dispatch beyond the fresh-eyes reviewer's named slot defaults to
  generation; ceiling needs a `[bee-tier: ceiling]` marker plus a one-line
  justification.

## Flow

0. **Enter the feature atomically (from `idle`).** ONE call —
   `node .bee/bin/bee.mjs state start-feature --feature "<slug>" --mode "<mode>"`
   — sets `idle → exploring` + feature + mode + resets all gates. Do this
   FIRST; never hand-write `state set --owner exploring --phase exploring`
   from `idle` (refused — `--owner` must match the pre-mutation phase).
   Feature already active → skip, you are resuming.

1. **Scope**
   - Classify `Quick` / `Standard` / `Deep`.
   - Read the critical patterns — bundle: `docs/knowledge/index.md`'s
     `## Critical patterns`; no bundle: `docs/history/learnings/critical-patterns.md`
     — and `.bee/state.json` if present.
   - Spans independent subsystems → pick one, defer the rest.
   - Backlog flip, brief check, command detection:
     `references/exploring-reference.md` ("Backlog flip", "Brief check",
     "Command detection").

2. **Domain** — classify each applicable type: `SEE` (user-visible surface)
   · `CALL` (API/CLI/webhook/SDK/service) · `RUN` (job/script/pipeline) ·
   `READ` (docs/emails/reports) · `ORGANIZE` (data model/layout/taxonomy).
   Load `references/gray-area-probes.md`, pick only relevant probes.

3. **Re-lane checkpoint** — once the scout's touch set is counted: measured
   evidence may demote `standard` → `small` once (files within threshold,
   zero hard-gate flags, zero open gray areas — all three). Never `tiny`,
   never twice. Log it, tick it. Rule:
   `bee-hive/references/routing-and-contracts.md` ("Re-lane checkpoint").

4. **Gray Areas**
   - Generate 2-4 unstated *product* decisions planning would otherwise
     guess. Exclude implementation choices, performance tuning, new scope —
     a candidate that matters only to the implementer belongs to planning.
   - Brief loaded (step 1) → draw candidates only from what it still marks
     unclear, skip the quick scout.
   - Else, quick scout only — one keyword pass, then read 2-3 relevant
     files: `rg "<feature-keyword>" src app packages --glob "*.{ts,tsx,js,jsx,py,md}" | head -20`.
     Cite the patterns found in your questions.
   - Pre-classify each candidate independent/dependent with its dependency
     edge — input to step 5's batching:
     `references/exploring-reference.md` ("Batching mechanics").

5. **Socratic Locking**
   - Ask in the fewest rounds the dependencies allow: independent survivors
     of a phase batch into ONE `AskUserQuestion` (up to 4); dependents ask
     alone, after the answer they hinge on lands. Broad (independent) first,
     then the dependents it gates. Each question stays concise,
     single-choice where possible, outcome-framed, CONTEXT / QUESTION /
     RECOMMENDATION / options format.
   - Materiality test; under `gate_bypass_level` `full`/`total`, apply the
     info-vs-approval litmus — *confident best answer already?* Yes → lock
     it as a decision, don't ask. No → genuine information, still ask, even
     under `total`. Blindspot pass, SEE mock:
     `references/exploring-reference.md` ("Materiality test",
     "Gate-bypass refinement", "Blindspot pass", "SEE mock").
   - After each answer, confirm it back and assign a stable ID: `D1`, `D2`…
   - An answer that settles a fuzzy domain word gets pinned like a
     decision: `references/exploring-reference.md` ("Pinned terms").
   - One answer, several decisions: lock the one asked about, echo the
     others as candidates to confirm one at a time.
   - Scope creep: mark deferred, return to the current question.

6. **Context Assembly**
   - Write `docs/history/<feature-slug>/CONTEXT.md` from
     `references/context-template.md`: boundary, domain types, locked
     decisions table with D-IDs, pinned terms, scout paths, canonical
     references, open questions, deferred ideas. Concrete language only —
     no placeholders, TODOs, or vague preferences.
   - Deferred Ideas also feed the product backlog:
     `references/exploring-reference.md` ("Deferred ideas backlog").
   - Fresh-eyes review (blocks only Gate 1, not the conversation):
     `references/exploring-reference.md` ("Fresh-eyes review").

7. **State And Handoff**
   - `node .bee/bin/bee.mjs state set --owner exploring --phase exploring --feature "<feature>" --summary "Exploring complete. CONTEXT.md is ready for planning." --next-action "Gate 1, then invoke bee-planning."`
   - Gate-bypass check FIRST — read the active `gate_bypass_level`; `full`/
     `total` lift the high-risk floor for **every** lane (already approved),
     so when the level covers Gate 1 here, skip the question, go straight
     to `bee-planning`: `references/exploring-reference.md`
     ("Gate 1 bypass mechanics").
   - Else present **Gate 1** per the Gate Presentation Contract: plain-
     language layer in chat (what we decided / why trustworthy / cost if
     wrong / what you are deciding), CONTEXT.md linked not pasted; then
     verbatim: "Decisions locked. Approve CONTEXT.md before planning?"
   - CONTEXT.md is the source of truth for every downstream agent; decision
     IDs are stable and cited, never reinterpreted.

## Headless

No Socratic dialogue. Lock only decisions the request states explicitly
(still with D-IDs); write every gray area into CONTEXT.md's
`Outstanding Questions` and the terminal report instead of asking. Gate 1 is
never self-approved — the report ends "awaiting Gate 1 approval".

## Red Flags

- blind-bundling a dependent question a prior answer could moot, or dumping
  unclassified questions together; or a question answered by the asker
- serializing independent questions into separate rounds when one batched
  message would do
- a question that fails materiality — immaterial, ungrounded, unanswerable
- deep implementation analysis or architecture proposals during exploring
- creating cells or writing code (except the `.bee/spikes/` SEE mock)
- a SEE mock imported by production code, or surviving outside `.bee/spikes/`
- teaching skipped while the user is visibly guessing — a decision locked
  from a guess
- locking a "decision" that is really an implementation choice
- scope creep absorbed instead of deferred
- CONTEXT.md with placeholders, or skipping the fresh-eyes review
- skipping decision locking because "the user seemed to imply it"

When a rule's letter stops serving its purpose here, say so out loud and
deviate with a recorded reason — boundary rules (gates, state, secrets) hold
as written; silent deviation is the defect (bee-hive routing reference,
"Judgment contract").

References: `references/exploring-reference.md`, `references/gray-area-probes.md`,
`references/context-template.md`.

Decisions captured and CONTEXT.md written. Invoke bee-planning skill.
