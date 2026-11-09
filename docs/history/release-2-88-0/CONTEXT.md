# Release 2.88.0 — Context

**Feature slug:** release-2-88-0
**Date:** 2026-11-09
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked for the release after approving uat for tiny-bound-delegation ("approve uat rồi release").

## What will be done

Publish 2.88.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-87-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 537d8a6d |

## Why 2.88.0

The released version is 2.87.0. This release ships tiny-bound-delegation: tiny
means one hand-authored file, `bee route` refuses `--lane tiny` with `--files`
above 1, and an inline tiny cap with more than one counted file is refused
without `--inline-reason`. New refusals are a minor bump.

The plugin manifests are mechanical companions bumped by the script, so the
route counts 0 hand-authored files; the inline cap records `--inline-reason`
(tiny-bound-delegation D2, D4).

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md | suspect | B21 to B23: the release permit, the deployment-stage dispatch and the release lane order. |
| docs/knowledge/areas/onboarding/release-identity-and-version-parity.md | suspect | One release version across every projection; the manifests the script bumps. |

### Code

| Path | What it decides |
|------|-----------------|
| scripts/release.sh | The whole release: bump, regen, test, commit, tag, push, wait for release-binaries, verify assets. |
