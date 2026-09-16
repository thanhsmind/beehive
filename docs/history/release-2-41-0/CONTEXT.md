# Release 2.41.0 — Context

**Feature slug:** release-2-41-0
**Date:** 2026-09-16
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked for a release in one word ("release") right after
`harness-worktree-relocation` merged to main. Its `uat` gate is still open; this
lane does not approve it and does not need it.

## What was found

`scripts/release.sh 2.41.0` refused with zero changes: it needs
`BEE_DISPATCH_ID` naming a deployment dispatch under the `deploy` role, and
`bee dispatch authorize` validates that record against the version, the main
commit, the approved plan hash, the stage, the role, the issuing session and a
two-hour lifetime. A deployment stage needs its own approved `bee-plan/v2`
plan, so the release has its own lane, shaped like `release-2-40-0`.

## What will be done

Publish 2.41.0 from `main` through the one sanctioned command. The script bumps
both plugin manifests, runs the regen chain, runs the declared test suite
BEFORE anything is tagged, makes the release commit, tags, pushes `main` and the
tag (carrying the 12 commits already waiting), waits for `release-binaries`, and
verifies the published binaries and `SHA256SUMS`.

## Why 2.41.0

The released version is 2.40.0. This release ships a new verb
(`bee worktree exit`), a new merge flag (`--detached`), new result fields
(`sessionRuntime`, `instruction`), a Pi `/bee-worktree-exit` command, and a
behavior change: `bee worktree merge` from main no longer merges while the
calling session still sits in that worktree. New surface and changed behavior
move the minor number.

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
- Approving `harness-worktree-relocation`'s `uat` gate; that stays with the user.
