---
date: 2026-07-28
feature: check-filter
categories: [verify-pipeline, performance]
severity: low
tags: [test-harness, scoped-verify, safety-properties]
---

# check-filter — feature close learnings

## What Happened

`BEE_CHECK_ONLY` now scopes a suite to the checks whose names match, in the
shared harness helper the 31 suites import. Substring by default,
`/regex/flags` when wrapped in slashes. Measured live on a 37-check suite:
unfiltered 37 passed; `refus` → 4 passed, 33 skipped; `/refus|corrupt/` → 7
passed, 30 skipped.

## Why the safety properties mattered more than the feature

A scoping tool that can be mistaken for a full run is worse than no tool,
because the mistake is silent and the reader is the delegator deciding whether
a feature ships. Three properties were specified before the mechanism:

1. **Unfiltered is byte-identical** — no skip counter, no extra text, same exit.
2. **Filtered is unmistakable** — skips are printed, counted, and the summary
   names the filter that produced them.
3. **Zero matches is red, never green** — a filter that matches nothing exits
   non-zero with a typed message, because a trivially green run is the exact
   failure mode a scoping flag invites.

All three verified by direct exit-code checks, not by reading the diff.

## Finding

The obvious test suite to try it on had its own local `check()` — 22 of the
repo's suites do — so the first verification showed no effect at all and
looked like a broken feature. It was a wrong test target, not a defect.
*Rule: when a change lands in a shared helper, verify against a file that
actually imports it; "no effect" on a non-adopter proves nothing either way.*
The predicate is exported so those 22 can adopt it incrementally.

## Ran under

R82 (worker ran no suites; MAIN verified at the boundary, green first pass),
R88 (temp-index commit, shared checkout untouched).
