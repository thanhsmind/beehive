---
type: bee.pattern
title: A cell that changes a shared mutator surface re-runs the sibling suites of that surface — its own new suite is not enough
description: "A cell touching a shared guard/dispatch surface must re-run every sibling suite that exercises it, not just its own new suite"
tags: [failure, verify-authoring, cross-cell, guards, silent-rot]
timestamp: 2026-07-20
bee:
  id: pattern-20260720-a-cell-that-changes-a-shared-mutator-surface
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT39", "original feature: multi-session-hardening"]
  polarity: pitfall
  critical: true
---

# A cell that changes a shared mutator surface re-runs the sibling suites of that surface — its own new suite is not enough

msh-2 shipped a claim-race suite, green at cap. Two cells later msh-4 added an ownership guard
to the same mutators (block/reopen); msh-4's verify ran only its own suites, so the msh-2 suite
silently went red and sat broken for two cells — discovered only when msh-6 wired the new suites
into the standing `commands.test` chain (the wiring step caught the feature's first real
cross-cell interaction bug at close instead of after ship). **Two rules:** (1) a cell that
changes shared guard/ownership/dispatch logic lists, in its own verify, every EXISTING suite
that exercises the same surface — grep the tests for the functions it edits at plan time;
(2) new suites are wired into the standing chain INSIDE the feature, never left as cap-only
artifacts — a suite that runs only at its birth cell's cap is orphaned from regression the day
that cell closes, and the wiring run itself is a detector (concurrency of truths, not ceremony).
