# Release 2.43.0 — Context

**Feature slug:** release-2-43-0
**Date:** 2026-09-22
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked "push code release bản mới" after accepting `finding-recheck-trigger` at the uat door.

## What will be done

Publish 2.43.0 from `main` through the one sanctioned command, under an authorized deployment dispatch, exactly as `release-2-42-0` did. The script bumps both plugin manifests, runs the regen chain, runs the declared test suite BEFORE anything is tagged, makes the release commit, tags, pushes `main` and the tag, waits for `release-binaries`, and verifies the published binaries and `SHA256SUMS`.

## Why 2.43.0

The released version is 2.42.0. This release ships three features:

- `leader-check-diff`: `bee cells leader-check` now checks a cell's `files_changed` against the real diff of its `cell: <id>` commits and records `commit_diff`.
- `finding-recheck-trigger`: `bee triggers` gains a `path-changed:<paths>` predicate anchored at HEAD, and the per-prompt reminder prints `triggers due: N`.
- `trigger-cache-path`: the prompt trigger cache moves to the git-ignored `.bee/cache/`.

A new predicate kind, a new reminder line and a new refusal are a minor bump, not a patch.

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
