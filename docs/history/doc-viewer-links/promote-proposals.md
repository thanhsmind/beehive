promote proposal for work item "doc-viewer-links" (docs/history/doc-viewer-links/plan.md) — 2 capped cell(s): dvl-1, dvl-2
anchor: history — docs/history/doc-viewer-links/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/doc-viewer-links/delivery.md

---
type: bee.delivery
title: doc-viewer-links — delivery
description: "Delivery record proposed by bee knowledge promote for work item doc-viewer-links: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-05
bee:
  id: doc-viewer-links-delivery
  lifecycle: active
  areas: [workflow-state, agent-contract]
  required_context: [docs/history/doc-viewer-links/plan.md]
  sources: [docs/history/doc-viewer-links/plan.md, .bee/cells/archive/doc-viewer-links/dvl-1.json, .bee/cells/archive/doc-viewer-links/dvl-2.json]
---

# doc-viewer-links — Delivery

## What shipped

- **dvl-1** — Added doc_viewer_prefix reader in state.rs and rendered it in the session preamble (Doc links section) and compaction capsule, with unit and render coverage; full cargo test suite green (4 file(s) changed)
- **dvl-2** — Write the doc-link rule into the agent contract and document the key (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dvl-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **dvl-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml && bee dev release-manifest --check`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work doc-viewer-links` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/doc-viewer-links/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "doc-viewer-links" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-05T04:19:39.954Z), the work item declares no bee.areas.

area workflow-state:
  - [dvl-1] Added doc_viewer_prefix reader in state.rs and rendered it in the session preamble (Doc links section) and compaction capsule, with unit and render coverage; full cargo test suite green — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/archive/doc-viewer-links/dvl-1.json)
  - [dvl-2] Write the doc-link rule into the agent contract and document the key — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/archive/doc-viewer-links/dvl-2.json)

area agent-contract:
  - [dvl-1] Added doc_viewer_prefix reader in state.rs and rendered it in the session preamble (Doc links section) and compaction capsule, with unit and render coverage; full cargo test suite green — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/archive/doc-viewer-links/dvl-1.json)
  - [dvl-2] Write the doc-link rule into the agent contract and document the key — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/archive/doc-viewer-links/dvl-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 0 pattern candidate(s), 0 file(s) written.