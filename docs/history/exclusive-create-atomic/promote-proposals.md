promote proposal for work item "exclusive-create-atomic" (docs/history/exclusive-create-atomic/CONTEXT.md) — 1 capped cell(s): eca-1
anchor: history — docs/history/exclusive-create-atomic/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/exclusive-create-atomic/delivery.md

---
type: bee.delivery
title: exclusive-create-atomic — delivery
description: "Delivery record proposed by bee knowledge promote for work item exclusive-create-atomic: 1 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-08-25
bee:
  id: exclusive-create-atomic-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/exclusive-create-atomic/CONTEXT.md]
  sources: [docs/history/exclusive-create-atomic/CONTEXT.md, .bee/cells/eca-1.json]
---

# exclusive-create-atomic — Delivery

## What shipped

- **eca-1** — Both the claim record and the path-lease record now publish by link(2) from a fully-written temp, so a loser never reads a half-written winner (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **eca-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test concurrency`

## Deviations

- **eca-1** — Added a same-file helper publish_exclusive/publish_tmp_path in reserve.rs rather than fsutil.rs: the natural home (fsutil tmp_path_for) is outside the cell files, so the helper lives in reserve.rs and claims.rs calls it as rsv::publish_exclusive.
- **eca-1** — publish_exclusive refuses an existing target before writing its temp. Without it the unit test reserve_bounded_retry_reports_conflict_when_takeover_is_blocked went red: it makes the leases directory 0o555, where the temp write fails EACCES instead of the EEXIST O_EXCL used to return. The pre-check restores O_EXCL ordering and opens no window - a target appearing after the check is still refused by link with AlreadyExists.
- **eca-1** — Pre-existing unrelated flake found: concurrency test store_lock_survives_a_pre_seeded_stale_lock_without_wedging_or_double_entry loses a state.json worker entry under the stale-lock steal. Measured 5/40 failures on the UNMODIFIED base vs 2/28 with this change - it is not caused by this cell and is not in its files.
- **eca-1** — bee reservations reserve reported a CONFLICT for both paths naming wk-eca-1 / eca-1 itself - my own pre-existing reservation, not a foreign hold, so I proceeded.
- **eca-1** — The cap first refused with no registered execution worker; I registered the dispatch fact I actually am with bee state worker add --nickname wk-eca-1 --cell eca-1 --tier generation --status running rather than recording a false --inline-reason.
- **eca-1** — Capped with --sync-ack: internal write mechanism only, no skill-visible behavior and skill files are outside this cell scope.
- **eca-1** — sync-ack: Internal write mechanism only: the two store records now publish by link(2) instead of create-then-write. Every CLI surface, refusal string, exit code and record shape is byte-identical, and no bee-planning/bee-swarming/bee-reviewing/bee-capturing text describes how a claim or lease file reaches disk. Skill files are also outside this cell's declared files.

## Provenance

Proposed by `bee knowledge promote --work exclusive-create-atomic` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/exclusive-create-atomic/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "exclusive-create-atomic" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-25T11:19:56.852Z), the work item declares no bee.areas.

area workflow-state:
  - [eca-1] Both the claim record and the path-lease record now publish by link(2) from a fully-written temp, so a loser never reads a half-written winner — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/eca-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell eca-1 — save as docs/knowledge/patterns/exclusive-create-atomic-eca-1-pitfall.md

---
type: bee.pattern
title: exclusive-create-atomic cell eca-1 — pitfall candidate
description: "Pitfall candidate mined from cell eca-1's capped trace: Added a same-file helper publish_exclusive/publish_tmp_path in reserve.rs rather than fsutil.rs: the natural home (fsutil tmp_path_for) is outside the cell fil…"
timestamp: 2026-08-25
bee:
  id: exclusive-create-atomic-eca-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/eca-1.json]
  polarity: pitfall
---

# exclusive-create-atomic cell eca-1 — pitfall candidate

## What the cell did

Both the claim record and the path-lease record now publish by link(2) from a fully-written temp, so a loser never reads a half-written winner

## Recorded evidence (verbatim from .bee/cells/eca-1.json)

- **deviation** — Added a same-file helper publish_exclusive/publish_tmp_path in reserve.rs rather than fsutil.rs: the natural home (fsutil tmp_path_for) is outside the cell files, so the helper lives in reserve.rs and claims.rs calls it as rsv::publish_exclusive.
- **deviation** — publish_exclusive refuses an existing target before writing its temp. Without it the unit test reserve_bounded_retry_reports_conflict_when_takeover_is_blocked went red: it makes the leases directory 0o555, where the temp write fails EACCES instead of the EEXIST O_EXCL used to return. The pre-check restores O_EXCL ordering and opens no window - a target appearing after the check is still refused by link with AlreadyExists.
- **deviation** — Pre-existing unrelated flake found: concurrency test store_lock_survives_a_pre_seeded_stale_lock_without_wedging_or_double_entry loses a state.json worker entry under the stale-lock steal. Measured 5/40 failures on the UNMODIFIED base vs 2/28 with this change - it is not caused by this cell and is not in its files.
- **deviation** — bee reservations reserve reported a CONFLICT for both paths naming wk-eca-1 / eca-1 itself - my own pre-existing reservation, not a foreign hold, so I proceeded.
- **deviation** — The cap first refused with no registered execution worker; I registered the dispatch fact I actually am with bee state worker add --nickname wk-eca-1 --cell eca-1 --tier generation --status running rather than recording a false --inline-reason.
- **deviation** — Capped with --sync-ack: internal write mechanism only, no skill-visible behavior and skill files are outside this cell scope.
- **deviation** — sync-ack: Internal write mechanism only: the two store records now publish by link(2) instead of create-then-write. Every CLI surface, refusal string, exit code and record shape is byte-identical, and no bee-planning/bee-swarming/bee-reviewing/bee-capturing text describes how a claim or lease file reaches disk. Skill files are also outside this cell's declared files.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.