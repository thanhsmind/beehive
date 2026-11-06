# Release 2.79.1 — Context

**Feature slug:** release-2-79-1
**Date:** 2026-11-06
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked on 2026-11-06 to release framework-feedback-rule through
`scripts/release.sh`, so agent-works, anphabe-goglbe and survey-platform get
it at their next onboarding refresh.

## What will be done

Publish 2.79.1 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-79-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`) and the CI-pinned opencode 1.18.16 first on PATH; it is done only when the script prints its final `OK`. | e8ed9dfa |

## Why 2.79.1

The released version is 2.79.0. This release ships framework-feedback-rule:
owner feedback given in the bee repo changes the framework itself, never
only an agent's private memory (rule agents-feedback-changes-framework).
