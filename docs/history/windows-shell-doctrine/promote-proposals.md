promote proposal for work item "windows-shell-doctrine" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): wsd-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/windows-shell-doctrine/delivery.md

---
type: bee.delivery
title: windows-shell-doctrine — delivery
description: "Delivery record proposed by bee knowledge promote for work item windows-shell-doctrine: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-06
bee:
  id: windows-shell-doctrine-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/wsd-1.json]
---

# windows-shell-doctrine — Delivery

## What shipped

- **wsd-1** — onboarding appends the Windows Environment section inside the managed AGENTS.md block when host_shell resolves to powershell; .bee/config.json wins over host detection (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wsd-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml onboard && .bee/bin/bee dev release-manifest --check`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work windows-shell-doctrine` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "windows-shell-doctrine" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-06T10:18:55.849Z), the work item declares no bee.areas.

area onboarding:
  - [wsd-1] onboarding appends the Windows Environment section inside the managed AGENTS.md block when host_shell resolves to powershell; .bee/config.json wins over host detection — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/wsd-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.