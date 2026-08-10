---
type: bee.pattern
title: "Race tests assert structure, never scheduler luck — and a race harness that hides the child's stderr is itself a bug"
description: "And a race harness that hides the child's stderr is itself a bug"
tags: [tests, ci, concurrency, determinism, windows, 2-core]
timestamp: 2026-07-21
bee:
  id: pattern-20260721-race-tests-assert-structure-never-scheduler-luck
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT1", "original feature: release-1-7-10-rc"]
  polarity: pitfall
  critical: false
---

# Race tests assert structure, never scheduler luck — and a race harness that hides the child's stderr is itself a bug

Three straight CI reds on 2-core runners were one class: assertions true only under many-core
scheduling. The durable rules: a losing racer's typed refusal is an OUTCOME, never a crash;
collect every racer's exit before asserting shared state; never assert child output under a
timeout tighter than process spawn (split timeout-semantics from capture-preservation); Windows
throws EBUSY/EPERM on rename/unlink over briefly-open files — wrap racing fs mutators in a
bounded transient-retry; and every race harness must print status+signal+stdout+stderr+error on
failure — an empty failure message blocked diagnosis for three runs. Prove race fixes
`taskset -c 0,1` 10/10, then trust only the exact-tag CI. Full story:
`20260721-ci-timing-flake-class.md`.
