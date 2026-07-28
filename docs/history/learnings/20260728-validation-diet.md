# Learnings — validation-diet (2026-07-28)

**Feature:** removed bee's predictive validation layer — the `bee-validating` skill,
the `validating` phase, the feasibility matrix, the delta rule, and the
`state validation-cache` verbs — merged Gate 3 into Gate 2, and replaced
probe-then-delete evidence with build-emitted evidence.

**Scale:** 16 cells, 3 slices, 27 commits (`bea867a7`…`2d34ca34`).
**Outcome:** `PASS run_verify: 115 suite(s)`. Feature-verify record green.
**Decisions:** D1-D15, `docs/history/validation-diet/CONTEXT.md`.

---

## L1 — The machinery to derive sibling suites already exists, and doctrine forbids using it

Three separate regressions escaped every cell-scoped verify and were caught only by
the post-wave full run:

| Escape | Cell that caused it | Suite that caught it |
|---|---|---|
| exported helper broke an exact-set export census | vd-2 (`lib/state.mjs`) | `test_misc.mjs`, via vd-5 |
| impact registry drifted | several | full verify only |
| stale knowledge index cascaded into 13 suites | vd-12 | full verify only |

The root cause is not carelessness. `scripts/impact-registry.json` **already maps**
`packages/bee/lib/state.mjs` → `packages/bee/tests/test_misc.mjs` as a direct edge —
confirmed live with `node scripts/impact_registry.mjs --query packages/bee/lib/state.mjs`.
`scripts/run_verify.mjs:813-909` already implements `--impacted-from-git` against that
same registry.

But `skills/bee-planning/references/planning-reference.md:162` forbids it from a cell's
verify: *"still never authored as a per-cell `verify`"* — and the wave-level fallback
that used to cover the gap was retired by `main-verifies`
(`skills/bee-executing/references/worker-details.md:166`, "main-verifies D4 retired the
wave-close impacted run"), which closed at 07:33:39 the **same day** this feature opened
Gate 1 at 14:09:40.

So the only unscoped check in the whole feature lifecycle was its very last step.

The proof that suite selection is a judgment call rather than a derivation: vd-1 and
vd-2 both edited `packages/bee/lib/state.mjs`. vd-1 included `test_misc.mjs` in its
verify. vd-2 did not. Same file, same registry edge, different human guess.

**Cheapest fix:** make the cap door cross-check. For each path in `cell.files` present
in the registry, run `queryRegistry(registry, cell.files, {level:1})` and require every
returned suite to appear in `cell.verify` — refuse or warn loudly, mirroring the
existing `ratioWarning` at `packages/bee/lib/cells.mjs:2003`. No new derivation logic;
`queryRegistry`/`normalizeQueryPath` already exist at `scripts/impact_registry.mjs:449-486`.

For the `wave-barrier` class, make the **next claim** — not the cap — run
`impact_registry.mjs --check`, `knowledge index --check` and `release_manifest.mjs --check`
and refuse a new claim while any is red. That converts "owed by the orchestrator at wave
close" from a memory obligation into a gate.

## L2 — A crash in a coverage gate is strictly worse than a failure

`scripts/tests/test_doctrine_parity.mjs` built its scan set from `git ls-files` — the
git **index** — then `readFileSync`'d each path. A `wave-barrier` deferred the
mirror-deletion regen, so the index still named deleted files, and the suite died on
`ENOENT`, hiding every assertion behind it. Fixed in vd-13 by one existence filter at
`:136`.

The same defect class is **still live** at `scripts/tests/test_portable_paths.mjs:23-34`:
`tracked` comes from `git ls-files -z` with no existence filter. It does not crash
(the loop only does string analysis), so it manifests as a silent coverage hole instead
— a staged-but-unindexed file with an illegal Windows character sails through green.

`scripts/tests/test_installers_e2e.mjs:191-199` does the same thing and is safe, because
it guards with `fs.existsSync` before use.

**Cheapest fix:** `scripts/tests/test_scan_set_hygiene.mjs` — grep `scripts/tests/**/*.mjs`
for `ls-files` and flag any file where a later `readFileSync`/`statSync` over that list
has no intervening `existsSync` filter. Mechanical, and it would have caught both.

## L3 — A hand-copied enum membership that fails open is invisible to its own tests

`GATED_PHASES` (`lib/guards.mjs:142`) and `PHASE_GATE` (`hooks/bee-session-close.mjs:28`)
each hardcoded `"validating"` independently of `PHASES`, and both fell through to
`allow`. Every existing gating test hand-built its phase fixture, so a broken cut would
have passed green while leaving source writes ungated. What made the cut safe was not
the removal — it was forcing one test (`scripts/tests/test_conformance.mjs`) to drive
the **real** state machine instead of a fixture.

The sweep found this is far wider than the two constants the feature fixed. The literal
set `['idle','compounding-complete']` is hand-copied verbatim in **six** places under
**three** names, none importing from `state.mjs`:

| Constant | Location |
|---|---|
| `TERMINAL_PHASES` | `packages/bee/lib/guards.mjs:151` |
| `TERMINAL_PHASES` | `packages/bee/lib/compaction.mjs:81` |
| `TERMINAL_PHASES` | `packages/bee/lib/scratch.mjs:62` |
| `NO_WORK_PHASES` | `packages/bee/lib/inject.mjs:235` |
| `NO_WORK_PHASES` | `packages/bee/lib/intent.mjs:49` |
| `TERMINAL_LANE_PHASES` | `packages/bee/lib/recovery.mjs:40` |

No test cross-checks them against each other or against `KNOWN_PHASES`. Only
`guards.mjs`'s copy is pinned at all. A partial edit on the next phase change produces
divergent behavior with nothing to catch the disagreement — and `guards.mjs:1294`
governs actual write-deny behavior, so a miss there re-opens exactly the hole D3/D13
just closed.

CONTEXT.md's Deferred Ideas flagged this, scoped to 2 of the 6+ instances.

**Cheapest fix:** `scripts/tests/test_terminal_phase_parity.mjs` — assert all six
constants' contents are identical to each other and consistent with `KNOWN_PHASES`,
naming the offending `file:line` on drift. Cheaper than deriving all six at import time,
and it ships as a tiny cell.

## L4 — CI never runs on push or pull request, so `main` can sit red indefinitely

Four suites were red on `main` at baseline `33d58a7e` before this feature started. They
were found only because a full verify was run **by accident** before the first claim.

`grep -n "^on:" .github/workflows/*.yml`: all three workflows (`ci.yml`, `windows.yml`,
`canary.yml`) are `schedule` + `workflow_dispatch` **only**. `ci.yml:4-7` is a nightly
cron at `0 16 * * *`. On failure it files or updates a GitHub issue and nothing else —
it never blocks a commit from landing.

Meanwhile `.bee/config.json` sets `commands.test` to `run_verify.mjs --impacted-from-git`,
which is by construction blind to any suite outside the changed files' closure. So
nothing in the daily loop and nothing in CI stops a red from persisting on `main` for up
to 24 hours, and the `verify-red` issue is advisory.

The workflow's CI status gate (AGENTS.md critical rule 14) says to check CI before the
first `cells claim`. It was not honored in this session — that is a process failure on
top of a machinery failure, and both are worth naming.

**Cheapest fix:** add `pull_request: {branches: [main]}` (or at minimum
`push: {branches: [main]}`) to `.github/workflows/ci.yml`. If that is too expensive
against the impacted-first velocity model, make the existing `verify-red` issue
self-blocking: refuse to open a new feature while an open, undismissed `verify-red`
issue exists for `main`.

## L5 — The advisor consult paid for itself, on cell decomposition rather than feasibility

The AO3/AO13 consult found that vd-1's own verify command (`test_cli_state.mjs`)
hand-wrote the very phase vd-1 was deleting — so vd-1 was **guaranteed red at its own
boundary**, with the fix in nobody's declared scope. Fixture ownership was reassigned
before any code was written: each cell fixes the suites its own verify trips on.

Worth noting against this feature's own thesis. The owner removed the predictive layer
because it reasoned about whether a plan *would* work. The advisor's value here was not
feasibility — it was reading the actual declared `files` and `verify` fields against the
actual test estate and finding an arithmetic contradiction. That is a different, cheaper,
and more mechanical kind of check than the one that was removed.

## L6 — What the "ship and look" tradeoff actually cost, measured

The plan forecast 13 cells. Execution needed 16, and roughly **7 of them were reactive
repairs** discovered only by running suites after earlier cells landed: vd-0, vd-5,
vd-6, vd-7, vd-13, vd-14, vd-15.

Only one of the seven looks like something a genuine pre-build challenge would plausibly
have caught: vd-3's first implementation of the merged gate added a new registry entry
and broke a structural invariant in `test_bee_cli.mjs` (every registry entry needs an
executable example) — discoverable by reading the suite before choosing the CLI shape.
It was caught and fixed inside the same cell at near-zero cost, which is the empirical
gate working as advertised.

The rest — exact export sets, exact byte counts, exact directory counts, an unrelated
commit landing mid-swarm — is arithmetic that only exists once the diff is real. No
feasibility matrix would have priced it in advance, and per D6 the matrix covered
nothing anyway (zero machine coverage; nothing to migrate).

**Honest accounting:** the approach paid a real, measurable tax at the seams — 7 of 16
cells — but the tax was for the defect class empirical gates catch cheaply, not the
class predictive review exists to catch. The decision holds.

## L7 — Surprise: `cells cap` never reads the field its own downstream obligations key off

vd-9, vd-10 and vd-11 all capped with `trace.behavior_change: false` despite being
`change_class: "behavior"`. `cells cap` (`packages/bee/lib/cells.mjs:1830-1835`) reads
the **top-level** `cell.behavior_change`; those cells only carried the value nested
inside `trace.behavior_change`, set at authoring time, which the cap logic never reads.

vd-1 (`change_class: "migration"`) shows the field correctly as `true`, which is why the
gap is invisible until several `behavior`-class cells land back to back.

Consequence: scribing-debt detection and the semantic goal-check judge tier both key off
`trace.behavior_change === true`, so all three cells silently miss those obligations.
Post-cap correction is refused by the tooling, so this cannot be repaired in place.

Unanticipated by every one of D1-D15 — none of them is about capping mechanics. Surfaced
only because this was the first feature with three high-risk `behavior` cells in a row.

## Residual, unfixed at close

- `skills/bee-xia/references/research-brief-template.md:54` still routes proof
  obligations to `bee-validating`. D11's completeness criterion was run as a manual
  sweep, never turned into a standing test, and no cell re-ran it after slice 3 closed —
  so the criterion that was supposed to guarantee completeness is itself unenforced.
- `packages/bee/hooks/test_write_guard.mjs:664` still hand-builds the retired phase
  value. Currently green only because the read path coerces it (D13).
- `.bee/cells/vd-12.json` has no `docs/history/validation-diet/reports/vd-12.md`; every
  other cell has one. Reporting-pipeline gap.

## Patterns worth reusing

- **Derive scope from the verify command, never from a planning-time list.** vd-4, vd-7,
  vd-8 and vd-13 all re-derived their own scope and each found items the dispatch missed.
  D7's rationale states the general rule: *a locked decision that carries line numbers
  becomes false the moment the file shifts.*
- **Path-scoped commits in a shared checkout.** With live siblings, the concurrent-worker
  guard refuses plain `git add`/`git commit`. Workers that succeeded used a temporary
  `GIT_INDEX_FILE` → `write-tree` → `commit-tree` → compare-and-swap `update-ref`.
- **Check mirror byte-identity after every edit, not at the end.** Edit the canonical
  copy, verify against it in isolation, mirror last, in the same commit.
- **Extract, never copy, a precondition being shared across a new path.** vd-3 extracted
  `requireFreshAdvisorForHighRisk` rather than duplicating the advisor check into the
  merged gate.
