# Release 2.94.0 — Context

**Feature slug:** release-2-94-0
**Date:** 2026-11-12
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked for the full build of sup-20261112-1646 and then a new release ("Làm hoàn chỉnh rồi release bản mới", relayed by waggledance).

## What will be done

Publish 2.94.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-93-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | ae491a14 |

## Why 2.94.0

The released version is 2.93.0. This release ships waggledance-cli-first: bee
doctrine, skills and the cross-project guard refusal name the `waggledance` CLI
first and the MCP tools as the fallback, with one request id shared by both
paths. A changed public remedy text and a doctrine rule change make it a minor
bump.
