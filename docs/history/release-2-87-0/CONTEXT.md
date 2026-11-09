# Release 2.87.0 — Context

**Feature slug:** release-2-87-0
**Date:** 2026-11-09
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked for a new release ("release bản mới").

## What will be done

Publish 2.87.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-86-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | e43ecfad |

## Why 2.87.0

The released version is 2.86.0. This release ships knowledge-freshness-mechanic
(freshness grading over the vc..HEAD graph, anchored-file cover at the cap door,
knowledge pulls counted per feature in timings and the report) and
knowledge-docs-in-the-flow (the read set in CONTEXT.md, refused shape or merged
approval without one, the decision-link error in knowledge check, template
section questions), plus the fable team roles and the writer role. New refusals
and a new role are a minor bump.

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
