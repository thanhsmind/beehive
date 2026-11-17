# Release 2.111.1 — Context

**Feature slug:** release-2-111-1
**Date:** 2026-11-17
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner approved uat for issue-sweep-nov17 and asked to push and release ("push code rồi release").

## What will be done

Publish 2.111.1 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-111-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 45c9ca92 |
| C1 | The published v2.111.1 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.111.1; a test-role cell checks it read-only. | 062375a6 |

## Why 2.111.1

The released version is 2.111.0. This release ships issue-sweep-nov17: eleven bee-report bug fixes (GitHub issues 158-171) in the write guard, the read-only worker guard, lease release on cap, proof parsing, cell deps validation, decisions reattribute, the nested checkout scan, the herding concurrency count and dispatch prepare's agent check. Bug fixes only, so a patch bump.


## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
