---
type: bee.pattern
title: The test-shaped-path advisory only reads paths, missing Rust's in-file #[cfg(test)] convention
description: bee cells finish's "touches no test-shaped path" advisory fires on a cell whose entire diff IS tests, because it heuristically matches file paths and Rust keeps tests inside src/*.rs behind #[cfg(test)] rather than in a separate test-shaped path
tags: [cells, verify, rust, false-positive]
timestamp: 2026-08-29
bee:
  id: pattern-20260829-test-shaped-path-blind-to-cfg-test
  lifecycle: active
  areas: [workflow-state]
  sources: ["cell lmd-2 (lane-model-diversity), 2026-08-29 — advisory fired on a test-only diff", "cell 0c927049-sourced observation, 2026-08-30 — same false positive reconfirmed against the close-usage-record cells"]
  polarity: pitfall
  evidence: observed
---

# The test-shaped-path advisory is blind to in-file `#[cfg(test)]`

`bee cells finish` prints a "touches no test-shaped path — consider adding
test coverage" advisory keyed on a path heuristic (does the diff touch
something that LOOKS like a test path — `tests/`, `*_test.rs`, etc). Rust's
dominant convention keeps unit tests inside the same file as the code they
cover, behind an in-file `#[cfg(test)] mod tests { ... }` block — so a cell
whose entire diff is new tests, added as a `#[cfg(test)]` module in
`src/*.rs`, still trips the advisory. Reconfirmed on two independent cells
(lane-model-diversity's lmd-2, and the close-usage-record cells) — every
test-only cell in this codebase is expected to trip it.

**Why the path heuristic misses it.** The matcher reads file PATHS only. A
path like `src/verbs/drivers/close.rs` carries no lexical signal that the
lines added inside it are `#[test]`/`#[cfg(test)]` — the signal lives in
diff CONTENT, not in the path.

## Fix direction (not yet implemented)

Teach the heuristic the in-file test convention — a diff-content signal
(`cfg(test)`/`#[test]` present in added lines), or suppress the advisory on
cells whose declared role is `test`. Filed as a defect observation only; no
skill text or behavior changed yet.
