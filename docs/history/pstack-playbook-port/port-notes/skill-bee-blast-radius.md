# Port notes — skill bee-blast-radius, cell ppb-21

Source: pstack skill `blast-radius` (`SKILL.md` only, no reference files) at commit
`b9ddc83`, MIT, copyright 2026 Lauren Tan. The notice is
`skills/bee-blast-radius/LICENSE.pstack`, a verbatim copy of the pstack `LICENSE`.
This file is the only place a pstack-only tool is named; no file under
`skills/bee-blast-radius/` names one.

Name: `bee-blast-radius` (the pstack name with the `bee-` prefix every ported skill
carries).

Kept as the base, with the swaps below: the opening promise and the "grep is not
the job" line, "Don't trust your own writeup", the five-rung "How sure are you"
ladder, the six steps, and the five-part hand-back (what it does, the one fact,
risks, cleared, before you merge).

Frontmatter follows `bee-how`: `name`, a description of 294 characters (what it
does, when to use it, what it is not for) and `metadata` with `ecosystem: bee` and
a `bee-cli` dependency that degrades to the leader reading the sources inline. The
source's `disable-model-invocation: true` is dropped, as in `bee-reflect` and
`bee-recall`: the description names the explicit trigger and its limits.

## Tool swaps

- "Companion to `how` and `why`" became `bee-how` and `bee-why`; the same-rules line
  in step 4 names `bee-why`.
- "Use `why` step 2 to pull the PR and commits" became `bee-why`'s Provenance sweep
  (`bee-researching/references/trace-and-provenance.md`), which reads the decision
  log, git history, feature history and knowledge bundle. The pstack skill's step
  number has no counterpart in bee, so the sweep is cited by name.
- Step 6, "run it as an `arena`. Ask several models the same question and merge the
  answers", became `bee-arena`, with its runners spelled as
  `bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json`,
  then `--role lane-2` and `--role lane-3`, each payload used exactly as returned.
  "Several models" is a team-table setting of the lane roles, never a model name in
  the text; no model or subagent type is named.
- "Write it through `unslop`" became `bee-unslop`.
- The verification posture cites `bee-principle-verify-before-reporting`, the bee
  home of "reproduce a suspected defect before you file it".
- "Solid versus React" in step 3 became "one runtime versus another": a framework
  pair belongs to the source's own project, not to bee.

## Added, not in the source

- Step 1, the knowledge bundle first: `bee knowledge search --text "<area or
  symbol>"` and the matching concepts before any code hunt, since a recorded
  pitfall is a risk someone already paid for.
- Step 3, a bee-specific sentence: the files bee copies into a host project and the
  records it persists, so a changed skill, hook, gate or record shape is checked
  against every project that vendored the old one.
- A `Reply` line (a read-only skill; the proving script is the only file added) and
  a `Handoff` section ("Invoke bee-hive skill"), as `bee-why`, `bee-recall` and
  `bee-tdd` have.

## Pointers kept as plain sentences

None. Every skill and principle the skill names exists in `skills/`.

## Dropped steps

- Cursor `/loop`, Graphite, bugbot and poteto-agent: the source cites none of
  them, so nothing was dropped for them.
- The source's `show-me-your-work` and `architect`: not cited, so nothing to swap.

## Trigger fixture

Five briefs added to `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`:
three expect (one Vietnamese) and two near (expect `bee-how` and `bee-reviewing`),
none naming the slug.
