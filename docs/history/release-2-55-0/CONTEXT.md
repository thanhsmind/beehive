# Release 2.55.0 — Context

**Feature slug:** release-2-55-0
**Date:** 2026-10-17
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user said on 2026-10-17: "Release".

## What will be done

Publish 2.55.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-54-0` did. The script
bumps both plugin manifests, runs the regen chain, runs the declared test
suite BEFORE anything is tagged, makes the release commit, tags, pushes
`main` and the tag, waits for `release-binaries`, and verifies the published
binaries and `SHA256SUMS`.

## Why 2.55.0

The released version is 2.54.0. This release ships five closed features
(68 commits since `v2.54.0`):

- `pstack-0-15-port`: `proof_carried` at `bee worktree merge` (a clean rebase
  no longer reads as stale proof), reasons on every rule-index spoken line and
  class-playbook step, the narrowed re-verify row, the goal-playback move, the
  mistake-seen-twice rule.
- `effort-at-the-door`: the dispatch door moves each dispatch's effort by kind
  and lane, stamps `[bee-kind:]`/`[bee-lane:]`, the model guard repairs or
  denies against the same ladder, economics carries four effort keys, `bee
  team show` prints the ladder.
- `pstack-playbook-rules`, `pstack-verify-trust`: doctrine in bee-swarming,
  bee-herding, bee-technical-writing, bee-planning and verify-app.
- `run-git-stdin-thread`: `run_git_stdin` cannot deadlock on a long patch-id
  stream.

New behavior and new payload keys for host repos are a minor bump.

## Risk named before the run

A direct `scripts/release.sh 2.55.0` on 2026-10-17 stopped at the permit:
`release authorization required — BEE_DISPATCH_ID is unset`; nothing changed.
The full crate on the effort-at-the-door head was green (4322 passed, 0
failed); the merged main has not run the whole declared suite since — the
script's test gate is that run, and a red there stops before any tag; the fix
is a fix-first cell, then a re-run at the same version (idempotent).

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
