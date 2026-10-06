promote proposal for work item "paseo-pi-hardening" (docs/history/paseo-pi-hardening/CONTEXT.md + docs/history/paseo-pi-hardening/plan.md) — 7 capped cell(s): pph-1, pph-2, pph-3, pph-4, pph-5, pph-6, pph-7
anchor: history — docs/history/paseo-pi-hardening/CONTEXT.md, docs/history/paseo-pi-hardening/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/paseo-pi-hardening/delivery.md

---
type: bee.delivery
title: paseo-pi-hardening — delivery
description: "Delivery record proposed by bee knowledge promote for work item paseo-pi-hardening: 7 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-10-06
bee:
  id: paseo-pi-hardening-delivery
  lifecycle: active
  areas: [bee-herding, hook-runtime]
  required_context: [docs/history/paseo-pi-hardening/CONTEXT.md, docs/history/paseo-pi-hardening/plan.md]
  sources: [docs/history/paseo-pi-hardening/CONTEXT.md, docs/history/paseo-pi-hardening/plan.md, .bee/cells/pph-1.json, .bee/cells/pph-2.json, .bee/cells/pph-3.json, .bee/cells/pph-4.json, .bee/cells/pph-5.json, .bee/cells/pph-6.json, .bee/cells/pph-7.json]
---

# paseo-pi-hardening — Delivery

## What shipped

- **pph-1** — Paseo run paths carry a 15 s CLI timeout; inspect every 3 s; one silent-idle nudge then an immediate idle-timeout end; observed model recorded in job.json (2 file(s) changed)
- **pph-2** — bee herding status shows stalled from UpdatedAt and model_mismatch for Paseo workers (1 file(s) changed)
- **pph-3** — worker-guard hook guards Paseo workers and the Pi supervisor; the Pi belt routes their shell calls to it fail-closed; no dispatch tools for Paseo workers (6 file(s) changed)
- **pph-4** — the Pi leader wakes from its own broker timer, the heartbeat backstop is */30 and is deleted on shutdown (4 file(s) changed)
- **pph-5** — bee doctor on pi shows a report-only paseo_ready row (2 file(s) changed)
- **pph-6** — herding.supervisor_runtime pi runs the supervisor tick on Pi with the guarded allowlist (1 file(s) changed)
- **pph-7** — the Paseo channel, worker-outward guard and supervisor concepts and the config reference describe the hardened behavior (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pph-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding` — 725 herding tests incl. nudge, second silent idle, failed send, observed model, timed-out spawn label; run by the leader in the worktree
- **pph-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding` — 735 herding tests incl. stalled from old UpdatedAt, fresh and missing UpdatedAt, recovered, model_mismatch rows; run by the leader
- **pph-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hooks && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_worker_guard_contracts --test pi_plugin_contracts` — 729 hooks tests, 7 worker-guard node contracts, 114 plugin contracts; one plugin test (deferred_exit_ordering) failed once under parallel load and passed alone and on the full re-run
- **pph-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts --test pi_plugin_contracts` — 24 heartbeat node contracts incl. timer, latch hold, delete, cron re-create; 114 plugin contracts; re-run after the judge revision ca64d2cb3
- **pph-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor` — 49 doctor tests incl. 5 paseo_ready tests, re-run after the leader fix
- **pph-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee control_loop` — 72 control_loop tests incl. claude byte-identical argv, pi argv with -e bee-guard, non-pi refusal, missing extension refusal, env and cwd on the spawn
- **pph-7** — `.bee/bin/bee knowledge check --json` — run in the worktree: 431 concepts, 0 errors; the leader checked that each concept cites paseo-pi-hardening D1-D10 with store ids

## Deviations

- **pph-1** — sync-ack: run-loop, status and control-loop internals change no bee-herding skill procedure; the bee-herding concepts are updated by pph-7
- **pph-2** — sync-ack: run-loop, status and control-loop internals change no bee-herding skill procedure; the bee-herding concepts are updated by pph-7
- **pph-3** — pi_plugin_contracts deferred_exit_ordering_settled_fork_switch_replacement_merge flaked once under load; passed on re-run
- **pph-4** — judge NEEDS_REVISION on the turn-start latch fixed in ca64d2cb3; re-judged PASS
- **pph-4** — sync-ack: run-loop, status and control-loop internals change no bee-herding skill procedure; the bee-herding concepts are updated by pph-7
- **pph-5** — leader removed invented PASEO_DAEMON and PASEO_HOST shortcuts after escalation
- **pph-6** — sync-ack: run-loop, status and control-loop internals change no bee-herding skill procedure; the bee-herding concepts are updated by pph-7

## Provenance

Proposed by `bee knowledge promote --work paseo-pi-hardening` from 7 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/paseo-pi-hardening/CONTEXT.md`, `docs/history/paseo-pi-hardening/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "paseo-pi-hardening" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-06T06:55:08.210Z), the work item declares no bee.areas.

area bee-herding:
  - [pph-1] Paseo run paths carry a 15 s CLI timeout; inspect every 3 s; one silent-idle nudge then an immediate idle-timeout end; observed model recorded in job.json — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pph-1.json)
  - [pph-2] bee herding status shows stalled from UpdatedAt and model_mismatch for Paseo workers — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/pph-2.json)
  - [pph-3] worker-guard hook guards Paseo workers and the Pi supervisor; the Pi belt routes their shell calls to it fail-closed; no dispatch tools for Paseo workers — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/pph-3.json)
  - [pph-4] the Pi leader wakes from its own broker timer, the heartbeat backstop is */30 and is deleted on shutdown — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/pph-4.json)
  - [pph-5] bee doctor on pi shows a report-only paseo_ready row — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pph-5.json)
  - [pph-6] herding.supervisor_runtime pi runs the supervisor tick on Pi with the guarded allowlist — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/pph-6.json)

area hook-runtime:
  - [pph-1] Paseo run paths carry a 15 s CLI timeout; inspect every 3 s; one silent-idle nudge then an immediate idle-timeout end; observed model recorded in job.json — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pph-1.json)
  - [pph-2] bee herding status shows stalled from UpdatedAt and model_mismatch for Paseo workers — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/pph-2.json)
  - [pph-3] worker-guard hook guards Paseo workers and the Pi supervisor; the Pi belt routes their shell calls to it fail-closed; no dispatch tools for Paseo workers — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/pph-3.json)
  - [pph-4] the Pi leader wakes from its own broker timer, the heartbeat backstop is */30 and is deleted on shutdown — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/pph-4.json)
  - [pph-5] bee doctor on pi shows a report-only paseo_ready row — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pph-5.json)
  - [pph-6] herding.supervisor_runtime pi runs the supervisor tick on Pi with the guarded allowlist — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/pph-6.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pph-1 — save as docs/knowledge/patterns/paseo-pi-hardening-pph-1-pitfall.md

---
type: bee.pattern
title: paseo-pi-hardening cell pph-1 — pitfall candidate
description: "Pitfall candidate mined from cell pph-1's capped trace: sync-ack: run-loop, status and control-loop internals change no bee-herding skill procedure; the bee-herding concepts are updated by pph-7"
timestamp: 2026-10-06
bee:
  id: paseo-pi-hardening-pph-1-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/pph-1.json]
  polarity: pitfall
---

# paseo-pi-hardening cell pph-1 — pitfall candidate

## What the cell did

Paseo run paths carry a 15 s CLI timeout; inspect every 3 s; one silent-idle nudge then an immediate idle-timeout end; observed model recorded in job.json

## Recorded evidence (verbatim from .bee/cells/pph-1.json)

- **deviation** — sync-ack: run-loop, status and control-loop internals change no bee-herding skill procedure; the bee-herding concepts are updated by pph-7

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pph-2 — save as docs/knowledge/patterns/paseo-pi-hardening-pph-2-pitfall.md

---
type: bee.pattern
title: paseo-pi-hardening cell pph-2 — pitfall candidate
description: "Pitfall candidate mined from cell pph-2's capped trace: sync-ack: run-loop, status and control-loop internals change no bee-herding skill procedure; the bee-herding concepts are updated by pph-7"
timestamp: 2026-10-06
bee:
  id: paseo-pi-hardening-pph-2-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/pph-2.json]
  polarity: pitfall
---

# paseo-pi-hardening cell pph-2 — pitfall candidate

## What the cell did

bee herding status shows stalled from UpdatedAt and model_mismatch for Paseo workers

## Recorded evidence (verbatim from .bee/cells/pph-2.json)

- **deviation** — sync-ack: run-loop, status and control-loop internals change no bee-herding skill procedure; the bee-herding concepts are updated by pph-7

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pph-3 — save as docs/knowledge/patterns/paseo-pi-hardening-pph-3-pitfall.md

---
type: bee.pattern
title: paseo-pi-hardening cell pph-3 — pitfall candidate
description: "Pitfall candidate mined from cell pph-3's capped trace: pi_plugin_contracts deferred_exit_ordering_settled_fork_switch_replacement_merge flaked once under load; passed on re-run"
timestamp: 2026-10-06
bee:
  id: paseo-pi-hardening-pph-3-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/pph-3.json]
  polarity: pitfall
---

# paseo-pi-hardening cell pph-3 — pitfall candidate

## What the cell did

worker-guard hook guards Paseo workers and the Pi supervisor; the Pi belt routes their shell calls to it fail-closed; no dispatch tools for Paseo workers

## Recorded evidence (verbatim from .bee/cells/pph-3.json)

- **deviation** — pi_plugin_contracts deferred_exit_ordering_settled_fork_switch_replacement_merge flaked once under load; passed on re-run

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pph-4 — save as docs/knowledge/patterns/paseo-pi-hardening-pph-4-pitfall.md

---
type: bee.pattern
title: paseo-pi-hardening cell pph-4 — pitfall candidate
description: "Pitfall candidate mined from cell pph-4's capped trace: judge NEEDS_REVISION on the turn-start latch fixed in ca64d2cb3; re-judged PASS"
timestamp: 2026-10-06
bee:
  id: paseo-pi-hardening-pph-4-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/pph-4.json]
  polarity: pitfall
---

# paseo-pi-hardening cell pph-4 — pitfall candidate

## What the cell did

the Pi leader wakes from its own broker timer, the heartbeat backstop is */30 and is deleted on shutdown

## Recorded evidence (verbatim from .bee/cells/pph-4.json)

- **deviation** — judge NEEDS_REVISION on the turn-start latch fixed in ca64d2cb3; re-judged PASS
- **deviation** — sync-ack: run-loop, status and control-loop internals change no bee-herding skill procedure; the bee-herding concepts are updated by pph-7
- **failure_signature** — broker timer's sendLeaderMessage bypasses the F1 turnStartPending latch, and on error clears it under the drain

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pph-5 — save as docs/knowledge/patterns/paseo-pi-hardening-pph-5-pitfall.md

---
type: bee.pattern
title: paseo-pi-hardening cell pph-5 — pitfall candidate
description: "Pitfall candidate mined from cell pph-5's capped trace: leader removed invented PASEO_DAEMON and PASEO_HOST shortcuts after escalation"
timestamp: 2026-10-06
bee:
  id: paseo-pi-hardening-pph-5-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/pph-5.json]
  polarity: pitfall
---

# paseo-pi-hardening cell pph-5 — pitfall candidate

## What the cell did

bee doctor on pi shows a report-only paseo_ready row

## Recorded evidence (verbatim from .bee/cells/pph-5.json)

- **deviation** — leader removed invented PASEO_DAEMON and PASEO_HOST shortcuts after escalation

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pph-6 — save as docs/knowledge/patterns/paseo-pi-hardening-pph-6-pitfall.md

---
type: bee.pattern
title: paseo-pi-hardening cell pph-6 — pitfall candidate
description: "Pitfall candidate mined from cell pph-6's capped trace: sync-ack: run-loop, status and control-loop internals change no bee-herding skill procedure; the bee-herding concepts are updated by pph-7"
timestamp: 2026-10-06
bee:
  id: paseo-pi-hardening-pph-6-pitfall
  lifecycle: draft
  areas: [bee-herding, hook-runtime]
  sources: [.bee/cells/pph-6.json]
  polarity: pitfall
---

# paseo-pi-hardening cell pph-6 — pitfall candidate

## What the cell did

herding.supervisor_runtime pi runs the supervisor tick on Pi with the guarded allowlist

## Recorded evidence (verbatim from .bee/cells/pph-6.json)

- **deviation** — sync-ack: run-loop, status and control-loop internals change no bee-herding skill procedure; the bee-herding concepts are updated by pph-7

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 7 capped cell(s) mined, 1 delivery draft, 12 area bullet(s), 6 pattern candidate(s), 0 file(s) written.