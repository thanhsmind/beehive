# Validation — rust-port Slice 3 (cells 22-24)

Lane: high-risk. Plan: `docs/history/rust-port/plan-slice3.md` (frozen at Gate 2). Verdict: **READY WITH CONSTRAINTS**.

## Reality gate

| Check | Result | Evidence |
|---|---|---|
| MODE FIT | PASS | Four risk flags — public contracts (reader signatures two live hooks call), a covered contract must change (every status-reader oracle calls today's signatures), existing proof must be replaced (parity suites must survive a signature change), multi-domain (bee-core + status + two hooks + bench). High-risk is the honest lane; `standard` would skip the persona panel on a change two per-event hooks depend on. |
| REPO FIT | PASS | Every call site the slice touches was re-derived from source by the plan-checker, not taken from prose. The duplicate-read anchors, the hook call sites (`chain_nudge.rs:133`, `state_sync.rs`), and the archive fallback (`cells.rs:152-178`) all resolve. |
| ASSUMPTIONS | PASS (after repair) | The load-bearing assumption — that reads can be counted per invocation without a production branch — was tested and came back **negative**: `#[cfg(test)]` cannot work because bee-core's `cfg(test)` is inactive when queen-bee's integration test links it, and a non-default cargo feature would be absent from the release binary bee-parity actually spawns. The cell now names an always-compiled atomic counter as the decision and owes a cost measurement instead of an off-state. |
| SMALLER PATH | PASS | Measuring time instead of counting reads was considered and rejected: a timing change cannot distinguish "reads deduplicated" from "the machine was quieter". The instrument is what makes the dedup falsifiable. |
| PROOF SURFACE | PASS (after repair) | Both counting cells' verify commands were under-covering their own must-haves; all three now run the targets their truths name. |

## Feasibility matrix

| Assumption | Risk | Proof required | Evidence | Result |
|---|---|---|---|---|
| Reads are countable per invocation without a production branch | HIGH | Mechanism resolved before dispatch | Plan-checker read the read sites and ruled out two of three mechanisms; the third ships in release | RESOLVED — mechanism decided, cost measurement owed |
| Today's baseline is 4 / 6 / 2 | HIGH | Call-graph derivation | **FALSIFIED as stated**: `status.rs:539` was omitted and `cells.rs:332` is feature-gated (the bench fixture writes a null feature). The totals reconcile per fixture; they are not fixture-independent | REPAIRED — the baseline is now a derived per-fixture table |
| Signature change is contained | MEDIUM | Workspace-wide caller enumeration | Two hook call sites confirmed; `last_durable_settlement`'s self-loading branch named | ACCEPTED |
| Dep resolution survives a pre-loaded inventory | HIGH | Archive-fallback coverage | `read_cell` searches `archive/*/<id>.json`; `list_cells` skips `archive/`; zero existing coverage | REPAIRED — a test with an archived capped dep is now required, with the correct feature-subdirectory path |
| The dedup is measurable | MEDIUM | Bench builds what it measures | queen-bench resolves the binary as a sibling of its own executable and has no cargo dependency on it | REPAIRED — verify builds first |
| Schedule | — | `cells schedule` | Three waves, `[22] → [23] → [24]`, zero cycles, zero unsatisfiable deps, no empty file globs | PASS |

## Plan-checker (adversarial, opus) — 5 blockers, all repaired

The checker verified all four of the split's justifications independently and found them true, then found five blockers:

1. **The baseline anchor set was wrong and self-sealing.** `status.rs:539` (`list_cells (counts)`) was omitted entirely, `cells.rs:332` does not fire on the bench fixture, and the total of 6 was reached by those two errors cancelling. The cell simultaneously forbade correcting the numbers, which would have sent a worker hunting a phantom instrument bug on any fixture with a feature set.
2. **None of the three counts is unconditional as stated.** Decisions: 2 unconditional, 2 crash-candidate-gated. Cells: 5 unconditional, 2 conditional. Transcript roots: 1 unconditional, not 2. Asserting them flat violates the standing house rule (decision `af2b0d2a` rule 3) that a fixture must be shown to reach the branch its test names.
3. **Dep resolution's archive fallback had no guard.** Feeding `ready_cells` a pre-loaded active-only inventory makes archived capped deps read as uncapped, and ready cells silently vanish — with byte-parity green, because the parity fixture has no archived deps.
4. **Two verify commands did not cover their own must-haves.** `--test read_accounting` selects one target; the must-haves named the parity legs and `heavyhooks_conformance`.
5. **The honest-failure path failed its own gate.** The status gate is a strict `p95 < budget`, so setting the budget to the measured p95 guarantees red — while the cell also forbade widening. Unsatisfiable as written.

## Cell review (cold pickup, opus) — 5 criticals, all repaired

1. **The frozen plan is the stale copy, and every cell pointed at it.** `plan-slice3.md:33/:52/:65` still carries the superseded numbers and the ruled-out "inert unless armed" seam shape. A cold worker reads the plan first. Each cell now names those lines as superseded by the cell itself.
2. **The archive path in the trap paragraph was wrong.** The real contract is `archive/*/<id>.json` — feature-subdirectoried. A fixture at `archive/<id>.json` is invisible to `read_cell`, so the required test would have gone red against unchanged code and the natural "fix" would have been to loosen it.
3. **The mechanism was presented as open while the proof surface had already closed it.** Now decided explicitly.
4. **A must-have named targets the verify never built.** `hook_conformance`, `modelguard_conformance` and the write-guard targets were never compiled, so a compile break from the signature change could ship green. Cell 23 now runs the whole `queen-bee` package.
5. **The release-cost must-have had no runnable evidence.** Cell 22's verify now runs the bench.

Minor findings folded in: the status gate anchor corrected to `main.rs:397`; `GitMemo` at `reviews.rs:302`; the sub-20 ms tiebreak defined (the headroom rule wins, 25 ms is a ceiling not a floor); the profile is only emitted on a red gate, so a green run must capture it deliberately; counters must be site-labelled so the baseline table is the test's own output; the recovery hoist must sit above the early return at `recovery.rs:453-455`; report paths added to the file globs; the consistency finding must be logged as a decision rather than only a trace field.

## Advisor consult (fable) — PROCEED WITH NOTES, all 4 applied

Full digest: `docs/history/rust-port/reports/advisor-slice3.md`. The sixth blocker, which neither earlier pass caught: **every instrument in the slice measures `build_status`, while the signature change lands on two per-event hooks.** An eager shared-read shape would convert chain-nudge's conditional cells scan into an unconditional one on every hook event with every verify leg green. Cell 22 now baselines both hook entry points and cell 23 re-asserts them. Second note: counter placement must sit at the lowest shared read primitives, or the refactor moves the load out from under the counter and the 1/1/1 proof becomes vacuous. Third: the consistency claim is corrected downward — one read instant per store, cross-store consistency still unguaranteed, never "snapshot semantics" — and no concurrent-mutation differential test is to be added. Fourth: the CI perf-smoke number travels with the dev budget.

The advisor also confirmed the split and recorded a recommendation for the deferred distribution slice: run a disposable dogfood spike before the CI build-matrix work, so real-runtime integration facts surface early. That is recorded for the deferred plan, not this slice.

## Constraints carried into execution

- The frozen plan is stale on the baseline numbers and the seam mechanism; the cells supersede it and say so.
- The seam ships in release builds; its cost must be measured, not asserted.
- Reach-proofs for conditional sites are baseline-only evidence — after the dedup, a hoisted read reports one count whether or not the conditional consumer still consumes.
- The hook read profile is part of the proof, not a side effect.

## Approval

Gate 3 auto-approved under `gate_bypass=total`. Advisor consult ran before the gate as required for high-risk work and is recorded as a non-stale `advisor_ref`. Verdict: READY WITH CONSTRAINTS; 10 findings across two review passes plus 4 advisor notes, all applied to the cells before dispatch.
