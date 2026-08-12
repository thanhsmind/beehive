# worktree-first-teeth — plan

Lane: `standard` · flags: multi-domain, public-contracts, covered-contract-change · 5 product files.

## Why

A live incident (beedashboard, 2026-08-12 05:13): session B, working in the
granted worktree `beedashboard--wt--scroll-fab`, was refused a reservation on
`crates/mdview/src/views.rs` because session A already held it. Session A was
doing full feature work (cell `hub-finished-compact-1`, lane `small`) **in the
MAIN checkout** and committed straight to `main` (`c79e071`) — the exact thing
`AGENTS.md` forbids and the exact thing the write guard already claims to
prevent.

Two separate defects made that possible, and a third made the refusal unreadable.

## What is actually broken

### D1 — the guard reads the wrong record for a lane-bound session

`main.rs:326-331` passes the raw `read_state(store_root)` (`main.rs:136`) into
`check_worktree_first`. Every *other* write check goes through
`resolve_write_record` (`checks.rs:45-146`), which resolves a session bound to a
lane to `.bee/lanes/<feature>.json` instead. So a lane-bound session is judged
against whatever feature `state.json` happens to name.

That is what happened: beedashboard's `state.json` named `targeted-reload`,
while the session was working the `hub-finished-compact` lane. The guard asked
"does `targeted-reload` hold a worktree?", got no, and allowed every write.

**Live-store distribution** (the shape a fixture must be built from — decision
`0cd7bc46`): 78 lane records across the two live repos (beehive 56,
beedashboard 22) against 2 default-pipeline `state.json` records. The lane shape
is the dominant shape, and it is the one the guard cannot see. Of the 78, 36
carry a `route`; 14 sit at phase `swarming`. `hub-finished-compact.json` itself
carries `phase: "swarming"`, `route.lane: "small"`, `feature:
"hub-finished-compact"` — everything the guard needed was on disk, in the record
it never read.

### D2 — the guard only fires when the feature ALREADY has a worktree

`check_worktree_first` (`hook_local.rs:546-549`) returns `Ok(None)` — allow —
when `find_feature_worktree_grant` finds no grant for the active feature. So it
refuses a main-checkout write only for a feature that already did the right
thing. A feature that never created its worktree is invisible to it. This is the
"a guard that tests one state is a law with a hole" pattern
(`docs/knowledge/patterns/20260713-a-guard-that-tests-one-state-is-a.md`).

`route_worktree_block` (`workflows.rs:663-672`) already prints the exact remedy
for this case — `bee worktree new --feature <f>` — and already promises "once
the worktree is granted, main refuses feature source edits". The promise is only
half kept.

### D3 — a same-store reservation conflict cannot name the holding checkout

Leases are control-plane: `control_root_for` (`leases.rs:98-162`) resolves every
checkout's lease directory to the MAIN store, so main and every worktree share
one lease dir. `conflict_out` (`reserve.rs:439-455`) prints
`- <agent> holds "<path>" (cell <cell>)` — agent and cell, never which checkout.
The lease record (`reserve.rs:256-271`) carries no root, so the refusal *cannot*
say it. The cross-worktree `FOREIGN_HOLD` refusal (`reserve.rs:160-194`) already
names the holder; the plain overlap refusal is the one that goes blind.

## What is NOT changing (and why)

Sharing the lease store across worktrees stays exactly as designed. Two
worktrees editing one file collide at `bee worktree merge`; refusing at reserve
time is cheaper than curing a git conflict later. Scoping `find_conflicts` by
holder — so same-path/different-worktree reservations stop conflicting — is
explicitly rejected: it only defers the collision to merge time. Decision
logged 2026-08-12, tags `reservations,worktrees,guards`.

## Shape — 2 cells, disjoint files, parallel

### `wtf-1` — close D1 and D2 in one guard change

`hooks/write_guard/{hook_local.rs,main.rs}` + `hooks/write_guard/tests.rs`.

1. Feed `check_worktree_first` the same record every other check uses — the
   resolved lane/default record from `resolve_write_record`, not raw
   `state.json`.
2. Add the missing arm: no grant for this feature + a code-touching lane +
   the MAIN checkout ⇒ deny, naming `bee worktree new --feature <f>`, the same
   remedy `route_worktree_block` prints.

Scope of the new arm, deliberately narrow:

- fires only at `phase == "swarming"` — the phase where cells execute. Every
  earlier phase already denies source writes (`checks.rs:477-509`); every later
  phase is the integration, scribing and release work main legitimately owns.
- lane `docs` and lane `tiny` never fire — `AGENTS.md` gives main "integration,
  docs-lane, release work, and a solo `tiny` fix".
- a missing `route` on the acting record stays fail-open (allow). 40 of 56
  beehive lane records carry no route; a guard cannot claim work is
  code-touching when nothing says so. `counter-teeth` already gave route-less
  claims their own teeth at `cells claim`.
- every existing exemption holds byte-for-byte: `.md` and gate-allowed prefixes
  (`worktree_first_exempt_rel`), `worktree_first: "off"` in config, non-ordinary
  checkouts, corrupt grants registry fails open.

Both failure directions are proven (decision `0cd7bc46`): one deny case built
from the live `hub-finished-compact.json` shape, and a no-deny case for each
carve-out above. An independent read runs before the cap.

### `wtf-2` — make a conflict refusal name the checkout

`verbs/reservations/{reserve.rs,leases.rs}` + `verbs/reservations/tests.rs`.

Write `holder` onto the path lease record — the value is already in scope at
`reserve.rs:51` (`roots.hold_topology()`), the same value the hold ledger row
already carries at `reserve.rs:331`. Surface it on `Resv`/`resv_to_value` and in
the `conflict_out` line, so the refusal reads `main holds "…"` instead of an
anonymous agent name. Reporting only — no conflict logic changes.

Back-compat is structural: all four lease readers use dynamic `Option` access,
no serde struct, so an old lease file without `holder` is a no-op. An ungranted
linked worktree has no topology at all (`hold_topology()` returns `None`); the
field is simply absent there.

## Verify

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml` — the declared
project suite, green at 1627 passed before this work started.
