promote proposal for work item "letter-digest" (docs/history/letter-digest/CONTEXT.md + docs/history/letter-digest/plan.md) — 3 capped cell(s): ld-1, ld-2, ld-3
anchor: history — docs/history/letter-digest/CONTEXT.md, docs/history/letter-digest/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/letter-digest/delivery.md

---
type: bee.delivery
title: letter-digest — delivery
description: "Delivery record proposed by bee knowledge promote for work item letter-digest: 3 capped cell(s), 6 recorded deviation(s)."
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

- **ld-1** — Every bee close now files its close letter at the moment of close; an existing letter re-composes unarmed, and a close-lettered run that goes silent is recovered (2 file(s) changed)
- **ld-2** — Compose daily and weekly digest files from letters and usage records (3 file(s) changed)
- **ld-3** — bee work set folds due digests at the control root and the weekly fold logs deduped lesson decisions (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ld-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml mailbox`
- **ld-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml mailbox`
- **ld-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml mailbox`

## Deviations

- **ld-1** — followed the plan
- **ld-2** — Periods compose ordered by the date they ENDED, not the date they started — a week starts before the days inside it, so a start-ordered pass wrote the week digest before the Tuesday digest it contains — found a better route
- **ld-2** — The digest frontmatter carries an unreadable[] list beside letters[] — the plan only said a torn letter is skipped with a stderr warning, which would let a torn letter vanish from the record with nothing kept — something else had to be fixed first
- **ld-3** — Mined the departure list from each letter STORED items rather than parsing the rendered departures section back out of the body — the items carry what/why/kind apart, so the miner never has to re-derive a kind from prose and can never disagree with the letter — found a better route
- **ld-3** — Dropped ld-2 module-wide #![allow(dead_code)] now that the door is wired, and narrowed it to the two struct fields still read only by tests — a module-wide allow would hide the next real dead path — something else had to be fixed first
- **ld-3** — sync-ack: No skill surface moves: this is internal library behaviour on the bee work set path (a digest file appears in .bee/human-mailbox, a lesson row appears in .bee/decisions.jsonl). No verb, flag, or agent procedure changed, and the cell declares affects_skills: [].

## Provenance

Proposed by `bee knowledge promote --work letter-digest` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/letter-digest/CONTEXT.md`, `docs/history/letter-digest/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "letter-digest" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-30T15:06:44.482Z), the work item declares no bee.areas.

area human-mailbox:
  - [ld-1] Every bee close now files its close letter at the moment of close; an existing letter re-composes unarmed, and a close-lettered run that goes silent is recovered — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/ld-1.json)
  - [ld-2] Compose daily and weekly digest files from letters and usage records — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/ld-2.json)
  - [ld-3] bee work set folds due digests at the control root and the weekly fold logs deduped lesson decisions — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/ld-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell ld-1 — save as docs/knowledge/patterns/letter-digest-ld-1-pitfall.md

---
type: bee.pattern
title: letter-digest cell ld-1 — pitfall candidate
description: "Pitfall candidate mined from cell ld-1's capped trace: followed the plan"
timestamp: 2026-08-30
bee:
  id: letter-digest-ld-1-pitfall
  lifecycle: draft
  areas: [human-mailbox]
  sources: [.bee/cells/ld-1.json]
  polarity: pitfall
---

# letter-digest cell ld-1 — pitfall candidate

## What the cell did

Every bee close now files its close letter at the moment of close; an existing letter re-composes unarmed, and a close-lettered run that goes silent is recovered

## Recorded evidence (verbatim from .bee/cells/ld-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ld-2 — save as docs/knowledge/patterns/letter-digest-ld-2-pitfall.md

---
type: bee.pattern
title: letter-digest cell ld-2 — pitfall candidate
description: "Pitfall candidate mined from cell ld-2's capped trace: Periods compose ordered by the date they ENDED, not the date they started — a week starts before the days inside it, so a start-ordered pass wrote the week dig…"
timestamp: 2026-08-30
bee:
  id: letter-digest-ld-2-pitfall
  lifecycle: draft
  areas: [human-mailbox]
  sources: [.bee/cells/ld-2.json]
  polarity: pitfall
---

# letter-digest cell ld-2 — pitfall candidate

## What the cell did

Compose daily and weekly digest files from letters and usage records

## Recorded evidence (verbatim from .bee/cells/ld-2.json)

- **deviation** — Periods compose ordered by the date they ENDED, not the date they started — a week starts before the days inside it, so a start-ordered pass wrote the week digest before the Tuesday digest it contains — found a better route
- **deviation** — The digest frontmatter carries an unreadable[] list beside letters[] — the plan only said a torn letter is skipped with a stderr warning, which would let a torn letter vanish from the record with nothing kept — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ld-3 — save as docs/knowledge/patterns/letter-digest-ld-3-pitfall.md

---
type: bee.pattern
title: letter-digest cell ld-3 — pitfall candidate
description: "Pitfall candidate mined from cell ld-3's capped trace: Mined the departure list from each letter STORED items rather than parsing the rendered departures section back out of the body — the items carry what/why/kind…"
timestamp: 2026-08-30
bee:
  id: letter-digest-ld-3-pitfall
  lifecycle: draft
  areas: [human-mailbox]
  sources: [.bee/cells/ld-3.json]
  polarity: pitfall
---

# letter-digest cell ld-3 — pitfall candidate

## What the cell did

bee work set folds due digests at the control root and the weekly fold logs deduped lesson decisions

## Recorded evidence (verbatim from .bee/cells/ld-3.json)

- **deviation** — Mined the departure list from each letter STORED items rather than parsing the rendered departures section back out of the body — the items carry what/why/kind apart, so the miner never has to re-derive a kind from prose and can never disagree with the letter — found a better route
- **deviation** — Dropped ld-2 module-wide #![allow(dead_code)] now that the door is wired, and narrowed it to the two struct fields still read only by tests — a module-wide allow would hide the next real dead path — something else had to be fixed first
- **deviation** — sync-ack: No skill surface moves: this is internal library behaviour on the bee work set path (a digest file appears in .bee/human-mailbox, a lesson row appears in .bee/decisions.jsonl). No verb, flag, or agent procedure changed, and the cell declares affects_skills: [].

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 3 area bullet(s), 3 pattern candidate(s), 0 file(s) written.