# Derived Check Hardening — Context

**Feature slug:** derived-check-hardening
**Date:** 2026-07-29
**Exploring session:** complete
**Scope:** Standard
**Domain types:** RUN (CI + verify machinery), CALL (cells CLI surface), READ (doctrine residuals)

## Feature Boundary

Close the findings validation-diet's compounding pass surfaced: wire the impact
registry into the cap door, give the two index-derived scan sets a hygiene suite,
pin the six hand-copied phase memberships with a parity suite, fix the
`cells cap` field the downstream obligations key off, move the CI cron, and clear
the two live doctrine residuals. Ends when the full verify is green with all six
checks in place. Does not change what a cell's `verify` field is allowed to
contain, and does not add a blocking gate anywhere.

## Feature Origin

Every item here is a finding from `docs/history/learnings/20260728-validation-diet.md`
(L1-L7 and the Residual section) and the six friction rows filed at that feature's
close. Nothing here is new scope — it is the debt that feature's own analysis named.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| E1 | The cap door cross-checks `cell.verify` against the impact registry and emits a **loud warning**, never a refusal. For each path in `cell.files` present in `scripts/impact-registry.json`, run `queryRegistry(registry, cell.files, {level:1})`; every returned suite missing from `cell.verify` is named on stderr. Follow the existing `ratioWarning` shape at `packages/bee/lib/cells.mjs:2003`. | Owner's call. A refusal on this door would gate every future cell on registry freshness — a stale or wrong edge would block all work. The warning is weaker than the L1 finding wanted; that residual is recorded in E9. |
| E2 | `.github/workflows/ci.yml`'s cron moves to `0 23 * * *`, staying **once per day**. No `push` or `pull_request` trigger is added. | Owner's call, made with the tradeoff stated. **This does not close L4:** main can still carry a red for up to 24 hours, and CI still only files an issue rather than blocking anything. The change is the detection *time*, not the detection *gap*. The P1 friction row stays open and is re-stated in E9. |
| E3 | `scripts/tests/test_portable_paths.mjs` gets the existence filter its sibling already has (`test_doctrine_parity.mjs:136`), and its scan set unions `git status --porcelain` so untracked-but-real files are actually covered. | Today it reads `git ls-files -z` with no filter. It does not crash only because its loop never touches the filesystem — so the bug is the silent one: a staged-but-unindexed file with an illegal Windows character passes green. |
| E4 | A new suite `scripts/tests/test_scan_set_hygiene.mjs` flags any file under `scripts/tests/**` and `packages/bee/**` that derives a path list from `git ls-files` and later reads those paths with no intervening existence filter. | Turns the hand-grep that found E3 into a standing check. It would have caught both the `test_doctrine_parity.mjs` crash and E3's live hole. |
| E5 | A new suite `scripts/tests/test_terminal_phase_parity.mjs` asserts the six hand-copied terminal-phase memberships agree with each other and with `KNOWN_PHASES`, naming the offending `file:line` on drift. The six are not refactored to derive from the enum in this feature. | `TERMINAL_PHASES` at `lib/guards.mjs:151`, `lib/compaction.mjs:81`, `lib/scratch.mjs:62`; `NO_WORK_PHASES` at `lib/inject.mjs:235`, `lib/intent.mjs:49`; `TERMINAL_LANE_PHASES` at `lib/recovery.mjs:40`. Each copy has its own semantics layered on the shared list, so deriving all six is a larger change; the parity suite catches the same class at a fraction of the risk, and `guards.mjs:1294` — the copy that governs write-denial — is the one that must never drift silently. |
| E6 | `cells cap` resolves `behavior_change` from the top-level field **or** `trace.behavior_change`, whichever is set, so a cell authored with only the nested value caps with the correct flag. Already-capped cells are not retroactively corrected. | `packages/bee/lib/cells.mjs:1830-1835` reads only the top-level field. vd-9, vd-10 and vd-11 all capped `false` despite `change_class: "behavior"`, silently missing scribing-debt detection and the semantic goal-check judge. Post-cap correction is refused by the tooling, so the fix is forward-only. |
| E7 | The two live doctrine residuals are cleared: `skills/bee-xia/references/research-brief-template.md:54` routes proof obligations to `bee-planning`'s shape gate instead of the deleted `bee-validating`, and `packages/bee/hooks/test_write_guard.mjs:664` stops hand-building the retired phase value. | Both are validation-diet's own acceptance criterion (D11) left unmet. The write-guard fixture is currently green only because the read path coerces the value — it is a test proving the wrong thing for an accidental reason. |
| E8 | D11's completeness criterion becomes a standing check rather than a remembered sweep: the scan-set hygiene suite of E4 gains an assertion that no live file outside `docs/decisions/**`, `docs/history/**` and the legacy-coercion code path describes a retired workflow stage as current behavior. | E7's two residuals existed precisely because the criterion was run by hand once and never again. A criterion with no enforcement is a note, not a criterion. |
| E9 | Two residuals ship **open and named**, not silently closed: the cap door is a warning rather than a gate (E1), and CI's 24-hour blind window survives (E2). Their friction rows stay open at their recorded severity, and the close report states both plainly. | The owner made both calls with the tradeoff stated. Recording them as resolved would make the backlog lie about the repo's actual coverage. |

### Agent's Discretion

- Slice boundaries and cell count.
- The exact detection technique in E4 and E8 (static source scan versus dynamic
  import), provided the check derives its own ground truth rather than comparing
  two hand-authored lists.
- Whether E5's parity suite reads the six constants by source scan or by import.

## Existing Code Context

### Integration Points

- `packages/bee/lib/cells.mjs:1830-1835` — `capCell`'s `behavior_change` read (E6).
- `packages/bee/lib/cells.mjs:2003` — the `ratioWarning` shape E1 must follow.
- `scripts/impact_registry.mjs:449-486` — `queryRegistry` / `normalizeQueryPath`, reused by E1 with no new logic.
- `scripts/tests/test_doctrine_parity.mjs:136` — the existence filter E3 copies.
- `scripts/run_verify.mjs:875-909` — `gitImpactedFiles()` / `statusPorcelainFiles()`, the union source E3 follows.
- `.github/workflows/ci.yml:4-7` — the cron E2 moves.

## Canonical References

- `docs/history/learnings/20260728-validation-diet.md` — L1-L7 and the Residual section; every decision here traces to one of them.
- `docs/knowledge/patterns/20260728-a-derivation-the-tooling-computes-but-doctrine-forbids-where-it-is-needed.md` — E1's pattern.
- `docs/knowledge/patterns/20260728-a-scan-set-from-the-git-index-crashes-the-gate-that-guards-it.md` — E3/E4's pattern.
- `docs/knowledge/patterns/20260728-one-membership-hand-copied-six-times-has-no-owner-and-no-alarm.md` — E5's pattern.

## Outstanding Questions

### Deferred To Planning

- [ ] Whether E4 and E8 belong in one suite or two. One file keeps the "derive your ground truth" theme together; two keeps each failure message narrow.

## Deferred Ideas

- Deriving the six terminal-phase memberships from `KNOWN_PHASES` at import — the real hardening behind E5, deferred because each copy carries its own semantics. File as a PBI.
- Making an open `verify-red` issue refuse the start of a new feature — the other half of L4, declined with E2. Stays in the backlog.
- `.bee/cells/vd-12.json` has no worker report while every other validation-diet cell does; a reporting-pipeline gap, not this feature's scope.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. E9 is the one every
downstream step must honor at close: two findings ship deliberately unclosed, and
the close report says so.
