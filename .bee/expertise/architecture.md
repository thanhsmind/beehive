# How to structure code

Named principles for structural decisions at any scale — a function, a
module, a service. Each entry is a handle: cite the principle you are
applying or the anti-pattern you are naming, so a structural argument
is about a known trade-off rather than taste.

## Routing: situation → principle

| Situation | Principle |
|---|---|
| The solution feels complex; unsure if the problem is | Simplicity First |
| Building for a need nobody has yet | YAGNI |
| Two working designs; one needs explaining | Prefer the Obvious Approach |
| One change forces edits across unrelated concerns | Separation of Concerns |
| Callers know things they will break on | Information Hiding |
| Files grouped by kind, not by change-reason | Cohesion over Convenience |
| Components blur into each other at the edges | Explicit Boundaries and Contracts |
| An import reaches into another module's guts | Depend on Interfaces, Not Internals |
| Stable code imports from churning code | Dependency Direction |
| A class hierarchy is bending to share behavior | Composition over Inheritance |
| The same fact lives in two places | Single Source of Truth |
| Code is littered with "this can't happen" checks | Make Illegal States Unrepresentable |
| Tempted to extract on the second occurrence | The Rule of Three |
| Judging whether a suspect module earns its complexity | The Deletion Test |
| Wrapping a dependency behind an interface "just in case" | One Adapter, Two Adapters |
| Deciding how to test a module against its dependencies | Dependency Taxonomy Drives Test Strategy |
| Old unit tests break after deepening a module they used to cover | Replace, Don't Layer |
| A structure smells wrong but lacks a name | Anti-patterns to Name on Sight |

## Principles

### Simplicity First

When a design feels complex → ask which kind: is the problem hard, or
is our solution complicated? Essential complexity comes from the
problem itself — a payroll engine encodes tax law, and no structure
makes tax law simple. Accidental complexity comes from our choices —
the abstraction layers, the indirection, the generality — and it is
the only kind we can remove. Audit by subtraction: for each layer,
ask what breaks if it is deleted. If the answer is "nothing, but it
felt more proper," it was accidental. The simplest design that solves
the actual problem wins over the cleverest, and a design that is easy
to change beats one that is theoretically complete.

### YAGNI

When code is being built for a requirement that exists only as a
prediction — "we'll need multi-tenancy eventually," "someone might
want a plugin here" → don't build it. Speculative structure costs
four times: the effort to build it, the delay to what is needed now,
the drag it adds to every change that must route around it, and the
rework when the future arrives shaped differently than predicted —
which it usually does. Build for today's requirement and keep the
code easy to change; changeability is the real hedge against the
future, not prediction.

The boundary: YAGNI vetoes speculative features and premature
flexibility, never quality. Tests, clear names, honest error
handling, and small functions are what keep code changeable — they
are how YAGNI stays affordable, not what it cuts.

### Prefer the Obvious Approach

When two approaches both work and one requires explanation → take the
one a reader understands on first pass. Cleverness is a cost paid by
every future reader, including you in six months; it buys something
only when the obvious approach genuinely cannot meet a measured need.
The tell is the justifying comment: if the code needs a paragraph
explaining why it isn't written the plain way, first check whether
the plain way was actually tried.

### Separation of Concerns

When touching one behavior forces you to understand and re-test
several unrelated ones → the concerns are fused; split them. Each
unit — function, module, service — should have one responsibility,
statable in one sentence without "and". Parsing and validating and
persisting in one function means a change to the persistence rules
risks the parser. The payoff is blast radius: a well-separated change
is reviewable in isolation and breaks only what it touches.

### Information Hiding

When callers would break if a design decision changed → that decision
belongs behind an interface. Choose module boundaries by asking what
is likely to change independently — the storage format, the vendor,
the algorithm — and hide exactly that. The interface exposes only
what callers need to do their job: not the record layout, not the
retry policy, not the fact that there is a cache. When the hidden
decision changes, one module's internals change and nothing else
moves. A module that "hides" nothing is just a folder.

### Cohesion over Convenience

When code is grouped by what it is — all helpers in `utils/`, all
types in `types/`, all constants in one file → regroup by why it
changes. The question for placement is: when this changes, what must
change with it? Things that change together for the same reason live
together; things that merely look alike or get used at the same time
do not. A `utils/` file is a place where code goes to lose its
change-reason — each function in it belongs to some concern, and
filing it as "util" hides which one.

### Explicit Boundaries and Contracts

When it is unclear where one component ends and the next begins →
draw the line and write down the deal. A boundary is a contract: the
data format, the operations, the error behavior the two sides agree
on. Inside the boundary, each side changes freely; at the boundary,
changes are negotiated. External systems get the strictest form —
translate their model into yours at the edge, so a vendor's API
change hits one translation layer instead of bleeding through the
codebase. A boundary nobody wrote down is a boundary that erodes one
convenient shortcut at a time.

### Depend on Interfaces, Not Internals

When an import path reads like a directory tour —
`orders.internal.db.helpers.format` → the dependency is on layout,
not contract, and reorganizing `orders` will break it. Depend on what
a module publishes as its surface; if the thing you need is not on
the surface, either the surface is missing an export or the
dependency should not exist — decide which, explicitly. This holds
between siblings in the same repo as much as across packages: each
module declares its public face, and everyone else takes only that.

### Dependency Direction

When something stable imports from something volatile → the arrow
points the wrong way. Dependencies should flow from the parts that
change often toward the parts that change rarely: business rules
should not import the web framework; the domain model should not
import the database driver. Read the import lines as a coupling map —
each one is a promise to move when the imported thing moves. When a
stable core must react to a volatile detail, invert the arrow: the
core defines the interface, the detail implements it, and the churn
stays on the churning side.

### Composition over Inheritance

When a class hierarchy is being bent to share behavior — a subclass
overriding a parent method to do nothing, a base class sprouting
flags to serve divergent children → stop inheriting and start
composing. Inheritance couples a child to its parent's entire
implementation and freezes one axis of variation into the type tree;
composition takes each capability as a part and assembles them,
letting axes vary independently. The litmus: inherit only when the
subtype is honestly substitutable everywhere the parent is expected.
"It needs most of that class's behavior" is a reason to hold a
collaborator, not to extend it.

### Single Source of Truth

When the same fact lives in two places → one of them will be wrong,
and readers cannot tell which. Every fact — a limit, a status list, a
derivable total — gets one authoritative home; everything else
derives from it at read time or is regenerated from it mechanically.
A cached or denormalized copy is acceptable only when it is
mechanically derived and the derivation is the only writer; a copy
maintained by parallel hand-edits is a data race with human hands.
When you find divergent copies, the fix is not syncing them — it is
electing the owner and demoting the rest to derivations.

### Make Illegal States Unrepresentable

When code is dotted with defensive checks for states that "can't
happen" → restructure the data so they truly can't. A connection
modeled as `{connected: bool, socket?: Socket}` permits
`connected: true` with no socket, and every reader must guard
against it; modeled as `Disconnected | Connected(socket)`, the
absurd combination has no spelling, and the guards evaporate. Prefer
types over runtime checks, checks at the boundary over checks
everywhere, and required constructor arguments over half-built
objects patched up later. Every state the structure cannot express
is a class of bug that cannot be written.

### The Rule of Three

When you meet the second occurrence of a pattern and feel the pull to
extract → wait for the third. Two points define any line: with two
examples you cannot tell the shared essence from coincidence, and an
abstraction guessed from two callers tends to fit neither once a
third arrives — it then grows parameters and conditionals to cover
cases it never anticipated. Duplication is a cheap, honest debt,
paid off easily when the pattern proves real; the wrong abstraction
is an expensive one, because unwinding shared code is far harder
than sharing duplicated code. Extract on the third occurrence, when
the invariant part has shown itself. And when an existing
abstraction already fits badly, inline it back into its callers,
delete what each does not use, and re-extract from what remains.

### The Deletion Test

When judging whether a suspect module earns its complexity → imagine
deleting it and inlining its behavior at every call site. If the
complexity it held reappears at each of the N call sites, the module
was doing real work and earned its keep — deleting it would only move
the complexity, not remove it. If the complexity simply vanishes with
no trace at any call site, the module was a pass-through: an
interface wrapped around nothing, paying the cost of indirection for
no behavior. A three-line `OrderValidator` wrapping one if-check
disappears cleanly on deletion — inline it and remove the module. A
`RetryingClient` wrapping a flaky network call does not: delete it,
and each of its five callers now needs to write its own backoff loop
— it was earning its keep.

### One Adapter, Two Adapters

When deciding whether a dependency needs a port and an adapter behind
it → count how many concrete adapters would actually exist. One
adapter — a single production implementation with no real test
double swapped in at that seam — is a hypothetical seam: the
interface exists for a variation that has not arrived, which is
Speculative Generality under another name. Two adapters — a
production implementation and a real stand-in used in tests, or two
live implementations that genuinely differ — makes the seam real,
because something actually varies across it. Wait to introduce the
port until the second adapter is actually needed, never because a
second one is merely imaginable. Wrapping a single Postgres call
behind a `Store` trait "in case we swap databases later" is one
adapter — delete the trait, call Postgres directly. Wrapping the same
call behind `Store` to run real Postgres in production and an
in-memory store in tests is two adapters — the trait earns its place.

### Dependency Taxonomy Drives Test Strategy

When a module depends on something outside itself → classify the
dependency before deciding how to test it, because the category
decides the strategy, not habit:

- **In-process** — pure computation, in-memory state, no I/O. No
  adapter needed; test the module directly.
- **Local-substitutable** — a real, local stand-in exists (an
  in-memory filesystem, an embedded database). Test against the
  stand-in; the seam stays internal, with no port at the module's
  external interface.
- **Remote-but-owned** — your own service across a network boundary.
  Define a port at the seam: an in-memory adapter drives the tests,
  the real transport drives production, and the logic stays in one
  deep module even though it is deployed across a wire.
- **True external** — a third party you do not control (a payment
  processor, an SMS gateway). Inject the dependency as a port and
  mock only that port at the seam; never mock a collaborator you own.

### Replace, Don't Layer

When a module is deepened behind a wider interface → delete the old
shallow unit tests that targeted its former pieces once interface-level
tests cover the same behavior through the new interface. Keeping both
is layering, not extra safety: two suites now assert overlapping
ground, and the old ones sit behind the new interface where a
legitimate internal refactor — reordering a step, renaming a helper —
turns them red with no behavior change. That red is the diagnosis: the
old tests were testing past the interface, and the fix is deletion,
not a rewrite. A module split into `helper()` and `validate()` with
their own unit tests, then merged behind one deep function: once tests
against the merged function's interface cover the same ground, delete
the two old unit tests the same day the replacement lands.

## Anti-patterns to Name on Sight

Naming a smell precisely is the first step to arguing about it
productively. Know these on sight:

- **God object** — one component that accumulated responsibilities
  until everything depends on it and nobody changes it safely.
  Detect by counting its reasons to change; more than a few is the
  diagnosis. Split along those reasons.
- **Circular dependency** — A imports B imports A. Neither can be
  understood, tested, or released alone. The cycle means the
  boundary is drawn wrong: merge them, or extract the shared piece
  both actually depend on into a third module below both.
- **Shotgun surgery** — one logical change requires small edits in
  many scattered files. The concern is fragmented; gather it into
  one home so the next such change lands in one place.
- **Speculative generality** — hooks, parameters, and abstract
  layers serving callers that do not exist. Unused flexibility is
  pure cost; delete it and let YAGNI hold the line.
- **Primitive obsession** — domain concepts passed around as bare
  strings and numbers: an email that is "just a string", money that
  is "just a float", so every function re-validates or forgets to.
  Wrap the concept in a type that validates once at construction,
  and let the rest of the code trust it.
