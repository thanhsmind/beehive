# Release 2.72.0 — Context

**Feature slug:** release-2-72-0
**Date:** 2026-11-03
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user said on 2026-11-03: "duyệt và release", after issue-sweep, the
cli-shape continuation and issue-sweep-2 landed on `main`.

## What will be done

Publish 2.72.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-71-0` did.

## Locked decisions

- D1 (`1201a724`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch, with the CI-pinned opencode-ai 1.18.16 first
  on PATH; it is done only when the script prints its final `OK`.

## Why 2.72.0

The released version is 2.71.0. This release ships the bee fixes for GitHub
issues #137 to #153 from issue-sweep and issue-sweep-2, and the guard that
reads a backslash-continued documented command as one command. Every recent
release took a minor bump.

## Risk named before the run

This machine's `opencode` is a wrapper script that the opencode contract test
cannot read; the run puts the CI-pinned opencode-ai 1.18.16
(`/tmp/oc11816/node_modules/.bin`) first on PATH, as 2.68.0 to 2.71.0 did
(decision `676b5133`).
