promote proposal for work item "decision-attribution" (docs/history/decision-attribution/CONTEXT.md + docs/history/decision-attribution/plan.md) — 3 capped cell(s): da-1, da-2, da-3
anchor: history — docs/history/decision-attribution/CONTEXT.md, docs/history/decision-attribution/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/decision-attribution/delivery.md

---
type: bee.delivery
title: decision-attribution — delivery
description: "Delivery record proposed by bee knowledge promote for work item decision-attribution: 3 capped cell(s), 6 recorded deviation(s)."
timestamp: 2026-08-25
bee:
  id: decision-attribution-delivery
  lifecycle: active
  areas: [decision-memory]
  required_context: [docs/history/decision-attribution/CONTEXT.md, docs/history/decision-attribution/plan.md]
  sources: [docs/history/decision-attribution/CONTEXT.md, docs/history/decision-attribution/plan.md, .bee/cells/da-1.json, .bee/cells/da-2.json, .bee/cells/da-3.json]
---

# decision-attribution — Delivery

## What shipped

- **da-1** — decisions log stamps a feature only from a lane-resolved target; the shared default record is never borrowed (2 file(s) changed)
- **da-2** — bee decisions log gains --feature; explicit value outranks the bound lane, a blank one refuses (4 file(s) changed)
- **da-3** — bee decisions reattribute lands and corrects 67 contradicted feature stamps on the live store; a second run reports 0 (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **da-1** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee decisions`
- **da-2** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
- **da-3** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **da-1** — Executed inline instead of via a dispatched execution worker; reason recorded as trace.inline_reason.
- **da-2** — Executed inline instead of via a dispatched execution worker; reason recorded as trace.inline_reason.
- **da-2** — Sync door acknowledged: the workflow-state file touched is a mechanical struct-field update with no gate behaviour change.
- **da-2** — sync-ack: The workflow-state touch is state_group/set_gate.rs, and it is purely mechanical: three LogParams literals gained feature: None because the struct grew a field. No gate behaviour, wording, or contract changed, so none of the area's owned skills has anything to say about it. The behavioural change in this cell is confined to decisions log, whose surface is documented in the command registry that ships with it.
- **da-3** — Executed inline instead of via a dispatched execution worker; reason recorded as trace.inline_reason.
- **da-3** — Corrected 67 records rather than the 23 named in D5 — same predicate and same mechanism, a wider blast radius than the first sample showed.

## Provenance

Proposed by `bee knowledge promote --work decision-attribution` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/decision-attribution/CONTEXT.md`, `docs/history/decision-attribution/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "decision-attribution" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-25T14:37:32.535Z), the work item declares no bee.areas.

area decision-memory:
  - [da-1] decisions log stamps a feature only from a lane-resolved target; the shared default record is never borrowed — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/da-1.json)
  - [da-2] bee decisions log gains --feature; explicit value outranks the bound lane, a blank one refuses — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/da-2.json)
  - [da-3] bee decisions reattribute lands and corrects 67 contradicted feature stamps on the live store; a second run reports 0 — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/da-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell da-1 — save as docs/knowledge/patterns/decision-attribution-da-1-pitfall.md

---
type: bee.pattern
title: decision-attribution cell da-1 — pitfall candidate
description: "Pitfall candidate mined from cell da-1's capped trace: Executed inline instead of via a dispatched execution worker; reason recorded as trace.inline_reason."
timestamp: 2026-08-25
bee:
  id: decision-attribution-da-1-pitfall
  lifecycle: draft
  areas: [decision-memory]
  sources: [.bee/cells/da-1.json]
  polarity: pitfall
---

# decision-attribution cell da-1 — pitfall candidate

## What the cell did

decisions log stamps a feature only from a lane-resolved target; the shared default record is never borrowed

## Recorded evidence (verbatim from .bee/cells/da-1.json)

- **deviation** — Executed inline instead of via a dispatched execution worker; reason recorded as trace.inline_reason.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell da-2 — save as docs/knowledge/patterns/decision-attribution-da-2-pitfall.md

---
type: bee.pattern
title: decision-attribution cell da-2 — pitfall candidate
description: "Pitfall candidate mined from cell da-2's capped trace: Executed inline instead of via a dispatched execution worker; reason recorded as trace.inline_reason."
timestamp: 2026-08-25
bee:
  id: decision-attribution-da-2-pitfall
  lifecycle: draft
  areas: [decision-memory]
  sources: [.bee/cells/da-2.json]
  polarity: pitfall
---

# decision-attribution cell da-2 — pitfall candidate

## What the cell did

bee decisions log gains --feature; explicit value outranks the bound lane, a blank one refuses

## Recorded evidence (verbatim from .bee/cells/da-2.json)

- **deviation** — Executed inline instead of via a dispatched execution worker; reason recorded as trace.inline_reason.
- **deviation** — Sync door acknowledged: the workflow-state file touched is a mechanical struct-field update with no gate behaviour change.
- **deviation** — sync-ack: The workflow-state touch is state_group/set_gate.rs, and it is purely mechanical: three LogParams literals gained feature: None because the struct grew a field. No gate behaviour, wording, or contract changed, so none of the area's owned skills has anything to say about it. The behavioural change in this cell is confined to decisions log, whose surface is documented in the command registry that ships with it.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell da-3 — save as docs/knowledge/patterns/decision-attribution-da-3-pitfall.md

---
type: bee.pattern
title: decision-attribution cell da-3 — pitfall candidate
description: "Pitfall candidate mined from cell da-3's capped trace: Executed inline instead of via a dispatched execution worker; reason recorded as trace.inline_reason."
timestamp: 2026-08-25
bee:
  id: decision-attribution-da-3-pitfall
  lifecycle: draft
  areas: [decision-memory]
  sources: [.bee/cells/da-3.json]
  polarity: pitfall
---

# decision-attribution cell da-3 — pitfall candidate

## What the cell did

bee decisions reattribute lands and corrects 67 contradicted feature stamps on the live store; a second run reports 0

## Recorded evidence (verbatim from .bee/cells/da-3.json)

- **deviation** — Executed inline instead of via a dispatched execution worker; reason recorded as trace.inline_reason.
- **deviation** — Corrected 67 records rather than the 23 named in D5 — same predicate and same mechanism, a wider blast radius than the first sample showed.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 3 area bullet(s), 3 pattern candidate(s), 0 file(s) written.