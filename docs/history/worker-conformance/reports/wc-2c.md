# wc-2c — A withdrawn test cell owes nothing and hides nothing

**Status:** [DONE] · worker Dave · lane high-risk · change_class bugfix

## Outcome

`testCellDebt` read a deliberately **dropped** `change_class: 'test'` cell two
contradictory ways at once: it counted toward `testCellCount` (suppressing the
`'missing'` kind) *and* landed in `offenders` as "not capped". One line, placed
**before** the counter increments, skips a dropped test cell entirely — so a
feature that drops its only test cell now falls through to the `'missing'` kind
rather than passing clean. Dropping the cell is never cheaper than writing it.
Only `dropped` is exempt; `open`, `claimed` and `blocked` still refuse.

## Verify

Red-first (`node packages/bee/tests/test_bee_cli.mjs`, before the fix):

```
401 passed, 11 failed
FAIL wc-2c: door "phase-departure (state set --phase)" REFUSES with the "missing" kind ...
     got: ... has 1 consolidated test cell(s) not green: wc2cdropphasedepartu-t (status: dropped — not capped)
FAIL wc-2c: door "phase-departure (state set --phase)" OPENS when a dropped test cell sits beside a green capped one ...
```

Green after the fix:

```
412 passed, 0 failed          # node packages/bee/tests/test_bee_cli.mjs
release_manifest --check: 448 file(s) match stored manifest
```

All 20 generated `wc-2c` rows pass (4 doors × {dropped⇒missing, dropped-beside-green⇒opens, open/claimed/blocked⇒still refuses}).

Full trace and evidence: `.bee/cells/wc-2c.json`.

## Files + commit

- `packages/bee/lib/state.mjs` — the dropped skip, before `testCellCount += 1`
- `packages/bee/tests/test_bee_cli.mjs` — 20 rows generated over the existing `DEBT_DOORS` matrix
- Regen chain (in order): `.bee/bin/lib/state.mjs`, `.bee/onboarding.json`,
  `docs/history/codex-harness-hardening/release-manifest.json`

## Deviations

1. The `start-feature` door owns an **earlier** guard (nonterminal / claimed
   cells must be resolved before a new feature starts), so it answers before
   the debt door is asked. The strict `(status: X — not capped)` wording is
   therefore skipped for that one door on the open/claimed/blocked rows;
   refusal itself, the cell id, and "never the `'missing'` kind" are asserted
   unconditionally on every door. Recorded because it narrowed an assertion I
   first wrote stricter than the system.
2. That tolerance was first written as `if (/not green/.test(out))` — keyed on
   output text rather than door identity. The advisor showed this weakened all
   four doors at once (a refusal that ever lost the phrase would silently skip
   the strict assertion everywhere). Rewritten to branch on
   `door.id.startsWith('start-feature')`, confining the tolerance to the one
   door that earned it.

## Consults

1 consult · advisor **fable** (`state advisor-ref record`, digest stored).
Ask: does skip-before-increment truly close the escape hatch; is the
conditional assertion a real weakening; any prohibition or second-order
effect. Answer: hatch closed (with all test cells dropped, `testCellCount`
stays 0 and `offenders` empty, so a code-touching capped behavior cell forces
`'missing'`); the conditional **was** a real weakening and was tightened as
above; no prohibition violated — `featureVerifyDebt` is untouched and
structurally unreachable (it filters `status !== 'capped'`), the wc-2
unrecorded branch is unchanged, no bypass wiring touched.
</content>
</invoke>
