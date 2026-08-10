---
type: bee.pattern
title: "A shared checkout with a second live session: check the out-of-scope tree before a blocking verify, and diff before diagnosing \"flaky\""
description: "A shared checkout with a second live session: check the out-of-scope tree before a blocking verify, and diff before diagnosing \"flaky\""
tags: [process, multi-session, verify-collision, diagnosis, git-status-first]
timestamp: 2026-07-19
bee:
  id: pattern-20260719-a-shared-checkout-with-a-second-live-session
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT42", "original feature: lane-ceremony-v3"]
  polarity: pitfall
  critical: false
---

# A shared checkout with a second live session: check the out-of-scope tree before a blocking verify, and diff before diagnosing "flaky"

The close-out full-verify chain went red twice on two hermetic write-guard tests while a
concurrent session's uncommitted `checkWrite` rewrite sat in the shared checkout — the test
copies the lib at run time, so it deterministically captured the mid-edit source; standalone
re-runs outside the collision window were green, and the feature closed with zero source
changes. Two rules: **(1)** before running a blocking verify chain while another session is
active in the same checkout, require `git status --short` OUTSIDE the acting cell's own
`files[]` to be clean — a dirty out-of-scope tree is a named-conflict abort, not a 349-test
run into a doomed red (mechanization: backlog P56). **(2)** when a red suite's failure text
names concepts owned by a different live feature, `git diff` the implicated paths BEFORE any
"is it flaky" retest — byte-identical failure text across runs is the signature of a content
collision, not flakiness, and "wait for their merge" is the wrong escalation until a diff
proves their *finished* semantics (not their in-flight edit) caused the red.
