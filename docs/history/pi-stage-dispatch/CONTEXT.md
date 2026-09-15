# Pi stage dispatch — Context

**Feature slug:** pi-stage-dispatch
**Date:** 2026-09-15
**Shaping session:** complete
**Scope:** Deep
**Domain types:** CALL | RUN

## Feature Boundary

A Pi leader session can run the advisor consult, the plan-step hat wave, and
every stage dispatch (gather, reviewer, cell) through the herding door, get each
worker's full answer back, and finish inside the stage budget — with no Claude
or Codex tool it does not have, and no manual workaround.

## Why now

The user reported that advisor consults, hat waves, and bee stages do not run
well in Pi. Three read-only reviews (2026-09-15) found the causes below. Each
cause was checked against the code or a real Pi session log.

| # | Cause (verified) | Evidence |
|---|---|---|
| F1 | Pi reads `.agents/skills`, which is rendered for Codex. Pi sees `spawn_agent` / `wait_agent` / `codex exec` steps it cannot run, and no `--runtime pi` step. | `.agents/skills/.bee-render.json` `"target_runtime": "codex"`; `onboard/render.rs:364-370` has no Pi target; `skills/` has zero `bee:only pi` blocks |
| F2 | The Pi `detached_delivery` note tells the agent to pass "the session id the pi preamble shows you". The preamble shows no session id. | `verbs/drivers/prepare.rs:98`; no session-id line in `hooks/session_preamble/` |
| F3 | A hat wave cannot keep its 10-minute budget. The only working path is a foreground `herding run` with `--ceiling 1800`. Pi bash timeouts (600 s, 900 s, 130 s) killed runs and lost results. | Pi logs: `MAIN/…01a0a0a3….jsonl:1346-1350, 2057-2066`; `SRR/…01a09eab….jsonl:292` |
| F4 | A detached result has no seat name, so the leader cannot match a result to its hat. | marker `herding/run.rs:2183-2191`; drain header `.pi/extensions/bee-guard.ts:746-751` |
| F5 | `herding run --json` returns a one-line summary and `report_path`. No Pi-facing text tells the leader to read `report_path`. Every Pi session read `.bee/mailbox/job-*/report-1.md` by hand; no drain injection appears in any Pi log. | `herding/run.rs:3213-3245`; 68 Pi session files |
| F6 | The hat prompt carries only the generic advisor line on both runtimes. The seat's perspective and instrument do not reach the worker. | `packages/bee/prompts/advisor.md`; `prepare.rs:974-1001`, `:5175-5177` |
| F7 | A Pi session relocates into the feature worktree, and there `bee dispatch prepare` refuses ("refused inside a granted feature worktree"). The `cd main && …` workaround then trips the write guard. | `verbs/mod.rs:178-181`; Pi logs `MAIN/…01a0a0a3….jsonl:1337-1340, 1641`; `PWSR/…01a07a87….jsonl:577` |
| F8 | Hats dispatched without a feature scope ran in main, where the uncommitted plan did not exist, and came back blocked. | Pi log `MAIN/…01a0a0a3….jsonl:2059-2063`; cwd only set from `worktree_location`, `prepare.rs:2342-2350` |
| F9 | Pi contract tests dispatch only `--kind gather --role extraction`. No advisor, hat, or reviewer dispatch, and no real `--inbox-session` round trip. | `tests/pi_plugin_contracts.rs:2183-2401`, `:2518-2972` |

Checked and fine: the plugin works on Pi 0.85.1 (no event, field, or tool
schema changed from 0.84.4); the preamble already names `--runtime pi`
(`52012cb1`); concurrent job ids no longer collide; deploy `issuer_session`
was fixed by `deploy-issuer-session`.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Decision log: `a20cf301`.

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Every dispatch and result-collection step a Pi agent reads in its skills is one Pi can run. No Codex-only or Claude-only step is the only instruction Pi sees for a stage. Codex keeps its own steps. | F1. Pi and Codex share `.agents/skills`, so the fix must not break Codex. |
| D2 | The Pi detached-delivery instruction names a session token the agent can really get, and a test runs the command taken from that instruction. | F2. A test that writes the token itself proves nothing. |
| D3 | A Pi hat wave keeps its 10-minute budget: seats run in parallel without a foreground wait longer than the budget, each result arrives named by its seat, and a seat that misses the budget is dropped and named — never silently lost. | F3, F4, and the wave budget in `bee-hive/references/gates-and-delegation.md` ("Hat wave"). |
| D4 | The leader gets each worker's full answer, not only the one-line summary, on every Pi dispatch kind. | F5. |
| D5 | A hat dispatch carries its seat's perspective and instrument from their one existing home, on every runtime, without `--brief-file`. | F6. Single source of truth; the hat table stays the home. |
| D6 | A Pi leader inside its feature worktree can run the dispatch door without leaving the worktree and without a shell workaround. | F7. |
| D7 | An advisor, hat, or reviewer dispatch for a feature runs in that feature's worktree, where its plan lives. | F8. |
| D8 | Pi contract tests cover advisor, hat, reviewer, and cell dispatch, and one detached round trip through the result drain. | F9. |
| D9 | Pi worker dispatch stays herding-only (decision `9f5c6d17`). Supported host stays `@earendil-works/pi-coding-agent`; version labels say 0.84–0.85. | Existing locked decisions; 0.85.1 verified compatible. |

### Agent's Discretion

Planning picks the skill-render mechanism for the shared tree, the seat-name
field, the instrument source, the result-read shape, and how the dispatch door
reaches the control plane from a worktree. Existing Claude and Codex behavior
must not change except where D5 applies to every runtime.

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` — the dispatch door; herding payload, detached note, prompt render.
- `packages/bee-rs/crates/bee/src/onboard/render.rs` — `bee:only <runtime>` rendering per skill root.
- `packages/bee-rs/crates/bee/src/herding/run.rs` — job id, inbox marker, result envelope.
- `.pi/extensions/bee-guard.ts` — result-inbox drain and session token.
- `packages/bee-rs/crates/bee/src/verbs/mod.rs` — the granted-worktree refusal.
- `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` — Pi contract suite and sandbox.

### Established Patterns

- Runtime-specific skill text lives in `bee:only <runtime>` blocks; rendered roots strip the others.
- Herding carries the prompt on stdin; bee bookkeeping stays with the leader.
- Blocking Pi checks fail closed; advisory Pi checks fail open.

## Canonical References

- `skills/bee-hive/references/gates-and-delegation.md` ("Hat wave")
- `docs/history/pi-parity-review-fixes/CONTEXT.md`
- `docs/history/pi-full-workflow-parity/CONTEXT.md`
- `docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md`

## Outstanding Questions

<!-- bee:not-a-deferral: This historical section records questions the live Pi runs answered (psd-8 evidence: advisor-ref record succeeded from the bound worktree leader; every leader bash call passed its own timeout) and ideas already filed in the backlog. -->
### Deferred To Planning

- [ ] `advisor-ref record` refused in two Pi sessions ("no active feature to anchor the consult to", phase `idle` and `compounding-complete`). Reproduce from a Pi leader bound to a live lane first; fix only if it reproduces — otherwise record the failed reproduction.
- [ ] Does the Pi bash tool apply a default timeout when the model passes none? The schema says no default; confirm with a live run before relying on it.

## Deferred Ideas

- A herding worker pane that stalls on its CLI's feedback survey sits until the 900 s idle timeout (Pi logs, 4 lineages). It is a worker-CLI stall on every runtime, not a Pi dispatch path — in the backlog.
- Herding CLI shape friction (`pane --list`, timing prefix breaking `jq`) — in the backlog.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
<!-- /bee:not-a-deferral -->
