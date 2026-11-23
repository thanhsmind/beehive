# Release 2.119.0 — Context

**Feature slug:** release-2-119-0
**Date:** 2026-11-23
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner said "push code rồi release" after paseo-017-compat merged, so
other projects get the current main on their next refresh.

## What will be done

Publish 2.119.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-118-0` did. The
release class playbook is `skills/bee-planning/playbooks/release.md`.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 729acbcf |
| C1 | The published v2.119.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.119.0; a test-role cell checks it read-only. | 371c2d3b |

## Why 2.119.0

The released version is 2.118.0. Since then main gained paseo-017-compat:
every plain `paseo send` bee makes now pins `--behavior interrupt`, so the
Paseo 0.17 sender-dependent default cannot move it. A behavior change the
owner approved as 2.119.0.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
