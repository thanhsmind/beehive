---
type: bee.delivery
title: release-2-57-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-57-0: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-10-21
bee:
  id: release-2-57-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-57-0/CONTEXT.md, docs/history/release-2-57-0/plan.md]
  sources: [docs/history/release-2-57-0/CONTEXT.md, docs/history/release-2-57-0/plan.md, .bee/cells/rel570-1.json]
---

# release-2-57-0 — Delivery

## What shipped

- **rel570-1** — bee 2.57.0 released: manifests 2.57.0, declared suite green before the tag, release commit c0e628937, tag v2.57.0 pushed, release-binaries run 39569147269 green, five binaries and SHA256SUMS published (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel570-1** — `bash scripts/release.sh 2.57.0 && bee dev release-manifest --check` — the script test gate ran the declared cargo suite before tagging with BEE_OPENCODE_SUITE_ALLOW_SKIP=1 (opencode on this machine is a wrapper script); release-binaries run https://github.com/thanhsmin…

## Deviations

- **rel570-1** — ran scripts/release.sh inline in the leader session under deployment permit, not through the paseo-pi-flash deploy worker — the worker failed to spawn twice in the Paseo main workspace (spawn pi ENOENT) and the user chose the inline run — hit an unforeseen obstacle
- **rel570-1** — test gate ran with BEE_OPENCODE_SUITE_ALLOW_SKIP=1 — opencode on PATH is a wrapper script on this machine; CI runs that suite — hit an unforeseen obstacle
- **rel570-1** — set skip-worktree on .bee/supervisor/broker-heartbeat.json for the run and cleared it after, as in 2.56.0 — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work release-2-57-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-57-0/CONTEXT.md`, `docs/history/release-2-57-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
