promote proposal for work item "pi-slp-dispatch-return" (docs/history/pi-slp-dispatch-return/CONTEXT.md + docs/history/pi-slp-dispatch-return/plan.md) — 2 capped cell(s): psdr-1, psdr-2
anchor: history — docs/history/pi-slp-dispatch-return/CONTEXT.md, docs/history/pi-slp-dispatch-return/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-slp-dispatch-return/delivery.md

---
type: bee.delivery
title: pi-slp-dispatch-return — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-slp-dispatch-return: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-03
bee:
  id: pi-slp-dispatch-return-delivery
  lifecycle: active
  required_context: [docs/history/pi-slp-dispatch-return/CONTEXT.md, docs/history/pi-slp-dispatch-return/plan.md]
  sources: [docs/history/pi-slp-dispatch-return/CONTEXT.md, docs/history/pi-slp-dispatch-return/plan.md, .bee/cells/psdr-1.json, .bee/cells/psdr-2.json]
---

# pi-slp-dispatch-return — Delivery

## What shipped

- **psdr-1** — A herded cell brief carries no bee command and no bee-build body; native brief byte-identical (3 file(s) changed)
- **psdr-2** — Pi bee_dispatch passes stage, feature, expertise and claim; verdict binds to its job and refuses blank proof and repeats; done cell results name the cap step (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **psdr-1** — `cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::drivers` — 429 passed; golden native render test and herding stdin test added in drivers/tests.rs
- **psdr-2** — `cd packages/bee-rs && cargo test --release -p bee --test pi_plugin_contracts` — 110 passed (104 at base); new verdict refusal, dispatch flag and injection tests

## Deviations

- **psdr-1** — the worker narrowed the prompt_skew test for worker-cell instead of leaving the stale .bee/bin/prompts copy to the wave-barrier regen; the leader ran bee dev regen and restored the original assertion in eb71124e1 — something else had to be fixed first
- **psdr-2** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work pi-slp-dispatch-return` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-slp-dispatch-return/CONTEXT.md`, `docs/history/pi-slp-dispatch-return/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell psdr-1 — save as docs/knowledge/patterns/pi-slp-dispatch-return-psdr-1-pitfall.md

---
type: bee.pattern
title: pi-slp-dispatch-return cell psdr-1 — pitfall candidate
description: "Pitfall candidate mined from cell psdr-1's capped trace: the worker narrowed the prompt_skew test for worker-cell instead of leaving the stale .bee/bin/prompts copy to the wave-barrier regen; the leader ran bee dev r…"
timestamp: 2026-10-03
bee:
  id: pi-slp-dispatch-return-psdr-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/psdr-1.json]
  polarity: pitfall
---

# pi-slp-dispatch-return cell psdr-1 — pitfall candidate

## What the cell did

A herded cell brief carries no bee command and no bee-build body; native brief byte-identical

## Recorded evidence (verbatim from .bee/cells/psdr-1.json)

- **deviation** — the worker narrowed the prompt_skew test for worker-cell instead of leaving the stale .bee/bin/prompts copy to the wave-barrier regen; the leader ran bee dev regen and restored the original assertion in eb71124e1 — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell psdr-2 — save as docs/knowledge/patterns/pi-slp-dispatch-return-psdr-2-pitfall.md

---
type: bee.pattern
title: pi-slp-dispatch-return cell psdr-2 — pitfall candidate
description: "Pitfall candidate mined from cell psdr-2's capped trace: followed the plan"
timestamp: 2026-10-03
bee:
  id: pi-slp-dispatch-return-psdr-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/psdr-2.json]
  polarity: pitfall
---

# pi-slp-dispatch-return cell psdr-2 — pitfall candidate

## What the cell did

Pi bee_dispatch passes stage, feature, expertise and claim; verdict binds to its job and refuses blank proof and repeats; done cell results name the cap step

## Recorded evidence (verbatim from .bee/cells/psdr-2.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.