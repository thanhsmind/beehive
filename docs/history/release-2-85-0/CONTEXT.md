# Release 2.85.0 — Context

**Feature slug:** release-2-85-0
**Date:** 2026-11-07
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked to build the v2 agent-work protocol into bee (verified-claims-triage). Owner feedback in beehive changes the framework and ships in a release.

## What will be done

Publish 2.85.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-84-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 7fe27bed |

## Why 2.85.0

The released version is 2.84.0. This release ships verified-claims-triage: the
`agents-verify-before-agreeing` rule (triage line, `[verified]`/`[unverified]`
claim labels, no agreement before the key assumption is verified,
evidence-first pushback, fast-lane read budget) and a new Stop-hook warning
when a reply agrees without a `[verified]` label. A new rule and a new hook
warning are a minor bump.
