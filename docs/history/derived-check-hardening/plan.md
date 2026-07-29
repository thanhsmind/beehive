# Derived Check Hardening — Plan

**Feature:** derived-check-hardening
**Lane:** standard (3 flags: public-contracts, covered-contract-change, multi-domain; no hard-gate flag)
**Date:** 2026-07-29
**Context:** `docs/history/derived-check-hardening/CONTEXT.md` — E1-E9 locked.
**Shape:** one slice, 7 cells.

## Approach

### Chosen path

Every item is independent debt from one prior feature's analysis, so this is a
flat slice rather than a staged migration. The only ordering constraint is that
two cells touch `packages/bee/lib/cells.mjs` (E1's warning and E6's field
resolution) — those serialize; everything else runs concurrently.

Two of the seven cells add **new suites** that derive their own ground truth
(E4/E8 hygiene, E5 parity). Those are the highest-value cells in the feature:
they convert findings that were caught by hand into checks that cannot be
forgotten. The rest are one-line-to-one-function repairs.

### Rejected

- **Deriving the six terminal-phase memberships from `KNOWN_PHASES`.** Each copy
  layers its own semantics on the shared list, so a mechanical derivation would
  change six behaviors at once to fix a drift risk. The parity suite (E5) catches
  the same class at a fraction of the blast radius. Deferred as a PBI.
- **Making the cap-door check a refusal.** Owner declined (E1). Recorded, not
  re-litigated.
- **Adding `push`/`pull_request` CI triggers.** Owner declined (E2). Recorded.

### Risk map

| Risk | Where | Mitigation |
|---|---|---|
| The cap-door warning fires on every cell and becomes noise nobody reads | `lib/cells.mjs` | Scope the query to `level:1` direct edges only, and print the missing suites by name — a warning that names a fix is actionable; a warning that says "check your verify" is not |
| E6 changes the flag every downstream obligation keys off, for all future cells | `lib/cells.mjs:1830-1835` | Resolve from either location, never overwrite an explicitly-set top-level value; existing capped cells stay untouched |
| A new hygiene suite (E4/E8) produces false positives and gets disabled | `scripts/tests/` | It must pass against the repo as it stands after E3 and E7 land — a new gate that ships red teaches everyone to ignore it |
| Two cells serialize on `cells.mjs` and the second silently reverts the first | `lib/cells.mjs` | Hard dependency in the cell graph, and the second cell's verify covers both behaviors |

### Files and order

`packages/bee/lib/cells.mjs` (+ `.bee/bin` twin) for E1 then E6;
`.github/workflows/ci.yml` for E2; `scripts/tests/test_portable_paths.mjs` for E3;
new `scripts/tests/test_scan_set_hygiene.mjs` for E4/E8; new
`scripts/tests/test_terminal_phase_parity.mjs` for E5;
`skills/bee-xia/references/research-brief-template.md` and
`packages/bee/hooks/test_write_guard.mjs` for E7.

## Cells

| Cell | Work | change_class | deps |
|---|---|---|---|
| dch-1 | E1: cap door queries the impact registry for each `cell.files` path and warns loudly, naming every direct-edge suite missing from `cell.verify`. Never refuses. | behavior | — |
| dch-2 | E6: `capCell` resolves `behavior_change` from the top-level field or `trace.behavior_change`. Forward-only. | bugfix | dch-1 |
| dch-3 | E2: `ci.yml` cron to `0 23 * * *`, still once daily. No new trigger. | refactor | — |
| dch-4 | E3: existence filter + `git status --porcelain` union on `test_portable_paths.mjs`'s scan set. | bugfix | — |
| dch-5 | E4 + E8: new `test_scan_set_hygiene.mjs` — unguarded `git ls-files` readers, plus the retired-stage completeness criterion. | test | dch-4, dch-7 |
| dch-6 | E5: new `test_terminal_phase_parity.mjs` over the six memberships. | test | — |
| dch-7 | E7: the two doctrine residuals — the xia template's routing line and the write-guard fixture's retired phase value. | bugfix | — |

`dch-5` depends on `dch-4` and `dch-7` because a new gate must ship green: both
its subjects have to be clean before the gate that checks them lands.

## Rollback

One commit per cell with the cell id — `git revert` per cell, any order except
`dch-2` before `dch-1`. The two new suites are additive; reverting them removes a
check and changes no behavior. E2 is a one-line config change.
