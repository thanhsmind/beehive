# Gate 2 report — pi-worktree-session-relocation

## Recommendation

Approve the merged shape and execution plan in
[plan.md](../plan.md). The selected route uses only Pi 0.85.1's supported
session-control context. It also preserves the normal agent-owned bee CLI flow.

## Reality gate

| Check | Result | Evidence |
|---|---|---|
| Outcome is nameable | PASS | One settled Pi conversation enters a verified worktree and returns to main before merge. |
| Target behavior is supported | PASS | Pi 0.85.1 provides `SessionManager.forkFrom`, registered command context `switchSession`, and expanded extension-command dispatch. |
| CLI integration point exists | PASS | `worktree new` has a final success result builder; linked `worktree merge` refuses before merge phases. |
| Verified identity exists | PASS | Worktree `Pre` carries Git-verified id and main root; strict grants remain authoritative. |
| Agent path exists | PASS after revision | CLI markers captured at `tool_result` become a private command only after `agent_settled`. |
| Failure recovery exists | PASS after revision | New zero-mutation `worktree enter --id` returns to an existing grant after merge refusal. |
| Data migration needed | NO | Session forks use Pi's current JSONL format. Bee store shape does not change. |
| Late switch rollback is fully atomic | NO, bounded | Pi invalidates the old runtime before target service construction completes. The plan names this upstream boundary and makes no false rollback claim. |

## Advisor consult

Five high-risk perspectives were prepared.

| Perspective | Result |
|---|---|
| User impact | Completed: 2 BLOCKER, 3 WARNING. Both blockers and two warnings changed the plan. |
| Value | Completed: 0 BLOCKER, 3 WARNING. Two warnings changed the plan; one was rejected as outside locked value. |
| Facts and gaps | Dropped: prepared runtime required an Agent tool unavailable in this session. |
| Alternatives | Dropped: prepared runtime required an Agent tool unavailable in this session. |
| Risks | Dropped: prepared runtime required an Agent tool unavailable in this session. |

No hard quorum applies. The leader checked the dropped perspectives during
synthesis. Source anchors, smaller paths, replay, path identity, ordering,
concurrency, and reversibility appear in plan.md.

### Accepted findings

1. **Agent-created worktree could not relocate.** Added a Pi-only CLI marker,
   `tool_result` capture, and private extension-command dispatch after
   `agent_settled`.
2. **Merge refusal stranded the user on main.** Added `worktree enter --id` and
   its Pi command as a verified recovery path.
3. **Relocation lacked activity feedback.** Added idle refusal and visible cwd,
   branch, and history-preserved status.
4. **TypeScript duplicated Rust flag validation.** Reduced belt validation to
   safe tokenization and transition validation. Rust keeps flag ownership.
5. **Self-hosting reload added lifecycle risk.** Removed it.

### Rejected finding

A standalone exit-without-merge mode was not added. D4 requires exit before
merge, not general navigation. Existing-worktree enter solves the recovery case
without widening the locked outcome.

Advisor source reports:

- `/home/thanhsmind/Projects/goglbe/beehive/.bee/mailbox/job-1788763958472/report-1.md`
- `/home/thanhsmind/Projects/goglbe/beehive/.bee/mailbox/job-1788764143125/report-1.md`

## Plan structure check

| Dimension | Verdict | Reason |
|---|---|---|
| Scope | PASS | Three cells cover CLI contract, Pi integration, and mapped verification. |
| Evidence | PASS | Each load-bearing claim names code or Pi 0.85.1 source. |
| Dependencies | PASS | CLI precedes belt; belt precedes live map. Serial order is explicit. |
| Failure design | PASS | Pre-switch, cancellation, late runtime, merge refusal, replay, path, and compatibility cases are explicit. |
| Proof | PASS | Targeted Rust/Node contracts, sandbox CLI behavior, mapped Pi flow, and the full declared suite are required. |

## Cold-pickup check

- **CRITICAL:** None. Each cell has owned files, ordered work, dependency, and proof.
- **MINOR:** The exact user-facing documentation file beyond the verify-app map
  is selected during pwsr-3. This is intentional because the scribing pass must
  extend the existing one-fact home after implementation settles.

## Execution cells

1. `pwsr-1`: emit verified CLI intents and add existing-worktree enter.
2. `pwsr-2`: consume intents through supported Pi command contexts.
3. `pwsr-3`: map and drive the complete user-visible flow.

## Risk controls

- No `process.chdir`.
- No session switch from an event context.
- No shell execution for command arguments.
- No caller-supplied transition path.
- No merge before the replacement session is active on main.
- No worktree deletion by enter or exit intent.
- No automatic retry after merge refusal.
- No claim that Pi's post-teardown runtime failure is extension-rollback-safe.

## Approval meaning

Approval authorizes the three serial cells in plan.md. It does not authorize a
plain navigation mode, another harness, worktree deletion during exit, or a Pi
runtime patch.
