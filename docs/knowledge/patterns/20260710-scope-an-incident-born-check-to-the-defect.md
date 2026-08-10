---
type: bee.pattern
title: "Scope an incident-born check to the defect class, never the first location"
description: "Scope an incident-born check to the defect class, never the first location"
tags: [failure, testing, control-bytes, tooling]
timestamp: 2026-07-10
bee:
  id: pattern-20260710-scope-an-incident-born-check-to-the-defect
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT17", "original feature: evolving-loop slice B"]
  polarity: pitfall
  critical: false
---

# Scope an incident-born check to the defect class, never the first location

The C0 control-byte sweep guarded `templates/**/*.mjs` because that is where the NUL first bit;
the actual cause — raw control bytes decoded from JSON-escaped tool parameters — can hit any
written file, and struck a committed markdown report two commits later (git shows it as binary,
grep goes silent). When mechanizing a check after an incident, ask "what code path produced this
state?" and sweep everything that path can write; fix the instance AND widen the check in the
same cell.
