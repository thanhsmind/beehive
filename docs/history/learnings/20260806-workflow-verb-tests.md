---
date: 2026-08-06
feature: workflow-verb-tests
categories: [workflow-state, verify-pipeline]
severity: low
tags: [coverage, verb-surface, seam-testing, test-shape]
---

# workflow-verb-tests — well-covered seams do not add up to a covered command

## What Happened

`state start-feature`, `state workflows list` and `state workflows close` had
zero test call sites. Everything under them was unit-tested — record creation
idempotence, close-by-feature keeping only the named one, the legacy seeding
gate, the store's own refusals — which is why the gap survived: every part was
proven, and the assembly was not. Sixteen integration tests now drive the
three verbs against temp repo fixtures. They found no defect.

## What Was Learned

**Coverage of the parts is not coverage of the command.** What the verb layer
owns is exactly what the seam tests cannot see: flag parsing, which record the
call resolves to, the wording of the refusal, the emitted payload, and the
exit code. A reviewer reading the unit tests would have called this area well
covered, because it is — one layer down.

**The shape of the function decides the shape of its test.** These three
resolve their root from the process working directory and print as a side
effect, so there was no honest in-process assertion to make; they had to be
driven as a built binary against a temp root. Where a sibling verb was split
into a `run_x_body(root, flags)`, the test is a cheap unit test. The split is
what makes verbs testable in-process — worth doing when a verb is written, not
retrofitted after a gap is found.

**A test-only cell reports defects, it does not fix them.** None turned up
here. Had one, the finding belonged in the backlog with its reproduction — a
cell that both writes the test and quietly changes the behavior it tests
proves nothing.

## Evidence

- Cell `wvt-1`, commit `28074d6b` —
  `packages/bee-rs/crates/bee/tests/workflow_verbs.rs`, 16 tests: a clean
  start plus all five guarded refusals proven to fail closed with zero
  mutations, listing over mixed status/phase, close by `--feature`, by `--id`,
  and by `--all-but-active`, each leaving every other record untouched, and
  both close modes refusing rather than degrading to "close everything" when
  the active feature cannot be resolved.
