# vd-2 — Migrate legacy validating state and close the write guard's fail-open tail

**Outcome:** [DONE] — Per D13 (two coupled parts). Part 1: added
`coerceLegacyPhase` (state.mjs, next to `isKnownPhase`) mapping legacy
`'validating'` -> `'planning'`, applied at `readState`, `readStateStrict`,
and `laneRecordFrom` (the single merge point shared by `readLane` AND
`readLaneStrict`) — covers both the default store and the lane path with
one function, not two copies. Part 2: closed `checkWrite`'s true
fall-through tail (guards.mjs:1366, confirmed by re-reading — not the
`GATED_PHASES` branch's own local return at :1319) from an unconditional
`return { allow: true }` to a deny narrowed to `!isKnownPhase(phase)`
(imported from state.mjs, no second hardcoded phase list). `reviewing`,
`scribing`, `compounding`, `grooming` still fall through to allow, exactly
as before. `TERMINAL_PHASES`/`GATED_PHASES` branches untouched. Both
changes mirrored byte-identically into `.bee/bin/lib/` last, in this same
commit, after verifying against the source copies only (never the live
hook mid-edit).

**Verify:** `node packages/bee/tests/test_guards.mjs && node packages/bee/tests/test_bee_write_guard_hook.mjs`
```
test_guards.mjs: 62 passed, 0 failed
test_bee_write_guard_hook.mjs: 33 passed, 0 failed
(combined exit 0)
```
Red-first evidence (git show of the prior state, per D9): `git show
bf1042e8:packages/bee/lib/guards.mjs` lines 1363-1367 show the
unconditional `return { allow: true }` tail; the same commit's
`test_guards.mjs` (lines 1057-1059) pinned it with a phase-`'executing'`
fixture asserting `allow === true`. That exact assertion passes green
against the pre-cell code — the fail-open door D13 closes. Full text:
`.bee/cells/vd-2.json` trace.verification_evidence.

**Files + commit:** `packages/bee/lib/state.mjs`, `packages/bee/lib/guards.mjs`,
`.bee/bin/lib/state.mjs`, `.bee/bin/lib/guards.mjs`,
`packages/bee/tests/test_guards.mjs` (this cell's fixture flip landed here;
`test_bee_write_guard_hook.mjs` needed no edit — none of its fixtures use an
unrecognized phase). Commit recorded after this report; full trace/evidence:
`.bee/cells/vd-2.json`.

**Deviations:**
- Swapped one unrelated `phase: 'validating'` literal (test_guards.mjs, the
  docs-history-code test) to `'planning'` — it's passed directly to
  `checkWrite` with no session, so it never reaches the read-side coercion,
  and under the new tail it would false-deny on an unrecognized phase for a
  test that has nothing to do with phase gating.
- Added 4 new tests pinning must_haves the existing suite didn't cover:
  legacy `state.json` coercion + `isKnownPhase` acceptance (the `state set`
  precondition), legacy lane coercion reaching `checkWrite` via a
  session bound to the lane, the four known-but-unhandled phases still
  allowing, and a no-`state.json` repo staying on the pre-existing intake
  path rather than the new deny.
- Several other pre-existing `phase: 'validating'` fixtures (lines ~449,
  ~1332, ~1426, ~1517) write real files to disk and read them back through
  `readState`/`readLaneStrict`/`resolvePipeline` — left untouched since the
  new coercion already turns them into `'planning'` transparently and none
  of their assertions depend on the literal phase name.
