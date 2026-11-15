# ppb-13c — bee-architect checked against its pstack source

Scope: what ppb-13 wrote in commit `52777acd0`. That is `skills/bee-architect/`
(`SKILL.md`, `references/design-red-flags.md`, `rationale-template.md`,
`runner-prompt.md`, `LICENSE.pstack`) and the port notes
`docs/history/pstack-playbook-port/port-notes/skill-bee-architect.md`. Each is
checked against the pstack skill `pstack/skills/architect/` (`SKILL.md` and the
same three references).

Verdict: no source step is missing. Every phase, every Phase E tell and every
scrap step is present, in bee's terms where a tool was swapped. The two texts
removed (the `arena` pointer and the four model names) both have port-notes
entries. The pstack-only tool fence is clean. No model and no subagent type is
picked by hand. The description is 296 characters and names what the skill is
not for. There are three findings. None of them blocks the cap.

Labels: **present** means the step is in the new file, in bee's terms where a
tool was swapped. **dropped (noted)** means it is gone and the port notes name
it. **dropped (not noted)** means it is gone and the notes say nothing, which
D2 (`063a21e4`) counts as missing. **missing** means a method step is gone with
no reason.

## `SKILL.md`

| Source step or rule | New home | Status |
|---|---|---|
| Frontmatter `name: architect` | `name: bee-architect` | present |
| Frontmatter `disable-model-invocation: true` | removed | dropped (noted: port notes, frontmatter paragraph) |
| Intro: sketch with `not implemented` bodies, synthesize across several model perspectives, fill in, throw out if wrong | line 18, "several independent design candidates" | present |
| Start: open a todolist with one entry per phase; five phases listed | lines 22-28, "task list" | present (swap noted) |
| A: build a real mental model; run the `how` skill over the subsystems | line 32, `bee-how` | present (swap noted) |
| A: `how` critique mode when existing structure is the constraint | line 32, kept as a sentence (read the traced path asking where the current shape resists) | present (no bee critique mode, noted) |
| A: naming a file is not grounding; produce the traced model | line 34 | present |
| A: run `why` when the design redefines ownership or layering | line 34, `bee-why` | present (swap noted) |
| A: skip only when greenfield | line 36 | present |
| B: run `arena` with the design-sketch task and Phase A artifacts | line 40, blind-lane procedure of `bee-hive/references/gates-and-delegation.md` | present (swap noted) |
| B: pass `runner-prompt.md` as each runner's prompt | line 44 | present |
| B: each candidate is a package per `rationale-template.md`, usage first | line 44 | present |
| B: use the configured architect runners (four named default models) | line 45, `lane-1` to `lane-3` team roles; model names gone | dropped (noted: four model names, tool swaps bullet 2) |
| B: design it twice, at least two structurally distinct candidates, whole-shape alternatives | line 46 | present |
| B: `exhaust-the-design-space` principle pointer | line 46, kept as a sentence | present (pointer as sentence, noted) |
| B: screen every candidate against `design-red-flags.md` | line 47 | present |
| B: compare on interface depth; prefer the smaller, simpler surface | line 48 | present |
| B: arena returns one synthesized package; synthesis fills the rationale's "Synthesis decision" | line 52, dossier `## Chosen` | present (swap noted) |
| C: default proceeds with no checkpoint | line 56 | present |
| C: opt in only on explicit ask; then pause for sign-off | line 58 | present |
| C: synthesis may ship as its own commit; `foundational-thinking` scaffold-first | line 60, kept as a sentence | present (pointer as sentence, noted) |
| C: planned breakage during fill-in is fine; `outcome-oriented-execution` | line 60, kept as a sentence | present (pointer as sentence, noted) |
| C: `interrogate` the synthesized sketch for adversarial pressure | line 60, kept as a sentence | present (pointer as sentence, noted) |
| C: human pushback is Phase A evidence; re-ground and re-run B | line 62 | present |
| D: replace bodies; the sketch is the contract | line 66 | present |
| D: deviations are signal; ask sketch wrong / requirement missed / overreach; surface, do not bolt on | line 68 | present |
| E: throw out a sketch that keeps producing friction; `redesign-from-first-principles`, `fix-root-causes` | line 72, kept as a sentence | present (pointer as sentence, noted) |
| E: the signal is a pattern | line 74 | present |
| E tell 1: same workaround shape across unrelated code | line 76 | present |
| E tell 2: unrelated edge cases needing special branches | line 77 | present |
| E tell 3: types needing escape hatches | line 78 | present |
| E tell 4: the "we need a lock" reflex | line 79 | present |
| E tell 5: callers must know internal rules | line 80 | present |
| E tell 6: two or more Phase D deviations of the same shape | line 81 | present |
| E: use judgment; data complexity is not design complexity | line 83 | present |
| Scrap 1: re-run `how` over what was built | line 87, `bee-how` | present |
| Scrap 2: redesign as if new constraints were day-one (`redesign-from-first-principles`) | line 88, name dropped | present (pointer as sentence, noted) |
| Scrap 3: subtract before adding (`subtract-before-you-add`) | line 89, `bee-principle-subtract-before-you-add` | present |
| Scrap 4: return to Phase B and re-run arena | line 90, "open the lanes again" | present |
| Outputs: usage first; one file or module map plus types; rationale per template | line 94 | present |

Additions with no source counterpart: the `metadata` block with the `bee-cli`
dependency, the lane-open rule on line 42, the rubric bullet on line 43, the
no-answer bullet on line 50, "An unattended run never opts in" on line 58, the
`bee-principle-lock-the-contract` sentence on line 66, the
`bee-principle-attack-the-premise` sentence on line 83, and the Handoff section.
The notes name the dependency, the lane-open rule and the two principle
pointers. The rest are not named, see finding 3.

## `references/design-red-flags.md`

Byte-identical to the source (`diff` prints nothing). Every rule is present.

## `references/rationale-template.md`

One line differs: the "Synthesis decision" note's link to `../../arena/SKILL.md`
now names the dossier's `## Chosen` section and links Phase B of the skill
(noted, tool swaps bullet 4). All eight sections (Problem, Usage, Shape,
Synthesis decision, Tradeoffs accepted, Alternatives considered, Open questions
and risks, Next implementation step) are present.

## `references/runner-prompt.md`

| Source rule | Status |
|---|---|
| Orchestrator passes the file to every runner with task, artifacts, isolated working dir, output path | present; "isolated working dir, output path" became "a lane writes no file" (noted, bullet 5) |
| Read the architect skill first; output one package per the template | present, `bee-architect` |
| The orchestrator compares on these axes to pick a base | present, "the leader grades candidates on the rubric" |
| Caller's usage first | present, unchanged |
| Data structures first | present, unchanged |
| Interface depth | present, unchanged |
| Shared state, per-actor state merged at read | present, `bee-principle-separate-before-serializing-shared-state` |
| Make boundaries visible: `not implemented`, `// TODO` pseudocode, doc comments | present; `// TODO` and doc comments became pseudocode and rationale prose (noted, no-code-comments bullet) |
| Encode invariants in types | present, `bee-principle-encode-lessons-in-structure` |
| Validate at boundaries, pure logic, thin shell | present, `bee-principle-boundary-discipline` |
| Single source of truth per invariant | present, unchanged |
| Idempotent transitions (`make-operations-idempotent`) | present, method as a sentence (noted) |
| Short call chains (`laziness-protocol`, `minimize-reader-load`) | present, method as a sentence (noted) |
| One of several runners on different models; do not hedge | present, "lanes, each possibly on a different model" |

## Pstack-only tool names

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-architect/` prints
nothing and exits 1. A wider search for `arena`, `interrogate`, `todolist` and
the source's model names (`fable`, `gpt-`, `grok`, `opus`) under
`skills/bee-architect/` also prints nothing. The only pstack names left are in
the port notes, which is where they belong.

## Hand-picked model or subagent_type

None. `rg -n -i 'subagent_type|model:|sonnet|haiku'` over `skills/bee-architect/`
prints nothing. Line 45 tells the leader to use the `bee dispatch prepare`
payload as returned and never pick a model or subagent type by hand; the model
per lane comes from the `lane-1` to `lane-3` team roles. Every principle the
skill names (`encode-lessons-in-structure`, `subtract-before-you-add`,
`lock-the-contract`, `attack-the-premise`,
`separate-before-serializing-shared-state`, `boundary-discipline`) exists under
`skills/`, and so do `bee-how` and `bee-why`.

## Description

296 characters (measured on the folded scalar after `description: >-`), under
the 300 cap. It says what the skill does, when to use it ("/bee-architect or asks
to architect or design non-trivial work") and when not to ("Not for cell planning
(bee-planning) or locking decisions (bee-shaping)").

## Findings

1. **The lane command drops `--brief-file`.** Line 45 gives
   `bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json`.
   The procedure it points at (`gates-and-delegation.md`, "Blind lanes and
   convergence", move 1) gives `--brief-file <path>` on every lane, and that flag
   is what pins `brief_sha256` for `bee blind check`'s brief-digest check
   (`packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`, the
   `--brief-file` resolver). A leader who copies line 45 as written sends lanes
   with no recorded brief, and line 49's `bee blind check` green step then
   cannot hold. The same line in `runner-prompt.md` line 3 has the same gap.
   Fix: add `--brief-file <path>` to both, or name the procedure's command and
   stop restating it. Worth fixing; does not block the cap.
2. **`--runtime claude` is fixed.** The procedure writes `--runtime <rt>`. The
   skill renders into the codex and opencode trees too, where `claude` is the
   wrong runtime. This is a runtime, not a model, so it does not break the
   no-hand-picked-model rule, but it is the same kind of hard-coding. Minor.
3. **Four additions are not named in the port notes.** The rubric bullet
   (line 43), the no-answer bullet (line 50) and "An unattended run never opts
   in" (line 58) come from the blind-lane procedure and bee's unattended rules,
   and the Handoff section is bee's standard skill exit. The port notes'
   "additions" bullet names only the two principle pointers. Nothing is lost;
   the notes are just incomplete. Minor.

## Proof

- `rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-architect/ ; test $? -eq 1`
  exits 0 (rg exit 1, no match). Static.
- `diff` of each source reference against its port, output above.
