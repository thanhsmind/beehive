promote proposal for work item "stageshow-frame-rehearsal" (.bee/lanes/stageshow-frame-rehearsal.json) — 2 capped cell(s): sfr-1, sfr-2
anchor: ledger — .bee/lanes/stageshow-frame-rehearsal.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/stageshow-frame-rehearsal/delivery.md

---
type: bee.delivery
title: stageshow-frame-rehearsal — delivery
description: "Delivery record proposed by bee knowledge promote for work item stageshow-frame-rehearsal: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-11-02
bee:
  id: stageshow-frame-rehearsal-delivery
  lifecycle: active
  required_context: [.bee/lanes/stageshow-frame-rehearsal.json]
  sources: [.bee/lanes/stageshow-frame-rehearsal.json, .bee/cells/sfr-1.json, .bee/cells/sfr-2.json]
---

# stageshow-frame-rehearsal — Delivery

## What shipped

- **sfr-1** — Ported the six generic stageshow scripts comment-free into bee-design-motion/additions/rehearse, added a new sheet.mjs grid capture, a node:test suite, the MIT license file and additions/frame-rehearsal.md, and linked it from the pack SKILL.md — commit 167c1249f, cell sfr-1. (11 file(s) changed)
- **sfr-2** — Added additions/frame-review.md critic brief and linked it from bee-design SKILL.md table and Check step (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sfr-1** — `node --test packages/bee/design/bee-design-motion/additions/rehearse/rehearse.test.mjs` — node --test packages/bee/design/bee-design-motion/additions/rehearse/rehearse.test.mjs — green:unit — 8/8 pass over the pure functions (jump vs glide vs appear, late-frame skip, layoutResult exit-cod…
- **sfr-2** — `rg -n "additions/frame-review.md" packages/bee/design/bee-design/SKILL.md` — docs-only cell; verify hits lines 102 and 172; a separate loop also confirmed all three relative links in frame-review.md resolve

## Deviations

- **sfr-2** — Credited Stageshow as MIT with its file name but no upstream URL or copyright holder — the local source copy carries neither and I did not invent one — hit an unforeseen obstacle
- **sfr-2** — Defined the closest gold example as the brief-named product or the nearest benchmark from the taste-rubric reference duel — bee-design has no examples folder like Stageshow — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work stageshow-frame-rehearsal` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/stageshow-frame-rehearsal.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell sfr-2 — save as docs/knowledge/patterns/stageshow-frame-rehearsal-sfr-2-pitfall.md

---
type: bee.pattern
title: stageshow-frame-rehearsal cell sfr-2 — pitfall candidate
description: "Pitfall candidate mined from cell sfr-2's capped trace: Credited Stageshow as MIT with its file name but no upstream URL or copyright holder — the local source copy carries neither and I did not invent one — hit an …"
timestamp: 2026-11-02
bee:
  id: stageshow-frame-rehearsal-sfr-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/sfr-2.json]
  polarity: pitfall
---

# stageshow-frame-rehearsal cell sfr-2 — pitfall candidate

## What the cell did

Added additions/frame-review.md critic brief and linked it from bee-design SKILL.md table and Check step

## Recorded evidence (verbatim from .bee/cells/sfr-2.json)

- **deviation** — Credited Stageshow as MIT with its file name but no upstream URL or copyright holder — the local source copy carries neither and I did not invent one — hit an unforeseen obstacle
- **deviation** — Defined the closest gold example as the brief-named product or the nearest benchmark from the taste-rubric reference duel — bee-design has no examples folder like Stageshow — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.