# Release 2.95.0 — Context

**Feature slug:** release-2-95-0
**Date:** 2026-11-12
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner said "release" after the worktree-file-links change merged to `main`.

## What will be done

Publish 2.95.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-94-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | b7bb64ae |

## Why 2.95.0

The released version is 2.94.0. This release ships worktree-file-links: a
report names a file in an unmerged feature worktree by its absolute path. A
doctrine rule change makes it a minor bump.
