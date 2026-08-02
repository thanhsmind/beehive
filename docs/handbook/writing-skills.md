# Writing bee skills (comb building)

> Maintainer guide — bee development, not shipped product.

Skills are code. They have bugs. Test them before deploying —
TDD-for-skills. Quoted headings resolve in the named reference.

**THE IRON LAW: NO SKILL WITHOUT A FAILING TEST FIRST.** Applies to edits
too — no exceptions, not for "simple additions," "just a section," or
"reference only."

## The constitution (what a skill body may contain)

Before the cycle, the placement question. Skills are the **craft** layer; the
CLI and hooks are the **machine**; `docs/` is **memory**. Six rules follow, and
a body that breaks one is fixed by moving content, not by rewording it:

- **Flow lives in the CLI.** A skill step invokes at most one flow verb. If a
  step must sequence three or more CLI calls to express one intent, that
  sequence is a missing verb — file it, don't narrate it.
- **The machine teaches at the point of contact.** Every refusal and flow-verb
  output names the next action. Prose never pre-explains what the machine
  will say anyway.
- **Single source for every rule.** A rule the machine enforces is *deleted*
  from prose. Prose keeps the intent ("cap with proof; the CLI refuses
  otherwise and tells you how to fix it").
- **Provenance exile.** No decision IDs, no deleted-rule changelogs, no
  bee-source `file:line` citations in a skill body or in CLI help. That map
  lives in `docs/decisions/`; provenance and creation logs live in
  `docs/decisions/skills/`.
- **Portability litmus.** Every skill must make sense dropped into a foreign
  repo. A sentence that only means something to someone developing bee itself
  belongs in `docs/`, not in the product.
- **Craft over choreography.** Bodies teach judgment — what makes a good cell,
  a good plan, a good finding — and delegate mechanics to the CLI. Depth that
  is craft rather than flow goes to `expertise/`, referenced as
  `.bee/expertise/<guide>.md`.

## The Core Cycle

| Phase | Action | Gate |
|---|---|---|
| RED | 3-5 pressure scenarios, ≥3 combined pressures, run WITHOUT the skill; record exact verbatim rationalizations, not paraphrases | HARD-GATE: no skill content written before this completes |
| GREEN | Minimal SKILL.md addressing only the RED rationalizations (hypothetical content bloats and gets skipped); re-run the same scenarios WITH the skill | Still failing → the skill is unclear; revise, do not proceed |
| REFACTOR | Any new rationalization → explicit negation + a rationalization-table row + a red-flag entry; re-run ALL scenarios | An agent violating a rule with the skill present is a skill bug, not agent error |
| VALIDATE | Manual checklist below, end to end; syntax-check any shipped script (`bash -n` for the cockpit's `.sh` helpers); write the creation log | No automated skill validator exists |

Scenario templates, the 7 pressure types, and the meta-test:
`writing-skills-references/pressure-test-template.md` ("The 7 Pressure Types").
Creation-log template: `writing-skills-references/creation-log-template.md`.
The log is a development record, not product: write it to
`docs/decisions/skills/<skill-name>-creation-log.md`, never inside the
skill directory.

## SKILL.md checklist (bee conventions)

- [ ] YAML frontmatter starts on line 1 (`---`); `name` hyphen-case with the `bee-` prefix, matches the directory exactly
- [ ] `description`: one purpose clause, then "Use when..." triggers — NEVER a workflow/step summary (agents follow the description and skip the body); third person, ≤1024 chars
- [ ] `metadata.version`, `metadata.ecosystem: bee`, and `metadata.dependencies` as a **mapping** (never a YAML array of objects — `writing-skills-references/checklist-examples.md` ("Dependency metadata style")). A skill that drives state declares the `bee-cli` dependency on `.bee/bin/bee` with its `missing_effect`
- [ ] Information density, not length: overflow to exactly one level of `references/`; a body line earns its place only by changing agent behavior — a line that doesn't belongs in `references/` or `expertise/`
- [ ] **Regrowth law:** a new learning lands in the knowledge bundle, `expertise/`, or `references/` by default; edit the body itself only for a load-bearing invariant
- [ ] **Per-turn rules (chat shape, communication) are never exiled to references** — they live in the always-loaded layer; a reference nothing forces open is a rule nothing follows
- [ ] Commands quoted in the body match the **live** surface — `bee --help --json` for porcelain, `--help --all --json` for everything — and use the flow spelling where one exists (`bee gate`, not `bee state gate`)
- [ ] Short `Headless` section; Red Flags or Hard rules list; HARD-GATE markers on critical stops (`writing-skills-references/checklist-examples.md` ("Persuasion principles"))
- [ ] Ends with a **References table**: one row per `references/` file and per `.bee/expertise/` guide, each with a *when to load* trigger — the routing surface is the skill's last section
- [ ] Cross-references other skills by name (`Invoke bee-planning`), never inlines their content

Description-trap example: `writing-skills-references/checklist-examples.md` ("Description trap").

## Headless

`mode:headless`: the Iron Law still binds — no skill content is written or
deployed without a completed RED phase and a GREEN verification. Ambiguous
design choices (scope, naming, which scenarios to run) are deferred to an
`Outstanding Questions` section of the terminal report, never guessed.

## Red Flags — STOP and Run Baseline Tests

writing skill content before creating any pressure scenarios · "I already
know what agents will do" · "it's just a small addition" · "academic
questions passed, that's sufficient testing" · description contains
workflow steps or a process summary · skill addresses hypothetical
scenarios not observed in baseline · deploying without re-running scenarios
WITH the skill (no green verification) · "the skill was good last month,
edits don't need testing" · restating a rule the CLI already enforces ·
citing a decision id or a bee source line in a body

All of these mean: stop, run baseline tests first. Violating the letter of
the rules is violating the spirit of the rules. Rationalizations already
seen and their reality: `writing-skills-references/rationalization-table.md`.

## Shipping the edit

A skill body is not the only copy. Regenerate the rendered trees
(`bee dev render-skill-trees`) so `.claude/skills/`, `.claude-plugin/skills/`,
and `.codex-plugin/skills/` move in lockstep — never hand-edit a render — and
run the declared suite (`bee test`) before the cap.

## Handoff

Skill pressure-tested, validated, and logged. Invoke bee-hive skill.

| Reference | When to Load |
|---|---|
| `writing-skills-references/pressure-test-template.md` | the 7 pressure types, ready-to-use scenario templates, the meta-test |
| `writing-skills-references/creation-log-template.md` | Creation-log template documenting the TDD process (written to `docs/decisions/skills/`) |
| `writing-skills-references/checklist-examples.md` | description trap, dependency-metadata YAML, persuasion-principle table |
| `writing-skills-references/rationalization-table.md` | common violation excuses and their reality, extended during REFACTOR |
