promote proposal for work item "stage-gate-phase-parity" (docs/history/stage-gate-phase-parity/plan.md) — 1 capped cell(s): sgpp-1
anchor: history — docs/history/stage-gate-phase-parity/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/stage-gate-phase-parity/delivery.md

---
type: bee.delivery
title: stage-gate-phase-parity — delivery
description: "Delivery record proposed by bee knowledge promote for work item stage-gate-phase-parity: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-09-19
bee:
  id: stage-gate-phase-parity-delivery
  lifecycle: active
  required_context: [docs/history/stage-gate-phase-parity/plan.md]
  sources: [docs/history/stage-gate-phase-parity/plan.md, .bee/cells/sgpp-1.json]
---

# stage-gate-phase-parity — Delivery

## What shipped

- **sgpp-1** — Pi stage tool gate follows the write guard's phase groups; grooming keeps edit/write, unrecognized phases narrow (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sgpp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --bins hooks::stage_tools` — 8 passed / 0 failed, red first; the whole hooks module also green at 626/626, and the rebuilt binary re-driven live against sandbox stores one run per phase

## Deviations

- **sgpp-1** — Ran the tiny cell inline in the main checkout rather than a feature worktree: solo session (bee status: Active workers: 1), one product file, which is the recorded tiny-fix exemption in AGENTS.md.
- **sgpp-1** — Kept the tiny lane past the JUDGE_OBLIGATION guard on hooks/ by recording judge_obligation_ack, but did NOT skip the independent read it protects: dispatched a bee-review worker, which found two P2 findings; both are fixed in this cell and the plan was revised to rev 1.
- **sgpp-1** — The approved cell verify named `--lib`, which this crate has no target for (`error: no library targets found in package bee`). Corrected the cell's verify to `--bins` via cells update before capping, rather than capping against a command that cannot run.

## Provenance

Proposed by `bee knowledge promote --work stage-gate-phase-parity` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/stage-gate-phase-parity/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell sgpp-1 — save as docs/knowledge/patterns/stage-gate-phase-parity-sgpp-1-pitfall.md

---
type: bee.pattern
title: stage-gate-phase-parity cell sgpp-1 — pitfall candidate
description: "Pitfall candidate mined from cell sgpp-1's capped trace: Ran the tiny cell inline in the main checkout rather than a feature worktree: solo session (bee status: Active workers: 1), one product file, which is the reco…"
timestamp: 2026-09-19
bee:
  id: stage-gate-phase-parity-sgpp-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/sgpp-1.json]
  polarity: pitfall
---

# stage-gate-phase-parity cell sgpp-1 — pitfall candidate

## What the cell did

Pi stage tool gate follows the write guard's phase groups; grooming keeps edit/write, unrecognized phases narrow

## Recorded evidence (verbatim from .bee/cells/sgpp-1.json)

- **deviation** — Ran the tiny cell inline in the main checkout rather than a feature worktree: solo session (bee status: Active workers: 1), one product file, which is the recorded tiny-fix exemption in AGENTS.md.
- **deviation** — Kept the tiny lane past the JUDGE_OBLIGATION guard on hooks/ by recording judge_obligation_ack, but did NOT skip the independent read it protects: dispatched a bee-review worker, which found two P2 findings; both are fixed in this cell and the plan was revised to rev 1.
- **deviation** — The approved cell verify named `--lib`, which this crate has no target for (`error: no library targets found in package bee`). Corrected the cell's verify to `--bins` via cells update before capping, rather than capping against a command that cannot run.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.