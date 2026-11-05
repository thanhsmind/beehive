# Release 2.78.0 — Context

**Feature slug:** release-2-78-0
**Date:** 2026-11-05
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked on 2026-11-05 to release the handoff-self-relaunch doctrine fix.

## What will be done

Publish 2.78.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-77-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`) and the CI-pinned opencode 1.18.16 first on PATH; it is done only when the script prints its final `OK`. | e8ed9dfa |

## Why 2.78.0

The released version is 2.77.0. This release ships handoff-self-relaunch: at
the context handoff mark the leader hands the remaining work to a fresh leader
through waggledance instead of asking the user to open a new session.
