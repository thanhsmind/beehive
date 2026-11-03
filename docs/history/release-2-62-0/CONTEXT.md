# Release 2.62.0 — Context

**Feature slug:** release-2-62-0
**Date:** 2026-10-25
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user, on 2026-10-25, after `issue-sweep-1024` merged:
"Release bản mới cho tôi" (release a new version for me).

## What will be done

Publish 2.62.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-61-0` did.

## Locked decisions

- D1 (`c38b2760`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch; it is done only when the script prints its
  final `OK`.

## Why 2.62.0

The released version is 2.61.0. This release ships 18 commits: the issue sweep
fixes (guard, worktree, dispatch and paseo supervision fixes), a new `doctor`
warning when design work has no design skill, and the design-packs install
command in INSTALL.md and README.md. A new doctor warning is new behavior, so
this is a minor bump.

## Risk named before the run

The script's test gate runs the whole declared suite before any tag. The local
opencode install is a wrapper script the opencode test cannot read; the gate runs
with BEE_OPENCODE_SUITE_ALLOW_SKIP=1, as recorded for releases 2.60.0 and 2.61.0;
CI runs that suite.
