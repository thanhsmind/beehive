# Release 2.50.0 — Context

**Feature slug:** release-2-50-0
**Date:** 2026-10-06
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked on 2026-10-06 to set the Pi team to run its workers on
Paseo and then push the code to a release.

## What will be done

Publish 2.50.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-49-0` did. The script
bumps both plugin manifests, runs the regen chain, runs the declared test
suite BEFORE anything is tagged, makes the release commit, tags, pushes
`main` and the tag, waits for `release-binaries`, and verifies the published
binaries and `SHA256SUMS`.

## Why 2.50.0

The released version is 2.49.0. This release ships five closed features:

- `mailbox-broker`: the mailbox broker routes worker results and questions.
- `paseo-pi`: Paseo is a new herding channel beside herdr.
- `paseo-heartbeat`: the Pi leader inside Paseo wakes from a Paseo heartbeat.
- `paseo-answers`: answers and steering reach Paseo workers.
- `paseo-observe`: status, interrupt, cancel and permit for Paseo workers.

A new transport and new verbs for host repos are a minor bump.

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
