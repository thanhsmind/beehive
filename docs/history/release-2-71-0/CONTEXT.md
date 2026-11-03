# Release 2.71.0 — Context

**Feature slug:** release-2-71-0
**Date:** 2026-11-03
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked on 2026-11-03: "release bản mới", after output spill,
`bee dispatch wave --run` and the paseo wait deadline landed on `main`.

## What will be done

Publish 2.71.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-70-0` did.

## Locked decisions

- D1 (`d5f80512`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch, with the CI-pinned opencode-ai 1.18.16 first
  on PATH; it is done only when the script prints its final `OK`.

## Why 2.71.0

The released version is 2.70.0. This release ships behavior that reaches
hosts: the opt-in output spill of read verbs, the `--run` flag on
`bee dispatch wave` with its `bee-swarming` line, and the deadline on the paseo
wait that ended healthy herding workers as `timed_out_idle`. Every recent
release took a minor bump.

## Risk named before the run

This machine's `opencode` is a wrapper script that the opencode contract test
cannot read; the run puts the CI-pinned opencode-ai 1.18.16
(`/tmp/oc11816/node_modules/.bin`) first on PATH, as 2.68.0 to 2.70.0 did
(decision `676b5133`).
