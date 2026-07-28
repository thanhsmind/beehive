---
date: 2026-07-28
feature: explicit-triage
categories: [workflow-state, cli, orchestration]
severity: medium
tags: [triage, route-record, census, parallel]
---

# explicit-triage — feature close learnings

## What Happened

Closed the last honest gap from the ak comparison: bee's triage counted flags
in the model's head and left no trace ("đoán lane"). Now `bee state route
--set` persists a validated route record — `class | lane | flags | files` —
on the feature's workflow record (enum-refused, free prose rejected), surfaced
in `status --json` and one preamble line, rewritten in place by re-lane, with
a soft claim warning as the safety net. The hive law says: count, then record,
same turn. Dogfooded at close: this feature's own route record was the verb's
first live write. The status-line half of the original comparison needed no
work — `[DONE]/[BLOCKED]/[HANDOFF]/[NOOP]` predates.

Wave 1 ran et-1 ∥ et-2 parallel (code vs law text, wave-barrier); a fix-first
cell (et-4) landed mid-feature when the wave-close goal-check exposed 3 red
census checks in `test_misc`.

## Findings

1. **Thin-body migrations owe the census a forwarding address.** Three census
   checks greped canonical contract prose at its old body locations; the diet
   moved the prose to references and one copy was consolidated away. The census
   had been failing quietly since diet-4 — twice triaged as "pre-existing
   unrelated" by workers whose A/B stash checks confirmed exactly that and
   moved on. *Rule: when instruction text moves, run the census (test_misc) in
   the migration cell's verify — prose relocation is a census event; and a
   "pre-existing failure" seen twice is a fix-first cell, not a footnote.*
2. **Census fixes get mutation-tested.** et-4 re-anchored the checks and then
   deliberately broke each asserted property (order, phrasing, heading) to
   prove the re-anchored checks still bite before reverting. *Rule: a
   re-anchored guard ships with its own red proof.*
3. **Barrier timing lesson applied and validated.** The barrier was paid
   immediately after wave 1 (foundation-fixes finding 4); the only friction
   left was an uncommitted barrier output, folded into its own chore commit.
4. **The write-guard's shell parsing fights compound commands** (`for` loops,
   `||` in decision text misread as targets) — third occurrence today. Filed
   as friction; workaround is splitting commands, but the guard should parse
   more precisely.

## Verification

Route verb: 329 + 44 green (11 new hermetic behavioral checks: enum refusals
write nothing, round-trip, exact preamble format, re-lane rewrites in place,
claim warns once). Census: 117/0. Fences green. Live dogfood:
`Route: class=feature lane=standard flags=1 [multi-domain] files=7`.
