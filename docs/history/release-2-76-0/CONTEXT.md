# Release 2.76.0 — Context

**Feature slug:** release-2-76-0
**Date:** 2026-11-05
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked on 2026-11-05 to merge the worker-status-digest worktree and
release the new feature.

## What will be done

Publish 2.76.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-75-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`) and the CI-pinned opencode 1.18.16 first on PATH; it is done only when the script prints its final `OK`. | 7e344313 |

## Why 2.76.0

The released version is 2.75.0. This release ships worker-status-digest (workers
write progress notes; leaders read a one-line status digest). Every recent
release took a minor bump.
