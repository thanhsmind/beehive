# Port notes — skill bee-arena, cell ppb-14

Source: pstack skill `arena` (`SKILL.md`, no reference files) at commit
`b9ddc83`, MIT, copyright 2026 Lauren Tan. The notice is
`skills/bee-arena/LICENSE.pstack`, a verbatim copy of the pstack `LICENSE`. This
file is the only place a pstack-only tool is named; no file under
`skills/bee-arena/` names one.

Kept as the base, with the swaps below: `SKILL.md` (six phases A to F, the
mandatory rationale, the graft and rejection record, the convergence and
divergence rules, Outputs).

Frontmatter follows `bee-how`: `name`, a description of 289 characters (what it
does, when to use it, what it is not for) and `metadata` with `ecosystem: bee`
and a `bee-cli` dependency that degrades to the leader writing the candidates
itself. The source's `disable-model-invocation: true` is dropped, as in
`bee-architect`: the description names the explicit trigger.

## Tool swaps

- Subagent spawning (Phase B, "spawn all N subagents in one message with
  `run_in_background: true`") became one
  `bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json`
  per runner (`lane-2`, `lane-3`), all issued in the same turn, each returned
  payload run in the background and used exactly as returned. No model or
  subagent type is named.
- "`arena runners` from `~/.cursor/rules/pstack-models.mdc`, else four named
  models" became the `lane-1` to `lane-3` roles of the bee team table. The four
  model names are dropped. A fourth runner reuses a lane role, because the seat
  roles are closed at three lanes. "Same model N times when generation-bound"
  became the fall-through of an unconfigured lane role to `advisor`.
- The cross-judge ("one model from the `arena cross-judge pool`, prefer a
  different family from the parent's, one readonly judge subagent") became
  `bee dispatch prepare --runtime claude --kind reviewer --json` with no
  `--role`. The reviewer kind runs on the review role, a different role than the
  lane runners, so a different model is a team setting. The "different family"
  preference became: when the payload names the runners' model, the synthesis
  note says so.
- The judge sees candidates "by path label"; bee adds "with no lane or model
  names", the blinding `bee-hive/references/gates-and-delegation.md` applies to
  convergence.
- Candidate output paths: an advisor runner is read-only, so "each candidate
  writes to its own location" became "the answer is the artifact plus its
  rationale and the leader saves each answer verbatim at that path". The
  worktree or `/tmp/arena-<slug>/candidate-<n>/` locations stay.
- "Open a todolist" became "open a task list" (the host's own list; no tool named).
- `separate-before-serializing-shared-state` principle skill became
  `bee-principle-separate-before-serializing-shared-state`.
- One addition with no pstack counterpart: Phase B points a design question that
  must end in a dossier at the bee blind-lane procedure
  (`bee-hive/references/gates-and-delegation.md`, "Blind lanes and convergence"),
  so no second design fan-out mechanism exists.
- A `Handoff` section ("Invoke bee-hive skill") is added, as `bee-architect` has.
- The typo "the the" in Phase A step 4 is fixed.

## Pointers kept as plain sentences

The principle does not exist in `skills/` yet, so its method stays as a sentence
and the name is dropped from the shipped text:

- `redesign-from-first-principles` principle (Phase E): "redesigning the base
  from first principles around it as if the graft had been a day-one assumption".
- `prove-it-works` principle (Phase F): "run it, test it or read it against the
  rubric, and report what that showed, never what should happen".
- `laziness-protocol` principle (Phase D tie-break): "do the least that works".

When the slice 4 principles land, a later cell may swap each sentence for the
pointer.

## Dropped steps

None dropped outright. Removed text: the Cursor rules-file path, the four model
names and the Cursor-only `run_in_background` flag, each covered by the swaps
above. Nothing in the source cites the Cursor loop, Graphite, bugbot or the
poteto-agent, so there was none to drop.

## Trigger fixture

Six briefs added to `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`:
four expect (one Vietnamese) and two near (expect `bee-reviewing` and
`bee-architect`), none naming the slug.
