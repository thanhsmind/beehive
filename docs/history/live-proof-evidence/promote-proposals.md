promote proposal for work item "live-proof-evidence" (docs/history/live-proof-evidence/CONTEXT.md + docs/history/live-proof-evidence/plan.md) — 2 capped cell(s): lpe-1, lpe-2
anchor: history — docs/history/live-proof-evidence/CONTEXT.md, docs/history/live-proof-evidence/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/live-proof-evidence/delivery.md

---
type: bee.delivery
title: live-proof-evidence — delivery
description: "Delivery record proposed by bee knowledge promote for work item live-proof-evidence: 2 capped cell(s), 5 recorded deviation(s)."
timestamp: 2026-09-22
bee:
  id: live-proof-evidence-delivery
  lifecycle: active
  required_context: [docs/history/live-proof-evidence/CONTEXT.md, docs/history/live-proof-evidence/plan.md]
  sources: [docs/history/live-proof-evidence/CONTEXT.md, docs/history/live-proof-evidence/plan.md, .bee/cells/lpe-1.json, .bee/cells/lpe-2.json]
---

# live-proof-evidence — Delivery

## What shipped

- **lpe-1** — green:live caps now need an evidence locator in the scope reason (3 file(s) changed)
- **lpe-2** — Move verify-app evidence out of /tmp and name the live-proof locator rule in the map and doctrine (7 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **lpe-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee cells::tests && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts` — the cell verify: cells::tests 288 passed 0 failed, pi_plugin_contracts 87 passed 0 failed; scoped to the two suites that cover the write path I touched, the rest of the bee suite was not run
- **lpe-2** — `.bee/bin/bee dev release-manifest --check && bash .bee/verify/verify-app/control-bee paths` — drove both: the manifest check matched 376 files, and paths printed evidence /home/thanhsmind/.local/state/bee-verify/evidence/probe, a root under the home state dir

## Deviations

- **lpe-1** — Ran the cell verify with PATH exported in a separate statement instead of the inline PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" prefix — the worktree shell guard refuses a cargo call whose PATH is computed inline — hit an unforeseen obstacle
- **lpe-1** — Updated the verification_evidence assertions at cells/tests.rs and pi_plugin_contracts.rs:7697 as well as the reason segments — those assertions read the reason segment back, so leaving them would have red the suite — the plan was wrong about a fact
- **lpe-1** — sync-ack: The swarming/doctrine wording for this rule is cell lpe-2's scope (CONTEXT.md D3); lpe-1's approved files are the three rust files only
- **lpe-2** — Included .bee/onboarding.json in the commit although the cell did not list it — bee onboard --apply inside the ordered regen chain rewrote its updated_at, and both ways back to the old bytes are guard-refused, so the honest end state is to reserve the path and land it — hit an unforeseen obstacle
- **lpe-2** — Rewrapped the bee-swarming step 5 sentence over three lines — the replacement text pushed that line past the wrap width used in the file — found a better route

## Provenance

Proposed by `bee knowledge promote --work live-proof-evidence` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/live-proof-evidence/CONTEXT.md`, `docs/history/live-proof-evidence/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell lpe-1 — save as docs/knowledge/patterns/live-proof-evidence-lpe-1-pitfall.md

---
type: bee.pattern
title: live-proof-evidence cell lpe-1 — pitfall candidate
description: "Pitfall candidate mined from cell lpe-1's capped trace: Ran the cell verify with PATH exported in a separate statement instead of the inline PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" prefix — the worktree shell g…"
timestamp: 2026-09-22
bee:
  id: live-proof-evidence-lpe-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/lpe-1.json]
  polarity: pitfall
---

# live-proof-evidence cell lpe-1 — pitfall candidate

## What the cell did

green:live caps now need an evidence locator in the scope reason

## Recorded evidence (verbatim from .bee/cells/lpe-1.json)

- **deviation** — Ran the cell verify with PATH exported in a separate statement instead of the inline PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" prefix — the worktree shell guard refuses a cargo call whose PATH is computed inline — hit an unforeseen obstacle
- **deviation** — Updated the verification_evidence assertions at cells/tests.rs and pi_plugin_contracts.rs:7697 as well as the reason segments — those assertions read the reason segment back, so leaving them would have red the suite — the plan was wrong about a fact
- **deviation** — sync-ack: The swarming/doctrine wording for this rule is cell lpe-2's scope (CONTEXT.md D3); lpe-1's approved files are the three rust files only

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell lpe-2 — save as docs/knowledge/patterns/live-proof-evidence-lpe-2-pitfall.md

---
type: bee.pattern
title: live-proof-evidence cell lpe-2 — pitfall candidate
description: "Pitfall candidate mined from cell lpe-2's capped trace: Included .bee/onboarding.json in the commit although the cell did not list it — bee onboard --apply inside the ordered regen chain rewrote its updated_at, and …"
timestamp: 2026-09-22
bee:
  id: live-proof-evidence-lpe-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/lpe-2.json]
  polarity: pitfall
---

# live-proof-evidence cell lpe-2 — pitfall candidate

## What the cell did

Move verify-app evidence out of /tmp and name the live-proof locator rule in the map and doctrine

## Recorded evidence (verbatim from .bee/cells/lpe-2.json)

- **deviation** — Included .bee/onboarding.json in the commit although the cell did not list it — bee onboard --apply inside the ordered regen chain rewrote its updated_at, and both ways back to the old bytes are guard-refused, so the honest end state is to reserve the path and land it — hit an unforeseen obstacle
- **deviation** — Rewrapped the bee-swarming step 5 sentence over three lines — the replacement text pushed that line past the wrap width used in the file — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.