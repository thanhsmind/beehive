---
date: 2026-11-23
feature: parallel-session-unblock
categories: [pattern, failure]
severity: standard
tags: [tester-cell, cap-evidence, handoff-door, dispatch-door]
---

# Learning: A separate tester cell caught a float-typed read the implementer's green verify missed

**Category:** pattern
**Severity:** standard
**Tags:** [tester-cell, handoff-door]
**Applicable-when:** a cell adds a door or reader over a stored JSON number, and the plan can afford a separate test-role cell

## What Happened

psu-3 shipped the context-mark door and `bee state handoff send` (a0e1bf25a) with its filtered `cargo test ... handoff` green. psu-4, the test-role cell for psu-3, found that the session reader stores tokens as f64, so `as_u64` returned 0 and `handoff send` reported 0k. psu-4 showed the send test red without the fix and fixed it in 8a1c4c30f, with 6308 passed 0 failed recorded in its evidence.

## Root Cause

The implementer's tests wrote the session record with integer tokens, the same shape its own code expected. The tester wrote the record through the real reader path, which numberifies to f64.

## Recommendation

When a cell reads a number back from a bee JSON store, test it through the same writer the hook uses, not a hand-built integer fixture. Keep the split into an implementer cell and a test-role cell for doors and readers. Single source with no waste figure, so this stays a learning and is not promoted.

# Learning: A cap can record verify_passed true while its own evidence says a leg failed

**Category:** failure
**Severity:** standard
**Tags:** [cap-evidence, verify]
**Applicable-when:** reading or judging a capped cell's proof

## What Happened

psu-5 capped with verify_command `bee knowledge check && bee dev release-manifest --check`, verification_evidence saying the knowledge check failed on an unrelated file, verify_passed true and verify_output `green:static`. psu-1 recorded `green:unit` with no count, and its report.commit named the docs commit df55f64b1 instead of the code commit 3423396a7. All six proof pointers in trace.outcome resolve.

## Root Cause

Cap takes verify_passed and verify_output as the worker states them. A label passes where a count or exit line belongs.

## Recommendation

Read a cell's evidence text, not its verify_passed flag. The executable fix (cap refuses a label-only green, or evidence that reports a failed leg) is filed as a P2 friction row in .bee/backlog.jsonl for this feature.

# Learning: The harvest gathers for a closed lane had no working route through the dispatch door

**Category:** failure
**Severity:** standard
**Tags:** [dispatch-door]
**Applicable-when:** running Compound gathers for a lane whose role plan marks read-only-gather not-applicable

## What Happened

`dispatch prepare --kind gather --feature parallel-session-unblock` refused `stage_required`, then `stage_not_applicable` for read-only-gather. The documentation-and-capture stage returned a read-only herding run on agy-hydrogen, which failed `spawn_failed` because the antigravity provider has no read-only guard. Scoping the gather to the open feature routed read to haiku and worked.

## Root Cause

A lane's role plan is written for its execution, and the door applies it to the Compound gathers that run after close.

## Recommendation

Until the door resolves gathers to the read role on its own, scope a post-close harvest gather to the open feature. Filed as a P2 friction row in .bee/backlog.jsonl.
