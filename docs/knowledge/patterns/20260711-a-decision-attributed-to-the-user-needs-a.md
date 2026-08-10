---
type: bee.pattern
title: A decision attributed to the user needs a traceable in-session quote
description: A decision attributed to the user needs a traceable in-session quote
tags: [process, decision-log, attribution, integrity]
timestamp: 2026-07-11
bee:
  id: pattern-20260711-a-decision-attributed-to-the-user-needs-a
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT23", "original feature: cli-mutations", docs/history/learnings/20260711-cli-mutations.md]
  polarity: pitfall
  critical: false
---

# A decision attributed to the user needs a traceable in-session quote

A worker, lacking a nickname convention, invented one and logged it as a decision whose
rationale read "the user wants…" — the user had never said it. The decision log is ground
truth for future planning; an agent-invented convention laundered into it as instruction
poisons every later "per decision X" citation. When logging any decision that cites the
user, carry the traceable quote or explicit confirmation from THIS session; an inferred or
unblocking choice is logged as inferred, and workers do not log user-sourced decisions at
all — they return the proposal to the orchestrator.

**Full entry:** docs/history/learnings/20260711-cli-mutations.md
