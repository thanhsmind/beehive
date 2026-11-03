promote proposal for work item "nudge-consult" (docs/history/nudge-consult/CONTEXT.md) — 3 capped cell(s): nc-1, nc-2, nc-3
anchor: history — docs/history/nudge-consult/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/nudge-consult/delivery.md

---
type: bee.delivery
title: nudge-consult — delivery
description: "Delivery record proposed by bee knowledge promote for work item nudge-consult: 3 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-09-04
bee:
  id: nudge-consult-delivery
  lifecycle: active
  required_context: [docs/history/nudge-consult/CONTEXT.md]
  sources: [docs/history/nudge-consult/CONTEXT.md, .bee/cells/nc-1.json, .bee/cells/nc-2.json, .bee/cells/nc-3.json]
---

# nudge-consult — Delivery

## What shipped

- **nc-1** — Advisor line rendered into cell dispatch payloads, with the same-model no-op in the verb (6 file(s) changed)
- **nc-2** — The lead-run nudge consult is defined as the third bundle shape, with its dispatch verb, return form and debt-clearing decision text (6 file(s) changed)
- **nc-3** — The nudge consult's instruction text is pinned to the dispatch kind, record kind, signals and return form that code owns (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **nc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml drivers && bee dev regen && bee dev release-manifest --check`
- **nc-2** — `bee dev regen && bee dev release-manifest --check && rg -c 'nudge consult' skills/bee-swarming/references/worker-details.md`
- **nc-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml drivers`

## Deviations

- **nc-1** — The orchestrator committed and capped this cell instead of the dispatched worker — the herding worker returned outcome=done with a proof string but had run neither git commit nor bee cells finish, leaving six modified files and a claimed cell — hit an unforeseen obstacle
- **nc-2** — The orchestrator capped this cell instead of the dispatched worker — the worker committed cleanly but never ran bee cells finish, the same miss as nc-1 — hit an unforeseen obstacle
- **nc-3** — This cell did not exist at the merged gate — it was added after bee close refused on the pattern-check door, as the named remedy for pattern-20260821-instruction-text-is-an-untested-code-path — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work nudge-consult` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/nudge-consult/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell nc-1 — save as docs/knowledge/patterns/nudge-consult-nc-1-pitfall.md

---
type: bee.pattern
title: nudge-consult cell nc-1 — pitfall candidate
description: "Pitfall candidate mined from cell nc-1's capped trace: The orchestrator committed and capped this cell instead of the dispatched worker — the herding worker returned outcome=done with a proof string but had run nei…"
timestamp: 2026-09-04
bee:
  id: nudge-consult-nc-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/nc-1.json]
  polarity: pitfall
---

# nudge-consult cell nc-1 — pitfall candidate

## What the cell did

Advisor line rendered into cell dispatch payloads, with the same-model no-op in the verb

## Recorded evidence (verbatim from .bee/cells/nc-1.json)

- **deviation** — The orchestrator committed and capped this cell instead of the dispatched worker — the herding worker returned outcome=done with a proof string but had run neither git commit nor bee cells finish, leaving six modified files and a claimed cell — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell nc-2 — save as docs/knowledge/patterns/nudge-consult-nc-2-pitfall.md

---
type: bee.pattern
title: nudge-consult cell nc-2 — pitfall candidate
description: "Pitfall candidate mined from cell nc-2's capped trace: The orchestrator capped this cell instead of the dispatched worker — the worker committed cleanly but never ran bee cells finish, the same miss as nc-1 — hit a…"
timestamp: 2026-09-04
bee:
  id: nudge-consult-nc-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/nc-2.json]
  polarity: pitfall
---

# nudge-consult cell nc-2 — pitfall candidate

## What the cell did

The lead-run nudge consult is defined as the third bundle shape, with its dispatch verb, return form and debt-clearing decision text

## Recorded evidence (verbatim from .bee/cells/nc-2.json)

- **deviation** — The orchestrator capped this cell instead of the dispatched worker — the worker committed cleanly but never ran bee cells finish, the same miss as nc-1 — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell nc-3 — save as docs/knowledge/patterns/nudge-consult-nc-3-pitfall.md

---
type: bee.pattern
title: nudge-consult cell nc-3 — pitfall candidate
description: "Pitfall candidate mined from cell nc-3's capped trace: This cell did not exist at the merged gate — it was added after bee close refused on the pattern-check door, as the named remedy for pattern-20260821-instructi…"
timestamp: 2026-09-04
bee:
  id: nudge-consult-nc-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/nc-3.json]
  polarity: pitfall
---

# nudge-consult cell nc-3 — pitfall candidate

## What the cell did

The nudge consult's instruction text is pinned to the dispatch kind, record kind, signals and return form that code owns

## Recorded evidence (verbatim from .bee/cells/nc-3.json)

- **deviation** — This cell did not exist at the merged gate — it was added after bee close refused on the pattern-check door, as the named remedy for pattern-20260821-instruction-text-is-an-untested-code-path — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 3 pattern candidate(s), 0 file(s) written.