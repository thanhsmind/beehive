# Release 2.56.0 — Context

**Feature slug:** release-2-56-0
**Date:** 2026-10-18
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The supervisor task waggledance#t-bd665a on 2026-10-18: release a new
beehive version — build the binaries, tag, push and finish the release.

## What will be done

Publish 2.56.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-55-0` did. The script
bumps both plugin manifests, runs the regen chain, runs the declared test
suite BEFORE anything is tagged, makes the release commit, tags, pushes
`main` and the tag, waits for `release-binaries`, and verifies the published
binaries and `SHA256SUMS`.

## Why 2.56.0

The released version is 2.55.0. This release ships the closed
`open-issues-sweep` feature (26 commits since `v2.55.0`): the cap report
from a file with `--report-file`, the herded-worker whole-tree git check,
promote-reminder clearing by kind and feature, the doc-deferral scan skip,
the onboard block on a host-edited verify-app projection, the `triggers add`
store check, the control-root plan-freeze fallback, control-root decisions
and backlog from a granted worktree, the v2 escalate flag, and the stale
claim gate break.

A new flag and new behavior for host repos are a minor bump.

## Risk named before the run

A direct `scripts/release.sh 2.56.0` on 2026-10-18 stopped at the permit:
`release authorization required — BEE_DISPATCH_ID is unset`; nothing changed.
The script's test gate runs the whole declared suite before any tag; a red
there stops before tagging, and the fix is a fix-first cell, then a re-run at
the same version (idempotent).

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
