promote proposal for work item "state-lock-lost-update" (docs/history/state-lock-lost-update/CONTEXT.md) — 1 capped cell(s): sll-1
anchor: history — docs/history/state-lock-lost-update/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/state-lock-lost-update/delivery.md

---
type: bee.delivery
title: state-lock-lost-update — delivery
description: "Delivery record proposed by bee knowledge promote for work item state-lock-lost-update: 1 capped cell(s), 5 recorded deviation(s)."
timestamp: 2026-08-25
bee:
  id: state-lock-lost-update-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/state-lock-lost-update/CONTEXT.md]
  sources: [docs/history/state-lock-lost-update/CONTEXT.md, .bee/cells/sll-1.json]
---

# state-lock-lost-update — Delivery

## What shipped

- **sll-1** — Serialized the stale-lock takeover on an O_EXCL claim keyed to the displaced acquisition; the TOCTOU that admitted a second LockGuard is closed (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sll-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test concurrency`

## Deviations

- **sll-1** — Full-file verify command is red on a PRE-EXISTING, unrelated failure: one_claimant_wins_the_cell_and_every_loser_is_a_typed_claimed_refusal (cells-claim JSON corruption in .bee/claims/, not the store lock). Measured 17/20 failures on the pristine base and 16/20 with this change, so it is neither caused nor worsened here. Full concurrency suite recorded: 18 passed, 1 failed (that one). Needs its own fix-first cell.
- **sll-1** — Fix landed in lock.rs only; workers.rs (also in the cell's files) needed no change and is byte-identical to HEAD.
- **sll-1** — Reservations for both files were already held under this cell and nickname when the worker started, so `reservations reserve` reported a self-conflict; treated as pre-reserved by the orchestrator, not a foreign hold.
- **sll-1** — The context's fact 1 ("the lost entry is w0, the FIRST racer") is contradicted by the captured run: the lost racer was w5. The lost racer is whichever won the takeover and was clobbered by the plain acquirer that took the rename vacancy. Context fact 2 (well-formed state.json) held.
- **sll-1** — Context's open question resolved: read_holder DID observe a freshly created, still-empty lock file (holder_after: null on a rename), so the create_new-then-write_all window at lock.rs:198 is reachable in practice via rename_for_takeover's post-rename read - just not via the staleness path.

## Provenance

Proposed by `bee knowledge promote --work state-lock-lost-update` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/state-lock-lost-update/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "state-lock-lost-update" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-25T11:20:49.469Z), the work item declares no bee.areas.

area workflow-state:
  - [sll-1] Serialized the stale-lock takeover on an O_EXCL claim keyed to the displaced acquisition; the TOCTOU that admitted a second LockGuard is closed — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/sll-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell sll-1 — save as docs/knowledge/patterns/state-lock-lost-update-sll-1-pitfall.md

---
type: bee.pattern
title: state-lock-lost-update cell sll-1 — pitfall candidate
description: "Pitfall candidate mined from cell sll-1's capped trace: Full-file verify command is red on a PRE-EXISTING, unrelated failure: one_claimant_wins_the_cell_and_every_loser_is_a_typed_claimed_refusal (cells-claim JSON c…"
timestamp: 2026-08-25
bee:
  id: state-lock-lost-update-sll-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/sll-1.json]
  polarity: pitfall
---

# state-lock-lost-update cell sll-1 — pitfall candidate

## What the cell did

Serialized the stale-lock takeover on an O_EXCL claim keyed to the displaced acquisition; the TOCTOU that admitted a second LockGuard is closed

## Recorded evidence (verbatim from .bee/cells/sll-1.json)

- **deviation** — Full-file verify command is red on a PRE-EXISTING, unrelated failure: one_claimant_wins_the_cell_and_every_loser_is_a_typed_claimed_refusal (cells-claim JSON corruption in .bee/claims/, not the store lock). Measured 17/20 failures on the pristine base and 16/20 with this change, so it is neither caused nor worsened here. Full concurrency suite recorded: 18 passed, 1 failed (that one). Needs its own fix-first cell.
- **deviation** — Fix landed in lock.rs only; workers.rs (also in the cell's files) needed no change and is byte-identical to HEAD.
- **deviation** — Reservations for both files were already held under this cell and nickname when the worker started, so `reservations reserve` reported a self-conflict; treated as pre-reserved by the orchestrator, not a foreign hold.
- **deviation** — The context's fact 1 ("the lost entry is w0, the FIRST racer") is contradicted by the captured run: the lost racer was w5. The lost racer is whichever won the takeover and was clobbered by the plain acquirer that took the rename vacancy. Context fact 2 (well-formed state.json) held.
- **deviation** — Context's open question resolved: read_holder DID observe a freshly created, still-empty lock file (holder_after: null on a rename), so the create_new-then-write_all window at lock.rs:198 is reachable in practice via rename_for_takeover's post-rename read - just not via the staleness path.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.