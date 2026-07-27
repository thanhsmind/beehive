---
type: bee.pattern
title: A cell scheduler that serializes on declared files alone misses shared regen side-effects
description: A cell scheduler that serializes on declared files alone misses shared regen side-effects
tags: [scheduling, regen-obligation, race-condition, cell-authoring]
timestamp: 2026-07-24
bee:
  id: pattern-20260724-scheduler-blind-to-regen-side-effects
  lifecycle: active
  sources: ["worktree-concurrency-guard cells wcg-1/wcg-2/wcg-3 (capped, trace.friction each independently names the same shape); reports/validation-e2e3.md (plan-checker BLOCKER: wcg-2/wcg-3 both silently rewrite release-manifest.json and .bee/onboarding.json via regen_obligation_ack, only wcg-3 declared those paths, risking a lost update if scheduled as one parallel wave)"]
  polarity: pitfall
  critical: true
---

# A cell scheduler that serializes on declared files alone misses shared regen side-effects

A cell that touches a hashed root (a lib file, a hook, a mirrored template)
carries a regen obligation — running a manifest/ledger regen command as part
of its own action, separate from the files it edits directly. That regen
command rewrites a shared artifact (a release manifest, an onboarding ledger)
whether or not the cell author listed that artifact in the cell's own
declared `files`. A scheduler that computes parallel-safe waves purely from
declared `files` sees no conflict between two such cells — each looks like it
only touches its own lib file and hook — and happily schedules them into the
same wave. Both workers then rewrite the exact same shared artifact at the
same time: a lost-update race the scheduler cannot see, because the collision
lives in an undeclared side-effect of the regen step, not in the cell's
stated scope. This shape recurred three separate times inside one three-cell
feature, independently, before an explicitly dispatched adversarial reviewer
caught the parallel-wave instance as a blocker.

**The tell:** two or more cells in the same wave each carry a
`regen_obligation_ack` or otherwise run a manifest/ledger regen command, but
their declared `files` lists don't all name the regen target(s) those
commands actually rewrite.

**The fix that shipped this time:** declare the shared regen artifacts (the
manifest, the ledger) in every cell's `files` that will actually touch them
via its regen step, not only the cell whose primary edit happens to be in the
same commit as the regen. Once declared, the scheduler's existing file-overlap
serialization does the rest — no new mechanism was needed, only complete
declaration.

**The generalizable gap this exposes:** the fix above is a per-feature
workaround, applied by a human/reviewer noticing the omission. The scheduler
itself has no way to independently know which files a cell's regen chain will
touch — it trusts the cell author's declared `files` completely. A
mechanized fix (not yet built) would have the scheduler consult a static or
derived map of "regen command → files it writes" for any cell whose files or
lane implies a regen obligation, and treat those as implicit shared
dependencies for scheduling purposes even when the cell author forgot to
declare them.
