---
type: bee.pattern
title: Dry-run negative-grep verifies against their own fixtures
description: Dry-run negative-grep verifies against their own fixtures
tags: [failure, verify-authoring, tests]
timestamp: 2026-07-12
bee:
  id: pattern-20260712-dry-run-negative-grep-verifies-against-their-own
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT26", "original feature: bee-footprint"]
  polarity: pitfall
  critical: false
---

# Dry-run negative-grep verifies against their own fixtures

A `! grep <banned>` verify predicate must be run against the tests/fixtures the work itself
will add before it is locked in: a RED-first test proving "<banned> is denied" necessarily
contains the banned string, making the stored verify unsatisfiable on re-run.
