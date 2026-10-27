# Release 2.64.0 — Context

**Feature slug:** release-2-64-0
**Date:** 2026-10-27
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user, on 2026-10-27, after `pi-1-4-capability-audit` merged and the capture queue was drained:
"Push lên rồi release" (push, then release).

## What will be done

Publish 2.64.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-63-0` did.

## Locked decisions

- D1 (`084eb232`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch; it is done only when the script prints its
  final `OK`.

## Why 2.64.0

The released version is 2.63.0. This release ships Pi 1.4.0 support in the
Pi belt (`before_agent_start` ignores custom runs; ceiling 1.4.0, pi-1-4-capability-audit),
Paseo 0.13 compatibility (workers spawn with `--no-wait`, floor 0.12.0, steer
through `paseo send`; paseo-013-compat), and the drained capture queue landing
80 delivery records, 12 concept rule lines and one new pattern in
`docs/knowledge`. A host-visible runtime change is a minor bump.

## Risk named before the run

The local opencode install is a wrapper script the opencode test cannot read;
the gate runs with BEE_OPENCODE_SUITE_ALLOW_SKIP=1, as recorded for releases
2.60.0 to 2.63.0; CI runs that suite.
