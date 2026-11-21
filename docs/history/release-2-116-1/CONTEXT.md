# Release 2.116.1 — Context

**Feature slug:** release-2-116-1
**Date:** 2026-11-22
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner closed test-ci-hygiene and asked to push a release, so other
projects get the current main on their next refresh.

## What will be done

Publish 2.116.1 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-116-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | e208dd54 |
| C1 | The published v2.116.1 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.116.1; a test-role cell checks it read-only. | 9f810c64 |

## Why 2.116.1

The released version is 2.116.0. Since then main gained no new behavior:
the test-ci-hygiene close routed its D1 and D2 into the knowledge bundle
and recorded its bookkeeping. Knowledge and record changes only, so a
patch bump.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
