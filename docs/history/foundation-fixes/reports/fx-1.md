# fx-1 — Workflow close transition + zombie-proof picker (D1+D2)

**Status:** [DONE]
**Worker:** exec-fx1

## Outcome

`startFeature` (`packages/bee/lib/state.mjs`) now closes the outgoing
feature's live workflow record(s) — `status: 'closed'` — right after the new
workflow is created (D1). `pickNewestActiveWorkflow`
(`packages/bee/lib/state-projection.mjs`) additionally excludes
`phase === 'compounding-complete'` records as defense in depth (D2).

Red-first: a throwaway probe (`.bee/spikes/foundation-fixes/`, deleted after
use per wave-barrier scope) reproduced both zombie shapes against pre-fix
code (`git stash` of the two changed files) — picker selected a lone
compounding-complete zombie; a second `startFeature` left the prior
feature's workflow record `status: 'active'`. Same probe green after
restoring the fix.

## Files touched

- `packages/bee/lib/state.mjs`
- `packages/bee/lib/state-projection.mjs`

## Verification

`node packages/bee/tests/test_state.mjs && node packages/bee/tests/test_workflow_store.mjs`
— 43 + 15 passed, 0 failed. Full trace/evidence: `.bee/cells/fx-1.json`.

## Deviations

None.

## Friction

None.
