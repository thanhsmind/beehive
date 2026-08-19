promote proposal for work item "uat-gate-before-merge" (docs/history/uat-gate-before-merge/plan.md) — 3 capped cell(s): ug-1, ug-2, ug-3
anchor: history — docs/history/uat-gate-before-merge/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/uat-gate-before-merge/delivery.md

---
type: bee.delivery
title: uat-gate-before-merge — delivery
description: "Delivery record proposed by bee knowledge promote for work item uat-gate-before-merge: 3 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-18
bee:
  id: uat-gate-before-merge-delivery
  lifecycle: active
  areas: [workflow-state, worktree-parallelism]
  required_context: [docs/history/uat-gate-before-merge/plan.md]
  sources: [docs/history/uat-gate-before-merge/plan.md, .bee/cells/ug-1.json, .bee/cells/ug-2.json, .bee/cells/ug-3.json]
---

# uat-gate-before-merge — Delivery

## What shipped

- **ug-1** — Add the uat gate to state, never auto-approvable (11 file(s) changed)
- **ug-2** — worktree merge refuses standard/high-risk features without uat approval (8 file(s) changed)
- **ug-3** — Docs + flow surfaces taught about the uat merge-acceptance stop (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ug-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ug-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ug-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work uat-gate-before-merge` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/uat-gate-before-merge/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "uat-gate-before-merge" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-17T10:28:25.242Z), the work item declares no bee.areas.

area workflow-state:
  - [ug-1] Add the uat gate to state, never auto-approvable — feature-wide sync per the scribing stamp, 11 file(s) changed (trace .bee/cells/ug-1.json)
  - [ug-2] worktree merge refuses standard/high-risk features without uat approval — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/ug-2.json)

area worktree-parallelism:
  - [ug-1] Add the uat gate to state, never auto-approvable — feature-wide sync per the scribing stamp, 11 file(s) changed (trace .bee/cells/ug-1.json)
  - [ug-2] worktree merge refuses standard/high-risk features without uat approval — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/ug-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 0 pattern candidate(s), 0 file(s) written.