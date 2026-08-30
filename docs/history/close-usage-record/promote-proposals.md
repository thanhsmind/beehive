promote proposal for work item "close-usage-record" (.bee/lanes/close-usage-record.json) — 1 capped cell(s): cur-usage-json-record
anchor: ledger — .bee/lanes/close-usage-record.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/close-usage-record/delivery.md

---
type: bee.delivery
title: close-usage-record — delivery
description: "Delivery record proposed by bee knowledge promote for work item close-usage-record: 1 capped cell(s), 5 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: close-usage-record-delivery
  lifecycle: active
  required_context: [.bee/lanes/close-usage-record.json]
  sources: [.bee/lanes/close-usage-record.json, .bee/cells/cur-usage-json-record.json]
---

# close-usage-record — Delivery

## What shipped

- **cur-usage-json-record** — Green close writes docs/history/<feature>/usage.json with per-session detail and feature totals; the close letter carries the usage line (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cur-usage-json-record** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml usage`

## Deviations

- **cur-usage-json-record** — Wrote the record with write_json_atomic instead of write_text_atomic — it is the same tmp-then-rename write plus the repo pretty-printer, and the payload is JSON — found a better route
- **cur-usage-json-record** — Replaced CloseUsage.sessions (a stored count) with a derived sessions() over the new details list — two homes for one number is how a line and a file start disagreeing — found a better route
- **cur-usage-json-record** — Put the usage line in its own Token usage letter section rather than under Usage — that section lists the docs a human opens to use the thing, and a token count is a different question — found a better route
- **cur-usage-json-record** — Loosened clean_store_green_close_reports_reason_clean from "porcelain is empty" to "nothing under .bee is dirty, only docs/ is untracked" — a green close now always leaves usage.json — something else had to be fixed first
- **cur-usage-json-record** — Two end-to-end close tests assert skipped >= 1 and starts_with rather than exact values — close_handler reads the ambient BEE_SESSION_ID, so an exact count would depend on the runner environment — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work close-usage-record` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/close-usage-record.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell cur-usage-json-record — save as docs/knowledge/patterns/close-usage-record-cur-usage-json-record-pitfall.md

---
type: bee.pattern
title: close-usage-record cell cur-usage-json-record — pitfall candidate
description: "Pitfall candidate mined from cell cur-usage-json-record's capped trace: Wrote the record with write_json_atomic instead of write_text_atomic — it is the same tmp-then-rename write plus the repo pretty-printer, and the payload is JS…"
timestamp: 2026-08-30
bee:
  id: close-usage-record-cur-usage-json-record-pitfall
  lifecycle: draft
  sources: [.bee/cells/cur-usage-json-record.json]
  polarity: pitfall
---

# close-usage-record cell cur-usage-json-record — pitfall candidate

## What the cell did

Green close writes docs/history/<feature>/usage.json with per-session detail and feature totals; the close letter carries the usage line

## Recorded evidence (verbatim from .bee/cells/cur-usage-json-record.json)

- **deviation** — Wrote the record with write_json_atomic instead of write_text_atomic — it is the same tmp-then-rename write plus the repo pretty-printer, and the payload is JSON — found a better route
- **deviation** — Replaced CloseUsage.sessions (a stored count) with a derived sessions() over the new details list — two homes for one number is how a line and a file start disagreeing — found a better route
- **deviation** — Put the usage line in its own Token usage letter section rather than under Usage — that section lists the docs a human opens to use the thing, and a token count is a different question — found a better route
- **deviation** — Loosened clean_store_green_close_reports_reason_clean from "porcelain is empty" to "nothing under .bee is dirty, only docs/ is untracked" — a green close now always leaves usage.json — something else had to be fixed first
- **deviation** — Two end-to-end close tests assert skipped >= 1 and starts_with rather than exact values — close_handler reads the ambient BEE_SESSION_ID, so an exact count would depend on the runner environment — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.