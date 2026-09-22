promote proposal for work item "skill-trigger-cases" (docs/history/skill-trigger-cases/CONTEXT.md + docs/history/skill-trigger-cases/plan.md) — 2 capped cell(s): skt-1, skt-2
anchor: history — docs/history/skill-trigger-cases/CONTEXT.md, docs/history/skill-trigger-cases/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/skill-trigger-cases/delivery.md

---
type: bee.delivery
title: skill-trigger-cases — delivery
description: "Delivery record proposed by bee knowledge promote for work item skill-trigger-cases: 2 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-09-22
bee:
  id: skill-trigger-cases-delivery
  lifecycle: active
  required_context: [docs/history/skill-trigger-cases/CONTEXT.md, docs/history/skill-trigger-cases/plan.md]
  sources: [docs/history/skill-trigger-cases/CONTEXT.md, docs/history/skill-trigger-cases/plan.md, .bee/cells/skt-1.json, .bee/cells/skt-2.json]
---

# skill-trigger-cases — Delivery

## What shipped

- **skt-1** — Fence every skill description with 104 trigger briefs plus an opt-in real-agent eval (2 file(s) changed)
- **skt-2** — bee-writing-skills checklist now requires trigger cases in the shared fixture (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **skt-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers` — the cell verify, and the only target this cell adds; two new files, no existing file touched, and bee dev release-manifest --check stayed green. The whole declared suite was NOT run
- **skt-2** — `.bee/bin/bee dev release-manifest --check` — docs-only row plus regen parity of the rendered copies; the cell verify command

## Deviations

- **skt-1** — followed the plan
- **skt-1** — Ran the verify command from a one-line scratchpad script instead of inline — the worktree shell guard refuses an inline cd plus a PATH prefix built from CARGO_HOME as unverifiable — hit an unforeseen obstacle
- **skt-2** — Wrote the row with a comma (near misses that must not, to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json) — the cell action text read must not to path, which is not a sentence — found a better route
- **skt-2** — Left .bee/onboarding.json dirty (updated_at timestamp only) instead of restoring it — the concurrent-worker guard refuses a tree-wide revert while a sibling worker is live in this checkout — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work skill-trigger-cases` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/skill-trigger-cases/CONTEXT.md`, `docs/history/skill-trigger-cases/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell skt-1 — save as docs/knowledge/patterns/skill-trigger-cases-skt-1-pitfall.md

---
type: bee.pattern
title: skill-trigger-cases cell skt-1 — pitfall candidate
description: "Pitfall candidate mined from cell skt-1's capped trace: followed the plan"
timestamp: 2026-09-22
bee:
  id: skill-trigger-cases-skt-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/skt-1.json]
  polarity: pitfall
---

# skill-trigger-cases cell skt-1 — pitfall candidate

## What the cell did

Fence every skill description with 104 trigger briefs plus an opt-in real-agent eval

## Recorded evidence (verbatim from .bee/cells/skt-1.json)

- **deviation** — followed the plan
- **deviation** — Ran the verify command from a one-line scratchpad script instead of inline — the worktree shell guard refuses an inline cd plus a PATH prefix built from CARGO_HOME as unverifiable — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell skt-2 — save as docs/knowledge/patterns/skill-trigger-cases-skt-2-pitfall.md

---
type: bee.pattern
title: skill-trigger-cases cell skt-2 — pitfall candidate
description: "Pitfall candidate mined from cell skt-2's capped trace: Wrote the row with a comma (near misses that must not, to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json) — the cell action text read must not t…"
timestamp: 2026-09-22
bee:
  id: skill-trigger-cases-skt-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/skt-2.json]
  polarity: pitfall
---

# skill-trigger-cases cell skt-2 — pitfall candidate

## What the cell did

bee-writing-skills checklist now requires trigger cases in the shared fixture

## Recorded evidence (verbatim from .bee/cells/skt-2.json)

- **deviation** — Wrote the row with a comma (near misses that must not, to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json) — the cell action text read must not to path, which is not a sentence — found a better route
- **deviation** — Left .bee/onboarding.json dirty (updated_at timestamp only) instead of restoring it — the concurrent-worker guard refuses a tree-wide revert while a sibling worker is live in this checkout — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.