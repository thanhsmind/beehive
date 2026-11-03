---
type: bee.pattern
title: A cell verify filter is plan text with a blind spot class
description: Widening a closed set (kinds, signals) trips guard tests living in OTHER modules whose names carry no filter hit — a narrow verify filter can pass while a real regression sits in an unrelated module
tags: [verify, cells, guard-tests, plan]
timestamp: 2026-08-29
bee:
  id: pattern-20260829-cell-verify-filter-blind-spot
  lifecycle: active
  areas: [workflow-state]
  sources: ["cell an-1 (advisor-nudge), 2026-08-29 — narrow supervisor filter passed 65/65 while the control_loop prompt-pin guard was red"]
  decisions: ["9e5eda5b"]
  polarity: pitfall
  evidence: observed
---

# A cell verify filter is plan text with a blind spot class

Cell `an-1` scoped its verify to a filter naming the module it touched
(supervisor) and passed 65/65. A real regression sat in `control_loop`'s
prompt-pin guard — a different module, whose test names carried no hit for
the filter — and it was red the whole time.

**The mechanism.** A verify filter narrows by name: kinds, signals, module
paths named in the plan. Widening a closed set (adding a new supervisor
kind, a new signal) is exactly the change class most likely to be caught by
a guard test that asserts the set is closed — and that guard test lives
wherever the closed set itself lives, which is routinely a different module
than the one the plan names. A filter written before the change is scoped
to where the author expected the blast radius, not where a closed-set guard
actually sits.

**The fix, not the workaround.** Closed-set cells now carry the full-suite
verify on purpose — the filter's convenience is not worth the blind spot
class it opens. This is a rule about *closed-set changes specifically*, not
a blanket "always run everything": a filter is still the right call for a
change with no closed-set surface to trip.

## Status

Recorded as a lesson (workflow-state, cell-authoring judgment around verify
scope). No skill text prescribes verify filters, so no skill changed.
