# Release 2.93.0 — Context

**Feature slug:** release-2-93-0
**Date:** 2026-11-11
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner approved uat for supervisor-read-scope and asked for the release ("duyệt — push and release 2.93.0", relayed by the waggledance leader).

## What will be done

Publish 2.93.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-92-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | ae491a14 |

## Why 2.93.0

The released version is 2.92.0. This release ships supervisor-read-scope: a repo
that sets the governed key guards.supervises to waggledance may read, read-only,
the .bee/ store and docs/knowledge/ of every project in the waggledance registry,
and bee knowledge report / bee report outcomes take --root. A new config key and
two new flags make it a minor bump.
