# Knowledge one-home — discovery map

## Destination

A locked decision set on how bee keeps every rule in exactly one home
and pushes that rule to the agent at the moment it applies — capture,
plan, cap, and the bee-develops-bee skill/spec sync — then one shaped
feature that implements it.

Spawned: knowledge-one-home — docs/history/knowledge-one-home/CONTEXT.md
(map closed 2026-08-22; remaining fog lines are shaping defaults, not
open decisions).

## Notes

- Symptoms (user, 2026-08-22): a rule change needs 3–4 reminders, each
  reminder finds one more fact; rule conflicts surface after the work
  is done; in bee-on-bee work code changes ship without the matching
  skill/expertise update, or a learning lands in docs/knowledge/ when the
  skill itself should have changed.
- Reality at map time: 255 concepts, 68 active decisions, 47 critical
  patterns. `knowledge context` ranks by vocabulary overlap
  (AUC 0.805) — knowledge is pulled on demand, never pushed at the
  moment it applies.
- User read of the root cause: one rule lives in several files, each
  with its own wording, none marked as the source. Retrieval scatter is
  a consequence, not the cause.
- Inventory (tickets/004, closed): 12 duplicated rules; 2 contradict,
  5 drift, 5 agree. No area declares the code or skill paths it owns —
  `authoritative_for` is a topic string, not a path list. AGENTS.md is
  the de-facto home for doctrine rules; the duplication boundary is
  declared in a knowledge concept but nothing checks it.
- Sibling map test-doctrine already settled per-cell test cadence; that
  rule's scatter was the first observed case.

## Decisions so far

- D1: fix at write time (outbound obligations at capture), not at read
  time — folded into D4
- D4: one home per rule (AGENTS.md for discipline, area spec for
  mechanism), `applied_at` outbound list in the home's frontmatter, bee
  computes the update list, cap refuses untouched entries, rule ids +
  `knowledge check` flags copies, capture stub answers the skill
  question — 27e55095 — tickets/001-outbound-obligations.md
- D2: conflict check runs at plan time, before the gate — folded into D5
- D5: bee derives conflict candidates from the plan (decisions active +
  `applied_at`); each gets a verdict on the plan; gate --merge refuses
  unverdicted candidates; plan-rev bump reruns — efd6cbaa —
  tickets/002-plan-time-conflict-check.md
- D3: code-without-skill detection = plan prediction + cap check against
  an ownership map in area frontmatter; miss refuses cap — 3ea7500a —
  tickets/003-code-without-skill-detector.md

- Stale copy found live: `bee cells cap --help` still says "bee close
  runs commands.test" (retired rule) — the string lives in Rust source
  and generated registry_payload.json, homes text search never visits.

## Not yet specified

- Ownership map + `applied_at` shape: field names, glob vs exact paths,
  and who writes the first map for all 15 areas (agent-suspected).
- Rule id scheme: format, where the id registry lives, how existing 12
  duplicated rules get ids and pointers (migration) (agent-suspected).
- How `knowledge check` recognizes a "rule-shaped block" without an id
  — heuristic or explicit fence marker (agent-suspected).
- Does the history tree (docs/history/) stay exempt from the copy check?
  (agent-suspected; likely yes).

## Out of scope

- Release checklist gaps — already fixed as its own example.
- The `knowledge context` ranking algorithm.
