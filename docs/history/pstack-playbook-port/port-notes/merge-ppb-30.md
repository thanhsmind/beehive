# Port notes — pstack principles merged into the bee principles, cell ppb-30

Source: pstack principles `principle-fix-root-causes`,
`principle-type-system-discipline`, `principle-boundary-discipline`,
`principle-build-the-lever`, `principle-encode-lessons-in-structure`,
`principle-migrate-callers-then-delete-legacy-apis`,
`principle-separate-before-serializing-shared-state` and
`principle-subtract-before-you-add`, MIT, copyright 2026 Lauren Tan. Each of
the nine bee skills below got `LICENSE.pstack`, a verbatim copy of the pstack
`LICENSE`. This file is the only place a pstack-only tool is named.

Rule applied (D5): the bee skill stays bee's. Every bee rule, the description
and the Check, Not when, Why and Depth frame are kept; only what the bee skill
lacked came in, in bee's voice. No description changed (all under 300
characters). The `expertise/` depth files are outside this cell and were not
edited.

## Mapping (decision 6ca3cd62)

- fix-root-causes goes to two skills, split by what each rule is about:
  `bee-principle-crash-site-versus-fault-site` and
  `bee-principle-reproduce-first`.
- type-system-discipline goes to `bee-principle-make-illegal-states-unrepresentable`.
- Each other source goes to its bee namesake: boundary-discipline,
  build-the-lever, encode-lessons-in-structure,
  migrate-callers-then-delete (source name ends `-legacy-apis`),
  separate-before-serializing-shared-state, subtract-before-you-add.

## What came in

crash-site-versus-fault-site (from fix-root-causes):
- Ask why until the answer is a cause, not another symptom.
- Restart bugs: suspect stale persistent state before code.
- A workaround that needs a paragraph to justify it means the code is wrong.
- A Check line: where was this value created and what else does the origin feed.

reproduce-first (from fix-root-causes):
- When it will not fail on demand, instrument instead of guessing: log the
  suspect value, read the actual error, vary one input.
- A restart is part of the repro when the report says it fails after one.
- A Check line.

make-illegal-states-unrepresentable (from type-system-discipline):
- Variants as a sum type, not optional fields; the completed-plus-optional-time
  example; "can this combination happen" as the tell.
- Types are constructions (head plus rest, start plus duration).
- Do not lie to the compiler: casts and bypassing assertions.
- Strengthen a type only where partiality appears, then stop; total functions.
- Four more checks (cast origin, comment-explains-validity, new variant,
  derive from owner). Not when: nothing would otherwise panic.
- Cross-references to `bee-principle-boundary-discipline` and
  `bee-principle-encode-lessons-in-structure`, where the source cited its own
  boundary and encode skills.

boundary-discipline:
- Pure-function examples: parsing, prompt construction, scoring.
- General-purpose mechanism inside, special-purpose policy at the edge.

build-the-lever:
- Make the tool safe to rerun; commit it when the work outlives the session.
- The tool is the artifact a reviewer reruns.
- A deterministic tool beats fan-out.
- The recipe file for workers: steps, verification contract, do-not-touch
  fences, outside their write scope.
- The bar is triviality, not repetition (a one-off earns a lever when the
  lever makes it checkable).
- Applying it produces a file: a Check line for the cited-without-a-tool case.
- Smallest tool, never a framework; cites laziness-protocol, prove-it-works,
  make-operations-idempotent.

encode-lessons-in-structure:
- The finer order inside the structural rungs: unrepresentable state, lint or
  banned API, canonical helper, runtime check; and why (workers copy the
  surrounding code).
- When the fix is structural, use only the structural fix.
- The feedback loop: capture, route (one-off, recurring, systemic), close the
  loop.
- The three anti-patterns.

migrate-callers-then-delete:
- Inventory every caller first.
- An adapter is exceptional, named, time-boxed.
- Two live paths slow cleanup and make the code feel append-only.

separate-before-serializing-shared-state:
- The three shapes of shared state to list (files, branches, records).
- Structural writers: sequential phases and compare-and-swap added to the list.
- Telling workers to take turns is not concurrency control.

subtract-before-you-add:
- Out-of-spec features drag validators behind them (persistence, retry on
  startup, schema migration).
- Simplify prompts: drop the redundant line and the oversized template.

## Left out because bee already has it

<!-- bee:not-a-deferral: a record or description of work already done, never a promise to act -->
- fix-root-causes: reproduce first (already `bee-principle-reproduce-first`);
  "instrument, do not guess" as a method (homed in `expertise/debugging.md`
  § Instrument before guessing, cited from the skill by behavior, not copied);
  check for the pattern, not the instance (already in crash-site); the
  "a null-check silences a crash" rule (already in crash-site).
- type-system-discipline: brand semantic primitives, validate once at creation,
  parse external data at the boundary, exhaustive matching, derive from an
  authoritative schema: all already in the bee skill. The source's
  `typescript-best-practices` pointer is dropped; that skill is deferred
  (CONTEXT, Deferred Ideas) and no bee counterpart exists.
- boundary-discipline: validate at the boundary, trust inside, no re-exported
  transport types, thin shell: already in the bee skill.
- migrate-callers-then-delete: tests move with the callers, delete tests that
  pin old internals, the bridge needs a removal condition, the "no external
  users" condition (bee's Not when states it inverted): already in the bee skill.
- separate-before-serializing-shared-state: eliminate the shared target by
  default, serialize only for a real invariant, the two-writers-one-JSON
  example: already in the bee skill.
- subtract-before-you-add: removal first, cut before polish, build for observed
  usage, no speculative validators, delete a content-free stub, leave the design
  simpler: already in the bee skill.
- encode-lessons-in-structure: the ladder, delete the prose once the fix
  exists, the instruction is the symptom: already in the bee skill.
<!-- /bee:not-a-deferral -->

## Tool swaps

- encode-lessons-in-structure: "brain note" becomes a record through
  `bee-capturing`; "concrete todo" becomes a backlog item. Skill, lint rule and
  principle stay as plain nouns.
- build-the-lever: "subagents" becomes workers; the fan-out pointer is
  `bee-fanout`. No subagent type or model is named.
- No other tool was swapped; no step was dropped for want of a bee counterpart.

## Added, not in the source

- A Check line on crash-site and reproduce-first (the source has the tests on
  other principles only); the bee frame has one.
- Cross-references: crash-site to reproduce-first; reproduce-first keeps its
  link to red-before-green.

## Regen note

`bee dev regen` also re-renders unrelated `.claude/skills/` entries in this
checkout, as in ppb-27 to ppb-29; those stay uncommitted. Only the nine
directories in each rendered tree, the render manifests, the onboarding hashes
and the release manifest are committed.
