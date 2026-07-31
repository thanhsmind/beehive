---
name: bee-capturing
description: >-
  Capture what settles into durable records — area specs, decisions, learnings. SELF-TRIGGERING: invoke this yourself, unprompted, the moment a rule, behavior, or value settles in discussion — the user never has to ask for knowledge to be recorded. Also use when execution completes, when documenting a screen/API/job/area, and when capturing learnings and decisions at feature close or on intentional abandon.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Logs decisions, queues captures, and syncs state through the vendored .bee/bin CLI.
---

# Capturing — keep what settles

Code remembers only outcomes. This skill owns everything else durable:
the **state layer** — `docs/knowledge/` with a bundle, else
`docs/specs/` — where every **area** (screen, API, job, pipeline, CLI
command, process: any unit with observable behavior) has one spec that
is its meaning; the decision log; and the learnings history.

## Capture the moment it settles

Detection is your own duty, unprompted. Most settlements are silent —
the user confirms a behavior, accepts an explanation, picks a value,
moves on — and a spoken settlement ("final", "ok ship it") is captured
that same turn, never deferred. Do not ask "should I document this?";
announce what settled, then record it:

- A rule, behavior, or tuned value settles → `bee decisions log` the
  same turn, plus a one-line stub via `bee capture add`; the full spec
  merge waits for the flush. High-risk work merges now, never queued.
- The user defers work ("later", "phase 2") → `bee backlog add` the
  same turn, announce-then-do.
- A settlement that contradicts shipped behavior is recorded as "not
  yet implemented — see backlog", never stated as current.

Close every task — cell, docs write, quick fix — with a capture line
or an explicit "nothing settled"; smallness is never the answer. What
deserves a record at all, and at what grain: `expertise/decisions.md`.

## Scribe — keep area specs current

The bar is the **rebuild test**: given only the spec (its Pointers
section deleted), a stranger rebuilds the same behavior on another
stack. One area = one spec, forever — locate before you create, update
in place, never fork a `-v2`. Outside Pointers, business vocabulary
only: no language, framework, class, table, or file name.

Deltas come from evidence — capped cells, verification output, UAT
records, user answers — never from plan.md, never from memory. A claim
backed by neither evidence nor a decision enters the spec only as an
Open Gap. A contradicted line is replaced, never kept alongside;
present tense only.

When to write: at feature close, merge the behavior deltas of every
capped `behavior_change` cell into the touched specs, once. When
documenting an area — or a legacy area has code but no spec —
inventory what code proves, then interview for what it cannot;
unanswered questions become Open Gaps, and a partial spec that states
its gaps beats an invented-complete one.

Spec craft — what to write, what to omit, honest gaps:
`expertise/documentation.md`. Template and mechanics:
`references/area-spec.md` ("Area Spec Template", "Merge Rules",
"Harvest Interview", "Rebuild Checklist"). In a bundle repo the CLI
names the write target and emits the frontmatter — ask it; never pick
paths or type frontmatter blocks by eye.

## Compound — close with what you learned

Runs once per feature, after `bee close` goes green — or when work is
abandoned with lessons worth keeping. "The session feels done" is
never a reason to skip it.

1. Flush the capture queue: `bee capture list`, then oldest-first give
   each stub its full spec merge and `bee capture flush --id <id>
   --into <spec>`. A stub is never dropped or summarized away.
2. From the feature's artifacts — CONTEXT.md, plan, cell traces,
   worker reports, review findings — write one dated learnings file
   (`references/promotion.md` ("Learnings File Template")). Delegate
   the reading to read-only subagents; keep synthesis here. Thin
   evidence means a thin file, never an invented finding.
3. Promote a learning only when it clears all three bars:
   multi-feature relevance, meaningful waste prevented, generalizable.
   Prefer an executable check over prose
   (`references/promotion.md` ("Promotion Decision Tree")).
4. Log the decisions future planning must honor; supersede outdated
   ones, never edit them. File unresolved friction with
   `bee backlog add` so grooming can hunt it later.
5. Housekeeping, warn-never-block: refresh the feedback digest and
   sweep the feature scratch — a failure here is a one-line warning,
   never a delay or reversal of the close.
6. Commit the close's own output as one commit, then record the close
   in state and register the head as a review candidate
   (`bee reviews candidate add`). The feature is truthfully unreviewed
   until a user-invoked review covers it.

## Hard rules

- The state layer is read-first, sync-on-change: read it before
  working in an area; sync it the moment behavior changes, not later.
- Specs describe behavior, never code — technology names live only in
  Pointers, and deleting Pointers must remove no business meaning.
- Historical records are never rewritten: decisions are superseded,
  learnings and logs appended. History lives in git, not in prose.
- Secrets and PII never enter a spec, decision, learning, or backlog
  row.

## Headless

Apply mechanical merges; log verbatim-quotable decisions. Harvest
questions, ambiguous merges, and critical promotions go to
`Outstanding Questions` — never self-answered.

## References

| File | When to load |
|---|---|
| `references/area-spec.md` | Area spec + system-overview templates, per-section and merge rules, harvest interview, bootstrap, rebuild checklist |
| `references/citations.md` | Citation discipline: short8 decision ids in specs and backlog rows |
| `references/promotion.md` | Learnings file template, promotion decision tree, critical promotion format, friction entries |
| `expertise/documentation.md`, `expertise/decisions.md` | Spec craft; what deserves a decision record |
