# uat-after-merge

## Destination

One config switch in `.bee/config.json` that moves the `uat` stop from
merge-time to close-time, so a small project can merge to main, reload the
app that runs from main, test it there, and only then accept the feature.
Arrived = the switch is shipped, documented, and turned on works.

## Notes

The owner runs a small project. The app they test serves out of the MAIN
checkout, so nothing is testable until the code is on main. Today's order is
test-at-staging → approve `uat` → merge. They need merge → reload → test →
approve, without losing the `uat` stop itself.

Facts gathered this session (all `packages/bee-rs/crates/bee/src`):

- `uat` is enforced in exactly ONE place: `bee worktree merge`
  (`verbs/worktree/phases.rs:294-320`, config read at `phases.rs:683-689`,
  lane rule at `phases.rs:698-700`, precheck at `phases.rs:725-762`).
  Everywhere else it is display-only.
- `bee close` never reads `uat` at all (`verbs/drivers/close.rs`, zero hits).
- `GATE_NAMES` at `state.rs:35` already contains `uat` and is a closed
  5-element constant — no new gate is needed, only a new place that reads it.
- `bee state gate --name uat` refuses `--actor auto` unconditionally
  (`state_group/set_gate.rs:657-662`), so `uat` stays user-only wherever it
  is enforced.
- There is NO post-merge or post-close human stop in bee today, and NO
  revert/rollback verb, cell type, or documented pattern after a merge lands.
- `judge-debt` (`close.rs:1195-1256`) is the structural precedent for a
  blocking close door that is lane-scoped and escapable by a deferral
  decision.

## Decisions so far

- **D1 — the switch is a new key `uat_stop`.** Values `"merge"` (default,
  today's behavior), `"close"` (merge first, accept after), `"off"`. The
  existing `uat_before_merge` stays readable as a back-compat alias
  (`true` → `"merge"`, `false` → `"off"`) so no repo breaks. Chosen over
  adding an `"after"` value to `uat_before_merge`, whose name would then
  contradict its own value.
- **D2 — the app under test runs from the main checkout.** That is the whole
  reason merge must come first; a per-feature served build is not the
  topology here.
- **D3 — `uat_stop: "close"` moves the stop, it does not remove it.**
  `bee worktree merge` stops refusing `WORKTREE_MERGE_UAT_PENDING`, and
  `bee close` gains a blocking `uat` door in its place.
- **D4 — the close-time `uat` door keeps today's lane rule.** It applies to
  `standard` and `high-risk` only; `tiny`/`small`/`docs`/`spike` are exempt,
  exactly as `uat_gate_applies_to_lane` already decides at merge time. One
  rule, one place, whichever end of the road the stop sits at.
- **D5 — a failed uat after merge is fixed forward.** A new cell on a new
  worktree, merged again. bee grows no revert mechanism; main may be broken
  for a while and that is accepted for a project this size.

## Not yet specified

Nothing blocking. Two shaping-level details fall out of D1-D5 and are
settled when the feature is planned, not before:

- The exact escape hatch for the close-time door (judge-debt's
  `judge-deferral` decision is the precedent to copy).
- The wording of the merge's `next_action` under `uat_stop: "close"` — it
  has to hand the user the reload-test-approve-close road in one line.

## Out of scope

- A revert/rollback mechanism for a merge already on main (D5).
- Per-feature served builds or per-feature ports (D2).
- Any change to the `uat` gate's user-only rule, or to `GATE_NAMES`.
- `staging_before_merge` — already shipped, independent, and unaffected.

## Closing

Map complete; no tickets were needed — the destination was nameable in one
session and every open question resolved in two interview rounds. Hands off
to the normal chain as a single feature: `uat-stop-placement`.
