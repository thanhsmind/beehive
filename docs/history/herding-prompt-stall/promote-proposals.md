promote proposal for work item "herding-prompt-stall" (docs/history/herding-prompt-stall/CONTEXT.md + docs/history/herding-prompt-stall/plan.md) — 16 capped cell(s): hps-1, hps-2, hps-3, hps-4, hps-5, hps-6, hps-7, hps-8, hps-9, hps-10, hps-11, hps-12, hps-13, hps-14, hps-15, hps-16
anchor: history — docs/history/herding-prompt-stall/CONTEXT.md, docs/history/herding-prompt-stall/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-prompt-stall/delivery.md

---
type: bee.delivery
title: herding-prompt-stall — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-prompt-stall: 16 capped cell(s), 6 recorded deviation(s)."
timestamp: 2026-08-21
bee:
  id: herding-prompt-stall-delivery
  lifecycle: active
  required_context: [docs/history/herding-prompt-stall/CONTEXT.md, docs/history/herding-prompt-stall/plan.md]
  sources: [docs/history/herding-prompt-stall/CONTEXT.md, docs/history/herding-prompt-stall/plan.md, .bee/cells/hps-1.json, .bee/cells/hps-2.json, .bee/cells/hps-3.json, .bee/cells/hps-4.json, .bee/cells/hps-5.json, .bee/cells/hps-6.json, .bee/cells/hps-7.json, .bee/cells/hps-8.json, .bee/cells/hps-9.json, .bee/cells/hps-10.json, .bee/cells/hps-11.json, .bee/cells/hps-12.json, .bee/cells/hps-13.json, .bee/cells/hps-14.json, .bee/cells/hps-15.json, .bee/cells/hps-16.json]
---

# herding-prompt-stall — Delivery

## What shipped

- **hps-1** — run.rs reads herdr lifecycle state per herdr's contract: idle-or-done ready gate, one herdr-observed pointer submit whose agent_prompt_stalled is the failure, blocked fails fast at all three wait points (1 file(s) changed)
- **hps-2** — Herding knowledge and skill reference record herdr's lifecycle contract and retire the idle-only gate and transition receipt (2 file(s) changed)
- **hps-3** — The worker-written ack file is the delivery receipt; deliver_pointer confirms on ack or result, never on a herdr lifecycle state, and the ack counts as a heartbeat (2 file(s) changed)
- **hps-4** — The brief-delivery knowledge page states the ack receipt and the stall/blocked failure detectors instead of the transition receipt (1 file(s) changed)
- **hps-5** — The knowledge pages, the bee-herding invariants and the README now teach the shipped delivery shape instead of the three retired rules, each retirement naming its superseding decision (5 file(s) changed)
- **hps-6** — deliver_pointer polls a working agent with exactly one send and resends only on a ready-with-no-ack transition, bounded by a resend count and a separate 180s wall-clock ack budget (1 file(s) changed)
- **hps-7** — A wait that gives up reads the pane and, when the text shows a confirmation cue, names the unanswered prompt and the remedy instead of a bare timeout (1 file(s) changed)
- **hps-8** — A herding.agents entry can declare a foreign tool's workspace-trust store, and bee appends the run's cwd to it before the pane split, fail-open (4 file(s) changed)
- **hps-9** — The prompt-diagnosis give-up message carries the same pane tail as the blocked message, and the config sample stops teaching the retired pointer-echo and resend rules (2 file(s) changed)
- **hps-10** — RealHerdr::agent_wait reads result.agent.agent_status through a pure extractor, with parse-level tests pinned to captured live herdr replies (1 file(s) changed)
- **hps-11** — The herdr prompt wait outlives herdr's own five-second stall window, and a herdr timeout reply falls through to the ack poll instead of failing the run (1 file(s) changed)
- **hps-12** — A worker pane is split off the roomiest pane in the caller's tab, and a parent too narrow for an agent TUI is refused before anything is created (1 file(s) changed)
- **hps-13** — The width guard moves to the child pane, and a tab with no room left yields a fresh tab's root pane instead of a sliver or a refusal (1 file(s) changed)
- **hps-14** — A stalled submission is retried under the existing bounds instead of ending the run, with a distinct terminal error when every send stalls (1 file(s) changed)
- **hps-15** — The four herding capture stubs are merged into their area specs and the faked-seam pattern is promoted with its live evidence and an applicable check (4 file(s) changed)
- **hps-16** — The faked-seam rule now lives in the craft layer onboarding copies into every project, stated without any project-specific vocabulary (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hps-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hps-2** — `rg -n 'idle-only|transition into working|idle only' docs/knowledge/areas/bee-herding/ skills/bee-herding/references/operational-invariants.md; rg -n 'blocked|agent_prompt_stalled|done' skills/bee-herding/references/operational-invariants.md`
- **hps-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hps-4** — `rg -n 'idle-only|idle only|transition into working|TRANSITION off a per-send baseline' docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md; rg -n 'ack-|agent_prompt_stalled|blocked' docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md`
- **hps-5** — `rg -n 'state transition the agent itself caused|resend ceiling|state-receipt delivery|Five states|which never marks a tab seen|re-sent a fixed number of times' docs/knowledge/areas/bee-herding/ skills/bee-herding/references/operational-invariants.md README.md`
- **hps-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hps-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hps-8** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hps-9** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hps-10** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hps-11** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hps-12** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hps-13** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hps-14** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hps-15** — `rg -n 'trust|blocked|confirmation cue' docs/knowledge/areas/bee-herding/agent-resolution-and-spawn-commands.md docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md; rg -n 'commit' docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md; ls docs/knowledge/patterns/20260821-a-faked-seam-hides-the-parse.md; .bee/bin/bee capture list`
- **hps-16** — `rg -n 'faked seam|crosses a process boundary|captured|pure function over a string' .bee/expertise/tests.md; rg -c '^## ' .bee/expertise/tests.md`

## Deviations

- **hps-1** — bee dispatch prepare resolved the transport to --agent agy-flash; pinned claude-sonnet instead because this cell's subject IS the defect in the agy dispatch path and running the repair through the path under repair risks losing the work. Transport shape, channel and brief were prepare's, verbatim.
- **hps-3** — The worker never committed - it returned a result with no commit field at all. The orchestrator made the path-scoped commit 0418c68 with the required 'cell: hps-3' trailer. Second instance this wave of a herding worker not carrying bee's commit bookkeeping.
- **hps-5** — The worker reported that docs/history/herding-prompt-stall/CONTEXT.md, named in its read_first, does not exist in the worktree - it was written in the main checkout and the branch predates it. Every worker this wave has been working without the feature's authority document, from decisions.jsonl and the code instead. Orchestrator defect, being repaired by bringing CONTEXT.md into the worktree.
- **hps-8** — The worker again returned no commit; the orchestrator made the path-scoped commit acd5945 with the required trailer. Fifth instance this wave.
- **hps-13** — The worker again returned no commit; the orchestrator made the path-scoped commit a365300 with the required trailer.
- **hps-15** — The worker merged all four stubs but did not run bee capture flush, reporting that its standalone-executor brief forbids running any bee command; it read .bee/capture-queue.jsonl directly instead. The orchestrator ran the four flushes. Worth noting: the brief tells a herding worker to run bee cells finish while also forbidding bee commands.

## Provenance

Proposed by `bee knowledge promote --work herding-prompt-stall` from 16 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-prompt-stall/CONTEXT.md`, `docs/history/herding-prompt-stall/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hps-1 — save as docs/knowledge/patterns/herding-prompt-stall-hps-1-pitfall.md

---
type: bee.pattern
title: herding-prompt-stall cell hps-1 — pitfall candidate
description: "Pitfall candidate mined from cell hps-1's capped trace: bee dispatch prepare resolved the transport to --agent agy-flash; pinned claude-sonnet instead because this cell's subject IS the defect in the agy dispatch pa…"
timestamp: 2026-08-21
bee:
  id: herding-prompt-stall-hps-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/hps-1.json]
  polarity: pitfall
---

# herding-prompt-stall cell hps-1 — pitfall candidate

## What the cell did

run.rs reads herdr lifecycle state per herdr's contract: idle-or-done ready gate, one herdr-observed pointer submit whose agent_prompt_stalled is the failure, blocked fails fast at all three wait points

## Recorded evidence (verbatim from .bee/cells/hps-1.json)

- **deviation** — bee dispatch prepare resolved the transport to --agent agy-flash; pinned claude-sonnet instead because this cell's subject IS the defect in the agy dispatch path and running the repair through the path under repair risks losing the work. Transport shape, channel and brief were prepare's, verbatim.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hps-3 — save as docs/knowledge/patterns/herding-prompt-stall-hps-3-pitfall.md

---
type: bee.pattern
title: herding-prompt-stall cell hps-3 — pitfall candidate
description: "Pitfall candidate mined from cell hps-3's capped trace: The worker never committed - it returned a result with no commit field at all. The orchestrator made the path-scoped commit 0418c68 with the required 'cell: hp…"
timestamp: 2026-08-21
bee:
  id: herding-prompt-stall-hps-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/hps-3.json]
  polarity: pitfall
---

# herding-prompt-stall cell hps-3 — pitfall candidate

## What the cell did

The worker-written ack file is the delivery receipt; deliver_pointer confirms on ack or result, never on a herdr lifecycle state, and the ack counts as a heartbeat

## Recorded evidence (verbatim from .bee/cells/hps-3.json)

- **deviation** — The worker never committed - it returned a result with no commit field at all. The orchestrator made the path-scoped commit 0418c68 with the required 'cell: hps-3' trailer. Second instance this wave of a herding worker not carrying bee's commit bookkeeping.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hps-5 — save as docs/knowledge/patterns/herding-prompt-stall-hps-5-pitfall.md

---
type: bee.pattern
title: herding-prompt-stall cell hps-5 — pitfall candidate
description: "Pitfall candidate mined from cell hps-5's capped trace: The worker reported that docs/history/herding-prompt-stall/CONTEXT.md, named in its read_first, does not exist in the worktree - it was written in the main che…"
timestamp: 2026-08-21
bee:
  id: herding-prompt-stall-hps-5-pitfall
  lifecycle: draft
  sources: [.bee/cells/hps-5.json]
  polarity: pitfall
---

# herding-prompt-stall cell hps-5 — pitfall candidate

## What the cell did

The knowledge pages, the bee-herding invariants and the README now teach the shipped delivery shape instead of the three retired rules, each retirement naming its superseding decision

## Recorded evidence (verbatim from .bee/cells/hps-5.json)

- **deviation** — The worker reported that docs/history/herding-prompt-stall/CONTEXT.md, named in its read_first, does not exist in the worktree - it was written in the main checkout and the branch predates it. Every worker this wave has been working without the feature's authority document, from decisions.jsonl and the code instead. Orchestrator defect, being repaired by bringing CONTEXT.md into the worktree.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hps-8 — save as docs/knowledge/patterns/herding-prompt-stall-hps-8-pitfall.md

---
type: bee.pattern
title: herding-prompt-stall cell hps-8 — pitfall candidate
description: "Pitfall candidate mined from cell hps-8's capped trace: The worker again returned no commit; the orchestrator made the path-scoped commit acd5945 with the required trailer. Fifth instance this wave."
timestamp: 2026-08-21
bee:
  id: herding-prompt-stall-hps-8-pitfall
  lifecycle: draft
  sources: [.bee/cells/hps-8.json]
  polarity: pitfall
---

# herding-prompt-stall cell hps-8 — pitfall candidate

## What the cell did

A herding.agents entry can declare a foreign tool's workspace-trust store, and bee appends the run's cwd to it before the pane split, fail-open

## Recorded evidence (verbatim from .bee/cells/hps-8.json)

- **deviation** — The worker again returned no commit; the orchestrator made the path-scoped commit acd5945 with the required trailer. Fifth instance this wave.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hps-13 — save as docs/knowledge/patterns/herding-prompt-stall-hps-13-pitfall.md

---
type: bee.pattern
title: herding-prompt-stall cell hps-13 — pitfall candidate
description: "Pitfall candidate mined from cell hps-13's capped trace: The worker again returned no commit; the orchestrator made the path-scoped commit a365300 with the required trailer."
timestamp: 2026-08-21
bee:
  id: herding-prompt-stall-hps-13-pitfall
  lifecycle: draft
  sources: [.bee/cells/hps-13.json]
  polarity: pitfall
---

# herding-prompt-stall cell hps-13 — pitfall candidate

## What the cell did

The width guard moves to the child pane, and a tab with no room left yields a fresh tab's root pane instead of a sliver or a refusal

## Recorded evidence (verbatim from .bee/cells/hps-13.json)

- **deviation** — The worker again returned no commit; the orchestrator made the path-scoped commit a365300 with the required trailer.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hps-15 — save as docs/knowledge/patterns/herding-prompt-stall-hps-15-pitfall.md

---
type: bee.pattern
title: herding-prompt-stall cell hps-15 — pitfall candidate
description: "Pitfall candidate mined from cell hps-15's capped trace: The worker merged all four stubs but did not run bee capture flush, reporting that its standalone-executor brief forbids running any bee command; it read .bee/…"
timestamp: 2026-08-21
bee:
  id: herding-prompt-stall-hps-15-pitfall
  lifecycle: draft
  sources: [.bee/cells/hps-15.json]
  polarity: pitfall
---

# herding-prompt-stall cell hps-15 — pitfall candidate

## What the cell did

The four herding capture stubs are merged into their area specs and the faked-seam pattern is promoted with its live evidence and an applicable check

## Recorded evidence (verbatim from .bee/cells/hps-15.json)

- **deviation** — The worker merged all four stubs but did not run bee capture flush, reporting that its standalone-executor brief forbids running any bee command; it read .bee/capture-queue.jsonl directly instead. The orchestrator ran the four flushes. Worth noting: the brief tells a herding worker to run bee cells finish while also forbidding bee commands.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 16 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 6 pattern candidate(s), 0 file(s) written.