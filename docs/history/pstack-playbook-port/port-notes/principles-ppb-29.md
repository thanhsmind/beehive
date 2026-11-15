# Port notes — principles guard-the-context-window, never-block-on-the-human, exhaust-the-design-space, experience-first, cell ppb-29

Source: pstack principles `principle-guard-the-context-window`,
`principle-never-block-on-the-human`, `principle-exhaust-the-design-space` and
`principle-experience-first` (one `SKILL.md` each, no reference files), MIT,
copyright 2026 Lauren Tan. The notice is `LICENSE.pstack` in each new skill
directory, a verbatim copy of the pstack `LICENSE`. This file is the only place
a pstack-only tool is named; no file under `skills/` or `expertise/` names one.

Names: `bee-principle-<slug>`, the pstack slug with the prefix that both
distribution pipes discover principle skills by.

Shape: the exact shape of `bee-principle-subtract-before-you-add`: a
`description` opening "Apply when", a body, then Check, Not when, Why and
Depth. The source's `disable-model-invocation: true` is dropped, as in every
bee principle skill. Each source description already opened "Apply when"; it
was reworded only where a tool was swapped (the gather) or for tense.

## Tool swaps

- guard-the-context-window: the source routes bulk to "subagents". The swap is
  a read-only gather, spelled
  `bee dispatch prepare --runtime claude --kind gather --json`. No subagent
  type or model is named.
- exhaust-the-design-space: the source's prototypes and "design it twice" map
  to the blind lanes, spelled
  `bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json`
  with `--role lane-2` and `--role lane-3`, the judge as `--kind reviewer`, and
  `bee blind check` for the blind brief. `bee-architect` and `bee-arena` exist
  in `skills/` and are named in the guide section.
- never-block-on-the-human: the source names no pstack tool. The bee boundary
  is added in the Boundaries paragraph: the uat acceptance at `bee close` stays
  the owner's (D4), and a locked decision is never reinterpreted to get
  unblocked.
- experience-first: no tool in the source; nothing swapped.

No step was dropped.

## Where each source part went

guard-the-context-window, depth in `expertise/thinking.md` § Guard the Context
Window (a rule about spending working memory is a way of working, and the
cell's file list names `thinking.md` and `decisions.md` only):

- The opening, the Why and the four pattern bullets (isolate large payloads,
  do not read what you will not use, keep frequently used content inline, size
  phases and cap scope): all kept, in the source's order.

never-block-on-the-human, depth in `expertise/decisions.md` § Never block on
the human (when to ask and when to proceed is a decision-making rule, next to
Locked versus open):

- The opening, the Why, the five pattern bullets and the three boundary
  bullets: all kept. "Code is cheap, attention is scarce" is in the opening.

exhaust-the-design-space, depth in `expertise/thinking.md` § Exhaust the
Design Space:

- The rule, the line that design it twice is this rule by another name, the
  second-flavor clause, the three "applies" cases and the three "does not"
  cases: all kept.

experience-first, depth in `expertise/thinking.md` § Experience First:

- The opening, the five bullets, the user-is-whoever-consumes-the-work
  paragraph and the foundations-serve-the-experience closing: all kept.

## Added, not in the source

- The Not when line on each skill: the source has no limit on guard,
  never-block or experience-first, and exhaust carries its own "when it
  doesn't". guard-the-context-window is limited by the exact-bytes case (a
  summary is not proof, see prove-it-works) and the one-or-two-file lookup;
  never-block-on-the-human by an irreversible action, product direction and a
  locked decision; experience-first by a tradeoff already settled by a locked
  decision or constraint, and a surface the owner called throwaway.
- A situation row in the routing table at the top of `thinking.md` and
  `decisions.md`.
- Cross-references to existing sections and skills: Design it twice, What makes
  a good unit of work and The smallest honest shape in `planning.md`; Prove it
  works in `tests.md`; Foundational Thinking in `architecture.md`; Record at
  the moment of settlement and Reinterpretation is quiet narrowing in
  `decisions.md`; the skills `bee-architect`, `bee-arena` and `bee-fanout`.

## Index rows

Four rows added to `## Principle homes` after the ppb-28 rows; no existing row
changed. Classes come from the pstack "when it applies" line:

- guard-the-context-window: `research, refactor, feature` — large outputs, long
  files and repeated reads are the research and sweep case, and "fan-out
  planning" is feature and refactor work.
- never-block-on-the-human: `feature, bugfix, refactor, perf, spike, docs` —
  "reversible work" is wherever work lands. release is left out because its
  steps are the irreversible ones the principle keeps confirming, and research
  because its deliverable is an answer.
- exhaust-the-design-space: `feature, spike` — "a novel UI interaction or
  architectural decision with no precedent". The source's own exclusions are
  bug fixes and refactors with a clear target state.
- experience-first: `feature, spike` — "product, UX or feature-scope
  tradeoffs", and the prototype-before-committing bullet is the spike case.

## Pointers kept as plain sentences

- experience-first: the source's "prototype in throwaway HTML" stays a plain
  sentence ("throwaway markup"). `bee-design` is not under `skills/` in this
  checkout, so no skill is named for it.

Every other skill or principle the new text names exists in `skills/`.

## Regen note

`bee dev regen` also re-rendered unrelated `.claude/skills/` entries in this
checkout, as in ppb-27 and ppb-28; those stay uncommitted. Only the four new
directories in each tree, the render manifests, the vendored expertise copies,
the onboarding hashes and the release manifest are committed.
