promote proposal for work item "start-feature-reservation-scope" (docs/history/start-feature-reservation-scope/plan.md) — 1 capped cell(s): sfrs-1
anchor: history — docs/history/start-feature-reservation-scope/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/start-feature-reservation-scope/delivery.md

---
type: bee.delivery
title: start-feature-reservation-scope — delivery
description: "Delivery record proposed by bee knowledge promote for work item start-feature-reservation-scope: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-18
bee:
  id: start-feature-reservation-scope-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/start-feature-reservation-scope/plan.md]
  sources: [docs/history/start-feature-reservation-scope/plan.md, .bee/cells/sfrs-1.json]
---

# start-feature-reservation-scope — Delivery

## What shipped

- **sfrs-1** — Scope start_default's reservation refusal to declared-path overlap or same-session holds (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sfrs-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **sfrs-1** — Also touched crates/bee/tests/workflow_verbs.rs (not a cell-declared file): its start_feature_refuses_over_an_active_reservation_with_zero_mutations test exercised exactly the old blanket-refusal behavior D1 removes, so it went red under the fix; renamed it to assert the new declared-path-overlap refusal and added a companion test for the now-allowed unrelated-path case, to keep the full-suite proof green.

## Provenance

Proposed by `bee knowledge promote --work start-feature-reservation-scope` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/start-feature-reservation-scope/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "start-feature-reservation-scope" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-18T13:02:46.191Z), the work item declares no bee.areas.

area workflow-state:
  - [sfrs-1] Scope start_default's reservation refusal to declared-path overlap or same-session holds — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sfrs-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell sfrs-1 — save as docs/knowledge/patterns/start-feature-reservation-scope-sfrs-1-pitfall.md

---
type: bee.pattern
title: start-feature-reservation-scope cell sfrs-1 — pitfall candidate
description: "Pitfall candidate mined from cell sfrs-1's capped trace: Also touched crates/bee/tests/workflow_verbs.rs (not a cell-declared file): its start_feature_refuses_over_an_active_reservation_with_zero_mutations test exerc…"
timestamp: 2026-08-18
bee:
  id: start-feature-reservation-scope-sfrs-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/sfrs-1.json]
  polarity: pitfall
---

# start-feature-reservation-scope cell sfrs-1 — pitfall candidate

## What the cell did

Scope start_default's reservation refusal to declared-path overlap or same-session holds

## Recorded evidence (verbatim from .bee/cells/sfrs-1.json)

- **deviation** — Also touched crates/bee/tests/workflow_verbs.rs (not a cell-declared file): its start_feature_refuses_over_an_active_reservation_with_zero_mutations test exercised exactly the old blanket-refusal behavior D1 removes, so it went red under the fix; renamed it to assert the new declared-path-overlap refusal and added a companion test for the now-allowed unrelated-path case, to keep the full-suite proof green.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.