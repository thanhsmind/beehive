# Shaping Reference — interview mechanics

Deep mechanics for the Explore move. The body states the rules; this file
carries the judgment detail.

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
