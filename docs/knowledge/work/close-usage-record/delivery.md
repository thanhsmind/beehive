---
type: bee.delivery
title: close-usage-record — delivery
description: "Delivery record for work item close-usage-record: 1 capped cell(s), 5 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: close-usage-record-delivery
  lifecycle: active
  required_context: [.bee/lanes/close-usage-record.json]
  sources: [.bee/lanes/close-usage-record.json, .bee/cells/cur-usage-json-record.json]
---

# close-usage-record — Delivery

## What shipped

- **cur-usage-json-record** — Green close writes `docs/history/<feature>/usage.json` with per-session detail and feature totals; the close letter carries the usage line (2 file(s) changed)

## Verify

- **cur-usage-json-record** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml usage` — green.

## Deviations

- Wrote the record with `write_json_atomic` instead of `write_text_atomic` — same tmp-then-rename write plus the repo pretty-printer, and the payload is JSON.
- Replaced `CloseUsage.sessions` (a stored count) with a derived `sessions()` over the details list — two homes for one number is how a line and a file start disagreeing.
- Put the usage line in its own "Token usage" letter section rather than under "Usage" — that section lists docs a human opens to use the thing, a token count is a different question.
- Loosened `clean_store_green_close_reports_reason_clean` from "porcelain is empty" to "nothing under `.bee` is dirty, only `docs/` is untracked" — a green close now always leaves `usage.json`.
- Two end-to-end close tests assert `skipped >= 1` and `starts_with` rather than exact values, since `close_handler` reads the ambient `BEE_SESSION_ID`.

## Provenance

Mined from 1 capped cell trace in `.bee/cells/`. No `bee.areas` declared — no area sync performed.
