promote proposal for work item "pi-beehive" (docs/history/pi-beehive/CONTEXT.md + docs/history/pi-beehive/plan.md) — 6 capped cell(s): pib-1, pib-2, pib-3, pib-4, pib-5, pib-6
anchor: history — docs/history/pi-beehive/CONTEXT.md, docs/history/pi-beehive/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-beehive/delivery.md

---
type: bee.delivery
title: pi-beehive — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-beehive: 6 capped cell(s), 9 recorded deviation(s)."
timestamp: 2026-09-02
bee:
  id: pi-beehive-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/pi-beehive/CONTEXT.md, docs/history/pi-beehive/plan.md]
  sources: [docs/history/pi-beehive/CONTEXT.md, docs/history/pi-beehive/plan.md, .bee/cells/pib-1.json, .bee/cells/pib-2.json, .bee/cells/pib-3.json, .bee/cells/pib-4.json, .bee/cells/pib-5.json, .bee/cells/pib-6.json]
---

# pi-beehive — Delivery

## What shipped

- **pib-1** — activity now fires on the Pi session shutdown with a Claude-shaped exit reason, so a quitting Pi session is marked exited instead of staying alive in the record (2 file(s) changed)
- **pib-2** — Closed the Pi session record on session_shutdown for every reason that truly ends the session, and skipped reload, which keeps the same session alive (2 file(s) changed)
- **pib-3** — agent_settled now parses the session-close verdict and injects only a block reason through sendUserMessage; an ordinary advisory nudge never opens a turn (2 file(s) changed)
- **pib-4** — The advisory-gap gate now covers both belts and derives the Pi side from source; the two unwired rules carry their name and the exclusion marker on their own line (2 file(s) changed)
- **pib-5** — The config reference now states the Pi belt row set, its excluded rules and the Claude rows with no Pi carrier; the release manifest was regenerated with a binary matching the source (2 file(s) changed)
- **pib-6** — state-sync fires on the Pi turn end, the contract derivations ignore commented-out code, and every row of the Pi config reference is anchored to the belt (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pib-1** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts`
- **pib-2** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts`
- **pib-3** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts`
- **pib-4** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts`
- **pib-5** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts && .bee/bin/bee dev release-manifest --check`
- **pib-6** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts`

## Deviations

- **pib-1** — Pi reasons are normalised to a Claude-shaped exit word before activity sees them, rather than passed through, because activity.rs deliberately ignores the word resume, which means the opposite thing on Pi — the plan was wrong about a fact
- **pib-2** — followed the plan
- **pib-3** — A fourth probe was added beyond the three the cell asked for, driving the real bee binary under gate_bypass=full so the block verdict is proven end to end and not only against a stub — found a better route
- **pib-3** — The worker reused the belts existing turnStartPending latch instead of adding one, which is what the cell asked for; recorded because the cell named it as a prohibition rather than an instruction — followed the plan
- **pib-4** — followed the plan
- **pib-5** — The regen ran through /home/thanhsmind/.cache/cargo-target/release/bee instead of the vendored .bee/bin/bee, because the vendored copy is a symlink into the main checkout and is behind the source it would regenerate from; the write guard correctly refuses replacing it from this worktree — hit an unforeseen obstacle
- **pib-6** — The last three documentation rows were corrected by the orchestrator rather than a worker, after the cell missed the same class of defect on three dispatches — hit an unforeseen obstacle
- **pib-6** — A shared comment-stripping helper was written once and applied to three derivations, rather than patching only the one the finding named — found a better route
- **pib-6** — The cell was reopened and capped a second time because an earlier cap was recorded by accident with placeholder values — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work pi-beehive` from 6 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-beehive/CONTEXT.md`, `docs/history/pi-beehive/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "pi-beehive" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-02T14:30:27.993Z), the work item declares no bee.areas.

area hook-runtime:
  - [pib-1] activity now fires on the Pi session shutdown with a Claude-shaped exit reason, so a quitting Pi session is marked exited instead of staying alive in the record — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pib-1.json)
  - [pib-2] Closed the Pi session record on session_shutdown for every reason that truly ends the session, and skipped reload, which keeps the same session alive — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pib-2.json)
  - [pib-3] agent_settled now parses the session-close verdict and injects only a block reason through sendUserMessage; an ordinary advisory nudge never opens a turn — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pib-3.json)
  - [pib-4] The advisory-gap gate now covers both belts and derives the Pi side from source; the two unwired rules carry their name and the exclusion marker on their own line — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pib-4.json)
  - [pib-6] state-sync fires on the Pi turn end, the contract derivations ignore commented-out code, and every row of the Pi config reference is anchored to the belt — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/pib-6.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pib-1 — save as docs/knowledge/patterns/pi-beehive-pib-1-pitfall.md

---
type: bee.pattern
title: pi-beehive cell pib-1 — pitfall candidate
description: "Pitfall candidate mined from cell pib-1's capped trace: Pi reasons are normalised to a Claude-shaped exit word before activity sees them, rather than passed through, because activity.rs deliberately ignores the word…"
timestamp: 2026-09-02
bee:
  id: pi-beehive-pib-1-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/pib-1.json]
  polarity: pitfall
---

# pi-beehive cell pib-1 — pitfall candidate

## What the cell did

activity now fires on the Pi session shutdown with a Claude-shaped exit reason, so a quitting Pi session is marked exited instead of staying alive in the record

## Recorded evidence (verbatim from .bee/cells/pib-1.json)

- **deviation** — Pi reasons are normalised to a Claude-shaped exit word before activity sees them, rather than passed through, because activity.rs deliberately ignores the word resume, which means the opposite thing on Pi — the plan was wrong about a fact
- **failure_signature** — activity on the Claude SessionEnd row is neither called in the session_shutdown handler nor named as a no-carrier exclusion, while the mapping comment claims a wiring the code does not have
- **failure_signature** — activity on the Claude SessionEnd row is neither called in the session_shutdown handler nor named as a no-carrier exclusion, while the mapping comment claims a wiring the code does not have

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pib-2 — save as docs/knowledge/patterns/pi-beehive-pib-2-pitfall.md

---
type: bee.pattern
title: pi-beehive cell pib-2 — pitfall candidate
description: "Pitfall candidate mined from cell pib-2's capped trace: followed the plan"
timestamp: 2026-09-02
bee:
  id: pi-beehive-pib-2-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/pib-2.json]
  polarity: pitfall
---

# pi-beehive cell pib-2 — pitfall candidate

## What the cell did

Closed the Pi session record on session_shutdown for every reason that truly ends the session, and skipped reload, which keeps the same session alive

## Recorded evidence (verbatim from .bee/cells/pib-2.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pib-3 — save as docs/knowledge/patterns/pi-beehive-pib-3-pitfall.md

---
type: bee.pattern
title: pi-beehive cell pib-3 — pitfall candidate
description: "Pitfall candidate mined from cell pib-3's capped trace: A fourth probe was added beyond the three the cell asked for, driving the real bee binary under gate_bypass=full so the block verdict is proven end to end and …"
timestamp: 2026-09-02
bee:
  id: pi-beehive-pib-3-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/pib-3.json]
  polarity: pitfall
---

# pi-beehive cell pib-3 — pitfall candidate

## What the cell did

agent_settled now parses the session-close verdict and injects only a block reason through sendUserMessage; an ordinary advisory nudge never opens a turn

## Recorded evidence (verbatim from .bee/cells/pib-3.json)

- **deviation** — A fourth probe was added beyond the three the cell asked for, driving the real bee binary under gate_bypass=full so the block verdict is proven end to end and not only against a stub — found a better route
- **deviation** — The worker reused the belts existing turnStartPending latch instead of adding one, which is what the cell asked for; recorded because the cell named it as a prohibition rather than an instruction — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pib-4 — save as docs/knowledge/patterns/pi-beehive-pib-4-pitfall.md

---
type: bee.pattern
title: pi-beehive cell pib-4 — pitfall candidate
description: "Pitfall candidate mined from cell pib-4's capped trace: followed the plan"
timestamp: 2026-09-02
bee:
  id: pi-beehive-pib-4-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/pib-4.json]
  polarity: pitfall
---

# pi-beehive cell pib-4 — pitfall candidate

## What the cell did

The advisory-gap gate now covers both belts and derives the Pi side from source; the two unwired rules carry their name and the exclusion marker on their own line

## Recorded evidence (verbatim from .bee/cells/pib-4.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pib-5 — save as docs/knowledge/patterns/pi-beehive-pib-5-pitfall.md

---
type: bee.pattern
title: pi-beehive cell pib-5 — pitfall candidate
description: "Pitfall candidate mined from cell pib-5's capped trace: The regen ran through /home/thanhsmind/.cache/cargo-target/release/bee instead of the vendored .bee/bin/bee, because the vendored copy is a symlink into the ma…"
timestamp: 2026-09-02
bee:
  id: pi-beehive-pib-5-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/pib-5.json]
  polarity: pitfall
---

# pi-beehive cell pib-5 — pitfall candidate

## What the cell did

The config reference now states the Pi belt row set, its excluded rules and the Claude rows with no Pi carrier; the release manifest was regenerated with a binary matching the source

## Recorded evidence (verbatim from .bee/cells/pib-5.json)

- **deviation** — The regen ran through /home/thanhsmind/.cache/cargo-target/release/bee instead of the vendored .bee/bin/bee, because the vendored copy is a symlink into the main checkout and is behind the source it would regenerate from; the write guard correctly refuses replacing it from this worktree — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pib-6 — save as docs/knowledge/patterns/pi-beehive-pib-6-pitfall.md

---
type: bee.pattern
title: pi-beehive cell pib-6 — pitfall candidate
description: "Pitfall candidate mined from cell pib-6's capped trace: The last three documentation rows were corrected by the orchestrator rather than a worker, after the cell missed the same class of defect on three dispatches —…"
timestamp: 2026-09-02
bee:
  id: pi-beehive-pib-6-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/pib-6.json]
  polarity: pitfall
---

# pi-beehive cell pib-6 — pitfall candidate

## What the cell did

state-sync fires on the Pi turn end, the contract derivations ignore commented-out code, and every row of the Pi config reference is anchored to the belt

## Recorded evidence (verbatim from .bee/cells/pib-6.json)

- **deviation** — The last three documentation rows were corrected by the orchestrator rather than a worker, after the cell missed the same class of defect on three dispatches — hit an unforeseen obstacle
- **deviation** — A shared comment-stripping helper was written once and applied to three derivations, rather than patching only the one the finding named — found a better route
- **deviation** — The cell was reopened and capped a second time because an earlier cap was recorded by accident with placeholder values — something else had to be fixed first
- **failure_signature** — the tools-logger row in the config reference overstates what the rule records, surviving the row-by-row re-read this cell was told to do
- **failure_signature** — the tools-logger row in the config reference overstates what the rule records, surviving the row-by-row re-read this cell was told to do

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 6 capped cell(s) mined, 1 delivery draft, 5 area bullet(s), 6 pattern candidate(s), 0 file(s) written.