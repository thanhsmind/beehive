# ppb-14c — bee-arena checked against its pstack source

Scope: what ppb-14 wrote in commit `1a72a53c5`. That is `skills/bee-arena/`
(`SKILL.md`, `LICENSE.pstack`) and the port notes
`docs/history/pstack-playbook-port/port-notes/skill-bee-arena.md`. Each is
checked against the pstack skill `pstack/skills/arena/SKILL.md`, which has no
reference files.

Verdict: no source step is missing. All six phases and every rule inside them
are present, in bee's terms where a tool was swapped. Every removed text (the
Cursor rules-file path, the eight model-name mentions, `run_in_background`,
`disable-model-invocation`) has a port-notes entry. The pstack-only tool fence
is clean. No model and no subagent type is picked by hand. The description is
289 characters and names what the skill is not for. `LICENSE.pstack` is
byte-identical to the pstack `LICENSE`. There are three findings. None blocks
the cap.

Labels: **present** means the step is in the new file, in bee's terms where a
tool was swapped. **dropped (noted)** means it is gone and the port notes name
it. **dropped (not noted)** means it is gone and the notes say nothing, which
D2 (`063a21e4`) counts as missing. **missing** means a method step is gone with
no reason.

## `SKILL.md`

| Source step or rule (source line) | New home | Status |
|---|---|---|
| Frontmatter `name: arena` (2) | `name: bee-arena` | present |
| Frontmatter description (3) | line 4, rewritten with a "Not for" clause | present |
| Frontmatter `disable-model-invocation: true` (4) | removed | dropped (noted: frontmatter paragraph) |
| Intro: fan out, read every candidate, pick base, graft, verify (9) | line 18 | present |
| Start: open a todolist, one entry per phase, why (13) | line 22, "task list" | present (swap noted) |
| Start: six phases listed (15-20) | lines 24-29 | present |
| A: the prompt is the contract; get it right before spawning (24) | line 33, "dispatching" | present |
| A1: state the artifact (26) | line 35 | present |
| A2: derive a 3-6 criterion concrete rubric; candidates see only the task (27) | line 36 | present |
| A3: runners from `arena runners` in the Cursor rules file (28) | line 37, `lane-1` to `lane-3` team roles | present (swap noted, bullet 2) |
| A3: default to four named models (28) | removed | dropped (noted: bullet 2, "four model names are dropped") |
| A3: more runners for multiple design directions (28) | line 37, a fourth runner reuses a lane role | present (noted) |
| A3: same model N times when generation-bound (28) | line 37, unconfigured lane falls through to `advisor` | present (noted) |
| A4: own output path per candidate, worktree or `/tmp/arena-<slug>/candidate-<n>/` (29) | line 38 | present |
| A4: shared path fails `separate-before-serializing-shared-state` (29) | line 38, `bee-principle-separate-before-serializing-shared-state` | present (noted) |
| B: spawn all N in one message, `run_in_background: true` (33) | line 42, all `bee dispatch prepare` calls in one turn, run in background | present (swap noted, bullet 1) |
| B: each gets task, grounding path, output path, artifact plus rationale (33) | line 42, plus "same brief bytes" | present |
| B: rationale mandatory, names alternatives and rejections (35) | line 44 | present |
| B: on dropout proceed with N-1 and note it (37) | line 46 | present |
| C: only after all candidates complete (41) | line 52 | present |
| C: judge model from `arena cross-judge pool` or four named models (41) | line 52, `--kind reviewer`, review role | present (swap noted, bullet 3); model names dropped (noted) |
| C: prefer a different model family from the parent's (41) | line 52, synthesis note says so when the payload names the runners' model | present (swap noted, bullet 3) |
| C: one readonly judge (41) | lines 52, 54 | present |
| C: judge sees rubric and candidates by path label, scores, recommends base (41) | line 54, adds "with no lane or model names" | present (addition noted, bullet 4) |
| C: runs in parallel with the parent's reading, not with candidates; why (41) | line 54 | present |
| D: read every candidate end to end; why (45) | line 58 | present |
| D: score per criterion; compare with judge; disagreement means bias or ambiguity; read both rationales (47) | line 60 | present |
| D: pick for extensibility; tie goes to cleaner boundary per Laziness Protocol (49) | line 62, "do the least that works" | present (pointer as sentence, noted) |
| D: record pick, reason and judge verdict in a synthesis note (51) | line 64 | present |
| E: walk each loser; one or two things per candidate (55) | line 68 | present |
| E: fold by hand per `redesign-from-first-principles`; no mechanical paste; one mental model (57) | line 70, method kept as a sentence | present (pointer as sentence, noted) |
| E: record grafts, sources, rejections and why; rejections highest signal (59) | line 72 | present |
| E: convergence means ship consensus, no graft (61) | line 74 | present |
| E: wild divergence means reframe and re-run, never average (61) | line 74 | present |
| F: same scrutiny per `prove-it-works`; no pass (65) | line 78, method kept as a sentence | present (pointer as sentence, noted) |
| F: a missed problem means re-frame (A) or missed graft (E); don't paper over (67) | line 80 | present |
| Outputs: one artifact; synthesis note with base, grafts, rejections, dropouts, verification (71) | line 84 | present |

Additions with no source counterpart: the `metadata` block with the `bee-cli`
dependency (noted, frontmatter paragraph), the advisor-writes-no-file sentence
in A4 (noted, bullet 5), the blind-lane redirect on line 48 (noted, bullet 8),
the "with no lane or model names" blinding (noted, bullet 4), the
"same brief bytes" sentence on line 42 (not noted, finding 3) and the Handoff
section (noted, bullet 9). The "the the" typo fix is noted.

## Pstack-only tool names

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-arena/` prints
nothing and exits 1. A wider search under `skills/bee-arena/` for
`subagent_type`, `model:`, `todolist`, `mdc`, `run_in_background` and the
source's model families (`fable`, `gpt-`, `grok`, `opus`, plus `sonnet`,
`haiku`) also prints nothing. The only pstack names left are in the port notes.

## Hand-picked model or subagent_type

None. Line 37 tells the leader to use the `bee dispatch prepare` payload as
returned and "never pick a model or a subagent type by hand"; per-runner models
come from the `lane-1` to `lane-3` roles. The judge goes through
`--kind reviewer`, which `prepare.rs` maps to the `review` role. Both claims
the skill makes about bee were checked against the code and the procedure:
`lane-1` to `lane-3` are in `SEAT_ROLES`
(`packages/bee-rs/crates/bee/src/verbs/drivers/models.rs`), and an unconfigured
seat falls through to `advisor` on an advisor-kind dispatch
(`skills/bee-hive/references/gates-and-delegation.md`, "Blind lanes and
convergence"). The one principle named,
`bee-principle-separate-before-serializing-shared-state`, exists under
`skills/`; the three unported principles are not named.

## Description

289 characters (measured on the folded scalar after `description: >-`), under
the 300 cap. It says what the skill does, when to use it ("/bee-arena, says
arena this, or when one attempt at a non-trivial artifact would lock in the
wrong shape") and when not to ("Not for design dossiers (blind lanes) or review
waves").

## Findings

1. **`--runtime claude` is fixed.** Lines 37 and 52 write `--runtime claude`.
   The delegation door writes `--runtime <rt>`, and the skill renders into the
   codex and opencode trees too, where `claude` is the wrong runtime. This is a
   runtime, not a model, so it does not break the no-hand-picked-model rule.
   Same finding as ppb-13c finding 2. Minor.
2. **No `--brief-file` on the runner command.** Line 42 promises every
   candidate "the same brief bytes", but line 37's command carries no
   `--brief-file <path>`, the flag that records the brief and pins its digest.
   An arena is not a blind-lane run, so `bee blind check` does not depend on it
   here, and nothing breaks; the promise is just not recorded. Fix if wanted:
   add `--brief-file <path>` to line 37. Minor.
3. **One addition is not named in the port notes.** "Every candidate gets the
   same brief bytes" (line 42) comes from the blind-lane procedure and has no
   pstack counterpart; the notes do not list it. Nothing is lost. Minor.

## Proof

- `rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-arena/ ; test $? -eq 1`
  exits 0 (rg exit 1, no match). Static.
- `diff skills/bee-arena/LICENSE.pstack <pstack>/LICENSE` prints nothing.
