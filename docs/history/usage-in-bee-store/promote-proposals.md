promote proposal for work item "usage-in-bee-store" (.bee/lanes/usage-in-bee-store.json) — 1 capped cell(s): uibs-move-usage-record
anchor: ledger — .bee/lanes/usage-in-bee-store.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/usage-in-bee-store/delivery.md

---
type: bee.delivery
title: usage-in-bee-store — delivery
description: "Delivery record proposed by bee knowledge promote for work item usage-in-bee-store: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: usage-in-bee-store-delivery
  lifecycle: active
  required_context: [.bee/lanes/usage-in-bee-store.json]
  sources: [.bee/lanes/usage-in-bee-store.json, .bee/cells/uibs-move-usage-record.json]
---

# usage-in-bee-store — Delivery

## What shipped

- **uibs-move-usage-record** — Close writes its token-usage record to .bee/usage/<feature>.json under the control root; close's own bookkeeping commit lands it (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **uibs-move-usage-record** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee usage`

## Deviations

- **uibs-move-usage-record** — Rewrote clean_store_green_close_reports_reason_clean as green_close_commits_the_usage_record_it_just_wrote — a green close now always writes .bee/usage/<feature>.json, so .bee is never clean at that point; the test now pins that the record is committed and asserts reason clean one step later — the plan was wrong about a fact
- **uibs-move-usage-record** — Widened green_close_commits_only_dirty_bee_paths_leaving_unrelated_dirt_uncommitted to expect .bee/usage/demo.json beside .bee/config.json — the record is a new .bee path the same path-scoped git add stages — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work usage-in-bee-store` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/usage-in-bee-store.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell uibs-move-usage-record — save as docs/knowledge/patterns/usage-in-bee-store-uibs-move-usage-record-pitfall.md

---
type: bee.pattern
title: usage-in-bee-store cell uibs-move-usage-record — pitfall candidate
description: "Pitfall candidate mined from cell uibs-move-usage-record's capped trace: Rewrote clean_store_green_close_reports_reason_clean as green_close_commits_the_usage_record_it_just_wrote — a green close now always writes .bee/usage/<featur…"
timestamp: 2026-08-30
bee:
  id: usage-in-bee-store-uibs-move-usage-record-pitfall
  lifecycle: draft
  sources: [.bee/cells/uibs-move-usage-record.json]
  polarity: pitfall
---

# usage-in-bee-store cell uibs-move-usage-record — pitfall candidate

## What the cell did

Close writes its token-usage record to .bee/usage/<feature>.json under the control root; close's own bookkeeping commit lands it

## Recorded evidence (verbatim from .bee/cells/uibs-move-usage-record.json)

- **deviation** — Rewrote clean_store_green_close_reports_reason_clean as green_close_commits_the_usage_record_it_just_wrote — a green close now always writes .bee/usage/<feature>.json, so .bee is never clean at that point; the test now pins that the record is committed and asserts reason clean one step later — the plan was wrong about a fact
- **deviation** — Widened green_close_commits_only_dirty_bee_paths_leaving_unrelated_dirt_uncommitted to expect .bee/usage/demo.json beside .bee/config.json — the record is a new .bee path the same path-scoped git add stages — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.