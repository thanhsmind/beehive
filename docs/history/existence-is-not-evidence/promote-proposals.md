promote proposal for work item "existence-is-not-evidence" (docs/history/existence-is-not-evidence/CONTEXT.md + docs/history/existence-is-not-evidence/plan.md) — 2 capped cell(s): eine-rust-claims-gate, eine-skill-mandates
anchor: history — docs/history/existence-is-not-evidence/CONTEXT.md, docs/history/existence-is-not-evidence/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/existence-is-not-evidence/delivery.md

---
type: bee.delivery
title: existence-is-not-evidence — delivery
description: "Delivery record proposed by bee knowledge promote for work item existence-is-not-evidence: 2 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: existence-is-not-evidence-delivery
  lifecycle: active
  required_context: [docs/history/existence-is-not-evidence/CONTEXT.md, docs/history/existence-is-not-evidence/plan.md]
  sources: [docs/history/existence-is-not-evidence/CONTEXT.md, docs/history/existence-is-not-evidence/plan.md, .bee/cells/eine-rust-claims-gate.json, .bee/cells/eine-skill-mandates.json]
---

# existence-is-not-evidence — Delivery

## What shipped

- **eine-rust-claims-gate** — Shape/merged gate approvals now refuse a plan.md whose load-bearing claims table is missing, malformed, or still guessed (3 file(s) changed)
- **eine-skill-mandates** — Landed the claims-table spec, Open Questions section, tiny/small inline evidence, the reality touch and pre-flight mandates, and the claims-audit home with a pointer-only hat row; regen chain green (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **eine-rust-claims-gate** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
- **eine-skill-mandates** — `bash -c '.bee/bin/bee dev regen && .bee/bin/bee onboard --repo-root . --json && .bee/bin/bee dev release-manifest --check && diff skills/bee-planning/references/planning-reference.md .claude/skills/bee-planning/references/planning-reference.md'`

## Deviations

- **eine-rust-claims-gate** — followed the plan
- **eine-rust-claims-gate** — sync-ack: The skill/template half of this feature is the plan's second cell, eine-skill-mandates (planning-reference.md, bee-planning/SKILL.md, review.md); this cell is the binary half by explicit scope split, and the two run in parallel on disjoint files.
- **eine-skill-mandates** — Wrote the Claims-table audit into expertise/review.md, not only the .bee/expertise/review.md the cell named — the cell named the rendered copy, and bee dev regen silently reverted my first edit to it; expertise/ at repo root is the source of truth for that file — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work existence-is-not-evidence` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/existence-is-not-evidence/CONTEXT.md`, `docs/history/existence-is-not-evidence/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell eine-rust-claims-gate — save as docs/knowledge/patterns/existence-is-not-evidence-eine-rust-claims-gate-pitfall.md

---
type: bee.pattern
title: existence-is-not-evidence cell eine-rust-claims-gate — pitfall candidate
description: "Pitfall candidate mined from cell eine-rust-claims-gate's capped trace: followed the plan"
timestamp: 2026-08-30
bee:
  id: existence-is-not-evidence-eine-rust-claims-gate-pitfall
  lifecycle: draft
  sources: [.bee/cells/eine-rust-claims-gate.json]
  polarity: pitfall
---

# existence-is-not-evidence cell eine-rust-claims-gate — pitfall candidate

## What the cell did

Shape/merged gate approvals now refuse a plan.md whose load-bearing claims table is missing, malformed, or still guessed

## Recorded evidence (verbatim from .bee/cells/eine-rust-claims-gate.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: The skill/template half of this feature is the plan's second cell, eine-skill-mandates (planning-reference.md, bee-planning/SKILL.md, review.md); this cell is the binary half by explicit scope split, and the two run in parallel on disjoint files.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell eine-skill-mandates — save as docs/knowledge/patterns/existence-is-not-evidence-eine-skill-mandates-pitfall.md

---
type: bee.pattern
title: existence-is-not-evidence cell eine-skill-mandates — pitfall candidate
description: "Pitfall candidate mined from cell eine-skill-mandates's capped trace: Wrote the Claims-table audit into expertise/review.md, not only the .bee/expertise/review.md the cell named — the cell named the rendered copy, and bee dev reg…"
timestamp: 2026-08-30
bee:
  id: existence-is-not-evidence-eine-skill-mandates-pitfall
  lifecycle: draft
  sources: [.bee/cells/eine-skill-mandates.json]
  polarity: pitfall
---

# existence-is-not-evidence cell eine-skill-mandates — pitfall candidate

## What the cell did

Landed the claims-table spec, Open Questions section, tiny/small inline evidence, the reality touch and pre-flight mandates, and the claims-audit home with a pointer-only hat row; regen chain green

## Recorded evidence (verbatim from .bee/cells/eine-skill-mandates.json)

- **deviation** — Wrote the Claims-table audit into expertise/review.md, not only the .bee/expertise/review.md the cell named — the cell named the rendered copy, and bee dev regen silently reverted my first edit to it; expertise/ at repo root is the source of truth for that file — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.