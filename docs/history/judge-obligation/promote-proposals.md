promote proposal for work item "judge-obligation" (.bee/logs/scribing-runs.jsonl + .bee/lanes/judge-obligation.json) — 2 capped cell(s): jo-1, jo-2
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/judge-obligation.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/judge-obligation/delivery.md

---
type: bee.delivery
title: judge-obligation — delivery
description: "Delivery record proposed by bee knowledge promote for work item judge-obligation: 2 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-12
bee:
  id: judge-obligation-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/judge-obligation.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/judge-obligation.json, .bee/cells/jo-1.json, .bee/cells/jo-2.json]
---

# judge-obligation — Delivery

## What shipped

- **jo-1** — Added authoring-time JUDGE_OBLIGATION beside REGEN_OBLIGATION in obligation.rs: cells add refuses a cell whose files touch a judge-required guard root at lane tiny/small/spike/docs, unless the cell's lane is standard/high-risk (already covered by close.rs's judge-debt door) or it carries judge_obligation_ack with a one-line reason. Judge-required roots (JUDGE_REQUIRED_ROOTS) are pinned both ways against the crate source tree by two tests, proved red by temporarily adding a fake guard dir outside the roots then green again after removing it. Synced the authoring spec (B52/R103 + pointer). Not wired into validate_new_cell (validate.rs) since that file is outside this cell's declared files; that one-line follow-up call remains open. (3 file(s) changed)
- **jo-2** — Wired assert_judge_obligation into validate_new_cell (beside assert_regen_obligation) and proved it end to end through the real cells add CLI door: refusal names both escapes and writes nothing, lane standard is accepted, the ack is recorded on the stored cell, and a tripped cell refuses the whole batch. Raised pattern-20260812 to evidence: wired. (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **jo-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **jo-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **jo-1** — The 'reservations reserve' CLI call hit the documented known cross-worktree-hold defect (self-referential hold naming this same cell/session); per contract did not retry, and proceeded using the reservation leases already granted at claim time.

## Provenance

Proposed by `bee knowledge promote --work judge-obligation` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/judge-obligation.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "judge-obligation" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-12T04:34:15.609Z), the work item declares no bee.areas.

area workflow-state:
  - [jo-1] Added authoring-time JUDGE_OBLIGATION beside REGEN_OBLIGATION in obligation.rs: cells add refuses a cell whose files touch a judge-required guard root at lane tiny/small/spike/docs, unless the cell's lane is standard/high-risk (already covered by close.rs's judge-debt door) or it carries judge_obligation_ack with a one-line reason. Judge-required roots (JUDGE_REQUIRED_ROOTS) are pinned both ways against the crate source tree by two tests, proved red by temporarily adding a fake guard dir outside the roots then green again after removing it. Synced the authoring spec (B52/R103 + pointer). Not wired into validate_new_cell (validate.rs) since that file is outside this cell's declared files; that one-line follow-up call remains open. — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/jo-1.json)
  - [jo-2] Wired assert_judge_obligation into validate_new_cell (beside assert_regen_obligation) and proved it end to end through the real cells add CLI door: refusal names both escapes and writes nothing, lane standard is accepted, the ack is recorded on the stored cell, and a tripped cell refuses the whole batch. Raised pattern-20260812 to evidence: wired. — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/jo-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell jo-1 — save as docs/knowledge/patterns/judge-obligation-jo-1-pitfall.md

---
type: bee.pattern
title: judge-obligation cell jo-1 — pitfall candidate
description: "Pitfall candidate mined from cell jo-1's capped trace: The 'reservations reserve' CLI call hit the documented known cross-worktree-hold defect (self-referential hold naming this same cell/session); per contract did…"
timestamp: 2026-08-12
bee:
  id: judge-obligation-jo-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/jo-1.json]
  polarity: pitfall
---

# judge-obligation cell jo-1 — pitfall candidate

## What the cell did

Added authoring-time JUDGE_OBLIGATION beside REGEN_OBLIGATION in obligation.rs: cells add refuses a cell whose files touch a judge-required guard root at lane tiny/small/spike/docs, unless the cell's lane is standard/high-risk (already covered by close.rs's judge-debt door) or it carries judge_obligation_ack with a one-line reason. Judge-required roots (JUDGE_REQUIRED_ROOTS) are pinned both ways against the crate source tree by two tests, proved red by temporarily adding a fake guard dir outside the roots then green again after removing it. Synced the authoring spec (B52/R103 + pointer). Not wired into validate_new_cell (validate.rs) since that file is outside this cell's declared files; that one-line follow-up call remains open.

## Recorded evidence (verbatim from .bee/cells/jo-1.json)

- **deviation** — The 'reservations reserve' CLI call hit the documented known cross-worktree-hold defect (self-referential hold naming this same cell/session); per contract did not retry, and proceeded using the reservation leases already granted at claim time.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 1 pattern candidate(s), 0 file(s) written.