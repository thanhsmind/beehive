---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-08-22 (auto, gate_bypass normal)
---

# Plan: merge-ready-fact

Mode: `standard` — 2 risk flags: public-contracts, multi-domain
Why this is the least workflow that protects the work: one new stored field that an
external reader depends on, whose truth is owned by five different verbs (cap, reopen,
close, gate, worktree merge) — a plan plus three bounded cells keeps every writer on one
helper instead of five hand-rolled writes.

## Requirements (from CONTEXT.md)

- **D1** — `merge_ready {since, branch, worktree_id, uat, blocked_by}` on the feature record;
  set by the cap that leaves zero open/claimed cells when a worktree grant exists; zero-cell
  features never get it.
- **D2** — close rewrites `blocked_by`; gate uat flips `uat`; worktree merge/unregister and any
  cell reopen delete it.
- **D3** — additive projection; never read by bee's gates or merge door; surfaced in
  `bee status --json` lane rows.

## Discovery

Gather digest 2026-08-22 (anchors in the cell actions): the one cap door is
`cap_cell_from_flags` (`verbs/cells/handlers_close.rs:153`, post-cap slot after the claim
release at `:561`); reopen paths are `run_reopen` (`:956`), `unclaim_cell` (`:917`) and
`judge-record` NEEDS_REVISION (`handlers_meta.rs:261-283`). The record mutation seam is
`state_group/ledger.rs` (`resolve_mutation_target` `:313`, `write_through_projection`
`:370`); unknown lane keys survive read/write (`lanes.rs:61`). Feature→worktree mapping:
`status_full/topology.rs:198 find_granted_worktree_for_feature`. Merge-time lane touch:
`worktree/phases.rs:495 close_the_lane_on_merge`; unregister has no lane touch
(`registry.rs:312`). Gate write: `set_gate.rs:885-912`. Close doors vector assembled at
`drivers/close.rs:1830-1841` (dry-run) and `:1924-1930` (real); refusal arms return early.
`bee status --json` lane rows: `topology.rs:313 build_lane_rows`.

## Approach

One helper module owns every write (`set_after_cap`, `clear`, `set_uat`, `set_blocked_by`)
through the existing mutation seam; the five verbs call it. Rejected: a sweeper that
recomputes on `status` (stale between runs); computing on read (waggledance is a file reader).

| Component | Risk | Proof |
|---|---|---|
| helper + cap/reopen wiring | MEDIUM — island scope of `list_cells` in a worktree; lock order | unit tests over cap→set, reopen→clear, zero-cell no-op |
| close/gate/merge/unregister wiring | MEDIUM — close's early-return arms | tests per verb |
| status surfacing + docs | LOW | test on lane row; knowledge check |

## Shape

| Cell | What | Proof |
|---|---|---|
| mrf-1 | helper module + cap/reopen/unclaim/judge-record wiring | `cargo test merge_ready` |
| mrf-2 | close `blocked_by`, gate `uat`, merge/unregister clear, status lane rows | `cargo test merge_ready` + close/worktree suites |
| mrf-3 | knowledge concept in workflow-state | `bee knowledge check` |

Deps: mrf-2 → mrf-1. mrf-3 parallel.

## Test matrix

- Happy: last cap with a grant → `merge_ready` with `since`, `branch wt/<f>`, `worktree_id`,
  `uat: pending`, `blocked_by: []`; `gate uat true` → `approved`; `worktree merge` → gone.
- Edge: cap with open siblings → absent; cap with no grant → absent; zero cells → absent;
  reopen after set → gone, next last-cap → fresh `since`; close with a blocking non-uat door →
  `blocked_by` names it; green close → `[]`; `gate uat false` → `pending`.
- Error: corrupt lane → helper returns without writing, verb result unchanged.

## Out of scope

waggledance reader; TTL/sweeper; reading merge_ready inside bee's own gates.
