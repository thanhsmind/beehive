---
type: bee.pattern
title: An exit code several paths produce cannot prove a side effect did not happen
description: "A test whose subject is that something must NOT happen has to assert the absence itself — the returned decision, or a counted side effect — because an exit code that both the refusal and the thing-you-feared return keeps the suite green while the side effect runs."
timestamp: 2026-09-20
bee:
  id: pattern-20260920-an-exit-code-several-paths-produce-cannot-prove-a-side-effect-did-not-happen
  lifecycle: active
  areas: [bee-herding]
  sources: ["docs/history/herding-preflight-test-seam/CONTEXT.md", "cell hpts-1 (commit 94c1b62bf)", "the defect it fixes: cell ihr-2, same day"]
  polarity: pitfall
  critical: true
---

# An exit code several paths produce cannot prove a side effect did not happen

Three tests were written to prove that a dispatch **proceeds** rather than
short-circuits. Each one called the real entry point and asserted
`ExitCode::FAILURE`, on the reasoning that a bare temp directory has no
transport configured, so proceeding must fail.

It did fail. It also, first, really split a pane and really started a real
billed agent, handing it a brief inside a `tempdir` the test dropped one line
later. The agent read nothing, sat idle, and nothing ever closed it. Twelve
leaked sessions accumulated in a single day while the suite reported green
every time, because `FAILURE` is what **both** paths return: the refusal the
author expected, and the spawn-then-fail they did not.

## Why the assertion could not catch it

The test's real subject was an absence — *no process is started*. The assertion
it used was an exit code, and an exit code is a funnel: many different
histories arrive at the same value. An assertion on a funnel can only tell you
that some path reached it, never which one. The side effect happened strictly
before the value the test looked at, and nothing in the value carried a trace
of it.

Worse, the wrong reasoning was written into the test as a comment — *"it fails
at transport setup rather than returning early"* — so it read as verified when
it had only been assumed. A comment asserting a mechanism the test does not
check is the disguise this pitfall wears.

## What to do instead

- **Assert the absence directly.** Split deciding from acting: have the code
  return a decision value, and let the test assert the decision. A function
  that decides and does not act cannot leak while being tested.
- **When the effect is outside the process, count it.** The proof that closed
  this defect was not a test at all — it was a before/after count of session
  files and live panes across two full suite runs. Some absences are simply
  not assertable from inside the suite, and saying so is honest.
- **Belt the neighbours.** Sibling tests that short-circuit *today* are one
  regression away from the same leak. Give their fixture an actively hostile
  configuration — here, an illegal transport in the test's own temp root — so
  a future fall-through refuses instead of spawning.

## Where it bites hardest

Anywhere a guard sits above an expensive or irreversible action: a spawn, a
payment, a delete, an outbound send. The guard's happy path and its refusal
often share a return value, and the test that "covers" the guard is then
asserting the one thing both sides agree on.

## Related

- [[pattern-20260812-a-guard-and-its-tests-are-one-model]] — a guard and its
  tests sharing one model prove only that the model agrees with itself; this
  is the same blindness expressed through a return value rather than a shared
  implementation.
- [[pattern-20260710-a-non-exposure-invariant-needs-a-test-on]] — a
  "never emit X" invariant needs a probe on every surface it crosses; a
  "never spawn" invariant needs one on the effect, not the exit code.
- [[bee-herding-the-run-verb-and-worker-outcomes]] — the pre-flight whose
  decision is now a pure function precisely so it can be asserted.
