# Release 2.79.0 — Context

**Feature slug:** release-2-79-0
**Date:** 2026-11-06
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked on 2026-11-06 to release context-mark-advisory through
`scripts/release.sh`, so agent-works, anphabe-goglbe and survey-platform get
it at their next onboarding refresh.

## What will be done

Publish 2.79.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-78-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`) and the CI-pinned opencode 1.18.16 first on PATH; it is done only when the script prints its final `OK`. | e8ed9dfa |

## Why 2.79.0

The released version is 2.78.0. This release ships context-mark-advisory: the
context mark is advice, never a stop; a handoff happens only on the user's
word or at feature close, and passes `follow_up_of` to waggledance.
