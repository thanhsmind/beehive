---
type: bee.pattern
title: An optimization inherits the lifetime of what it optimizes
description: "exec-speed measured, shipped and proved five real speedups in the runtime bee then ran on; five days later that runtime was retired and four of the five have no surviving code at all — the wins were real, the substrate was not, and nothing in the feature's record shows the substrate's remaining lifetime being weighed before the work was scoped."
tags: [performance, sequencing, scope, migration, sunk-cost]
timestamp: 2026-08-06
bee:
  id: pattern-20260806-an-optimization-inherits-the-lifetime-of-what-it-optimizes
  lifecycle: active
  areas: [rust-runtime, workflow-state]
  decisions: [44242df8 (exec-speed overhaul approved — group A runtime performance plus group B doctrine diet — locked as D1-D11 2026-07-30)]
  sources: ["exec-speed cells es-1..es-5 (traces in .bee/cells/, docs/history/exec-speed/CONTEXT.md, capped 2026-07-30 — measured before/after numbers recorded on every trace)", js-parity-cleanup (the last structural traces of the retired runtime removed 2026-08-04; docs/history/js-parity-cleanup/CONTEXT.md), "verified 2026-08-06: no .mjs entry point survives in the source tree, the test runner sets no compile-cache variable, reservations accepts one path per call, and the state-sync hook rebuilds unconditionally"]
  polarity: pitfall
  critical: false
---

# An optimization inherits the lifetime of what it optimizes

Performance work is unusually convincing: it comes with numbers, the numbers are
real, and every cell caps green. None of that says anything about how long the
thing being made faster will exist. A speedup's value is the saved time
multiplied by the remaining life of its substrate, and only one of those two
factors gets measured.

The instance: a feature profiled the runtime bee ran on, found that process
startup dominated roughly ninety percent of small-command wall time, and shipped
five measured wins — a compile cache and lazy-import split at the entry point, a
cache for test-suite children, hook fast paths, a batched reservation call, and a
skip for an unchanged store scan. Every number was honest: hook paths went from
about 77ms to about 58ms; a repeated check went from 216ms to 101ms. Five days
later the runtime those numbers belonged to was retired in favour of a port that
had been under way on its own branch, and today four of the five have no
surviving code at all — no entry point of that kind exists, the current test
runner sets no such variable, reservations takes one path per call, and the
state-sync hook rebuilds every time. The fifth survives only structurally, and in
a different shape.

Nothing in the feature's own record shows the overlap being weighed. That absence
is the finding: the work was well-executed and the question was never asked.

## The rule

- Before scoping performance work, ask what replaces the component and when.
  "Nothing planned" is a fine answer; "there is a port on a branch" is a
  different answer, and it changes the scope rather than cancelling it.
- Prefer optimizations that live in the layer that survives. Of this feature's
  eleven decisions, the six that changed how the team works — lane discipline,
  worker startup ceremony, when a judge runs, what a report file is for — cost
  nothing to carry across the port and are still in force; the five that changed
  the substrate went with it.
- Measure the substrate's expected remaining life the way you measure the
  speedup. An unqualified "saves 20ms per call" is half a claim.
- When the substrate does get replaced, say so where the work was recorded.
  Retired performance work that is never marked retired reads later as
  behavior — someone will eventually search for a compile cache that no code has
  mentioned for months.
