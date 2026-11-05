# Release 2.77.0 — Context

**Feature slug:** release-2-77-0
**Date:** 2026-11-05
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked on 2026-11-05 to release the machine-sized worker limit.

## What will be done

Publish 2.77.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-76-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`) and the CI-pinned opencode 1.18.16 first on PATH; it is done only when the script prints its final `OK`. | f46364d6 |

## Why 2.77.0

The released version is 2.76.0. This release ships machine-sized-waves (the
swarming doctrine drops the fixed 3-4 worker cap) and machine-cap-half (the
unset herding worker limit is CPU count / 2). Every recent release took a minor
bump.
