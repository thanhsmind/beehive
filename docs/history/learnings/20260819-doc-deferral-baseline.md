---
date: 2026-08-19
feature: doc-deferral-baseline
categories: [pattern, process]
severity: normal
tags: [workflow-state, tests, judging, review, multi-session]
---

# Learning: A guard whose every firing is a false positive is not mistuned, its premise is wrong for the domain

**Category:** pattern
**Severity:** normal
**Tags:** [workflow-state, tests, judging]
**Applicable-when:** a guard keeps blocking, every block gets waved through
with a recorded reason, and the fix reached for each time is another
per-instance exemption.

## What Happened

`bee close`'s doc-deferral door blocks when deferral-shaped prose in a
feature's touched docs cites no registered trigger. It had fired five times —
staging-optional, staging-lane, uat-gate-before-merge, test-doctrine-text-sweep,
auto-wait-mark — and every flagged line on every occasion was prose *describing*
deferral machinery, not prose deferring work. Five features, zero true
positives. Each was resolved by logging a per-feature exemption decision, and
two backlog rows had been filed proposing the same fix from the same angle:
scope the scan to the lines the feature changed.

The scan scope was not the root cause. The word list is `defer`, `later`,
`for now`, `revisit when` — and this repo's own domain is deferral queues and
triggers, so those are its nouns. A doc explaining how the deferred-work queue
behaves trips a detector for deferrals by saying true things about deferrals.
No word list can separate that, and neither can a tighter scan: a feature that
edits one line of a long-lived doc still inherits every pre-existing match in
it, because the scan is file-scoped.

The fix keeps the word list and the scan and adds a baseline: a tracked file
recording the deferral-shaped lines that already exist, seeded once repo-wide,
after which only NEW lines block. Identity is normalized line content, never
line number.

## Root Cause

Five per-feature exemption decisions are five signals that the guard's model
does not match the domain, read one at a time as five separate incidents. The
per-instance escape hatch worked well enough each time to keep anyone from
asking the question the fifth firing should have forced: what is the ratio of
true to false positives over the guard's whole life? Here it was zero to
thirteen-plus. A guard at that ratio is not a guard, it is a toll.

## What To Do Differently

- When reaching for a guard's escape hatch for the third time, stop and count.
  A guard whose every firing has been waved through is answering a question
  the codebase does not have.
- Distinguish tuning from premise. Tightening the scan would have delayed the
  false positives, not ended them, because the collision was between the
  detector's vocabulary and the domain's vocabulary.
- Freeze existing debt, police new debt. The lint-baseline rollout applies to
  any guard retrofitted onto a codebase that predates it.

## The Expensive Part: two decisions I wrote wrong

**D2 named a scope the code did not have.** I wrote "the baseline is seeded once
per repo" while the door's scan set is per-FEATURE — the union of capped cells'
changed files plus the feature's own history dir. So the first close would have
frozen only the docs that one feature happened to touch, and the next feature
touching a different long-lived doc would enter enforcement against an empty
entry and inherit every pre-existing line in it. The false positives would have
returned on a delay, after the feature that "fixed" them had closed. An
independent judge traced that consequence; the worker had implemented my words
faithfully.

The check this implies: when a decision names a scope — "per repo", "per
feature", "the whole tree" — verify that scope exists in the code the decision
lands in, rather than assuming the words carry it.

**D4 got the scope of its own rule wrong in the other direction.** D4 refused a
third escape, reasoning that a hand-edit path onto the baseline is how lint
baselines rot. True of the baseline, and irrelevant to what a peer session had
independently built: a site-local marker naming why a passage documents
deferral machinery, where an empty reason exempts nothing. That is not an edit
to the baseline at all. D8 reversed D4 once the peer made the distinction. The
decisive argument was theirs: a baseline cannot express intent, because asked
why a line is forgiven its only answer is "it was already there".

Both defects are the same shape as one I then made in conversation — telling a
peer a scoping rule "was never carried to the two neighbouring roots", when the
real axis was authored prose versus machine-written and only ONE root held
authored prose. A third session corrected me by quoting the cell text. Reading
the wrong feature of the thing under-applies a rule and over-applies it with
equal ease; the sharper check is which siblings hold the same KIND of content
the rule protects, never which siblings sit next to it in the path.

## The Cheap Part, twice

I wrote a vacuous test one commit after publishing a pattern about vacuous
tests. Capping the dry-run output at a constant, the test read:

```rust
let total = DOC_DEFERRAL_DRY_RUN_SAMPLE + 7;
assert_eq!(detail.matches("deferral-shaped prose").count(), DOC_DEFERRAL_DRY_RUN_SAMPLE);
```

Both sides move with the constant, so raising it to `100000` — deleting the cap
in every way that matters — left the test green. Literals fail that mutation
immediately. Knowing the pattern by name did not prevent writing it twenty
minutes later; running the mutation did. Any assertion pinning a bound, cap,
limit, or count earns one deliberate break-and-check.

Earlier in the same cell, an independent judge found the mirror image: two of
six pre-existing door tests had been left running in the door's SEED arm, where
it passes regardless of what the scan found, so neither could fail. Mutation
proved it — disabling the fenced-code exemption and the citation escape left
both green.

## Process Notes

Two sessions built two different fixes for the same door in parallel and
resolved it by talking rather than racing. The peer found my lane before landing
into a file I held, proposed dropping their own work, and was right that the two
mechanisms answer different questions — the baseline forgives the past
automatically and repo-wide, the marker states intent at the site for prose
written after the seed. Both land.

The merge then stopped at a uat gate on a `small`-lane bugfix, because the door
classifies through `mode` (which holds `feature`) instead of `route.lane` (which
holds `small`), and an unrecognized value fails closed. That is a live second
reproduction of a bug the same peer had already fixed on another branch — and
the remaining half of their fix edits the file this merge is holding, so the bug
blocks the merge that unblocks the fix for the bug.

## Evidence

- Judge pass 1: NEEDS_REVISION, DEV-A and REGRESS FAIL, both proved on
  unmutated code in a scratch copy.
- Judge pass 2: PASS, 19/19 checks, with all six pre-existing door tests
  independently mutation-tested and the repo-wide seed proved by a probe
  asserting the scan set is exactly one file while the baseline covers others.
- Measured seed cost on this repo: 1554 markdown files under `docs/`, 1258
  pre-existing deferral lines, ~1.7 s, one time.
- Dry-run detail before the cap: 143010 bytes, printed and embedded in JSON.
- Full suite at cap: 18 suites, 2116 passed, 0 failed.
