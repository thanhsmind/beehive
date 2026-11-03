# Prune reads the worktree capture queue by content — Context

**Feature slug:** prune-queue-copy
**Date:** 2026-10-15
**Shaping session:** complete (gate bypass full; the user asked to fix issue 106: "Vậy sửa lỗi 106 đi")
**Scope:** Quick
**Domain types:** RUN | ORGANIZE

## Feature Boundary

`bee worktree prune` keeps every worktree today, because each worktree's
`.bee/capture-queue.jsonl` holds a copy of main's queue (in this repo the file
is git-tracked, so every branch carries it; host repos gitignore it), and the
precious-state check keeps on any non-empty file
(https://github.com/thanhsmind/beehive/issues/106). This feature makes that
check read the queue's content, so a copy that holds nothing main lacks no
longer keeps the worktree. It ends at that one keep-condition; the handoff
check and every other prune condition stay as they are.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The capture-queue keep-condition fires only when the worktree's queue holds at least one PENDING stub — a `kind: "stub"` line whose `id` has no `kind: "flush"` line in the same file — whose `id` appears nowhere in the main checkout's `.bee/capture-queue.jsonl`. A stub main already holds, pending or flushed, is main's to answer. | Measured 2026-10-15: 31 worktree queues of 2576-2626 lines each, 0 stub ids absent from main's queue. |
| D2 | Every doubt still keeps (worktree-reclaim D2a, `f89d6cd6`): an unreadable worktree queue, an unparseable line, a line whose `kind` is neither `stub` nor `flush`, a stub line with no id, or a main queue that exists but cannot be read keeps the worktree. A missing main queue counts as empty, so any pending worktree stub keeps. | The guard protects work someone would be furious to lose; it relaxes only on proof. |
| D3 | The keep message names how many stubs are unique to the worktree and one example id, so a kept worktree says what to flush or merge. | A keep reason that cannot be acted on looks like the old false keep. |
| D4 | The `.bee/HANDOFF.json` keep-condition is unchanged. | Out of the issue. |

## Existing Code Context

- `packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs` `fn precious_state_present(worktree_root: &Path)` — the check to change; called from `classify_worktree` step (4), which has `check.main_root`.
- `docs/knowledge/areas/worktree-parallelism/pruning-dead-worktrees.md` row 4 — the concept to update.

## Outstanding Questions

(none)
