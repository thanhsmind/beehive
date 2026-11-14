# pstack Rust checks — Delivery

**Feature slug:** pstack-rust-checks
**Date:** 2026-11-14
**Decisions:** `docs/history/pstack-rust-checks/CONTEXT.md` (D1..D5)

## What shipped

Five checks that `pstack-craft-adoption` named and never built. Each one
reports or adds information. None adds a refusal.

| Decision | Cell | Commit | What a user gets |
|---|---|---|---|
| D1 | psrc-1 | ac8555d86 | `bee plan check [--feature <slug>] [--json]` runs four checks on `plan.md`: section order, the cited class playbook, every locked D-number, and the verbatim evidence of each `read` claims row. It exits 10 with findings. `bee gate --preview` prints the same findings as warning lines and still succeeds. |
| D2 | psrc-2 | 0dd04e5ba, ff0c2d767 | A worker prompt for a cell with `deps` carries an Upstream cells block. A capped dependency shows title, outcome, commit, files (20, then a count) and deviations (each cut to 300 characters); any other dependency shows its status. A cell without deps renders as before. |
| D3 | psrc-3 | 0e2f4d230 | A cap stores `trace.report.patch_id`. The carried-proof check of `bee worktree merge` reads it first and computes from the commit only when none is stored. The preflight stays advisory. |
| D4 | psrc-4 | 583351553 | `bee close` prints one warning per proof-line path that is missing under the repo root, and its JSON gains `dead_proof_pointers [{cell, path}]`. It never changes the outcome. |
| D5 | psrc-5 | af4f1abb2 | `bee dispatch wave --run --rolling --limit N` keeps at most N herding runs live and starts the next ready cell as each run ends. Native Agent-tool items return once under `native`. `--rolling` without `--run` refuses and names `--run`. |

The writer cell (psrc-6) synced the five knowledge concepts and added one
sentence to `skills/bee-planning/SKILL.md` § Gate: run `bee plan check`
before presenting the gate.

## Proofs each cell recorded

- psrc-1: verify green; `plan_group` 8 tests, `plan_claims` 21, `registry_contracts` 12 passed.
- psrc-2: the verify chain and the template-vars test pass after the leader fixed the worker-cell template drift (ff0c2d767).
- psrc-3: verify green; cap stores the patch-id and merge reads it first, with the compute-from-commit fallback.
- psrc-4: verify green; close warns on missing proof paths and the outcome is unchanged.
- psrc-5: the rolling refill tests and the two wave-door tests pass; one unrelated template-drift test failed at cap and was fixed by ff0c2d767.
- psrc-6: `bee knowledge check`, `bee dev release-manifest --check` and the `skill_triggers` test, run after `bee dev regen`.

## Departures recorded

- psrc-5 touched three files beyond the plan list as mechanical companions: the boolean-flag list in `catalog.rs`, the pinned flag count in `reservations/flags.rs` (247 to 248), and the two wave-door tests in `cells/tests.rs`.
- psrc-1 to psrc-5 left the skill update to psrc-6, which owns it.

## Skipped

- Plan step 6 (statechart) and step 7 (formal verification): no such skill on this host. psrc-5's tests cover the refill order instead.
- Promoting any check to a refusal: out of scope.
- Pattern candidates from `bee knowledge consolidate`: all five only restated the sync-ack deviation, so none was filed.
