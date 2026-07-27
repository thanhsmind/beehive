# plan — state-phase-lock-race (GH #70)

Mode **high-risk**. Decisions: `CONTEXT.md` D1–D8, plus the logged D2 amendment.

## Goal

Close the lost update on `.bee/state.json` so a live swarm can never read a
stale phase, by making every writer of that record hold the same `'state'`
lock — and prove it with a test that genuinely fails before the change.

## The four writers and what each needs

| # | Writer | Today | After |
|---|---|---|---|
| 1 | `bee-state-sync.mjs:127` (hook) | holds `'state'` | unchanged |
| 2 | `bee.mjs:2552` `withMutationLock`, workflow branch | holds `workflow:<id>` only | `workflow:<id>` → `'state'` (nested, in that order) |
| 3 | `bee.mjs:3284` `handleStateStartFeature` post-rebuild | no lock | wrapped in `'state'` |
| 4 | `bee.mjs:3304` `handleStateRebuildProjections` | no lock | wrapped in `'state'` |

The C1 fallback branch of `withMutationLock` (`bee.mjs:2555`) already takes
`'state'` and must NOT be double-wrapped — `lock.mjs` is non-reentrant, so a
second acquire of the same name from the same process stalls ~5 s and throws
`LockBusyError` (D3). Only the `if (wf)` branch changes.

`state.mjs:2592` (`startLane`, inside `startFeature`'s own `'state'` hold) and
`bee.mjs:2971` (`stateWorkerMutate`) and `bee.mjs:2945` (`handleStatePlanRevBump`)
are already correct and are not touched.

## Shape

```
bee.mjs:2552   if (wf) return withWorkflowLock(ctrlRoot, wf.id, fn);
            →  if (wf) return withWorkflowLock(ctrlRoot, wf.id, () => withStoreLock(root, 'state', fn));

bee.mjs:3284   if (lane) rebuildLaneProjection(...) else rebuildStateProjection(root)
            →  same, wrapped in withStoreLock(root, 'state', ...)

bee.mjs:3304   rebuildAllProjections(root)
            →  wrapped in withStoreLock(root, 'state', ...)
```

`rebuildStateProjection` / `rebuildLaneProjection` themselves are not touched
(D3) — they stay caller-serialized.

## Proof (D5)

New suite `scripts/tests/test_state_projection_race.mjs`, built in
`scripts/tests/test_store_lock.mjs`'s style — real OS child processes re-execing
the same file with a `--role` flag, never in-process async, because the writes
are synchronous and an async "race" never exercises them.

- Role A loops the state-sync-shaped write: `withStoreLock(root,'state', …
  rebuildStateProjection)`.
- Role B loops the CLI-shaped write: `withWorkflowLock(ctrlRoot, wf.id, …
  rebuildStateProjection)`.
- Detector: each role marks a shared `active.json` on entry to its critical
  section, holds, then asserts it was not clobbered — overlap is recorded to
  `violations.jsonl`. This bites directly on mutual exclusion, not on a final
  count that two racers could satisfy by luck.
- Lost-update assertion: a monotonic counter written through the same
  read-modify-write path must reach exactly `roles × iters`.
- Negative control: the same body with role B taking only the workflow lock —
  the pre-fix arrangement — must produce violations. Without this the suite
  could pass vacuously.
- The fixture is a temp repo the test creates; nothing is read from the live
  checkout, so no assertion can be satisfied by the ambient environment.

Red-first evidence: the suite is run against unmodified `bee.mjs` first and must
fail there; that output goes in the done-report before the fix lands.

## Advisor-required additions (consult 2026-07-27, `reports/advisor-consult.md`)

The consult refuted one of this plan's own validation claims. Three additions,
all inside `splr-1`:

1. **The inverse lock edge already exists and must be repaired here.**
   `handleStatePlanRevBump` holds `'state'` (`bee.mjs:2921`) and then calls
   `updateWorkflow` (`bee.mjs:2944`), which self-locks `workflow:<id>`
   (`workflow-store.mjs:389`). Verified independently. Restructure it to the
   canonical order — resolve the lane/workflow first, then
   `withWorkflowLock(ctrlRoot, wf.id, () => withStoreLock(root, 'state', body))`
   with `updateWorkflowAssumingLock` inside — so `workflow:<id>` → `'state'` is
   the single global order. Without this the fix trades a race for a
   deterministic ~5 s dual `LockBusyError`.
2. **The writer inventory above is incomplete.** Also in it:
   `state.mjs:2592` (already `'state'`), `bee.mjs:2971` (already `'state'`), and
   the narrow `bee.mjs:2465` workflow-branch fallback, which holds
   `workflow:<id>` only when the workflow closes between lock-branch selection
   (`:2551`) and the re-list (`:2463`) — silently fixed by the wrap. The test
   asserts the real invariant: **every production `writeState` runs under
   `'state'`**.
3. **The fix must be vendored into `.bee/bin/`.** This repo executes the
   vendored copy; `md5sum` of `.bee/bin/bee.mjs` equals `packages/bee/bee.mjs`
   today, so an un-vendored fix ships nothing and the suite would still be
   testing the canonical source.

Follow-up filed, not in scope: having the write-guard read workflow records
instead of the projection, removing its dependency on projection freshness.

## Cells

| id | scope | files | verify |
|---|---|---|---|
| `splr-1` | Close writers 2–4, repair the `handleStatePlanRevBump` inversion, re-vendor, and land the multi-process proof | `packages/bee/bee.mjs`, `scripts/tests/test_state_projection_race.mjs`, vendored `.bee/bin/` | `node scripts/tests/test_state_projection_race.mjs` |
| `splr-2` | rust-port mirror artifact (D7): `rust-port`-tagged PBI naming the behavior delta, affected parity fixtures checked/updated, and D6's constraint recorded so the future Rust `state`-group port lands already locked | `docs/`, `.bee/backlog.jsonl` via CLI, `crates/` comment only if a fixture moves | `node scripts/run_verify.mjs --impacted-from-git` |

Serial, not parallel — both cells touch overlapping files and `splr-2` depends on
what `splr-1` actually changed.

## Risks

- **Deadlock.** New edge `workflow:<id>` → `'state'`. The audit found no
  `'state'` → `workflow:<id>` edge anywhere in the repo, and no lock pair
  currently appears in both orders, so this introduces no cycle. Validation must
  re-confirm, specifically for `rebuildAllProjections`, which also rebuilds the
  reservations projection and could reach a `lease:<hash>` lock from inside the
  new `'state'` hold.
- **Self-deadlock.** Guarded by only wrapping the `if (wf)` branch. A test that
  runs a `state set` under a live workflow record catches a mistake here
  immediately (it would hang ~5 s then throw `LockBusyError`).
- **Throughput.** Two sessions on two different workflows now serialize on the
  mutation body. Accepted, D2 amendment.
- **Windows.** The lock primitive is already red on Windows CI for unrelated
  suites (D8). This feature does not fix that and must not be assumed green
  there.
