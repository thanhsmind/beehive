promote proposal for work item "wave-run" (docs/history/wave-run/CONTEXT.md) — 1 capped cell(s): wvr-1
anchor: history — docs/history/wave-run/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/wave-run/delivery.md

---
type: bee.delivery
title: wave-run — delivery
description: "Delivery record proposed by bee knowledge promote for work item wave-run: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-11-03
bee:
  id: wave-run-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/wave-run/CONTEXT.md]
  sources: [docs/history/wave-run/CONTEXT.md, .bee/cells/wvr-1.json]
---

# wave-run — Delivery

## What shipped

- **wvr-1** — bee dispatch wave gains --run: the herding half of a prepared wave now executes in parallel in one call and returns {ran, native, skipped}, while the prepare-only path stays byte-identical (commit 64fb311a4). (7 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wvr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee dispatch_wave && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts` — 9 dispatch_wave tests incl. 2 new dispatch_wave_run tests and 12 registry_contracts, rerun by the leader

## Deviations

- **wvr-1** — touched reservations/flags.rs (FLAG_ALONE_BOOLEANS gets run) and catalog.rs (PINNED_FLAG_COUNT 234 to 235), both required by any new boolean flag
- **wvr-1** — sync-ack: the bee-swarming line that teaches dispatch wave --run lands with the knowledge sync of this feature, after the code merge

## Provenance

Proposed by `bee knowledge promote --work wave-run` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/wave-run/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "wave-run" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-03T03:14:46.146Z), the work item declares no bee.areas.

area workflow-state:
  - [wvr-1] bee dispatch wave gains --run: the herding half of a prepared wave now executes in parallel in one call and returns {ran, native, skipped}, while the prepare-only path stays byte-identical (commit 64fb311a4). — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/wvr-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wvr-1 — save as docs/knowledge/patterns/wave-run-wvr-1-pitfall.md

---
type: bee.pattern
title: wave-run cell wvr-1 — pitfall candidate
description: "Pitfall candidate mined from cell wvr-1's capped trace: touched reservations/flags.rs (FLAG_ALONE_BOOLEANS gets run) and catalog.rs (PINNED_FLAG_COUNT 234 to 235), both required by any new boolean flag"
timestamp: 2026-11-03
bee:
  id: wave-run-wvr-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/wvr-1.json]
  polarity: pitfall
---

# wave-run cell wvr-1 — pitfall candidate

## What the cell did

bee dispatch wave gains --run: the herding half of a prepared wave now executes in parallel in one call and returns {ran, native, skipped}, while the prepare-only path stays byte-identical (commit 64fb311a4).

## Recorded evidence (verbatim from .bee/cells/wvr-1.json)

- **deviation** — touched reservations/flags.rs (FLAG_ALONE_BOOLEANS gets run) and catalog.rs (PINNED_FLAG_COUNT 234 to 235), both required by any new boolean flag
- **deviation** — sync-ack: the bee-swarming line that teaches dispatch wave --run lands with the knowledge sync of this feature, after the code merge

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.