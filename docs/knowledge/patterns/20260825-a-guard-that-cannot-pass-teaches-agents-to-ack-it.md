---
type: bee.pattern
title: A guard that cannot pass teaches agents to ack it
description: A guard that cannot pass teaches agents to ack it
tags: [failure, guards, doors, signal-quality]
timestamp: 2026-08-25
bee:
  id: pattern-20260825-a-guard-that-cannot-pass-teaches-agents-to-ack-it
  lifecycle: active
  areas: [workflow-state, doctrine-layer]
  sources: ["skill-report-stamps wave (cells srs-1, srs-2, srs-3 — all three capped with --sync-ack, 2026-08-25)", "wave-guard-gaps cell wgg-1 (decision f8be49c9 — the fix moved enforcement to authoring time)", "capture-queue measurement 2026-08-25: 157 pending stubs, 155 source touches-sweep, 0 settlements"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "count how often a door is answered with its own escape hatch rather than by satisfying it — three of three caps in one wave passing --sync-ack, or a blocker that has fired every session for weeks, is the measurement"
---

# A guard that cannot pass teaches agents to ack it

Twice in one day, in two unrelated parts of the system, the same thing happened.

**The cap-time sync door.** `affects_skills` holds repo-relative paths. Nothing validated or
documented that, so an orchestrator wrote skill *names*, and the door refused the cap with
`predicted but untouched: bee-reviewing` — a message that reads like a broken comparison rather
than a bad input. All three workers in the wave reached the same conclusion independently ("the
door can never match"), and all three capped with `--sync-ack`. Not one of them was being lazy;
each wrote an honest, reasoned justification. The door had taught them that acking was the normal
way to answer it.

**The capture-queue blocker.** `bee close` and every session preamble report `capture queue
OVERDUE — flush before new work`. Measured: 157 pending stubs, of which 155 were `source:
touches-sweep` citation rows — 116 belonging to one in-flight feature owned by a different live
session — and **zero** were settlements awaiting a spec merge. The blocker is unactionable by the
session that reads it, and it has been firing continuously. A genuine pending settlement would sit
invisible among 155 rows nobody reads.

The shared shape: **a door whose refusal is not the reader's fault, and whose escape hatch is
cheaper than its remedy, stops being a guard and becomes a toll.** The escape gets paid
reflexively. Worse, the ack is *recorded*, so the audit trail shows a considered decision every
time — which is exactly what makes the erosion invisible in review.

**The rule:** when a door is answered by its own escape hatch more than occasionally, that is a
defect in the door, not in the agents. Count the acks. Three of three caps in one wave, or a
blocker that has fired every session for a month, is the measurement — and the fix is at the
door, never a reminder telling agents to try harder.

Two fixes that work, both used here:

- **Move enforcement to the moment the mistake is made.** The `affects_skills` format now refuses
  at `cells add`/`update`, naming the exact `skills/<name>/SKILL.md` replacement, instead of
  detonating at cap where the refusal reads as a system fault (decision f8be49c9).
- **Do not pool a noisy signal with a scarce one.** Citation bookkeeping and real settlements
  share a queue and a threshold, so the loud one buries the quiet one. Separate the counts, or
  scope the blocker to what the reader can actually act on.

**The tell to watch for:** a refusal message that names something the reader did not do, or cannot
fix from where they stand. That message will be acked, and everything downstream of it will
quietly stop being checked.
