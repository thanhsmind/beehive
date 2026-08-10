---
type: bee.pattern
title: "A reviewer's cited line is a sample of a class — sweep the diff before re-review"
description: Sweep the diff before re-review
tags: [process, review, fix-pass, defect-class]
timestamp: 2026-07-11
bee:
  id: pattern-20260711-a-reviewers-cited-line-is-a-sample-of
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT21", "original feature: grill-deltas", docs/history/learnings/20260711-grill-deltas.md]
  polarity: pitfall
  critical: false
---

# A reviewer's cited line is a sample of a class — sweep the diff before re-review

The external reviewer failed the same one-file diff twice for one defect class (step-4 prose
writing into a file step 5 creates): round 1 cited one line, the fix repaired only that line,
round 2 found the sibling four lines away — present in the round-1 diff all along. When a
review finding names a *class* (temporal contradiction, missing null-check, banned idiom),
the fix pass greps the ENTIRE diff for the class signature and fixes every instance before
re-submitting. One cited line is a sample, not the population; each missed sibling costs a
full review round. Corollary for step-flow prose: an artifact created at step M is never
written by step N<M — use the pin-now/write-later idiom (D-ID pattern).

**Full entry:** docs/history/learnings/20260711-grill-deltas.md
