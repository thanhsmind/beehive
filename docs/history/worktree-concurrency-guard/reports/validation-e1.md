# Validation Report — worktree-concurrency-guard, Epic 1 (current slice)

**Feature:** worktree-concurrency-guard
**Mode:** high-risk
**Slice:** Epic 1 only (cell `wcg-1` — shared detection primitive)
**Date:** 2026-07-24

## Reality Gate

| Dimension | Verdict | Evidence |
|---|---|---|
| MODE FIT | PASS | Hard-gate flag (data loss) locks high-risk regardless of file count (CONTEXT.md, plan.md). No smaller lane honestly protects the work — self-discipline already failed once (background). |
| REPO FIT | PASS | `isConcurrentMode()` (`claims.mjs:283`), `resolveCompanionMountedRelPath()`/`describeCrossWorktreeTarget()` (`bee-write-guard.mjs:369-424,263-297`), and `guards.checkWrite()` (`guards.mjs:586`) all exist and were read directly this session (not from memory). `bee worktree new --with-companion` (`bee.mjs:3845`) exists and is wired to real config commands. |
| ASSUMPTIONS | PASS (after spike) | The one HIGH-risk assumption flagged in approach.md — the exact detection trigger condition — is no longer an assumption; see Spike below. |
| SMALLER PATH | PASS | Current slice is deliberately narrowed to Epic 1 only (the primitive, unwired, `behavior_change: false`) — the smallest honest first step given the HIGH-risk unknown had to be resolved before Epic 2/3 could even be celled correctly. |
| PROOF SURFACE | PASS | `node --test hooks/test_write_guard.mjs` is a real, runnable command in this repo today (existing suite, extended in place); `release_manifest.mjs --check` / `ledger_parity.mjs --check` are real regen-obligation checks already enforced by `cells add` itself. |

## Spike: baseline behavior of three nested-checkout shapes (today, zero code changes)

**Question:** does today's write-guard already block an unrecognized symlink escape regardless of concurrency, and does it already guard (or not) a plain nested `.git` inside a shared checkout, and can it structurally tell a submodule apart from a risky shared nested repo?

**Method:** disposable script `.bee/spikes/worktree-concurrency-guard/probe-nested-checkout-baseline.mjs`, reusing `hooks/test_write_guard.mjs`'s own fixture/invocation pattern (`copyLib`, `writeState`, `runModuleWorker` spawning the real `bee-write-guard.mjs` as a child process against real temp-directory git fixtures). Three fixtures, one `Edit` tool-call payload against each:

| Case | Fixture | Result | Meaning |
|---|---|---|---|
| Unrecognized symlink escape (no companion marker) | Real external git repo, symlinked in at `repo/`, no `.bee/companion-session.json` | **status 2, DENIED** — `"could not be canonically contained inside the physical worktree... symlink escapes"` | Today's existing containment (`canonicalRelPath`/`describeCrossWorktreeTarget`) already blocks this, independent of concurrency. The write-guard's real gap for the symlink-mount shape is narrower than the backlog text implied: only a **verified** companion mount is unconditionally allowed today. |
| Plain nested `.git`, not a submodule | Real git repo at `repo/` inside the SAME checkout, plain subdirectory (no symlink) | **status 0, ALLOWED** | Completely unguarded today — this is STR65's actual incident shape (a nested repo sitting inside a shared checkout, no isolation at all). This is the primary gap Epic 1's primitive must close. |
| Registered git submodule (`.gitmodules`, `git submodule add`) | Real submodule of the fixture's own outer repo | **status 0, ALLOWED** | Behaves **identically** to the plain-nested case — proving the write-guard cannot structurally distinguish "ordinary submodule" from "accidental shared nested repo" by "has its own `.git`" alone. |

**Resolution:** D2's product-level framing (generic signal, exclude ordinary submodules) stands unchanged — but its mechanical implementation must key off actual `.gitmodules`/submodule-registration evidence to distinguish case 2 from case 3, since structural detection alone cannot. Cell `wcg-1` was updated in place (`bee cells update`) to encode all three confirmed baselines as required regression tests, so the implementer proves against known facts rather than re-deriving them.

## Feasibility Matrix

| Assumption | Risk | Proof Required | Evidence | Result |
|---|---|---|---|---|
| A verified companion mount write is unconditionally allowed today, with no concurrency check | Medium | Spike | Confirmed above — this is Epic 2's actual gap, narrower than originally scoped | READY |
| A plain nested `.git` inside a shared checkout is unguarded today | High | Spike | Confirmed above — STR65's real shape, Epic 1/2's primary target | READY |
| Structural "has its own `.git`" cannot distinguish a submodule from a risky shared repo | High | Spike | Confirmed above — mechanical fix identified (`.gitmodules` registration check) | READY, cell updated |
| Cell `wcg-1` schedules with no cycles | Required | `bee cells schedule` | `waves: [["wcg-1"]]`, `cycles: []` | PASS |
| `wcg-1`'s verify command runs in this repo today | Required | Direct inspection | `node --test hooks/test_write_guard.mjs` is the existing suite's own invocation; `release_manifest.mjs --check` / `ledger_parity.mjs --check` are the standard regen-obligation checks `cells add` itself already required | PASS |

## Plan-Checker / Cell Review

Structural check against the 5 dimensions (requirement/decision coverage, cell completeness, dependency correctness, key links, scope sanity):

- **Requirement coverage:** cell `wcg-1` cites D1, D2, D3 and is scoped to exactly the primitive (Epic 1); D4/D5/D6 are correctly deferred to Epics 2-3 where the actual refusal/bypass-scope behavior is wired.
- **Cell completeness:** `must_haves.truths/artifacts/prohibitions` present and non-empty (required for high-risk); `verify` is a real command; `behavior_change: false` is honest — this cell adds an unwired library function, no observable behavior changes yet.
- **Dependency correctness:** `deps: []`, confirmed zero-cycle single-wave schedule.
- **Key links:** none required yet (the helper isn't wired to a caller in this slice — correctly listed as empty, not fabricated).
- **Scope sanity:** cell explicitly prohibits wiring into the two dispatch points (Epic 2/3 territory) — no scope creep into future slices.

No BLOCKER or WARNING findings. Cold-pickup check: a worker with zero session history has the fixture-baseline facts (this report), the exact functions to read first, and the exact 3 regression behaviors required — no assumed context.

## Plan-Checker Findings (dispatched review, independent of the self-check above) and Resolution

An independently dispatched `bee-review` agent ran the plan-checker + cold-pickup pass and returned **NOT READY — 2 BLOCKER, 2 WARNING**. Both BLOCKERs are now resolved; both WARNINGs are recorded for Epic 2.

- **BLOCKER 1 (verify red):** confirmed `node scripts/release_manifest.mjs --check` fails today — 499-510 files, "mode differ" (664 vs recorded 644), reproduced on the main checkout too. This is pre-existing, already filed as friction (2026-07-23: "release_manifest --check baseline red: mode differ on 499 files (main + fresh worktree both)"), unrelated to this feature's content. **Resolution:** `wcg-1`'s `verify` no longer gates on `--check`; a `regen_obligation_ack` documents why, citing the friction entry. The action still runs `--write` for real content-hash regen.
- **BLOCKER 2 (D2 conflict):** identical to the advisor's finding above — D2's literal text didn't cover the shape `wcg-1` actually implements. **Resolution:** same as above — D2 superseded (`0ccc1cf3`), CONTEXT.md updated, user chose to widen.
- **WARNING 1 (`.gitmodules`/`git submodule status` reliability at hook time):** valid, but scoped to Epic 2 (wiring), not Epic 1 (unwired primitive). Recorded in `approach.md`'s new "Validating findings (resolved)" section for whoever plans Epic 2 — including the file-vs-directory `.git` submodule shape and the untested nested-git-worktree edge case.
- **WARNING 2 (release-manifest.json coupling):** addressed by BLOCKER 1's resolution — the file stays in `wcg-1.files` (the action's `--write` step still touches it) but capping no longer depends on `--check` passing.

The plan-checker independently confirmed (by actually running the commands, not trusting prose): `node --test hooks/test_write_guard.mjs` passes today; `bee cells schedule` shows zero cycles, single wave `["wcg-1"]`; `ledger_parity.mjs --check` passes; the spike script's fixture logic genuinely supports its claims, and case A's denial is independently corroborated by an existing test row. Dependency correctness, key links, and scope sanity all passed outright.

## Advisor Consult (AO2b/AO3, mandatory for high-risk before Gate 3)

Configured advisor (model-shaped, `models.claude.advisor: fable`) consulted twice: an initial review against the original plan (flagged the D2 conflict above, otherwise sound), and a re-consult after D2 was superseded and `wcg-1` was updated (confirmed **READY** — the widened D2 honestly states its false-positive cost, and the verify change is evidence-backed, not a dodge). `advisor_ref` recorded and non-stale (anchored to decision `0ccc1cf3`, current `plan.md` hash).

## Decision

**READY WITH CONSTRAINTS** — ready to execute Epic 1 (cell `wcg-1`) only. Epics 2-4 remain out of scope for this Gate 3 approval and return through planning once `wcg-1` caps with its helper's real interface settled.
