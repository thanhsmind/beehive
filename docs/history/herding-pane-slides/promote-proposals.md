promote proposal for work item "herding-pane-slides" (.bee/lanes/herding-pane-slides.json + docs/history/herding-pane-slides/promote-proposals.md) — 2 capped cell(s): hpsl-1, hpsl-2
anchor: ledger — .bee/lanes/herding-pane-slides.json, docs/history/herding-pane-slides/promote-proposals.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-pane-slides/delivery.md

---
type: bee.delivery
title: herding-pane-slides — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-pane-slides: 2 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-21
bee:
  id: herding-pane-slides-delivery
  lifecycle: active
  required_context: [.bee/lanes/herding-pane-slides.json, docs/history/herding-pane-slides/promote-proposals.md]
  sources: [.bee/lanes/herding-pane-slides.json, docs/history/herding-pane-slides/promote-proposals.md, .bee/cells/archive/herding-pane-slides/hpsl-1.json, .bee/cells/archive/herding-pane-slides/hpsl-2.json]
---

# herding-pane-slides — Delivery

## What shipped

- **hpsl-1** — split direction is now fixed: right for a tab's first split, down after that; width halves at most once (1 file(s) changed)
- **hpsl-2** — the fixed split rule is now stated in role-dispatch, wave-runs, spawn-proof's retired-reason note, and the run-verb knowledge page (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hpsl-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee split`
- **hpsl-2** — `.bee/bin/bee dev regen && .bee/bin/bee dev release-manifest --check && rg -n 'wider than tall' skills docs | wc -l`

## Deviations

- **hpsl-2** — Wrote the knowledge line into the-run-verb-and-worker-outcomes.md (where bee herding run's split is described) instead of the cell's named agent-resolution-and-spawn-commands.md, which says nothing about direction; also touched spawn-proof.md to mark its retired reason rather than leave a dead rule standing.

## Provenance

Proposed by `bee knowledge promote --work herding-pane-slides` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/herding-pane-slides.json`, `docs/history/herding-pane-slides/promote-proposals.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hpsl-2 — save as docs/knowledge/patterns/herding-pane-slides-hpsl-2-pitfall.md

---
type: bee.pattern
title: herding-pane-slides cell hpsl-2 — pitfall candidate
description: "Pitfall candidate mined from cell hpsl-2's capped trace: Wrote the knowledge line into the-run-verb-and-worker-outcomes.md (where bee herding run's split is described) instead of the cell's named agent-resolution-and…"
timestamp: 2026-08-21
bee:
  id: herding-pane-slides-hpsl-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/herding-pane-slides/hpsl-2.json]
  polarity: pitfall
---

# herding-pane-slides cell hpsl-2 — pitfall candidate

## What the cell did

the fixed split rule is now stated in role-dispatch, wave-runs, spawn-proof's retired-reason note, and the run-verb knowledge page

## Recorded evidence (verbatim from .bee/cells/archive/herding-pane-slides/hpsl-2.json)

- **deviation** — Wrote the knowledge line into the-run-verb-and-worker-outcomes.md (where bee herding run's split is described) instead of the cell's named agent-resolution-and-spawn-commands.md, which says nothing about direction; also touched spawn-proof.md to mark its retired reason rather than leave a dead rule standing.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.