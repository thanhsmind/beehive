promote proposal for work item "role-surface-cleanup" (docs/history/role-surface-cleanup/CONTEXT.md) — 2 capped cell(s): rsc-1, rsc-2
anchor: history — docs/history/role-surface-cleanup/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/role-surface-cleanup/delivery.md

---
type: bee.delivery
title: role-surface-cleanup — delivery
description: "Delivery record proposed by bee knowledge promote for work item role-surface-cleanup: 2 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-08-25
bee:
  id: role-surface-cleanup-delivery
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [docs/history/role-surface-cleanup/CONTEXT.md]
  sources: [docs/history/role-surface-cleanup/CONTEXT.md, .bee/cells/rsc-1.json, .bee/cells/rsc-2.json]
---

# role-surface-cleanup — Delivery

## What shipped

- **rsc-1** — the fall-through warn fires once per dispatch, the registry speaks role language, the worker refusal says role (4 file(s) changed)
- **rsc-2** — both CI jobs and the declared commands.test run with no-fail-fast; a red target can no longer hide the targets behind it (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rsc-1** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
- **rsc-2** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **rsc-1** — Executed inline; reason on trace.inline_reason.
- **rsc-1** — sync-ack: workers.rs is a wording-only change inside a refusal string; the flag, key, and recorded rationale are untouched, so no owned skill has anything to update.
- **rsc-2** — Executed inline; reason on trace.inline_reason.
- **rsc-2** — files declares .bee/config.json though it is state, because the proof-gate parity makes it part of this change contract

## Provenance

Proposed by `bee knowledge promote --work role-surface-cleanup` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/role-surface-cleanup/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "role-surface-cleanup" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-25T21:34:09.510Z), the work item declares no bee.areas.

area doctrine-layer:
  - [rsc-1] the fall-through warn fires once per dispatch, the registry speaks role language, the worker refusal says role — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rsc-1.json)
  - [rsc-2] both CI jobs and the declared commands.test run with no-fail-fast; a red target can no longer hide the targets behind it — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/rsc-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rsc-1 — save as docs/knowledge/patterns/role-surface-cleanup-rsc-1-pitfall.md

---
type: bee.pattern
title: role-surface-cleanup cell rsc-1 — pitfall candidate
description: "Pitfall candidate mined from cell rsc-1's capped trace: Executed inline; reason on trace.inline_reason."
timestamp: 2026-08-25
bee:
  id: role-surface-cleanup-rsc-1-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/rsc-1.json]
  polarity: pitfall
---

# role-surface-cleanup cell rsc-1 — pitfall candidate

## What the cell did

the fall-through warn fires once per dispatch, the registry speaks role language, the worker refusal says role

## Recorded evidence (verbatim from .bee/cells/rsc-1.json)

- **deviation** — Executed inline; reason on trace.inline_reason.
- **deviation** — sync-ack: workers.rs is a wording-only change inside a refusal string; the flag, key, and recorded rationale are untouched, so no owned skill has anything to update.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell rsc-2 — save as docs/knowledge/patterns/role-surface-cleanup-rsc-2-pitfall.md

---
type: bee.pattern
title: role-surface-cleanup cell rsc-2 — pitfall candidate
description: "Pitfall candidate mined from cell rsc-2's capped trace: Executed inline; reason on trace.inline_reason."
timestamp: 2026-08-25
bee:
  id: role-surface-cleanup-rsc-2-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/rsc-2.json]
  polarity: pitfall
---

# role-surface-cleanup cell rsc-2 — pitfall candidate

## What the cell did

both CI jobs and the declared commands.test run with no-fail-fast; a red target can no longer hide the targets behind it

## Recorded evidence (verbatim from .bee/cells/rsc-2.json)

- **deviation** — Executed inline; reason on trace.inline_reason.
- **deviation** — files declares .bee/config.json though it is state, because the proof-gate parity makes it part of this change contract

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 2 pattern candidate(s), 0 file(s) written.