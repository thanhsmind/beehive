promote proposal for work item "paseo-visible-sessions" (docs/history/paseo-visible-sessions/CONTEXT.md) — 3 capped cell(s): pvs-1, pvs-2, pvs-3
anchor: history — docs/history/paseo-visible-sessions/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/paseo-visible-sessions/delivery.md

---
type: bee.delivery
title: paseo-visible-sessions — delivery
description: "Delivery record proposed by bee knowledge promote for work item paseo-visible-sessions: 3 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-11-07
bee:
  id: paseo-visible-sessions-delivery
  lifecycle: active
  required_context: [docs/history/paseo-visible-sessions/CONTEXT.md]
  sources: [docs/history/paseo-visible-sessions/CONTEXT.md, .bee/cells/pvs-1.json, .bee/cells/pvs-2.json, .bee/cells/pvs-3.json]
---

# paseo-visible-sessions — Delivery

## What shipped

- **pvs-1** — Start claude-session sub-sessions through paseo run inside Paseo (1 file(s) changed)
- **pvs-2** — Record why a native Claude dispatch fell back to claude-agent (1 file(s) changed)
- **pvs-3** — Documented the Paseo route (CONTEXT, Inside Paseo section, index line); timings.jsonl restored; commit 07f890653 (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pvs-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::` — change confined to herding/run.rs; all 838 herding tests ran
- **pvs-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prepare` — change confined to prepare.rs and its in-file tests
- **pvs-3** — `.bee/bin/bee knowledge check` — docs-only change; 0 OKF errors, 0 profile errors; git status clean after commit

## Deviations

- **pvs-1** — sync-ack: skills/bee-herding names no claude-session or session-model transport detail; the area doc claude-session-transport.md carries it and the docs cell pvs-3 updates it
- **pvs-2** — an escalated dispatch lands on channel session-model, not claude-agent; it still gets native_fallback_reason escalated

## Provenance

Proposed by `bee knowledge promote --work paseo-visible-sessions` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/paseo-visible-sessions/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pvs-1 — save as docs/knowledge/patterns/paseo-visible-sessions-pvs-1-pitfall.md

---
type: bee.pattern
title: paseo-visible-sessions cell pvs-1 — pitfall candidate
description: "Pitfall candidate mined from cell pvs-1's capped trace: sync-ack: skills/bee-herding names no claude-session or session-model transport detail; the area doc claude-session-transport.md carries it and the docs cell p…"
timestamp: 2026-11-07
bee:
  id: paseo-visible-sessions-pvs-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/pvs-1.json]
  polarity: pitfall
---

# paseo-visible-sessions cell pvs-1 — pitfall candidate

## What the cell did

Start claude-session sub-sessions through paseo run inside Paseo

## Recorded evidence (verbatim from .bee/cells/pvs-1.json)

- **deviation** — sync-ack: skills/bee-herding names no claude-session or session-model transport detail; the area doc claude-session-transport.md carries it and the docs cell pvs-3 updates it

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pvs-2 — save as docs/knowledge/patterns/paseo-visible-sessions-pvs-2-pitfall.md

---
type: bee.pattern
title: paseo-visible-sessions cell pvs-2 — pitfall candidate
description: "Pitfall candidate mined from cell pvs-2's capped trace: an escalated dispatch lands on channel session-model, not claude-agent; it still gets native_fallback_reason escalated"
timestamp: 2026-11-07
bee:
  id: paseo-visible-sessions-pvs-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/pvs-2.json]
  polarity: pitfall
---

# paseo-visible-sessions cell pvs-2 — pitfall candidate

## What the cell did

Record why a native Claude dispatch fell back to claude-agent

## Recorded evidence (verbatim from .bee/cells/pvs-2.json)

- **deviation** — an escalated dispatch lands on channel session-model, not claude-agent; it still gets native_fallback_reason escalated

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.