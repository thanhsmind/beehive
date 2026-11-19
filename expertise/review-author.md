# Review, from the author's side

This is the other half of `review.md`. That file is for the person who
reviews; this one is for the person whose work is reviewed — asking for a
review, answering what comes back, and reviewing your own change when no
fresh eyes are available. How a reviewer finds, grades and writes up a
finding lives in `review.md`.

## Where to look

| Situation / goal | Entry |
|---|---|
| Handing a change to a reviewer | Asking for a review |
| A finding has landed on your change | Receiving a finding |
| No fresh reviewer available | Reviewing your own work |

## Asking for a review

Review quality is bounded by what the reviewer is given. A diff arriving
with no context gets a diff-shaped review — the reviewer checks that the
code does what the code says, which is the one thing that is almost
always true.

Hand over four things. **What this must do**, in requirement terms, so
the reviewer can read against the requirement rather than against your
narrative. **What is in scope**, so out-of-scope observations stay
separate and the verdict stays meaningful. **Where you are least
confident** — the concurrency, the migration, the edge you could not
test — because directing attention at your own weak point buys more than
any amount of polish elsewhere. And **how to run it**: the command, the
fixture, the way to see it fail before your change.

Then make the diff readable: separate a mechanical rename from a
behavioral change rather than shipping both in one review, and say
plainly what you did not do and why. A reviewer who has to reverse-
engineer the intent spends their budget on comprehension and has nothing
left for defects.

## Receiving a finding

When a finding lands on your change → the first move is to reproduce it,
not to answer it. Reviewers are wrong regularly, and the only way to know
which case this is costs a minute: run the scenario. Replying from
memory of what you intended is how a real defect gets argued away by its
author, who is the person least able to see it.

Then respond in one of exactly three ways, each with what it requires:

- **Fixed** — the change is in, and the reply names how it was verified.
  "Good catch, fixed" without evidence is an assertion, and it is the
  most common place a fix that did not work gets closed.
- **Disputed** — with the reason the scenario cannot occur: the caller
  that guarantees the invariant, the constraint that rejects the input,
  the test that already covers it. A dispute is itself a claim and takes
  the same evidence standard as a finding; "that shouldn't happen" is not
  a dispute, it is the reviewer's point restated.
- **Accepted, not fixed here** — a legitimate answer when the finding is
  real but out of scope or below the bar for this change. It requires a
  named landing place, not "later." A "won't fix" with no record is a
  decision to carry the defect indefinitely, made silently.

Keep the exchange about the code. A finding is a claim about a program,
not about the person who wrote it, and treating it as such in both
directions is what keeps reviewers willing to file the uncomfortable one.
Where the disagreement is genuine and persists after both sides have run
the scenario, it is usually a hidden difference about the *requirement* —
surface that instead of trading fixes.

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
