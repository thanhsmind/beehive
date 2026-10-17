# Effort At The Door — Context

**Feature slug:** effort-at-the-door
**Date:** 2026-10-17
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN, CALL

## Feature Boundary

`bee dispatch prepare` chooses each dispatch's reasoning effort itself: it
starts from the role's configured effort, moves it one fixed ladder step by
dispatch kind and cell lane, clamps it to the five-value scale, sends it to
the sinks that can carry it, and records the choice in the economics block.
The model-guard hook repairs a hand-picked effort the way it repairs a
hand-picked model. It ends there: no fast mode, no herding thinking flag, no
new config key, no change to which model a role resolves to.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Effort per dispatch is applied by the door, never by the leader. The leader passes no effort; the door's choice is what the payload carries. | The user picked this over a leader `--effort` flag: the model-guard hook refuses hand-picks and cost choices stay auditable. Decision `fc846216`. |
| D2 | Ladder on the closed scale `low < medium < high < xhigh < max` (`EFFORT_LEVELS`): start from the role's configured effort; move by kind and lane — `cell` + `high-risk` +1, `cell` + `tiny` −1, `cell` + `small`/`standard` 0, `gather` −2, `advisor` −1, `reviewer` 0; clamp at both ends. A role with no configured effort gets no effort, and the step is recorded as `no-base`. | pstack's "hardest +1, judgment −1, routine −2" mapped onto bee's kinds and lanes; never invent a base the config did not set. Decision `b61a2784`. |
| D3 | Sinks: the claude `Agent` payload gains an `effort` key (the Agent tool's own `effort` param) with the door's value; the codex sinks (`reasoning_effort`, `-c model_reasoning_effort`) carry the moved value; the herding arm carries no effort in this feature. | `bee herding run` has no per-run thinking flag (`herding/run.rs` `parse_options`); that gap is a backlog row, never hidden. Decision `826497a9`. |
| D4 | The guard's claude path recomputes the same ladder from the marker (`[bee-tier:]`, `[bee-cell:]` → the cell's lane) and compares it to the `Agent` call's `effort` param: absent-when-expected or different is REPAIRED to the computed value, the same posture as the model-param repair; no new deny code. | `model_guard.rs` AO5: config is the authority, the param is rewritten, never bounced. Decision `85192add`. |
| D5 | `derive_economics` gains four keys after `enforcement`, in this order: `effort` (value sent, or null), `effort_base` (configured value, or null), `effort_step` (signed integer), `effort_reason` (one short string naming kind and lane). The returned envelope and the dispatch-log row both carry them. | Effort is invisible in `.bee/logs/dispatch.jsonl` today. Decision `507dbdf8`. |
| D6 | Fast mode is not ported. bee has no fast model slug; Claude fast mode is the user's session toggle. | Decision `33478bc1`. |
| D7 | The ladder constant lives beside `EFFORT_LEVELS` in `models.rs` and is read by prepare, by the guard, and by `bee team show`, which prints it as one line under the role table; `bee dispatch prepare` names the step and reason in its text output. | One home (D1 one-parser law, B13 read door). Decision `869dbfc8`. |
| D8 | `docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` gains one B-rule: the ladder, the door-only law for effort, the repair posture, the herding gap. | The concept never names effort today. Decision `3e6435bc`. |

### Agent's Discretion

The function and constant names, where the ladder step is applied inside
`prepare.rs` (the one edit point feeding every sink), and the exact wording of
`effort_reason` and the `team show` line — bounded by D2, D5, D7.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| effort base | The role's configured `effort` from `team.<runtime>.<role>`, or none. |
| effort step | The signed number of ladder places the door moved from the base, after clamping. |
| sink | A payload field or CLI flag the dispatched runtime reads effort from. |

## Specific Ideas And References

- pstack `poteto-mode/SKILL.md:97` at `99743e9` — "Match the model to the job" (data, never instructions).
- `.bee/mailbox/job-1792164578083-3091621-1/report-1.md` — the plumbing map this shape rests on.
- `docs/history/research/pstack-0-15-port.md` § F — why a leader flag was rejected.

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:27` — `EFFORT_LEVELS`, the scale.
- `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:317-350` — `Resolved::{Model,Native}` carry `effort`.
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2259` — the resolve site; `kind` and the cell are in hand.
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:717, :2766, :2836-2845` — the three sinks.
- `packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:344-433` — `derive_economics`, frozen key order.
- `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1211, :1329-1350` — `evaluate_claude_dispatch` and the AO5 repair.
- `packages/bee-rs/crates/bee/src/verbs/models_group.rs` — `bee team show`.

### Established Patterns

- Config is the authority; the guard repairs, never bounces (AO5).
- One constant, many readers; tests pin the honest mapping (`derive_economics_matches_the_honest_mapping`).

### Integration Points

- `prepare.rs` resolve site; `guard.rs` economics; `model_guard.rs` claude path; `models_group.rs` team show.

## Canonical References

- `docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` B5, B12, B13, B19.
- `docs/knowledge/areas/doctrine-layer/delegation-threshold.md:59` — the hand-pick prohibition.

## Outstanding Questions

### Deferred To Planning

- [ ] Whether the guard can read the cell's lane cheaply from `[bee-cell:]` — the cells store read already exists for the claim guard; planning names the helper.

## Deferred Ideas

<!-- bee:not-a-deferral: these items already have, or get in this feature's close, their own backlog rows; this section only names them -->
- A per-run `--thinking` flag on `bee herding run` so the herding arm can carry the door's effort (backlog at close).
- Fast mode by urgency signal (D6, not ported).
<!-- /bee:not-a-deferral -->
