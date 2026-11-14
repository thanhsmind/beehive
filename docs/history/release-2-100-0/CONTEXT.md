# Release 2.100.0 — Context

**Feature slug:** release-2-100-0
**Date:** 2026-11-14
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner said "Push hết code lên rồi release" after upkeep-due-signal merged to `main`.

## What will be done

Publish 2.100.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-99-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 311a4b41 |

## Why 2.100.0

The released version is 2.99.0. This release ships upkeep-due-signal: a new
read-only verb, `bee upkeep due`, two new audit kinds, and the two upkeep
skills at version 0.2. It also ships the knowledge-upkeep content run. A new
verb makes it a minor bump.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/project/overview.md | confirmed | names `scripts/release.sh` as the one release path |

### Code

| Path | What it decides |
|------|-----------------|
| `scripts/release.sh` | every release step, the test gate and the final OK line |
