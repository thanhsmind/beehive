# Port notes — pstack technical-writing merged into bee-technical-writing, cell ppb-35

Source: pstack skill `technical-writing` (`SKILL.md`), MIT, copyright 2026 Lauren Tan.
`skills/bee-technical-writing/` got `LICENSE.pstack`, a verbatim copy of the pstack
`LICENSE`. This file is the only place a pstack-only tool is named.

Rule applied (D5): the bee skill stays bee's. Every bee rule is kept, including the
project-standard precedence line, the "never rewrite quoted evidence" line, the PR
body and commit-message format, and the Headless section, none of which pstack has.
The bee-technical-writing description is unchanged, under the 300 budget.

## What came in

- Diátaxis: "Reach for it whenever you feel unsure what you are writing. Gut feel
  is often wrong here." after the compass sentence.
- Google style: "Read the awkward sentence aloud. If it stays awkward, rewrite it."
- Worked example: the "fixes, by layer" paragraph that maps each edit to the layer
  that caused it.

## Left out because bee already has it

- The three top rules, Vary the rhythm, the four layers and the review checklist:
  identical in bee.
- Reference-mode wording ("Be dry, complete, and sure: state facts..."): bee has the
  same text with one more full stop.
- The "don't churn" sentence in Global English: bee has it, joined with a full stop
  in place of the colon.

## Left out because bee differs

- pstack's "Add new offenders to `unslop`'s abstract-metaphor rule": bee keeps its
  own line, which proposes the addition in the reply with a diff and never edits
  `bee-unslop`.
- pstack's one-line PR and commit rule: bee's longer format (title, six sections,
  linked artifacts) already covers it.
- pstack's frontmatter (`disable-model-invocation`, its description): bee routes by
  its own description.

## Tool swaps

- None. The pstack technical-writing text names no pstack-only tool.

## Not ported

- No pstack step was dropped for lack of a bee counterpart.
- No bee skill or principle was missing for this merge.

## Regen note

`bee dev regen` also re-renders unrelated `.claude/skills/` entries in this
checkout, as in ppb-30 to ppb-34; those stay uncommitted. Only the
`bee-technical-writing` directory in each rendered tree, the render manifests, the
onboarding hash and the release manifest are committed.
