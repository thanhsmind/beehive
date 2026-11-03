# Release 2.47.0 — Context

**Feature slug:** release-2-47-0
**Date:** 2026-10-02
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked to close `pi-1-0-upgrade` and release the new version.

## What will be done

Publish 2.47.0 from `main` through the one sanctioned command, under an authorized deployment dispatch, exactly as `release-2-45-0` did. The script bumps both plugin manifests, runs the regen chain, runs the declared test suite BEFORE anything is tagged, makes the release commit, tags, pushes `main` and the tag, waits for `release-binaries`, and verifies the published binaries and `SHA256SUMS`.

## Why 2.47.0

The released version is 2.46.0. This release ships one closed feature, `pi-1-0-upgrade`:

- Pi 1.0.0 is the proven ceiling, with a 0.85.1 to 1.0.0 audit; `codemode` and `tool_search` pass the Pi guard.
- Harness-native dispatch on Pi: the leader gets `bee_dispatch` and `bee_advisor`, and write-guard refuses a Pi leader's source write in the execute phase.
- Settle obligations: `bee hook session-close` names owed bee work once, and Pi gets one forced turn for it.
- Mid-run input: the new `bee herding steer` verb, typed context carried into running Pi workers, and one log-or-ask turn per scope change.

A new verb and new tool surfaces are a minor bump.

## Risk named before the run

The belt ships inside the bee binary, so host repos get the Pi changes only through this release. A red suite stops the script before any tag; the fix is a fix-first cell, then a re-run of the script at the same version (it is idempotent).

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
