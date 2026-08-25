promote proposal for work item "claim-reserves-files" (docs/history/claim-reserves-files/CONTEXT.md) — 1 capped cell(s): crf-1
anchor: history — docs/history/claim-reserves-files/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/claim-reserves-files/delivery.md

---
type: bee.delivery
title: claim-reserves-files — delivery
description: "Delivery record proposed by bee knowledge promote for work item claim-reserves-files: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-08-25
bee:
  id: claim-reserves-files-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/claim-reserves-files/CONTEXT.md]
  sources: [docs/history/claim-reserves-files/CONTEXT.md, .bee/cells/crf-1.json]
---

# claim-reserves-files — Delivery

## What shipped

- **crf-1** — cells claim and claim-next now reserve the claimed cell's declared files under the claiming worker, matching the release the cap already performs; a conflict refuses typed and rolls the store back (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **crf-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml cells::`

## Deviations

- **crf-1** — reserve_path_atomic hardcodes session: None, so the claim calls the same shared pair one layer down (reserve_prechecks + reserve_exec) to thread its --session-id; without it every claim in concurrent mode would refuse with reserve's own SESSION_REQUIRED
- **crf-1** — rollback releases by (worker, cell), the only scoping the shared release door offers: if this call created a lease AND the worker also held an older lease for the same cell, the older one goes too. Recorded in the function doc; nothing is released when this call created nothing
- **crf-1** — sync-ack: skills/bee-swarming/SKILL.md:105 already states the guarantee this cell implements ('its listed files reserved under your nickname') — the skill was right and the code was wrong, so there is nothing in the area's skills to change; the cell declares affects_skills: []

## Provenance

Proposed by `bee knowledge promote --work claim-reserves-files` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/claim-reserves-files/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "claim-reserves-files" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-25T09:30:45.835Z), the work item declares no bee.areas.

area workflow-state:
  - [crf-1] cells claim and claim-next now reserve the claimed cell's declared files under the claiming worker, matching the release the cap already performs; a conflict refuses typed and rolls the store back — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/crf-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell crf-1 — save as docs/knowledge/patterns/claim-reserves-files-crf-1-pitfall.md

---
type: bee.pattern
title: claim-reserves-files cell crf-1 — pitfall candidate
description: "Pitfall candidate mined from cell crf-1's capped trace: reserve_path_atomic hardcodes session: None, so the claim calls the same shared pair one layer down (reserve_prechecks + reserve_exec) to thread its --session-…"
timestamp: 2026-08-25
bee:
  id: claim-reserves-files-crf-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/crf-1.json]
  polarity: pitfall
---

# claim-reserves-files cell crf-1 — pitfall candidate

## What the cell did

cells claim and claim-next now reserve the claimed cell's declared files under the claiming worker, matching the release the cap already performs; a conflict refuses typed and rolls the store back

## Recorded evidence (verbatim from .bee/cells/crf-1.json)

- **deviation** — reserve_path_atomic hardcodes session: None, so the claim calls the same shared pair one layer down (reserve_prechecks + reserve_exec) to thread its --session-id; without it every claim in concurrent mode would refuse with reserve's own SESSION_REQUIRED
- **deviation** — rollback releases by (worker, cell), the only scoping the shared release door offers: if this call created a lease AND the worker also held an older lease for the same cell, the older one goes too. Recorded in the function doc; nothing is released when this call created nothing
- **deviation** — sync-ack: skills/bee-swarming/SKILL.md:105 already states the guarantee this cell implements ('its listed files reserved under your nickname') — the skill was right and the code was wrong, so there is nothing in the area's skills to change; the cell declares affects_skills: []

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.