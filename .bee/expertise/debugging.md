# How to debug

Contents

- Reproduce first
- Read the error before forming theories
- State the hypothesis before the fix
- Instrument before guessing
- Bisection: three axes
- Environment versus code
- The fix is not done at "it works"
- Cleanup

## Reproduce first

A bug you cannot trigger is a bug you cannot verify fixed. Before reading
a line of implementation, get the failure happening on demand: the exact
command, the exact input, the exact output. If a report says "sometimes
fails on save," your first deliverable is a way to make it fail on
purpose — not a theory about why it might.

Then minimize. Strip the reproduction to the smallest input and shortest
path that still fails: delete half the input and re-run; if it still
fails, delete half again. A 3-line repro is worth an hour of shrinking,
because every element that remains is now evidence — if removing a field
makes the bug vanish, that field is implicated. A minimized repro often
names the culprit before you open the code.

The repro is also your finish line. "I changed something and the report
stopped coming in" is not a verified fix; "the repro failed before the
change and passes after it" is.

## Read the error before forming theories

The error message is the cheapest evidence you will ever get, and it is
routinely skimmed. Read all of it, literally. "Cannot read property 'id'
of undefined" says an object is missing — the question is which object and
why, not anything about `id`. "ENOENT: ./config/app.yaml" names the exact
path it tried; compare it character by character with the path you think
it should be using.

In the stack trace, find the deepest frame that is your code — the frames
inside libraries usually mark where your bad value was finally noticed,
not where it was made. And in a stream of output, scroll to the first
failure: errors cascade, and the twelfth message is usually debris from
the first. Fixing the last error on screen is treating a symptom of a
symptom.

Distinguish the crash site from the fault site. The crash is where the
invalid state was detected; the fault is where it was created, often far
earlier — a null returned three calls up, a config misread at startup, a
cache poisoned yesterday. Trace the bad value backward to its origin, and
fix it there. A null-check at the crash site silences the alarm and leaves
the fault standing.

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

When the cause is unclear, resist the speculative edit — the "maybe it's
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

Each experiment tests exactly one hypothesis and has a predicted outcome
before you run it. If you change two things and the behavior shifts, you
have learned almost nothing — you cannot attribute the shift. Slow, single
steps converge; broad flailing loops.

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
  This is the minimization from "Reproduce first," used as a search.
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

A related trap: pattern-matching a symptom to a remembered failure. "Last
time this timeout meant the proxy config" is a hypothesis, not a
diagnosis — the same symptom has many causes. Verify that the evidence in
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

Then look sideways once: does the same mechanism exist elsewhere? A bug
found in one call site of a pattern usually has siblings in the others.

## Cleanup

Debugging leaves debris: print statements, commented-out blocks, hardcoded
test values, loosened timeouts, disabled checks, temporary files. Before
finishing, remove all of it — diff your working tree against where you
started and justify every remaining line as part of the fix. A hardcoded
"user-123" or a check disabled "just while testing" that ships is not
residue; it is the next bug, planted by the person best positioned to
prevent it.

Two kinds of instrumentation may stay, deliberately: an assertion that
documents a real invariant, and a log line at a boundary you will want
visibility into next time — kept at an appropriate log level, with the
context to be useful. Keep those on purpose, with intent, not because
deleting them was forgotten. Everything else goes.
