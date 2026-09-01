promote proposal for work item "plan-conflicts-scope" (docs/history/plan-conflicts-scope/CONTEXT.md) — 2 capped cell(s): pcs-1, pcs-2
anchor: history — docs/history/plan-conflicts-scope/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/plan-conflicts-scope/delivery.md

---
type: bee.delivery
title: plan-conflicts-scope — delivery
description: "Delivery record proposed by bee knowledge promote for work item plan-conflicts-scope: 2 capped cell(s), 5 recorded deviation(s)."
timestamp: 2026-09-01
bee:
  id: plan-conflicts-scope-delivery
  lifecycle: active
  required_context: [docs/history/plan-conflicts-scope/CONTEXT.md]
  sources: [docs/history/plan-conflicts-scope/CONTEXT.md, .bee/cells/pcs-1.json, .bee/cells/pcs-2.json]
---

# plan-conflicts-scope — Delivery

## What shipped

- **pcs-1** — Red-first row pins derive to a proportionate candidate list on a 241-decision store; it fails today naming 241, and two green rows guard the fixture premise and D3 small-store floor (1 file(s) changed)
- **pcs-2** — The plan term set now drops terms above 3 percent document frequency once the store holds 200 active decisions, and the decision candidate list is ranked by hit count and capped at 50 (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pcs-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast -p bee --manifest-path packages/bee-rs/Cargo.toml`
- **pcs-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **pcs-1** — followed the plan
- **pcs-1** — sync-ack: Test-only cell: it adds rows to state_group/tests.rs and changes no behavior, so no owned skill of area workflow-state has anything to restate. The behavior change, and any skill sync it earns, belongs to pcs-2.
- **pcs-2** — D4 ranking is applied only when the 50-candidate cap actually truncates — an unconditional sort would reorder an under-cap list and break must_have 4, which promises an under-floor list identical to today — found a better route
- **pcs-2** — plan_terms keeps its one-argument signature and the filter runs as drop_saturating_terms inside derive_candidates, with a new derive_candidates_reported carrying the truncation count — the already-green fixture test calls plan_terms with one argument and asserts the UNFILTERED term set, so a signature change would have broken it — the plan was wrong about a fact
- **pcs-2** — sync-ack: No owned skill of area workflow-state describes the derive term filter or its thresholds; the change is internal to how the term set is built, the cell declares affects_skills empty by plan, and pcs-1 already pinned the new behavior in tests.rs

## Provenance

Proposed by `bee knowledge promote --work plan-conflicts-scope` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/plan-conflicts-scope/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pcs-1 — save as docs/knowledge/patterns/plan-conflicts-scope-pcs-1-pitfall.md

---
type: bee.pattern
title: plan-conflicts-scope cell pcs-1 — pitfall candidate
description: "Pitfall candidate mined from cell pcs-1's capped trace: followed the plan"
timestamp: 2026-09-01
bee:
  id: plan-conflicts-scope-pcs-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/pcs-1.json]
  polarity: pitfall
---

# plan-conflicts-scope cell pcs-1 — pitfall candidate

## What the cell did

Red-first row pins derive to a proportionate candidate list on a 241-decision store; it fails today naming 241, and two green rows guard the fixture premise and D3 small-store floor

## Recorded evidence (verbatim from .bee/cells/pcs-1.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: Test-only cell: it adds rows to state_group/tests.rs and changes no behavior, so no owned skill of area workflow-state has anything to restate. The behavior change, and any skill sync it earns, belongs to pcs-2.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pcs-2 — save as docs/knowledge/patterns/plan-conflicts-scope-pcs-2-pitfall.md

---
type: bee.pattern
title: plan-conflicts-scope cell pcs-2 — pitfall candidate
description: "Pitfall candidate mined from cell pcs-2's capped trace: D4 ranking is applied only when the 50-candidate cap actually truncates — an unconditional sort would reorder an under-cap list and break must_have 4, which pr…"
timestamp: 2026-09-01
bee:
  id: plan-conflicts-scope-pcs-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/pcs-2.json]
  polarity: pitfall
---

# plan-conflicts-scope cell pcs-2 — pitfall candidate

## What the cell did

The plan term set now drops terms above 3 percent document frequency once the store holds 200 active decisions, and the decision candidate list is ranked by hit count and capped at 50

## Recorded evidence (verbatim from .bee/cells/pcs-2.json)

- **deviation** — D4 ranking is applied only when the 50-candidate cap actually truncates — an unconditional sort would reorder an under-cap list and break must_have 4, which promises an under-floor list identical to today — found a better route
- **deviation** — plan_terms keeps its one-argument signature and the filter runs as drop_saturating_terms inside derive_candidates, with a new derive_candidates_reported carrying the truncation count — the already-green fixture test calls plan_terms with one argument and asserts the UNFILTERED term set, so a signature change would have broken it — the plan was wrong about a fact
- **deviation** — sync-ack: No owned skill of area workflow-state describes the derive term filter or its thresholds; the change is internal to how the term set is built, the cell declares affects_skills empty by plan, and pcs-1 already pinned the new behavior in tests.rs

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.