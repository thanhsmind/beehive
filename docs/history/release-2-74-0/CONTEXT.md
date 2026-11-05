# Release 2.74.0 — Context

**Feature slug:** release-2-74-0
**Date:** 2026-11-05
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user said "release" on 2026-11-05, after device-aware-cap landed on `main`.

## What will be done

Publish 2.74.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-73-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the CI-pinned opencode-ai 1.18.16 first on PATH; it is done only when the script prints its final `OK`. | ac0b0628 |

## Why 2.74.0

The released version is 2.73.0. This release ships the bee-report fixes and
device-aware-cap (machine-aware herding limits, prune at close). Every recent
release took a minor bump.
