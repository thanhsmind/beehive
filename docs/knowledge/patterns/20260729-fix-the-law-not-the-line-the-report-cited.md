---
type: bee.pattern
title: "Fix the law, not the line the report cited"
description: "Fix the law, not the line the report cited"
tags: [failure, doctrine, knowledge-layer, retirement, review-followthrough]
timestamp: 2026-07-29
bee:
  id: pattern-20260729-fix-the-law-not-the-line-the-report-cited
  lifecycle: active
  sources: ["original feature: budget-fence-removal", docs/history/learnings/20260729-budget-fence-removal.md]
  polarity: pitfall
  critical: true
---

# Fix the law, not the line the report cited

A document states the same rule in several registers — a Business Rule, an Edge Cases bullet, a
Pointers line, a frontmatter `decisions:` citation. A report or cell action quotes **one** of them,
because one anchor was enough to prove the defect. Patching that anchor leaves the rule alive
everywhere else, and the reader cannot tell which statement is current.

Retiring `budget-fence-removal`'s size law hit this four times in one concept. The cell fixed the
Pointers line its action named; the scribing sync found the Business Rule four sections above; a
compounding analyst found the Edge Cases bullet still live in HEAD, plus a fourth statement in a
neighbouring concept. **The failure mode recurred inside its own fix, twice.**

When retiring a rule, grep the concept — and its siblings — for the rule's **substance**, not for
the anchor a report happened to quote. A cited line is a sample of a class. This is the
knowledge-layer twin of "a reviewer's cited line is a sample of a class — sweep the diff before
re-review".

Cost is asymmetric: `docs/knowledge/` is what `bee knowledge context` feeds to future planning
sessions, so a stale concept is read as ground truth by every later feature.

**Full entry:** docs/history/learnings/20260729-budget-fence-removal.md
