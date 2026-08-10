---
type: bee.pattern
title: A NUL byte in a source file makes grep silently match nothing
description: A NUL byte in a source file makes grep silently match nothing
tags: [failure, tooling, grep, verification]
timestamp: 2026-07-10
bee:
  id: pattern-20260710-a-nul-byte-in-a-source-file-makes
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT14", "original feature: evolving-loop"]
  polarity: pitfall
  critical: false
---

# A NUL byte in a source file makes grep silently match nothing

`sortKey` joins fields with a NUL separator — a legitimate technique. Side effect: `grep`/`rg` treat
the whole file as **binary and print nothing, not even a zero count**. In a repo whose drift guards
are grep-over-source, this reads as "the symbol is gone". It briefly convinced an orchestrator that a
landed fix had vanished. If a grep over a source file returns empty rather than `0`, check for
control bytes before believing it.
