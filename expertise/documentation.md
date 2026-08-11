# How to Document a System

Documentation is the system's contract written down. Its audience is a
competent stranger with a question and no access to your memory. Every
rule below follows from taking that stranger seriously.

## Where to look

| Situation / goal | Entry |
|---|---|
| Judging whether a spec is good enough | The rebuild test |
| Deciding where a sentence belongs | Separate what from how |
| A sentence leans on meetings, chat, or shared context | Write for the reader who wasn't there |
| Choosing tense and framing | Write in the timeless present |
| Behavior changed and a spec covers it | Currency discipline |
| A spec is wrong and cannot be fixed now | Stale docs are worse than no docs |
| A rule could be read two ways | Precision |
| Organizing or restructuring a document | Structure for lookup, not narrative |
| Tempted to write everything down | What not to document |
| Behavior is undecided, unmeasured, or unverified | Honest gaps |

## The rebuild test

A good spec passes one test: a competent stranger, given only the spec,
could rebuild the behavior on a different stack — different language,
different framework, different storage — and users could not tell the
difference.

That test forces the right content. It rules out code tours ("the
handler calls the service, which calls the repository") because a
rebuilder on another stack has no handler and no repository. It rules
in behavior and rules: what comes in, what goes out, what is guaranteed,
what is rejected, what happens on failure.

> Fails the test: "OrderValidator runs the checks in `checks/` and
> throws ValidationError on the first failure."
>
> Passes: "An order is rejected if any line quantity is zero or
> negative, if the total exceeds the customer's credit limit, or if the
> shipping country is not in the supported list. The first failing rule
> is reported; remaining rules are not evaluated."

The second version survives a rewrite. The first is dead the day
`checks/` is renamed.

When you finish a spec, run the test explicitly: pick a behavior, cover
the code, and ask whether the spec alone pins it down. Every question
the imagined rebuilder would have to ask you is a hole in the spec.

## Separate what from how

Behavior — what the system guarantees — changes when requirements
change: slowly, deliberately, visibly. Implementation — how it is
achieved today — changes whenever someone refactors: constantly and
silently. Mix them in one document and the fast-churning half rots the
slow half, because a reader who finds three stale implementation details
stops trusting the contract sentences too.

So keep them physically apart. The spec states the contract: "Sessions
expire after 30 minutes of inactivity; any authenticated request resets
the timer." Implementation notes, if worth writing at all, live in a
separate section or file clearly marked as a snapshot: "Currently
enforced via a TTL index; see the session store." When the refactor
lands, the note dies and the contract stands untouched.

**The sorting question**, for every sentence: would this sentence still
be true after a full rewrite that preserves behavior? If yes, it belongs
in the spec. If no, it is an implementation note — date it, fence it
off, or leave it out.

## Write for the reader who wasn't there

The reader was not in the meeting, has not read the chat, and arrived
six months after you left. Sentences that depend on shared context are
noise to them:

- "As discussed, we now debounce the save." — Discussed where? The
  reader has no "discussed".
- "The new approach avoids the race." — New relative to what? In a
  year, this is the old approach, and the sentence is a riddle.
- "Uses the standard retry policy." — Standard where? Link it or
  state it.

Every term in a spec is one of three things: defined in the document,
conventional in the field (a reader can look up "idempotent"), or
linked to its definition. Project-private vocabulary — internal names
for states, roles, phases — must be defined at first use, every
document, even when it feels repetitive to you. It is not repetitive to
the stranger, and the stranger is the audience.

## Write in the timeless present

Write "the service retries", not "the service will now retry" or "we
changed it to retry". The words "now", "new", "recently", and "changed"
in a spec are all the same bug: they timestamp the document instead of
describing the system.

## Currency discipline

A spec updated at the moment behavior changes stays alive. A spec
updated "later" is already legacy — "later" is a queue that only grows,
and by the time later comes, the person who knew what changed is gone.

The rule that works is brutal and simple: **the change and its spec
update are one unit of work**. The behavior change is not done until the
spec matches, the same way it is not done until the tests pass. Any
process that reviews changes should ask "does a spec cover this
behavior, and was it updated?" with the same weight as "are there
tests?"

## Stale docs are worse than no docs

No docs send the reader to the code, skeptical and careful. Stale docs
are trusted — that is their whole function — so the reader builds on the
documented 30-minute timeout while the system enforces 15, and the
resulting defect is one neither the code's author nor the doc's author
can see, because each half is locally correct. A wrong spec is a defect
with the blast radius of everyone who reads it.

When you find a spec you cannot afford to update → mark it dead at the
top — "OUTDATED as of <date>; the code is authoritative" — so it stops
collecting victims. Honest deletion beats confident rot.

## Precision

Vague specs delegate the actual decision to whoever reads them, and
different readers decide differently. State values, defaults, ranges,
units, and error behavior concretely:

- Not "retries a few times", but "retries 3 times with exponential
  backoff starting at 1s (1s, 2s, 4s), then fails the request with a
  503."
- Not "handles large uploads", but "accepts uploads to 100 MB;
  larger requests are rejected with 413 before the body is read."
- Not "recent items", but "items created in the last 30 days, by
  creation timestamp, UTC."

**Attach an example to every rule that could be read two ways.** Rules
state the general case; examples pin the edge that the wording left
open:

> Rule: "Usernames are case-insensitive and unique."
> Example: "`Alice` and `alice` are the same user; registering the
> second is rejected as a duplicate."

Without the example, a reader can honestly believe both are stored and
merely compared leniently. If you cannot construct a disambiguating
example for a rule, you have not yet decided what the rule means —
which is worth discovering while writing, not in production.

**Error behavior is part of the contract**, not an appendix. For every
operation: what happens on bad input, on timeout, on conflict, on
partial failure? "Returns an error" is not an answer; which error, what
the caller can do about it, and what state the system is left in are.

## Structure for lookup, not narrative

Nobody reads a spec front to back. A reader arrives with one question —
"what happens when the payment is declined?", "where are timezones
handled?" — and the document's job is to route them to the answer in
one jump.

That dictates the shape:

- Headings name behaviors and topics, not chapters of a story.
  "Session expiry", "Declined payments", "Timezone handling" — a reader
  scanning the table of contents should find their question by name.
  "Overview", "Details", and "Miscellaneous" route nobody.
- Uniform, enumerable rules go in tables, not prose. Status codes,
  configuration keys with defaults, state transitions, permission
  matrices — a table lets the reader index straight to their row and
  makes a missing row visible. Prose hides both.
- Each section stands alone. State its own preconditions or link them;
  a section that only makes sense after reading three earlier sections
  fails the reader who jumped straight to it — which is every reader.
- Answer first, qualification second. Lead the section with the rule;
  put exceptions and rationale after. The reader skimming for the rule
  should hit it in the first sentence.

If two readers with different questions would both start reading at the
top, the structure has failed. Write the three most likely questions a
reader arrives with, then check each is answerable from the headings
alone.

## Draw what is drawable

When a behavior has a shape — a lifecycle of states, a sequence of
actors, a containment of parts, a routing of cases — describe it with a
Mermaid diagram, always, as its own section beside the prose. "Can this
be drawn?" is a check you run on every behavior you document; when the
answer is yes, the diagram is not optional polish, it is a section of
the document with the same standing as a table:

- The diagram carries the same business vocabulary as the prose —
  node and edge labels use the pinned terms, never code identifiers
  (those stay in Pointers).
- Prose states the rule; the diagram shows the shape. Neither
  substitutes the other: a diagram with no prose leaves the rule
  ambiguous, prose with no diagram makes the reader rebuild the shape
  in their head — the exact work the document exists to save.
- Pick the form by the question it answers: `stateDiagram` for
  lifecycles, `sequenceDiagram` for who-talks-to-whom, `flowchart` for
  routing and containment. One diagram answers one question; two
  questions get two diagrams.
- A contradicted diagram is replaced in place, like a contradicted
  line — never left beside its correction, never allowed to rot while
  the prose moves on. The currency discipline above applies to edges
  and nodes exactly as it applies to sentences.

The cut test works in reverse here: if adding the diagram would not
save a rebuilder from getting a transition, an ordering, or an
ownership wrong, the behavior had no shape worth drawing — do not
draw a list as a picture.

## What not to document

Every sentence you write is a sentence someone must maintain and a
sentence that can rot. Omit:

- **Code restated line by line.** "The function opens the file, reads
  each line, and closes it" documents nothing the code does not say
  better, and goes stale on the next refactor. If the code needs
  narration to be understood, fix the code.
- **The document's own history.** Changelogs of the doc ("2024-03:
  reworded section 2"), authorship trails, and revision notes belong in
  version control, which records them for free and keeps them out of
  the reader's way.
- **The full debate behind a decision.** Record the outcome and the
  one-line reason; link the decision record for the alternatives and
  the argument. "Timeout is 30 minutes (compliance requirement; see
  decision record)" is complete. Three paragraphs on the options
  considered belongs in the decision record, written once, not
  re-summarized in every spec that touches it.
- **What is obvious from convention.** If the system does exactly what
  any practitioner would expect, silence is fine. Spend words where the
  system surprises.

**The cut test:** if this sentence disappeared, would a rebuilder get
the behavior wrong, or a maintainer make a worse decision? If neither,
delete it.

## Honest gaps

Every contract has an edge, and the spec's job includes saying where.
An explicit gap — "Concurrent edits to the same draft: behavior
unspecified; last write currently wins but this is not guaranteed" —
tells the reader exactly where the guarantees end and their own
verification must begin.

Silent omission tells them nothing, and readers fill silence with
assumptions — usually the assumption that the system behaves reasonably,
which is precisely where it does not. The undocumented edge is where
the defects already live; leaving it unmarked hides the one place the
reader most needs a warning.

Mark gaps in the spec's own vocabulary and make them searchable:

- "Unspecified: ordering of webhook delivery across topics."
- "TODO: measure — p99 latency under concurrent bulk import is
  unmeasured; the 200ms figure above covers single-writer load only."
- "Known limit: recovery after partial batch failure is manual; see
  the runbook."

A spec with ten visible gaps is a better contract than a seamless one,
because the seamless one has the same ten gaps and lies about it. The
measure of a spec is not that it covers everything — it is that the
reader always knows whether they are standing on covered ground.
