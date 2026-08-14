promote proposal for work item "full-failure-evidence" (docs/history/full-failure-evidence/CONTEXT.md + docs/history/full-failure-evidence/plan.md) — 2 capped cell(s): ffe-1, ffe-2
anchor: history — docs/history/full-failure-evidence/CONTEXT.md, docs/history/full-failure-evidence/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/full-failure-evidence/delivery.md

---
type: bee.delivery
title: full-failure-evidence — delivery
description: "Delivery record proposed by bee knowledge promote for work item full-failure-evidence: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-14
bee:
  id: full-failure-evidence-delivery
  lifecycle: active
  areas: [verify-pipeline]
  required_context: [docs/history/full-failure-evidence/CONTEXT.md, docs/history/full-failure-evidence/plan.md]
  sources: [docs/history/full-failure-evidence/CONTEXT.md, docs/history/full-failure-evidence/plan.md, .bee/cells/archive/full-failure-evidence/ffe-1.json, .bee/cells/archive/full-failure-evidence/ffe-2.json]
---

# full-failure-evidence — Delivery

## What shipped

- **ffe-1** — Extracted the three failure-excerpt blocks into crate::fsutil::failure_excerpt; every existing test passes with only the three FAILURE_EXCERPT_MAX references retargeted (4 file(s) changed)
- **ffe-2** — Wired the test-runner runner and the refusal-text log line; register.md and tests updated (8 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ffe-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ffe-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work full-failure-evidence` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/full-failure-evidence/CONTEXT.md`, `docs/history/full-failure-evidence/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "full-failure-evidence" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-14T04:12:41.941Z), the work item declares no bee.areas.

area verify-pipeline:
  - [ffe-2] Wired the test-runner runner and the refusal-text log line; register.md and tests updated — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/archive/full-failure-evidence/ffe-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.