---
type: bee.pattern
title: A test run under a configuration must assert the configuration is live
description: A test run under a configuration must assert the configuration is live
tags: [failure, testing, vacuous-tests, fixtures, bypass]
timestamp: 2026-07-29
bee:
  id: pattern-20260729-a-test-run-under-a-configuration-must-assert-the-configuration-is-live
  lifecycle: active
  sources: ["original feature: worker-conformance", docs/history/learnings/20260729-worker-conformance.md]
  polarity: pitfall
  critical: true
---

# A test run under a configuration must assert the configuration is live

A test proves a door still refuses *even under* some setting — a bypass level, a feature flag, a
permissive mode. It seeds the setting into a fixture workspace, exercises the door, and asserts the
refusal.

If the seeding silently fails — wrong path, wrong key name, a schema the writer no longer accepts —
the door refuses anyway, because it would have refused with no setting at all. The row passes and
proves nothing. Adding a "still opens when nothing is owed" mirror does not help: the mirror shows
the row is not welded shut, and says nothing about whether the setting was ever read.

`worker-conformance` found every pre-existing bypass row in one suite shared this weakness; an
advisor consult caught it while reviewing new rows written in the same shape.

**Assert the configuration is live before exercising the behaviour** — read it back through the same
accessor the production code uses, and fail the row if it is not what was seeded. The check costs
one line and converts a whole class of rows from decorative to load-bearing.

The general form: whenever a test's premise is *state you set up elsewhere*, assert the premise
holds before asserting the conclusion. A conclusion that would also hold without the premise is not
evidence for it.

**Full entry:** docs/history/learnings/20260729-worker-conformance.md
