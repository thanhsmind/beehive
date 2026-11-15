# ppb-31c: check of the pstack how merge in ppb-31

Checked against commit `7414b8ebe` (ppb-31). The source is the pstack skill
`how` at `/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/how/`:
`SKILL.md` and its four references, `explorer-prompt.md`, `explainer-prompt.md`,
`critic-prompt.md` and `critique-rubric.md`. The bee side is
`skills/bee-how/SKILL.md` and `skills/bee-researching/references/trace-and-provenance.md`
("Trace", "Trace answer shape", "Critique"). No file other than this report was
edited.

## Verdict

The merge holds. Almost every source rule is present in Trace or in the bee-how
door. The rules that are left out each name a bee home that already holds them,
and the port notes list them correctly. Five small points are missing. None
changes what a trace or a critique produces. Two of them (M1, M5) concern how
the fan-out runs, not wording.

- **pstack-only tool names:** none. A case-insensitive scan of both bee files
  for `generalPurpose`, `grok`, `gpt-`, `fable`, `opus`, `readonly`, `Task`,
  `Glob`, `Grep`, `pstack` and `cursor` matches one line only:
  `trace-and-provenance.md:28` "Never hand-pick a `subagent_type`". That line is
  an older bee rule that forbids the pstack pattern; it is not a pstack tool
  call. The pstack name appears only in `port-notes/merge-ppb-31.md` and in the
  `LICENSE.pstack` file name.
- **bee rules removed:** none. The commit's diff to the two files has 11 `-`
  lines. Each one comes back in a `+` line with its original words kept and new
  words added: the description, the dependency reason, the read-only line, "are
  one entry point, not two", "Drop a section that does not apply", Overview, Key
  concepts, the How it works diagram sentence and Where things live.
- **Citations resolve:** `bee-interrogate` Step 3 (`SKILL.md:43`: three
  reviewers by default, two for a small change, same-model disclosure), Step 5
  (`SKILL.md:77`) and `references/lead-judgment.md` all exist. All eight cited
  principle skills exist under `skills/`. `LICENSE.pstack` is byte-identical to
  the pstack `LICENSE`. The bee-how description is 289 characters, under the 300
  budget.
- **D5 kept:** bee-how stays bee's. Trace stays in one home, and the door
  restates no step of it.

## Source rules

Present = the rule is in the bee text (with where). Left out = deliberately not
ported, with the bee home that already holds it. Missing = not in bee and not
named as left out.

### SKILL.md

| # | Source rule | Status | Where / home |
|---|---|---|---|
| 1 | Triggers: "how does X work", architecture, runtime flow, onboarding mental models | present | bee-how description and Scope |
| 2 | Trigger: placement, ownership and layering questions | present | description "which package or layer owns it"; Scope bullet 3 |
| 3 | Trigger: code walkthroughs before changing something | **missing (M4)** | the description has no walkthrough-before-change phrase, and the port notes do not name it |
| 4 | Why questions go to the why skill | present | description "Not for why (bee-why)"; Scope bullet 1 |
| 5 | Goal: a senior engineer's mental model, not annotated source | present | Trace answer shape, opening sentence |
| 6 | Two modes, Explain (default) and Critique | present | Trace; "A Trace that the question asks to judge ends in `Critique`" |
| 7 | Parse the question type (subsystem, feature flow, overview, runtime trace) | present | Trace step 1 (question as a path), step 2 (slices for a subsystem) |
| 8 | If ambiguous, state a best guess and do not ask | present | Trace step 1 |
| 9 | Assess complexity: simple goes inline, complex fans out | left out | Trace steps 2 and 4: entry points from evidence, one honest entry point runs inline |
| 10 | When in doubt lean simple; fan out later if the explainer hits a wall | **missing (M5)**, second half | step 4 runs inline but never says to fan out when an inline trace finds more entry points; the "lean simple" half is replaced by bee's evidence rule (step 2) |
| 11 | Split into 2–4 distinct slices so explorers do not duplicate work | present | Trace step 2: slices, floor two, ceiling four, disjoint |
| 12 | Spawn all explorers in a single message | **missing (M1)** | Trace step 3 does not say "all in one turn"; Critique step 1 does say it for reviewers |
| 13 | Explorer `subagent_type`, `model`, `readonly` | present (swapped) | Trace step 3: read-only worker through `bee dispatch prepare --kind gather` |
| 14 | Explorer walk: start broad, follow the thread, read the code, stop when nothing is hand-waved, note surprises | present | Trace step 3, walking brief |
| 15 | Explorer return: components, flow, files read, boundaries, non-obvious, open questions | present | Trace step 3 return shape (components found, ordered steps, boundaries, surprises, UNFOLLOWED, files read) |
| 16 | Overlap between explorers is fine; the explainer reconciles | present | Trace step 3 last sentence; step 5 |
| 17 | Step 2b: one explainer subagent explores and explains a simple question | left out | Trace step 4: the leader traces inline |
| 18 | Step 3: an explainer subagent synthesizes the findings | left out | Trace step 5: the leader writes the account; a return is evidence |
| 19 | Step 4: present the explainer's output, lightly edited | left out | moot under Trace step 5 |
| 20 | Output: Overview, Key Concepts, How It Works, Where Things Live, Gotchas; not every section needed | present | Trace answer shape |
| 21 | Critique only when architectural issues are asked for | present | Critique opening; bee-how Scope bullet 3 |
| 22 | Critique step 1: explain first | present | Critique opening: "Trace first and write the account" |
| 23 | One critic per configured model, all in one message | left out (swapped) | Critique step 1 cites `bee-interrogate` Step 3; "all in one turn" kept |
| 24 | Model lists are minimum levels; the lead escalates reasoning | left out | the dispatch door and the team table pick the tier |
| 25 | A critic gets the explanation, the file paths and the rubric | present | Critique step 2 |
| 26 | Lead judgment like interrogate: Act on, Consider, Noted, Dismissed; pragmatic lead, not aggregator | present (cited) | Critique step 4 cites `bee-interrogate` Step 5 and `lead-judgment.md` |
| 27 | Explanation first, critique verdict below; the explanation stands alone | present | Critique opening; bee-how Scope bullet 3 |

### references/explorer-prompt.md

| # | Source rule | Status | Where / home |
|---|---|---|---|
| 28 | Gather facts; a separate agent writes the prose, so favor accuracy over prose | left out | Trace step 5: a return is evidence; the return shape asks only for facts |
| 29 | Other explorers hold other slices; focus and go deep | present | Trace step 3 walking brief |
| 30 | Find files, find symbols, read the implementation; do not guess from names | present (tool-neutral) | "list the directories, search the key type and interface names … a file name is never evidence" |
| 31 | Pattern 1–2: find the entry point, trace the call chain and how data transforms | present | Trace step 2 (entry point) and step 3 ("callers, callees, data flow") |
| 32 | Pattern 3: map the key abstractions, read their definitions | present | walking brief "type definitions"; return "components found" |
| 33 | Pattern 4: find the boundaries, what goes in and what comes out | present | return "boundaries the path crosses"; Trace step 1 "what enters … what comes out" |
| 34 | Pattern 5: the non-obvious, historical artifacts, newcomer traps | present | return "surprises"; answer shape Gotchas "history, traps" |
| 35 | Say "I couldn't determine how X connects to Y" instead of guessing | present | walking brief; UNFOLLOWED |
| 36 | Exact paths, function and type names, line numbers | present | `path:line` throughout Trace |
| 37 | Flow return includes the data that flows between steps | **missing (M2)** | the return's ordered steps carry `path:line` and "what it calls next", not the data passed |

### references/explainer-prompt.md

| # | Source rule | Status | Where / home |
|---|---|---|---|
| 38 | Reconcile overlaps; settle contradictions by checking the code | present | Trace step 5, plus naming an unsettled disagreement under Gotchas |
| 39 | The explainer may check the code but should not re-explore | left out | the leader is the explainer (step 5) |
| 40 | Overview: one or two paragraphs; a reader can stop there | present | answer shape Overview |
| 41 | Key Concepts: brief, not exhaustive | present | "Only those; never exhaustive" |
| 42 | How It Works is the core and the longest section | **missing (M3)**, "longest" half | "core" is implied by the runtime-path wording; "longest" is not stated |
| 43 | Prose not pseudocode; reference files and functions; no large code blocks unless essential | present | answer shape How it works |
| 44 | Diagram: mermaid for structured flows, ASCII for simple ones; clarify, never decorate; skip when prose covers it | present | "only when it makes the path clearer than prose does: mermaid … ASCII" |
| 45 | Where Things Live: only what is needed to start | present | "not every file" |
| 46 | Gotchas: skip when empty | present | "Drop a section that does not apply" |
| 47 | Concrete language; the `UserService` / `AuthClient.refresh()` example | present | "Name the real parts", same example |
| 48 | Explain why a complex part is complex; do not pad a simple part | present | answer shape How it works |
| 49 | An analogy only when it helps | present | "never force one" |
| 50 | Acknowledge open questions honestly | present | UNFOLLOWED steps under Gotchas (step 6) |

### references/critic-prompt.md

| # | Source rule | Status | Where / home |
|---|---|---|---|
| 51 | Use the explanation as a map; form an opinion from the code; it may miss things or be charitable | present | Critique step 2 |
| 52 | Architectural problems, not line-level bugs or style | present | Critique step 3 |
| 53 | Severity structural / concern / observation, with definitions | present | Critique step 3, with the note that these are not P1–P3; "technical debt" in the observation definition is dropped, no rule lost |
| 54 | Finding names the components, the boundary, the coupling | present | "naming the components and the boundary" |
| 55 | Evidence: concrete code, the dependency chain | present | Critique step 3 |
| 56 | Impact: the concrete cost | present | Critique step 3 |
| 57 | Avoid list: line-level review, rewrites without a shown problem, abstraction with no stated gain, flagging intentional tradeoffs | present | Critique step 3 last paragraph |
| 58 | A sound architecture: say so; an empty critique is valid | present | Critique step 3 |
| 59 | Per-finding output template (Components, Finding, Evidence, Impact) | left out | the reviewer return form is `bee-interrogate` `references/reviewer-prompt.md`; Critique's close names severity, reviewers and rationale per finding |

### references/critique-rubric.md

| # | Source rule | Status | Where / home |
|---|---|---|---|
| 60 | Use the lenses that apply; not every lens applies | present | "Use the ones that apply" |
| 61 | Abstraction fit: real concept versus speculative indirection; boundaries split what changes independently; accidental coupling; business logic tangled with framework wiring; over-abstraction is as bad as under-abstraction; a flat design is fine | present | lens Abstraction fit, all six points |
| 62 | Data model: shaped for access versus first model; impedance mismatch; honest types | present | lens Data model, all three points |
| 63 | Boundary discipline: validation at entry points; errors at boundaries; typed shapes versus bags of optionals; testable in isolation | present | lens Boundary discipline, all four points |
| 64 | Evolution readiness: one file or everything; hardcoded assumptions; bolted on or integrated; legacy paths nobody uses; do not penalize hypothetical changes | present | lens Evolution readiness, all five points |
| 65 | Complexity versus value: concentrated where it is needed; simpler route; every component earns its place, no vestigial pieces | present | lens Complexity versus value, all three points |
| 66 | Consistency: same solution as elsewhere; a good reason or independent drift; unexplained inconsistency is a cost | present | lens Consistency |

## Missing points

- **M1 — parallel fan-out in one turn.** pstack spawns every explorer in one
  message. Trace step 3 says "fan one read-only worker at each entry point" but
  never says "all in one turn". The Critique step does. A reader could run
  gathers one after another. A fix is three words in step 3.
- **M2 — data between steps.** The explorer Flow return carries the data that
  moves between steps. Trace's return shape names the next call, not the data.
  The account's How it works asks for "where the data goes", so the leader
  needs that data and has to re-read the code for it.
- **M3 — How it works is the longest section.** A weighting cue, not a rule
  change.
- **M4 — walkthrough before a change.** The pstack trigger "code walkthroughs
  before changing something" is not in the description or the Scope. bee-how's
  "Not for … changing code" excludes making the change, not walking the code
  first, so the case routes correctly by its "how … runs" wording. Only the
  explicit trigger phrase is gone. The description has 11 characters of budget
  left, which may be why. The port notes do not say.
- **M5 — fan out after an inline pass.** pstack: if the single-pass explainer
  hits a wall, spawn explorers. Trace step 4 covers one honest entry point, but
  does not say what to do when an inline trace finds a second disjoint entry
  point mid-trace.

## Checked and not a finding

- Port notes say the Consistency lens has no bee principle. That is true: no
  `bee-principle-*` covers codebase pattern consistency, so the lens is a plain
  question.
- Critique "two for a narrow subsystem" and interrogate "a small change takes
  two" are the same rule applied to a different object. Not drift.
- The rendered `.claude/skills/bee-how/` gained `CREATION-LOG.md` and a larger
  `SKILL.md` diff than the other trees. `skills/bee-how/CREATION-LOG.md`
  already exists in the source tree, so that render was stale and caught up. It
  is not a content change from the merge.
