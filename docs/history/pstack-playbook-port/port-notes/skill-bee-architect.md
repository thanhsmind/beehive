# Port notes — skill bee-architect, cell ppb-13

Source: pstack skill `architect` (`SKILL.md` and `references/design-red-flags.md`,
`rationale-template.md`, `runner-prompt.md`) at commit `b9ddc83`, MIT, copyright
2026 Lauren Tan. The notice is `skills/bee-architect/LICENSE.pstack`, a verbatim
copy of the pstack `LICENSE`. This file is the only place a pstack-only tool is
named; no file under `skills/bee-architect/` names one.

Kept verbatim: `design-red-flags.md`. Kept as the base, with the swaps below:
`SKILL.md` (five phases, Phase E tells, Outputs), `rationale-template.md`,
`runner-prompt.md`.

Frontmatter follows `bee-how`: `name`, a description of 296 characters (what it
does, when to use it, what it is not for) and `metadata` with `ecosystem: bee`
and a `bee-cli` dependency that degrades to the leader writing both candidates.
The source's `disable-model-invocation: true` is dropped: no other bee skill
outside the upkeep and setup skills carries it, and the description already names
the explicit trigger.

## Tool swaps

- Subagent spawning through the `arena` skill (Phase B) became the blind-lane
  procedure of `bee-hive/references/gates-and-delegation.md` ("Blind lanes and
  convergence"): frame a rubric, fan out with
  `bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json`
  (`lane-2`, `lane-3`), converge into a dossier, `bee blind check --dossier`
  green before `bee decisions log`. Bee already has blind lanes, so no second
  mechanism was written. The payload is used as returned; no model or subagent
  type is named.
- "Your configured architect runners (defaults: four named models)" became the
  `lane-1` to `lane-3` roles of the bee team table. The four model names are
  dropped. Four runners became two or three lanes, the count the blind-lane
  procedure allows.
- The source opened the fan-out on every use. Bee opens lanes on the leader's own
  judgment when a decision is both high-stakes and ambiguous, or when the user
  orders them. The skill states both: a `/bee-architect` invocation is the order;
  reached from a playbook, the leader opens lanes only on the high-stakes and
  ambiguous test, and otherwise writes two candidates itself so "design it twice"
  holds without a dispatch.
- Arena's "returns one synthesized design package" became the dossier's base and
  grafts; the rationale's "Synthesis decision" section is filled from the
  dossier's `## Chosen` section. The template's link to the arena skill was
  rewritten to point at Phase B.
- The runner prompt's "isolated working directory, path to write outputs" became
  "a lane writes no file; its answer is the package and the leader copies it
  verbatim into the dossier", because an advisor lane is read-only.
- `how` became `bee-how`; `why` became `bee-why`.
- The how skill's "critique mode" has no bee counterpart (`bee-how` traces and
  has no critique mode). The step stayed as a sentence: read the traced path
  asking where the current shape resists the new one.
- "Open a todolist" became "open a task list" (the host's own list; no tool named).
- Principle skills that bee has: `separate-before-serializing-shared-state`,
  `encode-lessons-in-structure`, `boundary-discipline` (runner prompt),
  `subtract-before-you-add` (Phase E), all as `bee-principle-<slug>`.
- Two bee principle pointers are additions with no pstack counterpart in the
  source text: `bee-principle-attack-the-premise` on the Phase E trigger ("when a
  second fix fails the same check") and `bee-principle-lock-the-contract` in Phase D.
- The no-code-comments rule: the runner prompt's `// TODO` pseudocode and "doc
  comments stating intent and invariants" became pseudocode in the package prose
  and intent and invariants in the rationale, since bee writes no comment in code.

## Pointers kept as plain sentences

The skill or principle does not exist in `skills/` yet, so its method stays as a
sentence and the name is dropped from the shipped text:

- `arena` skill (cell ppb-14): replaced by the blind lanes above, as the
  procedure; no sentence is left over.
- `interrogate` skill (cell ppb-15): Phase C reads "press on the synthesized
  sketch with the hardest objection you can build, and answer it in the
  rationale".
- `exhaust-the-design-space` principle: "This is exhausting the design space,
  made concrete."
- `foundational-thinking` principle: Phase C reads "get the data shapes and the
  contract right first".
- `outcome-oriented-execution` principle: Phase C reads "the outcome is what is
  held fixed, not every intermediate state".
- `redesign-from-first-principles` and `fix-root-causes` principles: Phase E reads
  "redesign from first principles, and fix the root cause instead of the symptom".
- `laziness-protocol` and `minimize-reader-load` principles: the runner prompt's
  last bullet reads "do the least that works, and keep the reader's load small".
- `make-operations-idempotent` principle: the runner prompt's idempotence bullet
  states its method (run twice, crash halfway, the answer is "nothing new").

When ppb-14, ppb-15 and the slice 4 principles land, a later cell may swap the
sentence for the pointer.

## Dropped steps

None dropped outright. The `arena` pointer and the four model names are the only
text removed; each is covered by the swaps above.

## Trigger fixture

Seven briefs added to `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`:
four expect (one Vietnamese) and three near (expect `bee-planning`, `bee-shaping`,
`bee-how`, the last in Vietnamese), none naming the slug.
