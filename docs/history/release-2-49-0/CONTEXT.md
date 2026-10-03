# Release 2.49.0 — Context

**Feature slug:** release-2-49-0
**Date:** 2026-10-03
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked to push the code, release and close ("push code release
close") on 2026-10-03, after six features closed.

## What will be done

Publish 2.49.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-48-0` did. The script
bumps both plugin manifests, runs the regen chain, runs the declared test
suite BEFORE anything is tagged, makes the release commit, tags, pushes
`main` and the tag, waits for `release-binaries`, and verifies the published
binaries and `SHA256SUMS`.

## Why 2.49.0

The released version is 2.48.0. This release ships six closed features:

- `pi-slp-dispatch-return`: a herded cell worker gets one order; the Pi
  dispatch tool passes stage and feature; the verdict tool binds to its job.
- `pi-slp-operations`: seven refusals name one runnable fix.
- `refusal-followups`: the lane-mid-flight refusal names the bind command.
- `pi-slp-next-ops`: `bee orient` reads the session's own record and names one
  runnable next command; the per-turn hint carries it.
- `orient-merged-worktree`: no worktree-enter step after a merge.
- `pi-slp-measurement`: docs only.

Changed CLI output and a changed worker brief for host repos are a minor bump.

## Risk named before the run

The belt ships inside the bee binary, so host repos get the folder layout only through this release. A red suite stops the script before any tag; the fix is a fix-first cell, then a re-run of the script at the same version (it is idempotent).

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
