---
type: bee.pattern
title: Pre-code gates filter spec defects; only diff review catches implementation defects
description: Pre-code gates filter spec defects; only diff review catches implementation defects
tags: [process, review, stage-capability, destructive-code]
timestamp: 2026-07-11
bee:
  id: pattern-20260711-pre-code-gates-filter-spec-defects-only-diff
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT19", "original feature: skill-sync", docs/history/learnings/20260711-skill-sync.md]
  polarity: pitfall
  critical: false
---

# Pre-code gates filter spec defects; only diff review catches implementation defects

Three adversarial panel iterations, an advisor consult, and a 232-check red-first suite all
passed — then five isolated reviewers reading the ACTUAL DIFF found 9 real P1s (three of
them data-loss paths: stale-snapshot deletes, decoy version parsing, case-alias
sync-then-delete). Panels review artifacts → they catch specification defects; tests written
from the same spec share the code's blind spots → green proves conformance, not safety. For
destructive/mirror/guard logic, never skip or shrink the post-implementation isolated
review, and never count pre-code ceremony or test volume as implementation assurance.

**Full entry:** docs/history/learnings/20260711-skill-sync.md
