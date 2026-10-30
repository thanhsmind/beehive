promote proposal for work item "design-os-checks" (docs/history/design-os-checks/CONTEXT.md + docs/history/design-os-checks/plan.md) — 3 capped cell(s): doc-1, doc-2, doc-3
anchor: history — docs/history/design-os-checks/CONTEXT.md, docs/history/design-os-checks/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/design-os-checks/delivery.md

---
type: bee.delivery
title: design-os-checks — delivery
description: "Delivery record proposed by bee knowledge promote for work item design-os-checks: 3 capped cell(s), 5 recorded deviation(s)."
timestamp: 2026-10-30
bee:
  id: design-os-checks-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/design-os-checks/CONTEXT.md, docs/history/design-os-checks/plan.md]
  sources: [docs/history/design-os-checks/CONTEXT.md, docs/history/design-os-checks/plan.md, .bee/cells/doc-1.json, .bee/cells/doc-2.json, .bee/cells/doc-3.json]
---

# design-os-checks — Delivery

## What shipped

- **doc-1** — design gate pins ease-design 0.6.2, counts only baseline families, lints and ratchets chart, diagram, motion and flow artifacts, and gives a pre-feature baseline one advisory line instead of a failure (1 file(s) changed)
- **doc-2** — DEFAULT_UI_GLOBS now counts flow.json and *.flow.json, so a flow-model-only cell in a host with design packs is refused at cap without bee design gate (2 file(s) changed)
- **doc-3** — Charts, motion, web and figma packs ship a contract.md rendered from ease-design 0.6.2 ui schema; the sync is at v0.6.2 (369b041); a suite test fails on any pack ui command in no contract (16 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **doc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml design_group` — 24 design_group tests pass (11 new, 9 seen red first on base); also drove the built bee design gate with BEE_DESIGN_OS_CLI at the real design-os 0.6.2 dist/cli.js on copied examples bar.html, er.html…
- **doc-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml design_rule` — the cell verify filter runs all 7 design_rule tests, including the new flow-model test and the no-pick test
- **doc-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml shipped_design && rg -q '"version": "0.6.2"' packages/bee/design/UPSTREAM.json && rg -q 'ease-design@0.6.2' packages/bee/design/bee-design/SKILL.md && ! rg -q a7ad5f1 packages/bee/design` — the cell verify: 3 shipped_design tests pass (the parity test was red first, listing every uncovered pack command), UPSTREAM.json 0.6.2, skill pin 0.6.2, no a7ad5f1; renderer exit 1 on a missing ref …

## Deviations

- **doc-1** — a missing baseline file counts every family the run returns and applies the D7 ratchet to artifacts in full, rather than the 0.6.1 family set — D9 and D12 protect an existing old baseline, and a project with no baseline has no old count to protect — found a better route
- **doc-1** — under --write with artifacts present the failure summary reads N checks failed instead of N pages failed — a failed artifact lint is not a page; with no artifacts the line is unchanged — found a better route
- **doc-2** — ran the verify with PATH set to the literal cargo bin dir instead of the CARGO_HOME expansion — the worktree guard refuses a runtime variable in a cargo command — hit an unforeseen obstacle
- **doc-2** — sync-ack: cell names no affected skills; the glob change is a design cap default whose spec home is the onboarding design-packs concept, synced by its own docs cell
- **doc-3** — The contract intro names the ease-design schema --json output instead of a backticked ui schema — the parity test flagged that intro in the charts and motion contracts, which do not list schema — found a better route

## Provenance

Proposed by `bee knowledge promote --work design-os-checks` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/design-os-checks/CONTEXT.md`, `docs/history/design-os-checks/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "design-os-checks" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-30T15:15:22.453Z), the work item declares no bee.areas.

area onboarding:
  - [doc-1] design gate pins ease-design 0.6.2, counts only baseline families, lints and ratchets chart, diagram, motion and flow artifacts, and gives a pre-feature baseline one advisory line instead of a failure — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/doc-1.json)
  - [doc-2] DEFAULT_UI_GLOBS now counts flow.json and *.flow.json, so a flow-model-only cell in a host with design packs is refused at cap without bee design gate — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/doc-2.json)
  - [doc-3] Charts, motion, web and figma packs ship a contract.md rendered from ease-design 0.6.2 ui schema; the sync is at v0.6.2 (369b041); a suite test fails on any pack ui command in no contract — feature-wide sync per the scribing stamp, 16 file(s) changed (trace .bee/cells/doc-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell doc-1 — save as docs/knowledge/patterns/design-os-checks-doc-1-pitfall.md

---
type: bee.pattern
title: design-os-checks cell doc-1 — pitfall candidate
description: "Pitfall candidate mined from cell doc-1's capped trace: a missing baseline file counts every family the run returns and applies the D7 ratchet to artifacts in full, rather than the 0.6.1 family set — D9 and D12 prot…"
timestamp: 2026-10-30
bee:
  id: design-os-checks-doc-1-pitfall
  lifecycle: draft
  areas: [onboarding]
  sources: [.bee/cells/doc-1.json]
  polarity: pitfall
---

# design-os-checks cell doc-1 — pitfall candidate

## What the cell did

design gate pins ease-design 0.6.2, counts only baseline families, lints and ratchets chart, diagram, motion and flow artifacts, and gives a pre-feature baseline one advisory line instead of a failure

## Recorded evidence (verbatim from .bee/cells/doc-1.json)

- **deviation** — a missing baseline file counts every family the run returns and applies the D7 ratchet to artifacts in full, rather than the 0.6.1 family set — D9 and D12 protect an existing old baseline, and a project with no baseline has no old count to protect — found a better route
- **deviation** — under --write with artifacts present the failure summary reads N checks failed instead of N pages failed — a failed artifact lint is not a page; with no artifacts the line is unchanged — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell doc-2 — save as docs/knowledge/patterns/design-os-checks-doc-2-pitfall.md

---
type: bee.pattern
title: design-os-checks cell doc-2 — pitfall candidate
description: "Pitfall candidate mined from cell doc-2's capped trace: ran the verify with PATH set to the literal cargo bin dir instead of the CARGO_HOME expansion — the worktree guard refuses a runtime variable in a cargo comman…"
timestamp: 2026-10-30
bee:
  id: design-os-checks-doc-2-pitfall
  lifecycle: draft
  areas: [onboarding]
  sources: [.bee/cells/doc-2.json]
  polarity: pitfall
---

# design-os-checks cell doc-2 — pitfall candidate

## What the cell did

DEFAULT_UI_GLOBS now counts flow.json and *.flow.json, so a flow-model-only cell in a host with design packs is refused at cap without bee design gate

## Recorded evidence (verbatim from .bee/cells/doc-2.json)

- **deviation** — ran the verify with PATH set to the literal cargo bin dir instead of the CARGO_HOME expansion — the worktree guard refuses a runtime variable in a cargo command — hit an unforeseen obstacle
- **deviation** — sync-ack: cell names no affected skills; the glob change is a design cap default whose spec home is the onboarding design-packs concept, synced by its own docs cell

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell doc-3 — save as docs/knowledge/patterns/design-os-checks-doc-3-pitfall.md

---
type: bee.pattern
title: design-os-checks cell doc-3 — pitfall candidate
description: "Pitfall candidate mined from cell doc-3's capped trace: The contract intro names the ease-design schema --json output instead of a backticked ui schema — the parity test flagged that intro in the charts and motion c…"
timestamp: 2026-10-30
bee:
  id: design-os-checks-doc-3-pitfall
  lifecycle: draft
  areas: [onboarding]
  sources: [.bee/cells/doc-3.json]
  polarity: pitfall
---

# design-os-checks cell doc-3 — pitfall candidate

## What the cell did

Charts, motion, web and figma packs ship a contract.md rendered from ease-design 0.6.2 ui schema; the sync is at v0.6.2 (369b041); a suite test fails on any pack ui command in no contract

## Recorded evidence (verbatim from .bee/cells/doc-3.json)

- **deviation** — The contract intro names the ease-design schema --json output instead of a backticked ui schema — the parity test flagged that intro in the charts and motion contracts, which do not list schema — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 3 area bullet(s), 3 pattern candidate(s), 0 file(s) written.