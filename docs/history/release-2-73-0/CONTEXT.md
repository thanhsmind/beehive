# Release 2.73.0 — Context

**Feature slug:** release-2-73-0
**Date:** 2026-11-04
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user said on 2026-11-04: "push code rồi release", after lean-context and
finish-phase landed on `main`.

## What will be done

Publish 2.73.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-72-0` did.

## Locked decisions

- D1 (`ac0b0628`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch, with the CI-pinned opencode-ai 1.18.16 first
  on PATH; it is done only when the script prints its final `OK`.

## Why 2.73.0

The released version is 2.72.0. This release ships the prompt-context handoff
warning at 150k tokens, the 150k-or-65% handoff doctrine, and the feature
finish phase move. Every recent release took a minor bump.
