promote proposal for work item "lean-context" (docs/history/lean-context/CONTEXT.md + docs/history/lean-context/plan.md) — 2 capped cell(s): lnc-1, lnc-2
anchor: history — docs/history/lean-context/CONTEXT.md, docs/history/lean-context/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/lean-context/delivery.md

---
type: bee.delivery
title: lean-context — delivery
description: "Delivery record proposed by bee knowledge promote for work item lean-context: 2 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-11-04
bee:
  id: lean-context-delivery
  lifecycle: active
  areas: [hook-runtime, doctrine-layer]
  required_context: [docs/history/lean-context/CONTEXT.md, docs/history/lean-context/plan.md]
  sources: [docs/history/lean-context/CONTEXT.md, docs/history/lean-context/plan.md, .bee/cells/lnc-1.json, .bee/cells/lnc-2.json]
---

# lean-context — Delivery

## What shipped

- **lnc-1** — The prompt-context hook adds one handoff line when the latest request context reaches handoff_context_tokens (default 150000, 0 off) (2 file(s) changed)
- **lnc-2** — Handoff mark moved to about 150k tokens or 65% with feature close as a handoff point; the user set the docs role to the herding flash pane in main (9 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **lnc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- prompt_context statusline` — touched prompt_context.rs; leader re-ran it: 66 and 7 passed, 0 failed, including 4 new handoff tests (past, below, zero, missing or unreadable transcript)
- **lnc-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- agents_md doctrine parity && .bee/bin/bee dev release-manifest --check` — doctrine prose and regen parity only; 7 tests passed, manifest 599 files match

## Deviations

- **lnc-1** — statusline.rs needed no change: context_and_turns was already pub(crate)
- **lnc-1** — execute split into execute_to so tests capture the injected output
- **lnc-1** — sync-ack: skills/bee-hive/SKILL.md already names the 150k handoff mark through sibling cell lnc-2 in this feature
- **lnc-2** — bee dev regen also rewrote the rendered skill trees and .bee/onboarding.json; committed as regen output
- **lnc-2** — path-scoped commit because the guard refused staging with 2 live workers
- **lnc-2** — team.claude.docs changed by the user in the main config to the herding pane paseo-pi-agy-flash, not sonnet; the config guard reserves role models for the user
- **lnc-2** — sync-ack: AGENTS.md changed only in the handoff line of rule agents-context-handoff-65; agents-bee-defect-report text is unchanged

## Provenance

Proposed by `bee knowledge promote --work lean-context` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/lean-context/CONTEXT.md`, `docs/history/lean-context/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "lean-context" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-04T07:01:08.182Z), the work item declares no bee.areas.

area hook-runtime:
  - [lnc-1] The prompt-context hook adds one handoff line when the latest request context reaches handoff_context_tokens (default 150000, 0 off) — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/lnc-1.json)

area doctrine-layer:
  - [lnc-1] The prompt-context hook adds one handoff line when the latest request context reaches handoff_context_tokens (default 150000, 0 off) — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/lnc-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell lnc-1 — save as docs/knowledge/patterns/lean-context-lnc-1-pitfall.md

---
type: bee.pattern
title: lean-context cell lnc-1 — pitfall candidate
description: "Pitfall candidate mined from cell lnc-1's capped trace: statusline.rs needed no change: context_and_turns was already pub(crate)"
timestamp: 2026-11-04
bee:
  id: lean-context-lnc-1-pitfall
  lifecycle: draft
  areas: [hook-runtime, doctrine-layer]
  sources: [.bee/cells/lnc-1.json]
  polarity: pitfall
---

# lean-context cell lnc-1 — pitfall candidate

## What the cell did

The prompt-context hook adds one handoff line when the latest request context reaches handoff_context_tokens (default 150000, 0 off)

## Recorded evidence (verbatim from .bee/cells/lnc-1.json)

- **deviation** — statusline.rs needed no change: context_and_turns was already pub(crate)
- **deviation** — execute split into execute_to so tests capture the injected output
- **deviation** — sync-ack: skills/bee-hive/SKILL.md already names the 150k handoff mark through sibling cell lnc-2 in this feature

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell lnc-2 — save as docs/knowledge/patterns/lean-context-lnc-2-pitfall.md

---
type: bee.pattern
title: lean-context cell lnc-2 — pitfall candidate
description: "Pitfall candidate mined from cell lnc-2's capped trace: bee dev regen also rewrote the rendered skill trees and .bee/onboarding.json; committed as regen output"
timestamp: 2026-11-04
bee:
  id: lean-context-lnc-2-pitfall
  lifecycle: draft
  areas: [hook-runtime, doctrine-layer]
  sources: [.bee/cells/lnc-2.json]
  polarity: pitfall
---

# lean-context cell lnc-2 — pitfall candidate

## What the cell did

Handoff mark moved to about 150k tokens or 65% with feature close as a handoff point; the user set the docs role to the herding flash pane in main

## Recorded evidence (verbatim from .bee/cells/lnc-2.json)

- **deviation** — bee dev regen also rewrote the rendered skill trees and .bee/onboarding.json; committed as regen output
- **deviation** — path-scoped commit because the guard refused staging with 2 live workers
- **deviation** — team.claude.docs changed by the user in the main config to the herding pane paseo-pi-agy-flash, not sonnet; the config guard reserves role models for the user
- **deviation** — sync-ack: AGENTS.md changed only in the handoff line of rule agents-context-handoff-65; agents-bee-defect-report text is unchanged

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 2 pattern candidate(s), 0 file(s) written.