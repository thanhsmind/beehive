# uat-stop-placement — locked context

Source: `docs/discovery/uat-after-merge/MAP.md` (charting complete, no
tickets needed). Decisions below are locked; cite them, never reinterpret.

## The problem

The owner runs a small project whose product serves out of the MAIN
checkout. Nothing is testable until the code is on main. bee today enforces
the `uat` gate at `bee worktree merge`, so it asks the owner to accept work
they physically cannot run yet.

The fix is not to remove the `uat` stop. It is to make **merging a
publish-for-testing step rather than the finish line**: the code lands on
main so the product reloads, the owner keeps standing in the feature
worktree, and the feature is not done until `uat` is approved. A failed uat
is fixed in the worktree and merged again.

Said plainly: main plays the role staging plays for bigger projects.

## Locked decisions

**D1 — the switch is a new key `uat_stop`.**
Values `"merge"` (default, today's behavior), `"close"` (merge first, accept
after), `"off"` (no uat stop anywhere). The existing `uat_before_merge` key
stays readable as a back-compat alias: `true` reads as `"merge"`, `false`
reads as `"off"`. A value outside the three refuses rather than guessing.
Chosen over adding an `"after"` value to `uat_before_merge`, whose name
would then contradict its own value.

**D2 — the close-time `uat` door keeps today's lane rule.**
It applies to `standard` and `high-risk` only; `tiny`/`small`/`docs`/`spike`
are exempt — exactly the set `uat_gate_applies_to_lane` already decides at
`verbs/worktree/phases.rs:698-700`. The door is escapable by a
`uat-deferral` decision naming the feature, copying the `judge-debt`
precedent at `verbs/drivers/close.rs:1195-1256`.

**D3 — a failed uat after merge is fixed forward.**
A new cell on a new worktree, merged again. bee grows no revert or rollback
mechanism for a merge already on main. Main may be broken for a while and
that is accepted for a project this size.

**D4 — under `uat_stop: "close"`, exactly four things change.**

1. `bee worktree merge` stops refusing `WORKTREE_MERGE_UAT_PENDING`.
2. The post-merge lane write INVERTS. Today `close_the_lane_on_merge`
   (`verbs/worktree/phases.rs:457,465-471`) clears the feature's
   `waiting_on`+`run_state` pair via `clear_lane_waiting_on_pair`
   (`verbs/state_group/waiting_on.rs:199-213`) and rewrites `next_action` to
   "capture what settled, then bee close". Under `"close"`, a merge whose
   `uat` is still unapproved instead SETS `waiting_on` kind `"gate"`,
   subject `"uat: <feature>"`, and writes a `next_action` naming the road:
   reload the product, test it, then either approve `uat` or fix in the
   worktree and merge again.
3. Cleanup is forced OFF while `uat` is pending — `--cleanup` and
   `worktree_cleanup_on_merge: true` are both ignored. The worktree is the
   only place the fix can be written, and a torn-down worktree drops the
   grant, so the second merge would hit the no-granted-worktree refusal at
   `verbs/worktree/handlers.rs:103-108`.
4. `bee close` carries the blocking `uat` door from D2.

**D5 — nothing else moves.**
bee already treats a merge as non-terminal in every other respect, and each
of these stays exactly as it is:

- a merge never writes lane `phase` (`phases.rs:615-616` — "a merge can land
  one slice of many, so phase stays close's word alone");
- the worktree is KEPT by default (`handlers.rs:415-421`, absent means
  false);
- repeat merges of the same feature run normally; a second merge with no new
  commits is `ALREADY_UP_TO_DATE`, a no-op;
- nothing refuses further commits, claims, or cells for a merged feature;
- `bee state gate --name uat` keeps refusing `--actor auto`
  (`state_group/set_gate.rs:657-662`) — `uat` stays user-only at either end
  of the road;
- `GATE_NAMES` (`state.rs:35`) is untouched; `uat` is already in it;
- `waiting_on` kinds stay the closed pair `["gate", "question"]`
  (`verbs/workflow_store/record.rs:356`) — `gate` is the right kind;
- `staging_before_merge` is independent and unaffected.

## Out of scope

- A revert/rollback mechanism for a merge already on main.
- Per-feature served builds or per-feature ports.
- Any change to the `uat` gate's user-only rule, or to `GATE_NAMES`.
