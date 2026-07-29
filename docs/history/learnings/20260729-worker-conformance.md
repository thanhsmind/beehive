# worker-conformance — close learnings

**Date:** 2026-07-29
**Lane:** high-risk (3 flags: public contracts, multi-domain, validation removal)
**Cells:** 7 capped, 1 dropped · **Verify:** 117 suites green
**Subject:** stop asking a worker to author evidence to close a unit of work;
make "do we need more tests?" the required thought instead of "write tests" the
required output.

## What the feature did

Two refusals in the completion helper became non-blocking recorded warnings.
Absence of proof is now marked, and that marker arms the feature-boundary door
without buying any other exemption. The trailing test unit stayed unconditional
but leads with a coverage judgement. Every heavier refusal survived and is
proven surviving.

## The finding that mattered most

**A marker reused for a second purpose inherits every exemption the first one
bought.** The locked decision originally said absence of proof should reuse the
existing "pending" marker. That marker is not inert: it short-circuits six
refusal sites. Reusing it would have disarmed the red-first tier and the ratio
ceiling in slice 1 — *before* slice 2's negative controls existed to notice. The
fix was a new field whose only power is arming the door.

Caught by: the Gate 2 dispatched review wave, before any code was written.

## Every wrong assumption, and what caught it

| Assumption | Caught by | Cost if it had shipped |
|---|---|---|
| The absence marker can reuse the "pending" flag | Gate 2 review wave | Red-first and the ratio ceiling silently disarmed |
| Removing two doors leaves the rest enforceable | Gate 1 fresh-eyes review of CONTEXT.md | A feature could close with zero tests executed anywhere |
| "Proof" is one channel | Advisor consult at Gate 2 | The cells holding the *strongest* proof would have been marked unproven |
| Both sides of the new contract are tested | Semantic judge (1 of 7 checks failed) | A rename on either side leaves both suites green and the door dead |
| The seam fix crossed both readers | Advisor consult during the revision | Half-closed seam re-capped as closed |
| A dropped unit of work is inert | bee's own close-door, refusing at runtime | Dropping the only test unit could pass a feature clean |
| The bypass rows cannot pass vacuously | Advisor consult | Every pre-existing bypass row in the file shares the weakness |
| Doctrine states surviving refusals correctly | Independent text review | Ten overstatements; the default path defers four of them |

Seven advisor consults ran. **Zero returned clean** — every one produced at
least one adopted correction. The layered net worked: each mechanism caught a
class the others structurally could not.

## Orchestrator mistakes, named

1. **Dispatched a worker to a cell that could never become ready.** Answering a
   NEEDS_REVISION verdict by spawning a *dependent* repair cell — but the verdict
   reopens the parent, so the dependent's dep is permanently uncapped. One full
   worker dispatch produced zero code. A NEEDS_REVISION is absorbed by the
   reopened cell itself.
2. **Restated a code matrix from memory in a locked decision.** "red-first covers
   lane high-risk, all classes" was never true — the tier function leaves two
   classes at suite-green *before* the lane is consulted. The phrase propagated
   into `plan.md` and into a worker's dispatch verbatim. A worker caught it, not
   a reviewer.
3. **Hand-composed dispatch briefs from three sources with no consistency
   check.** The plan's slice queue and the cell records disagree on which cell
   owns the loosening (the plan still carries the swap, frozen). One cell told a
   worker main owns the verify while the lane's doctrine told it to run tests red
   itself. One advisor brief named a model without its tier and was refused by
   the model guard.
4. **Listed a refusal site count wrong in the cell action** — the exclusivity
   checks are inside the pending branch, not ungated. The worker and its advisor
   both caught it and wrote the narrower truth.

## Reusable patterns

- **Audit every reader before adopting a field as a signal.** Grep all read
  sites; a flag with N readers grants N behaviours you did not ask for.
- **A guard loosened without a negative control cannot be proven to still
  exist.** The pair ships in the same unit of work: one table proving the widened
  path passes, one proving every heavier tier still refuses.
- **A contract written by one unit and read by another owes one row crossing the
  seam with nothing hand-written**, plus a mirror so it cannot pass vacuously.
- **A test asserting behaviour under a configuration must assert the
  configuration is live** before exercising it — otherwise a silently-failed seed
  makes both halves of every pair pass.
- **A doctrine exemption the machine's predicate does not encode is a trap.**
  The planning doctrine says the highest-risk lane is never batched into a
  trailing test unit; the predicate has no such exemption and refuses the close.
  Doctrine now says so; the predicate was deliberately not changed.
- **Every status in a lifecycle enum needs a row at every door that reads it.**
  A withdrawn unit was read two contradictory ways at once because no test ever
  exercised that status.
- **Documentation about refusals must cite the guard condition, not the throw
  line.** Four surviving refusals are deferred by the *default* path; calling
  them unconditional misled every worker taking the default.
- **A locked decision describing a code matrix must quote the function and its
  line range** rather than paraphrasing it.

## What worked, and is worth keeping

The coverage-judgement rule ran live for the first time in this feature. It
found four of five parts of its story already covered, **wrote zero duplicate
rows**, and closed the one genuine gap — a door that had never been asked the
bypass question. That is the mechanism doing exactly what it was built for, on
its first run, on itself.

The close was also honest about what it does not fix: the completion helper
never validates recorded output content, and the feature-boundary record only
hashes a file it never reads. The new marker catches laziness, never deception.
Both are named as accepted residual risk rather than papered over.

## Open gaps carried forward

- Units completed before the marker existed carry none, so the coverage door
  cannot see them.
- The behaviour-change warning and the marker key on different things, so a unit
  can complete warned but unmarked.
- A workspace declaring only its impacted-test command as "none" takes an
  automatic waiver and completes unmarked.
- The doctrine/predicate disagreement on batching at the highest-risk lane.
