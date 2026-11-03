promote proposal for work item "install-target-dir" (.bee/lanes/install-target-dir.json) — 1 capped cell(s): itd-1
anchor: ledger — .bee/lanes/install-target-dir.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/install-target-dir/delivery.md

---
type: bee.delivery
title: install-target-dir — delivery
description: "Delivery record proposed by bee knowledge promote for work item install-target-dir: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-31
bee:
  id: install-target-dir-delivery
  lifecycle: active
  required_context: [.bee/lanes/install-target-dir.json]
  sources: [.bee/lanes/install-target-dir.json, .bee/cells/itd-1.json]
---

# install-target-dir — Delivery

## What shipped

- **itd-1** — CARGO_TARGET_DIR pinned on both installer build lines; release manifest re-hashed; failing install reproduced green (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **itd-1** — `bash -n scripts/install.sh; bee dev release-manifest --check; reproduce the failing install with CARGO_TARGET_DIR set`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work install-target-dir` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/install-target-dir.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 0 pattern candidate(s), 0 file(s) written.