promote proposal for work item "device-aware-cap" (docs/history/device-aware-cap/CONTEXT.md + docs/history/device-aware-cap/plan.md) — 2 capped cell(s): dac-1, dac-2
anchor: history — docs/history/device-aware-cap/CONTEXT.md, docs/history/device-aware-cap/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/device-aware-cap/delivery.md

---
type: bee.delivery
title: device-aware-cap — delivery
description: "Delivery record proposed by bee knowledge promote for work item device-aware-cap: 2 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-11-05
bee:
  id: device-aware-cap-delivery
  lifecycle: active
  required_context: [docs/history/device-aware-cap/CONTEXT.md, docs/history/device-aware-cap/plan.md]
  sources: [docs/history/device-aware-cap/CONTEXT.md, docs/history/device-aware-cap/plan.md, .bee/cells/dac-1.json, .bee/cells/dac-2.json]
---

# device-aware-cap — Delivery

## What shipped

- **dac-1** — Make herding worker limits aware of CPU count and free memory e893db34f60c34026d95f75475d9404afc5de92b (1 file(s) changed)
- **dac-2** — Run worktree prune sweep after a green close 6ec864db0 (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dac-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml wave::` — cargo test --release --manifest-path packages/bee-rs/Cargo.toml wave:: — green:unit — 68 tests passed covering CPU concurrency defaults, memory floor, and existing wave limits
- **dac-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml close::` — unit tests covering post-close prune invocation, dry-run/refusal bypass, and error recovery

## Deviations

- **dac-1** — sync-ack: skills/bee-herding names only the cockpit four-slot cap, which is outside check_limits; the limit change is documented in docs/knowledge/areas/bee-herding/waves-and-occupancy.md

## Provenance

Proposed by `bee knowledge promote --work device-aware-cap` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/device-aware-cap/CONTEXT.md`, `docs/history/device-aware-cap/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell dac-1 — save as docs/knowledge/patterns/device-aware-cap-dac-1-pitfall.md

---
type: bee.pattern
title: device-aware-cap cell dac-1 — pitfall candidate
description: "Pitfall candidate mined from cell dac-1's capped trace: sync-ack: skills/bee-herding names only the cockpit four-slot cap, which is outside check_limits; the limit change is documented in docs/knowledge/areas/bee-he…"
timestamp: 2026-11-05
bee:
  id: device-aware-cap-dac-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/dac-1.json]
  polarity: pitfall
---

# device-aware-cap cell dac-1 — pitfall candidate

## What the cell did

Make herding worker limits aware of CPU count and free memory e893db34f60c34026d95f75475d9404afc5de92b

## Recorded evidence (verbatim from .bee/cells/dac-1.json)

- **deviation** — sync-ack: skills/bee-herding names only the cockpit four-slot cap, which is outside check_limits; the limit change is documented in docs/knowledge/areas/bee-herding/waves-and-occupancy.md

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.