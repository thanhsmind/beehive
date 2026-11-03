---
type: bee.pattern
title: A control token in free text is injectable by construction; a fail-open contract needs malformed-input rows
description: A control token in free text is injectable by construction; a fail-open contract needs malformed-input rows
tags: [security, prompt-injection, control-channel, fail-open, test-matrix]
timestamp: 2026-07-11
bee:
  id: pattern-20260711-a-control-token-in-free-text-is-injectable
  lifecycle: active
  areas: [hook-runtime]
  sources: ["docs/history/learnings/critical-patterns.md#PAT20", "original feature: model-tier-guard", docs/history/learnings/20260711-model-tier-guard.md]
  polarity: pitfall
  critical: true
---

# A control token in free text is injectable by construction; a fail-open contract needs malformed-input rows

Two design-time rules review had to catch that planning should have owned:
1. **A free-text marker used as an authorization/control signal must be anchored to a
   reserved structural position** (first non-whitespace token of the field), never
   substring/window-searched — quoted or retrieved content containing the marker text
   otherwise satisfies the contract with no decision made. Add "marker embedded
   mid-content → rejected" as a mandatory adversarial test row at plan time.
2. **A stated fail-open/fail-safe contract is not implemented until malformed top-level
   input is a test-row class**: `null`, wrong-type payloads, throwing dependencies.
   Happy-path development never exercises fail-open; the contract crashed (exit 1) on
   `null` stdin despite being explicit in the plan.

**Full entry:** docs/history/learnings/20260711-model-tier-guard.md
