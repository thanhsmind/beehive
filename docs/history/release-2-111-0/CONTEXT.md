# Release 2.111.0 — Context

**Feature slug:** release-2-111-0
**Date:** 2026-11-17
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner approved uat for brief-intent-and-wip and outcome-scorecard and asked to release ("Duyệt uat release").

## What will be done

Publish 2.111.0 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-110-0` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the declared test path (clippy `-D warnings`, then the cargo suite) and the live smoke (no `--no-test`), the CI-pinned opencode 1.18.16 first on PATH and a private CARGO_TARGET_DIR; it is done only when the script prints its final `OK`. | 292386c8 |
| C1 | The published v2.111.0 carries the binaries and SHA256SUMS, the tag is on origin, and both manifests report 2.111.0; a test-role cell checks it read-only. | 0ca81e7e |

## Why 2.111.0

The released version is 2.110.0. This release ships brief-intent-and-wip (cell briefs carry why and out_of_scope, Qualify asks should-we questions, bee feature start warns past wip_limit) and outcome-scorecard (bee report outcomes --by-feature, herded caps keep reaction, every gate change is audited, fixes_feature). New host-visible behavior makes it a minor bump.


## Read set

<!-- bee:no-read-set: release execution; the only reads are scripts/release.sh and the previous release lane -->
