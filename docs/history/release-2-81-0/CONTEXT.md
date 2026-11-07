# Release 2.81.0 — Context

**Feature slug:** release-2-81-0
**Date:** 2026-11-07
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked on 2026-11-07 to make native-model sub-sessions visible
in Paseo and to ship the change through `scripts/release.sh`.

## What will be done

Publish 2.81.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-80-1` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`) and the CI-pinned opencode 1.18.16 first on PATH; it is done only when the script prints its final `OK`. | e1fb190a |

## Why 2.81.0

The released version is 2.80.1. This release ships paseo-visible-sessions:
inside Paseo a native Claude sub-session starts through `paseo run`, so it
shows in the Paseo app; the Paseo model check accepts Claude aliases; and a
claude-agent fallback logs its reason. It also carries the work merged
since 2.80.1: herding-job-feature, thin-boot-default and report-usage-herding.
A new behavior on the dispatch path is a minor bump.
