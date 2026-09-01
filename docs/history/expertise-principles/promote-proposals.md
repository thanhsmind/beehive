promote proposal for work item "expertise-principles" (docs/history/expertise-principles/CONTEXT.md + docs/history/expertise-principles/plan.md) — 9 capped cell(s): ep-1, ep-2, ep-3, ep-4, ep-5, ep-6, ep-7, ep-8, ep-9
anchor: history — docs/history/expertise-principles/CONTEXT.md, docs/history/expertise-principles/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/expertise-principles/delivery.md

---
type: bee.delivery
title: expertise-principles — delivery
description: "Delivery record proposed by bee knowledge promote for work item expertise-principles: 9 capped cell(s), 16 recorded deviation(s)."
timestamp: 2026-09-01
bee:
  id: expertise-principles-delivery
  lifecycle: active
  required_context: [docs/history/expertise-principles/CONTEXT.md, docs/history/expertise-principles/plan.md]
  sources: [docs/history/expertise-principles/CONTEXT.md, docs/history/expertise-principles/plan.md, .bee/cells/ep-1.json, .bee/cells/ep-2.json, .bee/cells/ep-3.json, .bee/cells/ep-4.json, .bee/cells/ep-5.json, .bee/cells/ep-6.json, .bee/cells/ep-7.json, .bee/cells/ep-8.json, .bee/cells/ep-9.json]
---

# expertise-principles — Delivery

## What shipped

- **ep-1** — Added the Principle homes index section with its first row and shipped skills/principle-red-before-green/SKILL.md (2 file(s) changed)
- **ep-2** — One shared reader renders the routed principles in both bee orient and the session preamble; nothing renders without a matching route (7 file(s) changed)
- **ep-3** — Fence the principle index against the skill dirs and the class vocabulary (1 file(s) changed)
- **ep-4** — AGENTS gains an eleventh rule obliging the reply to name each applied principle and what it changed (5 file(s) changed)
- **ep-5** — Five craft principles ship as index rows plus thin skill pages; parity fence, manifest and knowledge check all green (7 file(s) changed)
- **ep-6** — Five craft principles ship as thin skills with rows in the doctrine-layer principle index (7 file(s) changed)
- **ep-7** — Three craft principles shipped: reproduce-first, crash-site-versus-fault-site, never-invent-behavior-neither-side-has (5 file(s) changed)
- **ep-8** — Renamed the 14 principle skills into the bee- namespace so both distribution pipes ship them, repointed the index rows, and grew the parity fence a check that reads the shipped plugin trees (20 file(s) changed)
- **ep-9** — Three short pointers route workflow readers to the Principle homes index; no principle named (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ep-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml rule_index_parity && .bee/bin/bee knowledge check`
- **ep-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
- **ep-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test principle_index_parity`
- **ep-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check`
- **ep-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test principle_index_parity && .bee/bin/bee dev release-manifest --check && .bee/bin/bee knowledge check`
- **ep-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test principle_index_parity && .bee/bin/bee dev release-manifest --check && .bee/bin/bee knowledge check`
- **ep-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test principle_index_parity && .bee/bin/bee dev release-manifest --check && .bee/bin/bee knowledge check`
- **ep-8** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check`
- **ep-9** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity --test principle_index_parity && .bee/bin/bee knowledge check && .bee/bin/bee dev release-manifest --check`

## Deviations

- **ep-1** — Ran the parity fence as --test rule_index_parity instead of the cell verify text — the cell passed rule_index_parity as a test-NAME filter, which matches no test and ran 0 tests, so the verify as written could not fail — hit an unforeseen obstacle
- **ep-2** — Homed the shared reader in a new crate-root module packages/bee-rs/crates/bee/src/principles.rs and registered it in main.rs — the cell named no files and the two callers live in different module trees, so a crate-root module is the only place both can call without threading a signal through a layer — found a better route
- **ep-2** — Regenerated docs/history/codex-harness-hardening/release-manifest.json — ep-1 added skills/principle-red-before-green/SKILL.md without the manifest row, so devtools::release_manifest::tests::rebuild_reproduces_the_committed_manifest was already red on this base (295 vs 296 records) — something else had to be fixed first
- **ep-2** — Added two caller-side tests beyond the five the cell named, one per surface — the five unit tests prove the parser only, and a guard and its tests are one model unless a test drives the real call site — found a better route
- **ep-3** — followed the plan
- **ep-4** — Rewrote two stale counts in the index doc ("these ten are homed", "twenty rules where there are ten") into count-free wording — the eleventh rule made both sentences false, and a count restated in prose goes stale on every new rule — something else had to be fixed first
- **ep-4** — sync-ack: The AGENTS.md edit adds a new rule id; no existing rule's text changed, so no applied_at restatement of agents-capture-line-at-close or any sibling rule is stale.
- **ep-5** — followed the plan
- **ep-6** — followed the plan
- **ep-7** — followed the plan
- **ep-8** — Renamed all 14 skills to bee-principle-* and repointed the index, the fence prefix, principles.rs and its two caller fixtures rather than shipping principle-* — plan.md load-bearing claim 1 said both pipelines scan skills/ with no prefix filter, but skill_trees.rs list_bee_skill_dirs and onboard/render.rs list_bee_skill_entries both filter on a literal bee- name prefix, so the layer reached zero host repos while the suite stayed green — the plan was wrong about a fact
- **ep-8** — Landed the new fence check as the FIFTH pinned item, not the fourth the cell named — the file already pinned four (skills against rows both directions, class values, spoken line, guide anchor), so a fourth check had no free slot — the plan was wrong about a fact
- **ep-8** — Also namespaced the synthetic slugs in principles.rs TEST_INDEX and the two caller test fixtures — the assertion over this repo own index now requires the bee-principle- prefix, so a fixture left in the old shape would model a slug the product rejects — found a better route
- **ep-8** — Left docs/history/** references to old principle- slugs alone (plan.md line 93; research notes describing the pstack reference tree skills) and let bee dev regen rewrite the generated release-manifest.json itself — history records are archaeology, not live pointers, and hand-editing a generated manifest would fight the regen chain — found a better route
- **ep-8** — sync-ack: affects_skills deliberately listed BOTH sides of the rename (14 old principle-* paths plus 14 new bee-principle-* paths). A git mv leaves only the new paths touched, so the 14 old ones can never be touched by the cell that removes them - the prediction is satisfied, not missed.
- **ep-9** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work expertise-principles` from 9 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/expertise-principles/CONTEXT.md`, `docs/history/expertise-principles/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell ep-1 — save as docs/knowledge/patterns/expertise-principles-ep-1-pitfall.md

---
type: bee.pattern
title: expertise-principles cell ep-1 — pitfall candidate
description: "Pitfall candidate mined from cell ep-1's capped trace: Ran the parity fence as --test rule_index_parity instead of the cell verify text — the cell passed rule_index_parity as a test-NAME filter, which matches no te…"
timestamp: 2026-09-01
bee:
  id: expertise-principles-ep-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/ep-1.json]
  polarity: pitfall
---

# expertise-principles cell ep-1 — pitfall candidate

## What the cell did

Added the Principle homes index section with its first row and shipped skills/principle-red-before-green/SKILL.md

## Recorded evidence (verbatim from .bee/cells/ep-1.json)

- **deviation** — Ran the parity fence as --test rule_index_parity instead of the cell verify text — the cell passed rule_index_parity as a test-NAME filter, which matches no test and ran 0 tests, so the verify as written could not fail — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ep-2 — save as docs/knowledge/patterns/expertise-principles-ep-2-pitfall.md

---
type: bee.pattern
title: expertise-principles cell ep-2 — pitfall candidate
description: "Pitfall candidate mined from cell ep-2's capped trace: Homed the shared reader in a new crate-root module packages/bee-rs/crates/bee/src/principles.rs and registered it in main.rs — the cell named no files and the …"
timestamp: 2026-09-01
bee:
  id: expertise-principles-ep-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/ep-2.json]
  polarity: pitfall
---

# expertise-principles cell ep-2 — pitfall candidate

## What the cell did

One shared reader renders the routed principles in both bee orient and the session preamble; nothing renders without a matching route

## Recorded evidence (verbatim from .bee/cells/ep-2.json)

- **deviation** — Homed the shared reader in a new crate-root module packages/bee-rs/crates/bee/src/principles.rs and registered it in main.rs — the cell named no files and the two callers live in different module trees, so a crate-root module is the only place both can call without threading a signal through a layer — found a better route
- **deviation** — Regenerated docs/history/codex-harness-hardening/release-manifest.json — ep-1 added skills/principle-red-before-green/SKILL.md without the manifest row, so devtools::release_manifest::tests::rebuild_reproduces_the_committed_manifest was already red on this base (295 vs 296 records) — something else had to be fixed first
- **deviation** — Added two caller-side tests beyond the five the cell named, one per surface — the five unit tests prove the parser only, and a guard and its tests are one model unless a test drives the real call site — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ep-3 — save as docs/knowledge/patterns/expertise-principles-ep-3-pitfall.md

---
type: bee.pattern
title: expertise-principles cell ep-3 — pitfall candidate
description: "Pitfall candidate mined from cell ep-3's capped trace: followed the plan"
timestamp: 2026-09-01
bee:
  id: expertise-principles-ep-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/ep-3.json]
  polarity: pitfall
---

# expertise-principles cell ep-3 — pitfall candidate

## What the cell did

Fence the principle index against the skill dirs and the class vocabulary

## Recorded evidence (verbatim from .bee/cells/ep-3.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ep-4 — save as docs/knowledge/patterns/expertise-principles-ep-4-pitfall.md

---
type: bee.pattern
title: expertise-principles cell ep-4 — pitfall candidate
description: "Pitfall candidate mined from cell ep-4's capped trace: Rewrote two stale counts in the index doc (\"these ten are homed\", \"twenty rules where there are ten\") into count-free wording — the eleventh rule made both sen…"
timestamp: 2026-09-01
bee:
  id: expertise-principles-ep-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/ep-4.json]
  polarity: pitfall
---

# expertise-principles cell ep-4 — pitfall candidate

## What the cell did

AGENTS gains an eleventh rule obliging the reply to name each applied principle and what it changed

## Recorded evidence (verbatim from .bee/cells/ep-4.json)

- **deviation** — Rewrote two stale counts in the index doc ("these ten are homed", "twenty rules where there are ten") into count-free wording — the eleventh rule made both sentences false, and a count restated in prose goes stale on every new rule — something else had to be fixed first
- **deviation** — sync-ack: The AGENTS.md edit adds a new rule id; no existing rule's text changed, so no applied_at restatement of agents-capture-line-at-close or any sibling rule is stale.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ep-5 — save as docs/knowledge/patterns/expertise-principles-ep-5-pitfall.md

---
type: bee.pattern
title: expertise-principles cell ep-5 — pitfall candidate
description: "Pitfall candidate mined from cell ep-5's capped trace: followed the plan"
timestamp: 2026-09-01
bee:
  id: expertise-principles-ep-5-pitfall
  lifecycle: draft
  sources: [.bee/cells/ep-5.json]
  polarity: pitfall
---

# expertise-principles cell ep-5 — pitfall candidate

## What the cell did

Five craft principles ship as index rows plus thin skill pages; parity fence, manifest and knowledge check all green

## Recorded evidence (verbatim from .bee/cells/ep-5.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ep-6 — save as docs/knowledge/patterns/expertise-principles-ep-6-pitfall.md

---
type: bee.pattern
title: expertise-principles cell ep-6 — pitfall candidate
description: "Pitfall candidate mined from cell ep-6's capped trace: followed the plan"
timestamp: 2026-09-01
bee:
  id: expertise-principles-ep-6-pitfall
  lifecycle: draft
  sources: [.bee/cells/ep-6.json]
  polarity: pitfall
---

# expertise-principles cell ep-6 — pitfall candidate

## What the cell did

Five craft principles ship as thin skills with rows in the doctrine-layer principle index

## Recorded evidence (verbatim from .bee/cells/ep-6.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ep-7 — save as docs/knowledge/patterns/expertise-principles-ep-7-pitfall.md

---
type: bee.pattern
title: expertise-principles cell ep-7 — pitfall candidate
description: "Pitfall candidate mined from cell ep-7's capped trace: followed the plan"
timestamp: 2026-09-01
bee:
  id: expertise-principles-ep-7-pitfall
  lifecycle: draft
  sources: [.bee/cells/ep-7.json]
  polarity: pitfall
---

# expertise-principles cell ep-7 — pitfall candidate

## What the cell did

Three craft principles shipped: reproduce-first, crash-site-versus-fault-site, never-invent-behavior-neither-side-has

## Recorded evidence (verbatim from .bee/cells/ep-7.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ep-8 — save as docs/knowledge/patterns/expertise-principles-ep-8-pitfall.md

---
type: bee.pattern
title: expertise-principles cell ep-8 — pitfall candidate
description: "Pitfall candidate mined from cell ep-8's capped trace: Renamed all 14 skills to bee-principle-* and repointed the index, the fence prefix, principles.rs and its two caller fixtures rather than shipping principle-* …"
timestamp: 2026-09-01
bee:
  id: expertise-principles-ep-8-pitfall
  lifecycle: draft
  sources: [.bee/cells/ep-8.json]
  polarity: pitfall
---

# expertise-principles cell ep-8 — pitfall candidate

## What the cell did

Renamed the 14 principle skills into the bee- namespace so both distribution pipes ship them, repointed the index rows, and grew the parity fence a check that reads the shipped plugin trees

## Recorded evidence (verbatim from .bee/cells/ep-8.json)

- **deviation** — Renamed all 14 skills to bee-principle-* and repointed the index, the fence prefix, principles.rs and its two caller fixtures rather than shipping principle-* — plan.md load-bearing claim 1 said both pipelines scan skills/ with no prefix filter, but skill_trees.rs list_bee_skill_dirs and onboard/render.rs list_bee_skill_entries both filter on a literal bee- name prefix, so the layer reached zero host repos while the suite stayed green — the plan was wrong about a fact
- **deviation** — Landed the new fence check as the FIFTH pinned item, not the fourth the cell named — the file already pinned four (skills against rows both directions, class values, spoken line, guide anchor), so a fourth check had no free slot — the plan was wrong about a fact
- **deviation** — Also namespaced the synthetic slugs in principles.rs TEST_INDEX and the two caller test fixtures — the assertion over this repo own index now requires the bee-principle- prefix, so a fixture left in the old shape would model a slug the product rejects — found a better route
- **deviation** — Left docs/history/** references to old principle- slugs alone (plan.md line 93; research notes describing the pstack reference tree skills) and let bee dev regen rewrite the generated release-manifest.json itself — history records are archaeology, not live pointers, and hand-editing a generated manifest would fight the regen chain — found a better route
- **deviation** — sync-ack: affects_skills deliberately listed BOTH sides of the rename (14 old principle-* paths plus 14 new bee-principle-* paths). A git mv leaves only the new paths touched, so the 14 old ones can never be touched by the cell that removes them - the prediction is satisfied, not missed.
- **failure_signature** — c12b99b76a2a

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ep-9 — save as docs/knowledge/patterns/expertise-principles-ep-9-pitfall.md

---
type: bee.pattern
title: expertise-principles cell ep-9 — pitfall candidate
description: "Pitfall candidate mined from cell ep-9's capped trace: followed the plan"
timestamp: 2026-09-01
bee:
  id: expertise-principles-ep-9-pitfall
  lifecycle: draft
  sources: [.bee/cells/ep-9.json]
  polarity: pitfall
---

# expertise-principles cell ep-9 — pitfall candidate

## What the cell did

Three short pointers route workflow readers to the Principle homes index; no principle named

## Recorded evidence (verbatim from .bee/cells/ep-9.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 9 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 9 pattern candidate(s), 0 file(s) written.