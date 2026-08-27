# SLP Supervisor Heartbeat — Context

**Feature slug:** slp-supervisor-heartbeat
**Date:** 2026-08-27
**Shaping session:** complete (Lock consumed the closed map docs/discovery/slp-supervisor-lead-peer/MAP.md — no decision originated here)
**Scope:** Standard
**Domain types:** RUN | READ

## Feature Boundary

bee gains an observer: a `supervisor` role of the native herding
control loop that wakes cold on an interval, reads bee's existing
state surfaces, writes open-question intervention records into a
mailbox the target session reads at its next turn boundary, and —
around an explicit away/back mark — queues non-urgent asks and
delivers exactly one WakeReport on return. It never writes product
code, never dispatches work, never merges, never approves anything.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. IDs are bee decision-log ids (search with
`bee decisions search`).

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| 787a9eb0 | SLP is distilled into bee's skeleton; bee's locked rules (R2 human merge, R3 owner interlock, R4 permission split, gates) win on any conflict | the observer adds beside those rules, never relaxes them |
| 322695d6 | The supervisor runs as a new `supervisor` role of `bee herding control-loop` (default `--interval 900`), COLD per tick; model = configured `supervisor` role (open fall-through set), semantic escalation = the `advisor` role; tool surface = enumerated read/query only | cold ticks prevent context accumulation; model is pure config per 06e49368 |
| da7cb49b | Observation reads bee's seven existing surfaces (pane transcripts+screen classifier, activity records, waiting-on marks, session registry liveness, wave ledger/occupancy, cells+budgets, decisions+triggers); day-1 signals: struggling-loop, big-decision, danger-op | the heartbeat ships on existing surfaces alone — the cheap Detector poller is explicitly NOT this feature |
| c80debd7 | Interventions are FILE RECORDS in a mailbox, read by the target session at its NEXT turn boundary — never mid-turn pane injection; the record carries frequency-cap state (same point twice = escalate, never repeat); ordinary interventions surface to the human only in reports; danger-class UrgentAlerts notify immediately | a persistent record is what survives between cold ticks |
| 9f5cd250 | Presence is an explicit lightweight away/back mark with exactly two effects: it defines the report window, and non-urgent asks queue silently; gates and bypass levels untouched; on back exactly ONE WakeReport — markdown ≤10 lines, four sections (what happened / what was decided / what needs you / next action) — plus one push notification | permission control never hides in a presence flag |
| a8f4b8ab | Signal set adds: work exceeding 2× its recorded estimate (measured by the harness, never self-reported) and two consecutive submissions differing only in the same region; budget/overrun telemetry is computed by the deterministic layer and injected; the night queue sorts by the confidence×door predicate (one-way door + low confidence always waits / UrgentAlert) | — |
| c706053e | A NARROW opt-in silence-is-consent mode: only when explicitly enabled, only for non-gate queued asks, user-configured timeout, every auto-proceed logged and rendered prominently in the WakeReport; gates and one-way low-confidence asks always wait | recorded exception in the gate_bypass spirit; the timeout belongs to the deterministic layer, never the model |
| 66c4c251 | Reports carry a small health-metric set with two-sided bands (wrong-assumption rate, escalations/task, blocked rate, self-answered band, spec-flag band…); RAISING silence-is-consent is EARNED — zero human-reversed one-way decisions across 40–60 tasks, and the human still flips the switch; the WakeReport sorts assumptions/decisions by impact-if-wrong descending | |
| a020319d | This feature is the FIRST of the four slp clusters; dissent, blind lanes, and contract/original-request are separate features and out of this boundary | — |

### Agent's Discretion

Everything the decisions above leave open is implementer's choice at
planning: record schemas and file locations, hook wiring for
next-turn delivery, counter storage, prompt wording for the supervisor
role. Constraint: reuse existing machinery (control loop, mailbox
patterns, waiting-on, hooks) before inventing any new subsystem — the
research digests below name the anchors.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Supervisor | The OBSERVER role — reads, asks open questions, reports. Never a router/dispatcher (the harness spec uses the same word for a router; that meaning is rejected here). |
| Intervention | One open question (≤2 sentences, no asserted fault, no suggested answer) written as a mailbox record for one target session. |
| Presence mark | The away/back state with exactly the two effects of 9f5cd250. |
| WakeReport | The single ≤10-line, four-section report delivered on back. |
| UrgentAlert | A danger-class notice that skips the queue and notifies immediately. |
| Detector | The future cheap event poller — NOT in this feature (da7cb49b). |

## Specific Ideas And References

- The 7-step build order from the harness spec (App. A of the full
  text): validators/schemas first, the loop runs end-to-end with the
  simplest wiring before any role split; never debug several LLM
  layers at once.
- Intervention question style (SLP spec §4.7): open question, ≤2
  sentences, no asserted fault, no directional language; "already
  looked, chose silence" is a logged, legitimate outcome.

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/herding/control_loop.rs` — the loop
  driver: `Role` enum (:54-75), `allowed_tools_for` (:220-235),
  interval/timeout/backoff/stop-file machinery. The supervisor is a
  new role here, not a new loop.
- `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs` — open
  fall-through role set; `models.claude.supervisor` resolves with zero
  resolver changes.
- `skills/bee-herding/references/<role>-prompt.md` — role prompt
  template pattern to follow for `supervisor-prompt.md`.
- `packages/bee-rs/crates/bee/src/verbs/mailbox.rs` (human-mailbox)
  and `src/herding/mailbox.rs` (per-job round mailbox) — the two
  existing mailbox patterns; the intervention record follows them.
- Read surfaces with anchors: docs/history/research/slp-observer-surfaces.md.

### Established Patterns

- Cold role invocation per tick (dispatch/merge roles of bee-herding).
- Door-enforced obligations (judge-debt) — the model for anything the
  supervisor must not be able to skip.
- Hook-injected turn-boundary content (UserPromptSubmit / SessionStart
  preamble) — the delivery point for intervention records.

### Integration Points

- `bee herding control-loop --role supervisor --interval 900` — new role arm.
- `.bee/` state: a new intervention mailbox + presence mark + report
  window records (exact shape = planning).
- The activity/waiting-on hooks — where "read at next turn boundary" lands.

## Canonical References

- `docs/discovery/slp-supervisor-lead-peer/MAP.md` — the closed map and full decision gists.
- `docs/history/research/slp-supervisor-placement.md` — placement verdict + anchors.
- `docs/history/research/slp-observer-surfaces.md` — surface-by-surface audit, signal feasibility.
- `docs/history/research/agent-harness-slap-distill.md` (+ `-fulltext.md`) — metrics, earned autonomy, build order, intervention hygiene.
- `docs/specs/slp-supervisor-lead-peer/` — the source SLP spec (idea source only, per 787a9eb0).

## Outstanding Questions

### Resolve Before Planning

(none — every product decision is a locked D-ID above)

### Deferred To Planning

- [ ] Intervention record + queue schema and location under `.bee/` — read the two mailbox implementations, pick one pattern.
- [ ] Delivery wiring: which hook injects a pending intervention at the target session's next turn, and how the mark clears.
- [ ] Presence mark storage (state record vs own file) and how the report window is computed.
- [ ] Push notification transport for WakeReport/UrgentAlert — what exists on this machine (ntfy? PushNotification tool? terminal bell) — pick the cheapest real one.
- [ ] Where the 2×-estimate rule gets its estimate — cells carry no estimate field today; decide field vs skip-until-present.
- [ ] Metric counters: derive from existing stores at report time (preferred) vs persist counters.

## Deferred Ideas

- Detector (cheap signal poller over transcripts) — next feature-sized upgrade of this same role (da7cb49b).
- Sub-supervisor fan-in, voice WakeReport, heterogeneous lane models — Out of scope on the map, return as fresh efforts.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are bee decision-log
ids, stable. Planning reads locked decisions, code context, canonical
references, and deferred-to-planning questions. Planning's Gate 2
shape stage and reviewing use locked decisions for coverage and UAT.
