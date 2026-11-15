# Port notes — skill bee-figure-it-out, cell ppb-23

Source: pstack skill `figure-it-out` (`SKILL.md`, no reference files) at commit
`b9ddc83`, MIT, copyright 2026 Lauren Tan. The notice is
`skills/bee-figure-it-out/LICENSE.pstack`, a verbatim copy of the pstack `LICENSE`.
This file is the only place a pstack-only tool is named; no ported file names one.

Name: `bee-figure-it-out` (the pstack skill is `figure-it-out`).

Kept as the base, with the swaps below: the opening paragraph, the "don't
reinvent a playbook" paragraph, phases A to E and every bullet inside them, and
the reply line.

Frontmatter follows `bee-how`: `name`, a description of 285 characters (what it
does, when to use it, what it is not for) and `metadata` with `ecosystem: bee` and
a `bee-cli` dependency that degrades to the leader running the judge and the
fan-out inline. The source's `disable-model-invocation: true` is dropped, as in
`bee-interrogate` and `bee-comment-review`: the description names the explicit
trigger and its limit.

## Tool swaps

- "Playbook" (the thing it routes to) became "situation", the bee unit under a
  route class; the routing line points at `bee orient --class <class>`, which
  lists the situations.
- The opening todolist item, "read the Principles section of the poteto-mode
  skill", became "read the principles `bee orient --class <class>` lists for the
  task".
- Phase B's `architect` skill and `arena` became `bee-architect` and `bee-arena`.
  The arena runners became `bee dispatch prepare --runtime claude --kind advisor
  --role lane-1 --json` (then `lane-2`, `lane-3`) with `bee blind check` on each
  blind brief; the read-only judge became `--kind reviewer`. The payload runs
  exactly as returned. "Judge on a different model family" became a team-table
  setting, since no skill text picks a model.
- "Pair delegated work with a judge" in Phase C became the same reviewer
  dispatch.
- The fan-out line in Phase B points at `bee-fanout`, the bee name of pstack
  `swarm` (decision `6ca3cd62`).
- `show-me-your-work` became `bee-show-me-your-work`; Phase D also points at
  `bee decisions log` for a rule that settles.
- `principle-separate-before-serializing-shared-state` and
  `principle-encode-lessons-in-structure` became the bee principle skills of the
  same name.
- The "blast radius" mention in the rigor bullet became `bee-blast-radius`.

## Added, not in the source

- "Where each phase lands": the done predicate goes into `CONTEXT.md`
  (`bee-shaping`), the designed phase list into `plan.md` (`bee-planning`, held by
  `bee plan check`), the trail through `bee-show-me-your-work` (cell ppb-23).
- A `Reply` heading and a `Handoff` section ("Invoke bee-hive skill"), as
  `bee-comment-review` and `bee-blast-radius` have.

## Pointers kept as plain sentences

These pstack principles have no bee skill in `skills/` yet, so each method is a
plain sentence and becomes a citation when its principle lands.

Restored (ppb-37): all five principles exist now, and each sentence carries its
pointer in `skills/bee-figure-it-out/SKILL.md`: `bee-principle-prove-it-works`
and `bee-principle-never-block-on-the-human` (Phase A),
`bee-principle-foundational-thinking` (Phase B), `bee-principle-laziness-protocol`
and `bee-principle-sequence-verifiable-units` (Phase C). The laziness pointer sits
on Phase C's "make the smallest change" clause, the only place the shipped text
holds that rule, and not on a Phase B sentence.

- `principle-prove-it-works`: "prove it works, on the real artifact, and 'done
  well' has to be checkable" (Phase A).
- `principle-never-block-on-the-human`: "Reversible work proceeds without waiting
  on the human" (Phase A).
- `principle-foundational-thinking`: "build the foundation the rest stands on
  first" (Phase B).
- `principle-laziness-protocol`: "do the least that settles it" (Phase B).
- `principle-sequence-verifiable-units`: "Verify each unit before starting the
  next instead of batching checks at the end" (Phase C).

## Dropped steps

- Cursor `/loop`, Graphite, bugbot and poteto-agent: the source cites none of
  them, so nothing was dropped for them.
- The source's "todolist" wording is kept as plain words; no tool is named.

## Trigger fixture

Five briefs added to `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`:
three expect (one Vietnamese) and two near (expect `bee-planning` and
`bee-wayfinding`), none naming the slug.
