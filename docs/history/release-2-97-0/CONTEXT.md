# Release 2.97.0 — Context

**Feature slug:** release-2-97-0
**Date:** 2026-11-13
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner said "release" after onboard-workspace-protocol merged to `main`.

## What will be done

Publish 2.97.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-96-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 7fdfa320 |

## Why 2.97.0

The released version is 2.96.0. This release ships onboard-workspace-protocol:
every `bee onboard` result reports the `WORKSPACE_PROTOCOL.md` state and
prompts the agent to finish the file, and the bee-hive onboarding procedure
says where to look and what a good file holds. A new payload field and new
doctrine make it a minor bump.
