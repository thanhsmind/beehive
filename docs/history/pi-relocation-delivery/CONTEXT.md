# Pi relocation delivery — Context

**Feature slug:** pi-relocation-delivery
**Date:** 2026-09-16
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | CALL

## Feature Boundary

A Pi leader session that moves between the main checkout and its feature
worktree keeps everything the move used to drop: the results of detached
herding jobs launched before the move, and ownership of the cell it claimed.
The Pi runtime blocks in the skills describe every dispatch mechanism a Pi
leader or worker needs, with no Claude-only or Codex-only row left unanswered.

## Why now

A live end-to-end eval on 2026-09-15 (run `20260915-224413-891129`) drove one
standard feature through the whole bee lifecycle with a Pi leader
(`pi-gpt-5.6-luna`) and found the gaps below. Each was checked against the code
or the run's own evidence files.

| # | Finding (verified) | Evidence |
|---|---|---|
| F1 | Relocating into the worktree opened a new Pi session id. The cell worker had been launched with `--inbox-session <old id>`, so its marker stayed under `.bee/result-inbox/01a0a5be…/` and was never injected. The leader idled out its 900 s heartbeat window and had to be woken by hand. | `.pi/extensions/bee-guard.ts:841-844` (`startResultDrain` keys the drain on the current session id only); session header `parentSession` in `~/.pi/agent/sessions/…repo--wt--greet-shout--/…01a0a5c9….jsonl:1`; evidence log `pi-leader-log.md` ("worker-result delivery friction") |
| F2 | Relocating back to main repeated F1 for the goal-check job (`job-1789489409621-1048390-1`, marker stranded under `01a0a5c9…`). Delivery itself is sound when no relocation happens: three hat seats and one compound gather were injected normally. | marker dirs under `<run>/repo/.bee/result-inbox/`; injected `bee-result` blocks at 15:52:11Z, 15:52:28Z, 15:52:57Z and 17:08:32Z in the leader session log |
| F3 | After relocation the cell claim still named the pre-relocation session, which bee reports as "holder not alive", so the cap needed `--force-ownership` (audited) for work the same conversation had done. | `bee cells finish` refusal in `pi-leader-log.md` ("cap with proof"); `cells list` output naming session `01a0a5be…` |
| F4 | No contract test drives relocation and the result drain together; each is covered alone, which is why F1/F2 shipped unnoticed. | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` — relocation cases at `:4291-4300`, `:6194`; inbox cases elsewhere, no case combining them |
| F5 | A Pi worker handed an `Advisor:` line has no transport instruction it can run: the skill offers a model-shaped advisor (Claude), a Codex-native one, and a cli-shaped one, while `team.pi.advisor` is herding-shaped. | `skills/bee-swarming/references/worker-details.md:272-281`; `.bee/config.json` `team.pi.advisor` = `{"kind":"herding","agent":"pi-gpt-6-astra"}` |
| F6 | The Pi spawn-mechanics table carries one row (Result collection) where Claude and Codex carry seven: Spawn, Model, Result collection, Follow-up/rescue, Harness assist, Isolation guarantee, Subagent type. | `skills/bee-swarming/references/swarming-reference.md:375-404` |

Checked and fine, and therefore out of scope: the hat wave (three seats, 2 min
38 s, each result named by its seat), `advisor-ref record`, both gate
auto-approvals under `gate_bypass: full`, `worktree new`, cell dispatch through
`dispatch prepare --runtime pi --kind cell`, merge, judge, capture, compounding,
and the `uat` stop. Friction that is not Pi's: the hand-computed
`roster_sha256`, a missing `--kind`, the skipped `planning → swarming` phase
transition, and the leader's own worktree-first slip.

## Locked Decisions

These are fixed. Planning implements them exactly — cited, never reinterpreted.

| ID | Store ID | Decision | Rationale |
|----|----------|----------|-----------|
| D1 | `0833887d-9005-46de-84f4-df265da53cbb` | A Pi session that relocates between the main checkout and a feature worktree keeps receiving the results of detached herding jobs launched before the move. | F1, F2. |
| D2 | `52d1e3aa-97d3-4a0d-a89a-67f0822ec695` | A cell claim held by a Pi session survives that session's relocation: capping after the move needs no `--force-ownership` override. | F3. |
| D3 | `b527603f-226d-4a19-96e1-7feaecf090d4` | One Pi contract test drives relocation and detached result delivery together: a job launched before the move is delivered after it. | F4. |
| D4 | `3cb3523c-7aef-4b22-aa90-17970371dae7` | The Pi runtime blocks in the skills name every dispatch mechanism a Pi leader or worker needs: the worker-side advisor transport for a herding-shaped advisor, and the Pi spawn-mechanics rows Claude and Codex already carry. | F5, F6. |

These stay as written and constrain the work:

- `pi-worktree-session-relocation` D1/D2 — relocation is session replacement via
  `SessionManager.forkFrom` + `ctx.switchSession`, never a process `cwd` change.
  A fix must not reach for a cwd move instead.
- `pi-result-mailbox` — delivery is at-least-once with `job_id` as the dedupe
  key, and the injected block is header-only. A carried-over result is still one
  delivery of that job, so a job already injected must not be injected twice.
- `pi-stage-dispatch` D9 — Pi worker dispatch stays herding-only.

### Agent's Discretion

Planning picks where the carry-over happens (the belt at relocation time, the
drain's token resolution, or a bee verb the belt calls), how the claim follows
the session, the test's shape, and the wording of the skill rows — provided
Claude, Codex and OpenCode behavior does not change.

## In Scope

1. Result delivery across both relocation directions (enter, and exit before merge).
2. Cell-claim continuity across relocation.
3. One contract test covering relocation plus detached delivery.
4. The two Pi skill gaps (F5, F6).

## Out of Scope

- Any change to how relocation itself works (session replacement stays as locked).
- Delivery behavior when no relocation happens — it works today.
- Claude, Codex, and OpenCode dispatch behavior.
- The runtime-neutral friction listed above (`roster_sha256`, phase transition,
  worktree-first); those are separate backlog items, not this feature.

## Existing Code Context

### Reusable Assets

- `.pi/extensions/bee-guard.ts` — the belt: relocation handling, `startResultDrain`,
  `usableInboxToken`, `drainResultInbox`, `reclaimOrphanClaims`.
- `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs:50-61` — the
  transition intent, which already carries `piSessionId`.
- `packages/bee-rs/crates/bee/src/herding/run.rs` — where the `--inbox-session`
  marker is written.
- `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` — the Pi sandbox and
  its relocation and inbox cases.

### Established Patterns

- Blocking Pi checks fail closed; advisory Pi checks (the drain included) fail open
  and never throw into the session.
- Runtime-specific skill text lives in `bee:only <runtime>` blocks.

## Outstanding Questions

None. D1-D4 fix the product boundary; the mechanism is planning's to choose.
