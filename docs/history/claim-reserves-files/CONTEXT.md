# claim-reserves-files — locked context

## The defect

`packages/bee/prompts/worker-cell.md:37` tells every dispatched worker:

> The cell's listed files are reserved under your nickname when dispatch claimed
> them; reserve any ADDITIONAL path before writing

Both halves are false. Three independent lines of evidence:

1. **Source.** `packages/bee-rs/crates/bee/src/verbs/cells/claims.rs` contains no
   reserve call at all — `grep -n "reserve"` over that file returns nothing.
2. **Empirical.** In the `skill-report-stamps` session, `bee reservations list`
   printed `No reservations.` immediately after three cells were claimed with
   `--worker`. Probe reservations created moments later *did* appear, so the
   store was working.
3. **Independent worker reports.** Both `wgg-1` and `wgg-2` recorded a deviation
   that their cell's files were not pre-reserved, and reserved them by hand
   before writing.

The consequence is quiet and bad: a worker that **trusts** the prompt writes
unreserved, so the reservation protection the swarming contract depends on is
absent for every dispatched cell. Only a worker that happens to *distrust* its
own briefing is protected.

## Why the fix belongs at the claim side

The lifecycle already assumes these reservations exist. `finish_cap_and_release`
(`packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:745`) releases at
cap with:

```
release_reservations_for_agent(topo, root, agent, &cell_id)
```

where `agent` is `trace.worker` — the value `cells claim --worker` recorded. So
the release half of the pair was built, shipped, and is running today against
reservations that the claim half never creates. This is a missing half of an
existing designed symmetry, not a new policy.

Fixing the prompt instead would be the lesser fix: it would leave a documented
release-without-acquire and keep the acquisition racing the worker's first
write.

## Locked decisions

1. **`bee cells claim` reserves the cell's declared `files` under the claiming
   `--worker` identity, for the claimed cell id.** Same `(agent, cell)` key the
   cap-time release already uses, so acquire and release are symmetric by
   construction.
2. **A reservation conflict refuses the claim, and the refusal is typed and
   zero-mutation.** The claim file must not be left owned when the paths behind
   it could not be held. Whatever partial reservations were taken are rolled
   back before returning, so a refused claim leaves the store exactly as it
   found it. This mirrors `worktree new`'s post-creation rollback discipline.
3. **A cell with no `files` claims exactly as it does today.** Zero paths to
   reserve is not an error and adds no new refusal.
4. **`claim-next` inherits the same behavior** — it is the cross-session door
   and already skips cells whose declared files overlap a foreign hold, so it
   must not now claim one and leave it unreserved.
5. **The prompt is corrected to match the code, either way.** After this cell
   the sentence in `packages/bee/prompts/worker-cell.md` must describe what
   actually happens. If any part of the guarantee cannot be delivered, the
   prompt says so plainly rather than overstating it.

## Hard constraints

- **Never weaken the existing release path.** `finish_cap_and_release` and
  `release_reservations_for_agent` are not edited.
- **A claim that succeeds today, with no overlapping holds, still succeeds** —
  byte-identical output. The new refusal fires only on a genuine conflict.
- **No double-reserve.** Re-claiming a cell the same agent already holds must
  not error on its own prior reservations.
- **The TTL story stays coherent:** the reservation's life is tied to the
  claim's, so an expired or released claim does not strand a hold.

## Acceptance

- Claiming a cell with declared files creates reservations for exactly those
  paths under the `--worker` identity; `bee reservations list` shows them.
- Capping that cell releases exactly those reservations — the existing
  `released` array in the cap output names them, with no new code in the
  release path.
- A claim whose declared files conflict with another agent's active hold is
  refused, names the holder, and leaves both the claim store and the
  reservation store untouched.
- A cell with `files: []` claims exactly as before.
- The worker-cell prompt sentence matches the shipped behavior.

## Out of scope

- The `--only` versus `--` pathspec mismatch in the concurrent-worker git guard
  (filed P3 separately).
- The capture-queue noise blocker (filed P2 separately).
- Any change to how `dispatch prepare` builds its payload beyond the prompt
  sentence itself.
