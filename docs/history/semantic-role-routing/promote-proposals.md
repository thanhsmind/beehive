promote proposal for work item "semantic-role-routing" (docs/history/semantic-role-routing/CONTEXT.md + docs/history/semantic-role-routing/plan.md) — 8 capped cell(s): slr-1, slr-2, slr-3, slr-4, slr-5, slr-6, slr-7, slr-8
anchor: history — docs/history/semantic-role-routing/CONTEXT.md, docs/history/semantic-role-routing/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/semantic-role-routing/delivery.md

---
type: bee.delivery
title: semantic-role-routing — delivery
description: "Delivery record proposed by bee knowledge promote for work item semantic-role-routing: 8 capped cell(s), 15 recorded deviation(s)."
timestamp: 2026-09-14
bee:
  id: semantic-role-routing-delivery
  lifecycle: active
  required_context: [docs/history/semantic-role-routing/CONTEXT.md, docs/history/semantic-role-routing/plan.md]
  sources: [docs/history/semantic-role-routing/CONTEXT.md, docs/history/semantic-role-routing/plan.md, .bee/cells/slr-1.json, .bee/cells/slr-2.json, .bee/cells/slr-3.json, .bee/cells/slr-4.json, .bee/cells/slr-5.json, .bee/cells/slr-6.json, .bee/cells/slr-7.json, .bee/cells/slr-8.json]
---

# semantic-role-routing — Delivery

## What shipped

- **slr-1** — Planning SKILL.md updated with role assignment step; planning-reference has Role assignments section; AGENTS.md has dispatch-follows-plan rule; swarming-reference cites cell role as plan-approved assignment (4 file(s) changed)
- **slr-2** — model-roles-and-escalation.md has B18 covering plan-time role assignment (D1 fc2bb09a) and re-route discipline (1 file(s) changed)
- **slr-3** — Approved bee-plan/v2 packets validate, store, and display complete semantic role assignments while legacy packets stay unchanged (3 file(s) changed)
- **slr-4** — Dispatch preparation now enforces approved runtime, stage, cell role, roster, and plan identities before deterministic resolution (2 file(s) changed)
- **slr-5** — Cell role reroutes are validated and audited under lock, while native guards enforce effective approved cell and stage identities (6 file(s) changed)
- **slr-6** — Release execution now requires a fresh one-use deploy authorization bound to UUID, issuer session, feature, runtime, approved plan, version, main commit, and a bounded two-hour lifetime (4 file(s) changed)
- **slr-7** — Bee now emits authenticated Pi transitions from one granted linked worktree to another without weakening existing checks (3 file(s) changed)
- **slr-8** — Regenerated source-owned host projections and proved semantic routing, deploy authorization, and linked-worktree relocation in a real Pi 0.84.4 RPC process (41 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **slr-1** — `grep -q 'role assignment' .agents/skills/bee-planning/SKILL.md && grep -q 're-route' AGENTS.md` — parity/pointer checks on 4 doc files
- **slr-2** — `grep -q 'B18' docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` — docs pointer check on knowledge area file
- **slr-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml plan_packets` — 15 role-plan packet tests passed
- **slr-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers::tests` — 287 driver tests passed and 8 remained intentionally ignored
- **slr-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml reroute && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml model_guard` — 5 reroute tests, 72 model-guard tests, and 3 host contract rows passed
- **slr-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml deploy && bash -n scripts/release.sh` — 4 deploy tests passed and release script syntax passed
- **slr-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml enter_worktree && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml pi_plugin_contracts` — approved command passed 7 worktree tests; supplemental --test pi_plugin_contracts proof passed all 66 contracts
- **slr-8** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check` — 3594 Rust tests passed, 20 ignored; all integration suites passed; 376 manifest files matched

## Deviations

- **slr-1** — followed the plan
- **slr-1** — sync-ack: cell slr-1 touches rule home AGENTS.md only; applied_at files (bee-capturing, bee-hive, routing-and-contracts) not in scope
- **slr-2** — followed the plan
- **slr-3** — Added skills/bee-planning/references/planning-reference.md — the approved cell listed only its generated .agents mirror, and the sync door exposed the source-of-truth path — something else had to be fixed first
- **slr-4** — Re-delivered slr-4 through a fresh pane — the first acknowledged pane became idle after read-only analysis — hit an unforeseen obstacle
- **slr-5** — Leader recovered the committed result after transport exited without a result record — the worker pane closed after commit but before cap — hit an unforeseen obstacle
- **slr-5** — sync-ack: Decision 7c5c7e0d assigns bee-swarming source, generated mirror, and full regen to integration cell slr-8 before close
- **slr-6** — Decision f5b5ee99 added the actual native router path drivers/close.rs after the first round found the plan fact wrong
- **slr-6** — The authorized final correction added red-first forged lifetime, unsafe UUID-path, and absent caller-session cases, then amended the same cell commit
- **slr-6** — sync-ack: Decision 7c5c7e0d assigns bee-swarming source, generated mirror, and full regen to integration cell slr-8 before close
- **slr-7** — Used --test pi_plugin_contracts for the integration suite — the plan name filter selected zero integration tests — the plan was wrong about a fact
- **slr-8** — Decision 0364a7e1 expanded the cell to the complete mandatory regen output set
- **slr-8** — Decision 4d104bc6 corrected the source home packages/bee/AGENTS.block.md before the final regen
- **slr-8** — The second herding call reported a stall, then recovered its completed result without another dispatch
- **slr-8** — sync-ack: AGENTS.md changed only through source-first regeneration from packages/bee/AGENTS.block.md for the new unmarked semantic dispatch bullet; no existing marked rule changed, and agents_block_render_parity passed

## Provenance

Proposed by `bee knowledge promote --work semantic-role-routing` from 8 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/semantic-role-routing/CONTEXT.md`, `docs/history/semantic-role-routing/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell slr-1 — save as docs/knowledge/patterns/semantic-role-routing-slr-1-pitfall.md

---
type: bee.pattern
title: semantic-role-routing cell slr-1 — pitfall candidate
description: "Pitfall candidate mined from cell slr-1's capped trace: followed the plan"
timestamp: 2026-09-14
bee:
  id: semantic-role-routing-slr-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/slr-1.json]
  polarity: pitfall
---

# semantic-role-routing cell slr-1 — pitfall candidate

## What the cell did

Planning SKILL.md updated with role assignment step; planning-reference has Role assignments section; AGENTS.md has dispatch-follows-plan rule; swarming-reference cites cell role as plan-approved assignment

## Recorded evidence (verbatim from .bee/cells/slr-1.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: cell slr-1 touches rule home AGENTS.md only; applied_at files (bee-capturing, bee-hive, routing-and-contracts) not in scope

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell slr-2 — save as docs/knowledge/patterns/semantic-role-routing-slr-2-pitfall.md

---
type: bee.pattern
title: semantic-role-routing cell slr-2 — pitfall candidate
description: "Pitfall candidate mined from cell slr-2's capped trace: followed the plan"
timestamp: 2026-09-14
bee:
  id: semantic-role-routing-slr-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/slr-2.json]
  polarity: pitfall
---

# semantic-role-routing cell slr-2 — pitfall candidate

## What the cell did

model-roles-and-escalation.md has B18 covering plan-time role assignment (D1 fc2bb09a) and re-route discipline

## Recorded evidence (verbatim from .bee/cells/slr-2.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell slr-3 — save as docs/knowledge/patterns/semantic-role-routing-slr-3-pitfall.md

---
type: bee.pattern
title: semantic-role-routing cell slr-3 — pitfall candidate
description: "Pitfall candidate mined from cell slr-3's capped trace: Added skills/bee-planning/references/planning-reference.md — the approved cell listed only its generated .agents mirror, and the sync door exposed the source-o…"
timestamp: 2026-09-14
bee:
  id: semantic-role-routing-slr-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/slr-3.json]
  polarity: pitfall
---

# semantic-role-routing cell slr-3 — pitfall candidate

## What the cell did

Approved bee-plan/v2 packets validate, store, and display complete semantic role assignments while legacy packets stay unchanged

## Recorded evidence (verbatim from .bee/cells/slr-3.json)

- **deviation** — Added skills/bee-planning/references/planning-reference.md — the approved cell listed only its generated .agents mirror, and the sync door exposed the source-of-truth path — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell slr-4 — save as docs/knowledge/patterns/semantic-role-routing-slr-4-pitfall.md

---
type: bee.pattern
title: semantic-role-routing cell slr-4 — pitfall candidate
description: "Pitfall candidate mined from cell slr-4's capped trace: Re-delivered slr-4 through a fresh pane — the first acknowledged pane became idle after read-only analysis — hit an unforeseen obstacle"
timestamp: 2026-09-14
bee:
  id: semantic-role-routing-slr-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/slr-4.json]
  polarity: pitfall
---

# semantic-role-routing cell slr-4 — pitfall candidate

## What the cell did

Dispatch preparation now enforces approved runtime, stage, cell role, roster, and plan identities before deterministic resolution

## Recorded evidence (verbatim from .bee/cells/slr-4.json)

- **deviation** — Re-delivered slr-4 through a fresh pane — the first acknowledged pane became idle after read-only analysis — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell slr-5 — save as docs/knowledge/patterns/semantic-role-routing-slr-5-pitfall.md

---
type: bee.pattern
title: semantic-role-routing cell slr-5 — pitfall candidate
description: "Pitfall candidate mined from cell slr-5's capped trace: Leader recovered the committed result after transport exited without a result record — the worker pane closed after commit but before cap — hit an unforeseen o…"
timestamp: 2026-09-14
bee:
  id: semantic-role-routing-slr-5-pitfall
  lifecycle: draft
  sources: [.bee/cells/slr-5.json]
  polarity: pitfall
---

# semantic-role-routing cell slr-5 — pitfall candidate

## What the cell did

Cell role reroutes are validated and audited under lock, while native guards enforce effective approved cell and stage identities

## Recorded evidence (verbatim from .bee/cells/slr-5.json)

- **deviation** — Leader recovered the committed result after transport exited without a result record — the worker pane closed after commit but before cap — hit an unforeseen obstacle
- **deviation** — sync-ack: Decision 7c5c7e0d assigns bee-swarming source, generated mirror, and full regen to integration cell slr-8 before close

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell slr-6 — save as docs/knowledge/patterns/semantic-role-routing-slr-6-pitfall.md

---
type: bee.pattern
title: semantic-role-routing cell slr-6 — pitfall candidate
description: "Pitfall candidate mined from cell slr-6's capped trace: Decision f5b5ee99 added the actual native router path drivers/close.rs after the first round found the plan fact wrong"
timestamp: 2026-09-14
bee:
  id: semantic-role-routing-slr-6-pitfall
  lifecycle: draft
  sources: [.bee/cells/slr-6.json]
  polarity: pitfall
---

# semantic-role-routing cell slr-6 — pitfall candidate

## What the cell did

Release execution now requires a fresh one-use deploy authorization bound to UUID, issuer session, feature, runtime, approved plan, version, main commit, and a bounded two-hour lifetime

## Recorded evidence (verbatim from .bee/cells/slr-6.json)

- **deviation** — Decision f5b5ee99 added the actual native router path drivers/close.rs after the first round found the plan fact wrong
- **deviation** — The authorized final correction added red-first forged lifetime, unsafe UUID-path, and absent caller-session cases, then amended the same cell commit
- **deviation** — sync-ack: Decision 7c5c7e0d assigns bee-swarming source, generated mirror, and full regen to integration cell slr-8 before close

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell slr-7 — save as docs/knowledge/patterns/semantic-role-routing-slr-7-pitfall.md

---
type: bee.pattern
title: semantic-role-routing cell slr-7 — pitfall candidate
description: "Pitfall candidate mined from cell slr-7's capped trace: Used --test pi_plugin_contracts for the integration suite — the plan name filter selected zero integration tests — the plan was wrong about a fact"
timestamp: 2026-09-14
bee:
  id: semantic-role-routing-slr-7-pitfall
  lifecycle: draft
  sources: [.bee/cells/slr-7.json]
  polarity: pitfall
---

# semantic-role-routing cell slr-7 — pitfall candidate

## What the cell did

Bee now emits authenticated Pi transitions from one granted linked worktree to another without weakening existing checks

## Recorded evidence (verbatim from .bee/cells/slr-7.json)

- **deviation** — Used --test pi_plugin_contracts for the integration suite — the plan name filter selected zero integration tests — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell slr-8 — save as docs/knowledge/patterns/semantic-role-routing-slr-8-pitfall.md

---
type: bee.pattern
title: semantic-role-routing cell slr-8 — pitfall candidate
description: "Pitfall candidate mined from cell slr-8's capped trace: Decision 0364a7e1 expanded the cell to the complete mandatory regen output set"
timestamp: 2026-09-14
bee:
  id: semantic-role-routing-slr-8-pitfall
  lifecycle: draft
  sources: [.bee/cells/slr-8.json]
  polarity: pitfall
---

# semantic-role-routing cell slr-8 — pitfall candidate

## What the cell did

Regenerated source-owned host projections and proved semantic routing, deploy authorization, and linked-worktree relocation in a real Pi 0.84.4 RPC process

## Recorded evidence (verbatim from .bee/cells/slr-8.json)

- **deviation** — Decision 0364a7e1 expanded the cell to the complete mandatory regen output set
- **deviation** — Decision 4d104bc6 corrected the source home packages/bee/AGENTS.block.md before the final regen
- **deviation** — The second herding call reported a stall, then recovered its completed result without another dispatch
- **deviation** — sync-ack: AGENTS.md changed only through source-first regeneration from packages/bee/AGENTS.block.md for the new unmarked semantic dispatch bullet; no existing marked rule changed, and agents_block_render_parity passed

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 8 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 8 pattern candidate(s), 0 file(s) written.