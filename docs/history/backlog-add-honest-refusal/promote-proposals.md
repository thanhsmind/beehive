promote proposal for work item "backlog-add-honest-refusal" (.bee/logs/scribing-runs.jsonl + .bee/lanes/backlog-add-honest-refusal.json) — 2 capped cell(s): bah-1, bah-2
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/backlog-add-honest-refusal.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/backlog-add-honest-refusal/delivery.md

---
type: bee.delivery
title: backlog-add-honest-refusal — delivery
description: "Delivery record proposed by bee knowledge promote for work item backlog-add-honest-refusal: 2 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-21
bee:
  id: backlog-add-honest-refusal-delivery
  lifecycle: active
  areas: [rust-runtime]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/backlog-add-honest-refusal.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/backlog-add-honest-refusal.json, .bee/cells/bah-1.json, .bee/cells/bah-2.json]
---

# backlog-add-honest-refusal — Delivery

## What shipped

- **bah-1** — bee backlog add now refuses by naming the missing/out-of-enum/over-length flag instead of the router's generic argument-shape line, and every refusal leaves .bee/backlog.jsonl byte-identical (1 file(s) changed)
- **bah-2** — backlog.add declares its four required flags: bee backlog add --help now stars --type, --title, --severity and --layer, and a source-derived contract test locks the declaration to run_add's own const (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **bah-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml backlog`
- **bah-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml registry`

## Deviations

- **bah-1** — Tests spawn a child test process (the cells.rs --exact/--ignored pattern) because run_add resolves its root from process cwd; the four pure-message cases stay in-process via a new add_refusal() helper. Also confirmed out of scope: run_propose, run_pbi_add, run_findings and add_pbi's PbiAdd::Delegate still carry the dead delegate-to-Node 'return None' pattern, and the registry entry for backlog.add declares required:[] which is why the router claimed the required arguments were all present.

## Provenance

Proposed by `bee knowledge promote --work backlog-add-honest-refusal` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/backlog-add-honest-refusal.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "backlog-add-honest-refusal" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-21T15:12:07.469Z), the work item declares no bee.areas.

area rust-runtime:
  - [bah-1] bee backlog add now refuses by naming the missing/out-of-enum/over-length flag instead of the router's generic argument-shape line, and every refusal leaves .bee/backlog.jsonl byte-identical — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/bah-1.json)
  - [bah-2] backlog.add declares its four required flags: bee backlog add --help now stars --type, --title, --severity and --layer, and a source-derived contract test locks the declaration to run_add's own const — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/bah-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell bah-1 — save as docs/knowledge/patterns/backlog-add-honest-refusal-bah-1-pitfall.md

---
type: bee.pattern
title: backlog-add-honest-refusal cell bah-1 — pitfall candidate
description: "Pitfall candidate mined from cell bah-1's capped trace: Tests spawn a child test process (the cells.rs --exact/--ignored pattern) because run_add resolves its root from process cwd; the four pure-message cases stay …"
timestamp: 2026-08-21
bee:
  id: backlog-add-honest-refusal-bah-1-pitfall
  lifecycle: draft
  areas: [rust-runtime]
  sources: [.bee/cells/bah-1.json]
  polarity: pitfall
---

# backlog-add-honest-refusal cell bah-1 — pitfall candidate

## What the cell did

bee backlog add now refuses by naming the missing/out-of-enum/over-length flag instead of the router's generic argument-shape line, and every refusal leaves .bee/backlog.jsonl byte-identical

## Recorded evidence (verbatim from .bee/cells/bah-1.json)

- **deviation** — Tests spawn a child test process (the cells.rs --exact/--ignored pattern) because run_add resolves its root from process cwd; the four pure-message cases stay in-process via a new add_refusal() helper. Also confirmed out of scope: run_propose, run_pbi_add, run_findings and add_pbi's PbiAdd::Delegate still carry the dead delegate-to-Node 'return None' pattern, and the registry entry for backlog.add declares required:[] which is why the router claimed the required arguments were all present.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 1 pattern candidate(s), 0 file(s) written.