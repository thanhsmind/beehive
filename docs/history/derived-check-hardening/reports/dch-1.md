# dch-1 — Cap door warns when a cell's verify omits a direct-edge suite from the impact registry

**Status:** `[DONE]`

## Outcome

`capCell` (packages/bee/lib/cells.mjs, byte-identical twin .bee/bin/lib/cells.mjs)
now cross-checks each cell's static `files` against `scripts/impact-registry.json`
at cap time, reusing `queryRegistry`/`normalizeQueryPath` from
`scripts/impact_registry.mjs` verbatim (no new derivation logic). For every
`level:1` (direct-edge only) suite missing from the cell's `verify` field, it
writes a named, non-blocking warning to stderr and folds it into
`trace.warnings` alongside the existing `ratioWarning` (same shape/channel,
same non-blocking semantics). A missing, unreadable, or malformed registry is
a silent skip — wrapped in try/catch, never a throw.

Dogfooded on this cell's own cap: `dch-1`'s `verify` only covers
`test_cells.mjs`/`test_cli_cells.mjs`, but `packages/bee/lib/cells.mjs` has 9
direct-edge suites in the real registry — the cap printed the warning naming
all 9 missing suites and still capped (`status: "capped"`), live proof of the
non-blocking contract.

## Verify

Recorded command: `node packages/bee/tests/test_cells.mjs && node packages/bee/tests/test_cli_cells.mjs`
— 132 passed / 0 failed, then 40 passed / 0 failed (combined exit 0). Includes
4 new tests in `test_cells.mjs` (own dedicated fixture repos, never the shared
`root`, to avoid registry-fixture cross-test pollution):
- warns + still caps when verify omits a direct-edge suite (asserts both
  `trace.warnings` and the actual stderr bytes; also asserts the
  transitive-only sibling suite is never named — proves `level:1` scoping)
- no warning when verify already mentions the direct-edge suite
- missing registry file → silent skip, still caps
- malformed (invalid JSON) registry file → silent skip, still caps

A pre-existing test-economy D3 ratio-ceiling check (standard lane, >4)
fired on this diff (4 isolated fixture repos to cover 4 distinct
must_haves cleanly) and was waived with a recorded, audited justification
(`.bee/decisions.jsonl`) — the ceiling itself is unchanged.

## Files

- `packages/bee/lib/cells.mjs` (edited)
- `.bee/bin/lib/cells.mjs` (edited, byte-identical twin — diffed to confirm)
- `packages/bee/tests/test_cells.mjs` (edited, 4 new checks)

Full trace: `.bee/cells/dch-1.json`
