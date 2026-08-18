---
name: bee-shaping
description: >-
  Shape fuzzy intent into locked, buildable decisions. Use when a feature request has gray areas or unstated product decisions, when a new backlog item needs its first unattended triage (proceed or park), when resolved decisions must be locked into docs/history/<feature>/CONTEXT.md, or when a feature's implement plan must be rendered for gate review. One skill is the whole front door — the interviewer, the triage judge, the decision scribe, and the plan renderer. Not for implementation research, cell creation, or code.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: Records decisions, routes, and backlog items through the vendored bee binary. The binary is vendored into the repo by onboarding; no Node runtime is involved.
---

# Shaping — from fuzzy intent to a reviewable plan

**Explore** interviews a human through gray areas; **Qualify** triages a
backlog item unattended. Both feed **Lock**, the single writer of the
decision record; **Brief** renders the implement plan when the lane earns
one. `bee orient` shows which move the work needs.

## Explore (interactive)

**Entry check, before anything else.** If the request names no outcome
the user can state — pure fog, not a gray area — stop here and route to
bee-wayfinding instead of interviewing (D6.4); the triage table below is
for requests that already name a destination.

The craft is interviewing, not writing. Scout lightly, then triage the
request. With a bundle, the scout starts at the touched area's
`docs/knowledge/areas/<area>/index.md` and its Open Gaps (no bundle:
`docs/specs/<area>.md` when present): settled rules get cited, never
re-asked, and Open Gaps are ready-made interview questions. Default:
effort follows the signal, not a fixed script:

| Request | Signal | Next step |
|---|---|---|
| Clear | Bug report or bounded ask; outcome and scope both stated | Confirm your one reading, then Lock — likely tiny/small lane |
| Partially clear | Hedges ("probably", "I think"); named outcome with unstated states, errors, or consumers | Probe only the gray areas the hedges mark |
| Vague | "What if we…"; a solution with no named problem; an aside dropped into a longer thread | Full interview — surface the 2–4 unstated product decisions |

Default: do not ask how deep to go — the request's shape already answered. Ask
only *product* decisions planning would otherwise guess — states, error
shape, who consumes the output (`references/gray-area-probes.md`);
implementer-only questions go to planning. Deeper craft — question
cadence, teaching, pinned terms, the SEE mock — lives in
`references/shaping-reference.md` ("Interview craft"). The moves sound like:

> "You've used 'archived' and 'removed' for the same state. Is a
> dismissed item (a) archived — recoverable from a list — or (b) gone
> from every view? I'll pin whichever term you pick."

> "You asked for retry, but `sync/runner.js:88` already retries three
> times. Is the gap (a) invisible retries, or (b) too few attempts?"

> "Turning it around: what must the export never include — soft-deleted
> rows (a) stay out entirely, or (b) appear flagged?"

Make each reasoning move in plain words — "let me check what we're
assuming", not "applying First Principles"; the method names in
`.bee/expertise/thinking.md` are your routing table, not conversation
vocabulary. `bee decisions log` the moment each answer settles — never
batched at the end. Defer scope creep with `bee backlog add`, then
return to the current question. Never answer your own question.

Stop when every locked decision can be written without inventing intent
and each remaining unknown is a named Open Question; past that,
questions are stalling — never ask "anything else?", present what's locked.

## Qualify (headless triage)

The unattended front door for a new backlog item: judge whether it can
proceed into planning or must wait for a human. No questions are asked
on this path; everything unresolved is written down, never guessed.

1. Gather real evidence first — the item's text, the code and docs it
   touches, and the touched area's knowledge concepts
   (`docs/knowledge/areas/<area>/`; no bundle: `docs/specs/<area>.md`).
   Never judge from the row alone. A bug-shaped item gets its claim
   verified by reproducing it, or by recording the failed reproduction
   attempt as evidence, before any verdict. Check for duplication by
   searching decisions and the backlog for the same domain concept,
   never by matching the request's wording.
2. Risk territory (auth, data loss, security, external providers,
   validation removal) parks at any confidence — risk is a property of
   the change, not of the assessor's certainty.
3. Judge clarity and size over the evidence with the triage table above —
   your own reasoning, never a keyword match; a vague row parks here
   instead of interviewing.
4. `bee route` records the call. Proceed → Lock, then planning.
   Park → Lock writes the evidence and open questions into `Outstanding
   Questions`; a later human Explore starts there, not from scratch. When
   the park cause is vagueness, not risk, also run `bee discovery stub
   --effort <slug> --from '<item text>'` so the parked fog becomes a
   visible map stub instead of sinking silently (D6.3); risk parks keep
   the Outstanding Questions path unchanged.

## Lock (single writer)

`docs/history/<feature>/CONTEXT.md` (`references/context-template.md`)
is the one decision record, written here for both paths. Lock renders
what Explore or Qualify resolved — it never originates a decision, term,
boundary, or scope note; a section the input left silent is an Open
Question, never a guess. Concrete language only. Deferred ideas that are
real future work get `bee backlog add` in the same turn. Downstream work
cites D-IDs, never reinterprets them (AGENTS.md). When a
`docs/discovery/<effort>/` map backs the feature, Lock consumes the
map's Decisions so far — the settled answers and their D-IDs — straight
into CONTEXT.md, citing them; it never re-asks a question the map
already resolved (D8).

**The `tiny`/`docs` brief (D1, traceable-runs).** Every file-touching
request, at every lane, needs a brief written before any source edit —
not a new artifact type, the same file: a SHORT `docs/history/<feature>/CONTEXT.md`,
naming only what was asked, what was found, and what will be done in as
few lines as the request needs. No mini-Explore interview, no plan
ceremony added — the short brief earns exactly its own Gate 1 approval
(Lane ceremony table, `routing-and-contracts.md`), never a `plan.md`
requirement the lane didn't already have. A pure question that writes no
file skips Lock entirely (D6) — it gets no brief and no record.

## Brief (the gate's document)

When the lane calls for one, render
`docs/history/<feature>/implement-plan.md` — the one document human and
agent review at the gate: full template for standard/high-risk
(`references/implement-plan-template.md`), ~15-line mini-brief for small
(`references/mini-brief-template.md`), none below. Every section
projects from a named source (CONTEXT.md, plan.md, cells, verify
records); only Technical Design and Rollback are authored, and only
from what those artifacts imply — a choice they don't contain is an
Open Question, never smuggled in as the plan. Feedback lands on the
truth artifacts first, then the brief re-renders — never the brief
alone. Post-ship, standard/high-risk features get a walkthrough from
execution records, never the plan (`references/walkthrough-template.md`).

## Hard rules

- Lock and Brief render; they never originate scope, decisions, or
  approach — invented filler is the failure this skill exists to prevent.
- Explore locks product decisions only; implementation choices belong to
  planning. No architecture proposals, cells, or code — the one exception
  is a throwaway mock under `.bee/spikes/`.
- Gates belong to the user (AGENTS.md); shaping only presents one.

## References

| File | When to load |
|---|---|
| `references/gray-area-probes.md` | Generating gray-area questions per domain type |
| `references/shaping-reference.md` | Interview craft: materiality, blindspot pass, SEE mock, pinned terms, stopping |
| `references/context-template.md` | Writing CONTEXT.md |
| `references/implement-plan-template.md` | Full implement plan: template, section sources, writing guide |
| `references/mini-brief-template.md` | The small-lane ~15-line brief |
| `references/walkthrough-template.md` | Post-ship walkthrough |
| `.bee/expertise/decisions.md` | What makes a lockable decision |
| `.bee/expertise/INDEX.md` | A gray area is domain-shaped — what deletion means, what a contract promises callers, who may see what, how fast is fast enough, what an empty or failed state shows: route from the index for the questions worth asking |
