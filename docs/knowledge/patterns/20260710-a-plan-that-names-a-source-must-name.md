---
type: bee.pattern
title: A plan that names a source must name the reader that can open it
description: A plan that names a source must name the reader that can open it
tags: [process, planning, cells, scope]
timestamp: 2026-07-10
bee:
  id: pattern-20260710-a-plan-that-names-a-source-must-name
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT15", "original feature: evolving-loop"]
  polarity: pitfall
  critical: false
---

# A plan that names a source must name the reader that can open it

A cell mandated markdown frontmatter as a collection source, restricted content reads to the JSON-only
wrappers, and forbade bare filesystem reads in the module — with a two-file scope. No reader existed
for the source it required. The worker had to widen a shared helper outside its declared scope to do
the honest thing rather than game the security check. When a plan names a source, it names the reader
that can open it, or it grants the scope to build one.
