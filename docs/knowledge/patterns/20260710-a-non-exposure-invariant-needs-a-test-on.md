---
type: bee.pattern
title: A non-exposure invariant needs a test on every output surface it crosses
description: A non-exposure invariant needs a test on every output surface it crosses
tags: [security, security, boundaries, testing]
timestamp: 2026-07-10
bee:
  id: pattern-20260710-a-non-exposure-invariant-needs-a-test-on
  lifecycle: active
  areas: [verify-pipeline]
  sources: ["docs/history/learnings/critical-patterns.md#PAT16", "original feature: evolving-loop slice B"]
  polarity: pitfall
  critical: true
---

# A non-exposure invariant needs a test on every output surface it crosses

"Never render/emit X" written in a plan or SKILL.md is a request, not an enforcement. The stripped
cluster key was banned in prose at two altitudes and still reached the consuming agent via
`rank --json` spreading `...cluster`. When a value's absence from an output is a security
property, assert that absence with a test at EVERY surface the value crosses (lib return, CLI
output, prompt render) — the same root cause recurs one layer down from wherever you fixed it.
