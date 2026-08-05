promote proposal for work item "knowledge-in-flow" (.bee/logs/scribing-runs.jsonl + .bee/lanes/knowledge-in-flow.json) — 2 capped cell(s): kf-1, kf-2
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/knowledge-in-flow.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/knowledge-in-flow/delivery.md

---
type: bee.delivery
title: knowledge-in-flow — delivery
description: "Delivery record proposed by bee knowledge promote for work item knowledge-in-flow: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-05
bee:
  id: knowledge-in-flow-delivery
  lifecycle: active
  areas: [okf-profile, hook-runtime]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/knowledge-in-flow.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/knowledge-in-flow.json, .bee/cells/kf-1.json, .bee/cells/kf-2.json]
---

# knowledge-in-flow — Delivery

## What shipped

- **kf-1** — Invite every anchorable feature to load its knowledge context (3 file(s) changed)
- **kf-2** — Surfaced unapplied docs/history/<feature>/promote-proposals.md as a report-only line in bee orient (work.blockers) and the session preamble; applied-status reuses the existing best-scribing-stamp helper. Deviation: also touched hooks/session_preamble/budget.rs (the actual build_session_preamble call site, not listed in cell files) and verbs/status_full/tests.rs (new coverage) since the cell's file list predates the module split. (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **kf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **kf-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work knowledge-in-flow` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/knowledge-in-flow.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "knowledge-in-flow" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-05T08:39:38.140Z), the work item declares no bee.areas.

area okf-profile:
  - [kf-1] Invite every anchorable feature to load its knowledge context — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/kf-1.json)
  - [kf-2] Surfaced unapplied docs/history/<feature>/promote-proposals.md as a report-only line in bee orient (work.blockers) and the session preamble; applied-status reuses the existing best-scribing-stamp helper. Deviation: also touched hooks/session_preamble/budget.rs (the actual build_session_preamble call site, not listed in cell files) and verbs/status_full/tests.rs (new coverage) since the cell's file list predates the module split. — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/kf-2.json)

area hook-runtime:
  - [kf-1] Invite every anchorable feature to load its knowledge context — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/kf-1.json)
  - [kf-2] Surfaced unapplied docs/history/<feature>/promote-proposals.md as a report-only line in bee orient (work.blockers) and the session preamble; applied-status reuses the existing best-scribing-stamp helper. Deviation: also touched hooks/session_preamble/budget.rs (the actual build_session_preamble call site, not listed in cell files) and verbs/status_full/tests.rs (new coverage) since the cell's file list predates the module split. — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/kf-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 0 pattern candidate(s), 0 file(s) written.