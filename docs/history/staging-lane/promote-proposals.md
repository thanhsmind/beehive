promote proposal for work item "staging-lane" (docs/history/staging-lane/plan.md) — 4 capped cell(s): sl-1, sl-2, sl-3, sl-4
anchor: history — docs/history/staging-lane/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/staging-lane/delivery.md

---
type: bee.delivery
title: staging-lane — delivery
description: "Delivery record proposed by bee knowledge promote for work item staging-lane: 4 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-17
bee:
  id: staging-lane-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [docs/history/staging-lane/plan.md]
  sources: [docs/history/staging-lane/plan.md, .bee/cells/sl-1.json, .bee/cells/sl-2.json, .bee/cells/sl-3.json, .bee/cells/sl-4.json]
---

# staging-lane — Delivery

## What shipped

- **sl-1** — staging add: lazy create from main, merge feature, staged-set store, build hook (3 file(s) changed)
- **sl-2** — staging rebuild and status: invariant re-derivation (3 file(s) changed)
- **sl-3** — Add staging-lane teeth: merge refusal, commit guard, rebuild nudge (6 file(s) changed)
- **sl-4** — Docs and skill guidance: the topology is taught, not only enforced (10 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sl-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **sl-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **sl-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **sl-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **sl-2** — catalog.rs (not in cell.files) needed a PINNED_FLAG_COUNT bump for the new --without flag to keep the pinned flag-vocabulary test green — a mechanical, test-forced follow-on of the declared change.

## Provenance

Proposed by `bee knowledge promote --work staging-lane` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/staging-lane/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "staging-lane" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-17T11:54:18.136Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [sl-1] staging add: lazy create from main, merge feature, staged-set store, build hook — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sl-1.json)
  - [sl-2] staging rebuild and status: invariant re-derivation — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sl-2.json)
  - [sl-3] Add staging-lane teeth: merge refusal, commit guard, rebuild nudge — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/sl-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell sl-2 — save as docs/knowledge/patterns/staging-lane-sl-2-pitfall.md

---
type: bee.pattern
title: staging-lane cell sl-2 — pitfall candidate
description: "Pitfall candidate mined from cell sl-2's capped trace: catalog.rs (not in cell.files) needed a PINNED_FLAG_COUNT bump for the new --without flag to keep the pinned flag-vocabulary test green — a mechanical, test-fo…"
timestamp: 2026-08-17
bee:
  id: staging-lane-sl-2-pitfall
  lifecycle: draft
  areas: [worktree-parallelism]
  sources: [.bee/cells/sl-2.json]
  polarity: pitfall
---

# staging-lane cell sl-2 — pitfall candidate

## What the cell did

staging rebuild and status: invariant re-derivation

## Recorded evidence (verbatim from .bee/cells/sl-2.json)

- **deviation** — catalog.rs (not in cell.files) needed a PINNED_FLAG_COUNT bump for the new --without flag to keep the pinned flag-vocabulary test green — a mechanical, test-forced follow-on of the declared change.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 3 area bullet(s), 1 pattern candidate(s), 0 file(s) written.