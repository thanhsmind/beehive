# Release 2.45.0 — Context

**Feature slug:** release-2-45-0
**Date:** 2026-09-22
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked "release" after closing `mistake-fix-at` and `no-code-comments`.

## What will be done

Publish 2.45.0 from `main` through the one sanctioned command, under an authorized deployment dispatch, exactly as `release-2-44-0` did. The script bumps both plugin manifests, runs the regen chain, runs the declared test suite BEFORE anything is tagged, makes the release commit, tags, pushes `main` and the tag, waits for `release-binaries`, and verifies the published binaries and `SHA256SUMS`.

## Why 2.45.0

The released version is 2.44.0. This release ships two closed features:

- `mistake-fix-at`: every mistake record names the layer that stops it coming back (`--fix-at architecture|check|doctrine|none`, required on `bee mailbox reflect` and on a cap's `--mistake`); `bee close` files one backlog row per check- or architecture-layer mistake; the lesson miner keys on layer plus opening words.
- `no-code-comments`: the no-comment rule in `AGENTS.md` and the worker prompt; a write-guard arm that refuses a comment line in code where `no_code_comments` is on; `bee dev comment-baseline --check|--write` plus a ratchet test over a committed baseline; a verify-app feature map for the guard.

A new required flag on a shipped verb, a new config key and a new dev verb are a minor bump.

## Risk named before the run

`no_code_comments` is on in this repo only; hosts default to off. The ratchet test runs inside the declared suite, so a comment count that rose since the baseline reds the script's test gate before any tag. The fix is a fix-first cell, then a re-run of the script at the same version (it is idempotent).

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
- The batched removal of pre-existing comments (no-code-comments D6) stays separate grooming work.
