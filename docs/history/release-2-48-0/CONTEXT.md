# Release 2.48.0 — Context

**Feature slug:** release-2-48-0
**Date:** 2026-10-03
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked to release 2.48.0 after closing `pi-extension-split`.

## What will be done

Publish 2.48.0 from `main` through the one sanctioned command, under an authorized deployment dispatch, exactly as `release-2-45-0` did. The script bumps both plugin manifests, runs the regen chain, runs the declared test suite BEFORE anything is tagged, makes the release commit, tags, pushes `main` and the tag, waits for `release-binaries`, and verifies the published binaries and `SHA256SUMS`.

## Why 2.48.0

The released version is 2.47.0. This release ships one closed feature, `pi-extension-split`:

- The Pi guard is now a folder, `.pi/extensions/bee-guard/` (entry `index.ts`, 16 small modules), instead of one 3767-line file.
- `bee onboard` installs the folder, removes the legacy `.pi/extensions/bee-guard.ts` first, and prunes `.ts` files bee no longer ships.
- `bee doctor --runtime pi` checks every module file and names `bee onboard --apply` as the fix.

A changed install layout for host repos is a minor bump.

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
