---
name: bee-design
description: "Use when shaping, building, critiquing or auditing rendered UI, then checking it with the ease-design ui gate."
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    node:
      kind: command
      command: npx
      missing_effect: degraded
      reason: The Check step runs `bee design gate`, which calls the ease-design ui gate through npx and needs Node 20 or newer. Without it the skill still designs from its references and says plainly that the check did not run.
---

# bee-design — design taste for rendered UI

This skill carries the design-os taste (ease-design 0.6.1) inside bee. One workflow: Concept → Build →
Check → Hand over → Adjust. The taste is in `references/`; this file routes you to it. The check is
`bee design gate`, the ease-design `ui gate` held to a baseline; this skill never grades taste by script.

## Packs

A pack adds the taste for one project type. A host has only the packs it picked. Load a pack's
`SKILL.md` when the work is of its type; when that pack is not installed, say so and work from this core.

| Pack | Project type it serves |
| --- | --- |
| `bee-design-web` | Web pages and web apps: the web technique router, screen-pattern cards, signature devices, `DESIGN.md`, owner component kits |
| `bee-design-charts` | Charts, diagrams and multi-screen flows |
| `bee-design-motion` | Animation, scroll cinema, WebGL and gradient effects, generated media assets |
| `bee-design-apple` | Native SwiftUI on iOS, iPadOS and macOS |
| `bee-design-figma` | Figma canvas work and reading a design system out of Figma |

## How the references name things

- `references/<file>` is a file in the same skill.
- `bee-design-<pack>/references/<file>` is a file in that pack, a sibling skill of this one. It is absent
  when the pack is not installed.
- `design-os:<path>` is a file in the upstream design-os knowledge folder that bee does not carry
  (upstream tooling, governance or research inputs). Read it at https://github.com/jangtrinh/design-os.
- `ui <command>` is the ease-design CLI. Run it as `npx --yes ease-design@0.6.1 <command>`, and only with
  the commands and flags in [contract.md](references/contract.md). The gate is the exception: run it as
  `bee design gate` (step 3), never through `npx`.

## 1. Concept

Read [concepts.md](references/concepts.md) and write the one-sentence concept (surface, primary action,
shape, mode, data, states, widths). Pick a verb from its table and read only that reference, end to end.
Read the numbered [rules.md](references/rules.md) before writing markup. [contract.md](references/contract.md)
lists every `ui` command and flag this skill uses; call nothing that is not there.

| Need | Verb | Read |
| --- | --- | --- |
| Fuzzy brief, structure first | shape | [page-structures.md](references/page-structures.md) |
| A design or mock is supplied | implement | [build-loop.md](references/build-loop.md) |
| Review or score | critique | [taste-rubric.md](references/taste-rubric.md) |
| Final pass before shipping | polish | [design-tells.md](references/design-tells.md) |
| Empty, error, loading, long data | harden | [content-design.md](references/content-design.md) |
| Accessibility, layout, responsive | audit | `ui a11y-lint`, `ui validate-layout` |

Read [generation-craft-defaults.md](references/generation-craft-defaults.md) before writing markup for
shape and implement work.

When NOT to use: native macOS/iOS/iPadOS (`bee-design-apple`), Figma canvas (`bee-design-figma`), chart or
diagram work (`bee-design-charts`) — use their pack, then check the render here — and changes that alter
no rendered output.

## 2. Build

Write the page in its own folder, with every asset next to it; the whole folder is the evidence unit.
Sample data lives in one place and is labelled. Work from your project directory.

## 3. Check

Run the gate from the project directory, and put the same command in the cell's verify:

```sh
bee design gate
```

It runs `ui gate` on every page (the project's `design.pages` globs, or every `*.html`) and compares
each page with the baseline in `.bee/design/gate-baseline.json`. Exit 0 means no page gained errors or
lost design-token coverage, and every page with no baseline has 0 errors. Exit 1 names each page that
regressed, with its numbers. Fix the page, never the baseline: recording a baseline
(`bee design gate --write`) is a person's act, and bee refuses it while any cell is claimed. Warnings
and advisories stay visible but do not reject.

The gate needs Node 20 or newer. On a machine without it, `bee design gate` prints
`design gate: not run — Node 20 or newer is missing` and exits 0. Keep designing from the references,
and say plainly in the hand-over and in the cell report that the gate did not run and why: the proof
reason must then carry `design gate not run`. Never report an unrun gate as passed.

bee does not carry upstream's `build.mjs`, `validate.mjs` or `look.mjs`. In [rules.md](references/rules.md),
read "validate" as the gate above. Rules 2 and 3 bind upstream's build evidence folder and do not apply.
The look items `L1`-`L5` are not measured: when a browser is available, open the page at 375, 768 and
1440 px and check them by eye; otherwise list them under NOT_VERIFIED. Judge `L6`-`L10` yourself, or give
a fresh-context critic the rubric and the page. Machine checks do not grade clarity, taste or
conformance to WCAG.

## 4. Hand over

Use these fields, keeping failures and missing checks visible:

```text
EVIDENCE: <absolute page folder>
GATE: <the gate command, its exit and errorCount, or NOT RUN and the reason>
LOOK: <what you saw at 375, 768 and 1440 px for L1-L5, or NOT CHECKED>
EYE_ITEMS: <L6-L10: what you saw, per width, or NOT CHECKED>
CRITIC: <which separate subagent or session scored it, or NOT CHECKED>
NOT_VERIFIED: <screen readers, real devices, states not built, anything skipped>
```

Never write "should work" or "looks right".

## 5. Adjust

Fix from the number, keeping the structure; shorten copy before changing structure, and log any
structural change with the measurement that forced it. Rerun the gate after the last edit. At most two
fix rounds after the first full inspection, then hand over what remains.

| What goes wrong | Do this instead |
| --- | --- |
| `gate` finding | Read the finding, fix at its source; add `--tokens` for a project; rule 6 |
| `L1` or `L2` fails | Fix the layout, not the measurement; rules 7, 8 |
| `L3` names a blocked request or missing file | Copy the asset into the page folder; rules 5, 9 |
| `L4` or `L5` fails | Fix the asset path or the blank render; rule 9 |
| Same page for every brief | Name a different macrostructure first; rule 10 |
| Numbers or logos you cannot source | Mark as to confirm or cut; rule 11 |
| Data view with one state | Build empty, loading, error; rule 12 |
| Looks like a template | Change shape and focal placement, not colour; rule 13 |
| You scored your own page | Hand the page to a fresh context; rule 14 |
| No browser | List `L1`-`L5` under NOT_VERIFIED and say so |

## Roles on your runtime

Three roles, always filled by the runtime you are in; no model or vendor is named by this skill. The
critic is never the builder. A runtime with no second model uses a fresh subagent or session of itself.

| Runtime | Builder | Critic (fresh context) | Scout (read-only) |
| --- | --- | --- | --- |
| Claude Code | the main session | a subagent, named in the prompt | a subagent limited to read tools |
| Codex | the main session | a subagent you ask for explicitly | the `explorer` role |
| Gemini CLI | the main session | a custom subagent, `@name` | the `codebase_investigator` subagent |
| Antigravity | the main agent | a subagent the parent invokes | a subagent limited to read tools |
| Anything else | your session | a fresh session of your runtime | the same |

[rules.md](references/rules.md) rule 14 holds the critic's duties.

## More taste

| Need | Read |
| --- | --- |
| What static checks can and cannot prove about accessibility; never claim compliance | [accessibility.md](references/accessibility.md) |
| WCAG 2.2 rules design-os does not carry: focus not obscured, drag alternative, login paste, carousel pause, scrim contrast, error summary | [additions/wcag-2-2.md](additions/wcag-2-2.md) |
| OKLCH reasoning, WCAG contrast targets, 11-stop scales, semantic role mapping | [color-science.md](references/color-science.md) |
| The DTCG token model: primitive and semantic tiers, aliasing, role/foreground pairs | [token-taxonomy.md](references/token-taxonomy.md) |
| The eight UI-mode constraint sets and the universal hard style guide | [mode-constraints.md](references/mode-constraints.md) |
| UX laws, Gestalt perception, cognitive biases, trust, cognitive load | [ux-psychology.md](references/ux-psychology.md) |
| 32 reusable components across 8 categories, with a generation spec for each | [component-catalog.md](references/component-catalog.md) |
| Reuse what the design system declares before inventing | [design-reuse-ladder.md](references/design-reuse-ladder.md) |
| The design-system guidelines pack: ruling tiers, slots, the seal | [ds-guidelines.md](references/ds-guidelines.md) |
| The declared design stance: Never/Always/Voice and its precedence | [design-soul.md](references/design-soul.md) |
| Turning a stated need into the right design move; the three mandatory asks | [need-routing.md](references/need-routing.md) |
| Weak prompt to evidence-qualified delivery: typed brief, contract, curator | [qualified-delivery.md](references/qualified-delivery.md) |
| Grounding design in real user evidence; the anti-fabrication gate | [user-evidence.md](references/user-evidence.md) |
| Git-native design review and engineering handoff | [design-review.md](references/design-review.md) |
| What a visual regression gate catches that a code diff cannot | [visual-regression.md](references/visual-regression.md) |
| Why a false pass costs more than a crash; the four honesty rules | [verification-honesty.md](references/verification-honesty.md) |
| Domain vocabulary for lakehouse data platforms | [domain-packs/lakehouse.md](references/domain-packs/lakehouse.md) |

License: the design-os taste in this skill is PolyForm Shield 1.0.0 — [LICENSE.md](LICENSE.md) and [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) in this skill; upstream https://github.com/jangtrinh/design-os.
