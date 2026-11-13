# Release 2.98.0 — Context

**Feature slug:** release-2-98-0
**Date:** 2026-11-13
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner said "Push code release" after knowledge-skill-pair merged to `main`.

## What will be done

Publish 2.98.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-97-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 6f0247e6 |

## Why 2.98.0

The released version is 2.97.0. This release ships knowledge-skill-pair: two
new skills (`bee-knowledge-setup`, `bee-knowledge-upkeep`), the
`--knowledge-wrong` cap answer with its report section, and seven printed
knowledge writing rules. A new flag, a new report section and two new skills
make it a minor bump.
