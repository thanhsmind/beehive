# Release 2.96.0 — Context

**Feature slug:** release-2-96-0
**Date:** 2026-11-13
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner said "release" after trust-layers, release-smoke and deploy-verify-gate merged to `main`.

## What will be done

Publish 2.96.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-95-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite and the new live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 809046c6 |

## Why 2.96.0

The released version is 2.95.0. This release ships trust-layers (static-check
offer at onboarding, the static-rules craft page, `--source` on reflections,
the prose-load witness), release-smoke (a live smoke inside the release
script) and deploy-verify-gate (every bee-written deploy drives the product
before it goes live), plus verify-render-standalone and windows-test-build.
New flags and new doctrine make it a minor bump.
