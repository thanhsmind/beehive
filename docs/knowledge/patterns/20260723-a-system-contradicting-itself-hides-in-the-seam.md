---
type: bee.pattern
title: "A system contradicting itself hides in the seam, because each half passes inspection alone"
description: "One mechanism produces what another must consume, reclaim or check — and the contradiction survives every green run, because no unit test owns a handoff."
tags: [process, guards, seams, testing, self-contradiction]
timestamp: 2026-07-23
bee:
  id: pattern-20260723-a-system-contradicting-itself-hides-in-the-seam
  lifecycle: active
  sources: ["issues-46-53 cells i-2, i-3 (the write guard directing writes the sweeper could not see; the merge refusal blaming an unchangeable branch; traces in `.bee/cells/`, 2026-07-23)", docs/history/learnings/20260723-four-of-seven-bug-reports-named-the-wrong-cause.md]
  polarity: pitfall
  critical: false
---

# A system contradicting itself hides in the seam, because each half passes inspection alone

Two live defects from one batch, same shape:

- A write guard **refused** scratch-shaped writes elsewhere and told the author to write into the
  scratch root. The sweeper that reclaims that scratch listed only directories — so every plain file
  written **exactly where the guard sent it** was unreachable by every flag, including the one
  documented as clearing everything. 58 of 76 entries were unsweepable.
- A merge refusal derived its expected branch from a *mutable* feature field, then named the
  **branch** as the problem — a value that is correct, fixed at creation, and the one thing the
  operator must not change. The refusal pointed at a dead end.

Neither is a bug in either half. Read alone, the guard is right and the sweeper is right; the refusal
is right and the field is right. **The defect lives in the seam, and nothing owns the seam.**

**Why it survives every green run:** tests and reviews are organised around units. A contradiction
between two units is nobody's unit. Worse, it is invisible *because* both halves independently pass
inspection — a reviewer who checks each one carefully finds nothing, and concludes correctly that
each is fine.

**Rule.** When one mechanism decides where things go, specify the mechanism that reclaims, consumes
or checks them **against that same shape** — and assert the pair together, not each half alone. The
question to ask of any guard, emitter or router: *what does this produce, and who consumes it?* If
the answer names another mechanism, there must be a test that exercises the **handoff**, because
neither unit's own tests can see it.

**Corollary — state a rule as the property it protects, not the proxy you happened to check.** The
cleanup rule read "strictly post-commit", a proxy for "nothing would be lost". The proxy then
generalised to a path where no commit is made but nothing could be lost either, and a flag the caller
passed evaporated: exit zero, nothing done, nothing said. A proxy is safe only where it happens to
coincide with the property, and nothing marks the boundary where it stops.
