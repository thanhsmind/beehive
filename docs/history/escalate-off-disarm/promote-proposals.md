promote proposal for work item "escalate-off-disarm" (docs/history/escalate-off-disarm/CONTEXT.md) — 1 capped cell(s): eod-1
anchor: history — docs/history/escalate-off-disarm/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/escalate-off-disarm/delivery.md

---
type: bee.delivery
title: escalate-off-disarm — delivery
description: "Delivery record proposed by bee knowledge promote for work item escalate-off-disarm: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-08-25
bee:
  id: escalate-off-disarm-delivery
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [docs/history/escalate-off-disarm/CONTEXT.md]
  sources: [docs/history/escalate-off-disarm/CONTEXT.md, .bee/cells/eod-1.json]
---

# escalate-off-disarm — Delivery

## What shipped

- **eod-1** — escalate disarm now actually disarms a legacy ceiling cell; an explicit escalate:false outranks the legacy tier read everywhere and survives later backfill passes (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **eod-1** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **eod-1** — Executed inline instead of via a dispatched execution worker; reason recorded as trace.inline_reason.
- **eod-1** — sync-ack: The change is a predicate and storage-shape fix inside the escalation machinery; no skill teaches the disarm's on-disk spelling, and the operator-facing contract (bee cells escalate --off disarms) is exactly what the skills already describe — the fix makes the code match the taught behavior, not the reverse.

## Provenance

Proposed by `bee knowledge promote --work escalate-off-disarm` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/escalate-off-disarm/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "escalate-off-disarm" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-25T21:34:09.491Z), the work item declares no bee.areas.

area doctrine-layer:
  - [eod-1] escalate disarm now actually disarms a legacy ceiling cell; an explicit escalate:false outranks the legacy tier read everywhere and survives later backfill passes — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/eod-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell eod-1 — save as docs/knowledge/patterns/escalate-off-disarm-eod-1-pitfall.md

---
type: bee.pattern
title: escalate-off-disarm cell eod-1 — pitfall candidate
description: "Pitfall candidate mined from cell eod-1's capped trace: Executed inline instead of via a dispatched execution worker; reason recorded as trace.inline_reason."
timestamp: 2026-08-25
bee:
  id: escalate-off-disarm-eod-1-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/eod-1.json]
  polarity: pitfall
---

# escalate-off-disarm cell eod-1 — pitfall candidate

## What the cell did

escalate disarm now actually disarms a legacy ceiling cell; an explicit escalate:false outranks the legacy tier read everywhere and survives later backfill passes

## Recorded evidence (verbatim from .bee/cells/eod-1.json)

- **deviation** — Executed inline instead of via a dispatched execution worker; reason recorded as trace.inline_reason.
- **deviation** — sync-ack: The change is a predicate and storage-shape fix inside the escalation machinery; no skill teaches the disarm's on-disk spelling, and the operator-facing contract (bee cells escalate --off disarms) is exactly what the skills already describe — the fix makes the code match the taught behavior, not the reverse.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.