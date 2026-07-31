# How to Make and Record Decisions

A codebase is the residue of thousands of decisions, and the code records
only the outcomes — never the boundaries, the rejected alternatives, or
the reasons. A decision record is the small, durable artifact that keeps
a settled question settled. This guide covers which decisions deserve one,
how to write one, and how to keep the record honest as the work moves.

## Where to look

| Situation / goal | Entry |
|---|---|
| Unsure whether a decision deserves a record | The recurrence test |
| A discussion produced several commitments | The granularity test |
| A decision just landed in conversation | Record at the moment of settlement |
| Writing the record itself | Anatomy of a good record |
| Recognizing a record that carries no weight | Anti-patterns |
| Downstream work meets a settled question | Locked versus open |
| A locked decision no longer fits the budget or reality | Reinterpretation is quiet narrowing |
| Changing a recorded decision | Superseding, not editing |
| Writing a change that exists to honor a decision | Cite the decision at the work |
| Referencing decisions in specs and guides | Prose states the rule; records hold the lineage |
| Tempted to record everything | When not to record |

## The recurrence test

Record a decision when it settles an ambiguity that someone — a teammate,
a reviewer, a future implementer, you in six weeks — will hit again. That
is the whole test. If the question will recur and the code alone cannot
answer it, the answer needs a home outside anyone's head.

The recurring questions are recognizable by shape: *which of two valid
approaches did we commit to* (soft delete vs hard delete), *where is the
line* (what counts as personally identifying data in our logs), *what did
we deliberately not do* (no retry on webhook delivery — the consumer
dedupes), *what does this word mean here* ("account" is the billing
entity, not the login). Each of these looks obvious for about a week
after it is settled, and then it is a debate again.

**The rejected alternative is often the most valuable part.** Code shows
what was built; only the record shows that the other way was considered
and why it lost — which is exactly what stops the next person from
"fixing" the design back to the rejected option.

## The granularity test

One record names one choice and the boundary it draws. Not a meeting's
worth of narrative, not a design document, not three choices that
happened to be made the same afternoon.

The test: could someone cite this record to justify or reject a specific
change? "Timestamps are stored in UTC; conversion happens at render" —
citable. A reviewer can point at it and say "this diff violates it."
"We discussed the timestamp situation and aligned on an approach" — not
citable, because it does not say what the approach *is*. A record that
cannot settle a disagreement on its own has recorded the meeting, not
the decision.

**Never bundle.** If a discussion produced three commitments, that is
three records. Bundled records cannot be superseded independently: when
one of the three changes later, the bundle becomes half-true, and
half-true records are worse than none because they still get cited.

## Record at the moment of settlement

The trigger is audible: when you hear or say "okay, so we'll do X" →
capture it before the conversation moves on. That sentence *is* the
record, missing only its boundary and its why.

Write the record when the decision lands — in the same working session,
while the alternatives and the reason are still sharp — not in a cleanup
pass at the end. The end-of-work version is reconstructed from memory,
and reconstruction loses precisely the parts that mattered: the boundary
cases, the rejected option, the "and we explicitly do NOT do X."

The cost asymmetry is stark and it always points the same way. At the
moment of settlement, the record costs one line, because everything is
already loaded in your head. Six weeks later the same question costs the
whole debate again — the same arguments, reheated, often landing on a
*different* answer this time, which now silently disagrees with code
built on the first one. One line now, or an incoherent system later.

## Anatomy of a good record

Three parts, usually two to four lines total:

1. **The choice** — declarative, present tense, specific. "Retries use
   exponential backoff, capped at 5 attempts."
2. **The boundary** — where it applies and where it stops. "Applies to
   outbound webhooks only; internal queue consumers retry forever."
3. **One line of why** — the reason or the rejected alternative. "Capped
   because the downstream dedupes on id; infinite retry just delays the
   dead-letter alert."

The why line earns its keep the day someone wants to change the decision:
it tells them what has to still be true (or no longer true) for the
change to be safe. A choice without a why can only be obeyed or ignored —
it cannot be intelligently revisited.

Keep each part short enough that the record stays quotable. A record you
would paste whole into a review comment is the right size.

## Anti-patterns

- **The minutes entry.** "We discussed caching with the team and
  considered several options." No choice, no boundary, nothing citable.
  Delete it or finish it.
- **The decision buried in chat.** The choice was made, crisply, in a
  thread — and lives only there, unfindable in a month, invisible to
  anyone who was not in the room. Chat is where decisions happen; it is
  never where they are kept.
- **The code restatement.** "The function `parseConfig` parses the
  config." The code already says this, and says it more accurately.
  Records that restate code rot the instant the code changes and add
  noise to the records that carry real weight.
- **The aspirational record.** "We should eventually move to event
  sourcing." That is a wish, not a decision — nothing was settled, no
  boundary was drawn, nobody can act on it or violate it.

## Locked versus open

An open question can be argued; a locked decision can only be cited or
superseded. The distinction is the entire value of keeping records: work
downstream of a locked decision builds on it *without re-deriving it*,
which is what lets many hands work in parallel without converging on
different answers.

## Reinterpretation is quiet narrowing

When a locked decision turns out expensive → the words do not flex.
Locked means locked against reinterpretation too, and reinterpretation is
the subtle failure mode. "Support CSV export for all report types" does
not become "the three most common report types" because the full set
turned out to be expensive. That is not reading the decision — it is
quietly narrowing it to fit a budget, and it converts the record from a
commitment into a suggestion. If the decision as written no longer fits
reality, that is real information: take it to the decision's owner and
get an explicit change. The one move never available is honoring the
words while shaving the meaning.

## Superseding, not editing

Decisions change; records of them do not. When a locked decision is
revisited → the owner supersedes it: a new record states the new choice
and points at the old one, and the old record stays, marked superseded.

Never edit the old record in place. Work was built while it was in
force, and that work is only explicable with the record it was built
against. Erase the record and every past change that cited it becomes
unexplainable — a reviewer reading last quarter's diff finds it honoring
a rule that apparently never existed. The supersede chain is the history
of the system's mind changing, and that history is load-bearing.

**Superseding belongs to the decision's owner.** Anyone can propose it;
discovering mid-implementation that a decision is inconvenient is a
reason to raise it, never a license to override it.

## Cite the decision at the work

When a change exists to honor a decision → say so where the work is
described — in the plan item, the commit message, the PR description.
"Cap retries at 5 (per the webhook-retry decision)" turns a reviewer's
question from "why 5?" into a lookup. The citation completes a traceable
chain: requirement → decision → change, and the chain is what makes
review cheap. A reviewer who can trace every surprising choice to a
record reviews the diff; one who cannot must re-litigate the design.

The citation also protects the work itself. A change that visibly honors
a locked decision is defended by that decision; the same change without
the citation looks like an arbitrary preference, and arbitrary
preferences get "improved" away.

## Prose states the rule; records hold the lineage

Citations belong at the *work* — commits, plans, reviews — not woven
through long-lived prose. Documentation, guides, and specs should state
the rule in full and let the decision record hold the lineage.

The reason is mechanical: prose and records age at different rates. A
guide that says "timestamps are UTC (decision 14)" breaks twice — once
when decision 14 is superseded and the prose keeps pointing at the dead
record, and again for every reader who must chase the reference to learn
what the sentence already almost said. Inline decision numbers in prose
are pointers into a structure that reorganizes; they rot silently, and a
rotted citation is worse than none because it teaches readers to stop
trusting citations. Prose states the rule; the record, findable by
topic, holds who decided it and when.

## When not to record

Recording has a noise cost, and the cost compounds: every trivial record
dilutes the collection, and a diluted collection stops being read —
which silently defeats the important records too. Do not record:

- **Choices fully derivable from the code.** The function's behavior,
  the module layout, which library is imported. The code is the record,
  and it is always current.
- **Convention already written down elsewhere.** If the style guide says
  it, a decision record repeating it is a second copy that will drift.
- **Choices with no plausible second opinion.** Nobody will ever ask why
  the temp file is deleted after use. A record needs a question it
  answers; no question, no record.
- **Reversible micro-choices.** A variable name, a private helper's
  signature — anything the next person can change in five minutes
  without asking anyone. Locking these wastes the lock's authority.

The collection should read like a map of the system's genuinely contested
ground. If a stranger skimming it learns only trivia, the real decisions
are drowning; prune until every entry is one someone might actually cite.
