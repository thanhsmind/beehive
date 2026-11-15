# Release 2.107.0 — Context

**Feature slug:** release-2-107-0
**Date:** 2026-11-15
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked to release cite-knowledge-first ("Làm đi").

## What will be done

Publish 2.107.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-106-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 6f46ecff |

## Why 2.107.0

The released version is 2.106.0. This release ships cite-knowledge-first:
the leader doctrine now cites the docs/knowledge concept for how something
works and docs/history only for why. A doctrine change agents act on makes
it a minor bump.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
