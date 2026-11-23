# parallel-session-unblock — CONTEXT

Owner ask (2026-11-23): with parallel sessions, a leader often stops and asks
the owner to pick between "turn worktree_first off and commit on main" and
"close the other Claude session". The cause: `bee worktree new` refuses while
another session is live and the checkout holds a shared nested checkout, and
its only remedy, `--with-companion`, needs `commands.worktree_companion_start`
and `commands.worktree_companion_mount`, which most projects do not have. The
owner also reports that fresh sessions do not hand off when they pass the
token mark.

Root causes:

- `verbs/worktree/handlers.rs` refuses at `if shared_nested_found {` and the
  same refusal sits in `verbs/feature_group.rs` (the
  `has_any_shared_nested_checkout` match). No branch checks whether the
  companion remedy is configured, so the refusal is a dead end.
- The context mark is a prompt line only (`hooks/prompt_context.rs`
  `compute_handoff_warning`). It shows on a user prompt, so a leader that runs
  a long autonomous turn never sees it, and nothing refuses new work past the
  mark. The only hard door (`handoff_due` in `bee feature start`) fires after
  a merge, not on context size.

## Data shape

- `CompanionRemedy`: `Configured` | `NotConfigured`, read from
  `commands.worktree_companion_start` and `commands.worktree_companion_mount`
  (both non-empty → `Configured`).
- Session record field `context`: `{ "tokens": u64, "measured_at": iso }`,
  written by the PreToolUse hook only for a Bash call that runs a bee verb,
  from the payload's `transcript_path` through
  `devtools::statusline::context_and_turns`. Keyed by the payload's session id.
- `MarkDue { tokens, mark }`: a pure function
  `context_mark_due(config, session_record) -> Option<MarkDue>`;
  `handoff_context_tokens` (default 120000, `0` off) is the mark.
- `HandoffReason`: `MergedFeature { merged }` | `ContextMark { tokens, mark }`,
  passed to `handoff_door::compose_task_text` so one door carries both reasons.

## Locked decisions

- D1 (`e37ebc32`). With another session live and a shared nested checkout
  found, `bee worktree new` and `bee feature start` refuse only when
  `CompanionRemedy::Configured`. With `NotConfigured` they create the worktree
  and print one stderr warning line: the nested checkout stays guarded by the
  write guard while sessions are concurrent
  (`is_shared_nested_checkout_target`), and `--with-companion` is available
  once the two companion commands are set. A detection error still fails
  closed.
- D2 (`5380068f`). The context mark becomes a door. When the calling
  session's `context.tokens` is at or past the mark, a waggledance door is
  present and `handoff_auto` is on, these verbs refuse with
  `[handoff_due] context <N>k is past the <M>k mark; run bee state handoff
  send --task "<owner request and standing orders>"`:
  `dispatch prepare` with `--claim`, `dispatch wave`, `cells claim`,
  `cells claim-next`, and `feature start` (which also accepts its existing
  `--handoff-task` for this case). Mid-cell verbs (`cells cap`,
  `cells finish`, `worktree merge`, `dispatch prepare` without `--claim`) are
  never refused. A session with `BEE_HERDING_WORKER=1` is never refused. With
  no door or `handoff_auto` false, nothing is refused; the prompt line stays.
- D2a. New verb `bee state handoff send --task "<text>"`: builds the task
  with `HandoffReason::ContextMark`, the current feature and its open or
  claimed cells, runs `handoff_door::execute_handoff` (request id
  `bee-handoff-<session>-mark`), and on `Sent` releases the session, appends
  audit kind `handoff-dispatched`, and fails `[handoff_sent] run <id>: this
  session is released; tell the owner in one line and stop.` On `Failed` it
  fails `[handoff_failed] <reason>` and writes nothing. It refuses with no
  door present.
- D3. Doctrine (rule `agents-context-handoff-65`, playbooks `handoff.md` and
  `multi-session.md`) says: a worktree refusal for a shared nested checkout
  is never a reason to ask the owner to turn `worktree_first` off or to close
  another session; on `[handoff_due]` for the context mark the leader runs
  `bee state handoff send` at once.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md | suspect | holds the companion mount contract |
| docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md | suspect | holds the context mark prompt line |
| docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md | suspect | holds the handoff_due door this feature widens |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs` | the nested-checkout refusal at worktree new |
| `packages/bee-rs/crates/bee/src/verbs/feature_group.rs` | the same refusal at feature start, and handoff_due |
| `packages/bee-rs/crates/bee/src/nested_checkout.rs` | the down-scan |
| `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs` | the context mark prompt line |
| `packages/bee-rs/crates/bee/src/verbs/handoff_door.rs` | the waggledance handoff door |
| `packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs` | the session record and release |

## Skips

- architect skipped: owner-chosen behavior, one data field and one pure check.
- statechart skipped: the retry hazard is double send, closed by the
  deterministic request id.
- tutorial skipped: no other code calls these paths.
- `bee-how` gather skipped: the leader read the four refusal and mark sites
  directly before this brief.
