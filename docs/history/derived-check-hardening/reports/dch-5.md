# dch-5 — Turn the two hand-run sweeps into a standing scan-set hygiene suite

**Status:** `[DONE]`

## Outcome

Added `scripts/tests/test_scan_set_hygiene.mjs`, carrying E4 and E8 as
standing checks. Registers via `run_verify.mjs`'s existing `test_*.mjs` glob
discovery — `run_verify.mjs` untouched.

- **Check 1 (E4):** flags a file under `scripts/tests/**` or
  `packages/bee/**` that derives a path list from a git-ls-files invocation
  and reads from it with no `existsSync` guard in the enclosing function.
- **Check 2 (E8):** flags a live current-behavior file (`skills/**/SKILL.md`,
  `skills/**/references/**`, `docs/knowledge/**`, `docs/specs/**`,
  `AGENTS.md`/`CLAUDE.md`) describing a retired workflow stage as current.
  The retired token(s) are derived from `state.mjs`'s
  `LEGACY_PHASE_COERCIONS`, never hardcoded. Exception set: exactly
  `docs/decisions/**`, `docs/history/**`, and the legacy-coercion code path.

Both checks are proved on synthetic `--selftest` fixtures (permanent, run
every time the suite runs) plus a live temporary-injection-and-restore
against the real tree, each correctly naming the offending file.

## Verify

`node scripts/tests/test_scan_set_hygiene.mjs && node scripts/tests/test_doctrine_parity.mjs && node scripts/tests/test_portable_paths.mjs` — all PASS. Recorded via `cells verify`.

## Files + commit

- `scripts/tests/test_scan_set_hygiene.mjs`
- Commit `4b272990071ee2cb4184ef696c102a645473f026` — "test(dch-5): add scan-set hygiene suite (E4 + E8)"

## Reservations

Released.

Full trace: `.bee/cells/dch-5.json`
