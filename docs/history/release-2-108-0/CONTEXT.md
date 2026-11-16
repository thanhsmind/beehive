# Release 2.108.0 — Context

**Feature slug:** release-2-108-0
**Date:** 2026-11-16
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked to push and release the merged work ("Push code release").

## What will be done

Publish 2.108.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-107-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | d9862069 |

## Why 2.108.0

The released version is 2.107.0. This release ships decision-concept-links
(`bee decisions search` links each decision to the concepts that cite it),
monorepo-manifest-detect (the onboard detector finds Cargo.toml, pyproject.toml,
go.mod and *.csproj up to two directories down), clippy-in-test (the release
script runs a list-valued `commands.test` entry by entry) and
clippy-warnings-clean (bee-rs is clippy-clean under `-D warnings`). New
host-visible behavior makes it a minor bump.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
