---
type: bee.pattern
title: Fixture vendored-module lists break on transitive imports
description: Fixture vendored-module lists break on transitive imports
tags: [failure, tests, fixtures, imports]
timestamp: 2026-07-12
bee:
  id: pattern-20260712-fixture-vendored-module-lists-break-on-transitive-imports
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT28", "original feature: dispatcher-unify"]
  polarity: pitfall
  critical: false
---

# Fixture vendored-module lists break on transitive imports

test_bee_write_guard_hook vendors an explicit lib-module list into its fixture repo.
Adding an import to any vendored module (command-registry.mjs → reviews.mjs → cells.mjs)
throws only inside the fixture, and the hook FAILS OPEN — denial tests invert silently.
When a vendored module gains an import, chase the transitive closure into the fixture list.
