# run_git_stdin Thread — Context

**Feature slug:** run-git-stdin-thread
**Date:** 2026-10-17
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

One fix from backlog `run-git-stdin-deadlock` (P2, found by the pstack-0-15-port
slice judge): `run_git_stdin` writes the whole input to the child's stdin before
reading stdout, so `git patch-id --stable` on a branch of roughly 800+ commits
fills its 64 KiB stdout pipe, stops reading, and `bee worktree merge` hangs.
Plus the typo `A cleaner rebase` → `A clean rebase` in the merge-gate concept.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `run_git_stdin` moves the input into a spawned thread that does the `write_all` and drops the handle, then the parent calls `wait_with_output`; the thread is joined after and its result ignored (a broken pipe is the child exiting early, which the exit status already reports). `GitOut` and every fail-open path stay as they are. | A hang is not fail-open (pstack-0-15-port D2). |
| D2 | No new test: the existing rebase and stale tests pin behavior; an 800-commit fixture is not worth its runtime. | Named proof scope. |
| D3 | Lane `tiny`, one inline cell on main — no other session is live. | AGENTS.md: main takes a solo tiny fix when no other session is live. |

## Existing Code Context

- `packages/bee-rs/crates/bee/src/verbs/worktree/git.rs:103-130` — `run_git_stdin`, `write_all` before `wait_with_output`.
- `docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md:156` — `A cleaner rebase is not stale proof`.
