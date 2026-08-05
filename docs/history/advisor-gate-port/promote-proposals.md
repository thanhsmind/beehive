promote proposal for work item "advisor-gate-port" (.bee/logs/scribing-runs.jsonl) — 2 capped cell(s): agp-1, agp-2
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/advisor-gate-port/delivery.md

---
type: bee.delivery
title: advisor-gate-port — delivery
description: "Delivery record proposed by bee knowledge promote for work item advisor-gate-port: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-05
bee:
  id: advisor-gate-port-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/agp-1.json, .bee/cells/agp-2.json]
---

# advisor-gate-port — Delivery

## What shipped

- **agp-1** — Shipped in commit fb94ba8f; advisor_ref.rs carries the anchors, staleness rule, and the record/show verbs. (3 file(s) changed)
- **agp-2** — Shipped in commit 6fefd6ee; set_gate.rs carries the high_risk_advisor_refusal precondition off a fresh advisor consult. (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **agp-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml; then `bee state advisor-ref show --lane advisor-gate-port` must report no ref recorded, and after a record it must report fresh.`
- **agp-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml; then live: record a consult on lane advisor-gate-port and confirm `bee state gate --lane advisor-gate-port --merge --approved true` now succeeds, and that it refuses again once a new decision is logged.`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work advisor-gate-port` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "advisor-gate-port" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-05T13:32:32.814Z), the work item declares no bee.areas.

area workflow-state:
  - [agp-1] Shipped in commit fb94ba8f; advisor_ref.rs carries the anchors, staleness rule, and the record/show verbs. — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/agp-1.json)
  - [agp-2] Shipped in commit 6fefd6ee; set_gate.rs carries the high_risk_advisor_refusal precondition off a fresh advisor consult. — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/agp-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 0 pattern candidate(s), 0 file(s) written.