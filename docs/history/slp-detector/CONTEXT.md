# SLP Detector — Context

**Feature slug:** slp-detector
**Date:** 2026-10-15
**Shaping session:** complete (gate bypass full; the user said "Làm 1, 2" on the SLP + Paseo answer)
**Scope:** Standard
**Domain types:** RUN | READ

## Feature Boundary

bee gets the SLP Detector: a code-only tick that reads surfaces bee already
keeps, fires four typed Events into one new append-only store, and lets the
supervisor judge the four SLP signals bee did not detect before
(self-correction, boundary-approach, test-on-unstable-contract, budget-80).
The tick rides the supervisor control-loop tick and the Pi leader's existing
Paseo heartbeat. The feature ends at the supervisor's reading: delivery of
interventions, the leader's heartbeat rules, and Paseo itself do not change.

Spec: `docs/specs/slp-supervisor-lead-peer/slp-supervisor-lead-peer.md`
(§ 1 Detector, § 4.8 Event, § 5 S4, § 6 invariants 2 and 8).

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit. Store ids in the right column.

| ID | Decision | Store |
|----|----------|-------|
| D1 | The Detector is a code-only verb `bee detector tick --json`, no model call. It reads only existing surfaces (herding `job.json`, cell records, derived contract status, the job worktree's `git diff` name list) and appends Event rows to one new append-only store at the control root, `.bee/supervisor/events.jsonl`. It writes no other store, sends nothing to any session, notifies nobody; its only reader is the supervisor. | d19eec05 |
| D2 | Event row: `{id, ts, signal, job_id, cell_id, target_session, point_key, excerpt, source}`. `signal` is a closed set of four: `self-correction`, `boundary-approach`, `test-on-unstable-contract`, `budget-80`. `excerpt` is one sentence of context, never a conclusion or proposal. The four names join `KNOWN_SIGNALS` and `INTERVENTION_SIGNALS` of `bee supervisor record`. `point_key` is the slug the supervisor reuses as its intervention point key, so a second hit on one point escalates. | 5231cdf4 |
| D3 | Rules, deterministic, over every live herding job (a `job.json` with no final result for its current round). **budget-80**: elapsed since `created_at` ≥ 80% of `ceiling_secs`, or claims used ≥ 80% of `budgets.max_claims` (the count `check_cell_budgets` reads), or `usage.total_tokens` ≥ 80% of config `herding.budget.tokens_per_job` when set; one event per (job, budget kind). **test-on-unstable-contract**: the cell cites a decision tagged `contract:<name>` whose derived status is Unsettled and the job diff touches a test path (`tests/`, `spec/`, `*_test.*`, `*.test.*`, `*_tests.*`); one event per (job, contract). **boundary-approach**: the job diff touches a path outside the cell's non-empty declared `files`; one event per (job, path). **self-correction**: a path in the job's changed set at one tick is absent at a following tick while the job still runs; the detector keeps the last changed set in `.bee/supervisor/detector/<job_id>.json`; one event per (job, path). `estimate_minutes` stays unwritten. | 936eb9a8 |
| D4 | The supervisor reads events through `bee detector events [--unjudged] [--json]`, added to `SUPERVISOR_ALLOWED_TOOLS` and the prompt's read list. An event is judged when a subsequent observation or silence row names its id in a new optional `--events <id,...>` field of `bee supervisor record`; `--unjudged` lists events no row names yet. The prompt gains one paragraph per new signal: read the context around the event before speaking; a healthy stream is a silence row naming the event ids reviewed; an intervention on an event reuses its `point_key`. An event alone never creates an intervention and never changes delivery (c80debd7 stands). | 699974db |
| D5 | `bee detector tick` runs at two trigger points and by hand: in code at the start of every supervisor control-loop tick, before the model reads; and in the Pi leader's bee-guard extension beside `bee herding broker tick --json` on every Paseo heartbeat input it already handles. No second heartbeat, no schedule (paseo-pi D8). Detector events are never news for the leader. A tick fires each (job, signal, point_key) once ever, deduplicated against `events.jsonl`. | a6232035 |
| D6 | `bee herding status` shows, per job, the count of detector events by signal, read from `events.jsonl`; nothing else changes. `bee doctor` gains no row. | 36568aaa |
| D7 | Out of scope: a model-based transcript classifier, mid-turn delivery of interventions, any new Paseo heartbeat or schedule, the waggledance far-eye supervisor, writing `estimate_minutes`. | 8fac7f27 |

### Agent's Discretion

- Exact id format of an Event, the `excerpt` wording per rule, and the
  diff command used (planning picks the one `bee herding status` already
  uses for `no_diff_progress`).
- Where the test-path pattern list lives (one const, one home).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Detector | The code tick of D1. Reads, fires events, never speaks to a session. |
| Event | One row of `events.jsonl` (D2). A fact with one sentence of context. |
| Judged | An event whose id a subsequent observation or silence row names (D4). |
| Live job | A herding job whose current round has no final result file. |
| Point key | The slug shared by an event and the supervisor's intervention cap (D2). |

## Existing Code Context

From the code scout (gather report
`.bee/mailbox/job-1792075577054-1658640-1/report-1.md`). Paths under
`packages/bee-rs/crates/bee/` unless stated.

### Reusable Assets

- `src/verbs/supervisor.rs:231,303-313` — `KNOWN_KINDS`, `KNOWN_SIGNALS`, `INTERVENTION_SIGNALS`; `record_into` at `:336` validates `--kind`/`--signal` before append; store helpers `supervisor_dir`/`control_root_path` at `:200-220`.
- `src/verbs/cells/validate.rs:124-128,715,777-845` — `BUDGET_KEYS`, defaults, `check_cell_budgets` and `attempts_since_budget_reset` (the claims count D3 reuses).
- `src/herding/run.rs:3045-3153` — `usage` map written to `job.json` (`total_tokens`, `cost_usd`, `source`).
- `src/verbs/decisions/read.rs:324-422` — `ContractStatus`, `contract_status_over`, `contract_status`; `src/verbs/cells/handlers_write.rs:1244-1252,1323` — `event_has_contract_tag`, `contract_citation_refusal` (how a cell cites a contract through `decisions`).
- `src/herding.rs:915-935` — `insert_usage` on the status listing (where D6's event counts sit beside it).
- rig-5 `stalled`/`no_diff_progress` in `bee herding status` — the existing diff-progress read D3's changed set builds on.

### Established Patterns

- Append-only event-sourced stores at the control root (observations, interventions, presence, reports) — `events.jsonl` is the fifth, same resolution rule.
- Closed sets validated before append, refusal names the set — `record_into`.
- The control loop pins the shipped supervisor prompt to the closed sets in a test — `src/herding/control_loop.rs:1303-1321`, `read_prompt_file` `:530`.

### Integration Points

- `src/herding/control_loop.rs:81,115,330-355,686,916` — `Role::Supervisor`, `SUPERVISOR_DEFAULT_INTERVAL`, `SUPERVISOR_ALLOWED_TOOLS` (+ `detector events`), the iteration where D5's in-code tick runs.
- `.pi/extensions/bee-guard/events.ts:368-405` — the heartbeat `input` handler that runs `bee herding broker tick --json` (D5 adds `bee detector tick --json` beside it); `paseo-heartbeat.ts:101-108` `isHeartbeatPrompt`.
- `skills/bee-herding/references/supervisor-prompt.md` — read list (six commands today, "no eighth surface" wording to change), signal paragraphs, the record forms.
- `src/herding.rs:152` — verb match table where `detector` registers; `src/verbs/drivers/prepare.rs` untouched.
- Tests: `tests/pi_paseo_heartbeat_contracts.rs:195-198` (heartbeat prompt contract), `control_loop.rs` prompt-pin test.

## Canonical References

- `docs/specs/slp-supervisor-lead-peer/slp-supervisor-lead-peer.md` — the SLP interaction spec (Detector, Event, S4, invariants).
- `docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md` — the supervisor's contract; da7cb49b names the Detector as a separate feature.
- `docs/knowledge/areas/bee-herding/the-paseo-channel.md` — the leader heartbeat and broker timer.

## Outstanding Questions

### Answered at planning

- [x] The stall read uses `git diff HEAD` on the job cwd (`src/herding.rs` worktree_fingerprint); the D3 refinement (32492fc4) replaced that source with a diff against `base_sha` plus untracked files.
- [x] A live job is one whose `result-<round>.json` is absent in its mailbox (`mailbox.rs` result naming).

## Out-of-scope ideas

<!-- bee:not-a-deferral: this line records a backlog row that already exists; nothing here promises later work -->
- Model-tier detector rules over worker transcripts — backlog proposal row (2026-10-15).
<!-- /bee:not-a-deferral -->
