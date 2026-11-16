# Release 2.110.0 — Context

**Feature slug:** release-2-110-0
**Date:** 2026-11-16
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner approved uat for fix-at-loop and asked to release ("Duyệt realse").

## What will be done

Publish 2.110.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-109-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | ee21cfcf |
| C1 | The published v2.110.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.110.0; a test-role cell checks it read-only. | 3839bf00 |

## Why 2.110.0

The released version is 2.109.0. This release ships semantic-token-diet
(leaner instruction text and help output) and fix-at-loop (a fix-at mistake is
filed once across features, the digest folds copies and reports open, closed and
preamble load, and bee runs its own `clippy.toml` banning `Path::strip_prefix`).
New host-visible behavior makes it a minor bump.


## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
