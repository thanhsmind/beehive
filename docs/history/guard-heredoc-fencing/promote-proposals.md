promote proposal for work item "guard-heredoc-fencing" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): ghf-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/guard-heredoc-fencing/delivery.md

---
type: bee.delivery
title: guard-heredoc-fencing — delivery
description: "Delivery record proposed by bee knowledge promote for work item guard-heredoc-fencing: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: guard-heredoc-fencing-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/ghf-1.json]
---

# guard-heredoc-fencing — Delivery

## What shipped

- **ghf-1** — fence_heredocs wired before tokenize_deep: <<//<<- with quoted/unquoted/dash terminators, multi-heredoc per line, unterminated fails safe, <<< untouched; body content can never become a guard target (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ghf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work guard-heredoc-fencing` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "guard-heredoc-fencing" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-11T03:58:44.005Z), the work item declares no bee.areas.

area hook-runtime:
  - [ghf-1] fence_heredocs wired before tokenize_deep: <<//<<- with quoted/unquoted/dash terminators, multi-heredoc per line, unterminated fails safe, <<< untouched; body content can never become a guard target — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/ghf-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.