promote proposal for work item "pi-full-workflow-parity" (docs/history/pi-full-workflow-parity/CONTEXT.md + docs/history/pi-full-workflow-parity/plan.md) — 2 capped cell(s): pfp-1, pfp-2
anchor: history — docs/history/pi-full-workflow-parity/CONTEXT.md, docs/history/pi-full-workflow-parity/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-full-workflow-parity/delivery.md

---
type: bee.delivery
title: pi-full-workflow-parity — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-full-workflow-parity: 2 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-09-13
bee:
  id: pi-full-workflow-parity-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/pi-full-workflow-parity/CONTEXT.md, docs/history/pi-full-workflow-parity/plan.md]
  sources: [docs/history/pi-full-workflow-parity/CONTEXT.md, docs/history/pi-full-workflow-parity/plan.md, .bee/cells/pfp-1.json, .bee/cells/pfp-2.json]
---

# pi-full-workflow-parity — Delivery

## What shipped

- **pfp-1** — Add safe pause dismissal and close cleanup with projection synchronization (13 file(s) changed)
- **pfp-2** — Extend pi_lifecycle_end_to_end_onboarded_repo_parity with planned-next dismissal refusal, close refusal, adoption with claim fencing, pause dismissal, close, and orient (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pfp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml handoff && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts --test registry_dispatch` — handoff, close cleanup, and registry tests for pfp-1
- **pfp-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts pi_lifecycle_end_to_end_onboarded_repo_parity -- --exact --nocapture` — onboarded sandbox runs installed bee CLI end-to-end

## Deviations

- **pfp-1** — followed the plan
- **pfp-1** — sync-ack: cell pfp-1 declared affects_skills empty and updates internal workflow-state commands without changing skill interfaces
- **pfp-2** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work pi-full-workflow-parity` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-full-workflow-parity/CONTEXT.md`, `docs/history/pi-full-workflow-parity/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "pi-full-workflow-parity" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-13T14:51:36.097Z), the work item declares no bee.areas.

area workflow-state:
  - [pfp-1] Add safe pause dismissal and close cleanup with projection synchronization — feature-wide sync per the scribing stamp, 13 file(s) changed (trace .bee/cells/pfp-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pfp-1 — save as docs/knowledge/patterns/pi-full-workflow-parity-pfp-1-pitfall.md

---
type: bee.pattern
title: pi-full-workflow-parity cell pfp-1 — pitfall candidate
description: "Pitfall candidate mined from cell pfp-1's capped trace: followed the plan"
timestamp: 2026-09-13
bee:
  id: pi-full-workflow-parity-pfp-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/pfp-1.json]
  polarity: pitfall
---

# pi-full-workflow-parity cell pfp-1 — pitfall candidate

## What the cell did

Add safe pause dismissal and close cleanup with projection synchronization

## Recorded evidence (verbatim from .bee/cells/pfp-1.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: cell pfp-1 declared affects_skills empty and updates internal workflow-state commands without changing skill interfaces

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pfp-2 — save as docs/knowledge/patterns/pi-full-workflow-parity-pfp-2-pitfall.md

---
type: bee.pattern
title: pi-full-workflow-parity cell pfp-2 — pitfall candidate
description: "Pitfall candidate mined from cell pfp-2's capped trace: followed the plan"
timestamp: 2026-09-13
bee:
  id: pi-full-workflow-parity-pfp-2-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/pfp-2.json]
  polarity: pitfall
---

# pi-full-workflow-parity cell pfp-2 — pitfall candidate

## What the cell did

Extend pi_lifecycle_end_to_end_onboarded_repo_parity with planned-next dismissal refusal, close refusal, adoption with claim fencing, pause dismissal, close, and orient

## Recorded evidence (verbatim from .bee/cells/pfp-2.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 2 pattern candidate(s), 0 file(s) written.