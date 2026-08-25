---
type: bee.pattern
title: One red hides the rest — cargo test stops at the first failing target
description: One red hides the rest — cargo test stops at the first failing target
tags: [failure, tests, verification, release]
timestamp: 2026-08-25
bee:
  id: pattern-20260825-cargo-test-stops-at-the-first-failing-target
  lifecycle: active
  areas: [verify-pipeline]
  sources: ["statusline-binary-lookup -> lane-row-order -> opencode-contract-reds -> herding-registry-gap, 2026-08-25: four reds discovered one at a time, each revealed only by fixing the one above it"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "compare the per-target `test result:` lines against the number of targets the workspace has — a run that stops early prints fewer; `cargo test --no-fail-fast` prints them all"
---

# One red hides the rest — cargo test stops at the first failing target

Clearing one red test to unblock a release took four rounds, not one. Not because
the fixes were hard — each was small — but because `cargo test` stops after the
first *target* that fails, so the run never reaches the targets behind it.

The sequence, each step discovered only after the one above it went green:

1. a lane test asserting a filesystem enumeration order
2. a belt-parity gate calling a telemetry probe a blocking rule
3. tool anchors pinned to a minifier-generated identifier
4. four dispatcher commands the registry never declared

Three of the four were already on `origin/main`, some for days. Nobody was
ignoring them. The suite reported *one* failure every time it ran, and one
failure reads as one problem.

## The rule

"The suite is red" is not a count. Before estimating any work that depends on
green — a release above all — run `--no-fail-fast` once and read the real depth.
An estimate built on the first red is an estimate of the first red only.

## The release-shaped consequence

A release must not be planned around "fix the failing test", singular. Ask what
the whole board looks like first, then decide, because the answer changes what
the work is: one small fix and ship, or a stack in four different features owned
by four different people.

Two smaller edges worth keeping from the same hunt:

- **Check whether a red is even in CI before treating it as a blocker.** One of
  the four failed only locally: the test reads the *installed* third-party binary,
  CI pins that dependency, and the developer machine had drifted a version ahead.
  Same command, different answer, entirely by environment.
- **An anchor keyed on a minified identifier is a scheduled failure.** The bundle
  renamed one helper from `V` to `j` between builds and three anchors broke at
  once. The same file already knew this — it anchored its *primary* lookup on
  values for exactly that reason, then re-introduced the hazard for three
  secondary ones. Anchor on values the source really contains, never on a name a
  minifier chose.
