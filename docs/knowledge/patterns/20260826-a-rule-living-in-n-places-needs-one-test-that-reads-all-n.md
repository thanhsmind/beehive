---
type: bee.pattern
title: A rule living in N places needs one test that reads all N
description: When one command or value must be identical in several files, discipline and docs will not keep them synced — one test that reads every copy and diffs them will, and it catches the drift before it exists.
tags: [tests, ci, config, drift]
timestamp: 2026-08-26
bee:
  id: pattern-20260826-a-rule-living-in-n-places-needs-one-test-that-reads-all-n
  lifecycle: active
  areas: [verify-pipeline, doctrine-layer]
  sources: ["role-surface-cleanup rsc-2, 2026-08-25 — proof_gate's parity test went red the moment CI gained --no-fail-fast without commands.test moving in the same change", "packages/bee-rs/crates/bee/tests/proof_gate.rs (ci_runs_the_declared_test_command_and_adds_no_flags_to_it)"]
  polarity: practice
  critical: false
  evidence: exercised
  evidence_ref: "proof_gate.rs pins both workflows' cargo test invocation byte-for-byte against .bee/config.json commands.test; observed failing red on a workflow-only edit before it reached main, 2026-08-25"
---

# A rule living in N places needs one test that reads all N

The declared test command lives in three files: `.bee/config.json`
(`commands.test`), `.github/workflows/ci.yml`, and
`.github/workflows/windows.yml`. Adding `--no-fail-fast` to the two workflows
alone produced a red test within one suite run — before the drift ever reached
main — because `proof_gate.rs` reads all three surfaces and diffs them
byte-for-byte.

That is the pattern, and it earned its keep on first contact:

- **Discipline does not scale to N copies.** The same repo had already lived
  the failure the test's own message cites: `-- --test-threads=1` kept a flaky
  parallel suite green on CI for a whole cutover, because CI ran a *different*
  command from the local one and nobody noticed.
- **The test reads the real files, not a recorded expectation.** There is
  nothing to update when the rule legitimately changes — you change all N
  copies in one commit and the test stays green; change fewer and it names the
  file that lagged.
- **It converts a review-time concern into a build-time refusal.** No reviewer
  has to remember that CI and config must move together; the gate remembers.

When a rule must exist in several places — a command, a version pin, a model
name, a threshold — write the one test that opens every copy and compares,
before writing the second copy.
