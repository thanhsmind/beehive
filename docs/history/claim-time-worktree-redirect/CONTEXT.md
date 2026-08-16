# claim-time-worktree-redirect — CONTEXT

**Date:** 2026-08-16
**Lane:** standard (Rust claim path + doctrine/prompt text)
**Follows:** worktree-first-enforcement (docs-only, D3 "no guard/code change")
**Amended:** same day, after plan-check found two P1s in the refusal-shaped
v1 (see "What the plan-check corrected").

## What was asked

User hit the write-guard cross-worktree deny again on beedashboard: a
main-checkout session claimed a cell, then tried to Edit a file inside the
feature's granted worktree. The guard denied mid-edit; the session then
planned "unclaim, dispatch bee-build worker into the worktree" — itself a
dead end. User wants the flow to run correctly BEFORE the guard fires
("tránh tự báo tự sửa"): the first machine signal moves to claim time,
the guard stays the safety net.

## What was found (evidence)

- The containment deny keys purely on the hook payload's cwd vs the
  target realpath (`hooks/write_guard/hook_local.rs:536-573`). No
  exemption exists for dispatched workers: `BEE_AGENT_NAME` only feeds
  reservation-conflict checks (`checks.rs:514-539`).
- A Task-tool subagent inherits the parent session's OS cwd, so a
  bee-build worker dispatched FROM a main-checkout session cannot write
  into the worktree — the same deny fires on it.
- `cells claim` / `claim-next` never surface the feature's granted
  worktree (`handlers_write.rs:602-646`, `handlers_select.rs:658-843`).
  The wrong flow meets its first machine signal mid-edit, after claim.
- **Claiming from main is correct by bee's own topology.** The narrow
  door refuses `cells claim`/`claim-next` INSIDE a granted worktree
  (test `tests.rs:4750`, knowledge
  `docs/knowledge/areas/worktree-parallelism/control-plane-topology.md`:
  "finish takes the FULL door … every other cells verb stays narrow").
  `dispatch prepare --claim` also only runs from main and its
  `worktree_location` prompt block is a tested feature
  (`drivers/tests.rs:975-1035`). So the defect is never the claim — it
  is executing (editing) from main after it.

## What the plan-check corrected

v1 locked a claim-time REFUSAL. Plan-check proved that deadlocks: the
refusal's remedy ("claim from the worktree") is itself refused by the
narrow door (P1-a), and the refusal disables `dispatch prepare --claim`/
`wave` for every worktree-owning feature (P1-b). v2 replaces refusal
with annotation; no door changes, no dispatch changes.

## Decisions (locked, v2)

- **D1 — Claim annotates the execution location.** When the claimed
  cell's feature has a granted worktree, `cells claim` and
  `cells claim-next` append one line to their success output naming the
  worktree root: execution runs from a session rooted there; main must
  not edit it, and a subagent dispatched while cwd is main inherits main.
  The JSON result gains `worktree_root`. Unresolvable grant entries fail
  open (annotation silently absent), reusing the three-state
  `find_feature_worktree_grant` so fail-open is a testable case.
- **D2 — No refusal, no skips, no new switch.** The annotation is
  informational and unconditional (no `worktree_first` coupling); claim
  and claim-next semantics are otherwise byte-identical. The narrow-door
  contract ("claims run from main") stands unchanged.
- **D3 — Workers stop, never fight the guard.** `worker-cell.md` gains a
  first-step self-check: if the effective cwd is not `{{worktree_root}}`,
  return `[BLOCKED: session cwd is not the worktree — enter it or spawn
  the worker from a session rooted there]`, zero edit attempts. The
  existing Location block stays (it serves pane/cockpit workers whose
  cwd IS the worktree).
- **D4 — Doctrine names the dead end and the move.** bee-swarming and
  the AGENTS block state: claim from main, then MOVE THE SESSION into
  the worktree before dispatching execution workers (EnterWorktree on
  Claude Code, or a session/pane opened at the worktree path — the
  herding pattern); dispatching an execution worker while cwd is main
  cannot write into the worktree.
- **D5 — Guard unchanged.** The write-guard arms stay exactly as
  regression history set them.
- **D6 — Testability.** The new annotation check threads explicit paths
  (store root, cell feature) — never `std::env::set_current_dir` in
  tests; claim's acting-cwd is irrelevant to the annotation by design.

## Files

- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs` (claim)
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_select.rs` (claim-next)
- `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs` (new coverage; `wf_worktree_fixture` exists at :4644)
- `packages/bee/prompts/worker-cell.md`
- `skills/bee-swarming/SKILL.md`
- `packages/bee/AGENTS.block.md` (+ regen `AGENTS.md`)
- `docs/knowledge/areas/worktree-parallelism/routing-and-visibility.md` (sync)

## Open questions

- None.
