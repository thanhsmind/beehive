---
type: bee.pattern
title: A "never do X twice" constraint is invisible to a suite that only asserts outcomes
description: A "never do X twice" constraint is invisible to a suite that only asserts outcomes
tags: [failure, tests, performance, hooks, review]
timestamp: 2026-08-18
bee:
  id: pattern-20260818-a-never-do-x-twice-constraint-is-invisible-to-a-suite-that-only-asserts-outcomes
  lifecycle: active
  areas: [workflow-state, rust-runtime]
  sources: [".bee/cells/awm-2.json", "original feature: auto-wait-mark"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs (stop_reads_the_transcript_exactly_once_per_turn and a_late_perf_refresh_failure_still_reads_the_transcript_exactly_once — a cfg(test) thread-local in reads.rs logs every path read_jsonl is called with, so the assertion counts reads instead of inspecting outcomes)"
---

# A "never do X twice" constraint is invisible to a suite that only asserts outcomes

`auto-wait-mark` locked one performance constraint in its CONTEXT: the Stop hook must
not add a second transcript read per turn. The hook's behavior was then pinned by
thirteen tests — the mark is written, the subject is right, a declared wait survives,
non-Stop events write nothing, a broken transcript writes nothing, a failing store write
is logged. Every one of them passed. The full suite was green at 2052.

The hook read the transcript twice anyway. Then, after a rework, it read it twice again
on a narrower path. Neither leak moved a single assertion, because every assertion asked
"what came out?" and the constraint was about "how did it get there?".

The distinction is not about performance. A constraint of the form *never do X more than
once*, *never call Y*, *never touch Z* is a statement about the execution, not the
result — and an outcome assertion cannot observe it, no matter how many outcomes it
checks. The correct output is exactly the same whether the file is read once or ten
times. A green suite is not weak evidence here; it is *no* evidence, and it reads as
proof.

What closed it was instrumentation: a `#[cfg(test)]` thread-local recording every path
the reader was called with, and an assertion on the count. That test failed on the old
code with `left: 2 / right: 1` — the first thing in the whole episode that actually
distinguished the two states of the world.

Two further lessons from how it took three passes.

**The second leak hid behind an error path.** Rework one threaded the parsed events out
of the happy path and the counting test went green. But the producer could still fail
*after* its own read, and the failure discarded the events, so the consumer re-read. The
counting test only ever drove the success path. When you add a counter, enumerate the
branches that reach the counted call — a count of one on the happy path says nothing
about the error path, and error paths are exactly where a "we already have it" cache
gets dropped.

**The code documented the hole shut.** That rework shipped a comment stating the
re-reading branch had no caller inside the Stop dispatch. It had one, twenty lines
above. A comment asserting unreachability is a claim like any other and deserves the
same suspicion as a claim in a commit message — if a branch is genuinely unreachable,
delete it; if you cannot delete it, it is reachable. The fix that finally held removed
the branch entirely rather than re-describing it.

Both leaks were found by an independent semantic judge reading the control flow and
reproducing the count on a scratch copy. Neither was found by the suite, by the builder,
or by the orchestrator reviewing the diff.
