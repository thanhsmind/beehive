# worktree-first-enforcement — CONTEXT

**Backlog:** PBI p-a07607cb
**Date:** 2026-08-16
**Lane:** docs (doctrine + skill text only; no source code)

## What was asked

User hit a stall on beedashboard (bee 2.6.3): a session reported "blocked
by another session's cell (homepage-terminal-full holding views.rs +
app.js)" and stopped to ask the user how to proceed. User wants
continuous running: any code-touching action forks a worktree from the
start instead of editing main and hitting holds.

## What was found (evidence)

- The requested enforcement already ships. Write-guard worktree-first
  arms: `packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs:734-875`
  (granted arm + no-grant arm), route-time mirror
  `verbs/state_group/workflows.rs:638-674`, docs/tiny main privilege
  conditional on solo (decision b9639128, 2026-08-15).
- Cross-worktree same-path leases are advisory, not a hard deny:
  hard deny only when the holder's `workspace_id` matches the acting
  session's (`hooks/write_guard/checks.rs:366-374`). Two worktrees may
  edit the same file; they queue at `bee worktree merge` instead.
- The incident session's holder (homepage-terminal-full) DID work in a
  worktree (beedashboard commit e697df2, merged via `bee worktree merge`).
- Root cause of the stall: semantic overlap triage. The session's
  remaining backlog items all touched files an in-flight cell held and
  were likely to be swallowed by it; no doctrine told the agent to
  auto-defer them and continue, so it stopped and asked the user.
  Dispatch ranking (`skills/bee-herding/references/role-dispatch.md` §7)
  is also overlap-blind: it can dispatch an item straight into a
  guaranteed merge conflict.

## Decisions (locked)

- **D1 — Contention is triage data, never a user question.** When a
  backlog item's files overlap an in-flight cell or live worktree:
  prefer disjoint items first; split scope to the disjoint files when
  the split is natural; defer the overlapped remainder with a recorded
  reason ("likely swallowed by <cell>; re-triage after its merge").
  Report the deferral in one line and keep working. Ask the user only
  when the deferred set is the entire explicit ask.
- **D2 — Dispatch ranks overlap-aware.** The herding dispatch role
  checks candidate items against in-flight worktrees' claimed files and
  skips overlapped candidates this iteration (one chat-pane note), never
  spawning an agent into a known collision.
- **D3 — No guard/code change.** The Rust guard already enforces
  worktree-first; the no-grant arm's phase gate stays as regression
  history set it (decision 00504975). Doctrine and skill text only.

## Files

- `packages/bee/AGENTS.block.md` (Multi-session etiquette) + regen → `AGENTS.md`
- `skills/bee-herding/references/role-dispatch.md` (§7 rank step)
- `docs/knowledge/areas/worktree-parallelism/routing-and-visibility.md` (sync)

## Open questions

- None. Guard-arm phase-gate widening deferred (not implicated in the
  incident).
