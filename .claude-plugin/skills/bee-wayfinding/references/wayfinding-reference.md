# Wayfinding reference

Loaded from `SKILL.md` when charting a map, filing a ticket, or closing
one. Exact section wording, frontmatter shape, and the resolution
protocol live here so `SKILL.md` can stay short.

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
  once an answer (or a sharper question) makes it phraseable.
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
claimed-by: <agent-or-user, convention-only>
blocked-by: <NNN-<slug> | none>
---

## Question

<the question, stated precisely — the test is phrasing it, not
answering it>

## Answer

<filled on close: the settled answer, plus the D-ID it was logged
under>
```

Frontmatter is convention-only in v1 (no CLI guard): agents read and
write these lines by hand, and the frontier is computed by reading
them, not by a tool enforcing them.

## Resolution protocol

1. Resolve the ticket through its type's channel: grilling is a live
   exchange (the agent never answers its own question); research runs
   through a bee-researching / gather-tier subagent; prototype is a
   spike under `.bee/spikes/` the user reacts to; task is the manual
   work itself.
2. Write the answer into the ticket's `## Answer` section. Set
   `status: closed`.
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
