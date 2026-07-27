# Advisor consult — rust-port Slice 3 (pre-Gate-3)

Advisor: `fable` (model-shaped, read-only). Consulted before Gate 3 per AO2b/AO3. Advice is data for the gate decision, never the decision, and never overrides a locked decision.

**Verdict: PROCEED WITH NOTES (4).** All four notes applied to the cells before the gate; see below.

## What the advisor checked itself

Read all three cells in full, `plan-slice3.md`, `CONTEXT.md`, the six most recent decisions, and the rust-port-15 bench report; spot-checked the load-bearing anchors in source — the archive fallback (`cells.rs:154-172`), `list_cells` skipping `archive/` (`cells.rs:109`), `scribing_debt`'s no-feature early return, and the two hook call sites (`chain_nudge.rs:133`, `state_sync.rs`'s `cell_counts` loop). Confirmed the anchors the cells cite are real.

It tested and rejected the obvious objection ("an optimization with no measured payoff"): the rust-port-15 profile shows duplicated store I/O dominating — roughly 2.7 ms per decisions parse × 4, ~3 ms per cells scan × 6, a 15.7 ms recovery block containing a second transcript scan — against a measured ~13 ms in-process perfect-dedup floor.

## The strongest argument against proceeding, and the sixth blocker

**Every instrument in the slice measures `build_status`, while the signature change lands on the hooks.** Cell 23's blast radius includes `chain_nudge.rs:133` (`scribing_debt`) and `state_sync.rs` (`list_cells`), which run per lifecycle event under a tighter latency expectation than status. Today `scribing_debt` early-returns with no cells scan when no feature resolves. An eager shared-read shape would convert that conditional scan of 250+ cell files into an unconditional one on every hook event — and every verify leg in the slice would stay green, because `heavyhooks_conformance` checks correctness rather than reads, `read_accounting` never touches the hook entry points, and the bench never runs a hook.

Second, interacting blind spot: **counter placement can make the 1/1/1 proof vacuous.** Cell 22 pinned the counters' units but not their placement. A counter at today's reader-function entries would be bypassed by the very refactor it judges, since cell 23 introduces a new load point. The counters belong at the lowest shared read primitives, keyed by store class, so any path touching a store increments. Relatedly, cell 22's removal-based reach-proofs stop discriminating after the dedup (a hoisted read reports one count whether or not the conditional consumer still consumes), so they are baseline-only evidence and must be labelled as such.

## Within-invocation consistency

Acceptable, with a wording correction that matters: the dedup does **not** produce a snapshot. It reduces roughly eight read instants to one per store; decisions, cells and transcripts are still loaded at distinct moments, and a journal read racing an in-flight append remains possible. Record it as "each store is read at one instant per invocation; cross-store consistency remains unguaranteed, exactly as today."

D3 contracts the stores' formats and semantics, not the reference implementation's incidental read count, and nothing can legitimately depend on a torn read — so the divergence from the still-re-reading oracle under concurrent mutation is acceptable. No new proof is owed: the one property worth testing (a second invocation observes an intervening write) is already a cell-23 must-have. A concurrent-mutation differential test against the oracle would assert nondeterministic behaviour nobody contracted for and be flaky by construction — explicitly do not add one. Worth one sentence in the decision: status takes no D9 locks, so the lock and lease conformance surface is untouched.

## Was the split right

Yes. All four deferral facts were independently verified, and any flip today wires hosts to a binary that does not exist. There is no honest smaller flip inside this slice: a dogfood wiring of this repo's own release binary would either hand-edit managed wiring files — violating exactly the catalog-of-record discipline D7 exists to protect — or live in unmanaged local config that is not durable evidence, and it would put the live store of the repo *developing* the port behind hooks proven only on fixtures. It also shares no proof surface with this slice's cells.

The underlying instinct should not be lost, though: the deferred order (remaining hooks → distribution → flip) surfaces real-runtime integration facts — stdin plumbing, argv shape, working directory, wiring shape — last, after the distribution machinery is built. Recommendation for the deferred plan, not this slice: run a disposable dogfood spike (`.bee/spikes/`, local-only wiring, one or two fail-open observability hooks) at the head of the distribution step, as a cheap integration probe. A spike, not a flip: no catalog change, no checked-in wiring.

## The four notes, and how each was applied

1. **Extend read accounting to the two hook entry points** — applied to cell 22 (baseline both, with chain-nudge showing zero cells scans when no feature resolves and exactly one when a feature is set) and to cell 23 (re-assert those baselines after the change; conditional reads on the hook path stay conditional). This is where cell 23's consumer-side red-to-green transition now lives.
2. **Pin counter placement, not just units** — applied to cell 22: counters sit at the lowest shared read primitives, demonstrated by a test showing a direct primitive call increments the same counter a reader call does; reach-proofs labelled baseline-only evidence.
3. **Correct the consistency wording** — applied to cell 23: "one read instant per store, cross-store consistency still unguaranteed", never "snapshot semantics"; no concurrent-mutation differential test; the no-D9-locks note recorded.
4. **Carry the CI perf-smoke number with the dev budget** — applied to cell 24 at D5's 3× runner-variance ratio (a 25 ms dev budget implies 75 ms CI). The dogfood-spike recommendation is recorded here for the deferred distribution slice.
