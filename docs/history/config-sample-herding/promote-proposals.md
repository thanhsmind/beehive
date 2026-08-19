promote proposal for work item "config-sample-herding" (docs/history/config-sample-herding/CONTEXT.md) — 2 capped cell(s): cs-1, cs-2
anchor: history — docs/history/config-sample-herding/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/config-sample-herding/delivery.md

---
type: bee.delivery
title: config-sample-herding — delivery
description: "Delivery record proposed by bee knowledge promote for work item config-sample-herding: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-19
bee:
  id: config-sample-herding-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/config-sample-herding/CONTEXT.md]
  sources: [docs/history/config-sample-herding/CONTEXT.md, .bee/cells/cs-1.json, .bee/cells/cs-2.json]
---

# config-sample-herding — Delivery

## What shipped

- **cs-1** — Added herding block + _doc to both config samples; config-reference points at operational-invariants.md (3 file(s) changed)
- **cs-2** — onboard seeds .bee/config-sample.json create-if-missing via compile-time include_str; test guards drift (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cs-1** — `python3 -m json.tool .bee/config-sample.json > /dev/null && python3 -m json.tool .bee/config-sample-cli-executors.json > /dev/null && echo parse-ok`
- **cs-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml onboard`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work config-sample-herding` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/config-sample-herding/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "config-sample-herding" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-19T23:38:24.706Z), the work item declares no bee.areas.

area onboarding:
  - [cs-2] onboard seeds .bee/config-sample.json create-if-missing via compile-time include_str; test guards drift — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/cs-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.