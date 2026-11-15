# Release 2.105.0 — Context

**Feature slug:** release-2-105-0
**Date:** 2026-11-15
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked to push the code and release it.

## What will be done

Publish 2.105.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-104-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 50750a60 |

## Why 2.105.0

The released version is 2.104.0. This release ships peer-brief-craft (a herded
peer reads AGENTS.md, worker briefs point at the expertise depth layer, the
herding brief asks for principles_used, and `bee knowledge report` compares the
principles arm) and orient-host-playbook (`bee orient --class` prints playbook
paths that exist in the host project and a next line, the Stop hook warns on a
routed reply that names no principle or opens no playbook, and the bee-hive
close carries a Principles line). New behavior makes it a minor bump.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
