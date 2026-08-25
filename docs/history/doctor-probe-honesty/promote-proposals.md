promote proposal for work item "doctor-probe-honesty" (docs/history/doctor-probe-honesty/CONTEXT.md) — 1 capped cell(s): dph-1
anchor: history — docs/history/doctor-probe-honesty/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/doctor-probe-honesty/delivery.md

---
type: bee.delivery
title: doctor-probe-honesty — delivery
description: "Delivery record proposed by bee knowledge promote for work item doctor-probe-honesty: 1 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-08-25
bee:
  id: doctor-probe-honesty-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/doctor-probe-honesty/CONTEXT.md]
  sources: [docs/history/doctor-probe-honesty/CONTEXT.md, .bee/cells/dph-1.json]
---

# doctor-probe-honesty — Delivery

## What shipped

- **dph-1** — A failed probe with nothing newer now reports ok: None naming the failed rs-info probe, never 'matches source' (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dph-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml doctor::`

## Deviations

- **dph-1** — Added a second test (probe Failed AND a newer source input) to pin the unchanged row next to the new one, per CONTEXT acceptance
- **dph-1** — Added three doc-comment lines above binary_freshness_row so the doc states the new unknown case
- **dph-1** — First run hit the known ETXTBSY probe flake in binary_freshness_reports_not_ok_on_a_version_mismatch (CONTEXT lines 34-46); pre-existing and out of scope, green on the next three runs
- **dph-1** — Capped with --inline-reason: worker not registered in state.json workers[] by the orchestrator

## Provenance

Proposed by `bee knowledge promote --work doctor-probe-honesty` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/doctor-probe-honesty/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "doctor-probe-honesty" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-25T09:42:02.626Z), the work item declares no bee.areas.

area hook-runtime:
  - [dph-1] A failed probe with nothing newer now reports ok: None naming the failed rs-info probe, never 'matches source' — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/dph-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell dph-1 — save as docs/knowledge/patterns/doctor-probe-honesty-dph-1-pitfall.md

---
type: bee.pattern
title: doctor-probe-honesty cell dph-1 — pitfall candidate
description: "Pitfall candidate mined from cell dph-1's capped trace: Added a second test (probe Failed AND a newer source input) to pin the unchanged row next to the new one, per CONTEXT acceptance"
timestamp: 2026-08-25
bee:
  id: doctor-probe-honesty-dph-1-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/dph-1.json]
  polarity: pitfall
---

# doctor-probe-honesty cell dph-1 — pitfall candidate

## What the cell did

A failed probe with nothing newer now reports ok: None naming the failed rs-info probe, never 'matches source'

## Recorded evidence (verbatim from .bee/cells/dph-1.json)

- **deviation** — Added a second test (probe Failed AND a newer source input) to pin the unchanged row next to the new one, per CONTEXT acceptance
- **deviation** — Added three doc-comment lines above binary_freshness_row so the doc states the new unknown case
- **deviation** — First run hit the known ETXTBSY probe flake in binary_freshness_reports_not_ok_on_a_version_mismatch (CONTEXT lines 34-46); pre-existing and out of scope, green on the next three runs
- **deviation** — Capped with --inline-reason: worker not registered in state.json workers[] by the orchestrator

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.