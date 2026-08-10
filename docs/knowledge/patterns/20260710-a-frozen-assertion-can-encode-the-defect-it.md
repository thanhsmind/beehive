---
type: bee.pattern
title: "A frozen assertion can encode the defect it guards — the worker must stop, not rewrite"
description: "The worker must stop, not rewrite"
tags: [process, testing, frozen-assertions, review]
timestamp: 2026-07-10
bee:
  id: pattern-20260710-a-frozen-assertion-can-encode-the-defect-it
  lifecycle: active
  decisions: [c45d0fb3, b8fe5c81]
  sources: ["docs/history/learnings/critical-patterns.md#PAT11", "original feature: evolving-loop"]
  polarity: pitfall
  critical: false
---

# A frozen assertion can encode the defect it guards — the worker must stop, not rewrite

Twice, a "frozen" assertion asserted the exact vulnerability under repair — one written by the very
cell tasked with building that boundary, one pinning the defective syntax itself. 93 then 104 green
assertions proved conformance to a wrong spec, not safety. Both were found only because a worker hit
them while fixing a bug and returned `[BLOCKED]` quoting the assertion instead of "correcting" it.
**Keep that escape hatch.** A worker never unfreezes an assertion; the planner does, narrowly, with a
logged decision (`c45d0fb3`, `b8fe5c81`). Corollary: a drift guard that greps a module's own source
pins syntax, not behavior — and pinned syntax can be the bug.
