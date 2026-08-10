# Learnings — close-lands-bookkeeping (2026-08-10)

Feature: `bee close` now auto-commits its own tracked `.bee` bookkeeping
(path-scoped) after a green close — R81 in
`docs/knowledge/areas/workflow-state/gates.md`; backlog P2 row 708.

## What the incident taught

1. **A tool that writes tracked files owns landing them.** Close wrote
   bookkeeping into main's tracked bee-store and left the commit to whoever
   came next; combined with harness worktree isolation (which blocks a
   worktree-pinned session from any git against main) the flow dead-ended at
   `WORKTREE_MERGE_MAIN_DIRTY` with no agent able to clear it. The fix is at
   the writer, not the checker: the auto-commit landed, and its first live run
   was its own feature's close (`bookkeeping_commit: c3c03655`). The rejected
   alternative — exempting bee-store paths from merge's dirty check — would
   have hidden the dirt without landing it.

2. **Session worktree-pinning is transitive to subagents.** A dispatched
   worker inherits the orchestrator session's physical home; bee's write-guard
   correctly refused a cross-worktree write from a main-homed session's
   worker. The working pattern: move the SESSION into the feature worktree
   (EnterWorktree) before dispatching execution workers, exit back to main for
   integration verbs (merge, close). One session can orchestrate both sides,
   but only by moving, never by reaching across.

3. **Every bee state write from a session `cd`-ing into another checkout lands
   under that checkout's identity.** Same family as the ks-2 holder-"main"
   reservation pitfall already recorded in
   `worktree-parallelism/cross-worktree-holds.md`: run state-writing verbs
   from the checkout whose identity should own the row.

## Capture batch riding this run

The same compounding session flushed the whole capture queue (7 stubs, 5
features) into specs: harness-audit-hardening (help surface, onboarding merge
refusal, managed ignore, doc de-drift gap), preamble-surface-slim
(grouped-name briefing surface — stale flag-per-line spec section replaced),
knowledge-usable residual gaps (R82 pattern-check door, R83 promote-stub
convergence, dangling-source finding, search/bootstrap/report entry points),
and the two promote-proposal pointers (knowledge-search: already carried by
B10, area tags over-broad; close-lands-bookkeeping: empty proposal, R81
already recorded).
