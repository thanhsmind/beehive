# Release 2.115.0 — Context

**Feature slug:** release-2-115-0
**Date:** 2026-11-21
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner approved building all six parts of leader-context-thrift and asked
for its release as the next minor version, so other projects get it on their
next refresh.

## What will be done

Publish 2.115.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-114-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 0f5b4db9 |
| C1 | The published v2.115.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.115.0; a test-role cell checks it read-only. | 3338a3ae |

## Why 2.115.0

The released version is 2.114.0. Since then main gained new behavior: the
default handoff mark is 120000 tokens, a leader's unranged read of a long file
is refused, a handoff record carries verified, tree, key files, gotchas,
standing orders and ruled-out paths and is probed with a cold read,
`bee dispatch prepare --prompt-out` leaves the worker prompt in a file, and the
new verb `bee report context` measures leader context from session
transcripts. New capability, so a minor bump.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
