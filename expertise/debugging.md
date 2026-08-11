# How to debug

## Where to look

| Situation / goal | Entry |
|---|---|
| A bug report just arrived | Reproduce first |
| No red-capable command exists yet to trigger it on demand | Build the feedback loop first |
| The failing input is large or noisy | Minimize the repro |
| An error or stack trace is on screen | Read the error before forming theories |
| You know where it crashed, not why | Crash site versus fault site |
| About to change code to "fix" it | State the hypothesis before the fix |
| The cause is unclear | Instrument before guessing |
| The search space is large | Bisection: three axes |
| The bug looks impossible | Environment versus code |
| It passes alone and fails in the suite | When it only fails in company |
| It only fails where you cannot attach | Debugging what you cannot touch |
| The symptom looks like one you've seen before | The familiarity trap |
| The symptom just vanished | The fix is not done at "it works" |
| Wrapping up the fix | Cleanup |

## Reproduce first

A bug you cannot trigger is a bug you cannot verify fixed. Before reading
a line of implementation, get the failure happening on demand: the exact
command, the exact input, the exact output. If a report says "sometimes
fails on save," your first deliverable is a way to make it fail on
purpose — not a theory about why it might.

**The repro is also your finish line.** "I changed something and the
report stopped coming in" is not a verified fix; "the repro failed before
the change and passes after it" is.

## Build the feedback loop first

Reproduce first gets the failure happening once. This step turns that
one-off into a command: something you can run over and over, that goes
red on this bug and green once it's fixed. Before any hypothesis work,
build that command — bisection, instrumentation, and hypothesis-testing
all just consume it; without it, none of them converge.

Rank the ways to build one and pick the cheapest that still reaches the
bug:

1. A **failing test** at whatever seam the bug reaches — unit,
   integration, end-to-end.
2. A **direct HTTP call** — curl or a small client script — against a
   running server.
3. A **CLI run diffed against a fixture** — invoke the tool with a known
   input and compare its output to a saved known-good snapshot.
4. **Replaying a captured trace** — save the real request, payload, or
   event log to disk, then replay it through the code path in isolation.
5. A **throwaway harness** — the smallest slice of the system, one
   process with mocked dependencies, that reaches the bug in one call.
6. A **property or fuzz loop** — run hundreds of random inputs and watch
   for the failure shape, when the bug is "sometimes wrong" rather than
   "wrong on this one input."
7. A **bisection harness** — script "boot at state X, check, report" so
   `git bisect run` can drive it unattended.
8. A **differential run** — the same input through the old version and
   the new one (or two configs), diffing the two outputs.
9. A **human-in-the-loop script**, last resort, only when one step
   genuinely needs a human hand — the script still drives every step
   around that click, so the loop stays structured instead of becoming a
   manual retest.

Judge whatever you build against four bars: **red-capable** (it can
actually go red on *this* bug, not merely "sometimes errors"),
**deterministic** (same verdict every run), **fast** (seconds, not
minutes), and **runnable by you alone**, unattended.

**Hard gate: no red-capable command, no hypothesis work.** If you catch
yourself reading code to build a theory before this command exists,
stop — that is the exact failure this step prevents. "State the
hypothesis before the fix" and "Instrument before guessing" both assume
the loop already exists; they consume it, they do not replace it. If you
genuinely cannot build one after working down the ladder, say so
explicitly — do not quietly start guessing instead.

**Flaky bugs get a rate, not a repro.** When the failure is
non-deterministic, the target is not one clean trigger but a
reproduction rate high enough to debug against: run the loop many times,
add stress, narrow timing windows, and raise the rate until it's usable.
A loop that fails 1% of the time is not a loop yet; one that fails 50%
of the time is.

**Tag every temporary probe.** Give every log line, print, or breakpoint
added for this investigation one unique prefix, e.g. `[DEBUG-a4f2]` — so
cleanup (see "Cleanup" below) becomes a single grep for that prefix
instead of a manual re-read of the diff.

Example: a report says "the export sometimes drops rows." A failing
test can't reach it — rows only drop under load. Ladder rungs 1–4 don't
fit; rung 6 does: a property loop running the exporter against 500
randomly-sized inputs and asserting `output.length == input.length`
catches the drop about 8% of the time — not debuggable yet. Adding
concurrency to the loop (run the export while another job writes the
same table) raises the rate to 60%. That command is now the loop: run
it, watch it go red, and test every hypothesis against it in seconds.

## Minimize the repro

Then minimize. Strip the reproduction to the smallest input and shortest
path that still fails: delete half the input and re-run; if it still
fails, delete half again. A 3-line repro is worth an hour of shrinking,
because every element that remains is now evidence — if removing a field
makes the bug vanish, that field is implicated. A minimized repro often
names the culprit before you open the code.

## Read the error before forming theories

The error message is the cheapest evidence you will ever get, and it is
routinely skimmed. Read all of it, literally. "Cannot read property 'id'
of undefined" says an object is missing — the question is which object and
why, not anything about `id`. "ENOENT: ./config/app.yaml" names the exact
path it tried; compare it character by character with the path you think
it should be using.

In the stack trace, **find the deepest frame that is your code** — the
frames inside libraries usually mark where your bad value was finally
noticed, not where it was made. And in a stream of output, **scroll to
the first failure**: errors cascade, and the twelfth message is usually
debris from the first. Fixing the last error on screen is treating a
symptom of a symptom.

## Crash site versus fault site

The crash is where the invalid state was detected; the fault is where it
was created, often far earlier — a null returned three calls up, a config
misread at startup, a cache poisoned yesterday. When you know where it
crashed → trace the bad value backward to its origin, and fix it there.
A null-check at the crash site silences the alarm and leaves the fault
standing.

## State the hypothesis before the fix

Before changing code, say — out loud, in a comment, in your notes — what
you believe and what it predicts:

    "The total is wrong because discounts apply after tax instead of
    before, so a taxed $100 order with a 10% discount shows $99 instead
    of $97.20. Reordering the two steps should make the repro show $97.20."

That shape — *X because Y, so changing Z should produce W* — is the whole
discipline. It forces a mechanism (Y), a specific change (Z), and a
falsifiable prediction (W). If the prediction fails, the hypothesis is
dead and you have learned something precise. A change you cannot phrase
this way is not a fix; it is a guess, and a guess that happens to make the
symptom vanish is the most dangerous outcome available — the fault is
still there and you have stopped looking.

## Instrument before guessing

When the cause is unclear → resist the speculative edit — the "maybe it's
this" change made in hope. Speculative edits mutate the crime scene: after
three of them you no longer know whether the behavior you see is the bug,
your edits, or an interaction between them.

Instead, add observation aimed at a question:

- **Targeted logging**: print the suspect value at the boundary where you
  believe it goes wrong — with enough context (which iteration, which id)
  to interpret it. One precise log line beats twenty scattered ones.
- **Assertions**: assert the invariant you believe holds ("this list is
  sorted here", "this id is never empty") at the point you believe it. A
  firing assertion converts belief into evidence and moves the detected
  fault earlier, closer to its origin.
- **Input probing**: vary one aspect of the input at a time and watch what
  the failure does.

**One experiment, one hypothesis.** Each experiment has a predicted
outcome before you run it. If you change two things and the behavior
shifts, you have learned almost nothing — you cannot attribute the shift.
Slow, single steps converge; broad flailing loops.

## Bisection: three axes

When the search space is large, cut it in half rather than inspecting it
end to end. Bisection works along three axes; pick the one whose halves
are cheapest to test.

- **Over history — which change?** If it worked before and fails now, the
  bug entered in a specific change. Binary-search the revision history
  (most version control systems automate this): test the midpoint, keep
  the failing half, repeat. Thirty changes need five tests. The guilty
  change is a spotlight — the fault is in that diff.
- **Over input — which part?** Feed half the failing input; if it still
  fails, halve again; if it passes, the trigger is in the other half.
  This is the minimization from "Minimize the repro," used as a search.
- **Over the code path — which stage?** In a pipeline, check the
  intermediate value at the midpoint: correct there means the fault is
  downstream; wrong means upstream. Repeat inside the failing half until
  you are staring at the one stage where good data goes in and bad data
  comes out.

## Environment versus code

Before deep-diving the logic, spend two minutes on the boring facts,
because a large share of "impossible" bugs live there:

- Which version of the runtime, the dependencies, the tool is actually
  running — not which one you assume? Is the build you are running the
  build that contains your change?
- What is the working directory? Relative paths resolve against it, not
  against the file that mentions them.
- Permissions, environment variables, locale, and platform differences:
  path separators, case-sensitive versus case-insensitive filesystems,
  line endings, whether symlinks can be created at all. "Works on my
  machine" is usually one of these wearing a mask.

## When it only fails in company

When something passes alone and fails as part of a larger run — a test in
its suite, a job in its pipeline, a request under concurrency → the
subject of the investigation is no longer that unit. It is the state
shared between it and whatever ran before: a global, a cached
connection, a file on disk, a stubbed function never restored, a clock
that was frozen and not thawed, a record left in a store.

Two facts make this tractable. **Order is the input.** Run the same
members in a different order and the failure moves or vanishes, which
already proves the fault is in shared state rather than in the failing
unit. And **the search bisects**: run the first half of the preceding
members plus the failing one, then the half that still reproduces it,
until one predecessor remains. That predecessor — the polluter — is where
the fix goes, not the unit that reported the failure.

Two traps. When a run is randomly ordered, record and re-apply the seed
or the ordering, or you cannot reproduce what you just saw. And when the
bisection finds nothing, suspect the environment the run itself creates —
parallel workers sharing a directory, a port, or a database — because
then the failing pair is not "before and after" but "at the same time."

The durable fix is never to reorder the members permanently. It is to
give each one its own world (`tests.md`, "Every test owns its world"), so
ordering stops being an input at all.

## Debugging what you cannot touch

When a failure happens only where you cannot attach — a build machine,
another environment, a customer's installation → the method inverts.
Locally you form a hypothesis and run an experiment; here every run is
expensive and possibly unrepeatable, so you harvest evidence first and
form the hypothesis from what you already have.

Read what the environment already recorded, in this order: the exact
command and the exact arguments it ran; the versions of everything
involved; the full output, from the first failure rather than the last;
the timestamps around it; and whatever the surrounding system logged in
the same window. Correlate by identifier where one exists and by time
where it does not — the entries from the same moment in a different
component are usually the missing half of the story.

Then treat the *difference* as the suspect, not the code. Something is
true there and false here (`Environment versus code` lists the usual
candidates), and the fastest path is usually to reproduce the
environment rather than the failure — the same versions, the same
ordering, the same emptiness of a cache, the same absence of a file you
happen to have locally.

When a further run is unavoidable, make it count: add the instrumentation
that answers the specific question you cannot answer from the record, and
make the failure *keep* its evidence — the artifact, the log, the state
directory — because a failure that cleans up after itself teaches you
nothing and you may not get another one. And when a re-run passes with no
change, that is not a fix; it is a report that the failure is
intermittent, which is a different investigation with the same rules.

## The familiarity trap

When a symptom matches a remembered failure → treat the match as a
hypothesis, not a diagnosis. "Last time this timeout meant the proxy
config" — the same symptom has many causes. Verify that the evidence in
front of you supports *this* failure being that cause (does the log show
the proxy path at all?) before applying last time's fix. Familiarity is
where experienced debuggers lose the most time: they skip the verification
step precisely because they have seen it before.

## The fix is not done at "it works"

"It works now" is the beginning of the end, not the end. Before closing,
answer three questions:

- **Why did it break?** Name the mechanism, all the way down to the fault
  site. If you cannot explain why the old code failed, you do not know
  whether you fixed it or perturbed it — timing-sensitive and
  state-dependent bugs routinely "go away" under any change at all, then
  return.
- **Why does this fix address that cause?** The change should act at the
  fault site, not muffle the crash site. Wrapping the failure in a
  try/catch, adding a retry, or padding a timeout treats the symptom;
  say explicitly why your change removes the mechanism instead.
- **What now prevents regression?** Usually a test that reproduces the
  original failure and fails without the fix. Sometimes it is an
  assertion that makes the invalid state impossible to construct, or a
  type that rules it out. If the answer is "nothing," the bug is on a
  round trip.

**Then look sideways once:** does the same mechanism exist elsewhere? A
bug found in one call site of a pattern usually has siblings in the
others.

## Cleanup

Debugging leaves debris: print statements, commented-out blocks, hardcoded
test values, loosened timeouts, disabled checks, temporary files. Before
finishing, remove all of it — diff your working tree against where you
started and justify every remaining line as part of the fix. If every
temporary probe carried its `[DEBUG-xxxx]` prefix ("Build the feedback
loop first"), start with a single grep for that prefix — it should
return nothing — then confirm the diff read catches anything untagged.
A hardcoded
"user-123" or a check disabled "just while testing" that ships is not
residue; it is the next bug, planted by the person best positioned to
prevent it.

**Two kinds of instrumentation may stay, deliberately:** an assertion
that documents a real invariant, and a log line at a boundary you will
want visibility into next time — kept at an appropriate log level, with
the context to be useful. Keep those on purpose, with intent, not because
deleting them was forgotten. Everything else goes.
