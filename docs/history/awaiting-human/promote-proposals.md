promote proposal for work item "awaiting-human" (docs/history/awaiting-human/CONTEXT.md + docs/history/awaiting-human/plan.md) — 4 capped cell(s): ah-1, ah-2, ah-3, ah-4
anchor: history — docs/history/awaiting-human/CONTEXT.md, docs/history/awaiting-human/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/awaiting-human/delivery.md

---
type: bee.delivery
title: awaiting-human — delivery
description: "Delivery record proposed by bee knowledge promote for work item awaiting-human: 4 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-14
bee:
  id: awaiting-human-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/awaiting-human/CONTEXT.md, docs/history/awaiting-human/plan.md]
  sources: [docs/history/awaiting-human/CONTEXT.md, docs/history/awaiting-human/plan.md, .bee/cells/ah-1.json, .bee/cells/ah-2.json, .bee/cells/ah-3.json, .bee/cells/ah-4.json]
---

# awaiting-human — Delivery

## What shipped

- **ah-1** — A waiting mark exists, can be set, and makes the run read awaiting-approval (5 file(s) changed)
- **ah-2** — Three live ways for the waiting mark to end, with the hook path now covered by tests that drive the real hook entry point, including a failure injection proving the hook survives a failing clear (3 file(s) changed)
- **ah-3** — Six reporting surfaces enumerated and handled: five name a live wait, status --brief deliberately excluded per status-diet D1/D2; enumeration written to docs/history/awaiting-human/reports/ah-3-rework.md (7 file(s) changed)
- **ah-4** — Wired bee state waiting-on set/clear onto ah-1/ah-2's existing store functions, with D3 target resolution and projection sync (9 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ah-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ah-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ah-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ah-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **ah-4** — File list corrected: added waiting_on.rs (the new module), catalog.rs (PINNED_FLAG_COUNT bump for --subject), and tests/workflow_verbs.rs (true through-the-binary CLI proof) beyond the cell's guessed set.

## Provenance

Proposed by `bee knowledge promote --work awaiting-human` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/awaiting-human/CONTEXT.md`, `docs/history/awaiting-human/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "awaiting-human" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-14T15:50:45.902Z), the work item declares no bee.areas.

area workflow-state:
  - [ah-1] A waiting mark exists, can be set, and makes the run read awaiting-approval — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/ah-1.json)
  - [ah-2] Three live ways for the waiting mark to end, with the hook path now covered by tests that drive the real hook entry point, including a failure injection proving the hook survives a failing clear — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/ah-2.json)
  - [ah-3] Six reporting surfaces enumerated and handled: five name a live wait, status --brief deliberately excluded per status-diet D1/D2; enumeration written to docs/history/awaiting-human/reports/ah-3-rework.md — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/ah-3.json)
  - [ah-4] Wired bee state waiting-on set/clear onto ah-1/ah-2's existing store functions, with D3 target resolution and projection sync — feature-wide sync per the scribing stamp, 9 file(s) changed (trace .bee/cells/ah-4.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell ah-2 — save as docs/knowledge/patterns/awaiting-human-ah-2-pitfall.md

---
type: bee.pattern
title: awaiting-human cell ah-2 — pitfall candidate
description: "Pitfall candidate mined from cell ah-2's capped trace: D2 path ONE is unproven: rg over packages/bee-rs/crates/bee finds zero test references to clear_and_reap_waiting_on_best_effort (prompt_context.rs:338); 39face…"
timestamp: 2026-08-14
bee:
  id: awaiting-human-ah-2-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/ah-2.json]
  polarity: pitfall
---

# awaiting-human cell ah-2 — pitfall candidate

## What the cell did

Three live ways for the waiting mark to end, with the hook path now covered by tests that drive the real hook entry point, including a failure injection proving the hook survives a failing clear

## Recorded evidence (verbatim from .bee/cells/ah-2.json)

- **failure_signature** — D2 path ONE is unproven: rg over packages/bee-rs/crates/bee finds zero test references to clear_and_reap_waiting_on_best_effort (prompt_context.rs:338); 39facee4 added +70 hook lines and 0 hook tests, so neither 'the human's message clears the mark' nor 'a failing clear cannot break the turn' is covered by any test.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ah-3 — save as docs/knowledge/patterns/awaiting-human-ah-3-pitfall.md

---
type: bee.pattern
title: awaiting-human cell ah-3 — pitfall candidate
description: "Pitfall candidate mined from cell ah-3's capped trace: Census incomplete: hooks/compaction.rs build_compact_capsule reads .bee/state.json (:187) and renders phase (:1421) plus '- Gate pending: {gate}' (:1478) but n…"
timestamp: 2026-08-14
bee:
  id: awaiting-human-ah-3-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/ah-3.json]
  polarity: pitfall
---

# awaiting-human cell ah-3 — pitfall candidate

## What the cell did

Six reporting surfaces enumerated and handled: five name a live wait, status --brief deliberately excluded per status-diet D1/D2; enumeration written to docs/history/awaiting-human/reports/ah-3-rework.md

## Recorded evidence (verbatim from .bee/cells/ah-3.json)

- **failure_signature** — Census incomplete: hooks/compaction.rs build_compact_capsule reads .bee/state.json (:187) and renders phase (:1421) plus '- Gate pending: {gate}' (:1478) but never a live waiting_on mark, so the post-compaction surface stays silent about a human wait; it was neither covered nor reported, and no enumeration exists at all (trace.outcome null, no docs/history/awaiting-human/reports/). Secondary: bee status --brief (status_brief.rs:63-69) likewise reports phase+gates with no mark.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ah-4 — save as docs/knowledge/patterns/awaiting-human-ah-4-pitfall.md

---
type: bee.pattern
title: awaiting-human cell ah-4 — pitfall candidate
description: "Pitfall candidate mined from cell ah-4's capped trace: File list corrected: added waiting_on.rs (the new module), catalog.rs (PINNED_FLAG_COUNT bump for --subject), and tests/workflow_verbs.rs (true through-the-bin…"
timestamp: 2026-08-14
bee:
  id: awaiting-human-ah-4-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/ah-4.json]
  polarity: pitfall
---

# awaiting-human cell ah-4 — pitfall candidate

## What the cell did

Wired bee state waiting-on set/clear onto ah-1/ah-2's existing store functions, with D3 target resolution and projection sync

## Recorded evidence (verbatim from .bee/cells/ah-4.json)

- **deviation** — File list corrected: added waiting_on.rs (the new module), catalog.rs (PINNED_FLAG_COUNT bump for --subject), and tests/workflow_verbs.rs (true through-the-binary CLI proof) beyond the cell's guessed set.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 3 pattern candidate(s), 0 file(s) written.