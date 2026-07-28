# st-3 — brief payload contract suite (D3)

**Status:** [DONE]
**Worker:** exec-st3

## Outcome

Added 5 checks to `packages/bee/tests/test_bee_cli.mjs` covering assertions
(a)-(e) over st-1's `status --brief`: exact 7-key shape with no extras and
`route: null` before any route is recorded (a, b-before); brief payload
under 1024B on a hermetic fixture (d); the exact one-line text render (e);
full `status --json` (no `--brief`) still carries its pre-existing keys,
untouched by st-1 (c); and `route` populated in `--brief`, matching full
status byte-for-byte, once `state route --set` records one (b-after). One
hermetic temp-dir fixture (built via `start-feature` + gate through the
real dispatcher) carries the whole before/after story. `GATE_NAMES` added
to the file's existing `../lib/state.mjs` import for the text-render check.

## Files touched

- `packages/bee/tests/test_bee_cli.mjs`

## Verification

`node packages/bee/tests/test_bee_cli.mjs` — 334 passed, 0 failed (up from
329 before this cell; +5 new checks, all confirmed individually PASS).
Full trace/evidence: `.bee/cells/st-3.json`.

## Deviations / resolution history

None. Cell executed as specified; no deviations from the assigned scope.
A test-to-source ratio guard (test-economy D3) required a `ratio_waiver`
in the cap evidence, since this is a `change_class: test` cell touching
only a test file — recorded as expected shape for a dedicated test cell,
not a scope smell.

Commit: `eae06c0c` — "test(status-diet): st-3 — brief payload contract suite (D3)"
