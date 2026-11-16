# Release 2.109.0 — Context

**Feature slug:** release-2-109-0
**Date:** 2026-11-16
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked to release ("Release") after pushing main.

## What will be done

Publish 2.109.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-108-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | f9eaf44c |

## Why 2.109.0

The released version is 2.108.0. This release ships lint-setup-guidance (every
standard-commands notice names `bee config set --key commands.test --value
'<JSON list>'`, and bee-hive gains a "Set up the project's checks" procedure,
so an agent told to set up lint does it instead of reporting a missing
permission) and team-global-config (a global config layer, `.bee/team.json`,
`bee config split-team`, and guards on the global config file). New
host-visible behavior makes it a minor bump.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
