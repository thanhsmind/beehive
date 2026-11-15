# ppb-23c — bee-figure-it-out checked against its pstack source

Scope: what ppb-23 wrote in commit `a590767ab`: `skills/bee-figure-it-out/`
(`SKILL.md`, `LICENSE.pstack`) and the port notes
`docs/history/pstack-playbook-port/port-notes/skill-bee-figure-it-out.md`. They were
checked against the pstack skill `pstack/skills/figure-it-out/SKILL.md` at pstack
commit `b9ddc83`. The source has no reference files. Both SKILL.md files were
compared line by line, and the licence with `cmp`. The five rendered copies
(`.claude-plugin`, `.codex-plugin`, `.claude`, `.agents`, `.opencode`) were compared
with `diff -rq`, and all of them match `skills/bee-figure-it-out/`.

Verdict: no source step is missing. Every step and rule of the source is present,
swapped with a port-notes entry, or kept as a plain sentence with a port-notes
entry. Only one item was dropped: the `disable-model-invocation` frontmatter key,
which the port notes name. `LICENSE.pstack` is identical to the pstack `LICENSE`.
The pstack-only tool check is clean. No model and no subagent type is picked by
hand. The description is 285 characters and says when not to use the skill. There
are no findings.

Labels: **present** means the step is in the new file. **swapped (noted)** means
it is present with a bee tool or term in place of the pstack one, and the port
notes name the swap. **plain sentence (noted)** means a pstack principle citation
became the principle's method as a sentence, because no bee principle skill exists
yet, and the port notes say so. **dropped (noted)** means it is gone and the port
notes name it. **missing** means a step is gone with no reason.

## `figure-it-out/SKILL.md` → `bee-figure-it-out/SKILL.md`

| Source step or rule (source line) | New home (bee-figure-it-out line) | Status |
|---|---|---|
| Frontmatter `name: figure-it-out` (2) | `name: bee-figure-it-out` (2) | swapped (noted: "Name") |
| Frontmatter description (3) | 3-4, rewritten: what it does, when, when not | present |
| Frontmatter `disable-model-invocation: true` (4) | removed | dropped (noted: frontmatter paragraph) |
| When no playbook matches, design one; the workflow is the deliverable; bias toward rigor (9) | 18, "situation" for "playbook" | swapped (noted: "Tool swaps", first bullet) |
| Don't reinvent a playbook; route a focused single-unit task to Bug fix, Perf, Feature, Visual parity, Eval, Multi-phase plan (11) | 20, `bee orient --class <class>` lists the situations | swapped (noted: "Tool swaps", first bullet) |
| A large or cross-cutting version, or work reviewed after stepping away, belongs here (11) | 20 | present |
| Start: todolist, first item read poteto-mode Principles, then add phases (15) | 30, the principles `bee orient --class <class>` lists | swapped (noted: "Tool swaps", second bullet) |
| Phase A: ground first, then commit (19) | 34 | present |
| Phase A: done as a falsifiable predicate, `prove-it-works` (21) | 36 | plain sentence (noted: "Pointers kept as plain sentences") |
| Phase A: scope quantified, blockers raised early (22) | 37 | present |
| Phase A: rigor level biased high; one-way doors and blast radius get more; rigor is gates and artifacts (23) | 38, plus `bee-blast-radius` | present (noted: "Tool swaps", last bullet) |
| Phase A: present framing first; reversible work proceeds (`never-block-on-the-human`); a multi-hour run earns one checkpoint (25) | 40 | plain sentence (noted: "Pointers kept as plain sentences") |
| Phase B: atomic, independently-landable units; riskiest unknown first (29) | 44 | present |
| Phase B: scaffold and verification before features (`foundational-thinking`) (29) | 44 | plain sentence (noted: "Pointers kept as plain sentences") |
| Phase B: verification harness first, baseline from the pre-change state (31) | 46 | present |
| Phase B: one-way-door design runs `architect` (runs `arena`), diverse isolated candidates (32) | 47, `bee-architect`, `bee-arena`, runners via `bee dispatch prepare --kind advisor --role lane-1..3`, `bee blind check` | swapped (noted: "Tool swaps", third bullet) |
| Phase B: read-only judge on a different model family (32) | 47, `--kind reviewer`; model is a team-table setting | swapped (noted: "Tool swaps", third bullet) |
| Phase B: skip for mechanical work; a second arena is over-engineering (`laziness-protocol`) (32) | 47 | plain sentence (noted: "Pointers kept as plain sentences") |
| Phase B: fan out only across genuine seams, own worktree or branch (`separate-before-serializing-shared-state`); don't over-fan (33) | 48, `bee-fanout`, `bee-principle-separate-before-serializing-shared-state` | swapped (noted: "Tool swaps", fourth and sixth bullets) |
| Phase B: write the phase list down; that is what the human reviews (34) | 49, in `plan.md` | present (noted: "Added, not in the source", first bullet) |
| Phase B: add the design's steps as todos between C and D; run under C; weave D rows as steps land (36) | 51 | present |
| Phase C: each unit is an experiment: hypothesis, smallest change, measure, keep or revert (40) | 55 | present |
| Phase C: verify each unit before the next (`sequence-verifiable-units`) (41) | 56 | plain sentence (noted: "Pointers kept as plain sentences") |
| Phase C: inspect the artifact, never a self-report; suspect the observation method (43) | 58 | present |
| Phase C: pair delegated work with a judge; audit delegates; reset a gamed gate; fix a wrong gate in its own change (44) | 59, judge as `bee dispatch prepare --kind reviewer` | swapped (noted: "Tool swaps", fourth bullet) |
| Phase C: VERIFIED, NOT VERIFIED, INCONCLUSIVE; inconclusive is not a pass (45) | 60 | present |
| Phase D: log via `show-me-your-work`, one TSV, row per decision and unit, evidence as links (49) | 64, `bee-show-me-your-work`, plus `bee decisions log` | swapped (noted: "Tool swaps", sixth bullet) |
| Phase D: commit the trail when confidence has to be shown; prefer committed-script evidence; trail plus diff earns trust (49) | 64 | present |
| Phase E: check the whole against the predicate on the real product (53) | 68 | present |
| Phase E: encode recurring corrections as a gate (`encode-lessons-in-structure`) (53) | 68, `bee-principle-encode-lessons-in-structure` | swapped (noted: "Tool swaps", seventh bullet) |
| Reply: playbook, rigor and why, trail path, verified, open (55) | 70-72, under its own heading | present (noted: "Added, not in the source", second bullet) |

Additions with no source counterpart, each named in the port notes ("Added, not in
the source"): the `metadata` block with the `bee-cli` dependency; the "Where each
phase lands" section (22-26); the `bee decisions log` pointer in Phase D; the
Reply heading and the Handoff section (74-76).

## Cross-references checked

Each skill the new file cites exists in this worktree: `bee-architect`,
`bee-arena`, `bee-fanout`, `bee-show-me-your-work`, `bee-blast-radius`,
`bee-shaping`, `bee-planning`, `bee-hive`,
`bee-principle-separate-before-serializing-shared-state`,
`bee-principle-encode-lessons-in-structure`. Each bee verb it cites runs and exits
0 in this worktree: `bee orient --class feature`, `bee blind check --help`,
`bee plan check --help`, `bee decisions log --help`. The dispatch forms match
`skills/bee-architect/SKILL.md:45` and `skills/bee-arena/SKILL.md:37,52`, which use
the same `--kind advisor --role lane-N` runners and the same `--kind reviewer` judge
with no role. None of `prove-it-works`, `never-block-on-the-human`,
`foundational-thinking`, `laziness-protocol` or `sequence-verifiable-units` has a
skill under `skills/` (`ls skills | grep` finds none). That matches the port
notes' reason for keeping them as plain sentences. This is not a defect in ppb-23.

## pstack-only tool names

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-figure-it-out/` finds
nothing (exit 1). The source names none of these either. The pstack-only names it
does use (`poteto-mode`, `architect`, `arena`, `show-me-your-work`, the seven
`principle-*` skills) are all swapped or kept as plain sentences. The only place
they are still named is the port notes.

## Hand-picked model or subagent type

None. `rg -n -i 'model|subagent_type|opus|sonnet|haiku|fable|gpt'
skills/bee-figure-it-out/` matches only SKILL.md line 47, which says "which model
runs a role is a setting of the bee team table, never a choice made here". Every
runner and judge goes through `bee dispatch prepare`, and the payload is used
exactly as it returns.

## Description

285 characters, which fits in 300 (the source's is 341). It ends with "Not for a
task a situation fits", so it says when not to use the skill.

## Trigger fixture

`packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json` names
`bee-figure-it-out` on five lines, which matches the five briefs the port notes
claim.

## Licence

`cmp pstack/LICENSE skills/bee-figure-it-out/LICENSE.pstack` reports no
difference, so the two files are identical.

## Verify

```
$ rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-figure-it-out/ ; test $? -eq 1; echo $?
0
```
