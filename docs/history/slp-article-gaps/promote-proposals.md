promote proposal for work item "slp-article-gaps" (docs/history/slp-article-gaps/CONTEXT.md) — 2 capped cell(s): sag-1, sag-2
anchor: history — docs/history/slp-article-gaps/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/slp-article-gaps/delivery.md

---
type: bee.delivery
title: slp-article-gaps — delivery
description: "Delivery record proposed by bee knowledge promote for work item slp-article-gaps: 2 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-11-04
bee:
  id: slp-article-gaps-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/slp-article-gaps/CONTEXT.md]
  sources: [docs/history/slp-article-gaps/CONTEXT.md, .bee/cells/sag-1.json, .bee/cells/sag-2.json]
---

# slp-article-gaps — Delivery

## What shipped

- **sag-1** — bee report outcomes prints per-layer result-change counts from the main cell ledger (499d1576e) (5 file(s) changed)
- **sag-2** — Worker brief names Goal, Hard constraints, and Open choices apart; must_haves.open_choices documented (8 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sag-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- report_outcomes registry` — 38+3 tests incl. granted-worktree fixture watched red first; also ran the built binary from the granted worktree (read 1199 main-ledger cells incl. sag-1/sag-2); full declared suite 5049 passed, 1 re…
- **sag-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- drivers prompts && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pointer_integrity --test instruction_laws --test skill_triggers && .bee/bin/bee dev release-manifest --check` — the cell verify in full: 465 drivers/prompts unit tests incl. native_cell_prompt_render_is_byte_identical_to_golden and herding_cell_dispatch_stdin_contains_no_bee_commands confirmed by name, pointer…

## Deviations

- **sag-1** — Leader finished round 3 inline: ledger_root uses StoreRoots::main_root, granted-worktree test, rework counts reopened_at — two worker rounds both read the store root instead of the cell ledger; a one-line premise fix did not earn a third dispatch — found a better route
- **sag-1** — Raised COMMAND_SURFACE_BUDGET_CHARS in hooks/session_preamble/budget.rs from 2604 to 2614 — the new verb adds 10 chars to the command surface the budget pins — something else had to be fixed first
- **sag-2** — The bee cells dissent sentence sits inside a native_bookkeeping block in worker-cell.md, so herding workers get Goal/Hard constraints/Open choices without the command — the herding stdin test forbids any bee command and herding dissent routing is out of scope (p-05d2a4f4) — hit an unforeseen obstacle
- **sag-2** — Committed regenerated plugin copies, .bee/bin/prompts/worker-cell.md and .bee/onboarding.json beyond the cell files list, reserved under w-sag-2 after regen wrote them — the action orders regen outputs in the same commit — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work slp-article-gaps` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/slp-article-gaps/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "slp-article-gaps" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-04T03:15:51.982Z), the work item declares no bee.areas.

area workflow-state:
  - [sag-1] bee report outcomes prints per-layer result-change counts from the main cell ledger (499d1576e) — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/sag-1.json)
  - [sag-2] Worker brief names Goal, Hard constraints, and Open choices apart; must_haves.open_choices documented — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/sag-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell sag-1 — save as docs/knowledge/patterns/slp-article-gaps-sag-1-pitfall.md

---
type: bee.pattern
title: slp-article-gaps cell sag-1 — pitfall candidate
description: "Pitfall candidate mined from cell sag-1's capped trace: Leader finished round 3 inline: ledger_root uses StoreRoots::main_root, granted-worktree test, rework counts reopened_at — two worker rounds both read the stor…"
timestamp: 2026-11-04
bee:
  id: slp-article-gaps-sag-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/sag-1.json]
  polarity: pitfall
---

# slp-article-gaps cell sag-1 — pitfall candidate

## What the cell did

bee report outcomes prints per-layer result-change counts from the main cell ledger (499d1576e)

## Recorded evidence (verbatim from .bee/cells/sag-1.json)

- **deviation** — Leader finished round 3 inline: ledger_root uses StoreRoots::main_root, granted-worktree test, rework counts reopened_at — two worker rounds both read the store root instead of the cell ledger; a one-line premise fix did not earn a third dispatch — found a better route
- **deviation** — Raised COMMAND_SURFACE_BUDGET_CHARS in hooks/session_preamble/budget.rs from 2604 to 2614 — the new verb adds 10 chars to the command surface the budget pins — something else had to be fixed first
- **failure_signature** — ea09d6b4e83b

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sag-2 — save as docs/knowledge/patterns/slp-article-gaps-sag-2-pitfall.md

---
type: bee.pattern
title: slp-article-gaps cell sag-2 — pitfall candidate
description: "Pitfall candidate mined from cell sag-2's capped trace: The bee cells dissent sentence sits inside a native_bookkeeping block in worker-cell.md, so herding workers get Goal/Hard constraints/Open choices without the …"
timestamp: 2026-11-04
bee:
  id: slp-article-gaps-sag-2-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/sag-2.json]
  polarity: pitfall
---

# slp-article-gaps cell sag-2 — pitfall candidate

## What the cell did

Worker brief names Goal, Hard constraints, and Open choices apart; must_haves.open_choices documented

## Recorded evidence (verbatim from .bee/cells/sag-2.json)

- **deviation** — The bee cells dissent sentence sits inside a native_bookkeeping block in worker-cell.md, so herding workers get Goal/Hard constraints/Open choices without the command — the herding stdin test forbids any bee command and herding dissent routing is out of scope (p-05d2a4f4) — hit an unforeseen obstacle
- **deviation** — Committed regenerated plugin copies, .bee/bin/prompts/worker-cell.md and .bee/onboarding.json beyond the cell files list, reserved under w-sag-2 after regen wrote them — the action orders regen outputs in the same commit — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 2 pattern candidate(s), 0 file(s) written.