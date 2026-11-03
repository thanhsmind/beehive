promote proposal for work item "pi-parity-review-fixes" (docs/history/pi-parity-review-fixes/CONTEXT.md + docs/history/pi-parity-review-fixes/plan.md) — 4 capped cell(s): pprf-1, pprf-2, pprf-3, pprf-4
anchor: history — docs/history/pi-parity-review-fixes/CONTEXT.md, docs/history/pi-parity-review-fixes/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-parity-review-fixes/delivery.md

---
type: bee.delivery
title: pi-parity-review-fixes — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-parity-review-fixes: 4 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-09-14
bee:
  id: pi-parity-review-fixes-delivery
  lifecycle: active
  areas: [hook-runtime, workflow-state]
  required_context: [docs/history/pi-parity-review-fixes/CONTEXT.md, docs/history/pi-parity-review-fixes/plan.md]
  sources: [docs/history/pi-parity-review-fixes/CONTEXT.md, docs/history/pi-parity-review-fixes/plan.md, .bee/cells/pprf-1.json, .bee/cells/pprf-2.json, .bee/cells/pprf-3.json, .bee/cells/pprf-4.json]
---

# pi-parity-review-fixes — Delivery

## What shipped

- **pprf-1** — Pi session initialization now injects runtime-correct dispatch guidance on normal and compact paths. (6 file(s) changed)
- **pprf-2** — Pi now records pre-tool and nested user-wait activity without weakening write protection. (5 file(s) changed)
- **pprf-3** — Linearize workflow close and mailbox writes under workflow then handoff locks (8 file(s) changed)
- **pprf-4** — Pi doctor now reports fail-closed runtime health and has an accurate installed verification recipe. (8 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pprf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml session_init && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts injected` — runtime normalization, both injected branches, extracted command execution, and Pi herding-only dispatch passed
- **pprf-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts activity && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test opencode_plugin_contracts pi_belt && .bee/bin/bee dev release-manifest --check` — four Pi activity tests, full six-test OpenCode parity suite, and 376 release-manifest entries passed
- **pprf-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml handoff && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test workflow_verbs` — verified handoff and workflow_verbs suites
- **pprf-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml doctor::tests` — 32 doctor tests cover ready and all required Pi refusal classes

## Deviations

- **pprf-1** — followed the plan
- **pprf-2** — followed the plan
- **pprf-2** — sync-ack: Pi lifecycle behavior and its existing hook-runtime knowledge home changed together; no skill workflow changed.
- **pprf-3** — followed the plan
- **pprf-3** — sync-ack: Internal lock linearization does not alter skill workflows
- **pprf-4** — Retried the verification recipe after the user authorized release — two earlier worker results missed its complete JSON contract — hit an unforeseen obstacle
- **pprf-4** — sync-ack: Doctor behavior and its existing hook-runtime knowledge home changed together; no skill workflow changed.

## Provenance

Proposed by `bee knowledge promote --work pi-parity-review-fixes` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-parity-review-fixes/CONTEXT.md`, `docs/history/pi-parity-review-fixes/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "pi-parity-review-fixes" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-14T03:41:38.144Z), the work item declares no bee.areas.

area hook-runtime:
  - [pprf-1] Pi session initialization now injects runtime-correct dispatch guidance on normal and compact paths. — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/pprf-1.json)
  - [pprf-2] Pi now records pre-tool and nested user-wait activity without weakening write protection. — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/pprf-2.json)
  - [pprf-3] Linearize workflow close and mailbox writes under workflow then handoff locks — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/pprf-3.json)
  - [pprf-4] Pi doctor now reports fail-closed runtime health and has an accurate installed verification recipe. — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/pprf-4.json)

area workflow-state:
  - [pprf-1] Pi session initialization now injects runtime-correct dispatch guidance on normal and compact paths. — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/pprf-1.json)
  - [pprf-2] Pi now records pre-tool and nested user-wait activity without weakening write protection. — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/pprf-2.json)
  - [pprf-3] Linearize workflow close and mailbox writes under workflow then handoff locks — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/pprf-3.json)
  - [pprf-4] Pi doctor now reports fail-closed runtime health and has an accurate installed verification recipe. — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/pprf-4.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pprf-1 — save as docs/knowledge/patterns/pi-parity-review-fixes-pprf-1-pitfall.md

---
type: bee.pattern
title: pi-parity-review-fixes cell pprf-1 — pitfall candidate
description: "Pitfall candidate mined from cell pprf-1's capped trace: followed the plan"
timestamp: 2026-09-13
bee:
  id: pi-parity-review-fixes-pprf-1-pitfall
  lifecycle: draft
  areas: [hook-runtime, workflow-state]
  sources: [.bee/cells/pprf-1.json]
  polarity: pitfall
---

# pi-parity-review-fixes cell pprf-1 — pitfall candidate

## What the cell did

Pi session initialization now injects runtime-correct dispatch guidance on normal and compact paths.

## Recorded evidence (verbatim from .bee/cells/pprf-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pprf-2 — save as docs/knowledge/patterns/pi-parity-review-fixes-pprf-2-pitfall.md

---
type: bee.pattern
title: pi-parity-review-fixes cell pprf-2 — pitfall candidate
description: "Pitfall candidate mined from cell pprf-2's capped trace: followed the plan"
timestamp: 2026-09-14
bee:
  id: pi-parity-review-fixes-pprf-2-pitfall
  lifecycle: draft
  areas: [hook-runtime, workflow-state]
  sources: [.bee/cells/pprf-2.json]
  polarity: pitfall
---

# pi-parity-review-fixes cell pprf-2 — pitfall candidate

## What the cell did

Pi now records pre-tool and nested user-wait activity without weakening write protection.

## Recorded evidence (verbatim from .bee/cells/pprf-2.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: Pi lifecycle behavior and its existing hook-runtime knowledge home changed together; no skill workflow changed.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pprf-3 — save as docs/knowledge/patterns/pi-parity-review-fixes-pprf-3-pitfall.md

---
type: bee.pattern
title: pi-parity-review-fixes cell pprf-3 — pitfall candidate
description: "Pitfall candidate mined from cell pprf-3's capped trace: followed the plan"
timestamp: 2026-09-13
bee:
  id: pi-parity-review-fixes-pprf-3-pitfall
  lifecycle: draft
  areas: [hook-runtime, workflow-state]
  sources: [.bee/cells/pprf-3.json]
  polarity: pitfall
---

# pi-parity-review-fixes cell pprf-3 — pitfall candidate

## What the cell did

Linearize workflow close and mailbox writes under workflow then handoff locks

## Recorded evidence (verbatim from .bee/cells/pprf-3.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: Internal lock linearization does not alter skill workflows

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pprf-4 — save as docs/knowledge/patterns/pi-parity-review-fixes-pprf-4-pitfall.md

---
type: bee.pattern
title: pi-parity-review-fixes cell pprf-4 — pitfall candidate
description: "Pitfall candidate mined from cell pprf-4's capped trace: Retried the verification recipe after the user authorized release — two earlier worker results missed its complete JSON contract — hit an unforeseen obstacle"
timestamp: 2026-09-14
bee:
  id: pi-parity-review-fixes-pprf-4-pitfall
  lifecycle: draft
  areas: [hook-runtime, workflow-state]
  sources: [.bee/cells/pprf-4.json]
  polarity: pitfall
---

# pi-parity-review-fixes cell pprf-4 — pitfall candidate

## What the cell did

Pi doctor now reports fail-closed runtime health and has an accurate installed verification recipe.

## Recorded evidence (verbatim from .bee/cells/pprf-4.json)

- **deviation** — Retried the verification recipe after the user authorized release — two earlier worker results missed its complete JSON contract — hit an unforeseen obstacle
- **deviation** — sync-ack: Doctor behavior and its existing hook-runtime knowledge home changed together; no skill workflow changed.
- **failure_signature** — 81512c5fb302
- **failure_signature** — bb7662a134d3

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 8 area bullet(s), 4 pattern candidate(s), 0 file(s) written.