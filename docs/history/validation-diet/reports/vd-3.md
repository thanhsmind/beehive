# vd-3 — Merge the shape and execution gates into one approval, preserving both preconditions

**Outcome:** [DONE] — Per D2, `bee state gate` gains a flag-alone `--merge`
opt-in: `bee state gate --merge --approved true` flips
`approved_gates.shape` and `approved_gates.execution` together in one call,
mutually exclusive with `--name` (refused together). The standalone
`--name <gate>` path is byte-for-byte unchanged when `--merge` is absent.
Per D14, the merged path inherits the high-risk advisor-consult refusal via
`requireFreshAdvisorForHighRisk`, a helper extracted from the pre-existing
inline check (never copied) and shared by both the plain `--name execution`
branch and the `--merge` branch. Per D15, `--merge` stamps
`approved_for_plan_rev` on BOTH `shape` and `execution` (an array of two
stamp entries, normalized by a new `findGateStamp` helper), so `state
plan-rev bump` revokes both instead of leaving the merged gate
half-revoked; a plain `--name` approval still stamps only its own gate,
unchanged. `state-projection.mjs`'s `workflowGatesToApprovedGates` needed no
functional change — it already reads any gate's rev generically — so its
touch here is a comment correction (previously said "only execution is ever
stamped," which D15 makes inaccurate).

**Verify (cell's recorded command):**
`node packages/bee/tests/test_cli_state.mjs && node packages/bee/tests/test_state_projection.mjs && node packages/bee/tests/test_bee_cli.mjs`
```
test_cli_state.mjs: 120 passed, 0 failed
test_state_projection.mjs: 32 passed, 0 failed
test_bee_cli.mjs: 391 passed, 0 failed
(combined exit 0)
```
Full trace/evidence: `.bee/cells/vd-3.json`.

**Files + commit:** `packages/bee/bee.mjs`, `packages/bee/lib/state-projection.mjs`,
`packages/bee/lib/command-registry.mjs` + their 3 `.bee/bin` twins,
`packages/bee/tests/test_state_projection.mjs`. Commit: see `git log` for
this cell's commit (message carries `vd-3`).

**Deviations:**
- **Interface changed from the cell's literal text.** The cell's `action`
  describes "a merged approval path" as if it were its own command; my
  first implementation did exactly that — a new `state.gate.merge` registry
  entry / `handleStateGateMerge` handler. `node packages/bee/tests/test_bee_cli.mjs`
  then failed a suite-wide invariant outside my declared files
  ("every registry entry had its example executed at least once") — new
  registry entries need `assertExampleOk` coverage added inside
  `test_bee_cli.mjs`, which is not in this cell's `files` and which I must
  not edit. Rather than touch a file outside scope, I refactored the merge
  behavior into a new `--merge` boolean flag on the EXISTING `state.gate`
  registry entry (already covered by a pre-existing, untouched
  `assertExampleOk('state.gate', ...)` call). D2/D14/D15's substance is
  identical either way — one call, one flag, same advisor precondition,
  same plan-rev stamping — only the CLI surface shape changed (`bee state
  gate --merge --approved true` instead of `bee state gate merge --approved
  true`).
- **Found and fixed a real parser bug while wiring the new flag.** `--merge`
  is not in `bee.mjs`'s `FLAG_ALONE_BOOLEANS` closed set by default, so the
  generic arg parser greedily consumed the next token as `--merge`'s value
  (`--approved`), then choked on the bare `true` token — exactly the bug
  class the surrounding comment already documents for `show`/`set`/
  `feature-verify-pending`/`all-but-active`. Added `'merge'` to that set
  with a matching comment; verified no other command already uses a
  `--merge` value-flag (only the unrelated `worktree merge` subcommand
  exists, parsed as a positional token, not a flag).
- Migrated 3 hand-written `phase: 'validating'` fixtures in
  `test_state_projection.mjs` (~L227, ~L368, ~L386) to `'planning'` — vd-1
  already removed `'validating'` from the phase enum; each site only needed
  *some* phase distinct from the fixture's `'swarming'`/default, so intent
  is unchanged.
- Added 6 new tests to `test_state_projection.mjs` (spawning the real
  `bee.mjs` CLI via the shared `runModuleWorker` helper, never a throwaway
  probe) proving: one `--merge` call flips both gates; `--merge --name`
  refuses; the standalone verb still works; the high-risk advisor refusal
  and its fresh-ref pass-through under `--merge`; and `state plan-rev bump`
  revoking both gates when approved via `--merge`. This is the only test
  file in the cell's declared `files`, so all CLI-level proof for D2/D14/D15
  lives here rather than in `test_cli_state.mjs`/`test_bee_cli.mjs`.
