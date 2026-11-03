# Release 2.70.0 — Context

**Feature slug:** release-2-70-0
**Date:** 2026-11-02
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked on 2026-11-02: "Push code rease", after the stageshow frame
rehearsal and the design-check motion rule landed on `main`.

## What will be done

Publish 2.70.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-69-0` did.

## Locked decisions

- D1 (`3393ff35`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch; it is done only when the script prints its
  final `OK`.

## Why 2.70.0

The released version is 2.69.0. This release ships the frame-rehearsal scripts
in the bee-design-motion pack, the frame-only critic brief in bee-design, and
the Check-step motion rule: skill content that reaches hosts. Every recent
release took a minor bump.

## Risk named before the run

This machine has opencode 2.0.32, which the opencode contract test cannot
read; the run puts the CI-pinned opencode-ai 1.18.16 first on PATH, as the
2.68.0 and 2.69.0 runs did (decision `676b5133`).
