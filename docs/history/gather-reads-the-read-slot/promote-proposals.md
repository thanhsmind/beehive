promote proposal for work item "gather-reads-the-read-slot" (docs/history/gather-reads-the-read-slot/CONTEXT.md + docs/history/gather-reads-the-read-slot/plan.md) — 3 capped cell(s): grrs-1, grrs-2, grrs-3
anchor: history — docs/history/gather-reads-the-read-slot/CONTEXT.md, docs/history/gather-reads-the-read-slot/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/gather-reads-the-read-slot/delivery.md

---
type: bee.delivery
title: gather-reads-the-read-slot — delivery
description: "Delivery record proposed by bee knowledge promote for work item gather-reads-the-read-slot: 3 capped cell(s), 9 recorded deviation(s)."
timestamp: 2026-09-02
bee:
  id: gather-reads-the-read-slot-delivery
  lifecycle: active
  required_context: [docs/history/gather-reads-the-read-slot/CONTEXT.md, docs/history/gather-reads-the-read-slot/plan.md]
  sources: [docs/history/gather-reads-the-read-slot/CONTEXT.md, docs/history/gather-reads-the-read-slot/plan.md, .bee/cells/archive/gather-reads-the-read-slot/grrs-1.json, .bee/cells/archive/gather-reads-the-read-slot/grrs-2.json, .bee/cells/archive/gather-reads-the-read-slot/grrs-3.json]
---

# gather-reads-the-read-slot — Delivery

## What shipped

- **grrs-1** — A default gather resolves the read slot, travels under the winning name, and keeps bee-gather; the guard mirrors the walk on both halves of the fallback contract (4 file(s) changed)
- **grrs-2** — bee-gather declares [read, generation]; every rendered agent description opens by naming bee dispatch prepare (17 file(s) changed)
- **grrs-3** — dispatch.md, workers.md and doctrine now say a role-less gather resolves [read, generation] and travels under the winner (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **grrs-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml prepare model_guard drivers`
- **grrs-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml onboard status_full agents_block_render_parity && .bee/bin/bee dev release-manifest --check`
- **grrs-3** — `rg -n 'B14 — A gather asks for the read job' docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md && rg -n 'read' docs/product-description/delegation/workers.md | rg -q 'bee-gather'`

## Deviations

- **grrs-1** — The guard's fallback default is keyed on the name PREPARE travels under (the winner on the gather kind, the asked head elsewhere) instead of the winner on every kind — a winner-on-every-kind lookup makes the guard NARROWER than prepare on a null-review + herding-generation host, where prepare publishes opus off marker_role and the existing test an_explicitly_null_review_slot_inherits_the_generation_herding_fallback pins that opus stays admitted — the plan was wrong about a fact
- **grrs-1** — The cell's verify command `cargo test ... prepare model_guard drivers` is rejected by cargo (`unexpected argument 'model_guard'`); ran it as `cargo test ... -- prepare model_guard drivers` so libtest takes all three filters, and then ran the whole crate suite as well — hit an unforeseen obstacle
- **grrs-1** — Renamed the pinned test every_dispatch_kind_keeps_its_slot to every_dispatch_kind_names_its_slot_explicitly — 'keeps its slot' became a lie for gather the moment the arm split, and a test name that misdescribes its own assertion is the next reader's wrong turn — found a better route
- **grrs-2** — .bee/onboarding.json was applied with the binary built from this source (/home/thanhsmind/.cache/cargo-target/release/bee onboard --apply) instead of the vendored .bee/bin/bee — the vendored binary is a symlink to main's pre-change build, so its regen writes the old opencode rendered_from map with no `read` key; running it would have committed a record that disagrees with the source of truth — hit an unforeseen obstacle
- **grrs-2** — Trimmed `at the same tier` out of bee-build's description clause about bee-gather — the cell's own change makes that claim false the moment bee-gather asks for the read slot first — something else had to be fixed first
- **grrs-2** — The regen chain rewrites docs/history/codex-harness-hardening/release-manifest.json, not the cell's listed packages/bee/release-manifest.json (no such file exists); reserved the real path under w-grrs-2 before the write — the plan was wrong about a fact
- **grrs-3** — The doctrine entry is B17, not the B14 the cell named — B14, B15/B15a and B16 already exist in model-roles-and-escalation.md (agent-model-unpin, lane-model-diversity, pi-support), and those numbers are cited from other docs, so renumbering was worse than taking the next free number; the cell verify string was adapted to B17 — the plan was wrong about a fact
- **grrs-3** — workers.md line 14 moved OUT of the blockquote instead of staying a quoted FIX line — grrs-1 owns the new guard bytes and this cell must not quote text that does not exist yet, so the FIX is described by shape — found a better route
- **grrs-3** — docs/knowledge/index.md was left untouched: it is a GENERATED file (bee knowledge index) and lists no B-entries by name, so the cell's conditional did not fire — followed the plan

## Provenance

Proposed by `bee knowledge promote --work gather-reads-the-read-slot` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/gather-reads-the-read-slot/CONTEXT.md`, `docs/history/gather-reads-the-read-slot/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell grrs-1 — save as docs/knowledge/patterns/gather-reads-the-read-slot-grrs-1-pitfall.md

---
type: bee.pattern
title: gather-reads-the-read-slot cell grrs-1 — pitfall candidate
description: "Pitfall candidate mined from cell grrs-1's capped trace: The guard's fallback default is keyed on the name PREPARE travels under (the winner on the gather kind, the asked head elsewhere) instead of the winner on ever…"
timestamp: 2026-09-02
bee:
  id: gather-reads-the-read-slot-grrs-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/gather-reads-the-read-slot/grrs-1.json]
  polarity: pitfall
---

# gather-reads-the-read-slot cell grrs-1 — pitfall candidate

## What the cell did

A default gather resolves the read slot, travels under the winning name, and keeps bee-gather; the guard mirrors the walk on both halves of the fallback contract

## Recorded evidence (verbatim from .bee/cells/archive/gather-reads-the-read-slot/grrs-1.json)

- **deviation** — The guard's fallback default is keyed on the name PREPARE travels under (the winner on the gather kind, the asked head elsewhere) instead of the winner on every kind — a winner-on-every-kind lookup makes the guard NARROWER than prepare on a null-review + herding-generation host, where prepare publishes opus off marker_role and the existing test an_explicitly_null_review_slot_inherits_the_generation_herding_fallback pins that opus stays admitted — the plan was wrong about a fact
- **deviation** — The cell's verify command `cargo test ... prepare model_guard drivers` is rejected by cargo (`unexpected argument 'model_guard'`); ran it as `cargo test ... -- prepare model_guard drivers` so libtest takes all three filters, and then ran the whole crate suite as well — hit an unforeseen obstacle
- **deviation** — Renamed the pinned test every_dispatch_kind_keeps_its_slot to every_dispatch_kind_names_its_slot_explicitly — 'keeps its slot' became a lie for gather the moment the arm split, and a test name that misdescribes its own assertion is the next reader's wrong turn — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell grrs-2 — save as docs/knowledge/patterns/gather-reads-the-read-slot-grrs-2-pitfall.md

---
type: bee.pattern
title: gather-reads-the-read-slot cell grrs-2 — pitfall candidate
description: "Pitfall candidate mined from cell grrs-2's capped trace: .bee/onboarding.json was applied with the binary built from this source (/home/thanhsmind/.cache/cargo-target/release/bee onboard --apply) instead of the vendo…"
timestamp: 2026-09-02
bee:
  id: gather-reads-the-read-slot-grrs-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/gather-reads-the-read-slot/grrs-2.json]
  polarity: pitfall
---

# gather-reads-the-read-slot cell grrs-2 — pitfall candidate

## What the cell did

bee-gather declares [read, generation]; every rendered agent description opens by naming bee dispatch prepare

## Recorded evidence (verbatim from .bee/cells/archive/gather-reads-the-read-slot/grrs-2.json)

- **deviation** — .bee/onboarding.json was applied with the binary built from this source (/home/thanhsmind/.cache/cargo-target/release/bee onboard --apply) instead of the vendored .bee/bin/bee — the vendored binary is a symlink to main's pre-change build, so its regen writes the old opencode rendered_from map with no `read` key; running it would have committed a record that disagrees with the source of truth — hit an unforeseen obstacle
- **deviation** — Trimmed `at the same tier` out of bee-build's description clause about bee-gather — the cell's own change makes that claim false the moment bee-gather asks for the read slot first — something else had to be fixed first
- **deviation** — The regen chain rewrites docs/history/codex-harness-hardening/release-manifest.json, not the cell's listed packages/bee/release-manifest.json (no such file exists); reserved the real path under w-grrs-2 before the write — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell grrs-3 — save as docs/knowledge/patterns/gather-reads-the-read-slot-grrs-3-pitfall.md

---
type: bee.pattern
title: gather-reads-the-read-slot cell grrs-3 — pitfall candidate
description: "Pitfall candidate mined from cell grrs-3's capped trace: The doctrine entry is B17, not the B14 the cell named — B14, B15/B15a and B16 already exist in model-roles-and-escalation.md (agent-model-unpin, lane-model-div…"
timestamp: 2026-09-02
bee:
  id: gather-reads-the-read-slot-grrs-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/gather-reads-the-read-slot/grrs-3.json]
  polarity: pitfall
---

# gather-reads-the-read-slot cell grrs-3 — pitfall candidate

## What the cell did

dispatch.md, workers.md and doctrine now say a role-less gather resolves [read, generation] and travels under the winner

## Recorded evidence (verbatim from .bee/cells/archive/gather-reads-the-read-slot/grrs-3.json)

- **deviation** — The doctrine entry is B17, not the B14 the cell named — B14, B15/B15a and B16 already exist in model-roles-and-escalation.md (agent-model-unpin, lane-model-diversity, pi-support), and those numbers are cited from other docs, so renumbering was worse than taking the next free number; the cell verify string was adapted to B17 — the plan was wrong about a fact
- **deviation** — workers.md line 14 moved OUT of the blockquote instead of staying a quoted FIX line — grrs-1 owns the new guard bytes and this cell must not quote text that does not exist yet, so the FIX is described by shape — found a better route
- **deviation** — docs/knowledge/index.md was left untouched: it is a GENERATED file (bee knowledge index) and lists no B-entries by name, so the cell's conditional did not fire — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 3 pattern candidate(s), 0 file(s) written.