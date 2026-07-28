# pd-1 — Regen guard message: wave-barrier alternative (D2)

**[DONE]**

## Outcome

Extended `regenObligationRefusal`'s skip sentence in `packages/bee/lib/cells.mjs`
to name the recognized `"wave-barrier"` ack value, deferring the regen chain to
the orchestrator's wave-close commit (parallel-default D2). No change to when
the guard fires or refuses.

## Verify

`node --test-name-pattern REGEN packages/bee/tests/test_bee_cli.mjs 2>/dev/null || node packages/bee/tests/test_bee_cli.mjs`

```
315 passed, 0 failed
```

(harness does not honor `--test-name-pattern`; full file ran as a superset
covering the touched function — recorded via `cells verify`.)

## Files + commit

- `packages/bee/lib/cells.mjs`
- Commit: `34df2704`

## Consults

None.

Full trace/evidence: `.bee/cells/pd-1.json`.
