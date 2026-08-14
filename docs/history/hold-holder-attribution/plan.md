# hold-holder-attribution — plan (rev 1)

Route: class `feature` · lane `standard` · flags `public-contracts,covered-contract-change,multi-domain` · product files 6
Worktree: `beehive--wt--hold-holder-attribution` at `/home/thanhsmind/projects/goglbe/beehive--wt--hold-holder-attribution`

Rev 1 replaces rev 0 after a plan review. Rev 0 proposed re-stamping the ledger row's
owner and nothing else; the review proved that only moves the deadlock (see
"What rev 0 got wrong"). Rev 1 changes the rule every reader applies, not just the
value one writer writes.

## The defect

A worker standing inside a granted worktree, holding a clean reservation, was denied
every write to its own file by

> bee cross-worktree hold: "…" is held by checkout "main" (feature unknown, cell hoola-login-schema)

with no session on `main` actually touching that file, and with the ghost rows growing
on every retry.

Proven in source:

1. `src/roots.rs:558` — `hold_topology()` derives `holder` **only** from the acting
   checkout's own grant linkage: `"main"` from an ordinary checkout, the worktree id
   from a granted one. It takes no cell and no feature.
2. `src/verbs/reservations/reserve.rs:339` — the mirrored ledger row is stamped with
   exactly that. Control-plane commands must run from main — `src/verbs/drivers/prepare.rs:1162`
   says so in its own words ("already narrowed this to an ORDINARY checkout, so this
   is always `(workRoot, \"main\")`") — so a reserve made from main on behalf of a
   granted worktree's cell stamps `holder: "main"`. This is the normal path.
3. `src/verbs/reservations/reserve.rs:340` — the row's `feature` is hardcoded
   `Value::Null`, which is why the deny message reads `feature unknown`
   (`src/hooks/write_guard/checks.rs:400`).
4. `src/verbs/reservations/release.rs:171` — release clears a row only when
   `hold.holder == t.holder`, `t` again being the **acting** checkout. The worktree's
   holder is its own id, so it can never clear a `"main"`-stamped row.
5. `src/verbs/reservations/release.rs:82-111` — the `{cell, session}` pairs release
   scopes by are derived from **live** reservations. Once the cell is capped the leases
   are gone, so the rows become unreachable by any release at all.
6. `src/hooks/state_sync.rs:818-831` — an unreleased, unexpired row whose `session`
   matches the acting session has its `mirrored_at` **renewed**. While the main
   control-plane session heartbeats, its own ghost rows never expire, so neither the
   TTL nor `sweep` (`release.rs:282-293`, expired rows only) drains them.
7. `src/hooks/write_guard/checks.rs:398-417` — the guard reads those rows through
   `find_foreign_holds` (foreign = `holder != acting`, `write_guard/store.rs:767`) and
   hard-denies on an exclusive path.

Net: bee's architecture puts the control plane in main, which manufactures ledger rows
naming the wrong owner; bee's guard then enforces them against their real owner; and
bee's own heartbeat keeps them alive forever. The ledger is not lying about a
conflict — it is lying about **who**.

## The rule this feature installs

> A hold belongs to the work stream that owns the **cell**, never to the checkout that
> happened to type the command.

Concretely, one helper answers one question — *who owns this cell's holds?*

```
effective_holder(main_root, acting_holder, cell) =
    owning worktree id of cell's feature   (cell -> feature -> granted worktree)
    else acting_holder                     (today's value, unchanged)
```

The composition already exists: `crate::verbs::cells::read_cell` returns the cell
record carrying `"feature"` (`verbs/cells/read.rs:180`, re-exported at
`verbs/cells/mod.rs:386` — note `cells::read::` is private, the usable path is
`crate::verbs::cells::read_cell`), and
`find_granted_worktree_for_feature(main_root, feature)`
(`verbs/status_full/topology.rs:198`, `pub(crate)`, re-exported at
`status_full/mod.rs:337`) is already composed this exact way in
`verbs/cells/finish_support.rs:326`. It reads only `worktree-grants.json` and does
plain filesystem probes — no store lock, no git subprocess, no touch of the holds
ledger — so it is safe to call inside the `CROSS_WORKTREE_HOLDS_LOCK` section
(verified in review).

Every site that today asks *"is `holder == my checkout?"*" asks the helper instead.
That is the whole feature.

## What rev 0 got wrong

Rev 0 stamped the owning worktree on the write side and stopped. The review proved
three consequences, each of which is now inside the plan:

- **It moves the deadlock instead of closing it.** `release.rs:171` and
  `finish_support.rs:622` both filter by the acting holder, and `cells finish`
  typically runs from main (`finish_support.rs:287-290` states this). Worktree-stamped
  rows would become rows **main** cannot clear — with `state_sync.rs:829` keeping them
  alive. Same bug, mirrored.
- **It creates a new false deny on main.** `reserve.rs:161`, `checks.rs:408`, and
  `handlers_select.rs:460` (`has_foreign_hold`, used by `cells claim-next`) would all
  read a worktree-stamped row as foreign to main — so main's own control-plane reserve
  for that same cell would refuse itself with `FOREIGN_HOLD`, and `claim-next` from
  main would skip the cell.
- **It leaves the lease record lying.** `reserve.rs:272-279` stamps the lease's own
  `holder` and its comment claims it is "the same holder string the mirrored ledger row
  carries below". Changing one and not the other makes the comment false and keeps
  `conflict_out` (`reserve.rs:455-462`) reporting the old value.

## Slice 1

Three cells, serial — `hha-1` introduces the helper the other two consume, and all
three add cases to `src/verbs/reservations/tests.rs`.

### hha-1 — the write side tells the truth

The helper lands (beside the ledger code in `verbs/reservations/leases.rs`), and
`reserve` uses it for **both** the stamp and its own foreign check
(`reserve.rs:161`), so main reserving for a worktree's cell writes a row owned by that
worktree and does not then refuse itself. The row carries the resolved feature instead
of `null`. The lease record's holder (`reserve.rs:272-279`) is kept identical to the
ledger row's, and its comment stays true.

Fallback, unchanged from today: a cell with no feature, or a feature with no granted
worktree, keeps the acting topology holder. The feature lookup runs against the acting
store root — from main that is where control-plane-claimed cells live; from inside a
granted worktree the cell may be invisible, and the fallback then yields that
worktree's own id, which is the right answer anyway.

Proof: red-first cases — a reserve from main for a cell whose feature owns a granted
worktree mirrors `holder: "<wt id>"` with a non-null feature; a second reserve from
main on the same path/cell is a plain lease conflict, **not** `FOREIGN_HOLD`; the
lease record's holder equals the ledger row's; a cell with no granted worktree is
byte-identical to today. Plus one identity assertion the review demanded: the grants
map key, `LinkedRoots::id` (`roots.rs:561`), and the guard's `ctx.workspace_id`
(`checks.rs:211-216`) are the same string — asserted, not assumed.

### hha-2 — the release side clears what it owns

`release.rs:171` and the cap-time copy in `finish_support.rs:622` filter through the
helper, so whichever checkout runs the release clears the rows belonging to that
cell's owner. And an explicit `--cell` scopes the ledger pass directly instead of only
through pairs derived from live reservations (`release.rs:95-111`), so a capped or
unclaimed cell's rows stay reachable — closing the orphan window for every crash
between reserve and cap, and giving today's ghosts a drain.

Note the blast radius the review named: `release_exec` is also reached by
`release_reservations_for_agent` from `prepare.rs:999` and `prepare.rs:1386` (the
claim-unwind). Widening `--cell` scoping changes those unwinds too — intended, and
called out here so it is not a surprise.

Proof: red-first cases — a ledger row for cell `c` with **no** matching lease is
cleared by `release --agent a --cell c` (today: 0); a worktree-stamped row is cleared
by a release run from main for that cell; `release_scoped_to_other_cell_releases_nothing`
(`tests.rs:250`) and release without `--cell` stay byte-identical.

### hha-3 — the read side stops seeing its own work as foreign

`handlers_select.rs:460` (`has_foreign_hold`, behind `cells claim-next`) asks the
helper for the candidate cell, so main can still claim a cell whose holds its own
worktree owns. The write-guard (`checks.rs:398`) is deliberately **not** changed: once
a row names its real owner, a worker inside that worktree matches it and the deny stops
firing, while main being denied a write to a file the worktree owns is the correct
answer (main is already barred from that worktree's source anyway).

Proof: red-first case — `cells claim-next` from main returns a cell whose paths carry
worktree-owned holds; the existing foreign-hold skip still fires for a hold owned by a
*different* feature's worktree.

## Existing proof this touches

Inventoried before drafting; read before editing (all under `packages/bee-rs/crates/bee`):

- `src/verbs/reservations/tests.rs:108` `reserve_writes_node_shaped_lease_and_mirrored_hold` — plain reserve mirrors `holder: "main"`.
- `src/verbs/reservations/tests.rs:494` `granted_worktree_mirrors_under_its_id_and_blocks_main`.
- `src/verbs/reservations/tests.rs:547` `ungranted_worktree_skips_the_cross_worktree_section_entirely`.
- `src/verbs/reservations/tests.rs:228` `release_deletes_lease_and_marks_mirrored_hold`; `:250` `release_scoped_to_other_cell_releases_nothing`; `:268` `sweep_releases_expired_lease_and_hold`.
- `src/verbs/worktree/tests.rs:1227` `release_all_for_holder_is_holder_scoped`.
- `src/hooks/write_guard/tests.rs:748` `linked_worktree_reads_foreign_reservation_from_main_store`.
- `tests/concurrency.rs:1077` `every_mirrored_hold_survives_a_concurrent_reserve_into_the_shared_ledger` — the ledger append stays under its store lock; no cell may weaken that.

## Draining the ghosts already on disk

hha-2 supplies the mechanism, not the act. Rows written before this feature carry
`holder: "main"`, so they are cleared by running, **from the main checkout**, one
`bee reservations release --agent <agent> --cell <cell>` per affected cell. That step
belongs to whoever owns the affected repo and is recorded here so it is not mistaken
for something the code does by itself.

Verified by `commands.test`:
`cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
