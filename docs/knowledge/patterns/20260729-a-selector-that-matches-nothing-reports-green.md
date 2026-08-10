---
type: bee.pattern
title: A test selector that matches nothing reports green
description: A test selector that matches nothing reports green
tags: [failure, verification, verify-strings, vacuous-pass, cell-authoring]
timestamp: 2026-07-29
bee:
  id: pattern-20260729-a-selector-that-matches-nothing-reports-green
  lifecycle: active
  sources: ["original feature: budget-fence-removal", docs/history/learnings/20260729-budget-fence-removal.md]
  polarity: pitfall
  critical: false
---

# A test selector that matches nothing reports green

Substring-based test selection (`run_verify.mjs --only <token>`) silently drops a token that matches
no suite. The run then passes — for the wrong reason, having never executed the thing under test.

A cell that **creates** a suite and verifies with `--only <that-suite>` is the sharp case: at
authoring time the file does not exist, so the token matches nothing, and the verify is green before
a line is written. `budget-fence-removal`'s trailing test cell had exactly this shape; a dry-run
against `filterSuitesByOnly` showed it selecting three suites where four were intended.

Two rules:

- **Dry-run every verify string against the real selector before it reaches a worker**, and read the
  suite *count*, not just the exit code. A selector that resolves to fewer runnables than the author
  named is the tell.
- **A cell that creates a suite invokes it by path**, ahead of any `--only` form. The direct
  invocation fails loudly if the file was never created; the selector cannot.

Generalizes to any name-matching runner — `pytest -k`, `jest -t`, `go test -run`, `cargo test
<filter>`: a filter that matches zero tests exits 0 nearly everywhere.

**Full entry:** docs/history/learnings/20260729-budget-fence-removal.md
