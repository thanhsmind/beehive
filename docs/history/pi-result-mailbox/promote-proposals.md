promote proposal for work item "pi-result-mailbox" (docs/history/pi-result-mailbox/CONTEXT.md + docs/history/pi-result-mailbox/plan.md) — 5 capped cell(s): prm-1, prm-2, prm-3, prm-4, prm-5
anchor: history — docs/history/pi-result-mailbox/CONTEXT.md, docs/history/pi-result-mailbox/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-result-mailbox/delivery.md

---
type: bee.delivery
title: pi-result-mailbox — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-result-mailbox: 5 capped cell(s), 11 recorded deviation(s)."
timestamp: 2026-08-29
bee:
  id: pi-result-mailbox-delivery
  lifecycle: active
  required_context: [docs/history/pi-result-mailbox/CONTEXT.md, docs/history/pi-result-mailbox/plan.md]
  sources: [docs/history/pi-result-mailbox/CONTEXT.md, docs/history/pi-result-mailbox/plan.md, .bee/cells/prm-1.json, .bee/cells/prm-2.json, .bee/cells/prm-3.json, .bee/cells/prm-4.json, .bee/cells/prm-5.json]
---

# pi-result-mailbox — Delivery

## What shipped

- **prm-1** — The report rides the mailbox as a path-only additive envelope key, and --inbox-session makes the detached fact a flag with a pre-spawn marker (2 file(s) changed)
- **prm-2** — The Pi belt drains its result inbox and injects a header-only result envelope with pi-peer claim discipline (1 file(s) changed)
- **prm-3** — Drain fixtures prove at-least-once delivery, header-only fences and a never-throw drain; the Pi harness is now timeout-bounded and records injections (1 file(s) changed)
- **prm-4** — Lifted the pi not-production caveat at every site with text naming the delivered result contract and its at-least-once limits (7 file(s) changed)
- **prm-5** — result-inbox joins the generated ignore block; the pi herding payload carries a detached_delivery instruction naming --inbox-session (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **prm-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee`
- **prm-2** — `node --check .pi/extensions/bee-guard.ts`
- **prm-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts`
- **prm-4** — `rg -c "pi-result-mailbox" docs/config-reference.md && rg -n "report_path" skills/bee-swarming/references/swarming-reference.md docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md`
- **prm-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee`

## Deviations

- **prm-1** — Named the .gitignore gap instead of fixing it — .bee/result-inbox/ needs an ignore line, but that block is generated from onboard/templates.rs, a file this cell does not name — hit an unforeseen obstacle
- **prm-1** — Wrote the marker in execute() rather than execute_new() alone, so a detached --continue round also leaves one — the plan named the spawn path only, and a continue with no marker loses its result the same way — found a better route
- **prm-1** — Added MailboxResult.report_note beside report_path — the plan names report_note as an envelope key, and the pure envelope builder needs a carrier for it — something else had to be fixed first
- **prm-1** — sync-ack: The plan puts every skills/docs/knowledge edit for this feature in prm-4 (docs role), which names skills/bee-swarming/references/swarming-reference.md and both bee-herding knowledge homes as its own targets; this code cell writes no skill so the two never race the same file.
- **prm-2** — added a status row to the fenced header — the plan named five rows (job id, cell id, summary, proof, report_path) and a blocked result read as done is exactly the orchestrator-judgment corruption this feature exists to stop; still a one-line header field, no body — kind: addition
- **prm-2** — parked the interval in a globalThis slot as well as a module variable — Pi /reload can hand this file a fresh module scope while the old interval is still armed, and two drains racing one inbox is a double-delivery source the cell did not anticipate — kind: robustness
- **prm-3** — prm-2 added no NEW pi.on event, so the never-throw hand list needed no new row — instead the list is lifted into never_throw_event_rows() and GATED by every_advisory_event_the_belt_registers_has_a_never_throw_row against the belt-derived pi.on set, so the next event added cannot skip its row — makes the cell request self-enforcing rather than one-time — improvement
- **prm-4** — Renamed the config-reference heading (dropped the (preview) suffix) and updated both in-file anchor links — a heading that still said preview was the same standing lie as the callout under it — form deviation, in-scope file
- **prm-4** — Cleared two extra preview mentions the cell action did not enumerate (.bee/config-sample.json herding note, catalog-projections-and-activation.md:144) — both inside cell-named files and both said the same retired thing — scope-completion, must_have no site still says not production
- **prm-4** — Committed with an explicit -- pathspec through the shared index guard, excluding prm-w3's in-flight tests/pi_plugin_contracts.rs — concurrency hygiene, not a scope change
- **prm-5** — added a local #[cfg(test)] mod detached_delivery_tests inside prepare.rs — the truth "the pi herding payload mentions --inbox-session" needed a pinning test and the file already hosts two local test modules; stayed inside the two named files — scope-preserving

## Provenance

Proposed by `bee knowledge promote --work pi-result-mailbox` from 5 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-result-mailbox/CONTEXT.md`, `docs/history/pi-result-mailbox/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell prm-1 — save as docs/knowledge/patterns/pi-result-mailbox-prm-1-pitfall.md

---
type: bee.pattern
title: pi-result-mailbox cell prm-1 — pitfall candidate
description: "Pitfall candidate mined from cell prm-1's capped trace: Named the .gitignore gap instead of fixing it — .bee/result-inbox/ needs an ignore line, but that block is generated from onboard/templates.rs, a file this cel…"
timestamp: 2026-08-29
bee:
  id: pi-result-mailbox-prm-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/prm-1.json]
  polarity: pitfall
---

# pi-result-mailbox cell prm-1 — pitfall candidate

## What the cell did

The report rides the mailbox as a path-only additive envelope key, and --inbox-session makes the detached fact a flag with a pre-spawn marker

## Recorded evidence (verbatim from .bee/cells/prm-1.json)

- **deviation** — Named the .gitignore gap instead of fixing it — .bee/result-inbox/ needs an ignore line, but that block is generated from onboard/templates.rs, a file this cell does not name — hit an unforeseen obstacle
- **deviation** — Wrote the marker in execute() rather than execute_new() alone, so a detached --continue round also leaves one — the plan named the spawn path only, and a continue with no marker loses its result the same way — found a better route
- **deviation** — Added MailboxResult.report_note beside report_path — the plan names report_note as an envelope key, and the pure envelope builder needs a carrier for it — something else had to be fixed first
- **deviation** — sync-ack: The plan puts every skills/docs/knowledge edit for this feature in prm-4 (docs role), which names skills/bee-swarming/references/swarming-reference.md and both bee-herding knowledge homes as its own targets; this code cell writes no skill so the two never race the same file.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell prm-2 — save as docs/knowledge/patterns/pi-result-mailbox-prm-2-pitfall.md

---
type: bee.pattern
title: pi-result-mailbox cell prm-2 — pitfall candidate
description: "Pitfall candidate mined from cell prm-2's capped trace: added a status row to the fenced header — the plan named five rows (job id, cell id, summary, proof, report_path) and a blocked result read as done is exactly …"
timestamp: 2026-08-29
bee:
  id: pi-result-mailbox-prm-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/prm-2.json]
  polarity: pitfall
---

# pi-result-mailbox cell prm-2 — pitfall candidate

## What the cell did

The Pi belt drains its result inbox and injects a header-only result envelope with pi-peer claim discipline

## Recorded evidence (verbatim from .bee/cells/prm-2.json)

- **deviation** — added a status row to the fenced header — the plan named five rows (job id, cell id, summary, proof, report_path) and a blocked result read as done is exactly the orchestrator-judgment corruption this feature exists to stop; still a one-line header field, no body — kind: addition
- **deviation** — parked the interval in a globalThis slot as well as a module variable — Pi /reload can hand this file a fresh module scope while the old interval is still armed, and two drains racing one inbox is a double-delivery source the cell did not anticipate — kind: robustness

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell prm-3 — save as docs/knowledge/patterns/pi-result-mailbox-prm-3-pitfall.md

---
type: bee.pattern
title: pi-result-mailbox cell prm-3 — pitfall candidate
description: "Pitfall candidate mined from cell prm-3's capped trace: prm-2 added no NEW pi.on event, so the never-throw hand list needed no new row — instead the list is lifted into never_throw_event_rows() and GATED by every_ad…"
timestamp: 2026-08-29
bee:
  id: pi-result-mailbox-prm-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/prm-3.json]
  polarity: pitfall
---

# pi-result-mailbox cell prm-3 — pitfall candidate

## What the cell did

Drain fixtures prove at-least-once delivery, header-only fences and a never-throw drain; the Pi harness is now timeout-bounded and records injections

## Recorded evidence (verbatim from .bee/cells/prm-3.json)

- **deviation** — prm-2 added no NEW pi.on event, so the never-throw hand list needed no new row — instead the list is lifted into never_throw_event_rows() and GATED by every_advisory_event_the_belt_registers_has_a_never_throw_row against the belt-derived pi.on set, so the next event added cannot skip its row — makes the cell request self-enforcing rather than one-time — improvement

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell prm-4 — save as docs/knowledge/patterns/pi-result-mailbox-prm-4-pitfall.md

---
type: bee.pattern
title: pi-result-mailbox cell prm-4 — pitfall candidate
description: "Pitfall candidate mined from cell prm-4's capped trace: Renamed the config-reference heading (dropped the (preview) suffix) and updated both in-file anchor links — a heading that still said preview was the same stan…"
timestamp: 2026-08-29
bee:
  id: pi-result-mailbox-prm-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/prm-4.json]
  polarity: pitfall
---

# pi-result-mailbox cell prm-4 — pitfall candidate

## What the cell did

Lifted the pi not-production caveat at every site with text naming the delivered result contract and its at-least-once limits

## Recorded evidence (verbatim from .bee/cells/prm-4.json)

- **deviation** — Renamed the config-reference heading (dropped the (preview) suffix) and updated both in-file anchor links — a heading that still said preview was the same standing lie as the callout under it — form deviation, in-scope file
- **deviation** — Cleared two extra preview mentions the cell action did not enumerate (.bee/config-sample.json herding note, catalog-projections-and-activation.md:144) — both inside cell-named files and both said the same retired thing — scope-completion, must_have no site still says not production
- **deviation** — Committed with an explicit -- pathspec through the shared index guard, excluding prm-w3's in-flight tests/pi_plugin_contracts.rs — concurrency hygiene, not a scope change

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell prm-5 — save as docs/knowledge/patterns/pi-result-mailbox-prm-5-pitfall.md

---
type: bee.pattern
title: pi-result-mailbox cell prm-5 — pitfall candidate
description: "Pitfall candidate mined from cell prm-5's capped trace: added a local #[cfg(test)] mod detached_delivery_tests inside prepare.rs — the truth \"the pi herding payload mentions --inbox-session\" needed a pinning test an…"
timestamp: 2026-08-29
bee:
  id: pi-result-mailbox-prm-5-pitfall
  lifecycle: draft
  sources: [.bee/cells/prm-5.json]
  polarity: pitfall
---

# pi-result-mailbox cell prm-5 — pitfall candidate

## What the cell did

result-inbox joins the generated ignore block; the pi herding payload carries a detached_delivery instruction naming --inbox-session

## Recorded evidence (verbatim from .bee/cells/prm-5.json)

- **deviation** — added a local #[cfg(test)] mod detached_delivery_tests inside prepare.rs — the truth "the pi herding payload mentions --inbox-session" needed a pinning test and the file already hosts two local test modules; stayed inside the two named files — scope-preserving

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 5 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 5 pattern candidate(s), 0 file(s) written.