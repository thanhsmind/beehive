# Test doctrine — discovery map

## Destination

A locked decision set on how bee guides agents about tests — who picks
scope, what proof means per change type, where determinism remains —
then one shaped feature that implements it (CLI loosening + skill text
+ DoD wording).

Spawned: test-doctrine — docs/history/test-doctrine/CONTEXT.md

## Notes

- Reality at map time (2026-08-18, verified in code): per-cell full-suite
  is already gone (decision 13ce1858, test-cadence-boundary) — caps are
  commit-only; `commands.test` auto-runs at `bee close` /
  `bee worktree merge`, always full, blind to diff type. SHIPPED SINCE:
  the boundary auto-run is gone too — decisions 58ec9664/1f534837 made
  each cap record its own proof line, and the doors check that record
  and run nothing themselves.
- The session preamble still orders a full-suite run before the first
  claim ("never build on red"), even for docs work.
- AGENTS.md carried the stale line "bee cells finish runs them" at map
  time — contradicted 13ce1858. SWEPT: the test-doctrine feature fixed it;
  AGENTS.md now states the proof-line doctrine.
- Seed for scoped proof exists: the per-cell `verify` field.
- Standing pattern (docs/knowledge/patterns/20260721): local green is
  worthless without hermeticity — CI is the real gate. This is what
  makes local freedom affordable.

## Decisions so far

- D-58ec9664: agent owns test scope end to end (boundary included),
  records scope + reason; docs-only diffs skip the suite; session-start
  red check dropped; CI full stays the last net; DoD =
  proof-per-change-type as principle; CI-red-after-scoped-green =
  fix-first + mandatory captured learning.
- D-1f534837: no boundary auto-run — close/merge require a recorded
  proof line; proof string replaces the tests enum in the cap report;
  one feature ships the whole package (CLI + preamble + skill text +
  stale AGENTS.md lines) — tickets/001-enforcement-lines.md,
  tickets/003-rollout-shape.md.

## Not yet specified

(none — parity-proof composition and the capture loop dissolve into
the DoD principle, D-58ec9664; release-close determinism answered by
D-1f534837)

## Out of scope

- CI pipeline changes — CI stays full suite on every push/PR, by
  decision.
- Per-test selection tooling (nextest filters, test-impact analysis) —
  implementation detail for the spawned feature, not a map decision.
