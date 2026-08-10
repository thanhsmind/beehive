---
type: bee.pattern
title: "A tolerant regression net, frozen green BEFORE the edit, is what makes a load-bearing function safe to change"
description: "A tolerant regression net, frozen green BEFORE the edit, is what makes a load-bearing function safe to change"
tags: [process, test-first, regression-net, resolver, blast-radius, additive-change]
timestamp: 2026-07-16
bee:
  id: pattern-20260716-a-tolerant-regression-net-frozen-green-before-the
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT2", "original feature: worktree-feature-parallelism"]
  polarity: practice
  critical: false
---

# A tolerant regression net, frozen green BEFORE the edit, is what makes a load-bearing function safe to change

`resolveRoots` (two copies: throwing lib + non-throwing hook adapter) is the highest-blast-radius
function in the repo — every write-guard call resolves through it, and a logic bug that DENIES can
lock the session out of its own fix. It was changed safely by writing a P40 byte-for-byte
regression test FIRST, running it GREEN against the unmodified code, THEN making the edit purely
additive (compute `mainRoot`, consult the grant registry, add `{id,mainRoot,worktreeRoot}` fields;
the no-grant path returns exactly today's `storeRoot`). The net stayed 6/6 green after — that is
the proof of no regression, not an assertion. **Two rules:** (1) freeze a load-bearing function's
current behavior in a regression net and see it green before you touch it; (2) make the net
**tolerant of NEW fields** (pin the fields that exist, never assert the absence of others) so an
additive change stays compatible — a strict deep-equal net would have failed on the harmless new
fields and taught you nothing about real regressions.
