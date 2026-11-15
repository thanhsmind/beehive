# Port notes — pstack teach merged into bee-teach, cell ppb-33

Source: pstack skill `teach` (`SKILL.md`), MIT, copyright 2026 Lauren Tan.
`skills/bee-teach/` got `LICENSE.pstack`, a verbatim copy of the pstack
`LICENSE`. This file is the only place a pstack-only tool is named.

Rule applied (D5): the bee skill stays bee's. The Trace and Provenance sweep
procedures stay single-homed in
`skills/bee-researching/references/trace-and-provenance.md`; `bee-teach` restates
no step of either. Every bee rule is kept. The bee-teach description is
unchanged at 269 characters, under the 300 budget.

## What came in

- Step 3: "Explain how it works, don't just name it."
- Step 5: the diagram series "is not a wall" because each step is small and adds
  exactly one idea; three small growing diagrams beat one crowded diagram; a
  visual earns its place by teaching, not decorating.
- Voice: "Padding is the enemy, not ideas"; do not list functions and constants
  like a changelog; the reason for one name per concept, that switching between
  synonyms for the same thing makes the reader re-derive that they are the same.

## Left out because bee already has it

- Teach sits on top of `how` and `why`, blended into one explanation, with the
  confidence language kept: the opening paragraph names Trace and the Provenance
  sweep, the bee homes of those two skills, and keeps the hedges and the empty
  and UNSWEPT categories.
- Decide what they should walk away with, from why they ask: step 1.
- Do not redo the investigation, run both in parallel, match the size to the
  question, keep `why` narrow and record the skipped categories: step 2, which
  names every unswept category as UNSWEPT per the procedure.
- Plain definition first, tie it to the case, the smallest complete answer, never
  a wall, no framing labels: step 3.
- Conversation, not a lecture; no quizzes, no pacing theater: step 4.
- The mermaid and image-generation guidance and the build-up rule for generated
  images: step 5.
- Tight not terse, the target density example, sentence case, no em dashes,
  periods over commas, no mirror sentences or tidy closers, the words are
  directions and not labels: the voice paragraph.
- Reply is the explanation, never a report: the Reply paragraph.

## Left out because bee differs

- "Offer to go deeper or move on" and the one-shot "put any offer to go deeper at
  the end": bee closes on exactly one next action, never a menu, and the gate
  template wins at a gate. The one next action is a ready `/bee-how <question>` or
  `/bee-why <question>`; the Headless section covers the no-human run.
- "Plain spoken English": bee answers in the user's language.
- "Put the narrowing in the ask itself" for `why`: bee's Provenance sweep takes
  the narrowing as its own scope and reports the rest as UNSWEPT, so no ask
  wording is needed.
- Teach's `disable-model-invocation`: bee-teach is invoked by a request to
  understand, per its description.

## Tool swaps

- pstack `how` and `why` skill invocations become the bee procedures Trace and
  the Provenance sweep, already in bee-teach, with the `/bee-how` and `/bee-why`
  doors for the next action. No new tool name was added.

## Not ported

- No pstack step was dropped for lack of a bee counterpart.
- No bee skill or principle was missing for this merge.

## Regen note

`bee dev regen` also re-renders unrelated `.claude/skills/` entries in this
checkout, as in ppb-30 to ppb-32; those stay uncommitted. Only the `bee-teach`
directory in each rendered tree, the render manifests, the onboarding hash and
the release manifest are committed.
