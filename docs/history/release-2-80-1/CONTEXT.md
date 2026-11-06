# Release 2.80.1 — Context

**Feature slug:** release-2-80-1
**Date:** 2026-11-06
**Scope:** Tiny (release execution)
**Domain types:** RUN

## What was asked

The owner approved on 2026-11-06 a release of thin-worker-boot, so the
thin-boot pilot and the leader doctrine line run on the new binary.

## What will be done

Publish 2.80.1 from `main` through the one sanctioned command, under an
authorized deployment dispatch, exactly as `release-2-79-1` did.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The release runs only through `scripts/release.sh` under an authorized deployment dispatch, with the full test suite (no `--no-test`) and the CI-pinned opencode 1.18.16 first on PATH; it is done only when the script prints its final `OK`. | e8ed9dfa |

## Why 2.80.1

The released version is 2.80.0. This release ships thin-worker-boot: the
herding merged-config fix (issue 166), pane token counts, the thin peer
boot behind `dispatch.thin_peer_boot`, and the split of the bee block into
a thin shared block, `.bee/LEADER.md` and leader playbooks.
