# Release 2.82.0 — Context

**Feature slug:** release-2-82-0
**Date:** 2026-11-07
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked on 2026-11-07 for two framework fixes (a knowledge-before-code
nudge for leaders, and bookkeeping moved out of the decision store into an
audit log) and to ship them through `scripts/release.sh`.

## What will be done

Publish 2.82.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-81-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`) and the CI-pinned opencode 1.18.16 first on PATH; it is done only when the script prints its final `OK`. | e1fb190a |

## Why 2.82.0

The released version is 2.81.0. This release ships
knowledge-first-lean-decisions: a leader's first code read in a turn with no
`bee knowledge search` gets a one-line nudge; a new `bee audit log` holds gate
auto-approval audit, close/merge/release bookkeeping and door deferrals;
`bee decisions log` refuses those tags; and every close door reads the audit
log as well as the old decision rows. A new verb is a minor bump.
