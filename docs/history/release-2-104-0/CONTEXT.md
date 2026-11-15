# Release 2.104.0 — Context

**Feature slug:** release-2-104-0
**Date:** 2026-11-15
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner approved uat for pstack-playbook-port, closed it, and asked to release it.

## What will be done

Publish 2.104.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-103-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 8097faaf |

## Why 2.104.0

The released version is 2.103.0. This release ships pstack-playbook-port: the
23 pstack playbooks as situations under the nine route classes and a session
group, `bee orient --class` listing a class's situations, the plan check holding
a plan to a situation file, eleven new skills (bee-architect, bee-arena,
bee-interrogate, bee-show-me-your-work, bee-fanout, bee-tdd, bee-reflect,
bee-recall, bee-blast-radius, bee-comment-review, bee-figure-it-out) and
thirteen new principles. New behavior and new skills make it a minor bump.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
