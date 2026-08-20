promote proposal for work item "herding-bare-agent" (docs/history/herding-bare-agent/CONTEXT.md) — 2 capped cell(s): hba-1, hba-2
anchor: history — docs/history/herding-bare-agent/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-bare-agent/delivery.md

---
type: bee.delivery
title: herding-bare-agent — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-bare-agent: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-20
bee:
  id: herding-bare-agent-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-bare-agent/CONTEXT.md]
  sources: [docs/history/herding-bare-agent/CONTEXT.md, .bee/cells/hba-1.json, .bee/cells/hba-2.json]
---

# herding-bare-agent — Delivery

## What shipped

- **hba-1** — A bare herding run resolves its agent from the generation tier slot (1 file(s) changed)
- **hba-2** — Both reader surfaces state the four-step bare-run agent resolution order (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hba-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hba-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml registry`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work herding-bare-agent` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-bare-agent/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-bare-agent" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-20T12:21:18.651Z), the work item declares no bee.areas.

area bee-herding:
  - [hba-1] A bare herding run resolves its agent from the generation tier slot — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/hba-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.