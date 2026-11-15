# ppb-30c: check of the pstack improvements merged in ppb-30

Checked against commit `578647b6d` (ppb-30). The sources are the eight pstack
principle `SKILL.md` files at
`/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/`:
`principle-fix-root-causes`, `principle-type-system-discipline`,
`principle-boundary-discipline`, `principle-build-the-lever`,
`principle-encode-lessons-in-structure`,
`principle-migrate-callers-then-delete-legacy-apis`,
`principle-separate-before-serializing-shared-state` and
`principle-subtract-before-you-add`. Each source directory holds only that one
file. No file other than this report was edited.

## Verdict

Most source rules are present in the nine bee skills. Some source rules are
left out on purpose, and each one names the bee home that already holds it.
Seven small points are missing. None of them changes how a principle is applied,
but two of them are applicability conditions, not wording. One port-note
statement is wrong (see type-system-discipline, test 5).

- **pstack-only tool names:** none. A case-insensitive scan of the nine
  `SKILL.md` files for `cursor`, `/loop`, `graphite`, `bugbot`, `poteto`,
  `brain note`, `subagent`, `typescript-best`, `pstack`, `dump-to-sqlite`,
  `arena`, `architect`, `disable-model` and the pstack link form
  `principle-*/SKILL` returns no match. The swaps the port notes list are the
  ones in the text: "brain note" became a `bee-capturing` record, "concrete
  todo" became a backlog item, "subagents" became workers with a `bee-fanout`
  pointer, and "delegate skill" became a recipe file.
- **bee rules removed:** none. Every `-` line in the commit's nine `SKILL.md`
  diffs comes back in the `+` lines, reworded or extended. No description
  changed. The only other deleted lines are regenerated hashes in
  `.bee/onboarding.json` and the release manifest.
- **License and renders:** all nine `LICENSE.pstack` files match the pstack
  `LICENSE` byte for byte. The nine `SKILL.md` and `LICENSE.pstack` files are
  identical in `.claude-plugin/skills`, `.codex-plugin/skills`,
  `.claude/skills`, `.agents/skills` and `.opencode/skills`.
- **Frontmatter:** each source's `disable-model-invocation: true` is a pstack
  loader flag, not a rule, and is not carried.

Status words: **present** means the rule is in the bee skill body, or in its
depth section where noted. **Left out** means bee already holds the rule
somewhere else, and that home is named. **Missing** means it is in neither the
skill nor its depth section.

## fix-root-causes → crash-site-versus-fault-site and reproduce-first

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Don't paper over symptoms; trace each problem to its root cause and fix it there | present | crash-site body ("trace the bad value backward to its origin and fix it there") |
| 2 | Why: symptom fixes pile up, each workaround makes the system harder to reason about, and the real bug remains | present | crash-site Why line ("a series of local patches"); body ("leaves the fault standing") |
| 3 | Why: root-cause fixes are slower upfront but reduce total debugging time | missing | not in either skill or in `expertise/debugging.md`. Bee's Why line gives a different reason (patches hide evidence) |
| 4 | Reproduce first; without a repro you can't verify the fix | left out → `bee-principle-reproduce-first` | reproduce-first body opening; crash-site points to it |
| 5 | Ask "why" until you hit the root cause | present | crash-site body (new in ppb-30) |
| 6 | Resist guards; a nil check that silences a crash is a symptom fix | present (bee rule, existed before) | crash-site body ("A null-check at the crash site silences the alarm") |
| 7 | A workaround that needs a paragraph-long comment means the code is wrong; fix the code, not the comment | present | crash-site body (new) |
| 8 | Check for the pattern, not just the instance; grep and fix all instances | present (bee rule, existed before) | crash-site body ("search for the same shape elsewhere … Fix every instance") |
| 9 | When stuck, instrument; don't guess (add logging, read the actual error) | present | reproduce-first body (new); crash-site ("add observation before you guess"); `expertise/debugging.md` holds the method |
| 10 | Restart bugs: code doesn't change between runs, state does; suspect stale persistent state first (config, caches, lock files, serialized state) | present | crash-site body (new, all four examples); reproduce-first adds "a restart is part of the repro" |
| 11 | If clearing a state file restores behavior, make state validation the fix | present | crash-site body (new) |

## type-system-discipline → make-illegal-states-unrepresentable

<!-- bee:not-a-deferral: a record or description of work already done, never a promise to act -->
| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | The type checker is a proof assistant; a case the types let you ignore becomes a runtime failure | present (in substance) | Why line and opening ("let the compiler refuse"); the phrase "proof assistant" is not carried |
| 2 | Define errors and special cases out of existence instead of adding handlers | present | Why line ("Defining the special case out of existence beats writing a handler") |
| 3 | The tools are unrepresentable states, total functions and interface redesign | missing (interface redesign only) | the first two are in the body; "interface redesign" as a tool is not named anywhere |
| 4 | Applies to any typed language; `typescript-best-practices` gives syntax | left out → none, recorded | the pointer is dropped because that skill is deferred (CONTEXT § Deferred Ideas); port notes record it. The bee skill is language-neutral |
| 5 | Model variants as sum types, not a bag of optional fields | present | body (new) |
| 6 | The `{ completed; completedAt? }` anti-pattern; derive the flag or spell the variants | present | body (new), in language-neutral form |
| 7 | "Can this combination actually happen?" means the type is too loose | present | body (new) |
| 8 | Types are constructions: non-empty list = head + rest; time range = start + duration | present | body (new) |
| 9 | No representation is privileged; choose the shape that cannot build the illegal value (list of pairs read as an even-length list) | missing (the "no representation is privileged" point) | the body has "each cannot hold the illegal value" but not the point that any shape can be chosen. Not in `expertise/architecture.md` either |
| 10 | Brand semantic primitives; validate once at creation, trust downstream | left out → bee rule | body opening (newtype per meaning, existed before); per-language idiom list not carried |
| 11 | External data is untyped until parsed; parse at every boundary | left out → `bee-principle-boundary-discipline` | body sentence with cross-reference; `expertise/architecture.md` § Make Illegal States Unrepresentable |
| 12 | Don't lie to the type system: casts, unsafe coercion, bypassing assertions; prove it or accept the hazard | present | body (new) |
| 13 | Exhaustive matching is the compiler's job | left out → bee rule | body ("without a catch-all arm"); depth section. Per-language idioms not carried |
| 14 | Derive types from authoritative schemas; manual copies drift | left out → bee rule | body with cross-reference to encode-lessons-in-structure; depth section |
| 15 | Strengthen a type only where partiality appears, then stop; prefer total functions (sum vs head) | present | body (new) |
| T1 | Test: can a comment explain when this field combination is valid? Split it | present | Check line (new) |
| T2 | Test: do two arguments share a primitive but mean different things? Brand them | present | Check line (existed before) |
| T3 | Test: where did this `any`/`as`/`assertNotNull` come from? Trace it to the boundary | present | Check line (new) |
| T4 | Test: will the compiler name each place when a variant is added? | present | Check line (new) |
| T5 | Test: is this type duplicating a shape another file owns? Derive instead | missing as a test; the rule is present | the rule is in the body (row 14), but the Check line has only T1 to T4. `port-notes/merge-ppb-30.md` says four checks came in, including "derive from owner". That is wrong: the fourth new check is the variant question, and derive-from-owner is not in the Check line |
| T6 | Test: strengthening to keep an operation total, or only for precision? If nothing would panic, keep the plain type | present | Not-when line (new) |
<!-- /bee:not-a-deferral -->

## boundary-discipline

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Validation, narrowing and error handling sit at system boundaries; trust internal code | left out → bee rule | body opening (existed before) |
| 2 | Business logic in pure functions; thin, mechanical shell | left out → bee rule | body (existed before) |
| 3 | Why: scattered validation is noise and gives false safety; logic outside framework wiring can be tested without it | left out → bee rule | Why line (existed before) |
| 4 | At boundaries (CLI args, config, external APIs, network): validate, return errors, handle defensively | left out → bee rule | body opening lists the inputs; Not-when line covers defensive handling |
| 5 | Inside: typed data, propagate errors, no re-validation | left out → bee rule | body ("a check deep in a call chain means the edge did not finish its job") |
| 6 | Across the boundary, expose domain concepts, not the private representation | present | body (new: "expose domain concepts") |
| 7 | General-purpose mechanism inside, special-purpose policy at the edge | present | body (new) |
| 8 | Validate config at parse time; parse raw data into domain types at the boundary | left out → bee rule | body opening |
| 9 | Don't re-export transport, storage, framework or wire types | left out → bee rule | body ("such as a wire format or a storage row") |
| 10 | No redundant nil checks deep in call chains | left out → bee rule | body opening |
| 11 | Pure-function examples: parse functions, prompt construction, scoring | present | body (new, all three) |
| T1 | Test: is this data crossing a boundary right now? | left out → bee rule | Check line |
| T2 | Test: can this be a pure function the shell calls? | left out → bee rule | Check line |

## build-the-lever

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | When the work isn't trivial, build the tool instead of doing it by hand | left out → bee rule | body opening |
| 2 | Why, throughput: the tool does it the same way every time and reruns for free | present (in substance) | body opening ("a person … makes a different small mistake in every tenth unit"); "reruns" in body |
| 3 | Why, confidence: the tool is one artifact a reviewer can read and rerun; hand edits can only be checked by redoing them | present | body (new) |
| 4 | Default to the lever; skip it only for a couple of obvious edits | present | body (skip line existed before; "bar is triviality, not repetition" is new) |
| 5 | Do the first unit by hand, build the tool, rerun it on that unit and diff | left out → bee rule | body (existed before) |
| 6 | Make the lever safe to rerun; a reviewer will | present | body (new, cites make-operations-idempotent) |
| 7 | Kinds: codemod or script, generator, query for analysis, rerunnable check | present | body opening plus new examples; the pstack-specific "dump-to-sqlite" became "a query" |
| 8 | A deterministic lever beats fan-out | present | body (new) |
| 9 | When fanning out, write the recipe, verification contract and do-not-touch fences in one artifact all delegates read, outside their write scope | present | body (new, "recipe file", `bee-fanout`) |
| 10 | Applying it produces a file; cited with no tool in the diff means not applied | present | Check line (new) |
| 11 | Commit the lever when the work outlives the session | present | body (new) |
| 12 | A one-off earns a lever when it makes the work checkable | present | body (new) |
| 13 | Smallest script, never a framework (Laziness Protocol) | present | body (new, cites laziness-protocol) |
| 14 | Distinct from Encode Lessons in Structure (durable guardrail vs throughput on the work at hand) | missing | the skill does not cite encode-lessons-in-structure, and `expertise/planning.md` § Build the lever does not draw the line |
| 15 | For scripting the verification itself, see Prove It Works | present | body (new, cites prove-it-works) |

## encode-lessons-in-structure

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Encode recurring fixes in mechanisms, not textual instructions | left out → bee rule | body opening (the ladder) |
| 2 | Every error, human correction and unexpected outcome is a learning signal: capture, route, close the loop | present | body "Close the loop" paragraph (new); "unexpected outcome" is folded into "a check fails" |
| 3 | Why: text needs the reader to notice, remember and comply; structure enforces without cooperation | left out → bee rule | Why line |
| 4 | Pattern steps 1 to 3: can it be a lint rule, metadata flag, runtime check or script; encode and delete the instruction; otherwise make it prominent with an example | left out → bee rule | body ladder and "Prose then has a job" paragraph |
| 5 | Pick the strongest rung: unrepresentable state, lint or banned API, canonical helper, runtime check; agents copy the surrounding code | present | body (new); "agents" became "workers" |
| 6 | Don't paper over symptoms; if the fix is structural, use ONLY the structural fix | present | body (new sentence) |
| 7 | The instruction IS the symptom | left out → bee rule | body (existed before) |
| 8 | Capture every correction; decide one-off or pattern | present | body (new) |
| 9 | Route to the right layer: one-off → note, recurring → skill or lint rule, systemic → principle | present | body (new); "brain note" became a `bee-capturing` record |
| 10 | Close the loop: apply now or create a concrete todo | present | body (new); the todo became a backlog item |
| 11 | Anti-patterns: acknowledging without recording, recording without routing, fixing without generalizing | present | body (new, all three) |

## migrate-callers-then-delete-legacy-apis → migrate-callers-then-delete

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Migrate callers and remove the old API in the same wave; no compatibility layers | left out → bee rule | body opening |
| 2 | Don't keep a legacy path alive only because internal callers exist | left out → bee rule | body ("Skip the compatibility shim") |
| 3 | Inventory callers, migrate them, delete the old API immediately | present | body opening (new: "inventory every caller") |
| 4 | Temporary adapters are exceptional and time-boxed, not default architecture | present | body (new) |
| 5 | Tests assert the new contract; delete tests that only protect old internals | left out → bee rule | body (existed before) |
| 6 | Applies when no external users depend on backward compatibility | left out → bee rule | Not-when line, stated the other way round |
| 7 | Applies when the project can absorb coordinated breaking changes | missing | in neither the skill nor `expertise/architecture.md` § Migrate Callers, Then Delete. The Not-when line covers outside users only |
| 8 | Applies when the new API is part of a simplification or refactor initiative | missing | in neither the skill nor the depth section |
| 9 | Both APIs together give dual-path complexity, slow cleanup, and make the code feel append-only | present | Why line (new) |

## separate-before-serializing-shared-state

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | First ask whether actors truly need the same mutable object; if not, remove the sharing | left out → bee rule | body opening |
| 2 | When sharing is real, serialize through structure; instructions and conventions are not concurrency control | present | body ("never a convention callers must remember", existed before; "Telling workers to take turns is not concurrency control", new) |
| 3 | Why: concurrent writes cause intermittent, hard-to-reproduce races | left out → bee rule | Why line ("shows up only under load and survives every quiet test") |
| 4 | Identify shared state: files both read and write, branches both push to, APIs both define and consume | present | body (new; "APIs" became "records") |
| 5 | Default: remove the shared write target; give each actor its own file, key, branch or state directory; merge at read time | present | body ("branch" new); "state directory" is folded into "file" |
| 6 | Two workers writing their own field into one `state.json` still share it; one file per writer does not | present | body (new second half) |
| 7 | Serialize only for a real invariant: lockfiles, sequential phases, single-writer actor, atomic compare-and-swap | present | body ("sequential phases" and "compare-and-swap" new; the owning task is the single-writer actor) |
| 8 | "We need a lock" is a design smell to check, not the default answer | left out → bee rule | Why line |

## subtract-before-you-add

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Remove complexity first, then build; deletion gives a simpler base | left out → bee rule | body opening |
| 2 | Why: adding to a complex system compounds complexity | left out → bee rule | Why line |
| 3 | Why: removing first reveals the essential structure and often makes the next design obvious | missing | in neither the skill nor `expertise/planning.md` § Subtract before you add |
| 4 | Make simplification a continual investment; leave the design slightly simpler and more capable behind the same or a smaller surface | present, with one word missing | body and depth have "slightly simpler … same capability behind the same surface or a smaller one". "Continual investment" and "more capable" are not carried |
| 5 | Sequence removal before construction | left out → bee rule | body; Check line |
| 6 | Cut before you polish | left out → bee rule | body |
| 7 | Design for observed usage, not speculative edge cases | left out → bee rule | body |
| 8 | No speculative validators, parsers or guards beyond the spec | left out → bee rule | body ("a validator no contract asks for"); depth bullet |
| 9 | Out-of-spec features drag validators behind them (persistence, retry on startup, schema migration) | present | body (new) |
| 10 | Simplify prompts: remove redundant instructions and oversized templates | present | body (new) |
| 11 | Delete a reference with no new content instead of leaving a stub | left out → bee rule | body ("a stub that holds no content"); depth bullet |

## Missing items, in one list

1. fix-root-causes Why: root-cause fixes are slower upfront but cut total
   debugging time.
2. type-system-discipline: "interface redesign" as one of the tools for
   defining special cases out of existence.
3. type-system-discipline: "no representation is privileged" (choose the
   shape that cannot build the illegal value).
4. type-system-discipline test 5 as a Check question ("is this type
   duplicating a shape another file owns?"). The rule itself is present. The
   port notes wrongly say it came in as a check.
5. build-the-lever: the line that separates it from
   encode-lessons-in-structure.
6. migrate-callers-then-delete: two of the source's three applicability
   conditions (the project can absorb coordinated breaking changes; the new API
   is part of a simplification). Of the seven, these two matter most, because
   they narrow when the rule applies.
7. subtract-before-you-add Why: removal reveals the essential structure and
   makes the next design obvious. Also "continual investment" and "more
   capable" in the leave-it-simpler line.

None of these is a pstack-only tool. Each could be added to its skill in bee's
voice. That is a docs change outside this checker's scope; ppb-37, or a
follow-up cell, can pick it up.
