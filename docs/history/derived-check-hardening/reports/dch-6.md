# dch-6 — Pin the six hand-copied terminal-phase memberships with a parity suite

**Status:** `[DONE]`

## Outcome

Added `scripts/tests/test_terminal_phase_parity.mjs` (E5). Scan locations are
derived, not hardcoded: it globs every `.mjs` file in `packages/bee/lib` and
its `.bee/bin/lib` twin for top-level `const <NAME> = new Set([...])`
declarations named `TERMINAL_PHASES` / `NO_WORK_PHASES` / `TERMINAL_LANE_PHASES`
(the three names are the one irreducible domain fact — a hardcoded six
file:line list would be this suite's own version of the defect it exists to
catch). Only the three names and the two lib roots are fixed; file and line
are read off the match every run. Asserts all 12 discovered declarations
(6 canonical + 6 `.bee/bin` twins, verified live) agree with each other and
with `KNOWN_PHASES` (`packages/bee/lib/state.mjs`), naming the offending
`file:line` on drift.

**Deviation:** `scripts/run_verify.mjs` was not edited. Its `SUITES` array is
auto-discovered by globbing `test_*.mjs` under `scripts/tests/` (cs-4,
contention-split) — the new file registers itself; hand-editing the array
would reintroduce what cs-4 deliberately removed. Full note in the cell trace.

## Verify

Selftest (fixture-based, proves the checker bites — never mutates this tree):
`FAIL` on a drifted member set (names exact file:line), on a member outside
`KNOWN_PHASES` (names the stray value), and on a root contributing zero
declarations; `PASS` on a fully-agreeing fixture.

Live-repo proof: temporarily changed `packages/bee/lib/scratch.mjs`'s
`TERMINAL_PHASES` to `['idle']`, reran the suite — failed naming
`packages/bee/lib/scratch.mjs:62` exactly — then `git checkout --` restored
it (`git diff` clean, confirmed).

Cell verify, `node scripts/tests/test_terminal_phase_parity.mjs && node packages/bee/tests/test_guards.mjs`:
```
PASS test_terminal_phase_parity: 12 membership declaration(s) ... agree with
     each other and with the 9-entry KNOWN_PHASES enum: ["compounding-complete","idle"]
62 passed, 0 failed
```
Exit 0.

## Files + commit

- `scripts/tests/test_terminal_phase_parity.mjs` (new)
- commit `60f16da7`

## Reservations

Released (3 reservations + 3 cross-worktree holds).

Full trace: `.bee/cells/dch-6.json`
