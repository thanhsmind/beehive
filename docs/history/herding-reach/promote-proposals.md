promote proposal for work item "herding-reach" (.bee/lanes/herding-reach.json) — 4 capped cell(s): hrc-1, hrc-2, hrc-3, hrc-4
anchor: ledger — .bee/lanes/herding-reach.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-reach/delivery.md

---
type: bee.delivery
title: herding-reach — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-reach: 4 capped cell(s), 8 recorded deviation(s)."
timestamp: 2026-08-22
bee:
  id: herding-reach-delivery
  lifecycle: active
  required_context: [.bee/lanes/herding-reach.json]
  sources: [.bee/lanes/herding-reach.json, .bee/cells/hrc-1.json, .bee/cells/hrc-2.json, .bee/cells/hrc-3.json, .bee/cells/hrc-4.json]
---

# herding-reach — Delivery

## What shipped

- **hrc-1** — dispatch prepare payload carries transport_ready/transport_reason; fallback carries fallback_when; Delegation contract names the rule (3 file(s) changed)
- **hrc-2** — bee herding status is built: enable-marker state plus transport {ready, reason, pane_id}; registry unavailable block removed (2 file(s) changed)
- **hrc-3** — give-up errors carry the pane tail on no match; spawn_failed JSON carries a remedy naming the unwind; swarming reference documents it (2 file(s) changed)
- **hrc-4** — dispatch prepare resolves a cell's payload from its recorded tier; ceiling cells report channel session-model; unconfigured recorded tier is a typed refusal; economics carries tier_source (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hrc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee drivers`
- **hrc-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding registry`
- **hrc-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run`
- **hrc-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee drivers`

## Deviations

- **hrc-1** — worker commit bb47fac6 swept in hrc-2's in-progress herding.rs edits (git add -A in a shared worktree); hrc-2's own commit carries the rest — no conflict, noted for the pattern miner
- **hrc-1** — commit swept in sibling hrc-2 edits from the shared worktree
- **hrc-2** — worker's proof line omitted the result; orchestrator re-ran the suites: herding 322 passed, registry_contracts+registry_dispatch 18 passed
- **hrc-2** — herding.rs content landed in hrc-1 commit bb47fac6 (shared index sweep); hrc-2 commit carries the registry edit
- **hrc-3** — worker left the work uncommitted; orchestrator made the path-scoped commit 8bec3bb4
- **hrc-3** — worker did not commit; orchestrator committed path-scoped
- **hrc-4** — worker also edited guard.rs (not in the cell's file list) so derive_economics knows the session-model channel — required for the economics record; orchestrator re-ran drivers tests: 203 passed
- **hrc-4** — guard.rs edited beyond the listed files (economics channel)

## Provenance

Proposed by `bee knowledge promote --work herding-reach` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/herding-reach.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hrc-1 — save as docs/knowledge/patterns/herding-reach-hrc-1-pitfall.md

---
type: bee.pattern
title: herding-reach cell hrc-1 — pitfall candidate
description: "Pitfall candidate mined from cell hrc-1's capped trace: worker commit bb47fac6 swept in hrc-2's in-progress herding.rs edits (git add -A in a shared worktree); hrc-2's own commit carries the rest — no conflict, note…"
timestamp: 2026-08-22
bee:
  id: herding-reach-hrc-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/hrc-1.json]
  polarity: pitfall
---

# herding-reach cell hrc-1 — pitfall candidate

## What the cell did

dispatch prepare payload carries transport_ready/transport_reason; fallback carries fallback_when; Delegation contract names the rule

## Recorded evidence (verbatim from .bee/cells/hrc-1.json)

- **deviation** — worker commit bb47fac6 swept in hrc-2's in-progress herding.rs edits (git add -A in a shared worktree); hrc-2's own commit carries the rest — no conflict, noted for the pattern miner
- **deviation** — commit swept in sibling hrc-2 edits from the shared worktree

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hrc-2 — save as docs/knowledge/patterns/herding-reach-hrc-2-pitfall.md

---
type: bee.pattern
title: herding-reach cell hrc-2 — pitfall candidate
description: "Pitfall candidate mined from cell hrc-2's capped trace: worker's proof line omitted the result; orchestrator re-ran the suites: herding 322 passed, registry_contracts+registry_dispatch 18 passed"
timestamp: 2026-08-22
bee:
  id: herding-reach-hrc-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/hrc-2.json]
  polarity: pitfall
---

# herding-reach cell hrc-2 — pitfall candidate

## What the cell did

bee herding status is built: enable-marker state plus transport {ready, reason, pane_id}; registry unavailable block removed

## Recorded evidence (verbatim from .bee/cells/hrc-2.json)

- **deviation** — worker's proof line omitted the result; orchestrator re-ran the suites: herding 322 passed, registry_contracts+registry_dispatch 18 passed
- **deviation** — herding.rs content landed in hrc-1 commit bb47fac6 (shared index sweep); hrc-2 commit carries the registry edit

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hrc-3 — save as docs/knowledge/patterns/herding-reach-hrc-3-pitfall.md

---
type: bee.pattern
title: herding-reach cell hrc-3 — pitfall candidate
description: "Pitfall candidate mined from cell hrc-3's capped trace: worker left the work uncommitted; orchestrator made the path-scoped commit 8bec3bb4"
timestamp: 2026-08-22
bee:
  id: herding-reach-hrc-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/hrc-3.json]
  polarity: pitfall
---

# herding-reach cell hrc-3 — pitfall candidate

## What the cell did

give-up errors carry the pane tail on no match; spawn_failed JSON carries a remedy naming the unwind; swarming reference documents it

## Recorded evidence (verbatim from .bee/cells/hrc-3.json)

- **deviation** — worker left the work uncommitted; orchestrator made the path-scoped commit 8bec3bb4
- **deviation** — worker did not commit; orchestrator committed path-scoped

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hrc-4 — save as docs/knowledge/patterns/herding-reach-hrc-4-pitfall.md

---
type: bee.pattern
title: herding-reach cell hrc-4 — pitfall candidate
description: "Pitfall candidate mined from cell hrc-4's capped trace: worker also edited guard.rs (not in the cell's file list) so derive_economics knows the session-model channel — required for the economics record; orchestrator…"
timestamp: 2026-08-22
bee:
  id: herding-reach-hrc-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/hrc-4.json]
  polarity: pitfall
---

# herding-reach cell hrc-4 — pitfall candidate

## What the cell did

dispatch prepare resolves a cell's payload from its recorded tier; ceiling cells report channel session-model; unconfigured recorded tier is a typed refusal; economics carries tier_source

## Recorded evidence (verbatim from .bee/cells/hrc-4.json)

- **deviation** — worker also edited guard.rs (not in the cell's file list) so derive_economics knows the session-model channel — required for the economics record; orchestrator re-ran drivers tests: 203 passed
- **deviation** — guard.rs edited beyond the listed files (economics channel)

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 4 pattern candidate(s), 0 file(s) written.