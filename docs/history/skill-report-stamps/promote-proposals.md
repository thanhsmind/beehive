promote proposal for work item "skill-report-stamps" (docs/history/skill-report-stamps/CONTEXT.md) — 3 capped cell(s): srs-1, srs-2, srs-3
anchor: history — docs/history/skill-report-stamps/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/skill-report-stamps/delivery.md

---
type: bee.delivery
title: skill-report-stamps — delivery
description: "Delivery record proposed by bee knowledge promote for work item skill-report-stamps: 3 capped cell(s), 10 recorded deviation(s)."
timestamp: 2026-08-25
bee:
  id: skill-report-stamps-delivery
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [docs/history/skill-report-stamps/CONTEXT.md]
  sources: [docs/history/skill-report-stamps/CONTEXT.md, .bee/cells/srs-1.json, .bee/cells/srs-2.json, .bee/cells/srs-3.json]
---

# skill-report-stamps — Delivery

## What shipped

- **srs-1** — bee-reviewing ends on a required per-severity/per-axis count line with a verbatim empty case, plus a Boundaries block routing every excluded concern (1 file(s) changed)
- **srs-2** — bee-grooming carries the one-line finding stamp with a closed tag list, the required count line with its verbatim empty case, and an honesty boundary on uncounted savings figures (1 file(s) changed)
- **srs-3** — bee-capturing's close line is now a literal template with the verbatim nothing-settled case (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **srs-1** — `rg -n 'findings —|Boundaries' skills/bee-reviewing/SKILL.md`
- **srs-2** — `rg -n 'dead:|Honesty|Lean already' skills/bee-grooming/SKILL.md`
- **srs-3** — `rg -n 'captured:|nothing settled' skills/bee-capturing/SKILL.md`

## Deviations

- **srs-1** — First commit swept a sibling workers staged skills/bee-capturing/SKILL.md via the shared index; reset --soft and re-committed with --only, restoring that file to staged-uncommitted.
- **srs-1** — Capped with --sync-ack: SYNC_DOOR compares skill names to dirty paths and also counts two live sibling workers files.
- **srs-1** — sync-ack: SYNC_DOOR compares affects_skills names (bee-reviewing) against dirty paths (skills/bee-reviewing/SKILL.md) so it can never match, and it also attributes two live sibling workers files (bee-capturing, bee-grooming) to this cell. This cell touched only skills/bee-reviewing/SKILL.md; commit 93d799e is path-scoped to that one file. Widening affects_skills would falsely claim sibling work.
- **srs-2** — The file content was swept into sibling commit e10e720f (cell srs-3) by a concurrent whole-index commit; this cell ships empty marker commit 421ebf97 carrying the cell: srs-2 trailer
- **srs-2** — references/grooming-reference.md still tells the rendered proposal report to end on one top recommendation; the new count line governs the round reported in conversation — out of this cell's file scope, flagged for the orchestrator
- **srs-2** — SYNC_DOOR acked: it compares skill names against dirty paths
- **srs-2** — sync-ack: SYNC_DOOR compares affects_skills names (bee-grooming) against dirty paths (skills/bee-grooming/SKILL.md), so the two can never match; the prediction is correct in content and this cell touched only that one file.
- **srs-3** — A concurrent sibling worker committed the whole git index while my hunk was staged, so the content shipped inside commit 9bccd0b6 (cell srs-1); I added an empty marker commit e10e720f carrying the cell: srs-3 trailer rather than rewrite another worker's commit.
- **srs-3** — Capped with --sync-ack: the sync door also saw wk-srs-2's in-flight bee-grooming edit, which this cell did not touch.
- **srs-3** — sync-ack: Concurrent-worker race: my staged bee-capturing hunk was swallowed by sibling commit 9bccd0b6 (srs-1), so my own commit e10e720f is an empty trailer marker; the door also sees wk-srs-2's in-flight skills/bee-grooming/SKILL.md edit, which this cell never touched. Prediction bee-capturing is correct in content.

## Provenance

Proposed by `bee knowledge promote --work skill-report-stamps` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/skill-report-stamps/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "skill-report-stamps" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-25T08:31:12.771Z), the work item declares no bee.areas.

area doctrine-layer:
  - [srs-1] bee-reviewing ends on a required per-severity/per-axis count line with a verbatim empty case, plus a Boundaries block routing every excluded concern — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/srs-1.json)
  - [srs-2] bee-grooming carries the one-line finding stamp with a closed tag list, the required count line with its verbatim empty case, and an honesty boundary on uncounted savings figures — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/srs-2.json)
  - [srs-3] bee-capturing's close line is now a literal template with the verbatim nothing-settled case — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/srs-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell srs-1 — save as docs/knowledge/patterns/skill-report-stamps-srs-1-pitfall.md

---
type: bee.pattern
title: skill-report-stamps cell srs-1 — pitfall candidate
description: "Pitfall candidate mined from cell srs-1's capped trace: First commit swept a sibling workers staged skills/bee-capturing/SKILL.md via the shared index; reset --soft and re-committed with --only, restoring that file …"
timestamp: 2026-08-25
bee:
  id: skill-report-stamps-srs-1-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/srs-1.json]
  polarity: pitfall
---

# skill-report-stamps cell srs-1 — pitfall candidate

## What the cell did

bee-reviewing ends on a required per-severity/per-axis count line with a verbatim empty case, plus a Boundaries block routing every excluded concern

## Recorded evidence (verbatim from .bee/cells/srs-1.json)

- **deviation** — First commit swept a sibling workers staged skills/bee-capturing/SKILL.md via the shared index; reset --soft and re-committed with --only, restoring that file to staged-uncommitted.
- **deviation** — Capped with --sync-ack: SYNC_DOOR compares skill names to dirty paths and also counts two live sibling workers files.
- **deviation** — sync-ack: SYNC_DOOR compares affects_skills names (bee-reviewing) against dirty paths (skills/bee-reviewing/SKILL.md) so it can never match, and it also attributes two live sibling workers files (bee-capturing, bee-grooming) to this cell. This cell touched only skills/bee-reviewing/SKILL.md; commit 93d799e is path-scoped to that one file. Widening affects_skills would falsely claim sibling work.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell srs-2 — save as docs/knowledge/patterns/skill-report-stamps-srs-2-pitfall.md

---
type: bee.pattern
title: skill-report-stamps cell srs-2 — pitfall candidate
description: "Pitfall candidate mined from cell srs-2's capped trace: The file content was swept into sibling commit e10e720f (cell srs-3) by a concurrent whole-index commit; this cell ships empty marker commit 421ebf97 carrying …"
timestamp: 2026-08-25
bee:
  id: skill-report-stamps-srs-2-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/srs-2.json]
  polarity: pitfall
---

# skill-report-stamps cell srs-2 — pitfall candidate

## What the cell did

bee-grooming carries the one-line finding stamp with a closed tag list, the required count line with its verbatim empty case, and an honesty boundary on uncounted savings figures

## Recorded evidence (verbatim from .bee/cells/srs-2.json)

- **deviation** — The file content was swept into sibling commit e10e720f (cell srs-3) by a concurrent whole-index commit; this cell ships empty marker commit 421ebf97 carrying the cell: srs-2 trailer
- **deviation** — references/grooming-reference.md still tells the rendered proposal report to end on one top recommendation; the new count line governs the round reported in conversation — out of this cell's file scope, flagged for the orchestrator
- **deviation** — SYNC_DOOR acked: it compares skill names against dirty paths
- **deviation** — sync-ack: SYNC_DOOR compares affects_skills names (bee-grooming) against dirty paths (skills/bee-grooming/SKILL.md), so the two can never match; the prediction is correct in content and this cell touched only that one file.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell srs-3 — save as docs/knowledge/patterns/skill-report-stamps-srs-3-pitfall.md

---
type: bee.pattern
title: skill-report-stamps cell srs-3 — pitfall candidate
description: "Pitfall candidate mined from cell srs-3's capped trace: A concurrent sibling worker committed the whole git index while my hunk was staged, so the content shipped inside commit 9bccd0b6 (cell srs-1); I added an empt…"
timestamp: 2026-08-25
bee:
  id: skill-report-stamps-srs-3-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/srs-3.json]
  polarity: pitfall
---

# skill-report-stamps cell srs-3 — pitfall candidate

## What the cell did

bee-capturing's close line is now a literal template with the verbatim nothing-settled case

## Recorded evidence (verbatim from .bee/cells/srs-3.json)

- **deviation** — A concurrent sibling worker committed the whole git index while my hunk was staged, so the content shipped inside commit 9bccd0b6 (cell srs-1); I added an empty marker commit e10e720f carrying the cell: srs-3 trailer rather than rewrite another worker's commit.
- **deviation** — Capped with --sync-ack: the sync door also saw wk-srs-2's in-flight bee-grooming edit, which this cell did not touch.
- **deviation** — sync-ack: Concurrent-worker race: my staged bee-capturing hunk was swallowed by sibling commit 9bccd0b6 (srs-1), so my own commit e10e720f is an empty trailer marker; the door also sees wk-srs-2's in-flight skills/bee-grooming/SKILL.md edit, which this cell never touched. Prediction bee-capturing is correct in content.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 3 area bullet(s), 3 pattern candidate(s), 0 file(s) written.