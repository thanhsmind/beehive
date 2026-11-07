promote proposal for work item "report-usage-herding" (docs/history/report-usage-herding/plan.md) — 1 capped cell(s): ruh-2
anchor: history — docs/history/report-usage-herding/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/report-usage-herding/delivery.md

---
type: bee.delivery
title: report-usage-herding — delivery
description: "Delivery record proposed by bee knowledge promote for work item report-usage-herding: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-11-07
bee:
  id: report-usage-herding-delivery
  lifecycle: active
  areas: [workflow-state, bee-herding]
  required_context: [docs/history/report-usage-herding/plan.md]
  sources: [docs/history/report-usage-herding/plan.md, .bee/cells/ruh-2.json]
---

# report-usage-herding — Delivery

## What shipped

- **ruh-2** — bee report usage now merges herded worker sessions (job mailbox leader_session match, deduped, window-filtered) into the model rows and prints a herding-session count; registry help, LEADER.md and the transport concept updated; commit b6bbf91da (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ruh-2** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml report_usage` — 16 report_usage tests incl. 5 new herding ones; red shown with the reader stubbed. Full suite: 2 unrelated herding::run thin_peer tests fail only under this worker's ambient BEE_HERDING_*/CLAUDE_CODE…

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work report-usage-herding` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/report-usage-herding/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "report-usage-herding" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-07T01:24:05.390Z), the work item declares no bee.areas.

area workflow-state:
  - [ruh-2] bee report usage now merges herded worker sessions (job mailbox leader_session match, deduped, window-filtered) into the model rows and prints a herding-session count; registry help, LEADER.md and the transport concept updated; commit b6bbf91da — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/ruh-2.json)

area bee-herding:
  - [ruh-2] bee report usage now merges herded worker sessions (job mailbox leader_session match, deduped, window-filtered) into the model rows and prints a herding-session count; registry help, LEADER.md and the transport concept updated; commit b6bbf91da — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/ruh-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 0 pattern candidate(s), 0 file(s) written.