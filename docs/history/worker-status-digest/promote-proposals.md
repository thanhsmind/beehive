promote proposal for work item "worker-status-digest" (docs/history/worker-status-digest/CONTEXT.md + docs/history/worker-status-digest/plan.md) — 2 capped cell(s): wsdg-1, wsdg-2
anchor: history — docs/history/worker-status-digest/CONTEXT.md, docs/history/worker-status-digest/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/worker-status-digest/delivery.md

---
type: bee.delivery
title: worker-status-digest — delivery
description: "Delivery record proposed by bee knowledge promote for work item worker-status-digest: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-11-05
bee:
  id: worker-status-digest-delivery
  lifecycle: active
  required_context: [docs/history/worker-status-digest/CONTEXT.md, docs/history/worker-status-digest/plan.md]
  sources: [docs/history/worker-status-digest/CONTEXT.md, docs/history/worker-status-digest/plan.md, .bee/cells/wsdg-1.json, .bee/cells/wsdg-2.json]
---

# worker-status-digest — Delivery

## What shipped

- **wsdg-1** — Add one-line worker status digest mode and progress.jsonl routing cd0950262 (5 file(s) changed)
- **wsdg-2** — Teach workers to write progress notes and leaders to read only the status line (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wsdg-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml herding` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml herding — green:unit — 871 passed covering status_digest, herding status --…
- **wsdg-2** — `rg -n 'herding status --job' packages/bee/prompts/worker-cell.md skills/bee-swarming/SKILL.md packages/bee/AGENTS.block.md docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md && rg -n 'notify --kind progress' packages/bee/prompts/worker-cell.md` — hits in SKILL.md, AGENTS.block.md and the concept, one notify hit in worker-cell.md; docs-only cell

## Deviations

- **wsdg-1** — sync-ack: skill and doc text for the status digest ships in sibling cell wsdg-2
- **wsdg-2** — proof line names the two rg commands without the verify string's own expected: clauses, because those clauses carry the proof separator

## Provenance

Proposed by `bee knowledge promote --work worker-status-digest` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/worker-status-digest/CONTEXT.md`, `docs/history/worker-status-digest/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wsdg-1 — save as docs/knowledge/patterns/worker-status-digest-wsdg-1-pitfall.md

---
type: bee.pattern
title: worker-status-digest cell wsdg-1 — pitfall candidate
description: "Pitfall candidate mined from cell wsdg-1's capped trace: sync-ack: skill and doc text for the status digest ships in sibling cell wsdg-2"
timestamp: 2026-11-05
bee:
  id: worker-status-digest-wsdg-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/wsdg-1.json]
  polarity: pitfall
---

# worker-status-digest cell wsdg-1 — pitfall candidate

## What the cell did

Add one-line worker status digest mode and progress.jsonl routing cd0950262

## Recorded evidence (verbatim from .bee/cells/wsdg-1.json)

- **deviation** — sync-ack: skill and doc text for the status digest ships in sibling cell wsdg-2

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell wsdg-2 — save as docs/knowledge/patterns/worker-status-digest-wsdg-2-pitfall.md

---
type: bee.pattern
title: worker-status-digest cell wsdg-2 — pitfall candidate
description: "Pitfall candidate mined from cell wsdg-2's capped trace: proof line names the two rg commands without the verify string's own expected: clauses, because those clauses carry the proof separator"
timestamp: 2026-11-05
bee:
  id: worker-status-digest-wsdg-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/wsdg-2.json]
  polarity: pitfall
---

# worker-status-digest cell wsdg-2 — pitfall candidate

## What the cell did

Teach workers to write progress notes and leaders to read only the status line

## Recorded evidence (verbatim from .bee/cells/wsdg-2.json)

- **deviation** — proof line names the two rg commands without the verify string's own expected: clauses, because those clauses carry the proof separator
- **failure_signature** — f56c05fd7e35

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.