# docs-lane-liveness — CONTEXT

## What was asked

The user asked for the docs lane to take the same condition the tiny lane
took one feature ago (`dirty-main-conflicts` D4): main-checkout privilege
only while no other session is live. With a live peer, docs work routes
into a worktree like any feature; solo, everything stays exactly as fast
as today.

## What was found

After the tiny fix (`6c50b195`), the docs lane is the ONE remaining
unconditional writer into main. Two mechanisms grant it, and both must
carry the condition or the fix is toothless:

1. The lane arm: `check_worktree_first` returns early for `lane == "docs"`
   (`hook_local.rs:611-613`), on both the granted and no-grant arms.
2. The path arm: `worktree_first_exempt_rel` exempts every path ending
   `.md` (`hook_local.rs:545-547`). Docs work writes mostly `.md`, so
   gating the lane while leaving this blanket exemption would change
   nothing in practice.

`worktree_first_exempt_rel` has exactly one production caller — the
offender scan inside `check_worktree_first` (`hook_local.rs:618`) — so the
`.md` clause can be gated at that single consumer without touching any
other behavior.

The liveness predicate and the fail-open discipline are already settled by
the tiny fix in the same function: `is_concurrent_mode` (write_guard
`store.rs:383`), self-excluding via the threaded `session_id`,
`.unwrap_or(false)` on any read error.

## Decisions

- D1 — Docs-lane main privilege becomes conditional exactly like tiny's:
  allowed while no other live session exists; denied into a worktree when
  a live peer is present. Both grant mechanisms (the lane early-return and
  the `.md` suffix exemption) carry the same single condition, computed
  once.
- D2 — This narrows, not reverses, the earlier owner decision that kept
  the blanket `.md` exemption (decision 1033a66c): solo behavior is
  byte-identical to today, which is the case that decision protected. The
  new decision log entry supersedes it explicitly.
- D3 — Same discipline as the tiny fix: `is_concurrent_mode` reused (never
  reimplemented), the acting session never counts itself, fail-open on any
  session-store read error, and the prefix exemptions (`.bee/`, `docs/`,
  `plans/`, `AGENTS.md`) stay unconditional — bee's own bookkeeping and
  the merge auto-commit already cover those roots.
- D4 — The doctrine sentence follows the code in the same change:
  `AGENTS.md` (and its source `packages/bee/AGENTS.block.md`) says the
  main checkout takes docs-lane work "when no other session is live",
  the same clause tiny already carries; the knowledge area
  (`docs/knowledge/areas/worktree-parallelism/routing-and-visibility.md`)
  gets the same one-line update. A rule whose letter and machine disagree
  is the defect this whole day kept finding.

## Known limits, named not hidden

- The no-grant deny arm fires only at phase `swarming` — identical to the
  tiny carve-out's reach. Docs work outside swarming with a live peer
  still passes; widening the arm's phase reach is a separate decision.
- `docs/` prefix paths (e.g. `docs/specs/`) stay exempt via the prefix
  list even with a live peer. `docs/decisions`, `docs/knowledge`, and
  `docs/history/<f>` are auto-committed at merge, so the residual
  blockable surface is small (`docs/specs`); accepted, named here.

## Out of scope

- Widening the no-grant arm beyond phase swarming.
- The prefix exemption list (`GATE_ALLOWED_PREFIXES_INTAKE`).
- Any change to merge, lanes, or the liveness predicate itself.
