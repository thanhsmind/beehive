# Release 2.120.0 — Context

**Feature slug:** release-2-120-0
**Date:** 2026-11-23
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner said "Ok push code release" after parallel-session-unblock merged, so
other projects get the current main on their next refresh.

## What will be done

Publish 2.120.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-119-0` did. The
release class playbook is `skills/bee-planning/playbooks/release.md`.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 3bd6ee17 |
| C1 | The published v2.120.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.120.0; a test-role cell checks it read-only. | 7da00ca9 |

## Why 2.120.0

The released version is 2.119.0. Since then main gained parallel-session-unblock:
`bee worktree new` and `bee feature start` warn instead of refusing beside a
live session with a shared nested checkout when no companion is configured,
and past `handoff_context_tokens` the claim and dispatch verbs refuse
`[handoff_due]`, with the new `bee state handoff send` dispatching the fresh
leader through waggledance. Behavior changes the owner approved at uat.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
