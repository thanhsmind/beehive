promote proposal for work item "paseo-worktree-workspace" (docs/history/paseo-worktree-workspace/CONTEXT.md + docs/history/paseo-worktree-workspace/plan.md) — 4 capped cell(s): pw-1, pw-2, pw-3, pw-4
anchor: history — docs/history/paseo-worktree-workspace/CONTEXT.md, docs/history/paseo-worktree-workspace/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/paseo-worktree-workspace/delivery.md

---
type: bee.delivery
title: paseo-worktree-workspace — delivery
description: "Delivery record proposed by bee knowledge promote for work item paseo-worktree-workspace: 4 capped cell(s), 9 recorded deviation(s)."
timestamp: 2026-10-15
bee:
  id: paseo-worktree-workspace-delivery
  lifecycle: active
  required_context: [docs/history/paseo-worktree-workspace/CONTEXT.md, docs/history/paseo-worktree-workspace/plan.md]
  sources: [docs/history/paseo-worktree-workspace/CONTEXT.md, docs/history/paseo-worktree-workspace/plan.md, .bee/cells/pw-1.json, .bee/cells/pw-2.json, .bee/cells/pw-3.json, .bee/cells/pw-4.json]
---

# paseo-worktree-workspace — Delivery

## What shipped

- **pw-1** — Add Paseo workspace helpers and drop agent-scope env from every paseo child (4 file(s) changed)
- **pw-2** — Register worktrees as Paseo workspaces and archive them on teardown (7 file(s) changed)
- **pw-3** — Start Paseo workers in their worktree's workspace (2 file(s) changed)
- **pw-4** — Paseo channel concept gains a Worktree workspaces section (D1-D6, decision 57793b86) and paseo_ws.rs pointer (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pw-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding:: && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prepare` — 742, 69 and 4 passed, 0 failed; the cell touches herding and the prepare probe only
- **pw-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee worktree && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee workspace_store` — 300, 7, 1 and 16 passed, 0 failed; the cell touches worktree verbs and the workspace store only
- **pw-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::` — 746 passed, 0 failed; the cell touches the herding transport only
- **pw-4** — `rg -n 'paseo_workspace_id|--workspace|PASEO_AGENT_CWD|paseo_ws.rs' docs/knowledge/areas/bee-herding/the-paseo-channel.md` — docs-only cell; verify matched 9 lines; bee knowledge check 0 errors and bee knowledge index --check 0 stale also run

## Deviations

- **pw-1** — Executed inline by the leader after escalation: the paseo-pi-flash worker timed out idle after 900 s with no write
- **pw-1** — sync-ack: internal transport helpers; no skill-facing behavior changes; the concept update rides pw-4
- **pw-2** — Executed by the leader on the session model (escalated, decision logged after pw-1 worker idle timeout)
- **pw-2** — read_main_config is private to herding::run, so paseo_ws reads .bee/config.json inline
- **pw-2** — sync-ack: worktree verbs internal wiring; concept update rides pw-4
- **pw-3** — Executed by the leader on the session model (escalated)
- **pw-3** — The wait source and prepare probe scrubs moved to pw-1 per the hat wave
- **pw-3** — sync-ack: herding transport internal; concept update rides pw-4
- **pw-4** — Also rewrote the parent-cascade lines (How a run executes, Agent archival heading and body) and added --workspace to the run argv list — D5 makes workers non-children, so the old D10 text was now false — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work paseo-worktree-workspace` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/paseo-worktree-workspace/CONTEXT.md`, `docs/history/paseo-worktree-workspace/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pw-1 — save as docs/knowledge/patterns/paseo-worktree-workspace-pw-1-pitfall.md

---
type: bee.pattern
title: paseo-worktree-workspace cell pw-1 — pitfall candidate
description: "Pitfall candidate mined from cell pw-1's capped trace: Executed inline by the leader after escalation: the paseo-pi-flash worker timed out idle after 900 s with no write"
timestamp: 2026-10-15
bee:
  id: paseo-worktree-workspace-pw-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/pw-1.json]
  polarity: pitfall
---

# paseo-worktree-workspace cell pw-1 — pitfall candidate

## What the cell did

Add Paseo workspace helpers and drop agent-scope env from every paseo child

## Recorded evidence (verbatim from .bee/cells/pw-1.json)

- **deviation** — Executed inline by the leader after escalation: the paseo-pi-flash worker timed out idle after 900 s with no write
- **deviation** — sync-ack: internal transport helpers; no skill-facing behavior changes; the concept update rides pw-4

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pw-2 — save as docs/knowledge/patterns/paseo-worktree-workspace-pw-2-pitfall.md

---
type: bee.pattern
title: paseo-worktree-workspace cell pw-2 — pitfall candidate
description: "Pitfall candidate mined from cell pw-2's capped trace: Executed by the leader on the session model (escalated, decision logged after pw-1 worker idle timeout)"
timestamp: 2026-10-15
bee:
  id: paseo-worktree-workspace-pw-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/pw-2.json]
  polarity: pitfall
---

# paseo-worktree-workspace cell pw-2 — pitfall candidate

## What the cell did

Register worktrees as Paseo workspaces and archive them on teardown

## Recorded evidence (verbatim from .bee/cells/pw-2.json)

- **deviation** — Executed by the leader on the session model (escalated, decision logged after pw-1 worker idle timeout)
- **deviation** — read_main_config is private to herding::run, so paseo_ws reads .bee/config.json inline
- **deviation** — sync-ack: worktree verbs internal wiring; concept update rides pw-4

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pw-3 — save as docs/knowledge/patterns/paseo-worktree-workspace-pw-3-pitfall.md

---
type: bee.pattern
title: paseo-worktree-workspace cell pw-3 — pitfall candidate
description: "Pitfall candidate mined from cell pw-3's capped trace: Executed by the leader on the session model (escalated)"
timestamp: 2026-10-15
bee:
  id: paseo-worktree-workspace-pw-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/pw-3.json]
  polarity: pitfall
---

# paseo-worktree-workspace cell pw-3 — pitfall candidate

## What the cell did

Start Paseo workers in their worktree's workspace

## Recorded evidence (verbatim from .bee/cells/pw-3.json)

- **deviation** — Executed by the leader on the session model (escalated)
- **deviation** — The wait source and prepare probe scrubs moved to pw-1 per the hat wave
- **deviation** — sync-ack: herding transport internal; concept update rides pw-4

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pw-4 — save as docs/knowledge/patterns/paseo-worktree-workspace-pw-4-pitfall.md

---
type: bee.pattern
title: paseo-worktree-workspace cell pw-4 — pitfall candidate
description: "Pitfall candidate mined from cell pw-4's capped trace: Also rewrote the parent-cascade lines (How a run executes, Agent archival heading and body) and added --workspace to the run argv list — D5 makes workers non-c…"
timestamp: 2026-10-15
bee:
  id: paseo-worktree-workspace-pw-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/pw-4.json]
  polarity: pitfall
---

# paseo-worktree-workspace cell pw-4 — pitfall candidate

## What the cell did

Paseo channel concept gains a Worktree workspaces section (D1-D6, decision 57793b86) and paseo_ws.rs pointer

## Recorded evidence (verbatim from .bee/cells/pw-4.json)

- **deviation** — Also rewrote the parent-cascade lines (How a run executes, Agent archival heading and body) and added --workspace to the run argv list — D5 makes workers non-children, so the old D10 text was now false — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 4 pattern candidate(s), 0 file(s) written.