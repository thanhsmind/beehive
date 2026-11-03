promote proposal for work item "install-orchestrator-link" (docs/history/install-orchestrator-link/CONTEXT.md) — 2 capped cell(s): iol-1, iol-3
anchor: history — docs/history/install-orchestrator-link/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/install-orchestrator-link/delivery.md

---
type: bee.delivery
title: install-orchestrator-link — delivery
description: "Delivery record proposed by bee knowledge promote for work item install-orchestrator-link: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: install-orchestrator-link-delivery
  lifecycle: active
  required_context: [docs/history/install-orchestrator-link/CONTEXT.md]
  sources: [docs/history/install-orchestrator-link/CONTEXT.md, .bee/cells/archive/install-orchestrator-link/iol-1.json, .bee/cells/archive/install-orchestrator-link/iol-3.json]
---

# install-orchestrator-link — Delivery

## What shipped

- **iol-1** — README § Install carries the orchestrator (waggledance) subsection with the one-line installer and update note (1 file(s) changed)
- **iol-3** — Orchestrator install section moved verbatim README -> INSTALL.md (own ## section after Update/uninstall); README mention removed (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **iol-1** — `README § Install carries the orchestrator subsection; the linked raw URL resolves (HTTP 200).`
- **iol-3** — `README no longer mentions the waggledance installer; INSTALL.md carries the section; the raw install.sh URL resolves 200.`

## Deviations

- **iol-1** — followed the plan
- **iol-3** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work install-orchestrator-link` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/install-orchestrator-link/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell iol-1 — save as docs/knowledge/patterns/install-orchestrator-link-iol-1-pitfall.md

---
type: bee.pattern
title: install-orchestrator-link cell iol-1 — pitfall candidate
description: "Pitfall candidate mined from cell iol-1's capped trace: followed the plan"
timestamp: 2026-08-30
bee:
  id: install-orchestrator-link-iol-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/install-orchestrator-link/iol-1.json]
  polarity: pitfall
---

# install-orchestrator-link cell iol-1 — pitfall candidate

## What the cell did

README § Install carries the orchestrator (waggledance) subsection with the one-line installer and update note

## Recorded evidence (verbatim from .bee/cells/archive/install-orchestrator-link/iol-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell iol-3 — save as docs/knowledge/patterns/install-orchestrator-link-iol-3-pitfall.md

---
type: bee.pattern
title: install-orchestrator-link cell iol-3 — pitfall candidate
description: "Pitfall candidate mined from cell iol-3's capped trace: followed the plan"
timestamp: 2026-08-30
bee:
  id: install-orchestrator-link-iol-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/install-orchestrator-link/iol-3.json]
  polarity: pitfall
---

# install-orchestrator-link cell iol-3 — pitfall candidate

## What the cell did

Orchestrator install section moved verbatim README -> INSTALL.md (own ## section after Update/uninstall); README mention removed

## Recorded evidence (verbatim from .bee/cells/archive/install-orchestrator-link/iol-3.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.