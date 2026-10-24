# Release 2.60.0 — Context

**Feature slug:** release-2-60-0
**Date:** 2026-10-24
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user, on 2026-10-24, after `design-os-in-bee` merged and its uat passed:
"Duyệt uat đóng release" (approve uat, close, release).

## What will be done

Publish 2.60.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-59-0` did.

## Locked decisions

- D1 (`c38b2760`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch; it is done only when the script prints its
  final `OK`.

## Why 2.60.0

The released version is 2.59.0. This release ships design-os-in-bee: the
design packs, the `design` team role and `bee-design` agent, `bee onboard
--design-packs`, `bee design gate`, the UI-cell close rule, and the installers'
`--design-packs` / `-DesignPacks` options. New verbs, flags and host behavior
make this a minor bump.

## Risk named before the run

The script's test gate runs the whole declared suite before any tag. The local
opencode install is a wrapper script the opencode test cannot read; if the gate
stops on a red test, that is a fix-first cell, then a re-run at the same version
(idempotent).
