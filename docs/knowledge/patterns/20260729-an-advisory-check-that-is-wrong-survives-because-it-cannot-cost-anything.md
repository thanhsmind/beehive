---
type: bee.pattern
title: "An advisory check that is wrong survives, because it cannot cost anything"
description: "A false-positive matcher reported a reachable pointer as missing for months; two workers in one session read the warning, wrote it off as pre-existing noise, and moved on — a blocking check with the same defect would have been fixed the day it landed."
tags: [verification, checks, advisory, false-positive, gates, tick-contract-inline]
timestamp: 2026-07-29
bee:
  id: pattern-20260729-an-advisory-check-that-is-wrong-survives-because-it-cannot-cost-anything
  lifecycle: active
  sources: ["tick-contract-inline cell tci-2 (pointer matcher fix, scripts/skill_lint.mjs, commit d3a3cbde, 2026-07-29)", "tick-contract-inline cell tci-3 (blocking reachability suite, scripts/tests/test_always_loaded_rules.mjs, commit 69de4c1d)", "derived-check-hardening cell dch-7 and cell dch-1 worker reports (both recorded the same warning as 'pre-existing advisory, unrelated', 2026-07-29)", docs/history/tick-contract-inline/CONTEXT.md T4/T5, docs/history/learnings/20260729-tick-contract-inline.md N2]
  polarity: pitfall
  critical: true
---

# An advisory check that is wrong survives, because it cannot cost anything

The pointer check in `scripts/skill_lint.mjs` searched for a parenthetical containing
exactly one quoted heading. The line it was checking reads
`("Silent Bookkeeping", "Progress ticks")` — a pointer any reader follows without
trouble. So the check reported `bee-hive/SKILL.md has no pointer to "Progress ticks"`,
and had been reporting it for as long as that line carried two headings.

Two workers **in the same session** hit the warning. Both recorded it as
*"pre-existing advisory, unrelated"* and moved on. Both were individually right: it was
pre-existing, it was unrelated to their cell, and the lint always exits 0, so nothing
stopped them.

That is the whole mechanism. A false positive in a **blocking** check gets fixed the day
it lands, because it stops work — the defect and the pressure to fix it arrive together.
A false positive in an **advisory** check is stable. It costs nothing, so nobody pays it
down, and every reader who steps over it learns that this check's output is noise. By the
time it reports something real, the audience is gone.

The compounding damage is not the one missed finding. It is that the check's *true*
positives now cost the same to ignore as its false ones, so the whole output degrades to
a line people scroll past. In this case the rule the pointer guarded — a per-step
communication contract that applies every turn — sat unreachable in an on-demand
reference the entire time, with the warning firing continuously.

**Rule.** A check has two independent properties: whether it is correct, and whether it
can cost anything. The dangerous quadrant is incorrect-and-advisory, because it is the
only one that is *stable* — incorrect-and-blocking self-corrects under pressure, and
correct-and-advisory at least degrades gracefully. So: an advisory check earns its place
only while its findings are trusted, and the moment one is knowingly stepped over, the
check has stopped working regardless of what it prints. Fix it or delete it; a check
nobody acts on is worse than no check, because it occupies the slot where a real one
would go. And when a rule genuinely must not regress, the enforcement has to be able to
turn a build red — that is what `test_always_loaded_rules.mjs` does for the class of
failure this pattern describes, and why raising `skill_lint` to blocking was deferred
rather than skipped: it still carries other findings, and flipping it red before those
are cleared would just be a louder way of teaching people to ignore it.

See also
[[pattern-20260728-a-derivation-the-tooling-computes-but-doctrine-forbids-where-it-is-needed]]:
there the derivation existed but was unreachable from the decision that needed it; here
the check was reachable but declawed. Both end the same way — a correct mechanism sitting
next to the failure it was built to prevent.
