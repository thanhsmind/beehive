# Release 2.41.3 — Context

**Feature slug:** release-2-41-3
**Date:** 2026-09-18
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked "merge về main rồi release" after `pi-statusline-model-limits` was completed.

## What will be done

Publish 2.41.3 from `main` through the one sanctioned command, under an authorized deployment dispatch, exactly as `release-2-41-2` did. The script bumps both plugin manifests, runs the regen chain, runs the declared test suite BEFORE anything is tagged, makes the release commit, tags, pushes `main` and the tag, waits for `release-binaries`, and verifies the published binaries and `SHA256SUMS`.

## Why 2.41.3

The released version is 2.41.2. This release ships the `pi-statusline-model-limits` feature: Pi's embedded guard extension now queries 5-hour and weekly quota buckets for Antigravity models, and appends compact rate limits to the active-branch model usage statusline.

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
