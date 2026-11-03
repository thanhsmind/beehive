promote proposal for work item "worker-guard-stub-stdin" (docs/history/worker-guard-stub-stdin/plan.md) — 1 capped cell(s): wgs-1
anchor: history — docs/history/worker-guard-stub-stdin/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/worker-guard-stub-stdin/delivery.md

---
type: bee.delivery
title: worker-guard-stub-stdin — delivery
description: "Delivery record proposed by bee knowledge promote for work item worker-guard-stub-stdin: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-07
bee:
  id: worker-guard-stub-stdin-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/worker-guard-stub-stdin/plan.md]
  sources: [docs/history/worker-guard-stub-stdin/plan.md, .bee/cells/wgs-1.json]
---

# worker-guard-stub-stdin — Delivery

## What shipped

- **wgs-1** — worker-guard stubs drain stdin; the Linux EPIPE flake is gone under stress (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wgs-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_worker_guard_contracts` — 7 passed; stress 8 parallel loops x 15 runs: allow test 0/360 failures after the fix (1/120 before), deny test 0/120

## Deviations

- **wgs-1** — drained stdin in all four stubs of the file, not only the allow and deny stubs — the noenv stub also receives write-guard JSON and could hit the same EPIPE — found a better route
- **wgs-1** — sync-ack: test-only change

## Provenance

Proposed by `bee knowledge promote --work worker-guard-stub-stdin` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/worker-guard-stub-stdin/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "worker-guard-stub-stdin" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-07T07:49:07.298Z), the work item declares no bee.areas.

area hook-runtime:
  (no capped behavior_change cell exists for this feature)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wgs-1 — save as docs/knowledge/patterns/worker-guard-stub-stdin-wgs-1-pitfall.md

---
type: bee.pattern
title: worker-guard-stub-stdin cell wgs-1 — pitfall candidate
description: "Pitfall candidate mined from cell wgs-1's capped trace: drained stdin in all four stubs of the file, not only the allow and deny stubs — the noenv stub also receives write-guard JSON and could hit the same EPIPE — f…"
timestamp: 2026-10-07
bee:
  id: worker-guard-stub-stdin-wgs-1-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/wgs-1.json]
  polarity: pitfall
---

# worker-guard-stub-stdin cell wgs-1 — pitfall candidate

## What the cell did

worker-guard stubs drain stdin; the Linux EPIPE flake is gone under stress

## Recorded evidence (verbatim from .bee/cells/wgs-1.json)

- **deviation** — drained stdin in all four stubs of the file, not only the allow and deny stubs — the noenv stub also receives write-guard JSON and could hit the same EPIPE — found a better route
- **deviation** — sync-ack: test-only change

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.