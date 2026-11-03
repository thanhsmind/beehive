# Release 2.51.0 — Context

**Feature slug:** release-2-51-0
**Date:** 2026-10-06
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked on 2026-10-06: "release 2.51.0".

## What will be done

Publish 2.51.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-50-0` did. The script
bumps both plugin manifests, runs the regen chain, runs the declared test
suite BEFORE anything is tagged, makes the release commit, tags, pushes
`main` and the tag, waits for `release-binaries`, and verifies the published
binaries and `SHA256SUMS`.

## Why 2.51.0

The released version is 2.50.0. This release ships two closed features:

- `paseo-pi-hardening`: bounded Paseo calls, a silent-idle nudge, stalled status, the observed-model check, worker-guard for Paseo workers and the Pi supervisor, the leader broker timer, heartbeat cleanup and the doctor `paseo_ready` row.
- `herding-leader-toil`: `cells finish --from-job`, allow-listed control verbs from a granted worktree, `cells judge-record --from-text`, the Paseo event wait, opt-in isolated Pi folders, the doctor AppImage check and readable worker titles.

New flags and a new hook for host repos are a minor bump.

## Risk named before the run

The Pi extension and the worker-guard hook ship inside the bee binary, so host repos get them only through this release. A red suite stops the script before any tag; the fix is a fix-first cell, then a re-run of the script at the same version (it is idempotent).

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
