# How to Shape Work

Planning is not paperwork. It is the act of choosing a shape for the work —
its size, its order, its boundaries — before any of it exists. A good shape
makes the work almost implement itself; a bad one guarantees rework no
amount of careful coding can recover. This guide is about choosing well.

## Contents

- The smallest honest shape
- Reading risk before it reads you
- What makes a good unit of work
- Dependency thinking
- Plan the current slice; headline the rest
- The walking skeleton
- The smaller-path question
- Scope integrity: split, never shrink
- Plans are contracts
- The cold-pickup test

## The smallest honest shape

Match ceremony to real risk, not to how the request sounds. A one-line
config change described in an excited paragraph is still a one-line config
change. A "quick tweak" to session handling is not quick, no matter how it
was phrased. The request's tone carries zero information about its risk;
the touched surfaces carry all of it.

"Honest" is the load-bearing word. The smallest shape is not the one with
the least process — it is the smallest one that still covers what the work
actually endangers. Skipping a design pass on a rename is honest. Skipping
it on a schema change is not smaller, it is deferred and more expensive.

Default to the light end, then escalate on evidence. The reverse policy —
heavy ceremony by default, trimmed when someone complains — trains everyone
to route around the process, and then the process protects nothing.

## Reading risk before it reads you

Certain surfaces make work riskier than it looks, regardless of diff size.
Treat any of these as a signal to escalate the shape:

- **Auth, permissions, session handling.** Mistakes are silent and
  exploitable. There is no cosmetic change to an auth path.
- **Data model and migrations.** Code rolls back; data does not. A wrong
  column type shipped for a week is a week of corrupted rows.
- **Public contracts** — APIs, CLI flags, file formats, event schemas.
  Anything a consumer you can't see may depend on. Removing "unused"
  response fields is a breaking change until proven otherwise.
- **Cross-system boundaries.** Two services, a queue between them, a
  third-party API. Failure modes multiply at the seams, and the seams are
  exactly where local testing is weakest.
- **Behavior an existing test asserts.** The test is a signed statement
  that someone once cared. Changing the behavior means finding out why
  before deciding the test is merely stale.

Example: "add a `deleted_at` column and hide soft-deleted users" reads
like an afternoon. It touches the data model (migration), every query
that lists users (a contract with each caller), and probably an index.
The honest shape has a design step; the dishonest one has an incident.

De-escalate only on evidence, never on optimism. "The migration is
additive and no query filters on this column — verified by search" earns
a lighter shape. "It's probably fine" earns nothing.

## What makes a good unit of work

A unit of work is the atom of a plan: one thing a person or agent picks up,
finishes, and proves. Four properties make one good:

- **One outcome.** State it in a sentence with one verb. "Validate upload
  size on the server and return 413" is one outcome. "Fix uploads and
  clean up the handler" is two, and the second will be done badly while
  attention is on the first.
- **A verifiable exit state.** Before starting, you can write down the
  command or check that will prove it done — a test that flips green, an
  endpoint that returns the new field, a build that passes with the flag
  on. "Done when it feels solid" is not an exit state.
- **Explicit dependencies.** Name what must exist first. A unit that
  discovers its prerequisites mid-flight blocks, and blocked work rots.
- **Files named.** List where the change lands. This is the cheapest
  possible dry run: if you cannot name the files, you have not understood
  the work, and the plan is hiding a research task inside an
  implementation task. Split them.

Size follows from these properties rather than from a line-count rule: a
unit small enough to hold one outcome and one exit state is small enough.

## Dependency thinking

Two orders matter, and they are usually different:

- **Build order** — what must exist before what can be written. The
  migration before the query; the interface before its two implementors.
- **Discovery order** — what you must *learn* before what can be decided.
  Whether the third-party API paginates before you design the sync loop.

Plans fail when they sequence only build order. If a decision in unit 4
depends on a fact nobody has verified, that verification is itself a unit,
and it goes first — usually as a spike with a written answer as its exit
state. Front-load discovery: the most expensive dependency is the one you
find at unit 4 that invalidates units 1 through 3.

Draw the dependency edges explicitly, then look for what is *not* there:
units with no edges between them can proceed in parallel, and pretending
otherwise serializes work for no reason.

## Plan the current slice; headline the rest

Detail decays. Every specific decision made about phase 3 — file names,
function signatures, edge-case handling — is a bet placed before phase 1
has taught you anything. Most of those bets lose, and each loss costs
twice: once to write, once to notice it is now wrong and misleading.

So plan asymmetrically. The current slice gets full resolution: units,
dependencies, files, exit states. Later slices get headlines — one line
each stating the outcome, enough to prove the sequence is sane and the
current slice isn't painting anything into a corner. "Slice 3: replace
the polling loop with the webhook, remove the poller" is a fine headline.
Its unit breakdown, written today, is waste with a due date.

## The walking skeleton

For anything user-visible, the first slice is the thinnest end-to-end path
that actually runs: real request in, real behavior out, every layer touched
once. Not mocks, not stubs, not "the backend part first" — a narrow tunnel
through the whole system that a user (or a test acting as one) can traverse.

Example: for "CSV export of orders," the skeleton is a button that
downloads a real CSV containing one hard-coded-query column of real data.
No filters, no streaming, no encoding options. It is small and unimpressive
and it proves the route, the auth, the query path, and the download
plumbing all connect — the four places the surprises live.

Structural work — abstractions, generalization, performance — rides
*after* proof of life, because until the skeleton walks you do not know
which structure the system needs. Abstractions designed before the first
end-to-end pass are guesses wearing architecture's clothes; half of them
generalize the wrong axis, and by then they have dependents.

## The smaller-path question

Before presenting any plan, ask once, explicitly: *is there a cheaper
shape that still honors every constraint?* Not a cheaper shape that drops
a requirement — one that meets all of them with less machinery.

Answer with one line of evidence, not a feeling. "No — the requirement to
keep old links working forces the redirect table; nothing smaller
preserves them" is an answer. "This seems about right" is not; go look.
Common cheaper shapes hide in plain sight: an existing mechanism that
already does 80% of this, a config value instead of a subsystem, deleting
the thing instead of fixing it.

If the question finds a smaller path, redraft. The ten minutes of
redrafting is the highest-leverage time in the entire effort — every unit
you delete now is a unit nobody implements, reviews, or maintains.

## Scope integrity: split, never shrink

When agreed work will not fit the time or size available, there are
exactly two honest moves: split it into slices, or renegotiate it with
whoever owns the requirement. There is no third move where the plan
quietly delivers less than what was agreed and hopes the gap goes
unnoticed. Silent shrinkage is the most corrosive planning failure,
because the owner's mental model and the actual system diverge without
either party knowing.

Splitting is not shrinking. "Search ships this slice without typo
tolerance; typo tolerance is slice 2, here is its headline" preserves the
full commitment and makes the deferral a visible, reversible decision.
Propose the slice boundary — you know where the natural seams are — but
the *choice* of what waits belongs to the owner, because only the owner
knows which half of the value was the point.

## Plans are contracts

A plan the owner approved is a fixed point. It stays byte-stable from the
moment of approval; the approved text is what "on plan" means, and it is
the only thing drift can be measured against. If the artifact quietly
evolves under the work, "we followed the plan" becomes unfalsifiable.

Reality will still disagree with the plan — that is expected and fine.
When it does, the change is a new decision made in the open: state what
changed, why, and what it displaces, and get the owner's yes. Then the
plan is amended as a visible edit. What is never fine is discovering the
change in the diff. An edited plan and a changed plan look identical in
the file; only the conversation around them distinguishes a decision from
a drift.

## The cold-pickup test

The final check on any plan: could someone with zero conversation history
— none of the chat, none of the whiteboard, none of what "we all know" —
implement each unit correctly from the written artifacts alone?

Walk each unit and ask what an outsider would have to guess. Every guess
is a hole. The usual holes: a term of art defined only in discussion
("the *legacy* importer — which of the three old ones?"), a constraint
everyone agreed to verbally, an exit state that lives in someone's head,
a "the obvious place" that is obvious to exactly one person.

This is not bureaucracy; it is the definition of whether planning is
finished. A plan that requires its author standing next to it is not a
plan — it is a promise to be interrupted. Work that passes cold pickup
can be handed to anyone, parallelized freely, and resumed after a month.
Work that fails it has a single point of failure, and it is you.
