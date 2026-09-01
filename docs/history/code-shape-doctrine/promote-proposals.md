promote proposal for work item "code-shape-doctrine" (docs/history/code-shape-doctrine/CONTEXT.md) — 1 capped cell(s): csdoc-1
anchor: history — docs/history/code-shape-doctrine/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/code-shape-doctrine/delivery.md

---
type: bee.delivery
title: code-shape-doctrine — delivery
description: "Delivery record proposed by bee knowledge promote for work item code-shape-doctrine: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-01
bee:
  id: code-shape-doctrine-delivery
  lifecycle: active
  required_context: [docs/history/code-shape-doctrine/CONTEXT.md]
  sources: [docs/history/code-shape-doctrine/CONTEXT.md, .bee/cells/csdoc-1.json]
---

# code-shape-doctrine — Delivery

## What shipped

- **csdoc-1** — Added the four code-shape rules as one contract bullet in the worker brief source and the bee-build agent template, stated as judgment-and-review with no refusal (7 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **csdoc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee onboard --repo-root . --json && .bee/bin/bee dev release-manifest --check`

## Deviations

- **csdoc-1** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work code-shape-doctrine` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/code-shape-doctrine/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell csdoc-1 — save as docs/knowledge/patterns/code-shape-doctrine-csdoc-1-pitfall.md

---
type: bee.pattern
title: code-shape-doctrine cell csdoc-1 — pitfall candidate
description: "Pitfall candidate mined from cell csdoc-1's capped trace: followed the plan"
timestamp: 2026-09-01
bee:
  id: code-shape-doctrine-csdoc-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/csdoc-1.json]
  polarity: pitfall
---

# code-shape-doctrine cell csdoc-1 — pitfall candidate

## What the cell did

Added the four code-shape rules as one contract bullet in the worker brief source and the bee-build agent template, stated as judgment-and-review with no refusal

## Recorded evidence (verbatim from .bee/cells/csdoc-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.