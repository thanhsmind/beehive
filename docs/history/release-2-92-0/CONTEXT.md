# Release 2.92.0 — Context

**Feature slug:** release-2-92-0
**Date:** 2026-11-11
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner approved uat for cross-project-guard and asked for the release ("uat ok và release").

## What will be done

Publish 2.92.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-91-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | ebc0350b |

## Why 2.92.0

The released version is 2.91.1. This release ships cross-project-guard:
the write-guard refuses a read, search or shell command on another project's
tree and names the waggledance remedy, the leader doctrine gains rule
agents-other-project-to-its-leader, and config gains the governed key
guards.cross_project_allow. A new guard and a new config key make it a minor
bump.

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
