# et-1 — state route verb: validated route record + status/preamble surfacing + claim warn (D1-D3)

**Status:** [DONE]
**Worker:** exec-et1

## Outcome

New `bee state route --set|--show` verb: typed enum refusals for
`--class`/`--lane`/every `--flags` entry, non-negative-integer `--files`
(empty `--flags ""` = zero flags, a valid explicit value, not a missing
flag). Persists on the active feature's tracked record (session-bound lane
else default, no `--lane` targeting flag exposed — collides with the
route's own `lane` field, deferred per CONTEXT Outstanding Questions) and,
belt-and-suspenders, on the underlying live workflow-store.mjs record.
`status --json` gains a `route` block (null when absent). `inject.mjs`
preamble renders `Route: class=<c> | lane=<l> | flags=<n> [<names>] |
files=<n>` only when present. `cells claim` warns once on stderr when the
claimed cell's feature has no route — never refuses (D3 soft).

## Files touched

- `packages/bee/lib/workflow-store.mjs`
- `packages/bee/lib/command-registry.mjs`
- `packages/bee/bee.mjs`
- `packages/bee/lib/inject.mjs`
- `packages/bee/tests/test_bee_cli.mjs` (deviation — see below)

## Verification

`node packages/bee/tests/test_bee_cli.mjs && node packages/bee/tests/test_state.mjs`
— 319 + 44 passed, 0 failed. Full trace/evidence: `.bee/cells/et-1.json`.

## Deviations

1. Touched `test_bee_cli.mjs` (undeclared) — the new registry entry trips
   the pre-existing "every example executes" completeness test; added one
   minimal coverage check, deeper behavioral tests stay et-3's scope.
2. Belt-and-suspenders direct patch of the workflow-store.mjs record's
   `route` field, in addition to the projection write (literal D1 compliance).
3. `state.mjs` reserved but needed no edits (no new export required).

## Friction

None.
