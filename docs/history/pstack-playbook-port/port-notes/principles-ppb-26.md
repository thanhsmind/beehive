# Port notes — principles foundational-thinking, redesign-from-first-principles, outcome-oriented-execution, cell ppb-26

Source: pstack principles `principle-foundational-thinking`,
`principle-redesign-from-first-principles` and
`principle-outcome-oriented-execution` (one `SKILL.md` each, no reference
files) at commit `b9ddc83`, MIT, copyright 2026 Lauren Tan. The notice is
`LICENSE.pstack` in each new skill directory, a verbatim copy of the pstack
`LICENSE`. This file is the only place a pstack-only tool is named; no file
under `skills/` or `expertise/` names one.

Names: `bee-principle-<slug>`, the pstack slug with the prefix that both
distribution pipes discover principle skills by.

Shape: the exact shape of `bee-principle-subtract-before-you-add`: a
`description` opening "Apply when" (pstack opens the first one "Apply
before"; kept, since it names a moment before logic is written), a body, then
Check, Not when, Why and Depth. The source's `disable-model-invocation: true`
is dropped, as in every bee principle skill.

## Tool swaps

None. The three sources name no pstack tool: no loop command, stacked-PR
tool, review bot, subagent or model. Nothing was swapped and no step was
dropped.

## Where each source part went

foundational-thinking, depth in `expertise/architecture.md` §
Foundational Thinking: Data Structures First (a data-structure and
core-type decision is architecture craft):

- "Data structures first" and the late-versus-early cost: kept.
- Structural versus code-level decisions, and "DRY the structure, not every
  line": kept, with the pstack test line folded in.
- Concurrency corollary: kept as a one-line pointer to the existing One
  Writer for Shared State section instead of a second copy.
- Scaffold first, small single-purpose commits, one abstraction per
  increment: kept as bullets in the same section.
- "Subtraction comes before scaffolding": kept, as a pointer to Subtract
  before you add in `expertise/planning.md`.

redesign-from-first-principles, depth in `expertise/architecture.md` §
Redesign From First Principles (it is about the shape of an existing
design): the four source bullets became the four numbered steps, in the
source's order and words; the closing option-value sentence is kept.

outcome-oriented-execution, depth in `expertise/planning.md` §
Outcome-oriented execution (it sequences a migration): the source's Why, the
three core rules and the four guardrails all kept. "Declare where temporary
breakage is acceptable" became a bullet naming the planned phases that may
be red and the step that restores green.

## Added, not in the source

- The Not when line on each skill: the source has no limit. Foundational
  thinking is limited by subtract-before-you-add; redesign by Chesterton's
  fence; outcome-oriented execution by split-on-a-real-dependency and by a
  surface another worktree or client uses mid-migration.
- A situation row in the routing table at the top of each guide.

## Index rows

Three rows added to `## Principle homes` after the last existing row; no
existing row changed. Classes come from the pstack "when it applies" line:

- foundational-thinking: `feature, refactor` — "before writing logic:
  choosing core types and data structures".
- redesign-from-first-principles: `feature, refactor` — "integrating a new
  requirement into an existing design".
- outcome-oriented-execution: `refactor, feature` — "planned rewrites and
  migrations".

## Pointers kept as plain sentences

None. Every skill and principle the new text names exists in `skills/`.

## Regen note

`bee dev regen` also re-rendered unrelated `.claude/skills/` entries
(hub-served skill stubs and the other principle directories) in this
checkout; those were reverted and left out of the commit. Only the three new
directories, the render manifests, the vendored expertise copies and the
release manifest are committed.
