# Release 2.66.0 — Context

**Feature slug:** release-2-66-0
**Date:** 2026-10-29
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked on 2026-10-29: "Push code release", after ui-ux-pro-max-a11y-port merged.

## What will be done

Publish 2.66.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-65-0` did.

## Locked decisions

- D1 (`997faaba`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch; it is done only when the script prints its
  final `OK`.

## Why 2.66.0

The released version is 2.65.0. This release ships the bee-design WCAG 2.2
additions (`packages/bee/design/bee-design/additions/wcag-2-2.md`), new skill
content that reaches hosts. Every recent release took a minor bump.

## Risk named before the run

The local opencode install cannot be read by the opencode test; the gate runs
with BEE_OPENCODE_SUITE_ALLOW_SKIP=1, as recorded for releases 2.60.0 to 2.65.0;
CI runs that suite.
