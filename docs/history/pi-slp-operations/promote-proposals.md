promote proposal for work item "pi-slp-operations" (docs/history/pi-slp-operations/CONTEXT.md + docs/history/pi-slp-operations/plan.md) — 4 capped cell(s): rfx-1, rfx-2, rfx-3, rfx-4
anchor: history — docs/history/pi-slp-operations/CONTEXT.md, docs/history/pi-slp-operations/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-slp-operations/delivery.md

---
type: bee.delivery
title: pi-slp-operations — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-slp-operations: 4 capped cell(s), 6 recorded deviation(s)."
timestamp: 2026-10-03
bee:
  id: pi-slp-operations-delivery
  lifecycle: active
  required_context: [docs/history/pi-slp-operations/CONTEXT.md, docs/history/pi-slp-operations/plan.md]
  sources: [docs/history/pi-slp-operations/CONTEXT.md, docs/history/pi-slp-operations/plan.md, .bee/cells/rfx-1.json, .bee/cells/rfx-2.json, .bee/cells/rfx-3.json, .bee/cells/rfx-4.json]
---

# pi-slp-operations — Delivery

## What shipped

- **rfx-1** — Deployment dispatch refusals name one runnable command: bee gate --preview, the full deployment prepare form, or bee state lanes from main (2 file(s) changed)
- **rfx-2** — startFeature names session bind; CLAIMED names the full cell prepare form when the caller holds it, else claim-next (6 file(s) changed)
- **rfx-3** — Write-guard denies name the in-worktree path, bee --help --json for .bee/ paths, or the caller's own worktree root for a sibling worktree (2 file(s) changed)
- **rfx-4** — Preview mismatch refusals keep their first sentence and name the field and docs/history/<feature>/plan.md (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rfx-1** — `cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::drivers` — 429 passed; asserts for gate --preview, prepare --runtime with --role deploy, and state lanes added to existing refusal tests
- **rfx-2** — `cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::state_group && cargo test --release -p bee --bin bee verbs::cells && cargo test --release -p bee --test concurrency` — state_group 242, cells 355, concurrency 19; same-session and other-session CLAIMED asserted
- **rfx-3** — `cd packages/bee-rs && cargo test --release -p bee --bin bee hooks::write_guard` — 279 passed; new cross_worktree_denials test covers source path, .bee path and sibling worktree
- **rfx-4** — `cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::state_group::plan_packets` — covered inside the verbs::state_group run, 242 passed

## Deviations

- **rfx-1** — round 1 named the deployment prepare form without --role deploy, which the deploy permit requires; round 2 added it (525d6e7e2) — the plan was wrong about a fact
- **rfx-2** — no test for a sessionless holder; the code treats it as another session (equal only when both sessions are present), checked by reading claims.rs — something else had to be fixed first
- **rfx-2** — sync-ack: only refusal fix wording changed; no workflow-state skill quotes these refusal texts (rg over skills/ found none)
- **rfx-3** — round 1 returned early with no deny when root could not be canonicalized; round 2 falls back to the raw root (b9a08fbb0) so no deny can disappear — hit an unforeseen obstacle
- **rfx-4** — followed the plan
- **rfx-4** — sync-ack: only refusal fix wording changed; no workflow-state skill quotes these refusal texts (rg over skills/ found none)

## Provenance

Proposed by `bee knowledge promote --work pi-slp-operations` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-slp-operations/CONTEXT.md`, `docs/history/pi-slp-operations/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rfx-1 — save as docs/knowledge/patterns/pi-slp-operations-rfx-1-pitfall.md

---
type: bee.pattern
title: pi-slp-operations cell rfx-1 — pitfall candidate
description: "Pitfall candidate mined from cell rfx-1's capped trace: round 1 named the deployment prepare form without --role deploy, which the deploy permit requires; round 2 added it (525d6e7e2) — the plan was wrong about a fa…"
timestamp: 2026-10-03
bee:
  id: pi-slp-operations-rfx-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rfx-1.json]
  polarity: pitfall
---

# pi-slp-operations cell rfx-1 — pitfall candidate

## What the cell did

Deployment dispatch refusals name one runnable command: bee gate --preview, the full deployment prepare form, or bee state lanes from main

## Recorded evidence (verbatim from .bee/cells/rfx-1.json)

- **deviation** — round 1 named the deployment prepare form without --role deploy, which the deploy permit requires; round 2 added it (525d6e7e2) — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell rfx-2 — save as docs/knowledge/patterns/pi-slp-operations-rfx-2-pitfall.md

---
type: bee.pattern
title: pi-slp-operations cell rfx-2 — pitfall candidate
description: "Pitfall candidate mined from cell rfx-2's capped trace: no test for a sessionless holder; the code treats it as another session (equal only when both sessions are present), checked by reading claims.rs — something e…"
timestamp: 2026-10-03
bee:
  id: pi-slp-operations-rfx-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/rfx-2.json]
  polarity: pitfall
---

# pi-slp-operations cell rfx-2 — pitfall candidate

## What the cell did

startFeature names session bind; CLAIMED names the full cell prepare form when the caller holds it, else claim-next

## Recorded evidence (verbatim from .bee/cells/rfx-2.json)

- **deviation** — no test for a sessionless holder; the code treats it as another session (equal only when both sessions are present), checked by reading claims.rs — something else had to be fixed first
- **deviation** — sync-ack: only refusal fix wording changed; no workflow-state skill quotes these refusal texts (rg over skills/ found none)

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell rfx-3 — save as docs/knowledge/patterns/pi-slp-operations-rfx-3-pitfall.md

---
type: bee.pattern
title: pi-slp-operations cell rfx-3 — pitfall candidate
description: "Pitfall candidate mined from cell rfx-3's capped trace: round 1 returned early with no deny when root could not be canonicalized; round 2 falls back to the raw root (b9a08fbb0) so no deny can disappear — hit an unfo…"
timestamp: 2026-10-03
bee:
  id: pi-slp-operations-rfx-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/rfx-3.json]
  polarity: pitfall
---

# pi-slp-operations cell rfx-3 — pitfall candidate

## What the cell did

Write-guard denies name the in-worktree path, bee --help --json for .bee/ paths, or the caller's own worktree root for a sibling worktree

## Recorded evidence (verbatim from .bee/cells/rfx-3.json)

- **deviation** — round 1 returned early with no deny when root could not be canonicalized; round 2 falls back to the raw root (b9a08fbb0) so no deny can disappear — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell rfx-4 — save as docs/knowledge/patterns/pi-slp-operations-rfx-4-pitfall.md

---
type: bee.pattern
title: pi-slp-operations cell rfx-4 — pitfall candidate
description: "Pitfall candidate mined from cell rfx-4's capped trace: followed the plan"
timestamp: 2026-10-03
bee:
  id: pi-slp-operations-rfx-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/rfx-4.json]
  polarity: pitfall
---

# pi-slp-operations cell rfx-4 — pitfall candidate

## What the cell did

Preview mismatch refusals keep their first sentence and name the field and docs/history/<feature>/plan.md

## Recorded evidence (verbatim from .bee/cells/rfx-4.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: only refusal fix wording changed; no workflow-state skill quotes these refusal texts (rg over skills/ found none)

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 4 pattern candidate(s), 0 file(s) written.