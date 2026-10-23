# Release 2.58.0 — Context

**Feature slug:** release-2-58-0
**Date:** 2026-10-23
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user, on 2026-10-23, after `report-usage-verb` merged: "release".

## What will be done

Publish 2.58.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-57-0` did. The script
bumps both plugin manifests, runs the regen chain, runs the declared test
suite BEFORE anything is tagged, makes the release commit, tags, pushes
`main` and the tag, waits for `release-binaries`, and verifies the published
binaries and `SHA256SUMS`.

## Locked decisions

- D1 (`fb3a8c26`): the release runs only through `scripts/release.sh` under
  an authorized deployment dispatch; it is done only when the script prints
  its final `OK`.

## Why 2.58.0

The released version is 2.57.0. This release ships `bee report usage` (a
task's completion-report token block from the session transcript) and the
AGENTS.md "Report back to waggledance" rule. New behavior for host repos is a
minor bump.

## Risk named before the run

The script's test gate runs the whole declared suite before any tag. If the
gate stops on a red test, that is a fix-first cell, then a re-run at the
same version (idempotent).
