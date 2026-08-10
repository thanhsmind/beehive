---
type: bee.pattern
title: "The bill is turns × prefix: keep the prefix immutable, warm, and lean"
description: "The bill is turns × prefix: keep the prefix immutable, warm, and lean"
tags: [pattern, prompt-caching, prefix-stability, delegation, cost]
timestamp: 2026-07-15
bee:
  id: pattern-20260715-the-bill-is-turns-prefix-keep-the-prefix
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT4", "original feature: session-economics", docs/history/learnings/20260715-cache-economics.md]
  polarity: practice
  critical: false
---

# The bill is turns × prefix: keep the prefix immutable, warm, and lean

Prompt caching is prefix matching: every tool call re-sends the whole conversation and only a
byte-identical prefix bills at ~1/10 price — so a session's true cost is **turns ×
context-per-turn**. A marathon session hit ~99% cached (opus 1.4M new / 120M cached; all
subagents $0.53) by: (1) **never breaking the prefix** — append-only history, no compaction
(compaction rewrites the prefix and re-bills everything; a big context window matters because it
*postpones* it); (2) **staying inside the cache TTL** — continuous rhythm, no long idle gaps
mid-flow; (3) **rule 12 fan-out** — every multi-file gather in a subagent, only digests enter
the orchestrator's prefix, keeping it small AND stable; (4) **fewer, fatter turns** — batch
commands, never re-read, never poll: each avoided call is a full prefix re-bill avoided.
**Rule:** treat the prefix as an invariant and approaching-compaction as a cost cliff — split or
hand off *before* it. Full entry: docs/history/learnings/20260715-cache-economics.md
