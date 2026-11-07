# Release 2.84.0 — Context

**Feature slug:** release-2-84-0
**Date:** 2026-11-07
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner closed knowledge-outcome-measure and asked to commit and release.

## What will be done

Publish 2.84.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-83-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`) and the CI-pinned opencode 1.18.16 first on PATH; it is done only when the script prints its final `OK`. | 1601928e |

## Why 2.84.0

The released version is 2.83.0. This release ships knowledge-outcome-measure:
`bee dispatch prepare` holds a fixed 10% of cells out of the Learned context
block, records each cell's arm (`knowledge_arm`, `learned_context`) and reads
a new `knowledge_holdout_percent` config key; `bee knowledge report` gains an
`outcomes` section that compares the two arms. A new config key and a new
report section are a minor bump.
