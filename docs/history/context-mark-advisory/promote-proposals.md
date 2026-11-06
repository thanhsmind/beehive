promote proposal for work item "context-mark-advisory" (docs/history/context-mark-advisory/CONTEXT.md + docs/history/context-mark-advisory/plan.md) — 2 capped cell(s): cma-1, cma-2
anchor: history — docs/history/context-mark-advisory/CONTEXT.md, docs/history/context-mark-advisory/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/context-mark-advisory/delivery.md

---
type: bee.delivery
title: context-mark-advisory — delivery
description: "Delivery record proposed by bee knowledge promote for work item context-mark-advisory: 2 capped cell(s), 5 recorded deviation(s)."
timestamp: 2026-11-06
bee:
  id: context-mark-advisory-delivery
  lifecycle: active
  areas: [hook-runtime, doctrine-layer]
  required_context: [docs/history/context-mark-advisory/CONTEXT.md, docs/history/context-mark-advisory/plan.md]
  sources: [docs/history/context-mark-advisory/CONTEXT.md, docs/history/context-mark-advisory/plan.md, .bee/cells/cma-1.json, .bee/cells/cma-2.json]
---

# context-mark-advisory — Delivery

## What shipped

- **cma-1** — Make the context mark line advisory (1 file(s) changed)
- **cma-2** — Rewrite the context handoff rule as advisory in every home (49 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cma-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- prompt_context` — 43 prompt_context tests, red first on the old line text
- **cma-2** — `.bee/bin/bee dev release-manifest --check` — 600 manifest files match after bee dev regen; rg finds no stop-at-mark wording outside the worker [HANDOFF] token and history

## Deviations

- **cma-1** — ran inline on the leader: dispatch prepare refused limits.concurrency (decision logged)
- **cma-1** — sync-ack: cma-1 changed only prompt_context.rs; the AGENTS.md diff on this branch is cma-2's and touches only rule agents-context-handoff-65, agents-bee-defect-report text untouched
- **cma-2** — ran inline on the leader: dispatch prepare refused limits.concurrency (decision logged)
- **cma-2** — regen rewrote the rendered skill copies, .bee/onboarding.json and verify-app feature-gates.md copies that lagged an earlier source change
- **cma-2** — sync-ack: only rule agents-context-handoff-65 changed in AGENTS.md; status-display-vendoring.md cites the mark only for statusline colour, which stays; agents-bee-defect-report untouched

## Provenance

Proposed by `bee knowledge promote --work context-mark-advisory` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/context-mark-advisory/CONTEXT.md`, `docs/history/context-mark-advisory/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "context-mark-advisory" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-06T07:05:14.120Z), the work item declares no bee.areas.

area hook-runtime:
  - [cma-1] Make the context mark line advisory — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/cma-1.json)

area doctrine-layer:
  - [cma-1] Make the context mark line advisory — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/cma-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell cma-1 — save as docs/knowledge/patterns/context-mark-advisory-cma-1-pitfall.md

---
type: bee.pattern
title: context-mark-advisory cell cma-1 — pitfall candidate
description: "Pitfall candidate mined from cell cma-1's capped trace: ran inline on the leader: dispatch prepare refused limits.concurrency (decision logged)"
timestamp: 2026-11-06
bee:
  id: context-mark-advisory-cma-1-pitfall
  lifecycle: draft
  areas: [hook-runtime, doctrine-layer]
  sources: [.bee/cells/cma-1.json]
  polarity: pitfall
---

# context-mark-advisory cell cma-1 — pitfall candidate

## What the cell did

Make the context mark line advisory

## Recorded evidence (verbatim from .bee/cells/cma-1.json)

- **deviation** — ran inline on the leader: dispatch prepare refused limits.concurrency (decision logged)
- **deviation** — sync-ack: cma-1 changed only prompt_context.rs; the AGENTS.md diff on this branch is cma-2's and touches only rule agents-context-handoff-65, agents-bee-defect-report text untouched

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell cma-2 — save as docs/knowledge/patterns/context-mark-advisory-cma-2-pitfall.md

---
type: bee.pattern
title: context-mark-advisory cell cma-2 — pitfall candidate
description: "Pitfall candidate mined from cell cma-2's capped trace: ran inline on the leader: dispatch prepare refused limits.concurrency (decision logged)"
timestamp: 2026-11-06
bee:
  id: context-mark-advisory-cma-2-pitfall
  lifecycle: draft
  areas: [hook-runtime, doctrine-layer]
  sources: [.bee/cells/cma-2.json]
  polarity: pitfall
---

# context-mark-advisory cell cma-2 — pitfall candidate

## What the cell did

Rewrite the context handoff rule as advisory in every home

## Recorded evidence (verbatim from .bee/cells/cma-2.json)

- **deviation** — ran inline on the leader: dispatch prepare refused limits.concurrency (decision logged)
- **deviation** — regen rewrote the rendered skill copies, .bee/onboarding.json and verify-app feature-gates.md copies that lagged an earlier source change
- **deviation** — sync-ack: only rule agents-context-handoff-65 changed in AGENTS.md; status-display-vendoring.md cites the mark only for statusline colour, which stays; agents-bee-defect-report untouched

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 2 pattern candidate(s), 0 file(s) written.