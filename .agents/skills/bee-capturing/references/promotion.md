# Learnings and Promotion

Load during the Compound step. The judgment lives in SKILL.md; the
templates and the promotion bar live here.

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
