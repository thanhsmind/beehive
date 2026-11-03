# Design Sketch — Shape New Code Before The Gate

Load when new code has an open shape on `standard` or `high-risk` work:
several whole shapes are viable and no pattern in the repo fits. Never load
it for a bug fix, a rename, or a change with a pattern to copy. Lanes open
only after a traced model of the code the shape touches exists, per
`bee-researching/references/trace-and-provenance.md` ("Trace"), and the
lanes' brief names that trace in its read diet: a lane that designs against
a guessed runtime designs for the wrong system.

Adapted from pstack (cursor/plugins, pstack/skills/architect).

## Caller's usage first

Write the usage before the types: the README or quickstart the caller reads,
plus two or three real call sites in the caller's own code — what it
imports, what it calls, what comes back. Derive the types from that usage.
When the two disagree, change the sketch to fit the usage, never the
reverse: the caller's experience is the spec.

At the plan step, `hat-facts-gaps` reads this usage block against the plan's
`## Shape`, per `bee-hive/references/gates-and-delegation.md` ("Hat wave").
A call site the types cannot serve is a Structure BLOCKER.

## Sketch

Sketch the data structures first, then the signatures and module boundaries
the data flows through. Bodies stay `not implemented`, tricky logic stays
pseudocode, and doc comments state intent and invariants. A reader traces
data from input to output by the types and signatures alone.

Write the sketch into the feature's plan files under `docs/history/<feature>/`
— the `## Shape` section of `plan.md`, or a file beside it. Never write it as
source before the merged shape+execution gate is approved: source written
first locks in a shape nobody approved.

The implementation runs in `bee-swarming` cells after the gate, against the
sketch. A cell that needs something the sketch did not name records a
deviation: the sketch was wrong, a requirement was missed, or the cell
overreaches.

## Candidates

When several whole shapes are viable, run the candidates as blind lanes, per
`bee-hive/references/gates-and-delegation.md` ("Blind lanes and convergence")
— it holds when lanes open, the moves, and the record. Every dispatch reads
`--runtime <rt>`. A candidate is a whole shape, never a point fix inside one
shape.

The lanes' shared brief carries this runner discipline:

- Caller's usage first, per the section above.
- Data structures first. Trace each main access pattern through the
  structure. "We will add a map, an index, or a cache later" means the
  structure is wrong.
- Interface depth. Prefer a small interface that pulls complexity into the
  callee. Keep transport and wire types off the public API; parse into
  domain types behind it.
- Shared state. When two actors can both write, ask what happens. Unless the
  answer is "nothing", keep state per actor and merge at the read boundary.
- Visible boundaries. `not implemented` bodies, pseudocode for tricky logic,
  doc comments for intent and invariants.
- Invariants in types first, runtime checks second, prose comments last.
- Validate at external boundaries and trust the types inside. Business logic
  is pure functions; the shell stays thin.
- One source of truth per invariant: derive, never sync, per
  `bee-principle-single-source-of-truth`.
- Idempotent transitions: a second run, or a crash halfway, corrupts
  nothing.
- Short call chains. A flow a reader must chase across many files is a
  hierarchy to flatten.
- Produce the best design you can. Never hedge toward a safe middle: the
  differences between lanes are the signal convergence picks from.

Each lane returns a design package: the usage, the type sketch, the module
map, and a rationale in the shape of § Rationale. The leader writes the
rubric before the lanes run (move 0, Frame, in the section cited above).
Interface depth and § Design red flags are default rubric criteria, not the
whole rubric: add the criteria this question turns on. A shape that trips a
red flag loses on that criterion; prefer the shape that hides more
complexity behind a smaller public surface.

## Design red flags

- **Shallow module** — a large interface that hides little. Signs: callers
  call several methods to finish one operation; public options expose
  internal stages; learning the interface does not spare the caller the
  implementation. A deep module concentrates capability behind one
  interface; a deep call chain scatters it across layers. Never confuse the
  two.
- **Information leakage** — several modules depend on one internal decision
  (a representation, a policy, a protocol detail), so a change needs
  coordinated edits. A re-exported transport or wire type is leakage. Keep
  storage schemas, framework objects, and protocol details private.
- **Temporal decomposition** — modules cut by execution order (load,
  validate, transform, save) instead of by the knowledge they own, so one
  representation and its invariants repeat across boundaries. Group code by
  the decisions it protects.
- **Pass-through method** — a method that forwards the same arguments to a
  method of the same shape. Remove it, or move the work to the module that
  can finish it. Keep a forwarding boundary only when it adds policy,
  adaptation, or a distinct abstraction.

## Rationale

The prose that ships beside the sketch, in the plan, with sentence-case
headings:

- **Problem** — what the work does, and what makes the shape non-obvious:
  the existing types to interoperate with, the callers that cannot break,
  the invariants that cross the boundary.
- **Usage** — the caller's view, written first; the shape derives from it.
- **Shape** — data structures first, then the flow through the signatures.
  Name the load-bearing decisions, the invariants the types encode, where
  validation lives, and what the system does not do. State what complexity
  the public surface hides and why it is no larger. Cite the principle
  behind each decision; never restate it.
- **Synthesis decision** — taken from the blind-lane dossier: its chosen
  answer as the base, what the design took from each other proposal, and its
  rejected set with the reasons. When no lanes ran, say why.
- **Tradeoffs accepted** — one line each: "we accept X in exchange for Y".
  Name anything a reader could mistake for an oversight.
- **Alternatives considered** — required. Name a concrete alternative shape
  and why it lost, judged on interface depth. When the constraints forced
  the answer, write "this was the only viable shape because …". These are
  design alternatives, never flavors of the same shape.
- **Open questions and risks** — phrased as questions the human answers.
  Each one lands in the plan's `## Open Questions` before the gate.
- **Next implementation step** — the first thing a cell builds against the
  sketch, in one sentence.

## Scrap tells

When the implementation keeps producing friction the sketch cannot absorb,
scrap the sketch; never bolt fixes onto an unsound design. The signal is a
pattern, never a single instance:

- the same shape of workaround in unrelated code;
- unrelated edge cases that each need a special-case branch;
- types that need escape hatches (`any`, casts, optional fields that are
  always set) to compile;
- the "we need a lock" reflex where the sketch said the state was not
  shared;
- callers that must know the abstraction's internal rules to use it;
- repeated, independent cell deviations of the same shape.

A few edge cases do not condemn a shape: complexity in the data is not
complexity in the design.

A scrap returns to `bee-planning`. Trace what was built, per
`bee-researching/references/trace-and-provenance.md` ("Trace"); redesign as
if the new constraints held from day one; subtract before adding, so the new
sketch is smaller than the old one before it grows. Supersede the old sketch
through `bee decisions log --relation supersedes:<id>`, then take the new
shape back through the gate.
