# dch-8 — Make the cap door's registry import survive a vendored-lib-only fixture

**Status:** `[DONE]`

## Outcome

dch-1 wired the cap door's E1 impact-registry cross-check with a **static
top-level import** (`import { queryRegistry, normalizeQueryPath } from
'../../../scripts/impact_registry.mjs'`). `packages/bee/hooks/test_write_guard.mjs`'s
`copyLib()` vendors only `.bee/bin/lib/*.mjs` into a bare temp root with no
sibling `scripts/` directory, so the whole module load threw
`ERR_MODULE_NOT_FOUND` before `capCell`'s guarded try/catch ever ran — six
assertions in rows 5c/5d silently stopped being checked, and the suite
reported failures nowhere near the real cause. Same class of miss dch-1
itself exists to warn about.

Fix: dropped the static import; `queryRegistry` is now resolved with
`await import(...)` **inside** the existing guarded try block, right where
it's used. `capCell`'s `withStoreLock` callback had to become `async` to
support the `await` (`withStoreLock` already does `return await fn()`, so
this changes nothing about the lock's semantics). An absent module now hits
the same catch as a missing/malformed registry file — silent skip, never a
throw. Dropped the unused `normalizeQueryPath` import (never called in this
file). E1's loud non-blocking warning on a *reachable* registry is
untouched — no logic inside the try changed, only where the module gets
imported from.

## Proof (real cases, no probe script authored)

- **Before (current code, pre-fix):** `node packages/bee/hooks/test_write_guard.mjs`
  -> `6 FAILURE(S)`, all in rows 5c/5d (`legacy bee_cells.mjs cap` /
  `dispatcher bee.mjs cells cap`, each missing 3 assertions) — the exact
  failure the cell describes, reproduced before any edit.
- **After (fix applied, twins synced):** same command -> `ALL PASS`, rows
  5c/5d green. `node packages/bee/tests/test_cells.mjs` -> `132 passed, 0
  failed`, including all four dch-1 E1 tests (reachable-registry warning
  fires and names the missing suite; already-mentioned suite gets no
  warning; missing registry is a silent skip; malformed registry is a
  silent skip). `node packages/bee/tests/test_cli_cells.mjs` -> `40 passed,
  0 failed`.
- **E1 contract still live in the real checkout:** capping this very cell
  (registry reachable, real repo root) printed the expected non-blocking
  stderr warning naming the direct-edge suites missing from `dch-8`'s
  `verify` — proof the loud path survived the lazy-import change, not just
  the fixture's silent path.

## Verify

`node packages/bee/hooks/test_write_guard.mjs && node packages/bee/tests/test_cells.mjs && node packages/bee/tests/test_cli_cells.mjs`
-> exit 0 on all three. Recorded via `cells verify`.

## Files + commit

- `packages/bee/lib/cells.mjs`
- `.bee/bin/lib/cells.mjs` (byte-identical twin, moved in the same commit)
- Commit: see `.bee/cells/dch-8.json` trace for the sha recorded at cap.

Full trace: `.bee/cells/dch-8.json`
