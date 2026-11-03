# Release 2.65.0 — Context

**Feature slug:** release-2-65-0
**Date:** 2026-10-28
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The user approved the `owner-brief` uat on 2026-10-28 ("duyệt"). The spec drop sup-20261028-c9d4 asks for the release version that carries owner briefs, so other repos can upgrade.

## What will be done

Publish 2.65.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-64-0` did.

## Locked decisions

- D1 (`084eb232`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch; it is done only when the script prints its
  final `OK`.

## Why 2.65.0

The released version is 2.64.0. This release ships owner briefs: `--brief-file`
on `bee backlog pbi add` and `bee backlog propose`, the `bee backlog briefs`
owner view and check, the doctor `owner_briefs` note, `brief.language`, and the
skill and knowledge updates (owner-brief). A new host-visible verb is a minor bump.

## Risk named before the run

The local opencode install (2.0.32 through a mise wrapper) cannot be read by the
opencode test; the gate runs with BEE_OPENCODE_SUITE_ALLOW_SKIP=1, as recorded
for releases 2.60.0 to 2.64.0; CI runs that suite.
