# Shaping Reference — interview mechanics

Deep mechanics for the Explore move. The body triages the request and
shows the moves; this file carries the craft and judgment detail.

## Interview craft

- **One question per turn.** Each answer reframes what is worth asking
  next; a batch buries the reframe. Broad questions lead — they are the
  ones others hinge on.
- **Propose, then invert.** Lead with a concrete recommendation the user
  can accept in a word; then invert — "what must this NOT do?" — the
  boundary is a decision too.
- **Pin vocabulary.** When an answer settles a fuzzy domain word, confirm
  the term back and pin it (see "Pinned terms" below).
- **Teach before asking.** A user visibly guessing gets the 2–3 concepts
  needed to answer well before the question (see "Blindspot pass" below).
- **Ask only what passes the materiality test** (next section); what makes
  a decision worth locking at all: `.bee/expertise/decisions.md`.
- **Mark the wait.** Every interview question ends the turn waiting on
  the human: run `bee state waiting-on set --kind question --subject
  "<the question>"` before sending, so an external reader sees
  "waiting on you", not "idle" (full rule: bee-hive
  routing-and-contracts, "Question Format").

## When to stop

Stop when every locked decision can be written into CONTEXT.md without
inventing intent and each remaining unknown is named as an Open
Question. Effort matches the triage row, never a question quota: a
clear bug report can lock from a single confirmation; only a genuinely
vague idea earns the full 2–4 probes. Never ask the user how deep the
interview should go, never ask "anything else?", and never pad with
questions whose answers would not change scope, UX, data shape, or
acceptance criteria.

## Edge of the map

A fuzzy ask shrinks decision by decision, not by charging at the whole
destination.

- **The pull to build is a stop signal.** A mid-interview urge to "just
  implement it and find out" marks the edge of what is understood — turn
  it into the next question or a named Open Question, never into code.
  The SEE mock stays the one exception.
- **Fog test for deferrals.** Defer to the backlog only what can be
  stated precisely right now — one sentence, a nameable outcome. What
  cannot be stated yet is fog: hold it as an Open Question where it
  sits; a premature backlog row fakes clarity and poisons later triage.
- **Each locked answer redraws the map.** Re-triage what is still gray
  after every lock before choosing the next question — an answer often
  dissolves questions that seemed essential a turn ago.

## Materiality test

Every candidate question passes three checks before it is asked:

- **material** — the answer changes scope, architecture, UX, data model, or
  acceptance criteria
- **grounded** — cites scout evidence or a concrete uncertainty, never
  generic preference
- **answerable** — the user can pick an option, approve a default, or supply
  a reference

A failing question is never asked: pin it as a labeled assumption for the
decision record, or hand it to planning if only the implementer cares about
the answer.

## Blindspot pass

Teach before asking. When the user signals unfamiliarity with a gray area's
domain — says so, answers with guesses, or asks what the options mean —
invert for that area: explain the 2–3 concepts needed to answer well (one
short outcome-framed message, no jargon), *then* ask. A decision locked
from a guessed answer is a fake decision. The user can also request a full
"blindspot pass" by name: sweep the unknown-unknowns (what good looks like,
common potholes, prior art in this repo) before locking begins.

## SEE mock

React instead of describe. For a user-visible surface the user
knows-when-they-see-it but cannot describe, you MAY build a throwaway HTML
mock (2–4 variants, fake data, zero wiring) under
`.bee/spikes/<feature>/mocks/` and lock the decision from the user's
reaction, citing the chosen variant. This is the ONE exception to "shaping
never writes code": mock files only, only under `.bee/spikes/`, never
imported by anything, never promoted to production.

## Pinned terms

When an answer settles the meaning of a fuzzy domain word, confirm the term
back and pin it like a decision; Lock writes all pinned terms into
CONTEXT.md's `Terms` section, and the spec's Data Dictionary inherits them.

## One answer, several decisions

When one answer settles more than it was asked about, lock the one asked
about and echo the others back as candidates to confirm one at a time —
never lock a decision the user only implied.
