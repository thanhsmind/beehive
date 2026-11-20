# Release 2.113.0 — Context

**Feature slug:** release-2-113-0
**Date:** 2026-11-20
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked to push code and release after the fix-at debt work merged ("Push code rồi release").

## What will be done

Publish 2.113.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-112-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | cef8956c |
| C1 | The published v2.113.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.113.0; a test-role cell checks it read-only. | fab0bd4c |

## Why 2.113.0

The released version is 2.112.0. Since then main gained new behavior: gate
preview refuses a cell id held by another feature's cell, bee test warns when a
shared cargo target meets a live worker in another checkout, bee herding status
--json is bounded by default, the herding worker cap has one home in
herding.limits.concurrency, onboarding names a missing or due knowledge bundle,
and a source edit is refused while the route class is research. New
capability, so a minor bump.


## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
