promote proposal for work item "write-guard-precision" (docs/history/write-guard-precision/plan.md) — 2 capped cell(s): wgp-1, wgp-2
anchor: history — docs/history/write-guard-precision/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/write-guard-precision/delivery.md

---
type: bee.delivery
title: write-guard-precision — delivery
description: "Delivery record proposed by bee knowledge promote for work item write-guard-precision: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: write-guard-precision-delivery
  lifecycle: active
  required_context: [docs/history/write-guard-precision/plan.md]
  sources: [docs/history/write-guard-precision/plan.md, .bee/cells/wgp-1.json, .bee/cells/wgp-2.json]
---

# write-guard-precision — Delivery

## What shipped

- **wgp-1** — guards.rs code-extension exemptions landed in worktree commit 23ad32e; worker suite green there (1529 passed, 7 ignored) (2 file(s) changed)
- **wgp-2** — idle-gate safe-form git table + gc-2 Unresolved remedy landed in worktree commit fbcc40d7; worker suite green (1422 passed unit binary, full run ok) (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wgp-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wgp-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **wgp-1** — reservations skipped: control-plane verbs refused inside granted worktree; single sequential worker, no sibling risk
- **wgp-2** — reservations skipped: control-plane verbs refused inside granted worktree; single sequential worker, no sibling risk

## Provenance

Proposed by `bee knowledge promote --work write-guard-precision` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/write-guard-precision/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wgp-1 — save as docs/knowledge/patterns/write-guard-precision-wgp-1-pitfall.md

---
type: bee.pattern
title: write-guard-precision cell wgp-1 — pitfall candidate
description: "Pitfall candidate mined from cell wgp-1's capped trace: reservations skipped: control-plane verbs refused inside granted worktree; single sequential worker, no sibling risk"
timestamp: 2026-08-11
bee:
  id: write-guard-precision-wgp-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/wgp-1.json]
  polarity: pitfall
---

# write-guard-precision cell wgp-1 — pitfall candidate

## What the cell did

guards.rs code-extension exemptions landed in worktree commit 23ad32e; worker suite green there (1529 passed, 7 ignored)

## Recorded evidence (verbatim from .bee/cells/wgp-1.json)

- **deviation** — reservations skipped: control-plane verbs refused inside granted worktree; single sequential worker, no sibling risk

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell wgp-2 — save as docs/knowledge/patterns/write-guard-precision-wgp-2-pitfall.md

---
type: bee.pattern
title: write-guard-precision cell wgp-2 — pitfall candidate
description: "Pitfall candidate mined from cell wgp-2's capped trace: reservations skipped: control-plane verbs refused inside granted worktree; single sequential worker, no sibling risk"
timestamp: 2026-08-11
bee:
  id: write-guard-precision-wgp-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/wgp-2.json]
  polarity: pitfall
---

# write-guard-precision cell wgp-2 — pitfall candidate

## What the cell did

idle-gate safe-form git table + gc-2 Unresolved remedy landed in worktree commit fbcc40d7; worker suite green (1422 passed unit binary, full run ok)

## Recorded evidence (verbatim from .bee/cells/wgp-2.json)

- **deviation** — reservations skipped: control-plane verbs refused inside granted worktree; single sequential worker, no sibling risk

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.