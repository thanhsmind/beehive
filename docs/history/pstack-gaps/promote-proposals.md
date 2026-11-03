promote proposal for work item "pstack-gaps" (docs/history/pstack-gaps/CONTEXT.md + docs/history/pstack-gaps/plan.md) — 5 capped cell(s): pg-1, pg-2, pg-3, pg-4, pg-5
anchor: history — docs/history/pstack-gaps/CONTEXT.md, docs/history/pstack-gaps/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pstack-gaps/delivery.md

---
type: bee.delivery
title: pstack-gaps — delivery
description: "Delivery record proposed by bee knowledge promote for work item pstack-gaps: 5 capped cell(s), 6 recorded deviation(s)."
timestamp: 2026-09-01
bee:
  id: pstack-gaps-delivery
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [docs/history/pstack-gaps/CONTEXT.md, docs/history/pstack-gaps/plan.md]
  sources: [docs/history/pstack-gaps/CONTEXT.md, docs/history/pstack-gaps/plan.md, .bee/cells/pg-1.json, .bee/cells/pg-2.json, .bee/cells/pg-3.json, .bee/cells/pg-4.json, .bee/cells/pg-5.json]
---

# pstack-gaps — Delivery

## What shipped

- **pg-1** — Trace and Provenance sweep live in one bee-researching reference, cited from SKILL.md and the research playbook (4 file(s) changed)
- **pg-2** — Ten rule rows gained a spoken line and the operating block states the rule-invocation law (5 file(s) changed)
- **pg-3** — Added tests/rule_index_parity.rs pinning marker, index-row and spoken-line parity; seen red then green (1 file(s) changed)
- **pg-4** — Provenance categories 5, 6 and 7 now name the command that sweeps them (2 file(s) changed)
- **pg-5** — The provenance sweep's test-directory glob is anchored, so it returns only directories named tests (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pg-1** — `skills/bee-researching/references/trace-and-provenance.md exists with exactly the two H2 sections `## Trace` and `## Provenance sweep`; the Trace section names the 2-to-4 ceiling and the `bee dispatch prepare` door; the Provenance sweep section lists seven numbered categories and states the empty-by-name rule. `rg -n 'trace-and-provenance' skills/` returns a row in bee-researching/SKILL.md's References table AND a pointer line in bee-planning/references/planning-reference.md. `.bee/bin/bee dev regen` runs clean. `.bee/bin/bee dev release-manifest --check` is clean.`
- **pg-2** — ``rg -c 'spoken:' docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md` returns 10. `.bee/bin/bee knowledge check --json` still reports exactly ten homed rules with no new duplicate_rule_home or unknown_rule_ref finding. The invocation law appears in packages/bee/AGENTS.block.md and, after `.bee/bin/bee dev regen`, in AGENTS.md, and both name the index path. `git diff --stat` shows AGENTS.md changed by regen, never by hand. `.bee/bin/bee dev release-manifest --check` is clean.`
- **pg-3** — ``PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity` is green, AND the recorded red run (one spoken line removed) failed naming that rule id. Both command outputs are in the cap evidence.`
- **pg-4** — `All seven numbered rows in the Provenance sweep section name a command or a path. `bee dev release-manifest --check` is clean and the five rendered plugin copies match the source file.`
- **pg-5** — `The anchored command run in this repo returns only directories named exactly tests, and returns real hits against a known behavior term. bee dev release-manifest --check is clean and the rendered copies match the source.`

## Deviations

- **pg-1** — Re-capped after the first goal-check judge returned NEEDS_REVISION on D3; the missing sweep commands for provenance categories 5, 6 and 7 landed in follow-up cells pg-4 and pg-5 rather than inside this cell, because pg-1 had already merged to main — hit an unforeseen obstacle
- **pg-2** — Committed .bee/onboarding.json, which the cell did not name — the cell-ordered onboard --apply step rewrites its managed agents_block hash, so leaving it out would commit a stale record — hit an unforeseen obstacle
- **pg-2** — sync-ack: AGENTS.md gained one new paragraph outside every rule marker; no rule block changed text, so the applied_at restatements of agents-capture-line-at-close stay accurate
- **pg-3** — followed the plan
- **pg-4** — followed the plan
- **pg-5** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work pstack-gaps` from 5 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pstack-gaps/CONTEXT.md`, `docs/history/pstack-gaps/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "pstack-gaps" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-01T17:31:51.253Z), the work item declares no bee.areas.

area doctrine-layer:
  - [pg-1] Trace and Provenance sweep live in one bee-researching reference, cited from SKILL.md and the research playbook — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/pg-1.json)
  - [pg-2] Ten rule rows gained a spoken line and the operating block states the rule-invocation law — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/pg-2.json)
  - [pg-4] Provenance categories 5, 6 and 7 now name the command that sweeps them — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pg-4.json)
  - [pg-5] The provenance sweep's test-directory glob is anchored, so it returns only directories named tests — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pg-5.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pg-1 — save as docs/knowledge/patterns/pstack-gaps-pg-1-pitfall.md

---
type: bee.pattern
title: pstack-gaps cell pg-1 — pitfall candidate
description: "Pitfall candidate mined from cell pg-1's capped trace: Re-capped after the first goal-check judge returned NEEDS_REVISION on D3; the missing sweep commands for provenance categories 5, 6 and 7 landed in follow-up c…"
timestamp: 2026-09-01
bee:
  id: pstack-gaps-pg-1-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/pg-1.json]
  polarity: pitfall
---

# pstack-gaps cell pg-1 — pitfall candidate

## What the cell did

Trace and Provenance sweep live in one bee-researching reference, cited from SKILL.md and the research playbook

## Recorded evidence (verbatim from .bee/cells/pg-1.json)

- **deviation** — Re-capped after the first goal-check judge returned NEEDS_REVISION on D3; the missing sweep commands for provenance categories 5, 6 and 7 landed in follow-up cells pg-4 and pg-5 rather than inside this cell, because pg-1 had already merged to main — hit an unforeseen obstacle
- **failure_signature** — skills/bee-researching/references/trace-and-provenance.md:47-52 — provenance categories 5 (code comments), 6 (tests) and 7 (external tracker) name no sweep command or path, while categories 1-4 each do

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pg-2 — save as docs/knowledge/patterns/pstack-gaps-pg-2-pitfall.md

---
type: bee.pattern
title: pstack-gaps cell pg-2 — pitfall candidate
description: "Pitfall candidate mined from cell pg-2's capped trace: Committed .bee/onboarding.json, which the cell did not name — the cell-ordered onboard --apply step rewrites its managed agents_block hash, so leaving it out w…"
timestamp: 2026-09-01
bee:
  id: pstack-gaps-pg-2-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/pg-2.json]
  polarity: pitfall
---

# pstack-gaps cell pg-2 — pitfall candidate

## What the cell did

Ten rule rows gained a spoken line and the operating block states the rule-invocation law

## Recorded evidence (verbatim from .bee/cells/pg-2.json)

- **deviation** — Committed .bee/onboarding.json, which the cell did not name — the cell-ordered onboard --apply step rewrites its managed agents_block hash, so leaving it out would commit a stale record — hit an unforeseen obstacle
- **deviation** — sync-ack: AGENTS.md gained one new paragraph outside every rule marker; no rule block changed text, so the applied_at restatements of agents-capture-line-at-close stay accurate

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pg-3 — save as docs/knowledge/patterns/pstack-gaps-pg-3-pitfall.md

---
type: bee.pattern
title: pstack-gaps cell pg-3 — pitfall candidate
description: "Pitfall candidate mined from cell pg-3's capped trace: followed the plan"
timestamp: 2026-09-01
bee:
  id: pstack-gaps-pg-3-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/pg-3.json]
  polarity: pitfall
---

# pstack-gaps cell pg-3 — pitfall candidate

## What the cell did

Added tests/rule_index_parity.rs pinning marker, index-row and spoken-line parity; seen red then green

## Recorded evidence (verbatim from .bee/cells/pg-3.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pg-4 — save as docs/knowledge/patterns/pstack-gaps-pg-4-pitfall.md

---
type: bee.pattern
title: pstack-gaps cell pg-4 — pitfall candidate
description: "Pitfall candidate mined from cell pg-4's capped trace: followed the plan"
timestamp: 2026-09-01
bee:
  id: pstack-gaps-pg-4-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/pg-4.json]
  polarity: pitfall
---

# pstack-gaps cell pg-4 — pitfall candidate

## What the cell did

Provenance categories 5, 6 and 7 now name the command that sweeps them

## Recorded evidence (verbatim from .bee/cells/pg-4.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pg-5 — save as docs/knowledge/patterns/pstack-gaps-pg-5-pitfall.md

---
type: bee.pattern
title: pstack-gaps cell pg-5 — pitfall candidate
description: "Pitfall candidate mined from cell pg-5's capped trace: followed the plan"
timestamp: 2026-09-01
bee:
  id: pstack-gaps-pg-5-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/pg-5.json]
  polarity: pitfall
---

# pstack-gaps cell pg-5 — pitfall candidate

## What the cell did

The provenance sweep's test-directory glob is anchored, so it returns only directories named tests

## Recorded evidence (verbatim from .bee/cells/pg-5.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 5 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 5 pattern candidate(s), 0 file(s) written.