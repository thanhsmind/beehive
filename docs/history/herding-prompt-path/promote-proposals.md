promote proposal for work item "herding-prompt-path" (.bee/logs/scribing-runs.jsonl + .bee/lanes/herding-prompt-path.json + docs/history/herding-prompt-path/promote-proposals.md) — 1 capped cell(s): hpp-1
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/herding-prompt-path.json, docs/history/herding-prompt-path/promote-proposals.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-prompt-path/delivery.md

---
type: bee.delivery
title: herding-prompt-path — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-prompt-path: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: herding-prompt-path-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/herding-prompt-path.json, docs/history/herding-prompt-path/promote-proposals.md]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/herding-prompt-path.json, docs/history/herding-prompt-path/promote-proposals.md, .bee/cells/archive/herding-prompt-path/hpp-1.json]
---

# herding-prompt-path — Delivery

## What shipped

- **hpp-1** — read_prompt_file searches five skill roots in order, skills/ first, and names every path tried on failure (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hpp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml control_loop`

## Deviations

- **hpp-1** — followed the plan
- **hpp-1** — sync-ack: Cell scope is one file and declares affects_skills: []. The bee-herding skill text is unchanged by design - the prompt still lives at bee-herding/references/<role>-prompt.md, only the root bee searches for that tree widens from skills/ to the installed runtime prefixes. No documented herding contract moved; the cell routes the doc side to docs/knowledge/areas/bee-herding/overview.md via affects_specs, which is the scribe's file, not this worker's.

## Provenance

Proposed by `bee knowledge promote --work herding-prompt-path` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/herding-prompt-path.json`, `docs/history/herding-prompt-path/promote-proposals.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-prompt-path" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-30T06:10:30.938Z), the work item declares no bee.areas.

area bee-herding:
  - [hpp-1] read_prompt_file searches five skill roots in order, skills/ first, and names every path tried on failure — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/herding-prompt-path/hpp-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hpp-1 — save as docs/knowledge/patterns/herding-prompt-path-hpp-1-pitfall.md

---
type: bee.pattern
title: herding-prompt-path cell hpp-1 — pitfall candidate
description: "Pitfall candidate mined from cell hpp-1's capped trace: followed the plan"
timestamp: 2026-08-30
bee:
  id: herding-prompt-path-hpp-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/archive/herding-prompt-path/hpp-1.json]
  polarity: pitfall
---

# herding-prompt-path cell hpp-1 — pitfall candidate

## What the cell did

read_prompt_file searches five skill roots in order, skills/ first, and names every path tried on failure

## Recorded evidence (verbatim from .bee/cells/archive/herding-prompt-path/hpp-1.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: Cell scope is one file and declares affects_skills: []. The bee-herding skill text is unchanged by design - the prompt still lives at bee-herding/references/<role>-prompt.md, only the root bee searches for that tree widens from skills/ to the installed runtime prefixes. No documented herding contract moved; the cell routes the doc side to docs/knowledge/areas/bee-herding/overview.md via affects_specs, which is the scribe's file, not this worker's.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.