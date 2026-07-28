# st-1 — status --brief fast path (D1)

**Status:** [DONE]
**Worker:** exec-st1

## Outcome

`--brief` implemented in `packages/bee/bee.mjs` (`buildStatusBrief`/
`renderStatusBriefText`, `FLAG_ALONE_BOOLEANS`) and
`packages/bee/lib/command-registry.mjs` (new `brief` property + example).
Reads ONLY the state layer (`state.json` + `bypassLevel`/`shipVisibility`)
— zero cells/review/handoff/models reads. `--brief --json` emits exactly
the 7 D1 keys (route null when absent), 545B on this repo; `--brief` text
prints `phase=... feature=... mode=... gates=t/t/t/f bypass=...`. Full
`status` byte-shape unchanged. No `test_misc.mjs` allowlist was tripped,
so it was left untouched.

## Files touched

- `packages/bee/bee.mjs`
- `packages/bee/lib/command-registry.mjs`

## Verification

`node packages/bee/tests/test_bee_cli.mjs` — 329 passed, 0 failed.
Full trace/evidence: `.bee/cells/st-1.json`.

## Deviations / resolution history

First returned `[BLOCKED]`: the cell's original verify
(`test_bee_cli.mjs && test_misc.mjs`) could not record an honest pass —
`test_misc.mjs` carries a standing vendored-mirror parity guard that
necessarily trips while `.bee/bin` is unsynced, deferred to the
orchestrator's wave-barrier regen (this cell's own `regen_obligation_ack`
field). Consulted the advisor (fable), which concurred with blocking
rather than recording a documented-but-false pass. Orchestrator then
amended the cell's `verify` to `test_bee_cli.mjs` only (its own logged
authoring defect) and re-claimed the cell under exec-st1; re-verified
clean and capped. No new test authored — st-3 (slice-tail test cell)
owns the new `--brief` behavioral coverage in `test_bee_cli.mjs` per the
test-economy tier for `change_class: behavior` on a standard lane.

## Consults

1 consult — **fable**: asked whether to cap with a fully-disclosed
`--passed true` exception on the original (unamended) verify command, vs.
`[BLOCKED]`. Answer: `[BLOCKED]` — a verify-pass claim is a
machine-readable assertion regardless of annotation quality; exceptions
to the verify invariant are the orchestrator's to grant, not the
worker's to self-sanction. Endorsed dropping `test_misc.mjs` from the
verify command as the clean fix — which the orchestrator then did.
