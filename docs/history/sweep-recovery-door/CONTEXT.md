# Sweep Recovery Door — Context

**Feature slug:** sweep-recovery-door
**Date:** 2026-08-13
**Shaping session:** complete (carried decisions + two new answers)
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

The second half of the sweep work: a crashed session's record says so, and
`bee recovery scan` becomes a real command that releases what that session was
holding. This feature ends at the session record and the new command — the
sweep itself, its caller exclusion, its parked verdict and its store boundary
all shipped in `sweep-at-every-door` and are not reopened here.

## Lineage

This is slice 2 of the plan approved for `sweep-at-every-door`
(`docs/history/sweep-at-every-door/plan.md`, epics E3 and E4). That feature
closed green with slice 1 delivered — cells `sad-1` (commit `74b8cc6c`) and
`sad-2` (commit `9081bf90`), merged to main and re-verified there. Its plan is
frozen, so slice 2 runs as its own feature rather than reopening a closed one.
D2 and D3 below are carried verbatim from that feature's CONTEXT.md; D7 and D8
are new, answering gray areas D2 left open.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D2 | A heartbeat-stale session record is marked dead in place — `.bee/sessions/<id>.json` gains `status: "dead"` and `dead_at` — never deleted. No hard-stale deletion tier. | Carried from `sweep-at-every-door` (decision `5f8779d2`). The transcript pointer and lane on a dead session record are what recovery mines for unsettled work; deletion destroys that evidence. |
| D3 | `bee recovery scan` is built as a releasing door: it releases every qualifying crashed-session claim on invocation. No `--release` flag, no confirmation step. `bee recovery window` stays unbuilt and keeps its registry marker. | Carried from `sweep-at-every-door` (decision `501fa7c5`). The qualifying criteria are already conservative, so a second gesture would reintroduce the "nobody happened to run it" failure the work exists to remove. |
| D7 | Only `bee recovery scan` writes the dead mark. The sweep called from `bee orient` and `bee cells claim-next` keeps releasing claims and never touches a session record. | One writer, one door, one testable path. Accepted cost: the mark appears only when someone runs `recovery scan`. Acceptable because the mark aids a human reading the store — it is never a precondition of a release, and the release criteria stay heartbeat-age based and unchanged. Decision `dd4b5cba`. |
| D8 | The dead mark is reversible. When the heartbeat hook touches a session record carrying `status: "dead"`, it clears `status` and `dead_at` and records `revived_at`. | Deadness is inferred from heartbeat age (900s), which a live but idle or long-running session crosses routinely; an irreversible mark would leave live sessions permanently mislabelled at every call site that reads the record. `revived_at` preserves the history that it was once judged dead. Decision `fcc83358`. |

### Inherited constraints (from `sweep-at-every-door`, not restated as new decisions)

`bee recovery scan` reuses the shipped sweep, so it inherits its contract
whole: it never takes a claim owned by its own caller (D6/R97), it does not
sweep at all when it cannot resolve its caller (D6/R98), a reclaimed unit is
parked `blocked` rather than reopened (D4/R99), and it rewrites only units
readable in its own store, naming the ones it cannot reach (D5/R100).

### Agent's Discretion

- The exact wording and shape of `recovery scan`'s output, provided it names
  each released claim, each parked unit, and each unit it could not reach.
- Where the dead-mark write lives, provided it holds the `sessions` lock as
  the existing session writers do.
- Whether `revived_at` (D8) accumulates a history or holds only the most
  recent revival.
- Correcting the two registry descriptions this feature falsifies (see
  Canonical References), and repointing `sad-2`'s D6 decline text.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| dead mark | `status: "dead"` plus `dead_at` on a session record. An annotation, never a deletion, and never a precondition of a release. |
| revival | The heartbeat hook clearing a dead mark on a record whose session came back, stamping `revived_at`. |
| releasing door | A command that performs the sweep as its purpose, rather than as a step before its real work. `recovery scan` is the only one. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/status_full/recovery.rs:264-466` —
  `build_recovery_block` / `detect_crash_candidates` / `has_clean_end_trio`.
  The detection half already exists and is reused, not rewritten. It also
  already excludes the current session by env id (`:270-283`).
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_select.rs` — the shipped
  sweep, now caller-aware and store-safe. `recovery scan` calls it; it does not
  reimplement any of it.
- `packages/bee-rs/crates/bee/src/verbs/timings.rs` and `tmp_group.rs` — the
  single-file verb group pattern a new `recovery` group follows.
- `packages/bee-rs/crates/bee/src/hooks/state_sync.rs` — the heartbeat hook
  where D8's revival clearing lands.

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/mod.rs` — one `pub mod` line plus one
  `try_native` chain entry for the new group.
- `packages/bee-rs/crates/bee/src/generated/registry_payload.json` — the
  `recovery scan` marker flip. Write-guarded
  (`hooks/write_guard/guards.rs:216`) with no regen chain covering it
  (`bee dev regen` is render-skill-trees, onboard --apply,
  release-manifest --write, `router.rs:90-92`) — follow the guard's own
  named remedy.
- `packages/bee-rs/crates/bee/tests/registry_dispatch.rs` — the tripwire that
  walks every registry entry in both directions. It proves the flip and proves
  `recovery window` still refuses.
- `packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs` — session
  record read/write.
- `docs/handbook/register.md:304,384` — the declared-but-not-built rows;
  `recovery scan` leaves that list, `recovery window` stays.

## Canonical References

- `docs/history/sweep-at-every-door/plan.md` — the approved plan this is
  slice 2 of; epics E3 and E4 and their proof rows.
- `docs/knowledge/areas/workflow-state/claims-and-ownership.md` — R97-R101,
  the shipped sweep contract this feature inherits.
- `docs/knowledge/areas/workflow-state/recovery.md` — the area concept this
  feature changes.
- Two registry descriptions this feature falsifies and must correct:
  `recovery.scan`'s "Cheap and **side-effect-free** — never triggers mining",
  and `sad-2`'s D6 decline text, which points at `bee cells claim-next`
  because `recovery scan` did not exist when it shipped.

## Outstanding Questions

### Deferred To Planning

- [ ] Does `detect_crash_candidates`' clean-end-trio check belong in the
  release criteria, or only in the report? It currently filters the reported
  candidates; whether a session with a clean transcript ending but an expired
  claim should still have that claim released needs deciding against the code.
- [ ] Does `bee recovery scan` refuse from a granted worktree like its sibling
  cells verbs, or take the full door like `cells finish`
  (`docs/knowledge/areas/worktree-parallelism/control-plane-topology.md:100-117`)?
  It writes session records on the control plane and parks cells in a store —
  the same split `cells finish` had to resolve.
- [ ] Whether D8's revival clearing needs the `sessions` lock on the heartbeat
  path, given that path is throttled and fail-open by design
  (`hooks/state_sync.rs`).

## Deferred Ideas

- Pid-liveness on session records — still needs a stored pid neither claims nor
  sessions carry.
- `bee recovery window` — stays unbuilt per D3.
- Closing the >900s heartbeat gap. Still an accepted residual risk.

## Handoff Note

CONTEXT.md is the source of truth. D2 and D3 are carried verbatim and keep
their original decision ids; D7 and D8 are new. Downstream work cites D-IDs,
never reinterprets them.
