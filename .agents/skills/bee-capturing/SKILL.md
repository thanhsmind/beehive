---
name: bee-capturing
description: >-
  Capture what settles into durable records — area specs, decisions, learnings. SELF-TRIGGERING: invoke this yourself, unprompted, the moment a rule, behavior, or value settles in discussion — the user never has to ask for knowledge to be recorded. Also use when execution completes, when documenting a screen/API/job/area, and when capturing learnings and decisions at feature close or on intentional abandon.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: Logs decisions, queues captures, and syncs state through the vendored bee binary. The binary is vendored into the repo by onboarding; no Node runtime is involved.
---

# Capturing — keep what settles

Own the **state layer** — `docs/knowledge/` with a bundle, else
`docs/specs/` — where every **area** (screen, API, job, pipeline, CLI
command, process: any unit with observable behavior) has one spec that
is its meaning; the decision log; and the learnings history.
It is a different layer from `.bee/expertise/` (craft that holds in
any repo): read `.bee/expertise/knowledge.md` before standing one up
in a fresh project, and whenever the layer starts duplicating or
piling.

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

Close every task with a capture line or an explicit "nothing settled"
(AGENTS.md) — cell, docs write, quick fix alike; by default smallness is not
the answer. What deserves a record at all, and at what grain:
`.bee/expertise/decisions.md`.

## Scribe — keep area specs current

The bar is the **rebuild test**: given only the spec (its Pointers
section deleted), a stranger rebuilds the same behavior on another
stack. One area = one spec, forever — locate before you create, update
in place, never fork a `-v2`. A flow-shaped behavior — states,
sequence, containment, routing — always gets a Mermaid diagram
section, never prose alone (`references/area-spec.md` "Diagrams"). Outside Pointers, business vocabulary
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
`.bee/expertise/documentation.md`. Template and mechanics:
`references/area-spec.md` ("Area Spec Template", "Merge Rules",
"Harvest Interview", "Rebuild Checklist"). In a bundle repo the CLI
names the write target and emits the frontmatter — ask it; never pick
paths or type frontmatter blocks by eye.

## Compound — close with what you learned

Deferred by design, like review: a green `bee close` records capture as
pending (the capture queue plus uncaptured `behavior_change` cells are
the signal; `bee orient` keeps the reminder) — Compound then runs when
the owner chooses, batching several closed features into one session
when that is cheaper. Deferred is never dropped: the reminder stands
until this runs, and abandoning work with lessons worth keeping still
runs it.

1. Flush the capture queue: `bee capture list`, then oldest-first give
   each stub its full spec merge and `bee capture flush --id <id>
   --into <spec>`. A stub is never dropped or summarized away.
2. When `docs/history/<feature>/promote-proposals.md` exists it is the
   FIRST input, not a second opinion: the close already mined that
   feature's capped traces into a delivery draft, per-area candidate
   bullets, and pattern candidates. Review it, never apply it as
   written. Check every bullet against the bundle (already stated? and
   is it filed under the concept that actually owns it — the area tag
   comes from the work item and is routinely over-broad or wrong) and
   against the shipped source (does the code it describes still exist,
   or did a later port retire it?). Merge what survives; give what does
   not a recorded reason. Either way the loop closes with `bee state
   scribing-run --feature <feature>` — that stamp is the receipt the
   unapplied-proposal reminder reads, and "reviewed, nothing worth
   keeping" is a legitimate result that still owes it.
3. From the feature's artifacts — CONTEXT.md, plan, cell traces,
   worker reports, review findings — write one dated learnings file
   (`references/promotion.md` ("Harvest Discipline", "Learnings File
   Template")); read the touched areas' existing entries first, so the
   harvest extends the layer instead of restating it. Delegate
   the reading to read-only subagents; keep synthesis here. Thin
   evidence means a thin file, never an invented finding.
4. Promote a learning only when it clears all three bars:
   multi-feature relevance, meaningful waste prevented, generalizable.
   Prefer an executable check over prose, and when an already-promoted
   pattern recurs, escalate it to a durable owner — hook, guard,
   doctor check, or test — or record the one-line reason prose stays
   (`references/promotion.md` ("Promotion Decision Tree")).
5. Log the decisions future planning must honor; supersede outdated
   ones, never edit them. File unresolved friction with
   `bee backlog add` so grooming can hunt it later.
6. Housekeeping, warn-never-block: refresh the feedback digest and
   sweep the feature scratch — a failure here is a one-line warning,
   never a delay or reversal of the close.
7. Commit the close's own output as one commit, then record the close
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

`bee-hive` ("Headless") governs; apply mechanical merges and log
verbatim-quotable decisions — harvest questions, ambiguous merges, and
critical promotions defer.

## References

| File | When to load |
|---|---|
| `references/area-spec.md` | Area spec + system-overview templates, per-section and merge rules, harvest interview, bootstrap, rebuild checklist |
| `references/citations.md` | Citation discipline: short8 decision ids in specs and backlog rows |
| `references/promotion.md` | Harvest discipline (source ranking, what never becomes a record, denied writes), learnings file template, promotion decision tree, critical promotion format, friction entries |
| `.bee/expertise/knowledge.md` | The project knowledge layer as a system: what belongs in it, harvesting from finished work, routing, the always-loaded budget, migration rot |
| `.bee/expertise/documentation.md`, `.bee/expertise/decisions.md` | Spec craft; what deserves a decision record |
