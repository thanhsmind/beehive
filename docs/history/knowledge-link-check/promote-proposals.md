promote proposal for work item "knowledge-link-check" (.bee/logs/scribing-runs.jsonl) — 2 capped cell(s): klc-1, klc-2
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/knowledge-link-check/delivery.md

---
type: bee.delivery
title: knowledge-link-check — delivery
description: "Delivery record proposed by bee knowledge promote for work item knowledge-link-check: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: knowledge-link-check-delivery
  lifecycle: active
  areas: [okf-profile]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/klc-1.json, .bee/cells/klc-2.json]
---

# knowledge-link-check — Delivery

## What shipped

- **klc-1** — check_bundle now warns on dangling body links (md and wiki) (2 file(s) changed)
- **klc-2** — Added a host-repo docs/knowledge link integrity test (skips outside the repo) and fixed the five dangling wiki links it surfaced (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **klc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **klc-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work knowledge-link-check` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "knowledge-link-check" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-11T10:59:53.580Z), the work item declares no bee.areas.

area okf-profile:
  - [klc-1] check_bundle now warns on dangling body links (md and wiki) — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/klc-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.