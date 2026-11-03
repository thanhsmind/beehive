promote proposal for work item "issue-sweep-2" (docs/history/issue-sweep-2/CONTEXT.md + docs/history/issue-sweep-2/plan.md) — 3 capped cell(s): is2-1, is2-2, is2-3
anchor: history — docs/history/issue-sweep-2/CONTEXT.md, docs/history/issue-sweep-2/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/issue-sweep-2/delivery.md

---
type: bee.delivery
title: issue-sweep-2 — delivery
description: "Delivery record proposed by bee knowledge promote for work item issue-sweep-2: 3 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-11-03
bee:
  id: issue-sweep-2-delivery
  lifecycle: active
  areas: [workflow-state, worktree-parallelism]
  required_context: [docs/history/issue-sweep-2/CONTEXT.md, docs/history/issue-sweep-2/plan.md]
  sources: [docs/history/issue-sweep-2/CONTEXT.md, docs/history/issue-sweep-2/plan.md, .bee/cells/is2-1.json, .bee/cells/is2-2.json, .bee/cells/is2-3.json]
---

# issue-sweep-2 — Delivery

## What shipped

- **is2-1** — Added bee state session attach as an alias of the session bind handler, declared it in the registry with the same flags, and named it in every FIX line under verbs/state_group — 60e2285ab (9 file(s) changed)
- **is2-2** — Routing door now reads comma and multi-id D-ID lists on feature-local decisions (close.rs feature_local_hit + parse_d_sequence comma continuation), red-first proven; commit 89df525d9 (3 file(s) changed)
- **is2-3** — worktree merge now excludes untracked bee-* skill trees that also exist under the same root in main, so bee's own synced copies no longer refuse as dirty; a bee-* tree only the worktree holds still refuses. Commit 961c7e077. (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **is2-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee session && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts` — full declared suite on the issue-sweep-2 worktree rerun by the leader: 5053 passed 0 failed; worker recorded red before the fix
- **is2-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee routing` — full declared suite on the issue-sweep-2 worktree rerun by the leader: 5053 passed 0 failed; worker recorded red before the fix
- **is2-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee worktree` — full declared suite on the issue-sweep-2 worktree rerun by the leader: 5053 passed 0 failed; worker recorded red before the fix

## Deviations

- **is2-1** — also edited tests/route_lane_targeting.rs (assertions pinned the old FIX text) and hooks/session_preamble/budget.rs (COMMAND_SURFACE_BUDGET_CHARS 2600 to 2604 for the longer verb)
- **is2-1** — sync-ack: no skill names state session bind; the alias reaches users through FIX lines
- **is2-2** — sync-ack: close routing door internals; no skill teaches this matcher
- **is2-3** — sync-ack: worktree merge dirty check internals; no skill step changes

## Provenance

Proposed by `bee knowledge promote --work issue-sweep-2` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/issue-sweep-2/CONTEXT.md`, `docs/history/issue-sweep-2/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "issue-sweep-2" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-03T09:20:48.200Z), the work item declares no bee.areas.

area workflow-state:
  (no capped behavior_change cell exists for this feature)

area worktree-parallelism:
  (no capped behavior_change cell exists for this feature)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell is2-1 — save as docs/knowledge/patterns/issue-sweep-2-is2-1-pitfall.md

---
type: bee.pattern
title: issue-sweep-2 cell is2-1 — pitfall candidate
description: "Pitfall candidate mined from cell is2-1's capped trace: also edited tests/route_lane_targeting.rs (assertions pinned the old FIX text) and hooks/session_preamble/budget.rs (COMMAND_SURFACE_BUDGET_CHARS 2600 to 2604 …"
timestamp: 2026-11-03
bee:
  id: issue-sweep-2-is2-1-pitfall
  lifecycle: draft
  areas: [workflow-state, worktree-parallelism]
  sources: [.bee/cells/is2-1.json]
  polarity: pitfall
---

# issue-sweep-2 cell is2-1 — pitfall candidate

## What the cell did

Added bee state session attach as an alias of the session bind handler, declared it in the registry with the same flags, and named it in every FIX line under verbs/state_group — 60e2285ab

## Recorded evidence (verbatim from .bee/cells/is2-1.json)

- **deviation** — also edited tests/route_lane_targeting.rs (assertions pinned the old FIX text) and hooks/session_preamble/budget.rs (COMMAND_SURFACE_BUDGET_CHARS 2600 to 2604 for the longer verb)
- **deviation** — sync-ack: no skill names state session bind; the alias reaches users through FIX lines

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell is2-2 — save as docs/knowledge/patterns/issue-sweep-2-is2-2-pitfall.md

---
type: bee.pattern
title: issue-sweep-2 cell is2-2 — pitfall candidate
description: "Pitfall candidate mined from cell is2-2's capped trace: sync-ack: close routing door internals; no skill teaches this matcher"
timestamp: 2026-11-03
bee:
  id: issue-sweep-2-is2-2-pitfall
  lifecycle: draft
  areas: [workflow-state, worktree-parallelism]
  sources: [.bee/cells/is2-2.json]
  polarity: pitfall
---

# issue-sweep-2 cell is2-2 — pitfall candidate

## What the cell did

Routing door now reads comma and multi-id D-ID lists on feature-local decisions (close.rs feature_local_hit + parse_d_sequence comma continuation), red-first proven; commit 89df525d9

## Recorded evidence (verbatim from .bee/cells/is2-2.json)

- **deviation** — sync-ack: close routing door internals; no skill teaches this matcher

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell is2-3 — save as docs/knowledge/patterns/issue-sweep-2-is2-3-pitfall.md

---
type: bee.pattern
title: issue-sweep-2 cell is2-3 — pitfall candidate
description: "Pitfall candidate mined from cell is2-3's capped trace: sync-ack: worktree merge dirty check internals; no skill step changes"
timestamp: 2026-11-03
bee:
  id: issue-sweep-2-is2-3-pitfall
  lifecycle: draft
  areas: [workflow-state, worktree-parallelism]
  sources: [.bee/cells/is2-3.json]
  polarity: pitfall
---

# issue-sweep-2 cell is2-3 — pitfall candidate

## What the cell did

worktree merge now excludes untracked bee-* skill trees that also exist under the same root in main, so bee's own synced copies no longer refuse as dirty; a bee-* tree only the worktree holds still refuses. Commit 961c7e077.

## Recorded evidence (verbatim from .bee/cells/is2-3.json)

- **deviation** — sync-ack: worktree merge dirty check internals; no skill step changes

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 3 pattern candidate(s), 0 file(s) written.