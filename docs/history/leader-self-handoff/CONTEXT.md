# leader-self-handoff — CONTEXT

Owner ask (2026-11-24, Vietnamese): "Xem chỉnh việc này cho beehive chủ
động làm việc này không đợi như trên. Nên triển khai mechanic sao cho khi
triển khai project khác đều tự động." A bee leader hands off and follows up
through waggledance on its own. It never stops to ask the owner to open a
session or to re-run a step the agent can run. It is a bee mechanic, so
every onboarded project gets it on refresh.

Evidence:

- `bee state handoff send` dispatched with the fixed request id
  `bee-handoff-<session>-mark`. A second send from the same session
  returned the finished run `run-3481acb8c4fa9a75` and dispatched nothing.
  Backlog row under `pi-1-6-upgrade`.
- That run reported `done` while `wt/pi-1-6-upgrade` was never merged.
  The leader then told the owner to open a new session.

## Locked decisions

- D1. Each handoff dispatch attempt gets its own request id:
  `bee-handoff-<session>-<suffix>-<attempt>`, where `<attempt>` is the
  attempt's wall-clock milliseconds in base 36. One CLI run is one attempt;
  the waggledance request-id dedupe still guards a retried call inside it.
  `bee state handoff send` takes an optional `--follow-up-of <run>` that
  wins over the `PASEO_AGENT_ID` lookup and adds `--handoff`.
- D2. After a dispatch returns a run id, bee reads
  `waggledance runs show <run>`. A run whose status is `done` or `lost`,
  or whose `created_at` is earlier than the attempt start, is a dedupe of
  an old run. The outcome is `[handoff_deduped] run <id> was already
  <status>; nothing new was dispatched`. The session is not released and
  no ledger row is written. An unreadable `runs show` keeps the `Sent`
  outcome.
- D3. Every sent handoff writes one ledger file
  `.bee/handoffs/<request_id>.json` in the main checkout:
  `{request_id, run_id, project_id, session_id, feature, sent_at,
  task_file, expects, follow_ups, status}`. `expects` is a list of
  `{kind: "branch-merged", branch}`. bee adds `wt/<feature>` when that
  branch exists and main does not contain it at send time;
  `--expect-merge <branch>` on `handoff send` adds one more. `status` is
  `open`, `met` or `needs-owner`. One file per handoff, so two senders
  never share a write target.
- D4. A new verb `bee state handoff check [--run <id> --expect-merge
  <branch> [--feature <f>]] [--no-dispatch] [--json]` walks every `open`
  ledger row. `--run` first registers a row for a run that predates the
  ledger (request id `adopted-<run>`). For each row: a run that is not
  `done`/`lost` is `pending`. A finished run with every expectation met
  flips the row to `met`. A finished run with an unmet expectation
  dispatches a follow-up leader:
  `waggledance dispatch --project <p> --leader --follow-up-of <run>
  --handoff --request-id bee-handoff-followup-<run>-<n> --task-file <f>`,
  whose task names the unmet deliverable and carries the original task
  text. The row records the follow-up and now tracks the new run. After 3
  follow-ups the row becomes `needs-owner` and the check prints it.
  With no waggledance door the check lists unmet rows and dispatches
  nothing. `--no-dispatch` reports only.
- D5. The check runs on its own: the SessionStart hook and the Stop hook
  call it best-effort when the main checkout has at least one `open`
  ledger row. With no open row they call no waggledance process. A herding
  worker never runs it.
- D6. A turn-end net in the Stop hook. When the final assistant text asks
  the owner to open a new session, run `/clear`, or run a step the agent
  could run (a fixed English and Vietnamese phrase list), the session is a
  leader, `stop_hook_active` is not true, and a waggledance door is
  present, the Stop hook blocks the turn end. The reason names
  `bee state handoff send --task "<owner request; standing orders>"` and
  `bee state handoff check`. With no door it adds one advisory line and
  never blocks. A second stop in the same turn always passes.
- D7. Doctrine: `packages/bee/playbooks/handoff.md` and rule
  `agents-context-handoff-65` state D1 to D6 in leader terms: per-attempt
  ids, the deduped outcome, the follow-up check, and the turn-end net.

## Out of scope

- Any change to waggledance itself.
- Expectations other than a branch merge.
- Finishing `pi-1-6-upgrade` by hand in this feature: the live proof runs
  `bee state handoff check --run run-3481acb8c4fa9a75 --expect-merge
  wt/pi-1-6-upgrade`, and the dispatched follow-up leader finishes it.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md | suspect | holds the handoff send and feature start handoff contract |
| docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md | suspect | holds the context-mark door |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/handoff_door.rs` | request id, dispatch argv, task text |
| `packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs` | `run_handoff_send` |
| `packages/bee-rs/crates/bee/src/verbs/feature_group.rs` | feature start `--handoff-task` path and its tests |
| `packages/bee-rs/crates/bee/src/hooks/session_close/` | the Stop hook |
| `packages/bee-rs/crates/bee/src/hooks/session_init.rs` | the SessionStart hook |
| `packages/bee/playbooks/handoff.md` | handoff doctrine source |
