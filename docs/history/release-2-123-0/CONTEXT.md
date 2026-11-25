# Release 2.123.0 — Context

**Feature slug:** release-2-123-0
**Date:** 2026-11-26
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked to push the code on main and release it.

## What will be done

Publish 2.123.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-122-0` did. The
release class playbook is `skills/bee-planning/playbooks/release.md`.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 9f3a4a0d |
| C1 | The published v2.123.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.123.0; a test-role cell checks it read-only. | 783ed791 |

## Why 2.123.0

The released version is 2.122.0. Since then main gained the agy
runtime and its session env, the agy leader end-to-end spike fixes,
job-worker-adopt, deferred-queue request flags, verify-map and
knowledge upkeep, and the PBIs p-b454de67, p-3f1d7db8, p-e8eeae96,
p-41a23d81 and p-a0f03c4a.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
