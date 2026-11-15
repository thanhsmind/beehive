# Port notes — skill bee-tdd, cell ppb-18

Source: pstack skill `tdd` (`SKILL.md`, no reference files) at commit
`b9ddc83`, MIT, copyright 2026 Lauren Tan. The notice is
`skills/bee-tdd/LICENSE.pstack`, a verbatim copy of the pstack `LICENSE`. This
file is the only place a pstack-only tool is named; no file under
`skills/bee-tdd/` names one.

Name: `bee-tdd` (the pstack name with the `bee-` prefix every ported skill
carries).

Kept as the base: the opening paragraph, the "do not force a test" paragraph,
the seven workflow steps, "If a Failing Test Is Impractical", the six
guardrails and the three-line "Final Response", each in the pstack wording.

Frontmatter follows `bee-how`: `name`, a description of 285 characters (what it
does, when to use it, what it is not for) and `metadata` with `ecosystem: bee`.
No `bee-cli` dependency: the skill runs no bee command. The source's
`disable-model-invocation: true` is dropped, as in `bee-architect`, `bee-arena`,
`bee-interrogate` and `bee-fanout`: the description names the trigger and its
limits.

## Tool swaps

The source names no pstack tool: it spawns no subagent, names no model, loop,
stacked-PR tool or review bot. So no tool was swapped. Where bee already owns a
rule the source states in its own words, the skill points at it:

- The red-before-green rule (steps 3 to 6) points at
  `bee-principle-red-before-green` for the why, instead of restating it.
- "Encode intended behavior, not mirror the current implementation" cites
  `bee-principle-test-behavior-not-structure`.
- "Run a closest executable check instead" cites `bee-principle-reproduce-first`:
  that check is the red, run before and after the fix.
- The Tester contract paragraph points at `bee-swarming`, where the implementer
  and the tester are two cells. The pstack text assumes one worker holding both
  halves; the skill says it is the method for that worker, outside a cell or in
  a `tiny` lane.

## Added, not in the source

- The sentence naming the Tester contract in `bee-swarming` (above).
- "Write 'green' or 'passing' only beside output from a run you just made" in
  "Final Response", from `bee-principle-red-before-green`.
- A `Handoff` section ("Invoke bee-hive skill"), as `bee-interrogate` and
  `bee-fanout` have.

## Pointers kept as plain sentences

None. Every skill and principle the skill names exists in `skills/`.

## Dropped steps

None. The source has no step that needs a pstack-only tool.

## Trigger fixture

Five briefs added to `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`:
three expect (one Vietnamese) and two near (expect `bee-swarming` and
`bee-capturing`), none naming the slug.
