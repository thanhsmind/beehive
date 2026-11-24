# Release 2.121.0 — Context

**Feature slug:** release-2-121-0
**Date:** 2026-11-24
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner asked that bee leaders hand off and follow up through waggledance
on their own, never asking the owner to open a session or re-run a step, as
a bee mechanic every onboarded project gets. Feedback in beehive is a
framework change, and a framework change ships as a release.

## What will be done

Publish 2.121.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-120-0` did. The
release class playbook is `skills/bee-planning/playbooks/release.md`.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 0e1d1889 |
| C1 | The published v2.121.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.121.0; a test-role cell checks it read-only. | 6eb3dfe8 |

## Why 2.121.0

The released version is 2.120.0. Since then main gained leader-self-handoff:
each `bee state handoff send` attempt carries its own request id, a deduped
run ends `[handoff_deduped]`, every sent handoff writes a
`.bee/handoffs/<request_id>.json` row, the new `bee state handoff check`
dispatches a follow-up leader when a finished run left an expected merge
undone, and the Stop hook blocks a reply that asks the owner to open a new
session when a waggledance door exists. It also gained pi-1-6-upgrade: the
Pi belt stops its result drain at session shutdown and doctor names the live
pi package.

## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
