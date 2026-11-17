# Wayfinding reference

Loaded from `SKILL.md` when charting a map, filing a ticket, or closing
one. Exact section wording, frontmatter shape, and the resolution
protocol live here.

## MAP.md template

Path: `docs/discovery/<effort>/MAP.md`

```markdown
# <Effort name> — discovery map

## Destination

<1-2 lines: what "arrived" looks like — a spec, a locked decision, or a
change made>

## Notes

<domain context, standing preferences the interview surfaced>

## Decisions so far

- D<NN>: <one-line gist> — tickets/NNN-<slug>.md

## Not yet specified

- <fog: a suspected question, too dim to phrase sharply yet>

## Out of scope

- <consciously ruled out; never re-enters this map>
```

Section rules:

- **Destination** is written first, before any ticket exists, and only
  changes if the user re-scopes the effort out loud.
- **Notes** holds context, not decisions — nothing here is a settled
  answer.
- **Decisions so far** never restates a decision's full text; it gists
  and links. The ticket file and `bee decisions log` are the single
  source.
- **Not yet specified** is the fog list: questions the map knows exist
  but cannot phrase precisely yet. A line here graduates to a ticket
  once an answer (or a sharper question) makes it phraseable. A line
  the agent suspects but the user has not confirmed carries the
  `(agent-suspected)` marker until a round of Interview craft (below)
  confirms or drops it.
- **Out of scope** is one-way. A line that lands here never returns to
  this map; it can only come back as a fresh effort.

On exit (Session — Exit in `SKILL.md`), add a closing line under
Destination pointing at what the map spawned, for example:

```markdown
Spawned: <feature-slug> — docs/history/<feature-slug>/CONTEXT.md
```

## Ticket file template

Path: `docs/discovery/<effort>/tickets/NNN-<slug>.md`

`NNN` is a zero-padded three-digit sequence, assigned in creation order
within the effort. Never reused after a ticket closes, is dropped, or
moves to Out of scope.

```markdown
---
type: grilling | research | prototype | task
status: open | claimed | closed
claimed-by: <agent-or-user, reservation-backed — see below, display-only>
blocked-by: <NNN-<slug> | none, convention-only>
---

## Question

<the question, stated precisely — the test is phrasing it, not
answering it>

## Answer

<filled on close: the settled answer, plus the D-ID it was logged
under>
```

Claiming a ticket reserves its file: `bee reservations reserve --agent
<name> --cell <effort>-wayfinding --path
docs/discovery/<effort>/tickets/NNN-<slug>.md`, alongside the
display-only `claimed-by:` line above. A reservation deny means take
another ticket and report the conflict, never write through it; a dead
session's claim expires with its heartbeat instead of lying forever.
`blocked-by:` stays convention-only in v1 (no CLI guard): agents read
and write it by hand. The frontier is open, unblocked, **unreserved**
tickets.

## Interview craft

Round mechanics, restated in full:

- The **frontier** is every question askable now without guessing an
  answer not yet heard. Ask the whole frontier in one round, numbered,
  each carrying the agent's recommended answer — the recommendation
  never substitutes for the user's own pick.
- A question whose answer depends on another question still open this
  round belongs to the *next* round, not this one.
- Facts are the agent's job, never the user's. A frontier item that
  needs a repo or environment fact is not asked at all — dispatch a
  gather-tier subagent or a direct read for it, and ship the rest of
  the frontier now; only the questions that depend on that fact wait.
- Each round the user's answers reshape the map: settled decisions
  push the frontier outward and unblock what depended on them.
  Recompute the frontier before sending the next round.

Emit format — the exact shape a round goes out in:

```
❓ **Q1** - **<question title>**: <question body — states the choice
plainly, options where useful>

➡️ <the agent's recommended answer>
```

Worked example — charting "onboarding is wrong somewhere", round 1:

```
Frontier this round: Q1 and Q2 go to the user now. A third open item —
whether onboarding already has a tracked completion metric — is a
fact, not a decision: dispatched to a gather-tier subagent instead of
asked. Round 1 ships without waiting on it.

❓ **Q1** - **Who feels the onboarding pain?**: new signups, migrating
teams, or both? Decides whether the map splits into two destinations.

➡️ Both — the friction reports named both cohorts, but confirm before
splitting the map.

❓ **Q2** - **What does "wrong" mean here?**: drop-off, support load,
or time-to-value. Each points at a different destination.

➡️ Drop-off — the last retro flagged that number specifically.

(Q3, "which metric proves it's fixed", depends on Q2's answer and the
dispatched fact-read — held for round 2.)
```

Domain-modeling moves — pull these into any round when the interview
touches vocabulary or a stated behavior, not only when charting new
ground:

- **Challenge a term** that conflicts with an earlier pinned meaning —
  name the conflict, ask which one is meant.
- **Propose a canonical term** for a word the user is using loosely —
  offer the precise replacement, let the user confirm or correct it.
- **Invent a concrete edge scenario** to force precision when a
  relationship or boundary stays fuzzy in the abstract.
- **Cross-check a claim against the code** — when the user states how
  something works, verify it and surface any contradiction.

Domain-probe menus by output shape (SEE/CALL/RUN/READ/ORGANIZE) and the
materiality test that caps how many probes get asked: bee-shaping's
`bee-shaping/references/gray-area-probes.md` and its own
"Interview craft" (`bee-shaping/references/shaping-reference.md`).

## Spike rules

For prototype tickets — the cheap mock a with-user ticket resolves on.

- The ticket's `## Question` is stated at the top of the spike, in the
  file or the first line of output — a reader sees what it's answering
  before anything else.
- Runnable in one command or one double-click. No setup step, no
  reading required to start it.
- No persistence, no polish, no tests. State lives in memory; the
  point is answering the question fast, not shipping code.
- The full relevant state is shown after every action, so the user
  sees what changed without asking.
- A logic or state question reads as one LOGIC page, top to bottom:
  the title plus the question, a labelled state panel re-rendered
  after every action (labelled fields, never raw JSON), one free-play
  button per action, and guided-walkthrough steps that each reset to
  a known initial state.
- When the ticket's question is "which shape", build several variants
  side by side — one polished take answers nothing a menu wouldn't.
  More than five variants stops being different and starts being
  noise.
- On close: the verdict and what it settled go into the ticket's
  `## Answer`. The spike itself stays under `.bee/spikes/` as history —
  never deleted, never promoted in place.

## Resolution protocol

1. Resolve the ticket through its type's channel: grilling is a live
   exchange (the agent never answers its own question); research runs
   through a bee-researching / gather-tier subagent; prototype is a
   spike under `.bee/spikes/` the user reacts to; task is the manual
   work itself.
2. Write the answer into the ticket's `## Answer` section. Set
   `status: closed`, and release the claim's reservation (`bee
   reservations release --agent <name> --cell <effort>-wayfinding`).
3. Log the settled decision: `bee decisions log --relation
   touches:<id>` (or `supersedes:<id>`, or `none` if it settles
   nothing prior — the relation flag is required, AGENTS.md). This is
   the single source of the decision; the ticket and the map both cite
   its ID rather than restate it.
4. Update MAP.md's "Decisions so far": add one line — the decision ID
   and a short gist — linking the ticket file.
5. Graduate fog: if the answer sharpens a line sitting in "Not yet
   specified", create a new ticket for it and remove the fog line.
6. Cascade: if the answer invalidates another open ticket, close that
   ticket too with a one-line `## Answer` note pointing at the
   superseding decision; if a ticket turns out mis-scoped, move its
   line to "Out of scope" in MAP.md and drop the ticket file from the
   frontier.
7. One with-user ticket resolves per session; a research ticket may
   fan out several in parallel within the same session.
