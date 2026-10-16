# Release 2.54.0 — Context

**Feature slug:** release-2-54-0
**Date:** 2026-10-16
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user said on 2026-10-16: "ok push code release".

## What will be done

Publish 2.54.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-53-0` did. The script
bumps both plugin manifests, runs the regen chain, runs the declared test
suite BEFORE anything is tagged, makes the release commit, tags, pushes
`main` and the tag, waits for `release-binaries`, and verifies the published
binaries and `SHA256SUMS`.

## Why 2.54.0

The released version is 2.53.0. This release ships three closed features:

- `slp-detector`: the SLP Detector — `bee detector tick` fires four typed
  events (self-correction, boundary-approach, test-on-unstable-contract,
  budget-80) that the supervisor judges; the supervisor prompt reads ten
  signals; `bee herding status` shows per-job event counts.
- `slp-role-gaps`: dissent repeat refusal, handback keys on the finish
  report, `stalled` by no diff progress, advisor recommendation form.
- `prune-queue-copy`: worktree prune queue bookkeeping.

New verbs and new behavior for host repos are a minor bump.

## Risk named before the run

A direct `scripts/release.sh 2.54.0` on 2026-10-16 stopped at the permit:
`release authorization required — BEE_DISPATCH_ID is unset`; nothing changed.
The full suite on the merged branch was green (4799 passed, 0 failed, 40
suites). If the script's test gate is red, it stops before any tag; the fix is
a fix-first cell, then a re-run at the same version (idempotent).

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
