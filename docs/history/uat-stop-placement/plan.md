# uat-stop-placement — plan

Locked context: `docs/history/uat-stop-placement/CONTEXT.md` (D1-D5).
Charting: `docs/discovery/uat-after-merge/MAP.md`.

Lane: `standard`. Flags: public-contracts, covered-contract-change,
multi-domain.

## Shape

One shared policy module, then the two ends of the road in parallel, then
the paper. The `uat` stop is one idea; today it is spelled in one place
(`bee worktree merge`) and after this it is spelled in two, so the policy —
"which value is set, and does this lane care?" — must live in ONE module
both ends import, never copied.

```
usp-1  src/uat.rs            the policy: UatStop, uat_stop_config, lane rule
   |
   +-- usp-2  merge side     phases.rs + worktree/handlers.rs + worktree tests
   +-- usp-3  close side     drivers/close.rs + close tests
          |
          +-- usp-4  docs + skills + regen
```

usp-2 and usp-3 touch disjoint files and run concurrently once usp-1 lands.

## Slice 1 (the whole feature — it is one slice)

### usp-1 — the policy module

New `packages/bee-rs/crates/bee/src/uat.rs`:

- `enum UatStop { Merge, Close, Off }`.
- `uat_stop_config(main_root) -> Option<UatStop>` — fail-closed, modelled on
  `uat_before_merge_config` (`verbs/worktree/phases.rs:683-689`). Read order,
  stated once: `uat_stop` wins when present; else `uat_before_merge` is read
  as the D1 alias (`true`/absent → `Merge`, `false` → `Off`); else `Merge`.
  Any unrecognized value in either key → `None`, which the callers refuse on
  rather than guessing.
- `uat_gate_applies_to_lane(mode) -> bool` — MOVED here from
  `phases.rs:698-700` unchanged, so both ends share one lane rule (D2).
  `phases.rs` re-exports or calls it; no second copy.

Walking skeleton: this cell is not user-visible on its own, which is the one
justified exception — it is a pure policy read with no behavior of its own,
and usp-2 makes it visible in the same slice.

### usp-2 — the merge side (D4.1, D4.2, D4.3)

`verbs/worktree/phases.rs` + `verbs/worktree/handlers.rs` +
`verbs/worktree/tests.rs`:

1. The merge-time precondition at `phases.rs:294-320` fires only under
   `UatStop::Merge`. Under `Close` and `Off` it does not refuse.
2. `close_the_lane_on_merge` (`phases.rs:457,465-471`) branches. Under
   `Merge`/`Off` it keeps today's behavior exactly (clear the pair, point at
   `bee close`). Under `Close` with `uat` still unapproved AND the lane rule
   saying this lane cares, it instead sets `waiting_on` kind `"gate"`,
   subject `"uat: <feature>"`, and writes a `next_action` naming the road:
   reload the product, test it, then approve `uat` — or fix in the worktree
   and merge again. Under `Close` with `uat` already approved, today's clear
   behavior is correct and stays.
3. Cleanup is forced off while that wait is live (D4.3): `--cleanup` and
   `worktree_cleanup_on_merge: true` are both ignored, with one visible line
   saying why. `resolve_cleanup_on_merge` (`handlers.rs:430-437`) is the seam.

Still true afterwards, and tested: a merge never writes lane `phase`; the
warn-never-fail posture of the lane write is unchanged; `ALREADY_UP_TO_DATE`
writes no lane change; `MERGE_CONFLICT` and proof-debt paths write nothing.

### usp-3 — the close side (D4.4, D2)

`verbs/drivers/close.rs` + its tests: a new blocking `uat` door, built and
ordered beside `judge-debt` (`close.rs:1195-1256`), which is the precedent
for a lane-scoped blocking door with a deferral escape.

- Present only under `UatStop::Close`.
- Blocks when the lane rule says this lane cares and `gates.uat.approved` is
  false. Reads the gate the same way `uat_merge_precheck`
  (`phases.rs:725-762`) does — live workflow record first, default state
  record only as a same-feature fallback.
- Escapable by a `uat-deferral` decision naming the feature.
- Detail line names the remedy: `bee gate --name uat --approved true`.

### usp-4 — the paper

`docs/handbook/register.md` (config table row + the close-door row),
`docs/config-reference.md`, `.bee/config-sample.json`,
`skills/bee-hive/references/gates-and-delegation.md` and
`skills/bee-swarming/SKILL.md` (the `uat` stop now has two possible
positions; say so once in each), then the full regen chain and
`docs/history/codex-harness-hardening/release-manifest.json`.

## Test scoping

Code cells prove with the worktree/close suites they touch, named on the cap
proof line. usp-4 proves with the JSON/pointer/parity checks. CI runs the
full declared command on every push.

## What this plan deliberately does NOT do

Everything in CONTEXT.md D5. No revert mechanism, no new gate name, no new
`waiting_on` kind, no change to the user-only rule, no change to
`staging_before_merge`.
