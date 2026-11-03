promote proposal for work item "herding-caps-on-clean-report" (docs/history/herding-caps-on-clean-report/CONTEXT.md + docs/history/herding-caps-on-clean-report/plan.md) — 2 capped cell(s): wci-1, wci-2
anchor: history — docs/history/herding-caps-on-clean-report/CONTEXT.md, docs/history/herding-caps-on-clean-report/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-caps-on-clean-report/delivery.md

---
type: bee.delivery
title: herding-caps-on-clean-report — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-caps-on-clean-report: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-19
bee:
  id: herding-caps-on-clean-report-delivery
  lifecycle: active
  areas: [dispatch-door, herding, workflow-state]
  required_context: [docs/history/herding-caps-on-clean-report/CONTEXT.md, docs/history/herding-caps-on-clean-report/plan.md]
  sources: [docs/history/herding-caps-on-clean-report/CONTEXT.md, docs/history/herding-caps-on-clean-report/plan.md, .bee/cells/wci-1.json, .bee/cells/wci-2.json]
---

# herding-caps-on-clean-report — Delivery

## What shipped

- **wci-1** — Made finish instruction runnable, single, and terminal at the end of worker-cell prompt (5 file(s) changed)
- **wci-2** — Dispatched real execution worker in control-bee sandbox and verified self-cap (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wci-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee onboard && .bee/bin/bee dev release-manifest --check` — cell verify passed onboard unit tests and release manifest check
- **wci-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee onboard` — live drive in sandbox verified worker self-cap

## Deviations

- **wci-1** — checked worktree root for prompt skew in prepare.rs — dispatch prepare in a granted worktree would otherwise compare against main — hit an unforeseen obstacle
- **wci-2** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work herding-caps-on-clean-report` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-caps-on-clean-report/CONTEXT.md`, `docs/history/herding-caps-on-clean-report/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-caps-on-clean-report" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-19T05:37:54.605Z), the work item declares no bee.areas.

area dispatch-door:
  - [wci-1] Made finish instruction runnable, single, and terminal at the end of worker-cell prompt — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/wci-1.json)

area herding:
  - [wci-1] Made finish instruction runnable, single, and terminal at the end of worker-cell prompt — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/wci-1.json)

area workflow-state:
  - [wci-1] Made finish instruction runnable, single, and terminal at the end of worker-cell prompt — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/wci-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wci-1 — save as docs/knowledge/patterns/herding-caps-on-clean-report-wci-1-pitfall.md

---
type: bee.pattern
title: herding-caps-on-clean-report cell wci-1 — pitfall candidate
description: "Pitfall candidate mined from cell wci-1's capped trace: checked worktree root for prompt skew in prepare.rs — dispatch prepare in a granted worktree would otherwise compare against main — hit an unforeseen obstacle"
timestamp: 2026-09-19
bee:
  id: herding-caps-on-clean-report-wci-1-pitfall
  lifecycle: draft
  areas: [dispatch-door, herding, workflow-state]
  sources: [.bee/cells/wci-1.json]
  polarity: pitfall
---

# herding-caps-on-clean-report cell wci-1 — pitfall candidate

## What the cell did

Made finish instruction runnable, single, and terminal at the end of worker-cell prompt

## Recorded evidence (verbatim from .bee/cells/wci-1.json)

- **deviation** — checked worktree root for prompt skew in prepare.rs — dispatch prepare in a granted worktree would otherwise compare against main — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell wci-2 — save as docs/knowledge/patterns/herding-caps-on-clean-report-wci-2-pitfall.md

---
type: bee.pattern
title: herding-caps-on-clean-report cell wci-2 — pitfall candidate
description: "Pitfall candidate mined from cell wci-2's capped trace: followed the plan"
timestamp: 2026-09-19
bee:
  id: herding-caps-on-clean-report-wci-2-pitfall
  lifecycle: draft
  areas: [dispatch-door, herding, workflow-state]
  sources: [.bee/cells/wci-2.json]
  polarity: pitfall
---

# herding-caps-on-clean-report cell wci-2 — pitfall candidate

## What the cell did

Dispatched real execution worker in control-bee sandbox and verified self-cap

## Recorded evidence (verbatim from .bee/cells/wci-2.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 3 area bullet(s), 2 pattern candidate(s), 0 file(s) written.