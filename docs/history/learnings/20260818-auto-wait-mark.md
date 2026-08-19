---
date: 2026-08-18
feature: auto-wait-mark
categories: [pattern, process]
severity: normal
tags: [workflow-state, hooks, tests, judging, review]
---

# Learning: A constraint about how, not what, is invisible to a suite that asserts outcomes

**Category:** pattern
**Severity:** normal
**Tags:** [workflow-state, hooks, tests, judging]
**Applicable-when:** a locked decision constrains the execution — never
call X twice, never touch Y, never allocate Z — rather than the value
that comes out.

## What Happened

`auto-wait-mark` gives the Stop hook one job: record that control returned
to the human, on every turn end, so a dashboard can tell an idle run from
a blocked one. The user chose the broadest rule available — mark EVERY
stop, no text heuristic, no phrase table — which moved the signal from
"does a mark exist" into a third `kind` value, `turn-end`, and forced a
matching narrowing so `bee orient` stops calling that value a blocker.

The mechanism was small: three product files, one new enum value, one
guarded `if`. It still took two cells, three independent judge passes and
two reworks, and every defect found was found by the judge, never by the
suite and never by the orchestrator reading the diff.

## Root Cause

CONTEXT.md locked one constraint that no test could see: the hook "must
not add a second transcript read per turn". Thirteen behavior tests pinned
what the hook produced — the mark, its kind, its subject, the no-overwrite
rule, the non-Stop events, the malformed-transcript case, the failing-write
case. All green, full suite at 2052. And the hook read the transcript
twice.

An outcome assertion cannot observe an execution constraint. The output is
byte-identical whether the file is read once or ten times, so a green suite
here was not weak evidence — it was no evidence, wearing the costume of
proof. What finally distinguished the two states of the world was a
`#[cfg(test)]` thread-local counting every path the reader was called with,
and an assertion on the count: `left: 2 / right: 1` on the old code.

The second rework taught the sharper half. Rework one threaded the parsed
events out of the happy path and the counting test went green — but the
producer could still fail *after* its own read, and that failure discarded
the events, so the consumer re-read. The counting test only ever drove the
success path. Worse, that rework shipped a comment stating the re-reading
branch had no caller inside the Stop dispatch; it had one, twenty lines
above. The fix that held deleted the branch rather than re-describing it.

## What To Do Differently

- When a decision constrains execution rather than output, the cell must
  demand a test that observes the execution — a counter, a call log, an
  instrumented seam — and say plainly that an outcome test will not be
  accepted. Writing "prove it, do not assert it" into the cell is what
  produced the counting test; the first cell said only "reuse the existing
  resolver" and got a call-graph argument back.
- After adding a counter, enumerate every branch that reaches the counted
  call. A count of one on the happy path says nothing about the error path,
  and error paths are exactly where a "we already have it" cache gets
  dropped.
- Treat a comment asserting unreachability as a claim needing the same
  suspicion as a claim in a commit message. If a branch is genuinely
  unreachable, delete it; if it cannot be deleted, it is reachable.
- Feed each judge pass the previous verdict and demand exhaustive
  enumeration, not spot checks. Pass two found the residual only because it
  was told to hunt "any early return that happens after a read and discards
  what was read"; pass three was told to enumerate every path through the
  dispatch and found nothing left.

## Process Notes

Two secondary findings, both recorded to the backlog rather than fixed here:

- **The vocabulary widening could not be exercised by hand.** The
  PreToolUse CLI-shape guard runs `main`'s binary, so it rejected
  `--kind turn-end` from every shell even though the worktree's own freshly
  built binary accepted it. This is the recorded "source that ships without
  reinstalling the binary the hooks call is inert" pattern appearing in a
  new place — the guard, not just the hooks — and the failure presents as a
  bad command rather than a stale binary.
- **Disjoint file reservations are not a disjoint working tree.** Two
  workers with strictly non-overlapping file lists were correctly fanned out
  into the same worktree; one used a stash to isolate a compile check and
  briefly moved the other's uncommitted edits, and the other observed its
  in-progress work vanish and had to redo it. Reservations scope files;
  they do not scope tree-wide git operations.

## Evidence

- Judge pass 1 (`8461cccc`): NEEDS_REVISION, K2 and P3 FAIL — two full
  `std::fs::read` of the transcript per Stop.
- Judge pass 2 (`e6141ada`): NEEDS_REVISION, P3 still FAIL — reproduced by
  pointing `BEEHIVE_PERF_DIR` at a regular file so the rollup failed after
  its read; the counting test printed `left: 2`.
- Judge pass 3 (`3a57dbd3`): PASS, 19/19, both enumeration methods, with the
  new test independently reproduced as red against the pre-fix sources in a
  scratch copy.
- Full suite on the merged tree: 18 suites, 2097 passed, 0 failed.
