---
type: bee.pattern
title: Clearing a red by widening the threshold is not the same act as correcting what is measured — prove which one you did
description: "Both clear the red, both read identically in a commit summary, and only one leaves the guard able to detect what it exists to detect. A negative control is what separates them."
tags: [process, gates, metrics, gaming, negative-control]
timestamp: 2026-07-23
bee:
  id: pattern-20260723-clearing-a-red-by-widening-the-threshold
  lifecycle: active
  sources: ["okf-integration-close-f4 cell f4-7 (the drift telemetry punished a migrated area for growing; denominator corrected, band untouched, proven by a two-directional negative control; trace in `.bee/cells/`, 2026-07-23)", docs/history/learnings/20260723-a-metric-that-punishes-growth-and-a-spec-that-was-wrong.md]
  polarity: practice
  critical: false
---

# Clearing a red by widening the threshold is not the same act as correcting what is measured

A guard you own goes red. You have write access to the guard. There are now two ways out that both
end with a green chain and a commit summary reading "fixed the check":

1. **Loosen what it takes to pass** — widen the band, lower the minimum sample count, exclude the
   offending subject from the population, edit the pinned expectation.
2. **Correct what is being measured** — the guard is computing the wrong quantity, and computing the
   right one clears the red as a side effect.

Nothing in the red distinguishes them. The distinction lives entirely in whether the guard can still
catch the thing it exists to catch afterwards — and that is not visible in the diff.

**The rule: before touching a guard that has gone red, write down which of the two you are doing,
then PROVE it with a negative control — construct the failure the guard exists to catch and show it
still fires after your change.** Without that control, "I corrected the measurement" is
indistinguishable from "I widened the threshold", including to your future self.

**The case that produced this.** A drift metric divided anchors found in a *frozen historical
source* by the count of concept *files in a directory*. An area went red with its actual coverage
perfect — it had simply grown two concepts of new truth, which can never own a historical anchor. The
denominator could rise; the numerator physically could not. Four one-line edits would each have
cleared it and destroyed the signal. The fix changed the denominator to count only concepts that own
an anchor, leaving band, sample count, pins and population verifiably untouched.

The negative control did more than certify the fix: **it proved the old metric had been blind all
along.** An area with every anchor dumped into a single concept — the exact fault the ratio exists to
detect — scored comfortably *in band* under the file-counting denominator, because the empty sibling
files diluted it. The correction did not weaken the guard; it restored a detection that had been
silently absent.

**Secondary signal, useful but never sufficient:** a corrected measurement usually makes the
population *tighter*; a loosened threshold always makes it looser. Check it, but do not accept it in
place of the control.

**Corollary for the frozen/live asymmetry that causes this class:** when a ratio's numerator comes
from a frozen artifact and its denominator from the live tree, ask whether one side can move while
the other cannot. If it can, the metric will eventually punish healthy growth — and a band
calibrated on today's coincidence between the two populations will hide that until the first growth
event.
