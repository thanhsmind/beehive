# Release 2.118.0 — Context

**Feature slug:** release-2-118-0
**Date:** 2026-11-23
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner approved uat for auto-handoff-on-due and said "duyệt release", so
other projects get the current main on their next refresh.

## What will be done

Publish 2.118.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-117-0` did. The
release class playbook is `skills/bee-planning/playbooks/release.md`.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | f0435044 |
| C1 | The published v2.118.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.118.0; a test-role cell checks it read-only. | a74f8f67 |

## Why 2.118.0

The released version is 2.117.0. Since then main gained new behavior:
auto-handoff-on-due makes `bee feature start` hand the session off through
waggledance when a handoff is due. A new feature, so a minor bump.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
