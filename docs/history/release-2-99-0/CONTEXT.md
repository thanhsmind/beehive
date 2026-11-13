# Release 2.99.0 — Context

**Feature slug:** release-2-99-0
**Date:** 2026-11-14
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner said "Release" after refactoring-ui-craft merged to `main`.

## What will be done

Publish 2.99.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-98-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 7a394988 |

## Why 2.99.0

The released version is 2.98.0. This release ships refactoring-ui-craft: a new
bee-design addition, `additions/refactoring-ui-craft.md`, with 25 craft rules,
a symptom lookup, an overcorrections table and a not-a-finding table. New
shipped skill content makes it a minor bump.
