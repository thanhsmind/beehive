# device-aware-cap — CONTEXT

## Ask

The user read a post about tools that know the machine's power and clean up
after a swarm. They chose option A2: make bee's herding limit aware of the
machine, and run the worktree cleanup on its own at close.

## Found

- `herding.limits.concurrency` exists (`packages/bee-rs/crates/bee/src/herding/wave.rs:1129`)
  but is a fixed number, unset in this repo, so no cap applies today.
- `limits.unverifiable` refuses a run when no pane list is readable
  (`wave.rs:1131`). A default cap must not inherit that refusal.
- `bee worktree prune` exists and fails closed
  (`packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs:610`, `run_prune_core`).
  Nothing runs it on its own.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Unset `herding.limits.concurrency` defaults to CPU count / 4, never below 1. An explicit value wins. A default limit never refuses as `limits.unverifiable`. | a9e06a0f, 4bb9da28 |
| D2 | Refuse a new worker when available memory is below `herding.limits.min_free_mem_gb` (default 4). Unreadable memory skips the check. | 9bbef1a9, 4bb9da28 |
| D3 | A green `bee close` runs the prune sweep once from main; a prune failure is one line and never turns the close red. | 14b76800, ea534256 |

## Open questions

None.
