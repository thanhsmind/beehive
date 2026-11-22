# Release 2.117.0 — Context

**Feature slug:** release-2-117-0
**Date:** 2026-11-22
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner closed scheduled-curator and said "Release", so other projects
get the current main on their next refresh.

## What will be done

Publish 2.117.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-116-1` did. The
release class playbook is `skills/bee-planning/playbooks/release.md`.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 307ac212 |
| C1 | The published v2.117.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.117.0; a test-role cell checks it read-only. | a50d7848 |

## Why 2.117.0

The released version is 2.116.1. Since then main gained new behavior:
scheduled-curator added the curator due kind, its per-project config, the
deferred-queue work-queue verbs and the bee-curating skill. A new feature,
so a minor bump.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
