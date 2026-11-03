# Release 2.53.0 — Context

**Feature slug:** release-2-53-0
**Date:** 2026-10-15
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked on 2026-10-15: "chạy release rồi push lên".

## What will be done

Publish 2.53.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-52-0` did. The script
bumps both plugin manifests, runs the regen chain, runs the declared test
suite BEFORE anything is tagged, makes the release commit, tags, pushes
`main` and the tag, waits for `release-binaries`, and verifies the published
binaries and `SHA256SUMS`.

## Why 2.53.0

The released version is 2.52.0. This release ships four closed features:

- `paseo-cli-autodetect`: bee finds the CLI of the running Paseo daemon with no config.
- `paseo-worktree-workspace`: each bee worktree is a Paseo workspace, and herded workers start in it.
- `paseo-system-prompts`: a Paseo schedule prompt no longer clears a human wait mark, and the pi plugin tests never reach a real Paseo daemon.
- `paseo-worker-subagent`: a Paseo herded worker is a subagent of the lead that started it.

Behavior changes for host repos are a minor bump.

## Risk named before the run

The full declared suite had one failing test on 2026-10-15,
`pi_plugin_contracts::exit_worktree_marker_relocates_and_no_merge_command_runs`,
reported as failing at its base too. If the script's test gate is red, it stops
before any tag; the fix is a fix-first cell, then a re-run of the script at the
same version (it is idempotent).

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
