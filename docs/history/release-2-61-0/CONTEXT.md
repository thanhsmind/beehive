# Release 2.61.0 — Context

**Feature slug:** release-2-61-0
**Date:** 2026-10-24
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user, on 2026-10-24, after `design-packs-install-prompt` merged:
"Release bản mới" (release a new version).

## What will be done

Publish 2.61.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-60-0` did.

## Locked decisions

- D1 (`c38b2760`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch; it is done only when the script prints its
  final `OK`.

## Why 2.61.0

The released version is 2.60.0. This release ships design-packs-install-prompt:
both installers ask which design packs to add on an interactive install,
`bee onboard --json` reports `design_offer`, `bee onboard --help` lists
`--design-packs`, and the human install commands no longer pass `-y`. New
installer behavior and a new JSON field make this a minor bump.

## Risk named before the run

The script's test gate runs the whole declared suite before any tag. The local
opencode install is a wrapper script the opencode test cannot read; the gate runs
with BEE_OPENCODE_SUITE_ALLOW_SKIP=1, as recorded for this release; CI runs that
suite.
