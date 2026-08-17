# worktree-keep-on-merge — plan

Lane: standard (flags: data-model, covered-contract-change). Route: 6 product files.

## Goal

`bee worktree merge` keeps the worktree by default. It records the merged
worktree in the deferred queue (`.bee/deferred-queue.jsonl`) so the user can
cross-check it later. `bee worktree prune` stays the cleanup path and resolves
the queue entry when it removes the worktree.

Locked decision: worktree-keep-on-merge D1 (supersedes worktree-reclaim D1,
866cc946).

## Current behavior (evidence)

- Cleanup default-on: `resolve_cleanup_on_merge` at
  `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs:426` —
  `!no_cleanup_flag && config_enabled` (config key `worktree_cleanup_on_merge`,
  default true). `--cleanup` flag accepted but a no-op (`worktree/mod.rs:9-11`).
- Teardown: `perform_cleanup` (`merge.rs:596`) → `teardown_worktree`
  (`merge.rs:546`): `git worktree remove --force`, `git branch -d`, grant drop,
  `unregister_workspace`, `release_all_for_holder`.
- Deferred queue: `verbs/deferred_queue.rs`, fixed `KINDS =
  capture|scribe|review|promote`, `enqueue(root, kind, feature, cells, areas,
  files, reason)` appends to `.bee/deferred-queue.jsonl`.
- Prune: `run_prune_core` (`prune.rs:557`) classifies via `classify_worktree`
  and removes `Verdict::Dead` worktrees with the same teardown steps.
- `worktree list` (`registry.rs:186-215`) prints grants only; no merged/stale
  marker.

## Shape — one slice, 3 cells

### wkm-1 — merge keeps worktree, queues a worktree-cleanup entry

- Flip the default: merge performs teardown only when `--cleanup` is passed or
  `worktree_cleanup_on_merge` is explicitly `true` in `.bee/config.json`.
  Re-arm the `--cleanup` flag; `--no-cleanup` stays as an explicit keep (wins
  over config true).
- On the keep path (merge green, no teardown): add kind `worktree-cleanup` to
  `KINDS` and `enqueue` one entry — `feature` = worktree's feature slug (from
  creation identity; fall back to worktree id), `files` =
  `[<worktree_root>]`, `reason` = one line naming worktree id, branch, merge
  commit sha, and the remove command (`bee worktree prune`). Keep
  `cleanup_suggested_command` in merge output.
- Registration (grants + workspace record) stays — prune must still find it.
- Existing tests asserting cleanup-on-merge default flip to the new default;
  add coverage: keep path enqueues exactly one entry; `--cleanup` still tears
  down and enqueues nothing.

### wkm-2 — prune resolves the queue entry

- When `run_prune_core` removes a worktree (including record-only orphans), it
  resolves any unclaimed `worktree-cleanup` queue entries whose `files`
  contain that worktree root (or whose reason names the id), using the queue's
  existing event model (claim/resolve event append — follow
  `deferred_queue.rs` precedent; no hand-edit of the JSONL).
- `--dry-run` resolves nothing.
- Tests: prune removal resolves the matching entry; kept worktrees leave the
  entry untouched.

### wkm-3 — `worktree list` shows the kept-merged marker

- `run_list` cross-references unresolved `worktree-cleanup` entries: an id
  with a pending entry prints `<id> (granted, merged — pending cleanup)`.
- Tests: list output with and without a pending entry.

## Test scoping

`commands.test` (workspace cargo test) at every cap; new cases live beside the
touched modules (`worktree/merge.rs`, `worktree/prune.rs`,
`worktree/registry.rs`, `deferred_queue.rs` tests).

## Risks

- Disk growth: worktrees accumulate by design; `bee worktree prune` is the
  drain and its classifier already fails closed (worktree-reclaim D2a).
- Herding merge flow inherits the keep default — acceptable; prune drains.
- Queue kind addition touches a fixed KINDS list consumed by status/preamble
  surfaces; wkm-1 must sweep those match arms (`status_full`,
  `session_preamble/store.rs`, `chain_nudge.rs`) for exhaustive-match breaks.
