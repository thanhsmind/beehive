---
type: bee.pattern
title: Async assertions under a non-awaiting runner pass vacuously
description: Async assertions under a non-awaiting runner pass vacuously
tags: [failure, testing, concurrency, silent-green]
timestamp: 2026-07-14
bee:
  id: pattern-20260714-async-assertions-under-a-non-awaiting-runner-pass
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT35", "original feature: fresh-session-handoff", docs/history/learnings/20260714-fresh-session-handoff.md]
  polarity: pitfall
  critical: false
---

# Async assertions under a non-awaiting runner pass vacuously

`check(fn)` never awaits: an async test body reports PASS immediately and its
assertion failures become unhandled rejections. Concurrency tests belong in a
self-contained child orchestrator (fork racers, assert internally, exit 0/1)
invoked by ONE blocking spawnSync row — and their falsifiability is proven once by
deliberately breaking an invariant and watching the suite go red.

**Full entry:** docs/history/learnings/20260714-fresh-session-handoff.md
