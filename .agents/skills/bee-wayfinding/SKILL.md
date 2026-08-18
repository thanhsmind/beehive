---
name: bee-wayfinding
description: >-
  Chart a fog-state idea into a map of decision tickets before shaping. Use when the user says "brainstorm", "let's brainstorm", "explore this idea", "think through this", or "wayfinding" — in any language; when an itch or idea has no nameable outcome yet; when a feature request is too vague to shape; when `bee orient` resumes an open map with frontier tickets; or when a `bee route` Qualify verdict parks something for vagueness. Not for locking product decisions in one sitting (that's bee-shaping) or for building product work.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: unavailable
      reason: Ticket resolution logs the settled decision through `bee decisions log`; the map only gists and links it — the decision record stays the single source.
---

# Wayfinding — chart the fog before shaping

Flow position: the spine of the Discovery flow — entered directly (see
Entry, below); exits into `bee-shaping`'s Lock, which consumes the
map's Decisions so far.

## Entry

Signal: the request has no nameable outcome, or is too big for one
shaping session — "I feel our onboarding is wrong somewhere", "what if
we did something with AI here", "I want X but I can't say what X is".
Route here instead of bee-shaping. Also entered by: explicit invocation
(the user asks to brainstorm or explore an idea); `bee orient` resuming
an open map that has frontier tickets; a `bee route` Qualify verdict
that parked an item for vagueness — a map stub already exists there,
resume it, never re-chart from scratch.

If the sweep below turns up no fog, write no map: route straight to
bee-shaping.

## Session 1 — chart the map

1. **Name the destination first.** Interview until the user can say
   what "arrived" looks like: a spec to hand off, a decision to lock,
   or a change made. This is the one thing session 1 must produce; it
   fixes scope for everything that follows.
2. **Sweep wide, not deep, as an interview with the user.**
   Breadth-first across the whole space, in rounds of frontier
   questions (Interview craft, below) — surface the open decisions,
   don't resolve them. A fog line the agent suspects but the user has
   not confirmed still goes in "Not yet specified" — mark it
   `(agent-suspected)`.
3. **Write the map** at `docs/discovery/<effort>/MAP.md` — Destination,
   Notes, Decisions so far, Not yet specified, Out of scope. Template
   and exact section rules: `references/wayfinding-reference.md`.
4. **Create tickets** for what you can phrase sharply now, one file
   each at `docs/discovery/<effort>/tickets/NNN-<slug>.md`. The test:
   can you state the question precisely — not answer it. Fog too dim
   to phrase stays in "Not yet specified" until an answer sharpens it
   enough to graduate.
5. **Fire research tickets immediately**, in parallel, as gather-tier
   subagents (bee-researching). Everything else waits for a session.
6. Stop. Charting is one session's work.

## Interview craft

Interviews run in **rounds** over a **question frontier** — every
question askable now without guessing an answer not yet heard. Ask the
whole frontier in one round, numbered, each carrying the agent's
recommended answer — the user still picks. A question that depends on
an answer still open this round waits for the next round.

Facts are the agent's job, never the user's. A frontier question that
needs a repo or environment fact gets a dispatched read — a
gather-tier subagent or a direct read — instead of a question to the
user, and the rest of the frontier goes out while the read runs.

Round mechanics, the exact question format, a worked example, and the
domain-modeling moves (term challenges, edge scenarios, code
cross-checks): `references/wayfinding-reference.md` ("Interview
craft").

## Ticket types

| Type | Who | Resolves by |
|---|---|---|
| grilling (default) | with user | conversation, in interview-craft rounds (above; deeper craft and bee-shaping's probe menu — `references/wayfinding-reference.md` "Interview craft" and bee-shaping's `bee-shaping/references/gray-area-probes.md` plus its own "Interview craft") — the agent never answers its own question |
| research | agent alone | bee-researching / gather-tier subagent; findings linked from the ticket |
| prototype | with user | a cheap mock under `.bee/spikes/`, built to `references/wayfinding-reference.md` ("Spike rules"); the user reacts to it |
| task | either | manual work that unblocks a decision (provision access, move data) |

A with-user ticket only resolves through live exchange. `blocked-by:`
stays a convention-only frontmatter line — no CLI guard. Claiming a
ticket reserves its file (see "Later sessions", below); the
`claimed-by:` line is display-only, backed by that reservation. The
frontier is open, unblocked, **unreserved** tickets.

## Later sessions — one ticket at a time

1. Load MAP.md only (low-res). Take the user-named ticket, else the
   first frontier ticket. Claim it: `bee reservations reserve --agent
   <name> --cell <effort>-wayfinding --path
   docs/discovery/<effort>/tickets/NNN-<slug>.md`, plus the
   display-only `claimed-by:` line. A reservation deny means take
   another ticket and report the conflict — never write through it. A
   dead session's claim expires with its heartbeat instead of lying
   forever.
2. Resolve it. Zoom into related closed tickets on demand, never
   upfront.
3. Record: the answer goes in the ticket's `## Answer`, the ticket
   closes, and the settled decision is logged with
   `bee decisions log`. MAP.md's Decisions so far only gists the
   answer and links the D-ID — the decision record stays the single
   source. Full protocol: `references/wayfinding-reference.md`.
4. Graduate fog the answer sharpened into new tickets; close or drop
   tickets the answer invalidated; move mis-scoped ones to Out of
   scope. Update MAP.md.
5. One with-user ticket per session — research tickets are exempt,
   they fan out in parallel.

## Exit — hand off, don't build

The map is done when no tickets and no fog remain — nothing left to
*decide*. Each buildable feature then falls out of the map into the
normal chain: bee-shaping's **Lock** consumes the map's Decisions so
far (and their D-IDs) straight into the feature's
`docs/history/<feature>/CONTEXT.md`, citing settled answers rather
than re-asking them. From there: normal lanes, gates, planning,
swarming. Wayfinding adds no gate of its own — it edits only docs; the
destination-naming conversation in session 1 is its human checkpoint.
The map folder stays as history; MAP.md gets a closing line pointing
at the features it spawned.

## Hard rules

- Decide, don't build. The pull to "just do it" means you've reached
  the map's edge — hand off instead.
- Destination before tickets. Scope flows from it, never the reverse.
- One with-user ticket per session.
- The agent never stands in for the user's side of a conversation.
- Don't pre-slice the fog: ticket only what you can phrase sharply now.
- Out of scope never graduates; it returns only as a fresh effort.
- Locked decisions win: a map answer that contradicts a decision
  locked elsewhere is a question for the user, never a silent
  override.

## References

| File | When to load |
|---|---|
| `references/wayfinding-reference.md` | MAP.md template, ticket file template, the resolution protocol, Interview craft (round mechanics, worked example, domain-modeling moves), and Spike rules — exact section wording, frontmatter shape, decision-log and link mechanics |
