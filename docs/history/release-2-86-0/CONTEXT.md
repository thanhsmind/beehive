# Release 2.86.0 — Context

**Feature slug:** release-2-86-0
**Date:** 2026-11-07
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner approved uat for workspace-protocol and asked for a release
("Duyệt uat rồi release"). Owner feedback in beehive changes the framework
and ships in a release.

## What will be done

Publish 2.86.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-85-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | caf7c17d |

## Why 2.86.0

The released version is 2.85.0. This release ships workspace-protocol
(`bee protocol check`, `bee protocol init`, the doctor row, prompt and
protocol digests on every dispatch, `bee dispatch verify`, the no-menu and
no-leaning brief lines) and protocol-init-from-project (init renders from
the project, `--print`, the agent step in the bee-hive onboarding
reference). New verbs and new flags are a minor bump.
