# Gate 2 (shape + execution) — hold-holder-attribution — plan rev 1

Recorded under `gate_bypass: normal` — lane `standard`, no hard-gate flag, so the
level covers this gate. The recommendation below is what the approval selected.

## Route

class `feature` · lane `standard` · flags `public-contracts, covered-contract-change,
multi-domain` · product files 6 · worktree `beehive--wt--hold-holder-attribution`

## Green base

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
→ `1668 passed, 11 ignored (16 suites, 19.34s)`, exit 0, run before any claim.

## Review before the gate

A plan review ran against the source before this gate and returned
`PLAN: NEEDS …`. It confirmed all six defect claims against their anchors, corrected
two of them (`reserve.rs:339` is the stamp, `:338` is its comment; "only the TTL
clears them" is wrong — `state_sync.rs:818-831` renews a live session's rows forever,
so nothing clears them), and found one blocker plus five gaps. The plan was revised to
rev 1 and the gate re-recorded against it; `bee state plan-rev bump` invalidated the
rev-0 approval first, so nothing was claimed against the superseded shape.

The blocker: rev 0 fixed only the write side, which would have turned "rows the
worktree cannot clear" into "rows main cannot clear" — `release.rs:171` and
`finish_support.rs:622` both filter by the acting checkout, and `cells finish` runs
from main. Rev 1 changes the rule all readers apply instead.

## What is being built

One rule — *a hold belongs to the work stream that owns the cell, not the checkout
that typed the command* — expressed as one helper and applied at four sites:

- `hha-1` — the helper lands; `reserve` uses it for the ledger stamp **and** for its
  own foreign check, so main reserving for a worktree's cell no longer refuses itself.
  Row carries the real feature; the lease record's holder stays identical to the row's.
- `hha-2` — `release` and the cap-time copy in `finish_support` filter through the
  helper, and an explicit `--cell` reaches the ledger without a live reservation to
  derive it from.
- `hha-3` — `has_foreign_hold` behind `cells claim-next` asks the helper, so main can
  still claim a cell whose holds its own worktree owns.

The write-guard is deliberately unchanged: once a row names its real owner the worker
inside that worktree matches it and the deny stops firing, while main being denied a
write to that worktree's file is the correct answer.

## Why this size

The reported symptom is one false deny, but the ownership confusion is read at four
places. Fixing fewer than four leaves the same deadlock somewhere else — the review
demonstrated exactly that for the two-cell shape. No fifth cell: the reporter's
suggestion of a session exemption inside the write-guard protects against a state that
a correctly-attributed row no longer produces.

## Cost if the shape is wrong

Contained. Every site keeps today's value as its fallback, so a cell with no feature or
a feature with no granted worktree behaves byte-identically. The existing ledger tests
(`reserve_writes_node_shaped_lease_and_mirrored_hold`,
`granted_worktree_mirrors_under_its_id_and_blocks_main`,
`ungranted_worktree_skips_the_cross_worktree_section_entirely`,
`release_scoped_to_other_cell_releases_nothing`, `release_all_for_holder_is_holder_scoped`,
and the concurrency lost-update pair) stay green unchanged, so a wrong guess surfaces
as red rather than as silent drift.

## Plan

docs/history/hold-holder-attribution/plan.md (rev 1)
