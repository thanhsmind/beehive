# Release 2.106.0 — Context

**Feature slug:** release-2-106-0
**Date:** 2026-11-15
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked to push the code and release it.

## What will be done

Publish 2.106.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-105-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | fe262b0e |

## Why 2.106.0

The released version is 2.105.0. This release ships cited-claims: a checked
claim in a reply carries a numbered citation and the reply ends with a Sources
block, and the Stop hook counts such a reply as checked. New behavior makes it
a minor bump.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
