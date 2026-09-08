promote proposal for work item "herding-agent-brief" (.bee/logs/scribing-runs.jsonl + .bee/lanes/herding-agent-brief.json + docs/history/herding-agent-brief/promote-proposals.md) — 2 capped cell(s): hab-1, hab-2
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/herding-agent-brief.json, docs/history/herding-agent-brief/promote-proposals.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-agent-brief/delivery.md

---
type: bee.delivery
title: herding-agent-brief — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-agent-brief: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-08
bee:
  id: herding-agent-brief-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/herding-agent-brief.json, docs/history/herding-agent-brief/promote-proposals.md]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/herding-agent-brief.json, docs/history/herding-agent-brief/promote-proposals.md, .bee/cells/archive/herding-agent-brief/hab-1.json, .bee/cells/archive/herding-agent-brief/hab-2.json]
---

# herding-agent-brief — Delivery

## What shipped

- **hab-1** — Embedded the four bee agent bodies in prompt.rs and moved split_frontmatter to textutil.rs as the single parser (3 file(s) changed)
- **hab-2** — Every non-Agent dispatch payload now carries the related bee agent body above its kind brief (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hab-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers::prompt onboard::agents`
- **hab-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers`

## Deviations

- **hab-1** — followed the plan
- **hab-2** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work herding-agent-brief` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/herding-agent-brief.json`, `docs/history/herding-agent-brief/promote-proposals.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-agent-brief" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-08T14:06:23.529Z), the work item declares no bee.areas.

area bee-herding:
  - [hab-2] Every non-Agent dispatch payload now carries the related bee agent body above its kind brief — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/archive/herding-agent-brief/hab-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hab-1 — save as docs/knowledge/patterns/herding-agent-brief-hab-1-pitfall.md

---
type: bee.pattern
title: herding-agent-brief cell hab-1 — pitfall candidate
description: "Pitfall candidate mined from cell hab-1's capped trace: followed the plan"
timestamp: 2026-09-08
bee:
  id: herding-agent-brief-hab-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/archive/herding-agent-brief/hab-1.json]
  polarity: pitfall
---

# herding-agent-brief cell hab-1 — pitfall candidate

## What the cell did

Embedded the four bee agent bodies in prompt.rs and moved split_frontmatter to textutil.rs as the single parser

## Recorded evidence (verbatim from .bee/cells/archive/herding-agent-brief/hab-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hab-2 — save as docs/knowledge/patterns/herding-agent-brief-hab-2-pitfall.md

---
type: bee.pattern
title: herding-agent-brief cell hab-2 — pitfall candidate
description: "Pitfall candidate mined from cell hab-2's capped trace: followed the plan"
timestamp: 2026-09-08
bee:
  id: herding-agent-brief-hab-2-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/archive/herding-agent-brief/hab-2.json]
  polarity: pitfall
---

# herding-agent-brief cell hab-2 — pitfall candidate

## What the cell did

Every non-Agent dispatch payload now carries the related bee agent body above its kind brief

## Recorded evidence (verbatim from .bee/cells/archive/herding-agent-brief/hab-2.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 2 pattern candidate(s), 0 file(s) written.