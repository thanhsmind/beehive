promote proposal for work item "slp-detector" (docs/history/slp-detector/CONTEXT.md + docs/history/slp-detector/plan.md) — 4 capped cell(s): det-1, det-2, det-3, det-4
anchor: history — docs/history/slp-detector/CONTEXT.md, docs/history/slp-detector/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/slp-detector/delivery.md

---
type: bee.delivery
title: slp-detector — delivery
description: "Delivery record proposed by bee knowledge promote for work item slp-detector: 4 capped cell(s), 15 recorded deviation(s)."
timestamp: 2026-10-16
bee:
  id: slp-detector-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/slp-detector/CONTEXT.md, docs/history/slp-detector/plan.md]
  sources: [docs/history/slp-detector/CONTEXT.md, docs/history/slp-detector/plan.md, .bee/cells/det-1.json, .bee/cells/det-2.json, .bee/cells/det-3.json, .bee/cells/det-4.json]
---

# slp-detector — Delivery

## What shipped

- **det-1** — Detector event store, bee detector events, four signal names in both closed sets, --events on supervisor record; commit 3f41d09e2 (19 file(s) changed)
- **det-2** — bee detector tick with the four deterministic rules, base_sha recorded at spawn, tick_quiet for the control loop; commit 96878b399 (4 file(s) changed)
- **det-3** — Supervisor provider ticks the detector before argv resolves, allowlist carries detector events, status shows per-job event counts and unjudged, heartbeat runs the tick in finally; commit 42ee09a20 (6 file(s) changed)
- **det-4** — Supervisor prompt reads detector events as its seventh command, judges ten signals and names judged ids with --events; concepts and config reference document the Detector (15 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **det-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee supervisor && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee detector && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee catalog && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee control_loop && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts && .bee/bin/bee dev release-manifest --check` — every leg ok as the worker recorded (supervisor 89, detector 12, catalog 19, control_loop 72+1, registry_contracts 12, release-manifest 419 files match); the full crate suite also ran: 4781 passed, 0…
- **det-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee detector && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee catalog && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts` — detector 23 passed (bin), catalog 20 passed incl. detector_tick_resolves_and_declares_only_the_json_flag, registry_contracts 12 passed; herding::run 319 passed after the base_sha change; bee dev comm…
- **det-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee control_loop && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee herding && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts && .bee/bin/bee dev release-manifest --check` — the cell's declared verify legs ran in full from the worktree at 42ee09a20, exit 0 each: control_loop (three new tests confirmed by name), herding (two new status tests confirmed by name), pi_paseo_h…
- **det-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee control_loop && .bee/bin/bee knowledge index --check && .bee/bin/bee dev release-manifest --check` — 72 control_loop tests incl. both supervisor prompt pin tests passed; knowledge index 0 stale; manifest 419 files match; five copies cmp-equal to source

## Deviations

- **det-1** — Worker timed out bee's 1800 s ceiling (timed_out_ceiling) but the Paseo agent finished the cell on its own and wrote result-1.json; the leader capped from that result
- **det-1** — Touched herding/broker.rs and hooks/prompt_context.rs beyond the declared files: every caller of record_into/record_intervention_into passes the new events argument as None — mechanical, judged intact
- **det-1** — registry_payload.json hand-edited in the supervisor.list shape because no generator exists in the tree (decisions 3358743e, d515c1d7); file re-parsed, 212 commands
- **det-1** — sync-ack: hooks/prompt_context.rs changed by one mechanical None argument to record_intervention_into; no doctrine behavior changed; the prompt and concept docs are det-4's cell
- **det-2** — registry_payload.json gained detector.tick by a byte-precise insertion script (.bee/spikes/slp-detector/insert-tick.py), 212 -> 213 commands, the precedent det-1 set — followed the plan
- **det-2** — The session model executed det-2 after two paseo-pi-flash dispatches ended timed_out_idle with zero edits (rescue ladder rung 2, cell escalated) — bee dispatch prepare still returned the herding command despite escalate: true, so the leader ran the cell inline in the worktree — hit an unforeseen obstacle
- **det-2** — cargo was invoked by its absolute path /home/thanhsmind/.cargo/bin/cargo instead of the PATH prefix spelling because the host worktree guard refuses shell variables; the same test legs ran — hit an unforeseen obstacle
- **det-2** — sync-ack: herding/run.rs gains one job.json field (base_sha) the Detector reads; det-4 already rewrote the bee-herding supervisor prompt and the area concepts for the Detector
- **det-3** — The worker (paseo-pi-flash) committed and wrote result-1.json but its cells finish never ran: the paseo transport timed out at the end of the round; the leader capped from the recorded result — hit an unforeseen obstacle
- **det-3** — followed the plan
- **det-3** — sync-ack: det-4 already rewrote the bee-herding supervisor prompt and the area concepts for the Detector and its trigger points; this cell wires the code those docs describe
- **det-4** — committed the five .bee-render.json render stamps beside the copies, reserved first — bee dev regen rewrites them with the copies and det-1 committed them the same way — the plan was wrong about a fact
- **det-4** — the seventh command line also tells the reader to use --json to read event ids — the text form of bee detector events prints no id, and --events needs ids — the plan was wrong about a fact
- **det-4** — the point_key rule is stated for a row that takes --point-key, since observation rows take none and the det-1 nudge line keeps three signals — the plan was wrong about a fact
- **det-4** — area index.md did not change after bee knowledge index — the generated index carries nothing these edits touched — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work slp-detector` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/slp-detector/CONTEXT.md`, `docs/history/slp-detector/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "slp-detector" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-16T02:55:30.371Z), the work item declares no bee.areas.

area bee-herding:
  - [det-1] Detector event store, bee detector events, four signal names in both closed sets, --events on supervisor record; commit 3f41d09e2 — feature-wide sync per the scribing stamp, 19 file(s) changed (trace .bee/cells/det-1.json)
  - [det-2] bee detector tick with the four deterministic rules, base_sha recorded at spawn, tick_quiet for the control loop; commit 96878b399 — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/det-2.json)
  - [det-3] Supervisor provider ticks the detector before argv resolves, allowlist carries detector events, status shows per-job event counts and unjudged, heartbeat runs the tick in finally; commit 42ee09a20 — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/det-3.json)
  - [det-4] Supervisor prompt reads detector events as its seventh command, judges ten signals and names judged ids with --events; concepts and config reference document the Detector — feature-wide sync per the scribing stamp, 15 file(s) changed (trace .bee/cells/det-4.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell det-1 — save as docs/knowledge/patterns/slp-detector-det-1-pitfall.md

---
type: bee.pattern
title: slp-detector cell det-1 — pitfall candidate
description: "Pitfall candidate mined from cell det-1's capped trace: Worker timed out bee's 1800 s ceiling (timed_out_ceiling) but the Paseo agent finished the cell on its own and wrote result-1.json; the leader capped from that…"
timestamp: 2026-10-16
bee:
  id: slp-detector-det-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/det-1.json]
  polarity: pitfall
---

# slp-detector cell det-1 — pitfall candidate

## What the cell did

Detector event store, bee detector events, four signal names in both closed sets, --events on supervisor record; commit 3f41d09e2

## Recorded evidence (verbatim from .bee/cells/det-1.json)

- **deviation** — Worker timed out bee's 1800 s ceiling (timed_out_ceiling) but the Paseo agent finished the cell on its own and wrote result-1.json; the leader capped from that result
- **deviation** — Touched herding/broker.rs and hooks/prompt_context.rs beyond the declared files: every caller of record_into/record_intervention_into passes the new events argument as None — mechanical, judged intact
- **deviation** — registry_payload.json hand-edited in the supervisor.list shape because no generator exists in the tree (decisions 3358743e, d515c1d7); file re-parsed, 212 commands
- **deviation** — sync-ack: hooks/prompt_context.rs changed by one mechanical None argument to record_intervention_into; no doctrine behavior changed; the prompt and concept docs are det-4's cell

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell det-2 — save as docs/knowledge/patterns/slp-detector-det-2-pitfall.md

---
type: bee.pattern
title: slp-detector cell det-2 — pitfall candidate
description: "Pitfall candidate mined from cell det-2's capped trace: registry_payload.json gained detector.tick by a byte-precise insertion script (.bee/spikes/slp-detector/insert-tick.py), 212 -> 213 commands, the precedent det…"
timestamp: 2026-10-16
bee:
  id: slp-detector-det-2-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/det-2.json]
  polarity: pitfall
---

# slp-detector cell det-2 — pitfall candidate

## What the cell did

bee detector tick with the four deterministic rules, base_sha recorded at spawn, tick_quiet for the control loop; commit 96878b399

## Recorded evidence (verbatim from .bee/cells/det-2.json)

- **deviation** — registry_payload.json gained detector.tick by a byte-precise insertion script (.bee/spikes/slp-detector/insert-tick.py), 212 -> 213 commands, the precedent det-1 set — followed the plan
- **deviation** — The session model executed det-2 after two paseo-pi-flash dispatches ended timed_out_idle with zero edits (rescue ladder rung 2, cell escalated) — bee dispatch prepare still returned the herding command despite escalate: true, so the leader ran the cell inline in the worktree — hit an unforeseen obstacle
- **deviation** — cargo was invoked by its absolute path /home/thanhsmind/.cargo/bin/cargo instead of the PATH prefix spelling because the host worktree guard refuses shell variables; the same test legs ran — hit an unforeseen obstacle
- **deviation** — sync-ack: herding/run.rs gains one job.json field (base_sha) the Detector reads; det-4 already rewrote the bee-herding supervisor prompt and the area concepts for the Detector

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell det-3 — save as docs/knowledge/patterns/slp-detector-det-3-pitfall.md

---
type: bee.pattern
title: slp-detector cell det-3 — pitfall candidate
description: "Pitfall candidate mined from cell det-3's capped trace: The worker (paseo-pi-flash) committed and wrote result-1.json but its cells finish never ran: the paseo transport timed out at the end of the round; the leader…"
timestamp: 2026-10-16
bee:
  id: slp-detector-det-3-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/det-3.json]
  polarity: pitfall
---

# slp-detector cell det-3 — pitfall candidate

## What the cell did

Supervisor provider ticks the detector before argv resolves, allowlist carries detector events, status shows per-job event counts and unjudged, heartbeat runs the tick in finally; commit 42ee09a20

## Recorded evidence (verbatim from .bee/cells/det-3.json)

- **deviation** — The worker (paseo-pi-flash) committed and wrote result-1.json but its cells finish never ran: the paseo transport timed out at the end of the round; the leader capped from the recorded result — hit an unforeseen obstacle
- **deviation** — followed the plan
- **deviation** — sync-ack: det-4 already rewrote the bee-herding supervisor prompt and the area concepts for the Detector and its trigger points; this cell wires the code those docs describe

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell det-4 — save as docs/knowledge/patterns/slp-detector-det-4-pitfall.md

---
type: bee.pattern
title: slp-detector cell det-4 — pitfall candidate
description: "Pitfall candidate mined from cell det-4's capped trace: committed the five .bee-render.json render stamps beside the copies, reserved first — bee dev regen rewrites them with the copies and det-1 committed them the …"
timestamp: 2026-10-16
bee:
  id: slp-detector-det-4-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/det-4.json]
  polarity: pitfall
---

# slp-detector cell det-4 — pitfall candidate

## What the cell did

Supervisor prompt reads detector events as its seventh command, judges ten signals and names judged ids with --events; concepts and config reference document the Detector

## Recorded evidence (verbatim from .bee/cells/det-4.json)

- **deviation** — committed the five .bee-render.json render stamps beside the copies, reserved first — bee dev regen rewrites them with the copies and det-1 committed them the same way — the plan was wrong about a fact
- **deviation** — the seventh command line also tells the reader to use --json to read event ids — the text form of bee detector events prints no id, and --events needs ids — the plan was wrong about a fact
- **deviation** — the point_key rule is stated for a row that takes --point-key, since observation rows take none and the det-1 nudge line keeps three signals — the plan was wrong about a fact
- **deviation** — area index.md did not change after bee knowledge index — the generated index carries nothing these edits touched — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 4 pattern candidate(s), 0 file(s) written.