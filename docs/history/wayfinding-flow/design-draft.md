# Wayfinding flow — design draft (v1)

Status: DRAFT for discussion. Not shaped, not gated, no code.
Source of the pattern: mattpocock wayfinder skill (map of decision
tickets + fog of war), adapted to bee's existing machinery.

## Problem

bee's front door (bee-shaping) assumes the user already has *a request*
— maybe fuzzy, but nameable. It converges in one session to locked
decisions. There is no flow for the earlier state: the user has an
itch, an idea in fog, and cannot yet name the outcome. Today that user
either gets a premature shaping interview or nothing.

## Decisions so far (this design)

- D-A: Separate skill, not a new move inside bee-shaping. Shaping
  stays one-session convergent; wayfinding is multi-session divergent.
- D-B: Map lives in `docs/discovery/<effort>/` as plain markdown
  (MAP.md + one file per ticket). No new bee state machinery for v1.
- D-C: Skill name is `bee-wayfinding` (user pick; keeps the source
  pattern's name traceable).
- D-D: Open maps ARE visible in `bee status` / the session preamble
  from v1 ("open map: <name>, N frontier tickets"). This is the one
  code-touching piece of v1: the status command scans
  `docs/discovery/*/MAP.md`. Ships through the normal bee chain.
- D-E (widened 2026-08-17, user-approved): v1 activation ships four
  mechanisms, by hardness:
  1. Hard (code) — resume: `bee status` scans `docs/discovery/*/MAP.md`;
     the SessionStart preamble prints open maps every session, and
     `bee orient` returns `skill=bee-wayfinding` deterministically when
     an open map has frontier tickets and no feature is running.
  2. Hard — explicit invocation: user calls the skill / asks to
     brainstorm.
  3. Hard (code) — headless path: `bee route` Qualify, on a
     parked-for-vagueness verdict, creates a map stub in
     `docs/discovery/` instead of letting the item sink; the stub shows
     up in status. Fog never sinks silently.
  4. Semi-hard — mid-conversation fog: a mandatory first check at
     bee-shaping's entry ("no nameable outcome → stop, switch to
     bee-wayfinding"), with Gate 1 (user reviews CONTEXT.md) as the
     human backstop against invented decisions. Semantic fog detection
     cannot be enforced by hooks; this is the honest limit.
- D-F: claim/blocked lines in ticket files are convention-only in v1;
  no CLI guard.

## The flow

### Entry

Signal: the request has no nameable outcome, or is too big for one
shaping session. Examples: "I feel our onboarding is wrong somewhere",
"what if we did something with AI here", "I want X but I can't say
what X is". `bee orient` / bee-hive routes here instead of shaping.
Explicit invocation also works: user asks to brainstorm/explore an
idea.

### Session 1 — chart the map

1. **Name the destination first.** Interview until the user can say
   what "arrived" looks like: a spec to hand off, a decision to lock,
   or a change made. The destination fixes scope. This is the one
   thing session 1 must produce.
2. **Sweep wide, not deep.** Breadth-first interview across the whole
   space: surface the open decisions, don't resolve them.
3. **Write the map** at `docs/discovery/<effort>/MAP.md`:
   - Destination (1–2 lines)
   - Notes (domain, standing preferences)
   - Decisions so far (index: one line + link per closed ticket)
   - Not yet specified (the fog — suspected questions, too dim to ticket)
   - Out of scope (consciously ruled out; never re-enters)
4. **Create tickets** you can phrase sharply now — one file each,
   `docs/discovery/<effort>/tickets/NNN-<slug>.md`. Body = the
   question. The test: can you state the question precisely (not:
   answer it).
5. **Fire research tickets immediately** as parallel subagents
   (bee-researching / gather tier). Everything else waits.
6. Stop. Charting is one session's work.

If the sweep surfaces no fog — the way is already clear — no map:
route straight to bee-shaping.

### Ticket types (4, from wayfinder)

| Type | Who | Resolves by |
|---|---|---|
| grilling | with user (default) | conversation; agent NEVER answers its own question |
| research | agent alone | bee-researching / gather subagent; findings linked from ticket |
| prototype | with user | cheap mock under `.bee/spikes/`, user reacts to it |
| task | either | manual work that unblocks a decision (provision access, move data) |

A with-user ticket only resolves through live exchange. Blocking is a
`blocked-by:` line in the ticket file; the frontier = open, unblocked,
unclaimed tickets.

### Later sessions — one ticket at a time

1. Load MAP.md only (low-res). Pick the user-named ticket, else the
   first frontier ticket. Claim it (a `claimed-by:` line).
2. Resolve it. Zoom into related closed tickets on demand.
3. Record: answer goes in the ticket file, ticket closes,
   `bee decisions log` the settled decision (the map links the D-ID —
   the decision record stays the single source, the map only gists).
4. Graduate fog the answer sharpened into new tickets; delete/park
   tickets the answer invalidated; move mis-scoped ones to Out of
   scope. Update MAP.md.
5. One ticket per session (research tickets exempt — they fan out).

### Exit — hand off to the existing flow

The map is done when no tickets remain and the fog is empty: nothing
left to *decide*. Then each buildable feature falls out of the map into
the normal chain:

- bee-shaping **Lock** consumes the map's Decisions-so-far (and their
  D-IDs) straight into `docs/history/<feature>/CONTEXT.md` — settled
  answers are cited, never re-asked.
- From there: normal lanes, gates, planning, swarming. Wayfinding adds
  no gate of its own — it edits only docs, and the destination-naming
  conversation is its human checkpoint.
- The map folder stays as history; MAP.md gets a closing line pointing
  at the features it spawned.

## Hard rules (inherited from wayfinder + bee)

- Decide, don't build. The pull to "just do it" = you've reached the
  map's edge; hand off instead.
- Destination before tickets. Scope flows from it.
- One with-user ticket per session.
- The agent never stands in for the user's side of a conversation.
- Don't pre-slice the fog: ticket only what you can phrase sharply.
- Out of scope never graduates; it returns only as a fresh effort.

## Reuse of existing bee machinery

| Need | Existing piece |
|---|---|
| record a settled answer | `bee decisions log` |
| research ticket execution | bee-researching / gather-tier subagents |
| prototype | `.bee/spikes/` (already the one shaping exception) |
| deferred real work found mid-map | `bee backlog add` |
| routing in | `bee orient` / bee-hive route table |
| waiting-on-user marks | `bee state waiting-on set` |

## Open questions

None — D-A through D-F settled 2026-08-17 with the user. Next step
when the user says go: run this draft through the normal bee chain
(shaping consumes this doc) to build v1:

1. `bee-wayfinding` skill (skill + references; no code).
2. Status/preamble scan of `docs/discovery/*/MAP.md` + orient
   recommendation logic (Rust, D-D + D-E.1).
3. `bee route` park-for-vagueness → map stub (Rust, D-E.3).
4. bee-shaping entry check + Lock consuming a map's Decisions-so-far
   (skill text edit, D-E.4 + exit contract).
