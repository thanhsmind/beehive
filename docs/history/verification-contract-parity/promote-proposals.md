promote proposal for work item "verification-contract-parity" (docs/history/verification-contract-parity/CONTEXT.md) — 1 capped cell(s): vcp-2
anchor: history — docs/history/verification-contract-parity/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/verification-contract-parity/delivery.md

---
type: bee.delivery
title: verification-contract-parity — delivery
description: "Delivery record proposed by bee knowledge promote for work item verification-contract-parity: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-02
bee:
  id: verification-contract-parity-delivery
  lifecycle: active
  required_context: [docs/history/verification-contract-parity/CONTEXT.md]
  sources: [docs/history/verification-contract-parity/CONTEXT.md, .bee/cells/vcp-2.json]
---

# verification-contract-parity — Delivery

## What shipped

- **vcp-2** — New verification_contract_parity fence: verify-app name parity across derived surfaces, plus the green:live cap-proof case in agents-proof-at-cap (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **vcp-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test verification_contract_parity --test agents_block_render_parity --test rule_index_parity`

## Deviations

- **vcp-2** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work verification-contract-parity` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/verification-contract-parity/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell vcp-2 — save as docs/knowledge/patterns/verification-contract-parity-vcp-2-pitfall.md

---
type: bee.pattern
title: verification-contract-parity cell vcp-2 — pitfall candidate
description: "Pitfall candidate mined from cell vcp-2's capped trace: followed the plan"
timestamp: 2026-09-02
bee:
  id: verification-contract-parity-vcp-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/vcp-2.json]
  polarity: pitfall
---

# verification-contract-parity cell vcp-2 — pitfall candidate

## What the cell did

New verification_contract_parity fence: verify-app name parity across derived surfaces, plus the green:live cap-proof case in agents-proof-at-cap

## Recorded evidence (verbatim from .bee/cells/vcp-2.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.