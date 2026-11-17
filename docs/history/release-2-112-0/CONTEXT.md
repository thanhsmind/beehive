# Release 2.112.0 — Context

**Feature slug:** release-2-112-0
**Date:** 2026-11-17
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked to push and release after pstack-guide-gaps merged ("push code release").

## What will be done

Publish 2.112.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-111-1` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | c281e145 |
| C1 | The published v2.112.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.112.0; a test-role cell checks it read-only. | 759bba8e |

## Why 2.112.0

The released version is 2.111.1. This release ships pstack-guide-gaps: two new
skills (`bee-formal-verifying`, `bee-formal-upkeep`) with a statechart reference,
a new principle (`bee-principle-explain-the-number`) with a benchmark checklist,
and a ranked, capped principle list in `bee orient`, the session preamble and the
worker brief, with a cap-time check on principle names. New capability, so a
minor bump.


## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
