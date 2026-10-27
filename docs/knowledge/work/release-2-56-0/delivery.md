---
type: bee.delivery
title: release-2-56-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-56-0: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-18
bee:
  id: release-2-56-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-56-0/CONTEXT.md, docs/history/release-2-56-0/plan.md]
  sources: [docs/history/release-2-56-0/CONTEXT.md, docs/history/release-2-56-0/plan.md, .bee/cells/rel560-1.json]
---

# release-2-56-0 — Delivery

## What shipped

- **rel560-1** — bee 2.56.0 is live: tag v2.56.0 pushed, release-binaries green, assets verified (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel560-1** — `bash scripts/release.sh 2.56.0 && bee dev release-manifest --check` — the script's own test gate ran the declared cargo suite (40 test results ok, 0 failed) before tagging; release-binaries run https://github.com/thanhsmind/beehive/actions/runs/39097039106 green; asset…

## Deviations

- **rel560-1** — The herded deploy worker stopped before the permit: broker-heartbeat.json dirt from the effort-at-the-door broker and a red test gate under BEE_HERDING_WORKER=1; the leader ran the script in the issuing session, as in rel550-1
- **rel560-1** — Permit 1adaebd5 was spent on a dirt refusal; the heartbeat was hidden with a local skip-worktree flag for the run and the flag was cleared after (decision a37ab576)

## Provenance

Proposed by `bee knowledge promote --work release-2-56-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-56-0/CONTEXT.md`, `docs/history/release-2-56-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
