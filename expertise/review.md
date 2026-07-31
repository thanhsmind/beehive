# How to Review

Review exists to catch defects before they ship and to keep the codebase
honest. It is not commentary, not appreciation, and not a second design
pass. A review that produces one verified blocker is worth more than one
that produces thirty observations.

## Where to look

| Situation / goal | Entry |
|---|---|
| Deciding whether an observation is filable | What a finding is |
| Assigning blocker / major / minor | Severity calibration |
| Tempted to inflate, or torn between two levels | Severity is a spent signal |
| Reading a diff for defects | Adversarial reading |
| About to file a suspected defect | Verify before reporting |
| Cannot fully verify a finding | Label uncertainty exactly |
| Writing the finding up | Evidence standards |
| Found a bug outside the requested scope | Scope discipline |
| Tempted to file formatting or naming notes | Style versus substance |
| No fresh reviewer available | Reviewing your own work |

## What a finding is

A finding is a claim about a defect, backed by a concrete failure
scenario: these inputs, in this state, produce this wrong outcome. If you
cannot state the scenario, you do not have a finding yet — you have a
suspicion, and suspicions are your own work queue, not the author's.

"This looks wrong" is not a finding. "This might race" is not a finding.
Compare:

> Weak: "The cache invalidation here seems fragile."
>
> Finding: "If `update()` is called between the read at line 41 and the
> write at line 48, the stale value is written back and the update is
> silently lost. Repro: two concurrent calls with the same key."

**The no-questions test:** could the author, reading only what you wrote,
reproduce the failure or trace the broken path without asking you a
single question? If not, keep working before you file it.

**A finding also names what wrong means.** "Returns null" is only a defect
if the caller expects non-null; say which caller, and what breaks there.
Half of all disputed findings dissolve once the reviewer states the
expected behavior explicitly — sometimes because the reviewer discovers
the expectation was theirs alone.

## Severity calibration

Use three levels, and mean them:

- **Blocker** — merging this ships a defect or a data risk. Wrong
  results, corruption, data loss, a security hole, a broken contract
  that callers rely on. Blockers stop the merge; that is their whole
  meaning.
- **Major** — wrong under realistic conditions the change should
  handle: a plausible input class, a concurrent caller, a failure of a
  dependency it invokes. The happy path works; the realistic path does
  not.
- **Minor** — works, but costs the next reader or maintainer: a
  misleading name, a missing edge-case test, duplication that will
  drift, an error message that will send the debugger the wrong way.

Calibrate against consequence, not against how much the code offends
you. An ugly but correct function is minor. A tidy function that drops
an error on the floor is major or worse.

## Severity is a spent signal

Inflating severity destroys the channel. The first time an author
investigates a "blocker" and finds a style nit, every future blocker
from you gets triaged with suspicion — including the real one. Severity
is a signal you spend; spend it accurately. When genuinely torn between
two levels → pick the lower one and state the condition under which it
would be the higher: "minor, but major if this endpoint is exposed
publicly — I could not confirm which."

## Adversarial reading

Review against the requirement, not against the diff's own story. A diff
is a narrative written by its author; if you follow it line by line you
will check that the author did what they meant to do — which they almost
always did. The defects live in what the diff does not say.

Concretely:

- Start from the requirement or the bug report, not from the code. Write
  down what a correct change must handle. Then read the diff and check
  the list, not the diff.
- Ask what the change does NOT handle. Empty input, the largest input,
  the input that arrives twice, the call that fails halfway through, the
  caller that ignores the return value. Absence never shows up in a
  diff; you have to bring the checklist.
- Hunt the cases the author was least likely to have run. Authors test
  the path they built. If the change is about parsing, feed it the
  malformed case; if it is about retries, kill the dependency mid-retry;
  if it touches time, cross a boundary — midnight, month-end, a DST
  shift.
- Check the edges of the diff, not just its middle. The lines just
  outside the changed region — the caller that now receives a new value,
  the cleanup that assumed the old shape — are where changes break
  things without touching them.

The author's tests tell you what the author worried about. Your job is
the worry they did not have.

## Verify before reporting

Reproduce or trace every suspected defect before you file it. Run the
failing input if you can; if you cannot run it, walk the exact code path
by hand, line by line, and confirm each step of the failure actually
follows.

Plausible-but-wrong findings are expensive in a way that is easy to
underrate. Each one forces the author to do your verification for you,
burns their trust, and buries the real findings in the same report. A
review with three verified defects outranks one with three verified
defects and seven maybes — the maybes cost the three their credibility.

## Label uncertainty exactly

Uncertainty is allowed; unlabeled uncertainty is not. When you cannot
fully verify → say exactly what you established and what you did not:

> "Verified: `parse()` returns `undefined` for empty string (ran it).
> Unverified: whether any live caller passes empty string — I found no
> guard, but did not trace all call sites. Confidence: medium."

That report is useful. "This probably breaks on empty string" is not.

## Evidence standards

**Every finding cites file and line.** Not "the retry logic", but
`src/net/retry.ts:112`. A finding the author has to search for starts
its life as an argument.

**Quote the actual behavior, never a paraphrase.** Paraphrase is where
reviewer errors hide: you summarize what you think the code does, the
author reads the summary, and you argue about the summary instead of
the code. Paste the two lines in question, or the actual output, and
let them speak.

**Sketch the fix** — one or two lines is enough: "guard the empty case
before the loop", "take the lock across both operations". You do not
have to be right, and the author is free to fix it differently. The
point is diagnostic: a finding whose fix you cannot even sketch is
usually a finding you do not yet understand. Trying to sketch it is the
cheapest way to discover that — before the author does.

## Scope discipline

Review what was asked. If the request was "review the pagination
change", the verdict covers the pagination change — not the module's
architecture, not the neighboring function you happened to read.

When you spot a genuine bug in adjacent code → keep it, but keep it
apart. It goes in a separate section — "Out of scope, noted for
follow-up" — and it never counts against the verdict. Folding it in has
two failure modes: the change under review gets blocked for sins it did
not commit, and pre-existing defects get mislabeled as regressions,
which corrupts anyone later asking "when did this break?"

The verdict answers one question: is the change under review safe and
correct for what it claims to do? Everything else is a different
document.

## Style versus substance

**Follow the codebase's conventions, not yours.** Mechanical preferences
— formatting, naming patterns, import order, comment style — defer to
what the surrounding code does; check before filing a style note. If the
codebase writes it the author's way, there is no finding; if the
codebase is split, it is a convention question for the team, not a
review note for this author.

**Never trade a correctness finding for ten formatting notes.** A review
dominated by nits does damage twice: the author's attention budget is
finite and you just spent it on trivia, and the one major finding in the
pile reads as nit number eleven. If you have a major finding, lead with
it, and ask whether the nits are worth sending at all.

**Design disagreements are a middle category:** substantive, but not
defects. "I would have used a queue here" is not a finding unless you
can name what the current approach gets wrong — a scenario, a cost, a
maintenance trap. If you can, file it at honest severity. If you cannot,
it is a conversation, not a review note.

## Reviewing your own work

Self-review is real review, but the checklist inverts. A fresh reviewer
hunts your blind spots by accident; you have to hunt them on purpose,
because your blind spots are precisely the assumptions you made while
writing.

**Attack the assumptions, not the code.** While building, you decided
the input is always sorted, the config is always present, the callback
runs at most once. Those decisions are invisible in the diff and
invisible to you — they feel like facts. Write down every "always" and
"never" you relied on, then go verify each one against the actual
callers.

Immediate self-review mostly re-runs the mental model that produced the
bug; you re-read what you meant, not what you wrote. Two things break
that loop:

- **Distance.** Review after a gap — hours if you can get them, or at
  minimum after context-switching to something unrelated. The model
  fades; the text remains; the gap between them is where your bugs are.
- **A different lens.** Read the diff in a different order than you
  wrote it: bottom-up, or caller-first, or tests-first asking "what do
  these tests fail to pin down?" Any order but the one in your head.

When fresh eyes are available, prefer them for anything that matters —
not because they are smarter, but because they do not share your
assumptions. When they are not available, distance plus an inverted
reading order is the honest substitute, and "I reviewed it myself,
immediately, once" should be reported as what it is: barely reviewed.
