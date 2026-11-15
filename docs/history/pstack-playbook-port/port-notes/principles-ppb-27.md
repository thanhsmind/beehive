# Port notes — principles laziness-protocol, minimize-reader-load, model-the-domain, cell ppb-27

Source: pstack principles `principle-laziness-protocol`,
`principle-minimize-reader-load` and `principle-model-the-domain` (one
`SKILL.md` each, no reference files), MIT, copyright 2026 Lauren Tan. The
notice is `LICENSE.pstack` in each new skill directory, a verbatim copy of the
pstack `LICENSE`. This file is the only place a pstack-only tool is named; no
file under `skills/` or `expertise/` names one.

Names: `bee-principle-<slug>`, the pstack slug with the prefix that both
distribution pipes discover principle skills by.

Shape: the exact shape of `bee-principle-subtract-before-you-add`: a
`description` opening "Apply when", a body, then Check, Not when, Why and
Depth. The source's `disable-model-invocation: true` is dropped, as in every
bee principle skill.

## Tool swaps

None. The three sources name no pstack tool: no loop command, stacked-PR
tool, review bot, subagent or model. Nothing was swapped and no step was
dropped, and no dispatch spelling was needed in any of the three.

## Where each source part went

laziness-protocol, depth in `expertise/architecture.md` § Laziness Protocol
(it is about how much structure a design carries):

- The opening (writing is cheap, so borrow a maintainer's fatigue) and the
  prime directive: kept, the directive as the closing test line.
- The six rules (prefer deletion, flat call hierarchy, consolidate decisions,
  minimize the diff, question the threading, sweat the small leaks): all kept,
  in the source's order and numbers (the three-layer bound stays). Prefer
  deletion points at Subtract before you add in `expertise/planning.md`;
  consolidate decisions points at Single Source of Truth in the same guide.

minimize-reader-load, depth in `expertise/review.md` § Reader load (the source
says "when reviewing or shaping code", and the guide is where a reviewer reads
a diff for what it costs the next reader):

- The two axes, layers to trace and state to hold, and the Why: kept.
- The pattern bullets: collapse layers, make adjacent layers change the
  abstraction, demand interface compression, shrink state scope, name the
  invariant at the boundary, and the add-a-layer question: all kept as the
  finding list in the section.
- The thirty-second test: kept in the skill and the section.

model-the-domain, depth in `expertise/architecture.md` § Model the Domain (a
choice of structure is architecture craft):

- The Why, all eight structure bullets with the open-ended last one, "do not
  force an abstraction", and the two tells with the temporal-decomposition
  tell: all kept.

## Added, not in the source

- The Not when line on each skill: the source has no limit. Laziness is
  limited by Chesterton's fence and by the YAGNI boundary that it trims
  structure and never tests or error handling; reader load by the deletion
  test (a layer that hides a real decision or has a second implementation is
  the point); model-the-domain by subtract-before-you-add (a structure that
  removes no branch only adds indirection).
- A situation row in the routing table at the top of each guide section's
  file.
- In the model-the-domain section, two cross-references to existing sections:
  Make Illegal States Unrepresentable and The Rule of Three.

## Index rows

Three rows added to `## Principle homes` after the ppb-26 rows; no existing
row changed. Classes come from the pstack "when it applies" line:

- laziness-protocol: `refactor, feature, bugfix` — "refactoring, evaluating
  diff size, or tempted to add abstractions, layers, or signal threading";
  bugfix is added because diff size is judged on a fix too.
- minimize-reader-load: `refactor, feature` — "reviewing or shaping code
  that's hard to trace".
- model-the-domain: `feature, refactor` — "writing stateful logic, or code
  that branches a lot".

## Pointers kept as plain sentences

minimize-reader-load links `principle-guard-the-context-window`, which has no
bee skill yet. The link is dropped and the method kept as a plain sentence in
the skill and in the review section: working memory is finite for a human
reader and for an agent's context window alike. When that principle is ported, the sentence can point at it. Still open after
ppb-37: `skills/bee-principle-minimize-reader-load/` is outside that cell's
files, so the link back to `bee-principle-guard-the-context-window` is listed in
`principle-pointers.md` for a cell that owns that skill. Every other skill or principle the new
text names exists in `skills/`.

## Regen note

`bee dev regen` also re-rendered unrelated `.claude/skills/` entries
(hub-served skill stubs and the other principle directories) in this
checkout. The write guard refused `git checkout` and `git restore` with a
second worker live, so they could not be reverted and were left uncommitted.
Only the three new directories in each tree, the render manifests, the
vendored expertise copies, the onboarding hashes and the release manifest are
committed.
