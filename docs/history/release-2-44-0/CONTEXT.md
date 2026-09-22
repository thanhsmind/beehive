# Release 2.44.0 — Context

**Feature slug:** release-2-44-0
**Date:** 2026-09-22
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user asked "release 2.44.0" after closing `host-packaging-gaps`.

## What will be done

Publish 2.44.0 from `main` through the one sanctioned command, under an authorized deployment dispatch, exactly as `release-2-43-0` did. The script bumps both plugin manifests, runs the regen chain, runs the declared test suite BEFORE anything is tagged, makes the release commit, tags, pushes `main` and the tag, waits for `release-binaries`, and verifies the published binaries and `SHA256SUMS`.

## Why 2.44.0

The released version is 2.43.0. This release ships `host-packaging-gaps`:

- `bee doctor --runtime pi` judges a host by its install record, and passes with no herdr or tmux when every Pi role runs a no-pane Pi process.
- Onboarding writes a default `team.pi` role table, and adds it once to an existing config.
- `--runtime pi` in `bee onboard`, `install.sh` and `install.ps1`.
- Release binaries for macOS (arm64, x86_64) and ARM64 Linux; the installer maps them, checks sums with `shasum`, and smoke-runs the binary.

New runtime value, new defaults and new release targets are a minor bump.

## Risk named before the run

This is the first run of the three new `release-binaries` rows (`macos-latest`, `macos-15-intel`, `ubuntu-22.04-arm`), and `release.sh` now requires five binaries. If a new row fails, no tag assets publish and the script stops red; the fix is a fix-first cell, then a re-run of the script at the same version (it is idempotent).

## Locked Decisions

Active repo-scope decisions, cited not restated.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `11dcf251-aaf7-41b0-a1d2-5e53f9ca92db` | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | A hand-walked checklist skipped tagging and pushing twice before. |
| D2 | `5d56be49-2aff-49bb-bf74-eb7857b04798` | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
