# Learnings and Promotion

Load during the Compound step. The judgment lives in SKILL.md; the
templates, the harvest discipline, and the promotion bar live here.

## Harvest Discipline

How a feature's evidence becomes candidate learnings, before any
template applies.

**Read the layer before you read the feature.** Two reads, in this
order: the existing patterns and area concepts for every area this
feature touched, then the feature's own artifacts. Reversed, the
harvest produces entries the layer already holds — and a duplicate is
not free, because it clears the promotion bar on its own merits and
then taxes every future session twice.

**Rank the sources by what they observed.** Review findings and
verification output rank first: an independent reader or a command
saw them. Cell traces and worker reports rank second: an agent
believed them. The plan ranks last: it records intent from before
contact. A finding that exists only in the plan is not a finding, and
a worker's claim that no verification output supports is a lead, not
evidence.

**Mine the project, not the change.** The question is never "what did
this feature do" but "what will still be true for the next feature."
A convention a reviewer enforced, a constraint the work surfaced, a
trap that cost real time — those are learnings. The ids, the line
numbers, the particular bug are the *evidence* for a learning, never
the learning itself.

**Three things never become a record:**

- **A one-off of this change.** If restating it requires naming this
  feature's cells or files to make sense, it has not generalized yet.
- **Generic software advice.** That layer already exists and is
  written once, universally: `.bee/expertise/`. An entry that would
  read as sound advice dropped into any repo is in the wrong layer —
  and the knowledge bundle is where a reader looks for what is true
  *here*.
- **Anything the layer already holds.** When an existing entry is
  merely incomplete, extend that entry. A neighbor filed beside it
  splits the topic in two, and the next reader finds whichever half
  the index shows first.

**Every attributed claim carries its quote.** A learning that rests on
a review finding, a user answer, or a command's output quotes the
words it rests on, verbatim, and names where they came from. A
paraphrase cannot be checked later, and an attribution nobody can
check is how an invented finding enters the layer wearing the
authority of an observed one. Decisions additionally carry their
short8 id (`references/citations.md`) so a supersede sweep can reach
the passage.

**When writes are denied** — a read-only pass, a headless run without
authority, a feature abandoned before its owner returned — produce
the same analysis as a *proposal*: the candidate learnings with their
evidence, in the reply, and change nothing under `docs/knowledge/` or
`docs/history/`. A proposal nobody acted on costs one read. A record
written without authority is load-bearing the moment the next session
loads it.

## Learnings File Template

Path: `docs/history/learnings/YYYYMMDD-<slug>.md`. Slug:
`YYYYMMDD-<primary-topic>-<secondary-topic>`, lowercase hyphens only.
Multiple findings from one feature go in ONE dated file as repeated
Learning sections — never one file per finding. Redact secrets/PII
from every snippet first; a finding that cannot be safely redacted is
dropped and noted.

```markdown
---
date: YYYY-MM-DD
feature: <feature-name>
categories: [pattern, decision, failure]
severity: critical | standard
tags: [tag1, tag2]
---

# Learning: <Concise Title>

**Category:** pattern | decision | failure
**Severity:** critical | standard
**Tags:** [tag1, tag2]
**Applicable-when:** <when future agents should use this>

## What Happened

<2-4 concrete sentences. Name files, commands, tools, or flows.>

## Root Cause

<Why it happened, or why the pattern worked.>

## Recommendation

<Imperative rule: "When X, do Y." Specific enough to act on.>
```

Look for three kinds of finding in the feature's evidence: reusable
patterns that worked (name the pattern, cite where it appeared, state
when to reuse it), important choices (what was decided, what
alternatives existed, what surprised us), and failures (what happened,
the root cause, the check that would have caught it earlier).

## Promotion Decision Tree

1. Seen twice (review finding, user correction, repeated deviation)
   AND it clears all three promotion criteria — multi-feature
   relevance, meaningful waste prevented, generalizable? If not, it
   stays a learning entry.
2. Mechanizable? A grep/lint line in a verify command, a guard, a hook
   denial → **promote as the check**, note the check's location in the
   learnings file, done. File the check as a tiny/small cell if it
   cannot ship in-feature.
3. Not mechanizable (judgment, taste, product intent) → promote as
   prose per the format below.
4. Recurrence escalation: an ALREADY-promoted pattern whose violation
   happens again despite the doc never takes another doc line as the
   answer. Re-ask question 2 against the fresh evidence — can this
   recurrence become a hook denial, guard, doctor check, or test? Yes
   → that durable owner is the promotion, filed as a tiny/small cell
   when it cannot ship in-feature. No → prose survives only with a
   one-line recorded reason (in the learnings file or a decision log
   line) naming why no mechanical owner exists yet.

## Critical Promotion Format

**With a bundle:** author the promoted lesson as a pattern concept
under `docs/knowledge/patterns/`; the generated root index picks it up
on the next `bee knowledge index` — never append to
`critical-patterns.md`, which in a bundled repo is a pointer stub.
**With no bundle:** append this block to
`docs/history/learnings/critical-patterns.md`:

```markdown
## [YYYYMMDD] <Learning Title>
**Category:** pattern | decision | failure
**Feature:** <feature-name>
**Tags:** [tag1, tag2]

<2-4 sentence summary: what happened, root cause, and the future rule.>

**Full entry:** docs/history/learnings/YYYYMMDD-<slug>.md
```

The critical-patterns digest is injected into every session preamble —
every low-signal block taxes every future session. When in doubt, do
not promote.

## Decision Logging

```
bee decisions log --decision "..." --rationale "..." [--alternatives "..."] [--confidence N]
```

- Log only decisions with forward force: conventions adopted,
  approaches rejected with reasons, constraints discovered.
- Include `--alternatives` whenever real alternatives were weighed;
  add `--confidence N` when the evidence was partial.
- To change a past decision: `bee decisions supersede` — never rewrite
  the log.
- The logger rejects secret-like content and injection patterns; do
  not work around a rejection — redact instead.

## Friction Backlog Entry

Unresolved friction (from cell traces or the session) is filed so
grooming can hunt it later:

```
bee backlog add --type friction --severity <P1|P2|P3> --layer <layer> --title "<friction>" --detail "<predicted impact>" --feature <feature>
```

`layer` is optional but valuable: attribute the friction to exactly
one harness layer — `spec` (the task was underspecified), `context`
(the right information wasn't provided), `environment` (tooling/setup
failed), `verification` (feedback was missing or wrong), `state`
(continuity/records failed). Grooming aggregates these to find the
bottleneck layer.
