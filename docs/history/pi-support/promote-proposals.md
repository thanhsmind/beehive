promote proposal for work item "pi-support" (docs/history/pi-support/CONTEXT.md + docs/history/pi-support/plan.md) — 4 capped cell(s): pis-1, pis-2, pis-3, pis-4
anchor: history — docs/history/pi-support/CONTEXT.md, docs/history/pi-support/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-support/delivery.md

---
type: bee.delivery
title: pi-support — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-support: 4 capped cell(s), 9 recorded deviation(s)."
timestamp: 2026-08-29
bee:
  id: pi-support-delivery
  lifecycle: active
  required_context: [docs/history/pi-support/CONTEXT.md, docs/history/pi-support/plan.md]
  sources: [docs/history/pi-support/CONTEXT.md, docs/history/pi-support/plan.md, .bee/cells/pis-1.json, .bee/cells/pis-2.json, .bee/cells/pis-3.json, .bee/cells/pis-4.json]
---

# pi-support — Delivery

## What shipped

- **pis-1** — Pi enforcement belt written: enumerated built-in tool map, fail-safe unknown-tool routing to write-guard, fail-closed blocking runner, never-throw advisory wrappers, per-call .bee-directory passivity, model-guard named exclusion (1 file(s) changed)
- **pis-2** — runtime pi is legal at the dispatch door, herding-only with typed refusals on every other resolution, and the belt ships through onboard plus the release inventory (13 file(s) changed)
- **pis-3** — Pi belt fixture suite lands and the belt parity test derives a fourth belt from the Pi source, with model-guard excluded by name (2 file(s) changed)
- **pis-4** — models.pi documented as a herding-only preview runtime across the config sample, the config reference, and the hook-runtime knowledge area (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pis-1** — `node --check .pi/extensions/bee-guard.ts`
- **pis-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee && .bee/bin/bee dev release-manifest --check`
- **pis-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts`
- **pis-4** — `rg -n "models.pi" .bee/config-sample.json docs/config-reference.md && rg -ni "pi-result-mailbox" docs/config-reference.md`

## Deviations

- **pis-1** — Mapped Pi edit to MultiEdit and translated edits[].oldText/newText into old_string/new_string instead of a bare Edit passthrough — Pi edit takes an array of replacements, which is MultiEdit shape, and bee treats Edit/Write/MultiEdit identically so the honest name costs nothing — found a better route
- **pis-1** — Unmapped tools carrying a string command route as Bash instead of Write — Bash is write-capable in bee terms too and a Write-shaped route would hide a custom shell tool command from the guard, a real bypass — hit an unforeseen obstacle
- **pis-1** — Blocking runner blocks on an updatedInput repair it cannot apply to a field-translated tool — plan.md said Pi has no documented input-mutation contract, but docs/extensions.md tool_call documents mutable event.input, so the repair channel exists and dropping it would run the call unrepaired — the plan was wrong about a fact
- **pis-2** — Reserved and edited two files the cell did not name — verbs/drivers/tests.rs and verbs/models_group.rs — because widening DISPATCH_RUNTIMES and RUNTIMES turned 12 existing tests red (walks over both constants); the driver walks now configure a herding models.pi table instead of skipping pi, so pi stays inside the derived matrix — something else had to be fixed first
- **pis-2** — Ran the release-manifest proof with the binary built from this cell (/home/thanhsmind/.cache/cargo-target/release/bee) instead of the cell verify line's .bee/bin/bee, which is a symlink to the main checkout's binary built before this change and cannot know the new .pi/extensions root — the plan was wrong about a fact
- **pis-2** — Wired the wave door to unwind its own claim on a pi refusal rather than pushing an ok:false payload carrying claimed:true, so a refused pi wave leaks no claims — found a better route
- **pis-3** — Bounded the mapToolCall switch slice at the function's closing brace in both parsers — unbounded it ran on into sessionSource's switch (reason) and read case "new"/"reload" as routed tool names — hit an unforeseen obstacle
- **pis-3** — Built the linked-worktree fixture with a real git worktree add over an empty root commit instead of hand-writing the .git file and worktrees/<name> pair — the hand-built layout did not resolve through git rev-parse, so the belt found no store — the plan was wrong about a fact
- **pis-4** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work pi-support` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-support/CONTEXT.md`, `docs/history/pi-support/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pis-1 — save as docs/knowledge/patterns/pi-support-pis-1-pitfall.md

---
type: bee.pattern
title: pi-support cell pis-1 — pitfall candidate
description: "Pitfall candidate mined from cell pis-1's capped trace: Mapped Pi edit to MultiEdit and translated edits[].oldText/newText into old_string/new_string instead of a bare Edit passthrough — Pi edit takes an array of re…"
timestamp: 2026-08-29
bee:
  id: pi-support-pis-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/pis-1.json]
  polarity: pitfall
---

# pi-support cell pis-1 — pitfall candidate

## What the cell did

Pi enforcement belt written: enumerated built-in tool map, fail-safe unknown-tool routing to write-guard, fail-closed blocking runner, never-throw advisory wrappers, per-call .bee-directory passivity, model-guard named exclusion

## Recorded evidence (verbatim from .bee/cells/pis-1.json)

- **deviation** — Mapped Pi edit to MultiEdit and translated edits[].oldText/newText into old_string/new_string instead of a bare Edit passthrough — Pi edit takes an array of replacements, which is MultiEdit shape, and bee treats Edit/Write/MultiEdit identically so the honest name costs nothing — found a better route
- **deviation** — Unmapped tools carrying a string command route as Bash instead of Write — Bash is write-capable in bee terms too and a Write-shaped route would hide a custom shell tool command from the guard, a real bypass — hit an unforeseen obstacle
- **deviation** — Blocking runner blocks on an updatedInput repair it cannot apply to a field-translated tool — plan.md said Pi has no documented input-mutation contract, but docs/extensions.md tool_call documents mutable event.input, so the repair channel exists and dropping it would run the call unrepaired — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pis-2 — save as docs/knowledge/patterns/pi-support-pis-2-pitfall.md

---
type: bee.pattern
title: pi-support cell pis-2 — pitfall candidate
description: "Pitfall candidate mined from cell pis-2's capped trace: Reserved and edited two files the cell did not name — verbs/drivers/tests.rs and verbs/models_group.rs — because widening DISPATCH_RUNTIMES and RUNTIMES turned…"
timestamp: 2026-08-29
bee:
  id: pi-support-pis-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/pis-2.json]
  polarity: pitfall
---

# pi-support cell pis-2 — pitfall candidate

## What the cell did

runtime pi is legal at the dispatch door, herding-only with typed refusals on every other resolution, and the belt ships through onboard plus the release inventory

## Recorded evidence (verbatim from .bee/cells/pis-2.json)

- **deviation** — Reserved and edited two files the cell did not name — verbs/drivers/tests.rs and verbs/models_group.rs — because widening DISPATCH_RUNTIMES and RUNTIMES turned 12 existing tests red (walks over both constants); the driver walks now configure a herding models.pi table instead of skipping pi, so pi stays inside the derived matrix — something else had to be fixed first
- **deviation** — Ran the release-manifest proof with the binary built from this cell (/home/thanhsmind/.cache/cargo-target/release/bee) instead of the cell verify line's .bee/bin/bee, which is a symlink to the main checkout's binary built before this change and cannot know the new .pi/extensions root — the plan was wrong about a fact
- **deviation** — Wired the wave door to unwind its own claim on a pi refusal rather than pushing an ok:false payload carrying claimed:true, so a refused pi wave leaks no claims — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pis-3 — save as docs/knowledge/patterns/pi-support-pis-3-pitfall.md

---
type: bee.pattern
title: pi-support cell pis-3 — pitfall candidate
description: "Pitfall candidate mined from cell pis-3's capped trace: Bounded the mapToolCall switch slice at the function's closing brace in both parsers — unbounded it ran on into sessionSource's switch (reason) and read case \"…"
timestamp: 2026-08-29
bee:
  id: pi-support-pis-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/pis-3.json]
  polarity: pitfall
---

# pi-support cell pis-3 — pitfall candidate

## What the cell did

Pi belt fixture suite lands and the belt parity test derives a fourth belt from the Pi source, with model-guard excluded by name

## Recorded evidence (verbatim from .bee/cells/pis-3.json)

- **deviation** — Bounded the mapToolCall switch slice at the function's closing brace in both parsers — unbounded it ran on into sessionSource's switch (reason) and read case "new"/"reload" as routed tool names — hit an unforeseen obstacle
- **deviation** — Built the linked-worktree fixture with a real git worktree add over an empty root commit instead of hand-writing the .git file and worktrees/<name> pair — the hand-built layout did not resolve through git rev-parse, so the belt found no store — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pis-4 — save as docs/knowledge/patterns/pi-support-pis-4-pitfall.md

---
type: bee.pattern
title: pi-support cell pis-4 — pitfall candidate
description: "Pitfall candidate mined from cell pis-4's capped trace: followed the plan"
timestamp: 2026-08-29
bee:
  id: pi-support-pis-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/pis-4.json]
  polarity: pitfall
---

# pi-support cell pis-4 — pitfall candidate

## What the cell did

models.pi documented as a herding-only preview runtime across the config sample, the config reference, and the hook-runtime knowledge area

## Recorded evidence (verbatim from .bee/cells/pis-4.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 4 pattern candidate(s), 0 file(s) written.