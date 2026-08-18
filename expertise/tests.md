# How to test

## Where to look

| Situation / goal | Entry |
|---|---|
| Judging whether a test is worth writing | Confidence, not coverage |
| About to write a test for a change | The coverage audit |
| Deciding what a test may assert | Test behavior, not structure |
| Choosing where an expected value comes from | The independent-oracle rule |
| Naming a test, or it needs "and" in its name | One behavior, named after it |
| Choosing unit vs integration vs end-to-end | Pick the cheapest level that can fail |
| Two systems must agree on a shared interface | Pick the cheapest level that can fail |
| Replacing a dependency in a test | Fakes over stubs over mocks |
| Setting up fixtures and shared state | Every test owns its world |
| A test touches time, network, randomness, or ordering | The four determinism leaks |
| A test fails intermittently | Flaky is worse than missing |
| Picking which cases to cover | Choosing cases |
| The cases multiply and every one looks the same | Properties, when examples multiply |
| Fixing a reported bug | Red before green |
| The code under test is hard to reach | Test the real code, never the twin |
| Tempted to test everything | What not to test |
| The suite is slow, or a case needs a human to judge it | Fast, automated, unattended |
| About to run the suite | Running tests |
| A failure's cause is not obvious from the output | Instrument before guessing |
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

```rust
// Good — asserts the contract: invalid input is refused, and the refusal
// names what the caller must fix.
let err = claim_cell(&root, "missing-id").unwrap_err();
assert_eq!(err.reason, "CELL_NOT_FOUND");

// Bad — asserts the choreography. Passes only while claim_cell happens to
// call validate() exactly once, and breaks on any honest refactor.
assert_eq!(validate_calls.get(), 1);
```

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

## The independent-oracle rule

An expected value must come from a source the implementation cannot
influence — a known literal, a hand-worked example, the written spec.
Never recompute the expected value the same way the code under test
computes it: a test whose assertion mirrors the implementation's own
arithmetic passes by construction, because a bug shared by both sides
cancels itself out. When the code and the test derive the answer
through the same steps, the test cannot catch a mistake in those steps
— it can only catch a disagreement between the code and itself, which
never happens. Name this the **tautological test**: it moves the
coverage number without buying any confidence.

```rust
// Bad — the expected value is recomputed the way the code computes it.
// A bug in the summing logic ships inside both sides, and the test
// stays green no matter what the bug does.
let items = vec![Item { price: 10 }, Item { price: 5 }];
let expected: u32 = items.iter().map(|i| i.price).sum();
assert_eq!(calculate_total(&items), expected);

// Good — the expected value is an independent, known literal.
let items = vec![Item { price: 10 }, Item { price: 5 }];
assert_eq!(calculate_total(&items), 15);
```

## One behavior, named after it

A test name states the scenario and the expected outcome, so a red line in
the output explains itself without opening the file. If the name needs
"and," it is two tests.

```rust
// Vague — a red line here sends you to the source to learn what broke.
fn test_claim() { … }

// Clear — the failure explains itself.
fn claiming_a_cell_held_by_another_session_is_refused_with_its_holder() { … }
```

Follow arrange-act-assert: build the world, take the action, assert the
outcome. When setup grows long, lift it into a helper so the body stays the
scenario; when several tests build near-identical worlds, give the helper
defaults and let each test override the one field it cares about.

**Readable beats dry.** Tests are read far more often than written, and
they are read under pressure — a red line at the end of a long run. A
reader must be able to tell what a test asserts from the test body alone,
without chasing a shared fixture, a base class, or a helper three files
away. So a helper earns its place only when it *shortens* the scenario;
the moment factoring out duplication hides which value makes this case
different, keep the duplication. The rule that makes production code
better makes tests worse: deduplicating tests couples them, and coupled
tests fail together and get rewritten together.

## Pick the cheapest level that can fail

Before TDD on a new surface, confirm one question first: what is the
public interface, and which seams do we test? Agree the seams up front,
then test only at those seams — a test written before the interface is
settled ends up pinned to whatever shape the first draft happened to
take.

Match the level to the failure you are trying to catch, and prefer the
cheapest level that can actually catch it:

- **Unit** — one component, no I/O. Fast enough to run constantly; the right
  home for logic, edge cases, and algorithms.
- **Integration** — components against real collaborators (a real temp
  filesystem, a real store, a real lock). Slower, and usually the best
  confidence per unit of cost, because it catches the bugs that live in the
  seams where each piece is individually correct.
- **End-to-end** — the shipped entry point driven the way a user drives it
  (spawn the CLI, feed it stdin, read stdout and the exit code). Highest
  confidence, slowest, and the most sensitive to environment; keep the set
  small and load-bearing.
- **Contract** — the agreement between two independently-changed sides of a
  boundary: an API and its callers, a writer and its readers, a plugin host
  and its plugins. Each side tests itself against the same recorded contract,
  so a change that breaks the other side fails at home instead of in
  someone else's suite. Reach for it only where the sides genuinely ship
  apart; inside one deployable, an integration test already covers the seam.

No level substitutes for another. A suite of only unit tests never sees the
seams; a suite of only end-to-end tests is slow, and its failures point at
"something in the pipeline."

## Fakes over stubs over mocks

A test double replaces a dependency. The three kinds differ in what they
know:

- **Fake** — a real, working implementation with a shortcut: an in-memory
  store, a temp directory standing in for a repo. It executes logic, so it
  still catches malformed input.
- **Stub** — returns canned answers, knows nothing else.
- **Mock** — records calls so the test can assert which ones happened.

Prefer the real dependency when it is fast and deterministic — a temp
directory usually is. Otherwise prefer a fake. Reserve mocks for when the
interaction *is* the behavior under test (a notification was sent, a
payment was charged); otherwise they pin structure and break on refactors.

**A double is called with whatever the production code chooses to send.**
If your double branches on the shape of its input — argv, env, cwd, stdin —
guard each branch and exit cleanly on an unrecognized shape. A double that
falls through to a catch-all branch produces side effects in the wrong
context, and the resulting failure surfaces two layers away from the cause.

**If a test needs many doubles, the code under test has too many
dependencies.** Fix the design, not the test.

## Every test owns its world

Every test creates its own fixtures, and it cleans up after itself — or
better, writes into a per-test temporary location that needs no cleanup.
No test may depend on another test having run first, on shared mutable
state, or on leftovers from a previous run. A suite whose tests pass in
order but fail when filtered to one test is broken, even while it is green.

**Cleanup must survive the failing path.** A teardown written as the last
lines of the test body never runs on the run that matters — the one where
an assertion fired three lines earlier. Put cleanup where the runner
guarantees it (a teardown hook, a scope guard, a defer), or sidestep it
entirely with a per-test temp directory the OS reclaims. Otherwise the
first real failure leaves state behind, and the next run reports a
cascade whose first red is the only true one.

## The four determinism leaks

Determinism means the test's outcome depends only on the code under test.
The usual leaks and their fixes:

- **Clock**: never compare against `now()` from inside an assertion; inject
  the time or freeze it.
- **Network**: never touch a real endpoint; fake the boundary.
- **Randomness**: seed it or inject it.
- **Ordering**: never assert on the iteration order of an unordered
  collection; sort before comparing.

When output legitimately carries a varying token — a duration, a generated
id, a timestamp — do not weaken the assertion to a substring match.
Normalize the varying token and keep comparing everything else exactly; see
the differential-testing pattern.

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

## Properties, when examples multiply

An example test names one input and one expected output. A property names
a rule that must hold for *every* input: encode-then-decode returns the
original, sorting twice equals sorting once, a merge of two valid stores
is a valid store, the parser never panics on arbitrary bytes. When you
find yourself writing a sixth example that differs only in its numbers,
you are hand-sampling a property — state the property instead and let a
generator sample it.

```rust
// Example — one sample, and it says nothing about the next input.
assert_eq!(decode(&encode("héllo")), "héllo");

// Property — the rule itself, checked against generated inputs, and it
// shrinks a failure down to the smallest input that still breaks.
proptest!(|(s: String)| assert_eq!(decode(&encode(&s)), s));
```

Reach for a property when the input space is large and the rule is short:
round-trips, invariants preserved across an operation, ordering and
idempotence, equivalence between a fast path and a reference path (see the
differential-testing pattern). Stay with examples when the interesting
inputs are few and named — the boundary values, the three error cases —
because a generator will spend its budget on inputs you already know are
uninteresting.

**A property test still owes you a seed.** A generated failure that cannot
be replayed is a rumor. Record the failing case as its own example test the
moment the generator finds one: the generator's job is discovery, the
example's job is regression.

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

**The reachability ladder**, in order — take the first rung that works,
and record why when you skip one:

1. **Restructure.** Lift the logic into a unit with its own contract.
   Usually the code is hard to reach because one function does three
   jobs, and splitting it is the change you wanted anyway.
2. **Widen visibility deliberately.** Making a function crate-visible or
   package-private so its own tests can reach it is a fair trade, and a
   smaller one than the twin you were about to write.
3. **Test from higher up.** If the behavior is observable one level out,
   assert it there. Slower and coarser, but it exercises shipped code.
4. **Duplicate — last, and in writing.** If nothing else reaches it and
   the logic is load-bearing, a copy in the test is better than no test
   at all, provided the test says so in a comment: what it duplicates,
   why the real code is unreachable, and that it cannot detect drift in
   the original.

Adjust the code to fit the test, never the test to avoid the code.

**A passing test is not proof the real path ran.** When the system has a
fallback — a delegate, a retry, a cached answer — a test can pass because
the fallback answered while the code under test was never reached. If a
fallback exists, prove it stayed out of the way; see the
proving-the-code-under-test-ran pattern.

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

**When no new test is owed at all** — a change with no new behavior:
doc-only edits, mechanical renames, compiler-enforced type updates,
dead-code removal, a pure refactor already pinned by existing tests.
State the reason when you skip. The boundary that never qualifies:
"hard to test" is not "no behavior" — async paths, integration surfaces,
and error handling have behavior, and difficulty setting them up is an
argument for a better harness, never for skipping the test.

## Fast, automated, unattended

A suite is only worth what it is worth *running*. Two things decide that,
and both are properties of the suite, not of any one test.

**Slow suites stop being run.** Past the point where you hesitate before
running it, you start batching changes, and a red no longer names which
change caused it — the whole feedback loop the suite was built for is
gone. When a single test is slow, suspect the design before the test: code
that needs a live service, a full migration, or a heavy fixture to
exercise one branch is telling you the logic wants to come out where it
can be reached directly. Fixing that is a production change, and it is the
right one.

**Every test judges itself.** No manual setup step, no "then look at the
screenshot," no case that passes when a human squints at the output. A
test that needs a person to rule on it does not run in CI, does not run on
a teammate's machine, and in practice does not run. If the output is
genuinely visual or generative, assert the part that is machine-checkable
— the invariant, the shape, the diff against a reviewed snapshot — and
say plainly what remains unasserted.

## Running tests

**The agent owns test scope, and proves it on the cap.** `bee test`
executes the recorded `commands.test` and writes the record when you run
it, but no door runs it for you: pick the proof your change type needs
(code → related tests green; docs → parity/pointer checks; behavior →
judge verdict), run it yourself, and record a cap proof line
`<command> — <result> — <scope reason>`. `bee cells finish`, `bee close`,
and `bee worktree merge` check that recorded proof; they run nothing.
Green caps the work; a red result refuses the cap — never build on a red
base. A scoped-green proof whose CI later goes red is a fix-first cell
plus a captured learning on why the scope missed.

**Save the output to a file, then read the file.**

```bash
cargo test --release > /tmp/test.out 2>&1
```

A long run's failing case scrolls past, and a filtered pipe truncates
exactly the context you need. The saved file is the evidence: grep it,
cite its path in a summary or handoff, and reopen it later instead of
paying for a rerun.

**Read the run you already have before buying another.** Most runners
keep per-case stdout and stderr on disk after the run — look for "test
output" or "log directory" in the runner's docs, and read the failing
case's log directly. Re-running a suite to see what a test printed pays
full wall-clock for output that is already sitting in a file. If the
runner discards per-case output by default, turn that on once; it is
cheaper than every rerun it saves.

**Use a runner with per-test isolation and real parallelism.** Isolation
puts each test in its own process so env vars, temp state, and in-process
singletons cannot leak between cases; parallelism keeps the suite fast
enough that people actually run it. For the Rust engine, `cargo nextest
run` gives both. If a test only passes when the suite is serialized, that
test shares state it should own — fix the test rather than serializing
everyone else.

**Run the whole suite after a cross-cutting change.** When a change touches
something several components read — a shared schema, generated output, a
path convention, a store format — the suite nearest the edited file stays
green while a sibling suite quietly accumulates the breakage. Pay the
wall-clock cost when the change shape warrants it.

**An environment limit is not a failure.** When the host genuinely cannot
run a case — no symlink privilege, no shell, no network — the case skips
loudly and names the missing capability; it never fails, and it never
silently disappears. See the environment-limited-tests pattern.

## Instrument before guessing

When a failure's cause is not obvious from its output → add one or two
targeted prints on the suspect path, including inside any test double, run
once, and read the new evidence. One instrumented run beats three
hypothetical fixes, each of which changes two things and teaches you
nothing. Remove the instrumentation once the cause is understood.

If reading the output requires a debugger, the test is too coarse: it
exercises so much that its red line cannot point anywhere. When several
tests fail at once, start at the leaf — the innermost failure is usually
the cause, and the rest are its shadow.

## Every escaped bug is a missing test

When production breaks despite green tests → the incident has identified
the missing test for you. Before or with the fix, write the test that
would have caught it — reproduce the production failure at the smallest
size, watch it fail, then fix. Then ask the second question: what made
this scenario invisible to the suite? An untested error path, an
over-mocked boundary that hid the real interaction, an environment
difference the tests never exercise. **Close the class of gap, not just
the instance**, or the same hole ships the next bug too.

## Related guides

Neighboring disciplines, each loaded only when its trigger applies —
never alongside this file by default.

- [debugging.md](debugging.md) — read when a red is not a missing test but
  an unexplained defect: the repro is unstable, the error is being
  misread, or a fix "works" without a cause.
- [review.md](review.md) — read when judging someone else's tests rather
  than writing your own: what makes a finding about test quality worth
  filing, and what evidence it owes.

## Patterns

Reusable testing patterns live in `tests/patterns/` as individual files.
Each line below carries its load trigger — read a pattern only when its
trigger applies, never the whole directory. When a narrower theme
accumulates enough material to route on its own, give it a
`tests/<sub-topic>/` directory and a `tests/<sub-topic>.md` file carrying
its own `## Patterns` index — the index stays shallow that way, and a
reader lands on the sub-topic instead of scanning a flat list.

- [differential-testing](tests/patterns/differential-testing.md) — read when
  a second implementation must match an existing one: a port, a rewrite, a
  cache beside its source of truth, or a fast path beside a slow one.
- [proving-the-code-under-test-ran](tests/patterns/proving-the-code-under-test-ran.md)
  — read when the system has a fallback, delegate, retry, or cache that
  could satisfy the assertion while the code under test never ran.
- [environment-limited-tests](tests/patterns/environment-limited-tests.md) —
  read when a case needs a capability the host may not have: symlinks, a
  POSIX shell, elevated privilege, a network, a specific filesystem.
- [pinning-against-a-live-oracle](tests/patterns/pinning-against-a-live-oracle.md)
  — read when reimplementing behavior owned by something else — a platform
  API, a collation order, a serializer — and the expectations would
  otherwise be guesses.
- [asserting-on-generated-output](tests/patterns/asserting-on-generated-output.md)
  — read when the subject renders a template: prompts, codegen, reports,
  formatted messages.
- [comparing-structured-data](tests/patterns/comparing-structured-data.md) —
  read when asserting on a JSON/JSONL/YAML artifact, or that such a file was
  left unchanged.
