promote proposal for work item "herding-activity-hook" (docs/history/herding-activity-hook/CONTEXT.md) — 2 capped cell(s): hact-1, hact-2
anchor: history — docs/history/herding-activity-hook/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-activity-hook/delivery.md

---
type: bee.delivery
title: herding-activity-hook — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-activity-hook: 2 capped cell(s), 6 recorded deviation(s)."
timestamp: 2026-08-23
bee:
  id: herding-activity-hook-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-activity-hook/CONTEXT.md]
  sources: [docs/history/herding-activity-hook/CONTEXT.md, .bee/cells/hact-1.json, .bee/cells/hact-2.json]
---

# herding-activity-hook — Delivery

## What shipped

- **hact-1** — The activity hook runs in a herded pane and writes the job mailbox activity record (2 file(s) changed)
- **hact-2** — The run verb reads the pane's own activity.json ahead of the screen classifier at all three wait points, fenced by round and a 120s freshness bound (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hact-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee hooks::`
- **hact-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::`

## Deviations

- **hact-1** — Mailbox path arithmetic is restated in activity.rs (herding::mailbox is private to the herding module, and mailbox.rs belongs to a parallel cell)
- **hact-1** — A herded pane sets no waiting_on mark — that sink is bee-session state, and the pane holds no bee session
- **hact-2** — deliver_pointer took an 8th parameter, an activity_working probe, so the stall branch reacts only to the hook's own working and the hookless path stays byte-identical; 15 existing test call sites pass false for it
- **hact-2** — four export-line assertions updated for the new BEE_HERDING_JOB_ID entry (required by D2, not a behavior drift)
- **hact-2** — SYNC_DOOR acked: the cell declares affects_skills: [] and names no skill or knowledge file; the knowledge sync belongs to the scribe pass
- **hact-2** — sync-ack: cell hact-2 declares affects_skills: [] and names only the two herding source files; the operator-facing surface is unchanged (no new flag, no new failure wording), and the cell's own affects_specs points at docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md, which is not in this cell's files - a skill or knowledge edit here would be an out-of-scope write for a worker

## Provenance

Proposed by `bee knowledge promote --work herding-activity-hook` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-activity-hook/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-activity-hook" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-23T06:16:28.667Z), the work item declares no bee.areas.

area bee-herding:
  - [hact-1] The activity hook runs in a herded pane and writes the job mailbox activity record — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/hact-1.json)
  - [hact-2] The run verb reads the pane's own activity.json ahead of the screen classifier at all three wait points, fenced by round and a 120s freshness bound — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/hact-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hact-1 — save as docs/knowledge/patterns/herding-activity-hook-hact-1-pitfall.md

---
type: bee.pattern
title: herding-activity-hook cell hact-1 — pitfall candidate
description: "Pitfall candidate mined from cell hact-1's capped trace: Mailbox path arithmetic is restated in activity.rs (herding::mailbox is private to the herding module, and mailbox.rs belongs to a parallel cell)"
timestamp: 2026-08-23
bee:
  id: herding-activity-hook-hact-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hact-1.json]
  polarity: pitfall
---

# herding-activity-hook cell hact-1 — pitfall candidate

## What the cell did

The activity hook runs in a herded pane and writes the job mailbox activity record

## Recorded evidence (verbatim from .bee/cells/hact-1.json)

- **deviation** — Mailbox path arithmetic is restated in activity.rs (herding::mailbox is private to the herding module, and mailbox.rs belongs to a parallel cell)
- **deviation** — A herded pane sets no waiting_on mark — that sink is bee-session state, and the pane holds no bee session

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hact-2 — save as docs/knowledge/patterns/herding-activity-hook-hact-2-pitfall.md

---
type: bee.pattern
title: herding-activity-hook cell hact-2 — pitfall candidate
description: "Pitfall candidate mined from cell hact-2's capped trace: deliver_pointer took an 8th parameter, an activity_working probe, so the stall branch reacts only to the hook's own working and the hookless path stays byte-id…"
timestamp: 2026-08-23
bee:
  id: herding-activity-hook-hact-2-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hact-2.json]
  polarity: pitfall
---

# herding-activity-hook cell hact-2 — pitfall candidate

## What the cell did

The run verb reads the pane's own activity.json ahead of the screen classifier at all three wait points, fenced by round and a 120s freshness bound

## Recorded evidence (verbatim from .bee/cells/hact-2.json)

- **deviation** — deliver_pointer took an 8th parameter, an activity_working probe, so the stall branch reacts only to the hook's own working and the hookless path stays byte-identical; 15 existing test call sites pass false for it
- **deviation** — four export-line assertions updated for the new BEE_HERDING_JOB_ID entry (required by D2, not a behavior drift)
- **deviation** — SYNC_DOOR acked: the cell declares affects_skills: [] and names no skill or knowledge file; the knowledge sync belongs to the scribe pass
- **deviation** — sync-ack: cell hact-2 declares affects_skills: [] and names only the two herding source files; the operator-facing surface is unchanged (no new flag, no new failure wording), and the cell's own affects_specs points at docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md, which is not in this cell's files - a skill or knowledge edit here would be an out-of-scope write for a worker

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 2 pattern candidate(s), 0 file(s) written.