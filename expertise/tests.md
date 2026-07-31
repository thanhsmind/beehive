# How to test

## Where to look

| Situation / goal | Entry |
|---|---|
| Judging whether a test is worth writing | Confidence, not coverage |
| About to write a test for a change | The coverage audit |
| Deciding what a test may assert | Test behavior, not structure |
| Setting up fixtures and shared state | Every test owns its world |
| A test touches time, network, randomness, or ordering | The four determinism leaks |
| A test fails intermittently | Flaky is worse than missing |
| Picking which cases to cover | Choosing cases |
| Fixing a reported bug | Red before green |
| The code under test is hard to reach | Test the real code, never the twin |
| Tempted to test everything | What not to test |
| A bug shipped despite a green suite | Every escaped bug is a missing test |

## Confidence, not coverage

A test exists to buy confidence to change code. Each test encodes one
scenario and one expectation: "given this input and this state, the system
does that." When the suite is green, you should be able to refactor, extend,
or delete with the assurance that every encoded scenario still holds. A test
that does not buy confidence — because it duplicates another test, asserts
nothing meaningful, or fails randomly — is not neutral. It costs maintenance
and erodes trust in the suite. Judge every test you write by the confidence
it adds, not by the coverage number it moves.

## The coverage audit

Before authoring any test → never write it from the change alone. First
find what already covers the behavior you touched, and cite it concretely —
by file and by case name:

    Existing coverage for parseRange:
    - range.test: "parses open-ended upper bound"
    - range.test: "rejects reversed bounds"
    Gap: no case for a single-point range (min == max).

Then write only the gap. Duplicated coverage is not "extra safety"; it is
the same scenario paying two maintenance bills, and when the behavior
legitimately changes, every duplicate breaks at once and someone has to
decide which assertions still express intent.

**"No new test needed" needs the citation.** It is a legitimate verdict —
but only when backed by the citation above. "It's probably covered" is not
a verdict; it is a skipped audit. If you cannot name the file and case that
cover the scenario, the scenario is not covered.

## Test behavior, not structure

Test what the code observably does — its return values, its emitted events,
its persisted effects — not how it is internally arranged. **The refactor
litmus:** a refactor that preserves behavior must not break tests. If
renaming a private function, inlining a helper, or reordering internal
calls turns the suite red, the tests were pinned to structure, and they now
punish exactly the improvements they were supposed to enable.

The common ways structure-coupling creeps in:

- **Asserting call counts and call order on your own code.** Checking that
  `save()` called `validate()` exactly once tests the implementation's
  choreography, not its contract. Assert the outcome instead: invalid input
  was rejected, valid input was persisted.
- **Testing private functions directly.** If a private is complex enough to
  demand its own tests, that is a signal it wants to be a module with a
  public contract. Otherwise, test it through the public surface that uses
  it.
- **Over-mocking your own code.** Mocking your own collaborators replaces
  the real interaction with your assumption about it; the test then
  verifies the assumption, not the system. Mock at genuine boundaries —
  network, clock, filesystem, third-party services — and let your own
  objects talk to each other for real.

**The exception — when the algorithm *is* the deliverable:** a sort must
be stable, a cache must evict least-recently-used, a retry must back off
exponentially — those properties are the observable behavior, and asserting
them is correct even though they describe "how." Assert the property (the
output order, the eviction victim, the delay sequence), still not the
private call graph.

## Every test owns its world

Every test creates its own fixtures, and it cleans up after itself — or
better, writes into a per-test temporary location that needs no cleanup.
No test may depend on another test having run first, on shared mutable
state, or on leftovers from a previous run. A suite whose tests pass in
order but fail when filtered to one test is broken, even while it is green.

## The four determinism leaks

Determinism means the test's outcome depends only on the code under test.
The usual leaks and their fixes:

- **Clock**: never compare against `now()` from inside an assertion; inject
  the time or freeze it.
- **Network**: never touch a real endpoint; fake the boundary.
- **Randomness**: seed it or inject it.
- **Ordering**: never assert on the iteration order of an unordered
  collection; sort before comparing.

## Flaky is worse than missing

A flaky test is worse than a missing test: a missing test is a known blind
spot, while a flaky test trains everyone to ignore red — and an ignored red
is how real regressions ship. When a test flakes → stop and fix the
nondeterminism now, or delete the test and record the lost coverage as a
gap. Never retry-until-green, and never leave it flaking "for later."

## Choosing cases

For each behavior, cover three kinds of case, each at the smallest input
that demonstrates it:

- **The happy path**: the representative, intended use.
- **The edges**: empty input, one element, the boundary value itself and
  its neighbors (a limit of 10 needs 9, 10, and 11 — not 3 and 500),
  duplicates, already-sorted, unicode where text is parsed.
- **The error paths**: invalid input, missing dependencies, the failure
  modes the code claims to handle. Assert what the caller observes — the
  error type and message contract — not merely "it throws."

**Smallest demonstrating size** matters: a bug in pagination shows up with
three items and a page size of two. A 500-item fixture proves nothing more
and hides the intent of the case.

**Compose rather than multiply.** When behavior varies along independent
dimensions — say, three formats and four locales — do not write twelve
tests. Test each dimension's variants where the other is held fixed, then
add one test proving the dimensions are wired together. Combinatorial
suites look thorough but bury the one interaction case that actually
matters under repetition no one reads.

## Red before green

When fixing a reported bug → start by reproducing the bug as a failing
test. Write the test, run it, and watch it fail for the reported reason —
the same wrong value, the same error — before touching the fix. A test
written after the fix, that has never failed, proves nothing about the
bug: it may be asserting the wrong thing, exercising the wrong path, or
passing vacuously. The observed red is the evidence that this test detects
this bug; the subsequent green is the evidence the fix works.

**Green only beside fresh output.** The same honesty applies to reporting:
write "green," "passing," or "fixed" only next to actual command output
from a run you just performed. "Tests should pass now" is a prediction,
not a result. If you have not run it, say so.

## Test the real code, never the twin

Import and exercise the code that ships. Never test a copy of a function
pasted into the test file, a simplified reimplementation, or a "test
version" of the logic — those tests verify the twin, and the twin does not
run in production. The pull toward twins usually means the real code is
hard to reach: entangled with I/O, buried in a script without exports,
or constructed only inside a framework. Fix the reachability — extract the
logic, export the function, inject the dependency — and then test the real
thing. Improving testability this way is a legitimate production change,
not test scaffolding.

## What not to test

Do not spend tests on:

- **Framework and library behavior**: that the router routes, that the ORM
  saves, that a JSON parser parses. Their authors test that; your test
  would only re-verify your mock of them.
- **Third-party internals**: assert your integration — that you call the
  boundary with the right contract and handle its documented outcomes —
  not how the dependency behaves inside.
- **Trivial pass-throughs**: getters, setters, constructors that only
  assign, one-line delegations with no logic. A test that restates the
  line it tests can only fail when the line changes deliberately — it
  detects edits, not bugs.

The moment any of these grows a branch, a transformation, or a default, it
has logic, and logic gets a test.

## Every escaped bug is a missing test

When production breaks despite green tests → the incident has identified
the missing test for you. Before or with the fix, write the test that
would have caught it — reproduce the production failure at the smallest
size, watch it fail, then fix. Then ask the second question: what made
this scenario invisible to the suite? An untested error path, an
over-mocked boundary that hid the real interaction, an environment
difference the tests never exercise. **Close the class of gap, not just
the instance**, or the same hole ships the next bug too.
