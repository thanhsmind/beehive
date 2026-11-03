promote proposal for work item "ui-ux-pro-max-a11y-port" (.bee/lanes/ui-ux-pro-max-a11y-port.json) — 1 capped cell(s): uxp-1
anchor: ledger — .bee/lanes/ui-ux-pro-max-a11y-port.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/ui-ux-pro-max-a11y-port/delivery.md

---
type: bee.delivery
title: ui-ux-pro-max-a11y-port — delivery
description: "Delivery record proposed by bee knowledge promote for work item ui-ux-pro-max-a11y-port: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-29
bee:
  id: ui-ux-pro-max-a11y-port-delivery
  lifecycle: active
  required_context: [.bee/lanes/ui-ux-pro-max-a11y-port.json]
  sources: [.bee/lanes/ui-ux-pro-max-a11y-port.json, .bee/cells/uxp-1.json]
---

# ui-ux-pro-max-a11y-port — Delivery

## What shipped

- **uxp-1** — bee-design gains additions/wcag-2-2.md with six WCAG 2.2 rules, linked from SKILL.md; manifest regenerated (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **uxp-1** — `rg -c "additions/wcag-2-2.md" packages/bee/design/bee-design/SKILL.md && .bee/bin/bee dev release-manifest --check` — docs-only: link count 1 and 569 files match the manifest; cargo suite not run, no code changed

## Deviations

- **uxp-1** — tiny docs cell run inline by the leader
- **uxp-1** — research brief rides the same commit as provenance

## Provenance

Proposed by `bee knowledge promote --work ui-ux-pro-max-a11y-port` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/ui-ux-pro-max-a11y-port.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell uxp-1 — save as docs/knowledge/patterns/ui-ux-pro-max-a11y-port-uxp-1-pitfall.md

---
type: bee.pattern
title: ui-ux-pro-max-a11y-port cell uxp-1 — pitfall candidate
description: "Pitfall candidate mined from cell uxp-1's capped trace: tiny docs cell run inline by the leader"
timestamp: 2026-10-29
bee:
  id: ui-ux-pro-max-a11y-port-uxp-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/uxp-1.json]
  polarity: pitfall
---

# ui-ux-pro-max-a11y-port cell uxp-1 — pitfall candidate

## What the cell did

bee-design gains additions/wcag-2-2.md with six WCAG 2.2 rules, linked from SKILL.md; manifest regenerated

## Recorded evidence (verbatim from .bee/cells/uxp-1.json)

- **deviation** — tiny docs cell run inline by the leader
- **deviation** — research brief rides the same commit as provenance

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.