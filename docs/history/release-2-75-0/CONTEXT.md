# Release 2.75.0 — Context

**Feature slug:** release-2-75-0
**Date:** 2026-11-05
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked for release 2.75.0 on 2026-11-05, after agent-shell-rename
landed on `main`, and confirmed it after installing opencode on PATH.

## What will be done

Publish 2.75.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-74-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`) and the user-installed opencode on PATH; it is done only when the script prints its final `OK`. | 5ade702e |

## Why 2.75.0

The released version is 2.74.0. This release ships agent-shell-rename (the
renamed rendered agent shells, the bee-elder advisor shell, and the team roles
moved onto Opus). Every recent release took a minor bump.
