promote proposal for work item "cli-help-shape-guard" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): chsg-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/cli-help-shape-guard/delivery.md

---
type: bee.delivery
title: cli-help-shape-guard — delivery
description: "Delivery record proposed by bee knowledge promote for work item cli-help-shape-guard: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-06
bee:
  id: cli-help-shape-guard-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/chsg-1.json]
---

# cli-help-shape-guard — Delivery

## What shipped

- **chsg-1** — check_cli_shape breaks out before validating when a help key was parsed, so bee <group> <cmd> --help reaches the CLI help surface (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **chsg-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml cli_shape`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work cli-help-shape-guard` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "cli-help-shape-guard" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-06T09:24:46.987Z), the work item declares no bee.areas.

area hook-runtime:
  - [chsg-1] check_cli_shape breaks out before validating when a help key was parsed, so bee <group> <cmd> --help reaches the CLI help surface — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/chsg-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.