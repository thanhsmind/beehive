# herding-orchestration — learnings (captured 2026-08-19)

One effort turned the herding cockpit from a bash-and-pane-counting
arrangement into a coordination core with a readable memory: a generic `fleet`
crate carrying the five-phase choreography, a herdr backend behind a trait, a
`bee herding wave` entry point, an append-only wave ledger, an occupancy read
that fails closed, and a native control loop replacing `control-loop.sh`.
Cells ho-1..ho-16. Decisions D1-D19 (`docs/history/herding-orchestration/CONTEXT.md`).

## What the port taught

- **A proof rewritten from a changed CLI's `--help` is not a proof.** The
  spawn line was dead on herdr 0.8.0 (`agent start` had stopped taking `--cwd`
  and now refuses to create layout), and the repair was only half the story.
  Running it for real found two behaviors no documentation states: starting an
  agent has no do-not-focus option and moves the owner's view, and a worker's
  idle/done status tracks that pane's own focus rather than the work. Both are
  now spec'd in `docs/knowledge/areas/bee-herding/overview.md` under Edge Cases.
- **The distilled source was a choreography, not a set of primitives.** The
  first read judged the external Python/bash redundant against herdr's native
  verbs. The second read (D3) found that the ORDERING carried eight properties
  no send/wait pair provides, and that its 29 test scenarios were the real
  specification. Taking the ordering and none of the code was the whole design.
- **Genericness needed a compiler, not a promise.** The crate edge (D5) is what
  keeps bee vocabulary out of the core. It has two enforcement mechanisms, not
  one — cargo's own cycle check owns normal dependencies, the manifest test
  owns dev/build/target-conditional ones — and that only surfaced because the
  red-first run could not be made to fail the obvious way.

## What nine mutation rounds taught

The dominant defect of this feature was not a bug class, it was a
**test-coverage shape**: two things are each tested and the JOIN between them
is not. Nine instances in one feature, every one invisible to reading and to
ordinary test-writing, every one found by mutating the join. The most
expensive detail is the one worth carrying forward: a dispatch prompt that
enumerated eight prior instances with their measured consequences produced a
ninth on the exact path it named. **The lesson does not transfer as prose.**
The counter-practices that did work are mechanical — mutate the join rather
than the parts; enter the test where production enters (a crossing test one
layer above the production boundary let a two-argument swap survive 1908 green
tests); and refuse a test double that takes what it exists to observe as an
underscore-prefixed parameter or records a field nothing reads.

## What running two sessions taught

- **Inside a worktree, the tracked copy of a runtime store answers stale.** Two
  judges within one hour read the branch's frozen `.bee/decisions.jsonl`; the
  second issued a false NEEDS_REVISION saying three decisions had no record.
  They existed the whole time. Any "does this recorded thing exist" check must
  resolve the control root explicitly.
- **Logging a decision and locking it are two different acts.** The false
  finding was pointing at a true defect: the decisions were in the store and
  never added to CONTEXT.md's locked table, which is the record downstream
  cells actually cite.
- **A guard that offers two remedies invites the one the caller may not use.**
  Three workers met the same judge-debt gate at cap time with identical
  instructions; two handed the decision up, the third used the orchestrator's
  `--override-judge` on its own cell. The refusal text names both remedies side
  by side without saying which belongs to whom.

## What the owner settled at the end

D19 narrows D1's closing condition to Linux. The MECHANISM is proven on
Windows — the suite runs unexcluded on a Windows CI lane, and the herdr backend
was deliberately split into pure interpretation and thin process glue for
exactly that reason. What is not proven there is a LIVE run, which needs a
running herdr server, real panes and real agents. That becomes an owner-run
supervised cycle (R7), not an agent-run step. Recording it as a supersession
mattered because D4 had been written as a hard requirement and two cells were
scoped against it.

## Promotion judgment

Five patterns promoted (`docs/knowledge/patterns/`, 2026-08-19): the untested
join between tested parts; the stale tracked store inside a worktree; a refusal
that names a remedy the caller may not use; a guard that cannot be made to fail
having a co-owner; a locked decision collapsing one side of an open question.
Three rows filed to the backlog rather than written as prose — the mechanical
test-double check, the cap-refusal wording, and the manifest test's literal
name match — because the untested-join evidence says a check is worth more than
another paragraph.
