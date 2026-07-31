---
name: bee-shaping
description: >-
  Shape fuzzy intent into locked, buildable decisions. Use when a feature request has gray areas or unstated product decisions, when a new backlog item needs its first unattended triage (proceed or park), when resolved decisions must be locked into docs/history/<feature>/CONTEXT.md, or when a feature's implement plan must be rendered for gate review. One skill is the whole front door — the interviewer, the triage judge, the decision scribe, and the plan renderer. Not for implementation research, cell creation, or code.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Records decisions, routes, and backlog items through the vendored .bee/bin CLI.
---

# Shaping — from fuzzy intent to a reviewable plan

Four moves share this skill. **Explore** interviews a human through gray
areas; **Qualify** triages a backlog item unattended. Both feed **Lock**,
the single writer of the decision record; **Brief** renders the implement
plan when the lane earns one. `bee orient` shows which move the work needs.

## Explore (interactive)

The craft is interviewing, not writing. Scout the code lightly first, then
surface the 2–4 unstated *product* decisions planning would otherwise
guess — states, error shape, who consumes the output
(`references/gray-area-probes.md`). A question only the implementer cares
about goes to planning, never to the user.

- **One question per turn.** Each answer reframes what is worth asking
  next; a batch buries the reframe. Broad questions lead — they are the
  ones others hinge on.
- **Propose, then invert.** Lead with a concrete recommendation the user
  can accept in a word; then invert — "what must this NOT do?" — the
  boundary is a decision too.
- **Pin vocabulary.** When an answer settles a fuzzy domain word, confirm
  the term back and pin it; pinned terms ride into the decision record and
  seed the spec's data dictionary.
- **Teach before asking.** A user visibly guessing gets the 2–3 concepts
  needed to answer well, outcome-framed, before the question — a decision
  locked from a guess is fake. For a look the user knows-when-they-see-it,
  a throwaway mock beats a description
  (`references/shaping-reference.md`).
- Ask only what is material, grounded, and answerable; what makes a
  decision worth locking at all: `.bee/expertise/decisions.md`.

`bee decisions log` the moment each answer settles — never batched at the
end. Scope creep is deferred with `bee backlog add`, then back to the
current question. Never answer your own question, even when sure.

## Qualify (headless triage)

The unattended front door for a new backlog item: judge whether it can
proceed into planning or must wait for a human.

1. Gather real evidence first — the item's text, the code and docs it
   touches. Never judge from the row alone.
2. Risk territory (auth, data loss, security, external providers,
   validation removal) parks at any confidence — risk is a property of the
   change, not of the assessor's certainty.
3. Judge clarity and size with your own reasoning over the evidence —
   never a keyword match; zero matches against a list proves nothing.
4. `bee state route` records the call. Proceed → Lock, then planning.
   Park → Lock writes the evidence and open questions into
   `Outstanding Questions`, and the item waits for a human Explore pass —
   which starts from that brief instead of re-gathering.

No questions are asked on this path; everything unresolved is written
down, never guessed.

## Lock (single writer)

`docs/history/<feature>/CONTEXT.md`
(`references/context-template.md`) is the one decision record, written
here for both paths: boundary, locked decisions with stable D-IDs, pinned
terms, scout paths, open questions, deferred ideas. Lock renders what
Explore or Qualify resolved — it never originates a decision, term,
boundary, or scope note. A section the input left silent is an Open
Question, never a guess. Concrete language only. Deferred ideas that are
real future work get `bee backlog add` in the same turn. Downstream work
cites D-IDs; it never reinterprets them.

## Brief (the gate's document)

When the lane calls for one, render
`docs/history/<feature>/implement-plan.md` — the one document human and
agent review together at the gate. Full template for standard/high-risk
(`references/implement-plan-template.md`), ~15-line mini-brief for small
(`references/mini-brief-template.md`), none below that. Every section
projects from a named source (CONTEXT.md, plan.md, cells, verify records);
only Technical Design and Rollback are authored, and only from what the
artifacts already imply — a choice they don't contain is an Open Question,
never smuggled in as the plan. Feedback flows to the truth artifacts
first, then the brief re-renders; the brief is never the sole change site.
Post-ship, standard/high-risk features get a walkthrough reconstructed
from execution records, never from the plan
(`references/walkthrough-template.md`).

## Hard rules

- Lock and Brief render; they never originate scope, decisions, or
  approach — inventing content to fill a section is the failure this
  skill exists to prevent.
- Explore locks product decisions only; implementation choices belong to
  planning. No architecture proposals, cells, or code — the one exception
  is a throwaway mock under `.bee/spikes/`.
- Gates belong to the human: shaping ends by presenting one, never by
  approving one.

## References

| File | When to load |
|---|---|
| `references/gray-area-probes.md` | Generating gray-area questions per domain type |
| `references/shaping-reference.md` | Interview mechanics: materiality, blindspot pass, SEE mock, pinned terms |
| `references/context-template.md` | Writing CONTEXT.md |
| `references/implement-plan-template.md` | Full implement plan: template, section sources, writing guide |
| `references/mini-brief-template.md` | The small-lane ~15-line brief |
| `references/walkthrough-template.md` | Post-ship walkthrough |
| `.bee/expertise/decisions.md` | What makes a lockable decision |
