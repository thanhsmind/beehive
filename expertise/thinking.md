# Thinking methods

Named methods for examining a problem before committing to an answer.
Each entry is a handle for your own routing: the move it names reaches
the person you work with in plain words, and the reasoning stays
inspectable because the move is visible — not because its name is
spoken. The default mode underneath all of them is Socratic
questioning — ask before asserting.

## Routing: goal → method

| Goal / situation | Method |
|---|---|
| Default mode for examining any idea | Socratic Questioning |
| The approach is justified by convention, not need | First Principles |
| The problem statement smells like a symptom | 5 Whys |
| A plan is confident but untested against failure | Inversion |
| About to commit to a plan; want a last check | Pre-Mortem |
| "This case is different" — optimism from specifics | Outside View |
| Confidence built on unchecked assumptions | WYSIATI |
| A solution arrived before the need was named | Jobs to Be Done |
| Solving before the problem is actually understood | Double Diamond |
| Special cases keep multiplying; every fix adds a branch | Simplification Cascade |
| The domain's own vocabulary has stopped producing options | Forced Analogy |
| An alternative is being dismissed too easily | Steelmanning |
| Two positions both have real merit | Dialectic |
| The immediate effect looks good; downstream unclear | Second-Order Thinking |
| A design is being judged only at today's size | Scale Test |
| Two explanations fit the evidence | Occam's Razor |
| About to remove something that looks pointless | Chesterton's Fence |
| Stuck, or an explanation only makes sense to its author | Rubber-Duck Explanation |

## Methods

### Socratic Questioning

When helping someone examine an idea — any idea, any time → ask
questions instead of making assertions. This is the default posture;
every other method here is a specialized question. Rotate through the
question families: clarification ("what exactly do you mean by X?"),
assumptions ("what has to be true for this to hold?"), evidence ("what
have we actually observed, versus inferred?"), alternative perspectives
("how would someone invested in the opposite outcome read this?"), and
consequences ("if this is right, what else must follow?"). The goal is
that the idea gets examined, not that a predetermined conclusion gets
reached — a leading question dressed as inquiry is an assertion with
worse manners.

### First Principles

When an approach is defended with "that's how it's done" or inherited
from a template, framework habit, or previous project → decompose it
into the facts it rests on and rebuild from only those that survive
scrutiny. List what the approach assumes: about the data, the users,
the constraints, the tools. For each assumption ask two things: how do
we know it holds here, and what becomes possible if it doesn't? Most
inherited designs carry constraints from a context that no longer
exists — a cache added for a load profile the system never sees, a
queue added for a scale that never came. What survives the audit is
the real requirement; design for that.

### 5 Whys

When the stated problem describes an effect rather than a mechanism —
"deploys are slow," "users are confused" → ask why, take the answer,
and ask why of the answer, until the next why has no useful answer.
Each hop must be grounded in something observed, not something
plausible: a guessed link in the chain poisons every hop after it.
Chains fork; when an answer has two causes, follow both. You have hit
bedrock when the answer is something you can change directly and the
symptom above it would demonstrably stop.

### Inversion

When a plan is stated only in terms of how it succeeds → flip it: ask
what would guarantee it fails. Failure modes are concrete and
enumerable in a way success paths are not — "the migration fails if
the old and new schema drift while both are live" is actionable;
"the migration succeeds if everything goes smoothly" is not. List the
guaranteed-failure conditions, then check the plan against each. The
same flip tests a claim: instead of collecting support for it, ask
what evidence would prove it false, and go look for that.

### Pre-Mortem

When commitment is imminent — the design is chosen, the work is about
to start → set the scene as accomplished fact: it is months later and
this effort failed badly; write the story of what went wrong. Stating
failure as history rather than possibility licenses people to voice
the doubts that forward planning politely suppresses. Every cause in
the story converts to a named risk, and each risk gets one of three
dispositions before work starts: mitigated, monitored, or accepted
out loud.

### Outside View

When an estimate or bet is argued entirely from the specifics of this
case — "our team is strong, this codebase is clean, this one will be
different" → ask what class of thing this is and how that class
usually turns out. Migrations of this size, rewrites of this scope,
integrations with this kind of vendor: find the base rate, from the
repo's own history or from the record of similar efforts elsewhere.
Anchor on the base rate first, then adjust modestly for what is
genuinely distinguishing. The inside story is vivid; the reference
class is predictive.

### WYSIATI

When confidence is high and the supporting facts are few — a
conclusion built smoothly from whatever happened to be visible → name
what is missing before trusting what is present. The mind builds a
coherent story from available information and does not flag the gaps;
coherence feels like completeness. Ask: what would we need to know to
check this, and have we checked it? Then — and this is the working
half of the method — when the missing fact is fetchable, fetch it:
read the code path instead of assuming its behavior, search for the
library's actual contract, run the command and look. Research beats
debate wherever the fact exists.

### Jobs to Be Done

When a feature or tool is proposed before anyone has said what
someone is trying to get done → ask what job this would be hired for.
Probe the moment of need: when does the urge for this arise, what
triggers it, and what does the person do today when it strikes? Cover
the functional job (the task), but also the emotional one (what it
lets them stop worrying about) and the social one (what it signals).
If no one can describe the hiring moment, the proposal is a solution
in search of a problem — park it until the job is found.

### Double Diamond

When the framing already contains the answer — the request arrives as
"add a retry button" rather than "operations fail and users are
stranded" → run two diamonds, and refuse to enter the second early.
First diverge on the problem: gather how it actually shows up, for
whom, how often. Converge on a precise problem statement. Only then
diverge on solutions — several genuinely different shapes, not one
shape with variants — and converge on the best. Most wasted builds
are competent solutions to an unexamined problem; the first diamond
is where that waste is prevented.

### Simplification Cascade

When the design keeps growing special cases — a fourth branch for a
fourth variant, a flag to make the flag work → stop adding and look for
the one statement that would make several of them unnecessary at once.
The prompt is literally that: *if X were true, what could we delete?*
Then test whether X can be made true.

The tell that a cascade is available is repetition with variation: five
handlers whose bodies differ in two lines, a switch whose arms are the
same shape, a config matrix where most combinations are never used. Each
of those is a general case wearing several disguises, and finding the
general case removes the branches rather than organizing them. Sequence
matters — one insight usually unlocks the next, so re-ask the question
after each deletion instead of stopping at the first win. Where no such
statement exists, the variation is real and the branches are honest; that
is a finding too, and it ends the search rather than justifying a
premature abstraction.

### Forced Analogy

When the domain's own vocabulary has stopped producing options — every
idea is a variation of the current one → deliberately borrow a structure
from an unrelated field and ask what it would mean here. *What if we
treated this queue like a ledger? this cache like a lease? this config
like a migration? this permission like a reservation?*

Most transfers fail, and failing is cheap; the value is in the few that
carry a whole solved problem with them, including its known failure
modes. A borrowed structure arrives with vocabulary, invariants, and a
literature of what goes wrong — which is exactly what a genuinely novel
design lacks. Keep the borrowing honest: name the property that makes the
analogy hold, and the point where it stops holding. An analogy carried
past its breaking point is how a system ends up with a metaphor in its
type names and a different behavior underneath.

### Steelmanning

When an alternative is being waved off, a critique deflected, or an
option eliminated in one sentence → reconstruct the rejected position
in its strongest form before ruling on it. Supply its best evidence,
repair its weak phrasing, argue it as its most thoughtful proponent
would — then confirm: "the strongest version is X; is that fair?"
Critique only after the yes. Beating the weak version proves nothing
and forfeits whatever the strong version had to teach; often the
steelman contains a piece worth absorbing even when the position as
a whole still loses.

### Dialectic

When two positions each hold real merit and the debate has become
picking a winner → run thesis, antithesis, synthesis. State the first
position at full strength; state the genuine opposite at full
strength — not a strawman of the first; then look for the frame in
which the conflict dissolves. Synthesis is not splitting the
difference: "cache aggressively" versus "always serve fresh data"
does not resolve to "cache half the time" but to "classify what can
be stale and cache exactly that." The question shifts from "who is
right?" to "what view takes both seriously?"

### Second-Order Thinking

When a decision is judged only by its immediate effect → ask "and
then what?" at least twice. First-order effects are visible and
usually favorable — that is why the option is on the table. The
second order is where the cost lives: the quick fix that becomes the
pattern others copy, the flexible option that becomes the config
matrix nobody can test, the deadline saved by skipping the test that
teaches the team tests are skippable. Trace who reacts to the change,
how they adapt, and what their adaptation causes. A choice that wins
the first order and loses the second is a loss.

### Scale Test

When a design is being judged only at the size it has today → run it at
the extremes and see what breaks. A thousand times more: what runs out
first — memory, connections, a single writer, someone's patience, a
per-item call that was invisible at ten? A thousand times less: what
disappears entirely, and is the machinery still justified when it does?
Then the time axis: instantaneous, and a year long. What has to be
resumable, what has to be idempotent, what silently assumes it finishes
before anything else changes?

The extremes are diagnostic because they convert vague unease into a
named limit — "this holds until roughly one writer per partition" is
usable where "should scale fine" is not. The method finds two distinct
things: the component that fails first, which is the real capacity of the
design, and the machinery that only exists for a scale you do not have,
which is the part to delete. Note that this is a *design* question asked
before building; the measured version of it belongs to profiling, and
guessing here is not a substitute for measuring there.

### Occam's Razor

When multiple explanations fit the observed facts → prefer the one
requiring the fewest new assumptions, and test it first. The bug is
more likely a typo in the config than a race in the runtime; the
missing data more likely a filter than a corruption. This is a search
strategy, not a law of nature — the simple explanation is where to
start because it is cheapest to check, not guaranteed truth. Rule it
out with evidence before paying for the exotic theory, and let the
complex explanation win only after the simple one has demonstrably
failed.

### Chesterton's Fence

When something looks pointless and the urge is to delete it — a
guard clause with no obvious trigger, a sleep in a script, a config
flag nobody recognizes → find out why it was put there before taking
it down. Search the history, the linked issue, the test that fails
without it. Two outcomes, both good: the reason is obsolete and the
removal proceeds with confidence, or the reason is alive and a
production incident was just declined. "I don't see why this is
here" is the argument for investigating, never the argument for
removing. If the reason is genuinely unrecoverable, remove it as an
experiment with a watch on what breaks — not as a cleanup.

### Rubber-Duck Explanation

When you are stuck, or when an explanation works only for people who
already understand it → explain the thing from scratch, aloud or in
writing, to an audience that knows nothing. Every step must earn its
place: "and then it just works" is the tell that a step was skipped,
and the skipped step is where the wrong assumption lives. The method
works because explanation forces serialization — beliefs held vaguely
in parallel must be stated one at a time, in order, and the broken
one becomes visible the moment it has to be said plainly. The duck
never has to answer; the explanation itself does the debugging.

## Choosing and combining

Apply one method at a time, and phrase the move in plain words — "let
me check what we're assuming," "before we commit, let's write the story
of how this failed" — keeping the method's name as your internal
handle. Name it explicitly only when the person asks how you are
reasoning, or needs a handle to push back on the lens rather than the
conclusion. Methods chain: 5 Whys finds the root cause, Double Diamond
explores solutions to it, Pre-Mortem stress-tests the chosen one.
They do not stack in a single breath — running three lenses at once
produces mush from all of them. When two methods point the same way,
that is signal; when they conflict, the conflict itself is the
finding — surface it rather than silently picking the convenient
answer.
