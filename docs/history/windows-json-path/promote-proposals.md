promote proposal for work item "windows-json-path" (docs/history/windows-json-path/CONTEXT.md + docs/history/windows-json-path/plan.md) — 1 capped cell(s): wjp-1
anchor: history — docs/history/windows-json-path/CONTEXT.md, docs/history/windows-json-path/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/windows-json-path/delivery.md

---
type: bee.delivery
title: windows-json-path — delivery
description: "Delivery record proposed by bee knowledge promote for work item windows-json-path: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-22
bee:
  id: windows-json-path-delivery
  lifecycle: active
  required_context: [docs/history/windows-json-path/CONTEXT.md, docs/history/windows-json-path/plan.md]
  sources: [docs/history/windows-json-path/CONTEXT.md, docs/history/windows-json-path/plan.md, .bee/cells/wjp-1.json]
---

# windows-json-path — Delivery

## What shipped

- **wjp-1** — Built the herding ledger test's result JSON with serde_json::json! so Windows paths parse (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wjp-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee ledger_outcome_blocked_with_report_fills_evidence` — the one rewritten test, run on Linux; the Windows path shape is proven only by the verify-windows CI job after the push

## Deviations

- **wjp-1** — The source edit landed a few seconds before the lane's merged gate was recorded in the same batch; recorded with bee mailbox reflect
- **wjp-1** — sync-ack: test-only fix of a JSON escape in one herding unit test; no herding behavior or skill text changes

## Provenance

Proposed by `bee knowledge promote --work windows-json-path` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/windows-json-path/CONTEXT.md`, `docs/history/windows-json-path/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wjp-1 — save as docs/knowledge/patterns/windows-json-path-wjp-1-pitfall.md

---
type: bee.pattern
title: windows-json-path cell wjp-1 — pitfall candidate
description: "Pitfall candidate mined from cell wjp-1's capped trace: The source edit landed a few seconds before the lane's merged gate was recorded in the same batch; recorded with bee mailbox reflect"
timestamp: 2026-09-22
bee:
  id: windows-json-path-wjp-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/wjp-1.json]
  polarity: pitfall
---

# windows-json-path cell wjp-1 — pitfall candidate

## What the cell did

Built the herding ledger test's result JSON with serde_json::json! so Windows paths parse

## Recorded evidence (verbatim from .bee/cells/wjp-1.json)

- **deviation** — The source edit landed a few seconds before the lane's merged gate was recorded in the same batch; recorded with bee mailbox reflect
- **deviation** — sync-ack: test-only fix of a JSON escape in one herding unit test; no herding behavior or skill text changes

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.