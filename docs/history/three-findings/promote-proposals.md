promote proposal for work item "three-findings" (docs/history/three-findings/CONTEXT.md + docs/history/three-findings/plan.md) — 3 capped cell(s): thf-1, thf-2, thf-3
anchor: history — docs/history/three-findings/CONTEXT.md, docs/history/three-findings/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/three-findings/delivery.md

---
type: bee.delivery
title: three-findings — delivery
description: "Delivery record proposed by bee knowledge promote for work item three-findings: 3 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-09-18
bee:
  id: three-findings-delivery
  lifecycle: active
  areas: [bee-herding, hook-runtime]
  required_context: [docs/history/three-findings/CONTEXT.md, docs/history/three-findings/plan.md]
  sources: [docs/history/three-findings/CONTEXT.md, docs/history/three-findings/plan.md, .bee/cells/archive/three-findings/thf-1.json, .bee/cells/archive/three-findings/thf-2.json, .bee/cells/archive/three-findings/thf-3.json]
---

# three-findings — Delivery

## What shipped

- **thf-1** — Delete the dead paths block from the dispatch prompts (9 file(s) changed)
- **thf-2** — Point the codex probe concept at the mechanism that exists (1 file(s) changed)
- **thf-3** — Publish the two herding flags in the command registry (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **thf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check && .bee/bin/bee onboard --repo-root . --json` — leader re-ran the exact compound: exit 0, 0 failures, manifest 376 files matched, onboard up_to_date; cmp also confirms each template matches its .bee/bin twin
- **thf-2** — `.bee/bin/bee knowledge check --json && .bee/bin/bee knowledge index --check --json` — leader re-ran both checks after the worker: errors 0, profile_errors 0, index drift false
- **thf-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — leader re-ran the declared suite on the merged tree: exit 0, 36 result blocks, 0 failures, 4068 passed, including the catalog pinned-count test at 210

## Deviations

- **thf-1** — followed the plan
- **thf-2** — followed the plan
- **thf-3** — The worker proved with a filtered subset (cargo test ... -p bee -- catalog) instead of the cell's approved verify, so the leader re-ran the full declared command before this cap — the worker narrowed the proof, the plan did not — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work three-findings` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/three-findings/CONTEXT.md`, `docs/history/three-findings/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "three-findings" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-18T14:48:55.417Z), the work item declares no bee.areas.

area bee-herding:
  - [thf-3] Publish the two herding flags in the command registry — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/archive/three-findings/thf-3.json)

area hook-runtime:
  - [thf-3] Publish the two herding flags in the command registry — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/archive/three-findings/thf-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell thf-1 — save as docs/knowledge/patterns/three-findings-thf-1-pitfall.md

---
type: bee.pattern
title: three-findings cell thf-1 — pitfall candidate
description: "Pitfall candidate mined from cell thf-1's capped trace: followed the plan"
timestamp: 2026-09-18
bee:
  id: three-findings-thf-1-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/archive/three-findings/thf-1.json]
  polarity: pitfall
---

# three-findings cell thf-1 — pitfall candidate

## What the cell did

Delete the dead paths block from the dispatch prompts

## Recorded evidence (verbatim from .bee/cells/archive/three-findings/thf-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell thf-2 — save as docs/knowledge/patterns/three-findings-thf-2-pitfall.md

---
type: bee.pattern
title: three-findings cell thf-2 — pitfall candidate
description: "Pitfall candidate mined from cell thf-2's capped trace: followed the plan"
timestamp: 2026-09-18
bee:
  id: three-findings-thf-2-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/archive/three-findings/thf-2.json]
  polarity: pitfall
---

# three-findings cell thf-2 — pitfall candidate

## What the cell did

Point the codex probe concept at the mechanism that exists

## Recorded evidence (verbatim from .bee/cells/archive/three-findings/thf-2.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell thf-3 — save as docs/knowledge/patterns/three-findings-thf-3-pitfall.md

---
type: bee.pattern
title: three-findings cell thf-3 — pitfall candidate
description: "Pitfall candidate mined from cell thf-3's capped trace: The worker proved with a filtered subset (cargo test ... -p bee -- catalog) instead of the cell's approved verify, so the leader re-ran the full declared comma…"
timestamp: 2026-09-18
bee:
  id: three-findings-thf-3-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/archive/three-findings/thf-3.json]
  polarity: pitfall
---

# three-findings cell thf-3 — pitfall candidate

## What the cell did

Publish the two herding flags in the command registry

## Recorded evidence (verbatim from .bee/cells/archive/three-findings/thf-3.json)

- **deviation** — The worker proved with a filtered subset (cargo test ... -p bee -- catalog) instead of the cell's approved verify, so the leader re-ran the full declared command before this cap — the worker narrowed the proof, the plan did not — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 3 pattern candidate(s), 0 file(s) written.