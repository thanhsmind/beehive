# Context: merge-ready-fact

## Problem

The waggledance board has a "Ready to merge" column for features that are finished in
their worktree and wait for the human to merge. bee stores no such fact: a reader must
join worktree grants, every cell of the feature, and the lane's gates to derive it.
The user (2026-08-22, via the waggledance session) asked for a stored, file-readable fact.

## Locked decisions

Logged in `.bee/decisions.jsonl`; ids are the authority.

- **D1** (`cfccdde4`) — the feature record carries an optional `merge_ready`
  `{since, branch, worktree_id, uat: pending|approved, blocked_by: []}`; set by the cap
  that leaves zero open/claimed cells when a worktree grant exists for the feature; a
  zero-cell feature never gets it.
- **D2** (`e069ef73`) — `bee close` rewrites `blocked_by`; `bee gate --name uat` flips
  `uat`; `worktree merge` / `worktree unregister` delete it; any reopen of a cell of the
  feature deletes it (the next last-cap re-sets it).
- **D3** (`f2c16247`) — additive projection only: bee's own gates and merge door never read
  it; `bee status --json` lane rows surface it verbatim.

## Out of scope

waggledance's reader; changing uat/merge preconditions; any sweeper or TTL.
