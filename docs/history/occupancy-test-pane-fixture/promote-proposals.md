promote proposal for work item "occupancy-test-pane-fixture" (docs/history/occupancy-test-pane-fixture/plan.md) — 1 capped cell(s): otpf-1
anchor: history — docs/history/occupancy-test-pane-fixture/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/occupancy-test-pane-fixture/delivery.md

---
type: bee.delivery
title: occupancy-test-pane-fixture — delivery
description: "Delivery record proposed by bee knowledge promote for work item occupancy-test-pane-fixture: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-06
bee:
  id: occupancy-test-pane-fixture-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/occupancy-test-pane-fixture/plan.md]
  sources: [docs/history/occupancy-test-pane-fixture/plan.md, .bee/cells/otpf-1.json]
---

# occupancy-test-pane-fixture — Delivery

## What shipped

- **otpf-1** — paseo occupancy test passes an explicit empty pane set (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **otpf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee herding::wave::tests::occupancy` — 10 occupancy tests; red reproduced first by running the test binary with PATH=/nonexistent (Fallback(1) vs Live(1)), green after with and without pane tools; full suite left to CI

## Deviations

- **otpf-1** — sync-ack: test-only change; no owned skill describes this test fixture

## Provenance

Proposed by `bee knowledge promote --work occupancy-test-pane-fixture` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/occupancy-test-pane-fixture/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "occupancy-test-pane-fixture" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-06T17:02:38.581Z), the work item declares no bee.areas.

area bee-herding:
  (no capped behavior_change cell exists for this feature)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell otpf-1 — save as docs/knowledge/patterns/occupancy-test-pane-fixture-otpf-1-pitfall.md

---
type: bee.pattern
title: occupancy-test-pane-fixture cell otpf-1 — pitfall candidate
description: "Pitfall candidate mined from cell otpf-1's capped trace: sync-ack: test-only change; no owned skill describes this test fixture"
timestamp: 2026-10-06
bee:
  id: occupancy-test-pane-fixture-otpf-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/otpf-1.json]
  polarity: pitfall
---

# occupancy-test-pane-fixture cell otpf-1 — pitfall candidate

## What the cell did

paseo occupancy test passes an explicit empty pane set

## Recorded evidence (verbatim from .bee/cells/otpf-1.json)

- **deviation** — sync-ack: test-only change; no owned skill describes this test fixture

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.