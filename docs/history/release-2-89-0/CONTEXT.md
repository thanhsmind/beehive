# Release 2.89.0 — Context

**Feature slug:** release-2-89-0
**Date:** 2026-11-10
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked for the release after approving uat for knowledge-in-the-flow ("Duyệt uat push code release").

## What will be done

Publish 2.89.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-88-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | ebc0350b |

## Why 2.89.0

The released version is 2.88.0. This release ships knowledge-in-the-flow: the
new `bee knowledge pack` and `bee knowledge consolidate` verbs, the knowledge
delta recorded on every cap, the pack in hat-wave and advisor briefs, and
feature arms in `bee knowledge report`. New verbs are a minor bump.

The plugin manifests are mechanical companions bumped by the script, so the
route counts 0 hand-authored files.

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
