promote proposal for work item "claude-sub-session-dispatch" (docs/history/claude-sub-session-dispatch/CONTEXT.md + docs/history/claude-sub-session-dispatch/plan.md) — 3 capped cell(s): cssd-1, cssd-2, cssd-3
anchor: history — docs/history/claude-sub-session-dispatch/CONTEXT.md, docs/history/claude-sub-session-dispatch/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/claude-sub-session-dispatch/delivery.md

---
type: bee.delivery
title: claude-sub-session-dispatch — delivery
description: "Delivery record proposed by bee knowledge promote for work item claude-sub-session-dispatch: 3 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-11-06
bee:
  id: claude-sub-session-dispatch-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/claude-sub-session-dispatch/CONTEXT.md, docs/history/claude-sub-session-dispatch/plan.md]
  sources: [docs/history/claude-sub-session-dispatch/CONTEXT.md, docs/history/claude-sub-session-dispatch/plan.md, .bee/cells/cssd-1.json, .bee/cells/cssd-2.json, .bee/cells/cssd-3.json]
---

# claude-sub-session-dispatch — Delivery

## What shipped

- **cssd-1** — claude-session transport in herding run, live-proven (4 file(s) changed)
- **cssd-2** — native Claude dispatches route to sub-sessions, live-proven (3 file(s) changed)
- **cssd-3** — Documented claude-session dispatch and SendMessage steering: AGENTS block line, swarming-reference line, new claude-session-transport concept, area index link; commit a838512c4 (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cssd-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee claude_session` — 24 claude_session unit tests on a fake CLI passed, plus a leader live run of bee herding run --session-model haiku --read-only that returned outcome done with steer_to bee-run-42-1701186-1 and left t…
- **cssd-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prepare` — 9 new claude_session_door tests pass; leader full suite with claude on PATH 5205 passed 1 failed and with claude off PATH the same single red, the opencode projection drift also red on main and fixed…
- **cssd-3** — `.bee/bin/bee dev release-manifest --check` — docs-only change; bee dev release-manifest --check, bee knowledge index --check and the cargo suite were not run (worker may not run bee verbs; no release build in this worktree); AGENTS.md left for …

## Deviations

- **cssd-1** — --settings also carries an env object so BEE_HERDING_WORKER, BEE_HERDING_JOB_ID and BEE_HERDING_READ_ONLY reach a session the Claude supervisor starts; the live run proved the read-only guard held
- **cssd-1** — herding/tests.rs gained claude_session: None in four Options literals to compile
- **cssd-1** — sync-ack: the bee-herding skill drives the cockpit roles, which this transport does not change; cssd-3 documents the transport in docs/knowledge/areas/bee-herding and the swarming reference
- **cssd-2** — a test-only thread-local keeps the existing hundreds of Agent-payload unit tests hermetic on hosts with claude on PATH; production is always armed
- **cssd-2** — herding.rs: claude_session module made pub(crate) so the door reads the same binary name as herding run
- **cssd-2** — leader fixed tests/pi_plugin_contracts.rs fixture with dispatch.claude_sessions false in commit 1c50b3edf, outside the cell file list
- **cssd-2** — sync-ack: the bee-herding skill drives the cockpit roles, which this door change does not touch; cssd-3 documents it in docs/knowledge/areas/bee-herding and the swarming reference

## Provenance

Proposed by `bee knowledge promote --work claude-sub-session-dispatch` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/claude-sub-session-dispatch/CONTEXT.md`, `docs/history/claude-sub-session-dispatch/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "claude-sub-session-dispatch" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-06T13:57:40.926Z), the work item declares no bee.areas.

area bee-herding:
  - [cssd-1] claude-session transport in herding run, live-proven — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/cssd-1.json)
  - [cssd-2] native Claude dispatches route to sub-sessions, live-proven — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/cssd-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell cssd-1 — save as docs/knowledge/patterns/claude-sub-session-dispatch-cssd-1-pitfall.md

---
type: bee.pattern
title: claude-sub-session-dispatch cell cssd-1 — pitfall candidate
description: "Pitfall candidate mined from cell cssd-1's capped trace: --settings also carries an env object so BEE_HERDING_WORKER, BEE_HERDING_JOB_ID and BEE_HERDING_READ_ONLY reach a session the Claude supervisor starts; the liv…"
timestamp: 2026-11-06
bee:
  id: claude-sub-session-dispatch-cssd-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/cssd-1.json]
  polarity: pitfall
---

# claude-sub-session-dispatch cell cssd-1 — pitfall candidate

## What the cell did

claude-session transport in herding run, live-proven

## Recorded evidence (verbatim from .bee/cells/cssd-1.json)

- **deviation** — --settings also carries an env object so BEE_HERDING_WORKER, BEE_HERDING_JOB_ID and BEE_HERDING_READ_ONLY reach a session the Claude supervisor starts; the live run proved the read-only guard held
- **deviation** — herding/tests.rs gained claude_session: None in four Options literals to compile
- **deviation** — sync-ack: the bee-herding skill drives the cockpit roles, which this transport does not change; cssd-3 documents the transport in docs/knowledge/areas/bee-herding and the swarming reference

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell cssd-2 — save as docs/knowledge/patterns/claude-sub-session-dispatch-cssd-2-pitfall.md

---
type: bee.pattern
title: claude-sub-session-dispatch cell cssd-2 — pitfall candidate
description: "Pitfall candidate mined from cell cssd-2's capped trace: a test-only thread-local keeps the existing hundreds of Agent-payload unit tests hermetic on hosts with claude on PATH; production is always armed"
timestamp: 2026-11-06
bee:
  id: claude-sub-session-dispatch-cssd-2-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/cssd-2.json]
  polarity: pitfall
---

# claude-sub-session-dispatch cell cssd-2 — pitfall candidate

## What the cell did

native Claude dispatches route to sub-sessions, live-proven

## Recorded evidence (verbatim from .bee/cells/cssd-2.json)

- **deviation** — a test-only thread-local keeps the existing hundreds of Agent-payload unit tests hermetic on hosts with claude on PATH; production is always armed
- **deviation** — herding.rs: claude_session module made pub(crate) so the door reads the same binary name as herding run
- **deviation** — leader fixed tests/pi_plugin_contracts.rs fixture with dispatch.claude_sessions false in commit 1c50b3edf, outside the cell file list
- **deviation** — sync-ack: the bee-herding skill drives the cockpit roles, which this door change does not touch; cssd-3 documents it in docs/knowledge/areas/bee-herding and the swarming reference

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 2 pattern candidate(s), 0 file(s) written.