# pd-2 — Slice test: regen guard wave-barrier coverage

**[DONE]**

## Outcome

Extended `packages/bee/tests/test_bee_cli.mjs`'s existing REGEN_OBLIGATION suite
with three checks against pd-1's `wave-barrier` sentence: (a) the refusal
message names `"wave-barrier"` and the orchestrator's wave-close debt; (b) an
`addCell` with `regen_obligation_ack: "wave-barrier: ..."` is accepted and the
ack round-trips verbatim on the return value and the persisted cell file; (c) a
touching cell with neither the required manifest-check verify nor an ack still
refuses. Guard logic in `packages/bee/lib/cells.mjs` untouched (verified
unmodified in the diff).

## Verify

`node packages/bee/tests/test_bee_cli.mjs`

```
318 passed, 0 failed
```

## Files + commit

- `packages/bee/tests/test_bee_cli.mjs`
- Commit: `2de60edc`

## Consults

None.

Full trace/evidence: `.bee/cells/pd-2.json`.
