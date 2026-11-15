# Port notes — pstack how merged into bee-how and Trace, cell ppb-31

Source: pstack skill `how` (`SKILL.md` and its `references/`: explorer prompt,
explainer prompt, critic prompt, critique rubric), MIT, copyright 2026 Lauren
Tan. `skills/bee-how/` got `LICENSE.pstack`, a verbatim copy of the pstack
`LICENSE`. This file is the only place a pstack-only tool is named.

Rule applied (D5): the bee skill stays bee's. Trace stays single-homed in
`skills/bee-researching/references/trace-and-provenance.md`; `bee-how` stays a
door and restates no step. Every bee rule is kept. The bee-how description is
289 characters, under the 300 budget.

## What came in

bee-how (the door):
- Description and scope: a placement, ownership or layering question ("where
  should this live", "which package owns this", "is this the right layer") and a
  request for architectural problems are how questions; the door sends them to
  Trace's Critique, account first and verdict below it.
- The dependency reason names the reviewers a Critique sends out.

Trace (`trace-and-provenance.md`):
- Subsystem questions with no single call to follow are cut into distinct
  slices (data model and state, request path, configuration and metrics), each
  slice an entry point under the same floor, ceiling and disjointness.
- The worker return shape gains "components found" (explorer Components Found).
- A walking brief for workers: start broad, follow the thread through callers,
  callees, data flow and type definitions, read the code and never infer from a
  file name, stop when no step is hand-waved, go deep on the one entry point,
  overlap between returns is fine.
- Answer shape: written for a senior engineer new to the area; overview is one
  or two paragraphs; key concepts only the needed ones; prose not pseudocode and
  no code block unless a snippet is essential; mermaid for a structured flow,
  ASCII for a simple one; an analogy only when it helps; where things live is not
  every file.
- A new "Critique" section (pstack Critique Mode, critic prompt, rubric): when
  it runs, trace first, independent reviewers briefed identically with the
  account, the paths and the lenses, the four-part finding (severity structural,
  concern or observation; finding; evidence; impact), the avoid list, "an empty
  critique is valid", lead judgment, and the six lenses (abstraction fit, data
  model, boundary discipline, evolution readiness, complexity versus value,
  consistency).

## Left out because bee already has it

- Assess complexity, simple versus complex: Trace steps 2 and 4 already pick
  disjoint entry points (floor two, ceiling four) and run inline for one honest
  entry point.
- State the best-guess reading and do not ask: Trace step 1.
- Reconcile overlapping or contradicting findings against the code: Trace step 5.
- Name the real parts, explain why a complex step is complex, one line for a
  simple one, acknowledge open questions: the answer shape and UNFOLLOWED.
- Overview, Key concepts, How it works, Where things live, Gotchas: the same
  five sections already in the answer shape.
- The explainer and synthesizer subagents (steps 2b, 3, 4): bee's leader writes
  the account itself; a worker return is evidence, never pasted output. The
  explainer prompt's content is covered by the answer shape and step 5.
- Present the explainer's output lightly edited: moot, the leader writes it.
- The lead judgment buckets and the pragmatic-lead stance: homed in
  `bee-interrogate` Step 5 and `references/lead-judgment.md`; Critique cites it.
- Spawning one critic per configured model: homed in `bee-interrogate` Step 3
  (reviewer count, same-model disclosure); Critique cites it.

## Tool swaps

- Explorer, explainer and critic subagents with a hand-set `generalPurpose`
  type, model and `readonly` flag become workers and reviewers that go out
  through `bee dispatch prepare --runtime <rt> --kind gather` and `--kind reviewer`.
  No model or subagent type is named; the door returns them.
- "Same framework as the interrogate skill" becomes a citation of
  `bee-interrogate` Step 5.
- The pstack default model names and the configured how-explorer, how-explainer
  and how-critics model lists are dropped; bee's team table picks models per role.
- Each rubric lens cites the bee principle that homes its rule instead of
  restating it: the-deletion-test, minimize-reader-load, foundational-thinking,
  model-the-domain, boundary-discipline, redesign-from-first-principles,
  migrate-callers-then-delete, laziness-protocol. The Consistency lens has no
  bee principle and stays a plain question.

## Added, not in the source

- The no-door fallback: when `bee dispatch prepare` is absent, the leader applies
  the lenses one after the other and the account says so (the same fallback
  `bee-interrogate` carries).
- Critique findings use the pstack severity words and say they are not the
  review gate's P1 to P3, so the two vocabularies stay apart.

## Not ported

- The pstack `references/` files as separate files: bee-how stays a door, and
  their content lives in Trace; no skill or principle was missing.

## Closed by ppb-41 (the rows ppb-31c marked missing)

- M1, spawn every explorer in one turn: Trace step 3.
- M2, the flow return carries the data passed between steps: Trace step 3
  return shape.
- M3, How it works is the core and the longest section: Trace answer shape.
- M4, walkthrough before a change: the bee-how description (291 characters).
  "where its parts live" left the description to make room; Scope and the
  answer shape still carry it.
- M5, fan out when an inline pass finds more entry points: Trace step 4.

## Regen note

`bee dev regen` also re-renders unrelated `.claude/skills/` entries in this
checkout, as in ppb-30; those stay uncommitted. Only the `bee-how` directory in
each rendered tree, the `bee-researching` `trace-and-provenance.md` copy, the
render manifests, the onboarding hash and the release manifest are committed.
