---
type: bee.pattern
title: A contract written by one unit and read by another owes a row that crosses the seam
description: A contract written by one unit and read by another owes a row that crosses the seam
tags: [failure, testing, cross-cell-contracts, vacuous-tests, fixtures]
timestamp: 2026-07-29
bee:
  id: pattern-20260729-a-contract-written-by-one-unit-and-read-by-another-owes-a-row-that-crosses-the-seam
  lifecycle: active
  sources: ["original feature: worker-conformance", docs/history/learnings/20260729-worker-conformance.md]
  polarity: pitfall
  critical: true
---

# A contract written by one unit and read by another owes a row that crosses the seam

One cell writes a field. A sibling cell reads it. Each cell's tests are locally complete and green:
the writer asserts what it wrote, the reader is fed a hand-built fixture carrying the same field.
Nothing runs both sides in one pass.

That suite pair is green against a dead contract. Rename the field, change its shape, or move where
it is stamped, and the writer's tests still pass, the reader's fixtures still carry the old value,
and the door the field was supposed to arm quietly never arms again.

`worker-conformance` shipped exactly this and a semantic judge caught it on one of seven checks —
every door row was seeded by a hand-writing helper, so the producer/consumer seam was proven
nowhere. The repair drove a real close through the real command path and then hit a real door in the
same run, asserting whatever the producer actually stamped. Its first attempt crossed only one of
the two readers named in the failure signature; an advisor consult caught that too.

**One row must cross the seam with nothing hand-written**, plus a mirror where the field is absent
and the door opens, so the assertion cannot pass vacuously. Keep the hand-built fixture rows — they
cover breadth cheaply. The seam row buys something they structurally cannot.

When a verdict names N call sites, the fix owes N rows, and the re-check should assert that count.

**Full entry:** docs/history/learnings/20260729-worker-conformance.md
