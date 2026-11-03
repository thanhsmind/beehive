# Release 2.67.0 — Context

**Feature slug:** release-2-67-0
**Date:** 2026-10-30
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked on 2026-10-30: "push code và release", after bee-deploying-skill
merged and closed.

## What will be done

Publish 2.67.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-66-0` did.

## Locked decisions

- D1 (`997faaba`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch; it is done only when the script prints its
  final `OK`.

## Why 2.67.0

The released version is 2.66.0. This release ships the new `bee-deploying`
skill (learn, record and replay a project's deploy) and the
`bee onboard --skillstation` option, new skill content and a new flag that
reach hosts. Every recent release took a minor bump.

## Risk named before the run

The local opencode install cannot be read by the opencode test; the gate runs
with BEE_OPENCODE_SUITE_ALLOW_SKIP=1, as recorded for releases 2.60.0 to 2.66.0;
CI runs that suite.
