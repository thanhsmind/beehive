promote proposal for work item "harness-scratchpad-allowlist" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): hsa-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/harness-scratchpad-allowlist/delivery.md

---
type: bee.delivery
title: harness-scratchpad-allowlist — delivery
description: "Delivery record proposed by bee knowledge promote for work item harness-scratchpad-allowlist: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-06
bee:
  id: harness-scratchpad-allowlist-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/hsa-1.json]
---

# harness-scratchpad-allowlist — Delivery

## What shipped

- **hsa-1** — HarnessRoots::from_bases adds <temp>/claude-<getuid()> on unix, so the E1 scratchpad exemption matches the root the harness creates (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hsa-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml write_guard`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work harness-scratchpad-allowlist` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "harness-scratchpad-allowlist" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-06T09:57:03.393Z), the work item declares no bee.areas.

area hook-runtime:
  - [hsa-1] HarnessRoots::from_bases adds <temp>/claude-<getuid()> on unix, so the E1 scratchpad exemption matches the root the harness creates — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/hsa-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.