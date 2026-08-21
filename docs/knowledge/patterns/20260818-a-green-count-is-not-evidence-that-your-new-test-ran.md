---
type: bee.pattern
title: A green count is not evidence that your new test ran
description: A green count is not evidence that your new test ran
tags: [failure, tests, merges, verification]
timestamp: 2026-08-18
bee:
  id: pattern-20260818-a-green-count-is-not-evidence-that-your-new-test-ran
  lifecycle: active
  areas: [rust-runtime]
  sources: ["uat-approval-reaches-the-door, merge-conflict resolution of verbs/worktree/tests.rs, 2026-08-18"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "run the new test by exact name and read its own count — `cargo test --release <manifest> --bin <bin> <test_name>` printing `0 passed, N filtered out` is the signal; the whole-suite total cannot show it"
---

# A green count is not evidence that your new test ran

Resolving a merge conflict, a block of five new tests was re-inserted before the
last closing brace of a test file — which turned out to close the last
*function*, not the module. The block landed inside another test's body. It
compiled. The suite passed. The total even went up, because the other side's
tests had arrived in the same merge. Nothing anywhere said that the five tests
written to guard a security gate were never registered and never ran.

They were found only by running one of them by name and reading `0 passed,
1963 filtered out`.

**The rule:** after adding tests — especially after a merge, a file split, or
any move that re-anchors them — prove the *new* ones ran, by name or by a
count that changed by the number you added. A whole-suite green answers a
different question: it says nothing broke, never that anything new was checked.

The trap generalizes past insertion points. Any change that can silently
un-register a check — a renamed module, a dropped `mod` line, a fixture built
in a shape the check filters out, a guard whose scan set no longer contains the
file — produces the same reading: an honest green over a smaller set than you
believe you are measuring. The count is only meaningful against a set you have
verified, and the cheapest verification is to make the check fail on purpose
once.

Corollary for reviewers: when a cap's proof line is a suite total, it is
evidence about the suite, not about the cell. Ask which new case ran.
