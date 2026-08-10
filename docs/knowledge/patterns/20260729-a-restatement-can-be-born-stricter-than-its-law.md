---
type: bee.pattern
title: A restatement can be born stricter than the law it restates
description: A restatement can be born stricter than the law it restates
tags: [failure, doctrine, concurrency, restatement-drift, always-loaded]
timestamp: 2026-07-29
bee:
  id: pattern-20260729-a-restatement-can-be-born-stricter-than-its-law
  lifecycle: active
  sources: ["original feature: lane-plan-unconditional", docs/history/learnings/20260729-lane-plan-unconditional.md]
  polarity: pitfall
  critical: false
---

# A restatement can be born stricter than the law it restates

A rule stated in one place and operationalized in two will be obeyed as the operational copies
word it — and a copy can carry a precondition the law never had, from birth. No law has to change
for this to bite, which is what distinguishes it from stale-copy drift.

bee's concurrency law required a concurrency plan *before dispatching anything*. Both documents an
agent acts from restated it as "when work is ready **while another feature is already live**". The
case the law existed for — independent ready work and nothing busy — therefore triggered nothing.
The gap was measurable before it was noticed: seventeen lane records on disk against two occasions
of genuinely concurrent features.

Two rules follow:

- **Audit a multi-stated law against its own statement, never against its neighbours.** Agreement
  between restatements proves nothing; they can be uniformly narrower than the rule.
- **Where a tier can be checked, check it.** The same law's cell tier never drifted, because there
  compliance is the output of a required command an agent must argue against — not a sentence it
  must remember.

Mirror of the stale-copy case: a copy that survives a law change keeps teaching the old law; this
one teaches a narrower law that was never enacted. Audit for both directions.

**Full entry:** docs/history/learnings/20260729-lane-plan-unconditional.md
