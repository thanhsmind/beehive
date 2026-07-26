# Stage: exploring (`bee-exploring`)

**Purpose** — Turn a fuzzy, gray-area request into *locked decisions* recorded in
`docs/history/<feature>/CONTEXT.md`, so planning never has to guess at product
intent.

**When it runs** — For vague or new feature requests with unstated product
decisions. Runs before planning; skipped when scope is already clear.

## Inputs
- Critical-patterns digest, [`state.json`](../register.md#beestatejson).
- A quick read-only scout (keyword `rg` pass, 2–3 files) — enough to ask good
  questions, not to design.
- Backlog PBI status.

## Outputs
- `docs/history/<feature-slug>/CONTEXT.md` — boundary, domain types, a **locked
  decisions table with D-IDs**, pinned terms, deferred ideas.
- Optional throwaway UI mock under `.bee/spikes/<feature>/mocks/`.
- PBI adds for deferred ideas.

## Gate
**Gate 1** — "Decisions locked. Approve CONTEXT.md before planning?"

## State touched
`state start-feature` (idle→exploring), `state set --owner exploring`,
[`decisions log`](../register.md#beedecisionsjsonl),
[`backlog pbi add/status`](../register.md#beebacklogjsonl),
`state gate --name context`.

## Key rules
- **Never answer your own question** — gray areas are the user's to resolve.
- **Never** research implementation, propose architecture, create cells, or write
  code (spike mocks excepted).
- **Never invoke planning itself** — hand the approved gate to the user.
- Batch independent questions into one message; serialize only dependent ones.

## Source
`skills/bee-exploring/SKILL.md`
