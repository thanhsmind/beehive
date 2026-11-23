# auto-handoff-on-due — CONTEXT

Owner ask (2026-11-23): the handoff reminder is skipped. Five `handoff-waived`
rows in a host project show leaders opening a second feature with
`--same-session "<owner asked ...>"`, and the one handoff written was a
`pause`. The owner chose option B: bee performs the handoff itself.

Root cause: the context mark is a prompt line only
(`hooks/prompt_context.rs` `compute_handoff_warning`), and the one hard stop,
`bee feature start` `handoff_due` (`verbs/feature_group.rs`), takes any
non-empty agent-written reason. Bee never runs `waggledance dispatch`.

## Data shape

- `HandoffDoor`: `Absent` | `Present { binary, project_id }`. Present when a
  `waggledance` binary is on PATH (the existing probe in
  `hooks/write_guard/cross_project.rs`) and `waggledance list --json` holds a
  project whose `root_path` is this repo's main root; `project_id` is that
  row's id.
- `HandoffRequest { project_id, follow_up_of: Option<String>, request_id,
  task_file }` built by a pure function; a second pure function turns it into
  the argv.
- `HandoffOutcome`: `Sent { run_id, request_id }` | `Failed { reason }`.

## Locked decisions

- D1. `bee feature start` that hits `handoff_due` with a present door and
  `handoff_auto` on runs the handoff itself:
  `waggledance dispatch --project <id> --leader [--follow-up-of <run> --handoff]
  --request-id <id> --task-file <file>`. On success it releases the session
  (same effect as `bee state session release`), writes no lane and no
  worktree, and exits non-zero with `[handoff_sent] run <run_id>: this session
  is released; tell the owner in one line and stop.`
- D2. The task text comes from a new flag `--handoff-task "<text>"`: the
  owner's request in their words plus standing orders. On `handoff_due` with a
  present door and no `--handoff-task`, the start refuses with
  `[handoff_due]` naming `--handoff-task` as the only way on. Bee writes the
  task file under `.bee/tmp/` with a header it composes: the feature merged in
  this session, the requested slug, class, lane, files and flags, and the
  exact `bee feature start` command the successor runs, then the
  `--handoff-task` text.
- D3. With a present door and `handoff_auto` on, `--same-session` is refused
  (`[handoff_due] --same-session is not accepted while bee can hand off`).
  `handoff_auto` is a boolean config key, default true, and governed: only
  the owner edits it by hand; `bee config set` refuses it. With the door
  absent or `handoff_auto` false, today's behavior stays: refusal plus the
  `--same-session "<reason>"` waiver and its `handoff-waived` audit row.
- D4. `follow_up_of` is the id of the run in
  `waggledance runs --project <id> --full` whose `pane_id` equals
  `paseo:<PASEO_AGENT_ID>`. No env var or no match: plain `--leader`, no
  `--follow-up-of`, no `--handoff`. `request_id` is
  `bee-handoff-<session id>-<feature>`, so a retry never sends twice.
- D5. Success appends audit row kind `handoff-dispatched` (note carries run id
  and request id). A failed dispatch (non-zero exit, unparsable output)
  refuses with `[handoff_failed]` and the stderr tail, releases nothing, and
  writes no audit row; it never falls back to a waiver.
- D6. This path writes no `.bee/HANDOFF.json`: the waggledance task carries
  the handoff. The doctrine (rule `agents-context-handoff-65`, playbook
  `handoff.md`) tells the leader to pass `--handoff-task` and never to waive.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md | suspect | holds the handoff_due door and the same-session waiver this feature changes |
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | unverified | holds the waggledance CLI-first leader door bee now calls |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/feature_group.rs` | the handoff_due refusal and the waiver audit row |
| `packages/bee-rs/crates/bee/src/integration_queue.rs` | `record_merger` and `merged_feature_of`, the merged-in-this-session fact |
| `packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs` | the waggledance binary probe and the registry read |
| `packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs` | session release |
| `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs` | the context-mark prompt line, left as is |

## Skips

- architect skipped: one obvious shape, two pure functions and one shell call.
- statechart skipped: the only retry hazard is double send, closed by the
  deterministic request id (D4).
- tutorial skipped: no other code calls this path.
- Out of scope: a handoff for a session that passes the 120k mark inside one
  feature (no feature start happens); the prompt line stays as is.
