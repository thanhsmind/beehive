# Release 2.68.0 — Context

**Feature slug:** release-2-68-0
**Date:** 2026-10-30
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked on 2026-10-30: "Push code release", after the statechart
playbook pointer landed on `main`.

## What will be done

Publish 2.68.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-67-0` did.

## Locked decisions

- D1 (`3393ff35`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch; it is done only when the script prints its
  final `OK`.

## Why 2.68.0

The released version is 2.67.0. This release ships the design gate's chart,
diagram, flow and motion artifact checks at ease-design 0.6.2, the design
pack command contracts, and the `statechart` step in the feature and bugfix
playbooks — new checks and skill content that reach hosts. Every recent
release took a minor bump.

## Risk named before the run

The local opencode install cannot be read by the opencode test; the gate runs
with BEE_OPENCODE_SUITE_ALLOW_SKIP=1, as recorded for releases 2.60.0 to 2.67.0;
CI runs that suite.
