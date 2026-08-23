promote proposal for work item "tmux-herding-transport" (docs/history/tmux-herding-transport/CONTEXT.md + docs/history/tmux-herding-transport/plan.md) — 6 capped cell(s): tht-1, tht-2, tht-3, tht-4, tht-5, tht-6
anchor: history — docs/history/tmux-herding-transport/CONTEXT.md, docs/history/tmux-herding-transport/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/tmux-herding-transport/delivery.md

---
type: bee.delivery
title: tmux-herding-transport — delivery
description: "Delivery record proposed by bee knowledge promote for work item tmux-herding-transport: 6 capped cell(s), 19 recorded deviation(s)."
timestamp: 2026-08-22
bee:
  id: tmux-herding-transport-delivery
  lifecycle: active
  areas: [bee-herding, hook-runtime]
  required_context: [docs/history/tmux-herding-transport/CONTEXT.md, docs/history/tmux-herding-transport/plan.md]
  sources: [docs/history/tmux-herding-transport/CONTEXT.md, docs/history/tmux-herding-transport/plan.md, .bee/cells/tht-1.json, .bee/cells/tht-2.json, .bee/cells/tht-3.json, .bee/cells/tht-4.json, .bee/cells/tht-5.json, .bee/cells/tht-6.json]
---

# tmux-herding-transport — Delivery

## What shipped

- **tht-1** — herding.transport selects the probe; both probes gained a tmux arm and transport.kind (3 file(s) changed)
- **tht-2** — run's private Herdr trait is now pub(crate) PaneTransport; Liveness and PaneGeom pub(crate); behavior unchanged (1 file(s) changed)
- **tht-3** — RealTmux implements PaneTransport over tmux verbs with a screen classifier and stub-tmux tests (2 file(s) changed)
- **tht-4** — bee herding run selects RealHerdr or RealTmux from herding.transport, refuses an illegal value before any side effect, and names the transport in dry-run JSON (2 file(s) changed)
- **tht-5** — Record the tmux pane id in the activity record (2 file(s) changed)
- **tht-6** — tmux transport documented: herding.transport plus every herding.tmux.* knob in the invariants reference and the annotated sample, a D1-D4 Transport section with the D5 source on the run-verb page, and the herdr-cli dependency reason split by transport (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **tht-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee transport_`
- **tht-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run`
- **tht-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::tmux`
- **tht-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run`
- **tht-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee activity`
- **tht-6** — `.bee/bin/bee knowledge check && .bee/bin/bee knowledge index --check && .bee/bin/bee dev release-manifest --check`

## Deviations

- **tht-1** — Added #[allow(dead_code)] to the two kept herdr-only delegators: production now calls the _for siblings, so the bin build warned. Signatures and existing tests untouched.
- **tht-1** — Pre-existing red at base, untouched by this cell: crates/bee/tests/opencode_plugin_contracts.rs two failures (activity PreToolUse matcher, opencode invalid-tool anchor) — confirmed failing with my changes stashed.
- **tht-1** — sync-ack: The cell's files list carries no skill path: phase 1 of the plan puts the docs/skill/knowledge sync in its own later cell, and this cell is the config key plus the two probe arms only.
- **tht-2** — PaneGeom fields also made pub(crate): a pub(crate) type a sibling module must construct needs constructible fields
- **tht-2** — Renamed Herdr -> PaneTransport inside comments and PanicHerdr panic strings that name the trait, and trimmed 8 box-drawing chars off the seam banner to keep column alignment; no method name, argv or doc-comment content changed
- **tht-2** — Commit ba3cd40 is empty: a concurrent worker (cell tht-5) git-added this file into commit c9f063bd before this cell committed; c9f063bd carries the exact rename diff (verified line by line), ba3cd40 records the cell trailer
- **tht-2** — sync-ack: Behavior-neutral rename of a crate-private trait; no herdr verb, no user-visible behavior and no documented contract changed, so skills/bee-herding/* has nothing to sync
- **tht-3** — process_info format gained a leading #{pane_id} field: list-panes -t <pane> resolves to the pane WINDOW and lists every pane in it, so the cell's three-field format could not tell the target row from a sibling worker's
- **tht-3** — agent_prompt preflights the pane and refuses a blocked screen before typing — the cell did not name it, D3's prohibition (no key into a dialog) requires it
- **tht-3** — pane_split falls open to tmux's default even split when the parent geometry cannot be read, rather than failing the spawn
- **tht-3** — sync-ack: skills/bee-herding docs land in tht-6, the slice's dedicated sync cell; this cell adds an unwired module only. In-module tests are required, not skipped: crates/bee is a binary crate with no lib target, so no tests/ path can reach these items.
- **tht-4** — Reserved packages/bee-rs/crates/bee/src/herding/tmux.rs (cell-sanctioned) for the one-line name() override
- **tht-4** — Extracted read_main_config from execute_new so run() and execute_new share one config read instead of two copies
- **tht-4** — emit_result gained a transport param (one caller); the dry-run transport key is asserted through select_transport().name() rather than by capturing stdout, which emit_result writes directly
- **tht-4** — Capped with --sync-ack: skills/bee-herding docs land in tht-6
- **tht-4** — sync-ack: skills/bee-herding docs for the transport key land in tht-6, the feature's dedicated docs cell
- **tht-5** — sync-ack: hook-runtime identity field, not a herding gesture: the cell declares affects_skills [] and no bee-herding skill instruction changes; the activity-record knowledge page is updated in this cell
- **tht-6** — bee dev regen rewrote 15 paths outside the cell files list (the five rendered skill trees plus .bee/onboarding.json timestamp); reserved each under w-tht-6 before committing them with the cell
- **tht-6** — added tmux-herding-transport D1-D4 to the run-verb page frontmatter decisions as well as D5 to sources — the new section states those four decisions, so the page must declare them

## Provenance

Proposed by `bee knowledge promote --work tmux-herding-transport` from 6 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/tmux-herding-transport/CONTEXT.md`, `docs/history/tmux-herding-transport/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "tmux-herding-transport" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-22T17:21:02.536Z), the work item declares no bee.areas.

area bee-herding:
  - [tht-1] herding.transport selects the probe; both probes gained a tmux arm and transport.kind — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/tht-1.json)
  - [tht-4] bee herding run selects RealHerdr or RealTmux from herding.transport, refuses an illegal value before any side effect, and names the transport in dry-run JSON — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/tht-4.json)
  - [tht-5] Record the tmux pane id in the activity record — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/tht-5.json)

area hook-runtime:
  - [tht-1] herding.transport selects the probe; both probes gained a tmux arm and transport.kind — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/tht-1.json)
  - [tht-4] bee herding run selects RealHerdr or RealTmux from herding.transport, refuses an illegal value before any side effect, and names the transport in dry-run JSON — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/tht-4.json)
  - [tht-5] Record the tmux pane id in the activity record — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/tht-5.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell tht-1 — save as docs/knowledge/patterns/tmux-herding-transport-tht-1-pitfall.md

---
type: bee.pattern
title: tmux-herding-transport cell tht-1 — pitfall candidate
description: "Pitfall candidate mined from cell tht-1's capped trace: Added #[allow(dead_code)] to the two kept herdr-only delegators: production now calls the _for siblings, so the bin build warned. Signatures and existing tests…"
timestamp: 2026-08-22
bee:
  id: tmux-herding-transport-tht-1-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/tht-1.json]
  polarity: pitfall
---

# tmux-herding-transport cell tht-1 — pitfall candidate

## What the cell did

herding.transport selects the probe; both probes gained a tmux arm and transport.kind

## Recorded evidence (verbatim from .bee/cells/tht-1.json)

- **deviation** — Added #[allow(dead_code)] to the two kept herdr-only delegators: production now calls the _for siblings, so the bin build warned. Signatures and existing tests untouched.
- **deviation** — Pre-existing red at base, untouched by this cell: crates/bee/tests/opencode_plugin_contracts.rs two failures (activity PreToolUse matcher, opencode invalid-tool anchor) — confirmed failing with my changes stashed.
- **deviation** — sync-ack: The cell's files list carries no skill path: phase 1 of the plan puts the docs/skill/knowledge sync in its own later cell, and this cell is the config key plus the two probe arms only.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell tht-2 — save as docs/knowledge/patterns/tmux-herding-transport-tht-2-pitfall.md

---
type: bee.pattern
title: tmux-herding-transport cell tht-2 — pitfall candidate
description: "Pitfall candidate mined from cell tht-2's capped trace: PaneGeom fields also made pub(crate): a pub(crate) type a sibling module must construct needs constructible fields"
timestamp: 2026-08-22
bee:
  id: tmux-herding-transport-tht-2-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/tht-2.json]
  polarity: pitfall
---

# tmux-herding-transport cell tht-2 — pitfall candidate

## What the cell did

run's private Herdr trait is now pub(crate) PaneTransport; Liveness and PaneGeom pub(crate); behavior unchanged

## Recorded evidence (verbatim from .bee/cells/tht-2.json)

- **deviation** — PaneGeom fields also made pub(crate): a pub(crate) type a sibling module must construct needs constructible fields
- **deviation** — Renamed Herdr -> PaneTransport inside comments and PanicHerdr panic strings that name the trait, and trimmed 8 box-drawing chars off the seam banner to keep column alignment; no method name, argv or doc-comment content changed
- **deviation** — Commit ba3cd40 is empty: a concurrent worker (cell tht-5) git-added this file into commit c9f063bd before this cell committed; c9f063bd carries the exact rename diff (verified line by line), ba3cd40 records the cell trailer
- **deviation** — sync-ack: Behavior-neutral rename of a crate-private trait; no herdr verb, no user-visible behavior and no documented contract changed, so skills/bee-herding/* has nothing to sync

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell tht-3 — save as docs/knowledge/patterns/tmux-herding-transport-tht-3-pitfall.md

---
type: bee.pattern
title: tmux-herding-transport cell tht-3 — pitfall candidate
description: "Pitfall candidate mined from cell tht-3's capped trace: process_info format gained a leading #{pane_id} field: list-panes -t <pane> resolves to the pane WINDOW and lists every pane in it, so the cell's three-field f…"
timestamp: 2026-08-22
bee:
  id: tmux-herding-transport-tht-3-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/tht-3.json]
  polarity: pitfall
---

# tmux-herding-transport cell tht-3 — pitfall candidate

## What the cell did

RealTmux implements PaneTransport over tmux verbs with a screen classifier and stub-tmux tests

## Recorded evidence (verbatim from .bee/cells/tht-3.json)

- **deviation** — process_info format gained a leading #{pane_id} field: list-panes -t <pane> resolves to the pane WINDOW and lists every pane in it, so the cell's three-field format could not tell the target row from a sibling worker's
- **deviation** — agent_prompt preflights the pane and refuses a blocked screen before typing — the cell did not name it, D3's prohibition (no key into a dialog) requires it
- **deviation** — pane_split falls open to tmux's default even split when the parent geometry cannot be read, rather than failing the spawn
- **deviation** — sync-ack: skills/bee-herding docs land in tht-6, the slice's dedicated sync cell; this cell adds an unwired module only. In-module tests are required, not skipped: crates/bee is a binary crate with no lib target, so no tests/ path can reach these items.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell tht-4 — save as docs/knowledge/patterns/tmux-herding-transport-tht-4-pitfall.md

---
type: bee.pattern
title: tmux-herding-transport cell tht-4 — pitfall candidate
description: "Pitfall candidate mined from cell tht-4's capped trace: Reserved packages/bee-rs/crates/bee/src/herding/tmux.rs (cell-sanctioned) for the one-line name() override"
timestamp: 2026-08-22
bee:
  id: tmux-herding-transport-tht-4-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/tht-4.json]
  polarity: pitfall
---

# tmux-herding-transport cell tht-4 — pitfall candidate

## What the cell did

bee herding run selects RealHerdr or RealTmux from herding.transport, refuses an illegal value before any side effect, and names the transport in dry-run JSON

## Recorded evidence (verbatim from .bee/cells/tht-4.json)

- **deviation** — Reserved packages/bee-rs/crates/bee/src/herding/tmux.rs (cell-sanctioned) for the one-line name() override
- **deviation** — Extracted read_main_config from execute_new so run() and execute_new share one config read instead of two copies
- **deviation** — emit_result gained a transport param (one caller); the dry-run transport key is asserted through select_transport().name() rather than by capturing stdout, which emit_result writes directly
- **deviation** — Capped with --sync-ack: skills/bee-herding docs land in tht-6
- **deviation** — sync-ack: skills/bee-herding docs for the transport key land in tht-6, the feature's dedicated docs cell

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell tht-5 — save as docs/knowledge/patterns/tmux-herding-transport-tht-5-pitfall.md

---
type: bee.pattern
title: tmux-herding-transport cell tht-5 — pitfall candidate
description: "Pitfall candidate mined from cell tht-5's capped trace: sync-ack: hook-runtime identity field, not a herding gesture: the cell declares affects_skills [] and no bee-herding skill instruction changes; the activity-re…"
timestamp: 2026-08-22
bee:
  id: tmux-herding-transport-tht-5-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/tht-5.json]
  polarity: pitfall
---

# tmux-herding-transport cell tht-5 — pitfall candidate

## What the cell did

Record the tmux pane id in the activity record

## Recorded evidence (verbatim from .bee/cells/tht-5.json)

- **deviation** — sync-ack: hook-runtime identity field, not a herding gesture: the cell declares affects_skills [] and no bee-herding skill instruction changes; the activity-record knowledge page is updated in this cell

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell tht-6 — save as docs/knowledge/patterns/tmux-herding-transport-tht-6-pitfall.md

---
type: bee.pattern
title: tmux-herding-transport cell tht-6 — pitfall candidate
description: "Pitfall candidate mined from cell tht-6's capped trace: bee dev regen rewrote 15 paths outside the cell files list (the five rendered skill trees plus .bee/onboarding.json timestamp); reserved each under w-tht-6 bef…"
timestamp: 2026-08-22
bee:
  id: tmux-herding-transport-tht-6-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/tht-6.json]
  polarity: pitfall
---

# tmux-herding-transport cell tht-6 — pitfall candidate

## What the cell did

tmux transport documented: herding.transport plus every herding.tmux.* knob in the invariants reference and the annotated sample, a D1-D4 Transport section with the D5 source on the run-verb page, and the herdr-cli dependency reason split by transport

## Recorded evidence (verbatim from .bee/cells/tht-6.json)

- **deviation** — bee dev regen rewrote 15 paths outside the cell files list (the five rendered skill trees plus .bee/onboarding.json timestamp); reserved each under w-tht-6 before committing them with the cell
- **deviation** — added tmux-herding-transport D1-D4 to the run-verb page frontmatter decisions as well as D5 to sources — the new section states those four decisions, so the page must declare them

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 6 capped cell(s) mined, 1 delivery draft, 6 area bullet(s), 6 pattern candidate(s), 0 file(s) written.