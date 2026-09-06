promote proposal for work item "herding-cockpit-completeness" (docs/history/herding-cockpit-completeness/CONTEXT.md + docs/history/herding-cockpit-completeness/plan.md) — 10 capped cell(s): hcc-1, hcc-2, hcc-3, hcc-4, hcc-5, hcc-6, hcc-7, hcc-8, hcc-9, hcc-10
anchor: history — docs/history/herding-cockpit-completeness/CONTEXT.md, docs/history/herding-cockpit-completeness/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-cockpit-completeness/delivery.md

---
type: bee.delivery
title: herding-cockpit-completeness — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-cockpit-completeness: 10 capped cell(s), 11 recorded deviation(s)."
timestamp: 2026-09-06
bee:
  id: herding-cockpit-completeness-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-cockpit-completeness/CONTEXT.md, docs/history/herding-cockpit-completeness/plan.md]
  sources: [docs/history/herding-cockpit-completeness/CONTEXT.md, docs/history/herding-cockpit-completeness/plan.md, .bee/cells/hcc-1.json, .bee/cells/hcc-2.json, .bee/cells/hcc-3.json, .bee/cells/hcc-4.json, .bee/cells/hcc-5.json, .bee/cells/hcc-6.json, .bee/cells/hcc-7.json, .bee/cells/hcc-8.json, .bee/cells/hcc-9.json, .bee/cells/hcc-10.json]
---

# herding-cockpit-completeness — Delivery

## What shipped

- **hcc-1** — Mark enum plus read/write/clear mark helpers on job.json (1 file(s) changed)
- **hcc-2** — Add pane_send_key to PaneTransport for herdr, tmux, and test doubles (3 file(s) changed)
- **hcc-3** — Interrupted and cancelled run outcomes from the job mark, continue refusal with FIX line, retryable envelope bit (1 file(s) changed)
- **hcc-4** — bee herding interrupt and cancel verbs with registry entries (4 file(s) changed)
- **hcc-5** — Add mark_orphans and transition_status helpers with unit tests (1 file(s) changed)
- **hcc-6** — Project stalled and recovered in run poll tick with progress lines (1 file(s) changed)
- **hcc-7** — Add jobs array to status and orphan sweep to status and occupancy (2 file(s) changed)
- **hcc-8** — Add git handoff block to done and blocked run envelopes (1 file(s) changed)
- **hcc-9** — Carry retryable bit on wave bucket rows and ledger worker rows (4 file(s) changed)
- **hcc-10** — Synced herding knowledge docs and config reference with cockpit signals (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hcc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::mailbox`
- **hcc-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::`
- **hcc-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run`
- **hcc-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::job_verbs && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts --test registry_dispatch`
- **hcc-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::mailbox`
- **hcc-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run`
- **hcc-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::`
- **hcc-8** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run`
- **hcc-9** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::wave`
- **hcc-10** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts`

## Deviations

- **hcc-1** — sync-ack: internal mailbox.rs helpers with no surface change; the bee-herding skill sync is hcc-10's docs cell after hcc-3..9 land
- **hcc-2** — sync-ack: skill docs update scheduled for slice 3 per plan.md
- **hcc-3** — sync-ack: run.rs behaviour change; the bee-herding skill and knowledge sync is hcc-10's docs cell after hcc-3..9 land
- **hcc-4** — sync-ack: new herding verbs; the bee-herding skill and knowledge sync is hcc-10's docs cell after hcc-3..9 land
- **hcc-5** — sync-ack: internal herding change; skill sync is hcc-10
- **hcc-6** — sync-ack: internal herding change; skill sync is hcc-10
- **hcc-7** — sync-ack: internal herding change; skill sync is hcc-10
- **hcc-8** — sync-ack: internal herding change; skill sync is hcc-10
- **hcc-9** — WorkerRow struct initialization in run.rs and control_loop.rs required retryable: None — field addition broke compilation — something else had to be fixed first
- **hcc-9** — sync-ack: internal herding change; skill sync is hcc-10
- **hcc-10** — sync-ack: this cell is the docs sync

## Provenance

Proposed by `bee knowledge promote --work herding-cockpit-completeness` from 10 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-cockpit-completeness/CONTEXT.md`, `docs/history/herding-cockpit-completeness/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-cockpit-completeness" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-06T11:27:47.612Z), the work item declares no bee.areas.

area bee-herding:
  - [hcc-3] Interrupted and cancelled run outcomes from the job mark, continue refusal with FIX line, retryable envelope bit — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/hcc-3.json)
  - [hcc-4] bee herding interrupt and cancel verbs with registry entries — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/hcc-4.json)
  - [hcc-6] Project stalled and recovered in run poll tick with progress lines — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/hcc-6.json)
  - [hcc-7] Add jobs array to status and orphan sweep to status and occupancy — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/hcc-7.json)
  - [hcc-8] Add git handoff block to done and blocked run envelopes — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/hcc-8.json)
  - [hcc-9] Carry retryable bit on wave bucket rows and ledger worker rows — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/hcc-9.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hcc-1 — save as docs/knowledge/patterns/herding-cockpit-completeness-hcc-1-pitfall.md

---
type: bee.pattern
title: herding-cockpit-completeness cell hcc-1 — pitfall candidate
description: "Pitfall candidate mined from cell hcc-1's capped trace: sync-ack: internal mailbox.rs helpers with no surface change; the bee-herding skill sync is hcc-10's docs cell after hcc-3..9 land"
timestamp: 2026-09-06
bee:
  id: herding-cockpit-completeness-hcc-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hcc-1.json]
  polarity: pitfall
---

# herding-cockpit-completeness cell hcc-1 — pitfall candidate

## What the cell did

Mark enum plus read/write/clear mark helpers on job.json

## Recorded evidence (verbatim from .bee/cells/hcc-1.json)

- **deviation** — sync-ack: internal mailbox.rs helpers with no surface change; the bee-herding skill sync is hcc-10's docs cell after hcc-3..9 land

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hcc-2 — save as docs/knowledge/patterns/herding-cockpit-completeness-hcc-2-pitfall.md

---
type: bee.pattern
title: herding-cockpit-completeness cell hcc-2 — pitfall candidate
description: "Pitfall candidate mined from cell hcc-2's capped trace: sync-ack: skill docs update scheduled for slice 3 per plan.md"
timestamp: 2026-09-06
bee:
  id: herding-cockpit-completeness-hcc-2-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hcc-2.json]
  polarity: pitfall
---

# herding-cockpit-completeness cell hcc-2 — pitfall candidate

## What the cell did

Add pane_send_key to PaneTransport for herdr, tmux, and test doubles

## Recorded evidence (verbatim from .bee/cells/hcc-2.json)

- **deviation** — sync-ack: skill docs update scheduled for slice 3 per plan.md

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hcc-3 — save as docs/knowledge/patterns/herding-cockpit-completeness-hcc-3-pitfall.md

---
type: bee.pattern
title: herding-cockpit-completeness cell hcc-3 — pitfall candidate
description: "Pitfall candidate mined from cell hcc-3's capped trace: sync-ack: run.rs behaviour change; the bee-herding skill and knowledge sync is hcc-10's docs cell after hcc-3..9 land"
timestamp: 2026-09-06
bee:
  id: herding-cockpit-completeness-hcc-3-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hcc-3.json]
  polarity: pitfall
---

# herding-cockpit-completeness cell hcc-3 — pitfall candidate

## What the cell did

Interrupted and cancelled run outcomes from the job mark, continue refusal with FIX line, retryable envelope bit

## Recorded evidence (verbatim from .bee/cells/hcc-3.json)

- **deviation** — sync-ack: run.rs behaviour change; the bee-herding skill and knowledge sync is hcc-10's docs cell after hcc-3..9 land

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hcc-4 — save as docs/knowledge/patterns/herding-cockpit-completeness-hcc-4-pitfall.md

---
type: bee.pattern
title: herding-cockpit-completeness cell hcc-4 — pitfall candidate
description: "Pitfall candidate mined from cell hcc-4's capped trace: sync-ack: new herding verbs; the bee-herding skill and knowledge sync is hcc-10's docs cell after hcc-3..9 land"
timestamp: 2026-09-06
bee:
  id: herding-cockpit-completeness-hcc-4-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hcc-4.json]
  polarity: pitfall
---

# herding-cockpit-completeness cell hcc-4 — pitfall candidate

## What the cell did

bee herding interrupt and cancel verbs with registry entries

## Recorded evidence (verbatim from .bee/cells/hcc-4.json)

- **deviation** — sync-ack: new herding verbs; the bee-herding skill and knowledge sync is hcc-10's docs cell after hcc-3..9 land

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hcc-5 — save as docs/knowledge/patterns/herding-cockpit-completeness-hcc-5-pitfall.md

---
type: bee.pattern
title: herding-cockpit-completeness cell hcc-5 — pitfall candidate
description: "Pitfall candidate mined from cell hcc-5's capped trace: sync-ack: internal herding change; skill sync is hcc-10"
timestamp: 2026-09-06
bee:
  id: herding-cockpit-completeness-hcc-5-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hcc-5.json]
  polarity: pitfall
---

# herding-cockpit-completeness cell hcc-5 — pitfall candidate

## What the cell did

Add mark_orphans and transition_status helpers with unit tests

## Recorded evidence (verbatim from .bee/cells/hcc-5.json)

- **deviation** — sync-ack: internal herding change; skill sync is hcc-10

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hcc-6 — save as docs/knowledge/patterns/herding-cockpit-completeness-hcc-6-pitfall.md

---
type: bee.pattern
title: herding-cockpit-completeness cell hcc-6 — pitfall candidate
description: "Pitfall candidate mined from cell hcc-6's capped trace: sync-ack: internal herding change; skill sync is hcc-10"
timestamp: 2026-09-06
bee:
  id: herding-cockpit-completeness-hcc-6-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hcc-6.json]
  polarity: pitfall
---

# herding-cockpit-completeness cell hcc-6 — pitfall candidate

## What the cell did

Project stalled and recovered in run poll tick with progress lines

## Recorded evidence (verbatim from .bee/cells/hcc-6.json)

- **deviation** — sync-ack: internal herding change; skill sync is hcc-10

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hcc-7 — save as docs/knowledge/patterns/herding-cockpit-completeness-hcc-7-pitfall.md

---
type: bee.pattern
title: herding-cockpit-completeness cell hcc-7 — pitfall candidate
description: "Pitfall candidate mined from cell hcc-7's capped trace: sync-ack: internal herding change; skill sync is hcc-10"
timestamp: 2026-09-06
bee:
  id: herding-cockpit-completeness-hcc-7-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hcc-7.json]
  polarity: pitfall
---

# herding-cockpit-completeness cell hcc-7 — pitfall candidate

## What the cell did

Add jobs array to status and orphan sweep to status and occupancy

## Recorded evidence (verbatim from .bee/cells/hcc-7.json)

- **deviation** — sync-ack: internal herding change; skill sync is hcc-10

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hcc-8 — save as docs/knowledge/patterns/herding-cockpit-completeness-hcc-8-pitfall.md

---
type: bee.pattern
title: herding-cockpit-completeness cell hcc-8 — pitfall candidate
description: "Pitfall candidate mined from cell hcc-8's capped trace: sync-ack: internal herding change; skill sync is hcc-10"
timestamp: 2026-09-06
bee:
  id: herding-cockpit-completeness-hcc-8-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hcc-8.json]
  polarity: pitfall
---

# herding-cockpit-completeness cell hcc-8 — pitfall candidate

## What the cell did

Add git handoff block to done and blocked run envelopes

## Recorded evidence (verbatim from .bee/cells/hcc-8.json)

- **deviation** — sync-ack: internal herding change; skill sync is hcc-10

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hcc-9 — save as docs/knowledge/patterns/herding-cockpit-completeness-hcc-9-pitfall.md

---
type: bee.pattern
title: herding-cockpit-completeness cell hcc-9 — pitfall candidate
description: "Pitfall candidate mined from cell hcc-9's capped trace: WorkerRow struct initialization in run.rs and control_loop.rs required retryable: None — field addition broke compilation — something else had to be fixed first"
timestamp: 2026-09-06
bee:
  id: herding-cockpit-completeness-hcc-9-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hcc-9.json]
  polarity: pitfall
---

# herding-cockpit-completeness cell hcc-9 — pitfall candidate

## What the cell did

Carry retryable bit on wave bucket rows and ledger worker rows

## Recorded evidence (verbatim from .bee/cells/hcc-9.json)

- **deviation** — WorkerRow struct initialization in run.rs and control_loop.rs required retryable: None — field addition broke compilation — something else had to be fixed first
- **deviation** — sync-ack: internal herding change; skill sync is hcc-10

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hcc-10 — save as docs/knowledge/patterns/herding-cockpit-completeness-hcc-10-pitfall.md

---
type: bee.pattern
title: herding-cockpit-completeness cell hcc-10 — pitfall candidate
description: "Pitfall candidate mined from cell hcc-10's capped trace: sync-ack: this cell is the docs sync"
timestamp: 2026-09-06
bee:
  id: herding-cockpit-completeness-hcc-10-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hcc-10.json]
  polarity: pitfall
---

# herding-cockpit-completeness cell hcc-10 — pitfall candidate

## What the cell did

Synced herding knowledge docs and config reference with cockpit signals

## Recorded evidence (verbatim from .bee/cells/hcc-10.json)

- **deviation** — sync-ack: this cell is the docs sync

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 10 capped cell(s) mined, 1 delivery draft, 6 area bullet(s), 10 pattern candidate(s), 0 file(s) written.