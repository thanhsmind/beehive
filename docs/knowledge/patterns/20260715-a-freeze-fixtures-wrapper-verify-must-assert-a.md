---
type: bee.pattern
title: "A freeze fixture's wrapper verify must assert a printed sentinel, not a filename or bare exit"
description: "A freeze fixture's wrapper verify must assert a printed sentinel, not a filename or bare exit"
tags: [failure, freeze-first, sentinel-verify, false-pass, wrapper]
timestamp: 2026-07-15
bee:
  id: pattern-20260715-a-freeze-fixtures-wrapper-verify-must-assert-a
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT36", "original feature: codex-harness-hardening", docs/history/learnings/20260715-codex-harness-hardening-slice0.md]
  polarity: pitfall
  critical: false
---

# A freeze fixture's wrapper verify must assert a printed sentinel, not a filename or bare exit

A "red-now" freeze (a regression/lint that documents a defect before it is fixed) is only
trustworthy if its wrapper verify can tell "red for the right reason" from a crash. Two traps,
both make the wrapper false-PASS on a crash that never exercised the defect: (1) grepping for a
**bare filename** the fixture merely *reads* — a stack trace mentions that path too; (2) checking a
**bare non-zero exit** — node's uncaught-throw exit is `1`, indistinguishable from a lint's
"violations found" `1`. Rule: the fixture prints a **specific sentinel on the controlled defect
path only** (`FREEZE-RED: <specific>`, `CENSUS-VIOLATION <file>:<line>`) and exits a **distinct
code** (a sentinel like `3`, not `1`); the wrapper asserts sentinel-string AND that code. Keep
red-now freezes OUT of the mandatory verify command until the fix flips them green, so the baseline
stays green meanwhile. **Full entry:** docs/history/learnings/20260715-codex-harness-hardening-slice0.md
