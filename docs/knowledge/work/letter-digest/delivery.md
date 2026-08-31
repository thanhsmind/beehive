---
type: bee.delivery
title: letter-digest — delivery
description: "Delivery record for work item letter-digest: 3 capped cell(s), 6 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: letter-digest-delivery
  lifecycle: active
  areas: [human-mailbox]
  required_context: [docs/history/letter-digest/CONTEXT.md, docs/history/letter-digest/plan.md]
  sources: [docs/history/letter-digest/CONTEXT.md, docs/history/letter-digest/plan.md, .bee/cells/ld-1.json, .bee/cells/ld-2.json, .bee/cells/ld-3.json]
---

# letter-digest — Delivery

## What shipped

- **ld-1** — Every `bee close` now files its close letter at the moment of close; an existing letter re-composes unarmed, and a close-lettered run that goes silent is recovered (2 file(s) changed)
- **ld-2** — Compose daily and weekly digest files from letters and usage records (3 file(s) changed)
- **ld-3** — `bee work set` folds due digests at the control root and the weekly fold logs deduped lesson decisions (3 file(s) changed)

## Verify

- **ld-1/ld-2/ld-3** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml mailbox` — green.

## Deviations

- **ld-1** — followed the plan.
- **ld-2** — Periods compose ordered by the date they ENDED, not started — a week starts before the days inside it, so a start-ordered pass wrote the week digest before the Tuesday digest it contains.
- **ld-2** — The digest frontmatter carries an `unreadable[]` list beside `letters[]`, so a torn letter is skipped but not vanished from the record.
- **ld-3** — Mined the departure list from each letter's STORED items rather than re-parsing the rendered departures section, so the miner can never disagree with the letter.
- **ld-3** — Dropped ld-2's module-wide `#![allow(dead_code)]` now that the door is wired, narrowed to the two struct fields still read only by tests.
- **ld-3** — sync-ack: no skill surface moves (internal library behaviour only); the cell declares `affects_skills: []`.

## Provenance

Mined from 3 capped cell traces in `.bee/cells/` and `docs/history/letter-digest/CONTEXT.md`, `docs/history/letter-digest/plan.md`. Already fully reflected in `docs/knowledge/areas/human-mailbox/overview.md` (LD1-LD4 decisions cited, digest/lesson data-dictionary entries present, Business Rules #13/#15 cover this feature's behavior) — this record adds no further area edit.
