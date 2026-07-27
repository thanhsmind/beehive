---
date: 2026-07-26
feature: rust-port
categories: [proof-discipline, byte-compatibility, performance, orchestration]
severity: high
tags: [port, frozen-oracle, parity-harness, judge-loop, budget-supersession, preserve-order, coverage-gaps]
---

# rust-port Slice 2 — nine cells, four rework rounds, one budget superseded

Slice 2 ported the status read spine, the review derivation, the workflow-store projection, and the two heavy checkpoint handlers to the compiled runtime, and built the instruments that prove the port faithful. Every cell capped with a passing verify; four needed a rework round after an independent judge found the proof weaker than the claim.

## What Happened

**The port itself landed clean.** Status assembly reaches byte-for-byte equality with the frozen reference across six parity legs (machine-readable and rendered text, over a quiet and an enriched fixture, with and without full lane detail), each with its own seeded-divergence control asserted on the output diff specifically. The handlers reached sixteen then nineteen conformance cases against the frozen handlers. Workspace tests grew 298 → 318.

**The proofs were where the defects were.** Of the four rework rounds, none was triggered by wrong ported behaviour. Every one was a proof that could not see what it claimed:

- A reader named explicitly in a cell's own action had no test calling it at all — twenty-one green tests, and the one reader the cell existed to prove was unproven.
- A fixture's comment claimed a deep-equality property the fixture did not construct.
- A synchronization step's authoritative rebuild branch was never entered by any fixture: every case seeded zero work records and took the do-nothing shortcut, under a test named for the branch it never reached.
- Every store-file comparison in one suite compared parsed values, under a serializer configuration whose map equality ignores key order — while the contract those files live under is byte-for-byte equality.

**Two real defects surfaced from chasing those gaps.** Enabling the serializer's order-preserving feature — necessary for byte-parity — silently re-aliased map removal to a swap-with-last removal, changing the emitted key order at two pre-existing call sites, one of which writes a store file under the byte contract. Fixing that uncovered a second, independent key-order break: the ported lane record emitted the struct's declaration order where the reference emits its defaults' order.

**One defect had been live on the main branch for hours with no signal.** An earlier cell's verify is a binary invocation; the scheduled build runs the compiler and the test suite and never shells that binary. When a later cell grew a repository into the shared fixture, the binary's safety check refused it, and that verify stayed red across four merges while every visible signal stayed green.

**The performance target moved, with evidence.** Status assembly measures 52 ms at the 95th percentile spawn-inclusive on the size-pinned fixture, against 179–193 ms for the reference — a real 3.5× win with zero subprocesses — but ten times the original 5 ms target. The measured cause is duplicated reads inside a single invocation: a 700 KB journal parsed four times, a 250-record directory scanned six times, transcript roots walked twice. Perfect elimination floors near 17–20 ms. The target was superseded per its own escape clause to an interim 70 ms regression guard with a mandatory follow-up, rather than the fixture being shrunk.

**One orchestration failure was mine.** A judge raised five findings on one cell; I relayed four. The fifth — a whole trigger branch with no fixture — was lost in the relay and only surfaced when the next round's judge repeated it, costing an extra cell.

## Root Cause

Three distinct roots, and it is worth keeping them apart:

1. **A comparison inherits the blind spots of the type it compares.** Parsed-value equality under an order-preserving map is set equality. Nothing about the assertion looks wrong; it simply cannot observe the dimension the contract is made of. The same shape recurs whenever the contract's dimension (bytes, order, timing, identity) is not the dimension the comparison operates on.
2. **Green is a statement about what ran, never about what did not.** Every gap above — the uncalled reader, the unentered branch, the unexercised trigger, the unreached lane path — is a case where the suite reported truthfully on a smaller thing than anyone believed it covered. Two of them were caused by inputs the harness inherited rather than pinned: an ambient session identifier that resolved to nothing in the fixture quietly removed a whole branch from every developer's run.
3. **Structured findings degrade when they pass through free text.** A judge's verdict is a list of typed checks with ids. Relaying it as prose lost one. Nothing in the loop compared what came back against what went out.

## Recommendation

1. **When enabling a dependency feature, audit what it re-aliases, not only what it adds.** Sweep every existing call site of the affected operations in the same change, and state the blast radius in the deviation record. A deviation record that says "purely additive" about a flag that changes a data structure's identity is a false claim about behaviour, however true it is about the manifest.
2. **Compare along the contract's own dimension.** Under a byte contract, compare bytes; redact volatile values in the raw text and make the redactor fail closed. Never parse to normalize. Add a permanent meta-test asserting both halves — that the naive comparison really is blind here, and that the real one catches it — so the instrument itself is guarded when the blindness comes from a dependency.
3. **For every branch a cell exists to port, name the fixture that reaches it and prove reachability by removal.** Neutralize the seeding and require the test to fail. A test's name is not evidence, and an empty-collection shortcut is the commonest way a named branch goes unentered. For table-driven cases, a row set where no case carries two competing fields cannot detect a priority-order regression at all.
4. **Pin every ambient input a harness reads, then assert the pin worked.** Clear inherited identity, set per-scenario values, give both legs identical environments, and require a positive marker in the compared output proving the branch you pinned for was reached. When a new scenario does not grow the compared payload, it probably reached nothing new.
5. **Give every verify command a home that re-runs it.** A verify that is a binary invocation, not a collected test, has an expiry date nobody wrote down. Register it in whatever the scheduled run executes, or wrap it in a test the runner collects. Treat fixture generators as having consumers: when one cell grows a shared fixture, check every proof that reads it in the same change.
6. **When a budget proves unreachable, publish the measurement and move the budget explicitly.** Report the profile that explains the number, record the supersession with the follow-up that tightens it, and keep the per-command budgets separate so a loose one cannot retire a tight one. Never shrink the fixture, never widen the tolerance.
7. **Relay judge verdicts mechanically, never as prose.** Enumerate every non-PASS check id from the prior verdict into the rework dispatch, and require each to come back named as fixed or explicitly deferred with an owner. Free-text summarizing of a structured verdict is how a finding disappears.
8. **Give reviewers a sanctioned scratch area inside the working tree.** Falsification — reintroduce the defect, watch the specific test go red — is the strongest instrument a judge has, and the judge that could not run it produced the weaker round. Scratch under the tree, swept afterwards, keeps that instrument available without letting a reviewer touch tracked files.

## Orchestration notes (first-hand, not in the cell records)

Two interruptions this slice left no trace in any cell trace, because they are harness events rather than work events: a worker terminated mid-run by a usage limit, and another by a dropped connection. Both resumed from their own transcript with reservations and claim intact and lost nothing. A third case is worth naming: a previous session claimed a cell and stopped before writing a line, leaving a claim owned by a dead session with a stale heartbeat. Re-claiming and dispatching was correct, but nothing in the tooling distinguishes "claimed and being worked" from "claimed by a session that no longer exists" except a heartbeat a reader has to interpret.
