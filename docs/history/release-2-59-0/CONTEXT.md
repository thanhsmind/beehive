# Release 2.59.0 — Context

**Feature slug:** release-2-59-0
**Date:** 2026-10-23
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user, on 2026-10-23, after `issues-sweep-1023` merged: "release bản mới
rồi dọn worktree" (release the new version, then clean the worktrees).

## What will be done

Publish 2.59.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-58-0` did.

## Locked decisions

- D1 (`7a67a0c9`): the release runs only through `scripts/release.sh` under
  an authorized deployment dispatch, after `windows-ci-followup` merges and
  CI is green; it is done only when the script prints its final `OK`.

## Why 2.59.0

The released version is 2.58.0. This release ships the issues-sweep-1023
fixes (#117, #118, #120-#127, #96) and the Windows follow-up. New flags
(`route --set --lane-record`) and a new onboarding `.gitattributes` block are
new behavior for host repos, so this is a minor bump.

## Risk named before the run

The script's test gate runs the whole declared suite before any tag. Three
pi_plugin_contracts tests flake when run in parallel (recorded at 2.58.0), and
the local opencode install is a wrapper script the opencode test cannot read.
If the gate stops on a red test, that is a fix-first cell, then a re-run at
the same version (idempotent).
