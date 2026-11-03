promote proposal for work item "bee-deploying-skill" (docs/history/bee-deploying-skill/CONTEXT.md + docs/history/bee-deploying-skill/plan.md) — 3 capped cell(s): dep-1, dep-2, dep-3
anchor: history — docs/history/bee-deploying-skill/CONTEXT.md, docs/history/bee-deploying-skill/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/bee-deploying-skill/delivery.md

---
type: bee.delivery
title: bee-deploying-skill — delivery
description: "Delivery record proposed by bee knowledge promote for work item bee-deploying-skill: 3 capped cell(s), 6 recorded deviation(s)."
timestamp: 2026-10-30
bee:
  id: bee-deploying-skill-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/bee-deploying-skill/CONTEXT.md, docs/history/bee-deploying-skill/plan.md]
  sources: [docs/history/bee-deploying-skill/CONTEXT.md, docs/history/bee-deploying-skill/plan.md, .bee/cells/dep-1.json, .bee/cells/dep-2.json, .bee/cells/dep-3.json]
---

# bee-deploying-skill — Delivery

## What shipped

- **dep-1** — bee-deploying skill, its record and script templates, and 5 trigger cases added (4 file(s) changed)
- **dep-2** — bee onboard --skillstation added end to end with a loud skip when the CLI is absent (6 file(s) changed)
- **dep-3** — Named --skillstation in all six onboard doc homes and added the two-ways install paragraph (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dep-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers` — the cell verify, run with the cargo binary at /home/thanhsmind/.cargo/bin; fixture completeness test passed, the paid real-agent eval is ignored and was not run
- **dep-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee onboard::` — the cell verify; 239 passed, 1 ignored, after the wave-close regen cleared the opencode projection drift
- **dep-3** — `rg -c -e --skillstation INSTALL.md README.md skills/bee-hive/references/onboarding.md docs/product-description/maintenance/onboarding.md docs/knowledge/areas/onboarding/host-project-artifacts.md .bee/verify/verify-app/features/onboard.md` — docs-only cell, the cell verify parity count is the proof (INSTALL.md:2 README.md:2 skills/bee-hive/references/onboarding.md:1 product onboarding.md:2 host-project-artifacts.md:2 onboard.md:2); no co…

## Deviations

- **dep-1** — The draft templates carried hash comment lines in the script and in the record code blocks; the script now holds bare placeholder lines and the record shows Preconditions and Verify as Command | Must show tables — the cell forbids comments beyond the shebang line — the plan was wrong about a fact
- **dep-1** — Committed with a path-scoped commit after an intent-to-add stage, not a plain add — the concurrent-worker guard refused staging into the shared index — hit an unforeseen obstacle
- **dep-2** — The apply stages a copy of the rendered skill under .bee/tmp/skillstation-publish and points the CLI at the copy instead of at .claude/skills/<name> — skillstation 0.9.0 skill commit --yes closes and unlinks the folder it is given, which would delete the per-project copy — the plan was wrong about a fact
- **dep-2** — The paseo worker hit the 1800s ceiling before capping; the leader stripped the comments it had written, re-ran the proof, committed and capped on its behalf — hit an unforeseen obstacle
- **dep-3** — Put the INSTALL.md flag row in the bee onboard Flags table, not the install-script table with the PowerShell column, and the two-ways paragraph says the install scripts do not pass the flag yet — scripts/install.sh:165 refuses unknown options, so a bash-column row would be false — the plan was wrong about a fact
- **dep-3** — README Useful flags sentence names --skillstation as bee onboard only, for the same reason — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work bee-deploying-skill` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/bee-deploying-skill/CONTEXT.md`, `docs/history/bee-deploying-skill/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "bee-deploying-skill" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-30T10:48:31.513Z), the work item declares no bee.areas.

area onboarding:
  - [dep-2] bee onboard --skillstation added end to end with a loud skip when the CLI is absent — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/dep-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell dep-1 — save as docs/knowledge/patterns/bee-deploying-skill-dep-1-pitfall.md

---
type: bee.pattern
title: bee-deploying-skill cell dep-1 — pitfall candidate
description: "Pitfall candidate mined from cell dep-1's capped trace: The draft templates carried hash comment lines in the script and in the record code blocks; the script now holds bare placeholder lines and the record shows Pr…"
timestamp: 2026-10-30
bee:
  id: bee-deploying-skill-dep-1-pitfall
  lifecycle: draft
  areas: [onboarding]
  sources: [.bee/cells/dep-1.json]
  polarity: pitfall
---

# bee-deploying-skill cell dep-1 — pitfall candidate

## What the cell did

bee-deploying skill, its record and script templates, and 5 trigger cases added

## Recorded evidence (verbatim from .bee/cells/dep-1.json)

- **deviation** — The draft templates carried hash comment lines in the script and in the record code blocks; the script now holds bare placeholder lines and the record shows Preconditions and Verify as Command | Must show tables — the cell forbids comments beyond the shebang line — the plan was wrong about a fact
- **deviation** — Committed with a path-scoped commit after an intent-to-add stage, not a plain add — the concurrent-worker guard refused staging into the shared index — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell dep-2 — save as docs/knowledge/patterns/bee-deploying-skill-dep-2-pitfall.md

---
type: bee.pattern
title: bee-deploying-skill cell dep-2 — pitfall candidate
description: "Pitfall candidate mined from cell dep-2's capped trace: The apply stages a copy of the rendered skill under .bee/tmp/skillstation-publish and points the CLI at the copy instead of at .claude/skills/<name> — skillsta…"
timestamp: 2026-10-30
bee:
  id: bee-deploying-skill-dep-2-pitfall
  lifecycle: draft
  areas: [onboarding]
  sources: [.bee/cells/dep-2.json]
  polarity: pitfall
---

# bee-deploying-skill cell dep-2 — pitfall candidate

## What the cell did

bee onboard --skillstation added end to end with a loud skip when the CLI is absent

## Recorded evidence (verbatim from .bee/cells/dep-2.json)

- **deviation** — The apply stages a copy of the rendered skill under .bee/tmp/skillstation-publish and points the CLI at the copy instead of at .claude/skills/<name> — skillstation 0.9.0 skill commit --yes closes and unlinks the folder it is given, which would delete the per-project copy — the plan was wrong about a fact
- **deviation** — The paseo worker hit the 1800s ceiling before capping; the leader stripped the comments it had written, re-ran the proof, committed and capped on its behalf — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell dep-3 — save as docs/knowledge/patterns/bee-deploying-skill-dep-3-pitfall.md

---
type: bee.pattern
title: bee-deploying-skill cell dep-3 — pitfall candidate
description: "Pitfall candidate mined from cell dep-3's capped trace: Put the INSTALL.md flag row in the bee onboard Flags table, not the install-script table with the PowerShell column, and the two-ways paragraph says the instal…"
timestamp: 2026-10-30
bee:
  id: bee-deploying-skill-dep-3-pitfall
  lifecycle: draft
  areas: [onboarding]
  sources: [.bee/cells/dep-3.json]
  polarity: pitfall
---

# bee-deploying-skill cell dep-3 — pitfall candidate

## What the cell did

Named --skillstation in all six onboard doc homes and added the two-ways install paragraph

## Recorded evidence (verbatim from .bee/cells/dep-3.json)

- **deviation** — Put the INSTALL.md flag row in the bee onboard Flags table, not the install-script table with the PowerShell column, and the two-ways paragraph says the install scripts do not pass the flag yet — scripts/install.sh:165 refuses unknown options, so a bash-column row would be false — the plan was wrong about a fact
- **deviation** — README Useful flags sentence names --skillstation as bee onboard only, for the same reason — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 3 pattern candidate(s), 0 file(s) written.