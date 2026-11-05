promote proposal for work item "agent-shell-rename" (docs/history/agent-shell-rename/plan.md) — 2 capped cell(s): asr-1, asr-2
anchor: history — docs/history/agent-shell-rename/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/agent-shell-rename/delivery.md

---
type: bee.delivery
title: agent-shell-rename — delivery
description: "Delivery record proposed by bee knowledge promote for work item agent-shell-rename: 2 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-11-05
bee:
  id: agent-shell-rename-delivery
  lifecycle: active
  areas: [doctrine-layer, bee-herding]
  required_context: [docs/history/agent-shell-rename/plan.md]
  sources: [docs/history/agent-shell-rename/plan.md, .bee/cells/asr-1.json, .bee/cells/asr-2.json]
---

# agent-shell-rename — Delivery

## What shipped

- **asr-1** — Rename rendered agent shells to hive roles and add bee-elder advisor shell a4f134610 (25 file(s) changed)
- **asr-2** — Move the prose, the site guide and the knowledge concept to the new shell names (17 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **asr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml — green:unit — 4592 unit tests passed; 3 wave-barrier regen tests deferred …
- **asr-2** — `rg -n 'bee-build|bee-gather|bee-extract|bee-review\b' skills site/guide docs/knowledge/areas docs/product-description` — leader re-ran it in the worktree: exit 1, zero hits over skills, site/guide, docs/knowledge/areas and docs/product-description

## Deviations

- **asr-1** — sync-ack: the skills the workflow-state area owns were moved to the new names by sibling cell asr-2 (skills/bee-swarming/references/swarming-reference.md, same wave, commit fc9fb92)

## Provenance

Proposed by `bee knowledge promote --work agent-shell-rename` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/agent-shell-rename/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "agent-shell-rename" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-05T06:26:11.783Z), the work item declares no bee.areas.

area doctrine-layer:
  (no capped behavior_change cell exists for this feature)

area bee-herding:
  (no capped behavior_change cell exists for this feature)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell asr-1 — save as docs/knowledge/patterns/agent-shell-rename-asr-1-pitfall.md

---
type: bee.pattern
title: agent-shell-rename cell asr-1 — pitfall candidate
description: "Pitfall candidate mined from cell asr-1's capped trace: sync-ack: the skills the workflow-state area owns were moved to the new names by sibling cell asr-2 (skills/bee-swarming/references/swarming-reference.md, same…"
timestamp: 2026-11-05
bee:
  id: agent-shell-rename-asr-1-pitfall
  lifecycle: draft
  areas: [doctrine-layer, bee-herding]
  sources: [.bee/cells/asr-1.json]
  polarity: pitfall
---

# agent-shell-rename cell asr-1 — pitfall candidate

## What the cell did

Rename rendered agent shells to hive roles and add bee-elder advisor shell a4f134610

## Recorded evidence (verbatim from .bee/cells/asr-1.json)

- **deviation** — sync-ack: the skills the workflow-state area owns were moved to the new names by sibling cell asr-2 (skills/bee-swarming/references/swarming-reference.md, same wave, commit fc9fb92)

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell asr-2 — save as docs/knowledge/patterns/agent-shell-rename-asr-2-pitfall.md

---
type: bee.pattern
title: agent-shell-rename cell asr-2 — pitfall candidate
description: "Pitfall candidate mined from cell asr-2's capped trace: a9c98468e397"
timestamp: 2026-11-05
bee:
  id: agent-shell-rename-asr-2-pitfall
  lifecycle: draft
  areas: [doctrine-layer, bee-herding]
  sources: [.bee/cells/asr-2.json]
  polarity: pitfall
---

# agent-shell-rename cell asr-2 — pitfall candidate

## What the cell did

Move the prose, the site guide and the knowledge concept to the new shell names

## Recorded evidence (verbatim from .bee/cells/asr-2.json)

- **failure_signature** — a9c98468e397

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.