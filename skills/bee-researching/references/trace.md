# Trace

Load only after `bee-researching` is active. `Trace` answers "how does this
actually run?". It is read-only: the outcome is an account, never a diff. A
Trace that the question asks to judge ends in `Critique`.

The outcome is a runtime path — the ordered steps a call actually takes, each
anchored `path:line`. A file list is not a trace.

1. State the question as a path: what enters, what runs, what comes out.
   When the target is vague, write your reading of it as the first line of
   the account and go on. Never stop to ask; a stated reading can be
   corrected, a stalled trace cannot.
2. Pick the entry points from evidence — a CLI verb, an HTTP route, a hook, a
   test that drives the behavior. Two is the FLOOR and four is the CEILING.
   Entry points must be DISJOINT: two names that reach the same first frame
   are one entry point, not two. When the question is a whole subsystem with
   no single call to follow ("how does the rate limiter work?"), cut it into
   distinct slices instead — data model and state, the request path,
   configuration and metrics — and treat each slice as an entry point: the
   same floor, ceiling and disjointness hold.
3. Fan one read-only worker at each entry point, every one spawned in the
   same turn. The ONE door is
   `.bee/bin/bee dispatch prepare --runtime <rt> --kind gather --json`. Run
   that door first, then run exactly the tool and payload it returns. Never
   hand-pick a `subagent_type`, a `model`, or a tier marker — the door
   returns them. Brief each worker with the question as a path, its one
   entry point, and this return shape, so the returns fold without
   rework:
   - the components found: each key type, service or abstraction with its
     `path:line` and one sentence on what it does;
   - the ordered steps, each with its `path:line`, what it calls next and
     the data it passes there;
   - the boundaries the path crosses (process, network, storage, runtime);
   - the surprises;
   - the UNFOLLOWED steps, each with its reason;
   - the files it read.

   Tell each worker how to walk: start broad (list the directories, search
   the key type and interface names), then follow the thread from its entry
   point through callers, callees, data flow and type definitions. Read the
   code; a file name is never evidence of what a function does. Stop when
   the path from trigger to effect has no hand-waved step. Other workers
   hold the other entry points, so go deep on this one and do not try to
   cover everything. A part it could not trace goes under UNFOLLOWED; "I
   could not determine how X reaches Y" beats a guess. Overlap between
   returns is fine, because step 5 reconciles.
4. One honest entry point means NO fan-out: the trace runs inline, and the
   account says so in one line. Splitting one path into two workers to reach
   the floor is theater. When the inline pass turns up a second disjoint
   entry point, stop and fan out per step 3.
5. Fold every return into ONE account. The leader writes the path; a worker's
   return is evidence, never pasted output. When two returns disagree on a
   step, open that step's anchor and settle it from the code. When the code
   does not settle it, name the disagreement under Gotchas — never pick one
   return silently.
6. Name every step the trace could not follow as UNFOLLOWED, with the reason
   — dynamic dispatch, a generated file, a binary, a network call. A gap
   named is data; a gap hidden is a wrong map.

Close with anchors a reader can open: `path:line` per step, or the command
that ran.

## Trace answer shape

Write the account for a senior engineer new to the area: enough to build a
working mental model and start working there, not annotated source. Drop a
section that does not apply.

- **Overview** — one or two paragraphs: what the thing is, what it does, and
  why it exists. A reader can stop here and decide whether to read on.
- **Key concepts** — the types, services, or abstractions a reader needs to
  follow the path, with brief definitions. Only those; never exhaustive.
- **How it works** — the core, and the longest section: the runtime path in
  prose: what starts it, each step,
  where the data goes, and the decision points, each step anchored
  `path:line`. When a step is complex, say why it is complex; when a step
  is simple, give it one line. Prose, never pseudocode, and no code block
  unless one snippet is essential to a point. Add a diagram only when it
  makes the path clearer than prose does: mermaid for a structured flow
  (sequence, flowchart, component graph), ASCII for a simple relationship.
  An analogy goes in only when it helps; never force one.
- **Where things live** — the files a reader opens first to work here, not
  every file.
- **Gotchas** — surprising behavior, history, traps, and the UNFOLLOWED
  steps from step 6.

Name the real parts. Not: "the service delegates to the client". Yes: "the
`UserService` calls `AuthClient.refresh()`".

## Critique

Run it only when the question asks for architectural problems or
improvements, or asks whether code sits in the right place: "where should
this live", "which package owns this", "is this the right layer". Trace
first and write the account; a critique of an architecture nobody traced is
opinion. The account then stands on its own, and the verdict goes below it,
so a reader who only wants to understand never wades through critique.

1. Send independent reviewers, never your own read alone. One
   `.bee/bin/bee dispatch prepare --runtime <rt> --kind reviewer --json`
   per reviewer, all in one turn, with no `--role`, each payload run
   exactly as returned. The dispatch, count and same-model rules are
   `bee-interrogate` Step 3: three reviewers, two for a narrow subsystem.
   When the door is absent, apply the lenses below one after the other
   yourself and say so in one line.
2. Brief every reviewer the same: the Trace account, the file paths, and the
   lenses below. Say that the account is a map that may miss things or
   frame them charitably, so each reviewer forms an opinion from the code.
   The brief carries the artifact and the question, never your verdict.
3. Each reviewer looks for architectural problems, not line-level bugs or
   style: is the subsystem built well for what it must do and for how it
   will have to change? Each finding carries four parts:
   - **Severity** — `structural` (wrong abstraction boundary, broken data
     model, coupling that will block future work), `concern` (harder to
     work with or reason about, not fundamentally broken) or `observation`
     (a tradeoff that may not age well, a pattern at odds with the rest of
     the codebase). These are not the review gate's P1 to P3.
   - **Finding** — the issue, naming the components and the boundary.
   - **Evidence** — concrete code: the dependency chain, not "too coupled".
   - **Impact** — what it costs: harder to test, harder to change, a
     performance cliff at scale.

   Reviewers do not suggest a rewrite without showing the problem in the
   current design, do not ask for "more abstraction" without saying what it
   would solve, and do not flag an intentional tradeoff with a clear
   benefit. A sound architecture gets "it is sound"; an empty critique is a
   valid result.
4. Judge as the lead, per `bee-interrogate` Step 5 and
   `bee-interrogate/references/lead-judgment.md`: bucket every finding Act on, Consider,
   Noted or Dismissed, a pragmatic lead and not an aggregator. A finding
   about behavior reaches the verdict only after you traced it.

The lenses. Use the ones that apply; the principle beside each homes the
rule, so cite it rather than restate it.

- **Abstraction fit** — does each abstraction stand for a real concept or is
  it an indirection "in case we need it"? Do the boundaries separate things
  that change independently? Is there coupling through shared
  implementation detail, or business logic tangled with framework wiring?
  Over-abstraction is as much a defect as under-abstraction; a flat design
  is fine when the domain is simple. `bee-principle-the-deletion-test`,
  `bee-principle-minimize-reader-load`.
- **Data model** — is the data shaped for how it is read, or for how it was
  first modeled? Does code keep reshaping it because the model does not
  match the access? Do the types claim more structure than exists at
  runtime? `bee-principle-foundational-thinking`,
  `bee-principle-model-the-domain`.
- **Boundary discipline** — validation at the entry points or scattered
  inside? Errors handled at the boundary or caught and rethrown at every
  layer? Data crossing as typed shapes or as bags of optional fields? Can
  the subsystem be tested without the whole system running?
  `bee-principle-boundary-discipline`.
- **Evolution readiness** — if the likeliest next requirement landed
  tomorrow, would it touch one file or everything? Which hardcoded
  assumptions would break? Does the design look integrated or bolted on?
  Is a legacy path kept for a caller that no longer exists? Judge only
  changes the codebase's trajectory makes plausible, never hypothetical
  ones. `bee-principle-redesign-from-first-principles`,
  `bee-principle-migrate-callers-then-delete`.
- **Complexity versus value** — is the complexity concentrated where the
  logic is hard, or spent on boilerplate, indirection and configuration? Is
  there a simpler way to the same behavior? Does every component earn its
  place, or is some of it left over from an earlier design?
  `bee-principle-laziness-protocol`, `bee-principle-the-deletion-test`.
- **Consistency** — does this area solve a problem the way the rest of the
  codebase does? Inconsistency is not automatically bad; unexplained
  inconsistency is a maintenance cost.

Close with the account first, then the verdict buckets, each finding with
its severity, its reviewers and a one-line rationale.
