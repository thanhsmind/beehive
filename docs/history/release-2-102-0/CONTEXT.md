# Release 2.102.0 — Context

**Feature slug:** release-2-102-0
**Date:** 2026-11-14
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked to release the trimmed skill descriptions after the hub republish.

## What will be done

Publish 2.102.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-101-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 311a4b41 |

## Why 2.102.0

The released version is 2.101.0. This release ships skill-description-trim
(every routable skill description fits 300 characters, with a suite fence) and
the Skillstation hub serving of more engine skills and the design packs. New
served skills and a new fence make it a minor bump.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/project/overview.md | confirmed | names `scripts/release.sh` as the one release path |

### Code

| Path | What it decides |
|------|-----------------|
| `scripts/release.sh` | every release step, the test gate and the final OK line |
