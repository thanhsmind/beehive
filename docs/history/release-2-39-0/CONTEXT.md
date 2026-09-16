# Release 2.39.0 — Context

**Feature slug:** release-2-39-0
**Date:** 2026-09-16
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user approved the `uat` gate for `pi-relocation-delivery` and asked, in one
line, to push the code and cut a release.

## What was found

`scripts/release.sh` refuses to run without a release permit: it requires
`BEE_DISPATCH_ID` naming a dispatch prepared under the `deploy` role at the
`deployment` stage, and `bee dispatch authorize` then validates that record
against the requested version, the current main commit, the approved plan hash,
the stage, the role, the issuing session and a two-hour lifetime. A dispatch for
that stage requires an approved `bee-plan/v2` plan carrying a role plan, so the
release needs its own plan — the closed feature's legacy plan cannot carry it.

## What will be done

Publish 2.39.0 from `main` through the one sanctioned command. The script owns
the whole release: bump both plugin manifests, run the regen chain, run the
declared test suite BEFORE anything is tagged, make the release commit, tag,
push `main` and the tag (which carries the 22 commits already waiting), wait for
the `release-binaries` workflow to go green, and verify the published release
carries the binaries and `SHA256SUMS`.

## Why 2.39.0

Current released version is 2.38.0. This release ships a new public CLI verb
(`bee cells rebind-session`) and changed Pi belt behavior (the relocation carry
and the claim reassignment), so the minor number moves.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch — never by hand-walking bump, tag or push. | The script exists because a hand-walked checklist skipped tagging and pushing twice before (2.6.6, 2.6.7), and the permit machinery is what makes the deploy role's authority auditable. |
| D2 | The release is done ONLY when the script prints its final `OK` line — tag pushed, CI green, assets verified. | A release commit without that line is not a release. |

## Out of Scope

- Any source change: this lane publishes what `main` already holds.
- Re-proving `pi-relocation-delivery`; it closed with its proof and its uat approval.
