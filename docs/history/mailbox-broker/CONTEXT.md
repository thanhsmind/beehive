# Mailbox Broker — Context

**Feature slug:** mailbox-broker
**Date:** 2026-10-03
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | CALL

## Feature Boundary

A dispatched worker that needs an answer can ask one question and stop; an active, non-LLM broker on bee's own control loop gets that answer from the advisor role or the human and re-dispatches the worker with it. The feature ends at that loop: the leader session, the supervisor observer, and the gates do not change.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The broker is code, not an LLM. It ticks on bee's own herding control loop, with no Paseo dependency. It reuses the existing result-inbox and supervisor delivery paths and adds no second mailbox. Workers stay passive. The supervisor stays observer-only. (store `62b88465`, touches `322695d6`) | Keeps the locked observer rule; the seatworks research warns that a second mailbox duplicates result-inbox recovery. |
| D2 | Only dispatched workers post questions: execution cells and hat or lane seats. The leader session keeps asking the human directly. (store `0c094609`) | — |
| D3 | A worker that needs an answer ends its run with a new typed outcome `question` and stops. The broker gets the answer, then re-dispatches the same cell or seat as the next round with the question and the answer carried in. No answer is typed into a live pane. (store `caf5f716`) | Rides the existing rounds and prior-rounds machinery on every runtime. |
| D4 | The broker routes each question to the configured `advisor` role first. The question goes to the human only when it is a product or gate decision, or when the advisor answer says it is not sure. (store `b9a3e488`) | Gates stay human-owned. |
| D5 | A question routed to the human while the human is away is queued in the existing presence queue and shown in the wake report. It proceeds without the human only under the existing opt-in consent-sweep rules. (store `c88d31bb`, touches `c706053e`, `9f5cd250`) | No new timeout path. |

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| broker | The non-LLM tick that reads posted questions, routes each one, and re-dispatches the asking worker with its answer. It decides only routing, never content. |
| question | One typed worker outcome carrying a single question about the worker's own cell or seat. It ends the run; it is not a message to a live session. |
| passive worker | A worker that never polls and never waits: it asks by stopping, and it learns the answer only from its next round's prompt. |

## Specific Ideas And References

- Paseo `create_heartbeat` and `notifyOnFinish`: a daemon is the active side and agents are woken or notified. The user named this pattern, as SLP and seatworks use it. Bee takes the shape, not the tool (D1).

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/herding/control_loop.rs` — the tick engine: interval, stop file, timeout, backoff, cold child per iteration.
- `packages/bee-rs/crates/bee/src/herding/run.rs` — `bee herding run`, its typed outcomes, and `--inbox-session`.
- `packages/bee-rs/crates/bee/src/herding/mailbox.rs` — the per-job mailbox, brief, ack, report and result files.
- `.pi/extensions/bee-guard/result-inbox.ts` — Pi drain: claim by atomic rename, requeue on failed injection.
- `packages/bee-rs/crates/bee/src/verbs/supervisor.rs` — interventions, presence, wake report, consent-sweep.
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` — the dispatch door; the "Prior rounds" block of a cell prompt.

### Established Patterns

- Typed worker outcomes (`done`, `died`, `paused_limit`, `timed_out_ceiling`, `interrupted`, `cancelled`) — `the-run-verb-and-worker-outcomes.md`; `question` joins this set.
- Cold ticks with exactly one record per tick — the supervisor observer.

### Integration Points

- The herded worker wrapper tells the worker to ignore bee workflow instructions (`herding/mailbox.rs`). The ask instruction must reach the worker through that wrapper.
- The `advisor` role resolved through `bee dispatch prepare --kind advisor`.

## Canonical References

- `docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md` — the observer rule D1 keeps.
- `docs/knowledge/areas/bee-herding/presence-wake-reports-and-earned-autonomy.md` — the away queue and consent-sweep D5 reuses.
- `docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md` — the outcome set D3 extends.
- `docs/history/research/slp-supervisor-placement.md` — why the tick is bee's control loop, not Paseo.
- `docs/history/research/seatworks-slp-pi-small-models.md` — finding 8, no second mailbox.

## Outstanding Questions

### Deferred To Planning

- [ ] How a worker signals `question` on each runtime (a result field, a verdict tool status, or a file) — read `tool-verdict.ts` and the result file contract.
- [ ] How the advisor answer marks "not sure" so D4 can route it to the human — a fixed field in the advisor result.
- [ ] How the broker tells a product or gate question from a technical one (D4) — a worker-set kind field versus the advisor's call.
- [ ] Round limit: how many question rounds one cell may take before it is treated as blocked.
- [ ] Whether the broker runs as a new control-loop role or as a step of an existing role.

## Deferred Ideas

- Leader questions through the mailbox — out by D2.
- Answer injection into a live worker — out by D3.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
