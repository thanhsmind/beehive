# Release 2.103.0 — Context

**Feature slug:** release-2-103-0
**Date:** 2026-11-15
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner approved uat for per-task-router and asked to release it.

## What will be done

Publish 2.103.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-102-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | ddb536fb |

## Why 2.103.0

The released version is 2.102.0. This release ships per-task-router: per-task
routing with `bee orient --class`, a preamble that shows a route only to the
session that owns it, a Principles block in every worker prompt, principle
answers and proof floors at cap (with `green:red-first`), and a separate tester
for every cell with files, enforced at close and merge. New verbs and new cap
doors make it a minor bump.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
