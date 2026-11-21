# Release 2.114.0 — Context

**Feature slug:** release-2-114-0
**Date:** 2026-11-21
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner said "Ok" to releasing the 200k default handoff mark, so other
projects get it on their next refresh.

## What will be done

Publish 2.114.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-113-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 0ef919c0 |
| C1 | The published v2.114.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.114.0; a test-role cell checks it read-only. | 45c28595 |

## Why 2.114.0

The released version is 2.113.0. Since then main gained new behavior: the
default handoff mark is 200000 tokens, a merge stores the merging session and
feature start refuses a second feature in it, bee herding result prints a
digest for a job, thin worker launches drop the skill and agent listings, and
the leader doctrine file and the cell brief are shorter. New capability, so a
minor bump.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
