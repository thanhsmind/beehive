# Release 2.83.0 — Context

**Feature slug:** release-2-83-0
**Date:** 2026-11-07
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner rule of 2026-11-07 (decision dc19f610): the full declared suite
runs once per feature, at its end, before the merge to main. The owner asked
for it to be written, checked and shipped through `scripts/release.sh`.

## What will be done

Publish 2.83.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-81-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`) and the CI-pinned opencode 1.18.16 first on PATH; it is done only when the script prints its final `OK`. | e1fb190a |

## Why 2.83.0

The released version is 2.82.0. This release ships full-suite-once: `bee
cells add` and `bee cells update` refuse a cell whose verify is the bare full
suite unless it carries `full_suite_reason`; `bee test` stamps the checkout
HEAD; `bee worktree merge` refuses a code feature without a fresh green `bee
test` record and takes a new `--full-suite-waiver` flag; doctrine, preamble,
skills and knowledge carry the once-per-feature rule. A new flag is a minor
bump.
