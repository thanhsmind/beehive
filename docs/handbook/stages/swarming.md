# Stage: swarming (`bee-swarming`)

**Purpose** — Orchestrate bounded workers over validated cells (standard/high-risk
waves) or dispatch one execution worker (tiny/small). The orchestrator never
implements directly.

**When it runs** — After Gate 3 approval, with current-slice cells open and
validated.

## Inputs
- `cells schedule --json`, [`state.json`](../register.md#beestatejson),
  [reservations](../register.md#beereservationsjson), `CONTEXT.md` / `plan.md`.

## Outputs
- Capped cells with verify evidence, worker status tokens.
- `.bee/logs/dispatch.jsonl` traces. (Active workers are *derived* — live
  heartbeat sessions + cell claims; `state worker.*` verbs are compat no-ops.)
- Orchestrator-authored done-report (tiny/small).

## Gate
None directly — it relies on Gate 3 already being approved.

## State touched
[`cells claim/claim-next/show/tier/judge/judge-record/cap`](../register.md#beecellsfeature-njson),
[`reservations reserve/release/sweep/list`](../register.md#beereservationsjson)
(backed by the sharded lease store), [`HANDOFF.json`](../register.md#beehandoffjson)
via the per-workflow handoff mailbox.

## Key rules
- **The orchestrator claims a cell before spawning** (D1) — workers never
  self-select or claim their own cell.
- In standard/high-risk, **never implement cells yourself**.
- **Small-lane cells (1–3) are processed serially** — one live execution worker at
  a time; 2+ concurrent small-lane workers is a wave shape wearing a small lane.
- **Goal-check every `[DONE]` yourself** — re-run the verify, run the frozen and
  semantic judges. A worker's word is never the evidence.

## Source
`skills/bee-swarming/SKILL.md`
