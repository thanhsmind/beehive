# Release 2.63.0 — Context

**Feature slug:** release-2-63-0
**Date:** 2026-10-27
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user, on 2026-10-27, after `ub-mode-principles` merged:
"push rồi realease" (push, then release).

## What will be done

Publish 2.63.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-62-0` did.

## Locked decisions

- D1 (`084eb232`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch; it is done only when the script prints its
  final `OK`.

## Why 2.63.0

The released version is 2.62.0. This release ships two new routed principle
skills, `bee-principle-attack-the-premise` and `bee-principle-build-the-lever`,
an architecture guide section on one writer for shared state, and the tracked
`bee-design` Codex agent file. New principles that `bee orient` names are new
host behavior, so this is a minor bump.

## Risk named before the run

The local opencode install is a wrapper script the opencode test cannot read;
the gate runs with BEE_OPENCODE_SUITE_ALLOW_SKIP=1, as recorded for releases
2.60.0 to 2.62.0; CI runs that suite.
