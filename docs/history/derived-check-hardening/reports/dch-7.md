# dch-7 — Clear the two doctrine residuals the retired stage left behind

**Status:** `[DONE]`

## Outcome

Both E7 residuals fixed, reading naturally rather than as find-and-replace
scars:

- `skills/bee-xia/references/research-brief-template.md:54` — proof
  obligations now route to bee-planning's shape gate instead of the
  deleted bee-validating stage.
- `packages/bee/hooks/test_write_guard.mjs` row 24 — fixture phase
  `"validating"` -> `"planning"` (a phase `guards.mjs`'s `GATED_PHASES`
  can actually produce), with a comment explaining the row now exercises
  the real gate policy instead of `readState`'s legacy-phase coercion.
  That coercion in `state.mjs` is untouched (locked decision).

Previously blocked: sibling `dch-1` added a static import of
`scripts/impact_registry.mjs` into `packages/bee/lib/cells.mjs`, which
`ERR_MODULE_NOT_FOUND`'d inside this suite's vendored-lib-only fixture
(rows 5c/5d, unrelated to dch-7's own rows 24/25, which passed clean
throughout). `dch-8` (commit `977e6bed`) made the import resolve lazily
inside cells.mjs's existing try/catch, fixing the fixture. Re-verified
after that fix landed.

## Verify

`node packages/bee/hooks/test_write_guard.mjs && node scripts/skill_lint.mjs`:
```
ALL PASS
1 advisory warning(s) — nothing blocks
```
(warning is pre-existing, unrelated to this cell's files)

## Files + commit

- `skills/bee-xia/references/research-brief-template.md`
- `packages/bee/hooks/test_write_guard.mjs`

## Reservations

Released.

Full trace: `.bee/cells/dch-7.json`
