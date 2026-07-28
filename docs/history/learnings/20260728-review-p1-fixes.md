---
date: 2026-07-28
feature: review-p1-fixes
categories: [orchestration, verify-pipeline, review]
severity: high
tags: [review, defect-class, invert-the-default, converging-rounds]
---

# review-p1-fixes — what an independent panel found, and what closing it taught

## What Happened

A four-reviewer panel (code-quality, architecture, security, test-coverage) reviewed
16 features and 78 commits with no session history. It raised **5 P1, 12 P2, 9 P3** —
and every P1 sat in the verification path this session had just rewritten. Two rounds
of fixes closed all five; a third finding remains open and filed.

| Round | Raised | Outcome |
|---|---|---|
| Panel | 5 P1 | 4 fix cells, all suites green |
| Delta re-review 1 | 2 NOT CLOSED + 4 new | 4 more fix cells |
| Delta re-review 2 | 3 CLOSED, 1 new P1 | filed, cell specified, not yet landed |

## The pattern that kept beating us

Three rounds, one shape: **a rule expressed as a list of places instead of a single
question.** The debt guard knew which phases to watch, so a new phase escaped it. Then
it knew which doors to guard, so the swap door escaped. Then the swap door knew which
debts to ask about, so test-cell debt escaped. Each fix was correct and each was
immediately outflanked, because the knowledge lived in the callers.

The cache told the same story from the other side: it tried to *detect* unsafe suites
by scanning their source for path tokens, and three suites reached live files
indirectly — including, memorably, the very suite that keeps the check filter honest,
which could be cached away while the filter was broken.

**Rule learned: when a guard keeps missing cases, stop adding cases. Invert the
default.** Caching became opt-in by declaration (6 of 115 suites qualify — the other
109 had been cached without anyone knowing what they read). Debt became "cleared only
by evidence": an unreadable cell store now means DEBT, not clear.

## What review was worth

Every P1 was invisible from inside the work:

- A test suite that passed locally and **fails on any CI runner** — the first push
  would have reddened the base branch and, by this repo's own rule, blocked every
  subsequent claim.
- A cache that reports green for checks it never ran — under a law that had just made
  the boundary run the *only* local proof.
- Guards that looked complete and had a working bypass, demonstrated by live repro.

None came from reading diffs harder. They came from four readers with no memory of why
anything was done, and from a delta re-review that refused to take the first round's
word for it.

## Recommendations

- **Re-review the fix, not just the finding.** Round one would have shipped believing
  itself done; the sweep for the defect *class* is what found the rest.
- **Make reviewers reproduce, not read.** Every confirmed close in rounds 2-3 came with
  exit codes from a scratch repo, and every census re-anchor was proved by mutating the
  thing it guards and watching it go red.
- **An allowlist a human maintains will drift.** Two cells this session existed only to
  add names to an export allowlist after another cell added exports; filed as friction
  toward making the declaration part of the adding cell's own obligation.

## Open

One P1 remains (feature-swap door does not ask test-cell debt), reproduced, specified in
cell `p3-1`, filed as a review finding. Released with it recorded as a known issue at the
owner's decision: it weakens internal process discipline, not user data.
