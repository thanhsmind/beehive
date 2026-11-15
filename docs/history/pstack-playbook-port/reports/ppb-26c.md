# ppb-26c: check of the principles ported in ppb-26

Checked against commit `06e570157` (ppb-26). The sources are the pstack
`principle-foundational-thinking`, `principle-redesign-from-first-principles`
and `principle-outcome-oriented-execution` `SKILL.md` files (each directory holds
only that one file). No file other than this report was edited.

## Verdict

Every source rule of the three principles is present in bee. None is left out
and none is missing. No pstack-only tool name appears in the new skills or in
their expertise sections, and no bee rule was removed. Two lines were added that
the source does not have, and the port notes record both: the "Not when" limit
on each skill, and the routing-table rows. The foundational-thinking description
opens "Apply before" where the plan's shape says "Apply when"; the port notes
record that choice too.

"Present" means the rule is in the bee skill body, its depth section, or both.
Depth sections: `expertise/architecture.md` § Foundational Thinking: Data
Structures First, § Redesign From First Principles; `expertise/planning.md` §
Outcome-oriented execution. The vendored copies under `.bee/expertise/` match
these sections byte for byte.

## foundational-thinking

<!-- bee:not-a-deferral: a record or description of work already done, never a promise to act -->
| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Structural decisions protect option value | present | depth, "Two kinds of decision" paragraph |
| 2 | Code-level decisions protect simplicity | present | depth, same paragraph |
| 3 | Over-engineering is often a premature decision that closes doors; the right foundation keeps doors open | present | depth, same paragraph; skill Why line |
| 4 | Data structures first: get the shape right before logic, so downstream code is obvious | present | skill body, first paragraph; depth opening |
| 5 | Define core types early, trace every access pattern, match the dominant paths | present | skill body; depth opening; skill Check line |
| 6 | A late data-structure change is a rewrite, an early one often a one-line diff | present | skill body; depth opening |
| 7 | DRY the structure, not every line; types and data models converge | present | skill body ("converge the structure, not every line"); depth |
| 8 | Three similar statements beat a premature abstraction | present | skill body; depth |
| 9 | Prefer explicit over clever | present | skill body; depth |
| 10 | Test behavior and edge cases, not line counts | present | depth only ("Test behavior and edge cases, not line counts") |
| 11 | Concurrency corollary: ask what happens if another actor modifies shared state; isolate if not "nothing" | present | skill body; depth bullet, pointing at One Writer for Shared State (`expertise/architecture.md:210`) |
| 12 | Scaffold first: ask whether every later phase benefits; CI, linting, test infra, shared types | present | skill body; depth bullet |
| 13 | Sequence for option value: setup before features, tests before fixes | present | depth bullet (skill body says "do it first") |
| 14 | Small single-purpose commits | present | skill body; depth bullet |
| 15 | Each increment lands a coherent abstraction or deepens one; no capability spread across callers as special-case coordination | present | depth bullet "One abstraction per increment" |
| 16 | Subtraction comes before scaffolding | present | skill body ("after removing dead weight"); depth bullet pointing at Subtract before you add (`expertise/planning.md:271`) |
<!-- /bee:not-a-deferral -->

## redesign-from-first-principles

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Do not bolt a change onto the existing design | present | skill body; depth opening |
| 2 | Redesign as if the requirement were there from the start; result looks like the day-one build | present | skill body; depth opening; description unchanged from source |
| 3 | Read all affected files, understand the design holistically | present | skill body; depth step 1 |
| 4 | Ask what you would build from scratch with the new requirement | present | skill body; depth step 2; skill Check line |
| 5 | Propagate through every reference: types, docs, examples, rationale sections | present | skill body; depth step 3 |
| 6 | Think holistically, deliver incrementally | present | skill body; depth step 4 |
| 7 | This is the method for preserving option value | present | depth closing sentence |

## outcome-oriented-execution

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Optimize for the verifiable end state over smooth intermediate states | present | skill body; depth opening |
| 2 | Why: keeping each step stable creates compatibility code that becomes long-lived debt | present | skill body; depth opening; skill Why line |
| 3 | Converge on the target architecture; prove at explicit verification boundaries | present | skill body; depth opening |
| 4 | Core: end-state integrity over transitional stability | present | depth bullet 1 |
| 5 | Core: intermediate breakage acceptable when planned, scoped, reversible | present | skill body; depth bullet 1 |
| 6 | Core: always run final verification before declaring done | present | skill body; depth bullet 4 |
| 7 | Guardrail: use for planned rewrites and migrations with explicit phase boundaries | present | description; depth opening and closing paragraph |
| 8 | Guardrail: declare where temporary breakage is acceptable | present | skill body; depth bullet 2 (adds "which step must be green again") |
| 9 | Guardrail: keep high-signal checks for actively touched areas | present | skill body; depth bullet 3 |
| 10 | Guardrail: full static and runtime verification at plan completion | present | skill body; depth bullet 4 |

## Added, not in the source

- Each skill's "Not when" line: foundational-thinking points at
  subtract-before-you-add, redesign at Chesterton's fence, outcome-oriented
  execution at split-on-a-real-dependency. All three skills exist in `skills/`.
- outcome-oriented-execution adds a limit the source lacks: it never covers a
  user-facing surface another worktree or client depends on mid-migration
  (skill Not when line; closing paragraph of the depth section). This narrows
  the source rule rather than contradicting it. The port notes list it under
  "Added, not in the source".
- One routing-table row per principle in the two guides' situation tables.

## pstack-only tool names

None. `rg -i "cursor|/loop|graphite|bugbot|poteto|subagent_type|claude-|opus|sonnet|arena"`
over the three skill directories returns no match (exit 1). The same terms
over the three depth sections return nothing. The sources name no tool either,
so there was nothing to swap (D2).

## Bee rules removed

None. Commit `06e570157` deletes lines in only three files, and none of them
removes a rule:

- `.bee/onboarding.json`: 3 hash values replaced by the new guide hashes.
- `docs/history/codex-harness-hardening/release-manifest.json`: 6 `sha256`
  values replaced.
- `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`:
  the `anchors:` line replaced by a copy that keeps every earlier entry and
  adds the three new `SKILL.md` paths.

The two expertise guides and the skill files only gained lines.

## Other checks

- `LICENSE.pstack` in each of the three directories is byte-identical to the
  pstack `LICENSE` (`cmp`, no output).
- `principle_index_parity`: 5 tests pass (one row per principle skill, each
  row a skill, a spoken line before applied_at, known classes only, an existing
  guide heading, both plugin trees).
