promote proposal for work item "test-doctrine-text-sweep" (.bee/logs/scribing-runs.jsonl + .bee/lanes/test-doctrine-text-sweep.json) — 3 capped cell(s): tdt-1, tdt-2, tdt-3
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/test-doctrine-text-sweep.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/test-doctrine-text-sweep/delivery.md

---
type: bee.delivery
title: test-doctrine-text-sweep — delivery
description: "Delivery record proposed by bee knowledge promote for work item test-doctrine-text-sweep: 3 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-18
bee:
  id: test-doctrine-text-sweep-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/test-doctrine-text-sweep.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/test-doctrine-text-sweep.json, .bee/cells/tdt-1.json, .bee/cells/tdt-2.json, .bee/cells/tdt-3.json]
---

# test-doctrine-text-sweep — Delivery

## What shipped

- **tdt-1** — Rewrote both stale boundary-run text/comment assertions; full suite green (2 file(s) changed)
- **tdt-2** — Registry payload: three command descriptions still promise a boundary test run (2 file(s) changed)
- **tdt-3** — Swept the retired boundary-run doctrine out of the handbook, then out of the three claims the sweep itself got wrong (8 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **tdt-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **tdt-2** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **tdt-3** — `rg -U -n 'prove\s+at\s+the\s+boundary|runs?\s+(the\s+declared\s+tests|commands\.test)|records\s+\{?tests:?\s*.?boundary|runner\s+is\s+.test.,\s*.finish.|read\s+by\s+.cells\s+finish.|against\s+the\s+staged\s+merge' docs/handbook docs/codebase-overview.md .bee/config-sample.json`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work test-doctrine-text-sweep` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/test-doctrine-text-sweep.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "test-doctrine-text-sweep" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-18T13:01:03.665Z), the work item declares no bee.areas.

area workflow-state:
  - [tdt-1] Rewrote both stale boundary-run text/comment assertions; full suite green — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/tdt-1.json)
  - [tdt-2] Registry payload: three command descriptions still promise a boundary test run — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/tdt-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell tdt-3 — save as docs/knowledge/patterns/test-doctrine-text-sweep-tdt-3-pitfall.md

---
type: bee.pattern
title: test-doctrine-text-sweep cell tdt-3 — pitfall candidate
description: "Pitfall candidate mined from cell tdt-3's capped trace: handbook-sweep-under-reach: overview.md and stages/swarming.md keep the retired boundary-run doctrine, the verify grep cannot see them because the phrase wraps…"
timestamp: 2026-08-18
bee:
  id: test-doctrine-text-sweep-tdt-3-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/tdt-3.json]
  polarity: pitfall
---

# test-doctrine-text-sweep cell tdt-3 — pitfall candidate

## What the cell did

Swept the retired boundary-run doctrine out of the handbook, then out of the three claims the sweep itself got wrong

## Recorded evidence (verbatim from .bee/cells/tdt-3.json)

- **failure_signature** — handbook-sweep-under-reach: overview.md and stages/swarming.md keep the retired boundary-run doctrine, the verify grep cannot see them because the phrase wraps a newline, and three rewritten passages carry boundary-era tails
- **failure_signature** — revision-still-incomplete: the register's logs section keeps finish and close as declared-command runners, and the cap still records the retired false-clean single-line grep instead of the proven multiline sweep

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 1 pattern candidate(s), 0 file(s) written.