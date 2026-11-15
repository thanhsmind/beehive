# Port notes — pstack unslop merged into bee-unslop, cell ppb-34

Source: pstack skill `unslop` (`SKILL.md`), MIT, copyright 2026 Lauren Tan.
`skills/bee-unslop/` got `LICENSE.pstack`, a verbatim copy of the pstack
`LICENSE`. This file is the only place a pstack-only tool is named.

Rule applied (D5): the bee skill stays bee's. Every bee rule is kept, including
rules 32 and 33 and the Scope and Headless sections, which pstack lacks. The
bee-unslop description is unchanged, under the 300 budget.

## What came in

- Rules 1 (puffery), 2 (name-dropping), 4 (promotional language), 6 (formulaic
  challenges) and 21 (cutoff disclaimers), under their original numbers. The
  bee skill had left gaps there.
- Rule 13: the sentence that parentheses in place of an em dash trade one tell
  for another.
- Process step 3, "Add voice", with the Adding voice section: the closing
  Self-audit becomes step 4. Three checks came in as the pstack method:
  acknowledge complexity, use "I" when it fits, let some mess in.

## Left out because bee already has it

- "Have opinions", "Vary rhythm" and "Be specific" from pstack's Adding soul:
  `bee-technical-writing` § Vary the rhythm homes all three, with the mode limit
  on opinion. The new section cites it and restates none of them.
- Rules 3, 5, 7 to 12, 14 to 20, 22 to 31: already identical in bee.

## Left out because bee differs

- pstack's description line "Must always apply": bee routes by its description
  and the Communication contract, and the Scope section limits what the rules
  touch.
- Unscoped "Let some mess in": bee bounds it by Scope, so verbatim lines are
  never loosened.

## Tool swaps

- None. The pstack unslop text names no pstack-only tool.

## Not ported

- No pstack step was dropped for lack of a bee counterpart.
- No bee skill or principle was missing for this merge.

## Regen note

`bee dev regen` also re-renders unrelated `.claude/skills/` entries in this
checkout, as in ppb-30 to ppb-33; those stay uncommitted. Only the `bee-unslop`
directory in each rendered tree, the render manifests, the onboarding hash and
the release manifest are committed.
