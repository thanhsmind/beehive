# Release 2.69.0 — Context

**Feature slug:** release-2-69-0
**Date:** 2026-10-31
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked on 2026-10-31: "Push code và release", after the pstack
0.15.36 port landed on `main`.

## What will be done

Publish 2.69.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-68-0` did.

## Locked decisions

- D1 (`3393ff35`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch; it is done only when the script prints its
  final `OK`.

## Why 2.69.0

The released version is 2.68.0. This release ships the pstack 0.15.36 port:
run-proven review findings, decide-before-measure in the perf playbook, and
the sharpen and goal-playback moves at shaping intake — skill content that
reaches hosts. Every recent release took a minor bump.

## Risk named before the run

This machine has opencode 2.0.32, which the opencode contract test cannot
read; the run puts the CI-pinned opencode-ai 1.18.16 first on PATH, as the
2.68.0 re-run did (decision `676b5133`).
