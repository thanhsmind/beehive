# Stage: exploring (`bee-shaping` — Explore / Qualify / Lock)

**Purpose** — Turn a fuzzy, gray-area request into *locked decisions* recorded in
`docs/history/<feature>/CONTEXT.md`, so planning never has to guess at product
intent.

**When it runs** — For vague or new feature requests with unstated product
decisions. Runs before planning; skipped when scope is already clear. Three moves
share the skill: **Explore** interviews a human, **Qualify** triages a backlog row
unattended, and both feed **Lock**, the single writer of the decision record.
(**Brief** — rendering the implement plan — is the fourth move and belongs to
[planning](planning.md)'s gate, not to this stage.)

## Inputs
- The request itself, plus a light read-only scout — enough to ask good questions,
  never enough to design. Effort follows the request's own signal: a clear ask
  confirms one reading and locks; a vague one earns the full interview.
- Critical-patterns digest, [`state.json`](../register.md#beestatejson), backlog
  PBI status.
- For Qualify: the item's text plus the code and docs it touches — never the row
  alone.

## Outputs
- `docs/history/<feature-slug>/CONTEXT.md` — boundary, domain types, a **locked
  decisions table with D-IDs**, pinned terms, deferred ideas, and an
  `Outstanding Questions` section for whatever stayed unresolved.
- The shaping anchor: `bee shape --request "<verbatim>" --acceptance "<what done
  means>"`, written before any code is touched.
- A recorded triage call on the Qualify path (`bee route`).
- PBI adds for deferred ideas; an optional throwaway mock under `.bee/spikes/`.

## Gate
**Gate 1** — "Decisions locked. Approve CONTEXT.md before planning?"

## State touched
`state start-feature` (idle→exploring), `state set --owner exploring`,
[`bee shape`](../register.md#beeintentkeyjson),
[`decisions log`](../register.md#beedecisionsjsonl) — the moment each answer
settles, never batched at the end,
[`backlog add` / `backlog pbi add/status`](../register.md#beebacklogjsonl),
`bee route --set` (Qualify), `bee gate --name context`.

## Key rules
- **Never answer your own question** — gray areas are the user's to resolve.
- **Product decisions only.** States, error shape, who consumes the output.
  Implementer-only questions belong to planning; no architecture proposals, cells,
  or code — the one exception is a throwaway mock under `.bee/spikes/`.
- **Lock renders; it never originates.** A section the input left silent is an
  Open Question, never a guess. Invented filler is the failure this stage exists
  to prevent.
- **Risk territory parks at any confidence** on the Qualify path — auth, data
  loss, security, external providers, validation removal. Risk is a property of
  the change, not of the assessor's certainty.
- **Reasoning moves are made in plain words** — "let me check what we're assuming",
  not the method's name. `.bee/expertise/thinking.md` is a routing table, not
  conversation vocabulary.
- Stop when every locked decision can be written without inventing intent. Past
  that, questions are stalling — never ask "anything else?"; present what's locked.
- **Never invoke planning itself** — hand the approved gate to the user.

## Source
`skills/bee-shaping/SKILL.md` + `references/{gray-area-probes, shaping-reference, context-template}.md`;
craft in `.bee/expertise/decisions.md` and, for domain-shaped gray areas, one
guide routed from `.bee/expertise/INDEX.md`
