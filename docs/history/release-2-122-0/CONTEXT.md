# Release 2.122.0 — Context

**Feature slug:** release-2-122-0
**Date:** 2026-11-25
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked for a code eval of cited sources: open the cited file
and search for the related text, so a citation that points nowhere is
caught. The owner then asked to push and release it.

## What will be done

Publish 2.122.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-121-0` did. The
release class playbook is `skills/bee-planning/playbooks/release.md`.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 77873f5e |
| C1 | The published v2.122.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.122.0; a test-role cell checks it read-only. | 8aea692b |

## Why 2.122.0

The released version is 2.121.0. Since then main gained
citation-source-check: the Stop hook checks each cited source of the
last reply against disk and the session commands, and prints one
advisory line naming the sources that fail.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
